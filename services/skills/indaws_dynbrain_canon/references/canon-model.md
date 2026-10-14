# The canonical model

A managed article carries eleven properties, all prefixed `canon_`. The prefix is
unchanged from the functional document on purpose: a future Odoo module can adopt
these keys as they are.

There is no ORM behind any of this. No `write()` override, no `_sql_constraints`,
no ACL on the fields. What keeps the model coherent is `scripts/canon_state.py`
plus the daily pass, which calls `validate_properties()` over everything managed
and **reports** what slipped through. Detecting a bad state next morning is worse
than a constraint and far better than nothing.

## The switch

`canon_managed` absent or false means **invisible to this system**: not swept, not
deadlined, not validated. That is the state of all 2.880 existing articles, and
installing this changes nothing about them.

## The eleven keys

| Key | Type | What it is for |
| --- | --- | --- |
| `canon_managed` | boolean | The master switch |
| `canon_state` | selection | One of the six states below |
| `canon_territory` | selection | Territory by **field**, never by location |
| `canon_owner` | many2one res.users | Who signs it. Who gets told |
| `canon_truth_source` | selection | `derived` · `decided` · `observed` |
| `canon_verified_on` / `canon_verified_by` | date / m2o | When, and by whom |
| `canon_review_deadline` | date | Set when a review opens |
| `canon_expires_on` | date | Annual floor. **Only for `decided`** |
| `canon_superseded_by` | m2o knowledge.article | The supersession chain |
| `canon_last_novelty` | m2o knowledge.article | Last novelty that hit it |

Two mechanical facts about where these can live, both verified against production
and both failing **silently** if ignored:

- An article can only carry properties if its **direct parent** declares
  `article_properties_definition`. Inheritance does not cascade to grandchildren.
- A **root** (`parent_id = False`) can never carry them at all.

In both cases the write returns success and stores nothing. Always re-read to
confirm.

## The six states

```
draft ──► in_review ──► verified ──► expiring ──► unverified
                            │            │             │
                            │            └──────┐      │
                            │                   ▼      ▼
                            └──────────────►  verified  ◄── deprecated
                                    (all of verified/expiring/unverified
                                     can go to deprecated; deprecated can
                                     come back to verified)
```

`verified` and `expiring` are the **eligible** states: indexed, and swept.
Everything else is out of both. Note what that means for `expiring` — passing a
deadline *warns*, it does not de-index. Only the grace period expiring does.

Nothing here is irreversible. Every state can return to `verified` with one
action, and no transition ever deletes or archives anything.

`draft` and `in_review` cannot be deprecated: an article nobody ever signed has no
supersession chain to record, so retiring it would be a delete wearing a
deprecation's clothes.

## The three truth sources

| Source | Who maintains it | Expires? | Lives under |
| --- | --- | --- | --- |
| `decided` | A person, who signs it | Yes, annually | *Cómo trabajamos* |
| `derived` | Its importer (Git, a system) | **Never** | *Referencia viva* |
| `observed` | Nobody; it is a record | No | *Aprendido* |

The truth source is not decoration — it drives the triage. An `observed` article
never contradicts anything, because it describes what happened rather than
prescribing what to do. So filing a document under the right child of its territory
*is* an instruction to the sweep.

## Deadlines

- A review opened today falls due in the territory's `review_deadline_days`,
  default **21**.
- A **high-risk** territory gets **7 days, fixed**. Its declared days are ignored
  rather than minimised, so it cannot shorten its own deadline by declaring fewer.
- An `expiring` article drops to `unverified` after `grace_days`, default **7**.
- `decided` truth gets an annual floor: re-sign a year after verification. A 29
  February lands on the 28th rather than skipping into March.

## The one guard that matters

The RAG exclusion rule matches `canon_managed = True AND canon_state NOT IN
(verified, expiring)`. So **writing the switch without verifying in the same breath
retires the article from the index.** Marking 500 articles managed to start curating
them would silently retire 500.

`activate_canonicity()` therefore takes the state and the switch together, and
`assert_index_safe()` refuses any managed-and-not-eligible write. There is
deliberately no way to set `canon_managed` alone through this module.

**`supersede()` is the exception, by design.** Retiring a source that a curated
article replaced is exactly a managed-and-not-eligible write, and it is the point:
the source leaves the index, keeps its body, keeps its place in the tree, and gains
an auditable pointer to what replaced it. The guard protects activation, not
retirement — conflating the two would make the curation path impossible.
