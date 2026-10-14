# Post-Migration Data Integrity Playbook (v_prev ↔ v_new, SQL)

Reusable battery of **raw-SQL** checks to prove that an Odoo database survived a
version upgrade intact, run **table-by-table over both Postgres databases**
(origin `v_prev` and migrated `v_new`). This is the low-level, full-DB
counterpart of `references/migration-accounting-checks.md`:

| | `migration-accounting-checks.md` | this playbook |
|---|---|---|
| Layer | live API (odoo-pilot) | raw SQL (`psql`) on both DBs |
| Scope | accounting + master smoke tests | **every common table** + financial + relational + analytic/security/translations |
| Path | any migration (upgrade, rebuild, split) | version **upgrade** v_prev→v_new (enum/schema renames matter) |
| Output | per-check pass/fail vs baseline | per-round diff list, each diff *explained or investigated* |

Use both: this playbook catches structural/row-level loss across the whole DB;
the accounting reference proves the numbers a controller signs cuadran.

This battery is the generalisation of the one that worked on **Pharmactive
v15→v18** (5 rounds, origin vs migrated). Every "expected diff" below is a real
finding from that migration, kept as a worked example.

---

## When to use

- Right after a version upgrade (Odoo S.A. upgrade service, indaws_migration_kit,
  or a manual `-u all` rebuild), before any functional sign-off.
- Iteratively, once per correction round, until every diff is either **explained**
  (with a one-line reason in the migration log) or **fixed**.
- Whenever a stakeholder says "faltan datos" — this turns a vague fear into a
  concrete, auditable list.

You need read access to both Postgres DBs. The snippets assume:

```bash
PREV="docker compose exec db psql -U odoo -d <db_v_prev>"   # origin
NEW="docker compose exec db psql -U odoo -d <db_v_new>"     # migrated
# or plain: psql "postgresql://odoo:odoo@host:5432/<db>"
```

All snippets are parametrisable by table/field — replace the `:table` /
`:field` placeholders. Run the same query on both DBs and diff the output
(`psql ... -At -c "..." > prev.txt` / `> new.txt`, then `diff`).

---

## THE GOVERNING PRINCIPLE

> **Every difference is either explained or investigated. Enum renames and
> system/analytic *additions* are NOT data loss. Real loss is `v_new < v_prev`
> in genuine transactional data with no explanation.**

Three buckets for any diff you find:

1. **Explained — not loss.** Documented cause: enum rename across versions
   (`done`→`sale`, `product`→`consu`), dedup of true duplicates, cleanup of
   zero-quantity rows, system rows added by the upgrade, analytic lines
   rebuilt by the v16 redesign, new security groups/rules. Write the one-line
   reason and move on.
2. **Investigated — pending.** A decrement or mismatch you cannot yet explain.
   Open it, find the cause, then it moves to bucket 1 or 3.
3. **Real loss — fix.** `v_new < v_prev` on real transactional rows (orders,
   invoices, moves, partners, products) with no benign cause. Stop sign-off.

A decrement is a **red flag, not a verdict**. Two real Pharmactive decrements
that were *correct*:

- `stock_move_line` **−3679**: zero-quantity lines cleaned up by the upgrade;
  the lines with real `quantity` were byte-identical on both sides.
- `res_partner_bank` **−13**: true duplicates deduplicated.

Conversely, *increments* in `account_analytic_line`, `res_groups`,
`ir_model_access`, `ir_rule` are **expected** (the new version adds them) and
are NOT proof of corruption.

---

## Round 1 — Table row counts (catch `v_new < v_prev`)

Goal: a count of every table present on **both** DBs, side by side, flag where
`v_new < v_prev`. Decrements are the only thing that needs explaining
(increments from system/analytic/security are expected).

### 1a. Enumerate common tables and their live row counts

`pg_class.reltuples` is a fast estimate; use it to triage, then confirm exact
counts on suspicious tables.

```sql
-- Run on EACH db, dump to prev_counts.txt / new_counts.txt
SELECT c.relname AS table, c.reltuples::bigint AS approx_rows
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind = 'r' AND n.nspname = 'public'
ORDER BY c.relname;
```

```bash
$PREV -At -F$'\t' -c "<query above>" | sort > prev_counts.txt
$NEW  -At -F$'\t' -c "<query above>" | sort > new_counts.txt
join -t$'\t' prev_counts.txt new_counts.txt \
  | awk -F'\t' '$3 < $2 {print $1"\tPREV="$2"\tNEW="$3"\tDELTA="$3-$2}'
# -> lists only tables that SHRANK. These are your Round-1 worklist.
```

### 1b. Exact count for a suspect table (parametrisable)

```sql
-- :table  e.g. stock_move_line, sale_order, account_move
SELECT count(*) FROM :table;
```

### 1c. Prove a decrement is benign (zero-qty / duplicate cleanup pattern)

```sql
-- Were the removed rows all "empty"? (stock_move_line example: qty 0)
-- v_prev:
SELECT
  count(*) FILTER (WHERE quantity = 0 OR quantity IS NULL) AS zero_rows,
  count(*) FILTER (WHERE quantity <> 0)                    AS real_rows
FROM stock_move_line;
-- The real_rows count MUST match v_new's total (or v_new's real_rows).
```

```sql
-- Duplicate dedup (res_partner_bank example): count true duplicates in v_prev
SELECT count(*) - count(DISTINCT (partner_id, sanitized_acc_number)) AS dup_rows
FROM res_partner_bank;
-- dup_rows should equal the decrement (e.g. 13).
```

Tables that legitimately appear/disappear (uninstalled modules, renamed
models) are noise — confirm against the module list, do not treat as loss.

---

## Round 2 — Financial integrity

Goal: the books cuadran and totals per document type are conserved. A
`move_type` reclassification (e.g. `entry`↔`in_invoice`) is **not** loss — it
moves a row between buckets; the global sums stay equal.

### 2a. Global balance (must be 0 on both, and equal)

```sql
SELECT
  round(sum(debit), 2)  AS debit,
  round(sum(credit), 2) AS credit,
  round(sum(debit) - sum(credit), 2) AS balance
FROM account_move_line
WHERE parent_state = 'posted';
-- balance = 0 on BOTH dbs. debit and credit identical across dbs.
```

### 2b. Totals per move_type (reclassification check)

```sql
SELECT am.move_type,
       count(*)                          AS moves,
       round(sum(aml.balance), 2)        AS sum_balance
FROM account_move am
JOIN account_move_line aml ON aml.move_id = am.id
WHERE am.state = 'posted'
GROUP BY am.move_type
ORDER BY am.move_type;
```

If `entry` shrinks while `in_invoice` grows by the same magnitude and the
**grand total is unchanged**, that is a reclassification → explained. The
invariant to assert: `sum(balance)` over ALL move_types is identical v_prev vs
v_new even if the per-type split moved.

### 2c. Document totals (SO / PO / invoices)

```sql
-- Sales orders
SELECT count(*) AS orders,
       round(sum(amount_total), 2) AS total,
       round(sum(amount_untaxed), 2) AS untaxed
FROM sale_order;
```

```sql
-- Purchase orders
SELECT count(*) AS orders,
       round(sum(amount_total), 2) AS total
FROM purchase_order;
```

```sql
-- Invoices/bills by type
SELECT move_type, count(*) AS n,
       round(sum(amount_total), 2)        AS total,
       round(sum(amount_total_signed), 2) AS total_signed
FROM account_move
WHERE state = 'posted'
  AND move_type IN ('out_invoice','out_refund','in_invoice','in_refund')
GROUP BY move_type ORDER BY move_type;
```

Counts and totals must match per type (or differences must net to zero via a
documented reclassification).

---

## Round 3 — Field-by-field (sampling) + version enum renames

Goal: spot-check that real field values survived. The **only** systematic
diffs allowed are documented enum renames between versions. These are NOT
corruption.

### Known version enum renames (expect these, do not flag as loss)

| Model.field | v_prev value | v_new value | Version |
|---|---|---|---|
| `sale.order.state` | `done` | `sale` | v17 dropped the `done` state |
| `product.template.type` | `product` | `consu` + `is_storable=True` | v18 storable refactor |
| `account.move` (others) | per release notes | — | check the version's migration notes |

```sql
-- sale.order.state distribution (compare both dbs)
SELECT state, count(*) FROM sale_order GROUP BY state ORDER BY state;
-- v_prev rows in 'done' should appear as 'sale' in v_new; the SUM is conserved.
```

```sql
-- product storable refactor (v18): 'product' -> 'consu' + is_storable
-- v_new:
SELECT type, is_storable, count(*)
FROM product_template GROUP BY type, is_storable ORDER BY type;
-- count(type='product')[v_prev] == count(type='consu' AND is_storable)[v_new]
```

### Generic field sampling (parametrisable)

Match the same business records on both DBs by a stable key and diff a field.
Use a deterministic ORDER BY + LIMIT so both samples are the same rows.

```sql
-- :table :field :key  -- e.g. sale_order / name / id
SELECT id, :field
FROM :table
ORDER BY id
LIMIT 200;
-- diff prev vs new; the only deltas should be documented enum renames.
```

```sql
-- Partner core fields sample
SELECT id, name, vat, email, country_id
FROM res_partner WHERE active = true ORDER BY id LIMIT 200;
```

```sql
-- Invoice header sample (legal fields)
SELECT id, name, ref, invoice_date, amount_total, partner_id
FROM account_move
WHERE move_type = 'out_invoice' AND state = 'posted'
ORDER BY id LIMIT 200;
```

Any value diff that is NOT a known enum rename goes to bucket 2 (investigate).

---

## Round 4 — Relational integrity & inventory

Goal: stock on-hand is byte-identical, there are zero orphans, and the number
of documents-with-lines is conserved.

### 4a. Stock on-hand in internal locations (MUST be identical)

```sql
SELECT round(sum(sq.quantity), 3) AS on_hand
FROM stock_quant sq
JOIN stock_location sl ON sl.id = sq.location_id
WHERE sl.usage = 'internal';
-- Identical v_prev vs v_new. Per-product breakdown if it differs:
```

```sql
SELECT sq.product_id, round(sum(sq.quantity), 3) AS qty
FROM stock_quant sq
JOIN stock_location sl ON sl.id = sq.location_id
WHERE sl.usage = 'internal'
GROUP BY sq.product_id ORDER BY sq.product_id;
```

### 4b. Orphans must be 0 (parametrisable FK check)

```sql
-- move_line without its move
SELECT count(*) FROM stock_move_line ml
LEFT JOIN stock_move m ON m.id = ml.move_id
WHERE ml.move_id IS NOT NULL AND m.id IS NULL;
```

```sql
-- generic orphan template: child rows whose parent FK is dangling
-- :child :fk :parent
SELECT count(*) FROM :child c
LEFT JOIN :parent p ON p.id = c.:fk
WHERE c.:fk IS NOT NULL AND p.id IS NULL;
-- examples: account_move_line.move_id -> account_move
--           sale_order_line.order_id  -> sale_order
--           account_move_line.partner_id -> res_partner
```

Run on v_new (and v_prev for a baseline — v_prev should already be 0; if not,
the source was already dirty and the upgrade may have cleaned it).

### 4c. Documents-with-lines conserved

```sql
-- number of orders that have at least one line (parametrisable header/line)
-- :header :line :fk   e.g. sale_order / sale_order_line / order_id
SELECT count(DISTINCT h.id)
FROM :header h JOIN :line l ON l.:fk = h.id;
```

Same count on both DBs for sale_order/sale_order_line, account_move/
account_move_line, purchase_order/purchase_order_line, stock_picking/stock_move.

---

## Round 5 — Analytic, security & translations

Goal: handle the three areas where the migrated DB **grows on purpose** and
where storage format changes. Growth here is expected; the test is "nothing
lost / nothing altered", not "counts equal".

### 5a. Analytic lines — v16 tag→`analytic_distribution` redesign ADDS rows

The v16 analytic redesign reconstructs analytic lines from
`analytic_distribution`. So `account_analytic_line` count in v_new is
**≥** v_prev. Do NOT compare counts. Compare **by id**: every v_prev line must
still exist with the same amount, and the new lines must be auditable
(have a `move_line_id`).

```sql
-- (a) 0 original lines lost: every v_prev id present in v_new
--     dump ids on both and diff:
SELECT id FROM account_analytic_line ORDER BY id;     -- on both dbs
-- comm -23 prev_ids.txt new_ids.txt  -> MUST be empty (no id only in prev)
```

```sql
-- (b) 0 amounts altered on surviving ids: dump (id, round(amount,2)) on both
SELECT id, round(amount, 2) FROM account_analytic_line ORDER BY id;
-- join on id; any amount diff on a shared id = real corruption (bucket 3)
```

```sql
-- (c) the N new lines are auditable (carry a move_line_id we can trace)
--  on v_new, for ids NOT in v_prev:
SELECT count(*) FILTER (WHERE move_line_id IS NULL) AS new_lines_without_source,
       count(*)                                     AS new_lines_total
FROM account_analytic_line
WHERE id > (SELECT max(id) FROM account_analytic_line);  -- or: id NOT IN prev set
-- new_lines_without_source SHOULD be 0 (every reconstructed line is traceable).
```

Verdict pattern: **0 perdidas, 0 importes alterados, N nuevas con
`move_line_id` auditable** → explained, not loss.

### 5b. Security — groups / access / rules only GROW

```sql
SELECT 'res_groups'       AS t, count(*) FROM res_groups
UNION ALL SELECT 'ir_model_access', count(*) FROM ir_model_access
UNION ALL SELECT 'ir_rule',         count(*) FROM ir_rule;
-- v_new >= v_prev for all three. A DECREMENT means a group/ACL/rule was lost
-- in the upgrade (bucket 2/3 — investigate which module owned it).
```

### 5c. Translations — `name` becomes jsonb in v16+

In v16+ translatable fields like `name` are stored as `jsonb` (`{"es_ES": ...,
"en_US": ...}`), not as side `ir_translation` rows. Compare the **language you
care about** by extracting it from the jsonb on v_new and against the plain
column / ir_translation on v_prev.

```sql
-- v_new (jsonb): the es_ES value of a translatable name (parametrisable :table)
SELECT id, name->>'es_ES' AS name_es
FROM :table          -- e.g. product_template
ORDER BY id LIMIT 200;
```

```sql
-- v_prev (<=v15): plain column, or ir_translation side table
SELECT id, name FROM product_template ORDER BY id LIMIT 200;
-- or, when the value lived in ir_translation:
SELECT res_id, value FROM ir_translation
WHERE name = 'product.template,name' AND lang = 'es_ES' ORDER BY res_id;
```

The `es_ES` value on v_new (`name->>'es_ES'`) must equal the v_prev translated
value. A NULL `->>'es_ES'` on a row that had a Spanish name in v_prev = a lost
translation (investigate).

---

## Per-round verdict table (paste into migration log)

| Round | Area | Result | Explained diffs | Open / loss |
|---|---|---|---|---|
| 1 | Row counts | | zero-qty cleanup, dedup, uninstalled modules | |
| 2 | Financial | | move_type reclassification (net 0) | |
| 3 | Field sampling | | enum renames (state, type) | |
| 4 | Relational/inventory | | — (orphans MUST be 0) | |
| 5 | Analytic/security/transl. | | analytic +N, security +N | |

Sign-off requires the **Open / loss** column empty across all five rounds.

---

## Integration with the rest of the skill

- This is part of **Modo B** (Verification), same sub-mode family as
  `migration-accounting-checks.md`. Run **before** any Modo C execution — never
  run transactional flows on v_new until all five rounds are green, or you
  contaminate the data you are verifying.
- Test taxonomy: type 8 `DATA_INTEGRITY`, subtypes `count_comparison`,
  `balance_comparison`, `valuation_comparison`. See `references/test-taxonomy.md`.
- Complements `developer-skills:odoo-database-upgrader` FASE 5 (post-upgrade SQL) — that
  skill focuses on the upgraded dump's Postgres; this playbook diffs **both** ends
  round by round with the explain-or-investigate discipline.
