# Workflow Detail — CRM Extraction and Spreadsheet

## STEP 1 — CRM Data Extraction

### 1.1 Connection

Delegate authentication to `common-skills:odoo-pilot`. **Connection and batch patterns are not
documented in this skill** — they live in the cross-repo pointer
`references/shared/odoo-connection.md`. Default credentials for the corporate Odoo:

| Variable | Default |
|----------|-------------|
| `ODOO_URL` | from `odoo_discovery.py` / pack `instance.json` `odoo_url` (example: `https://portal.dynapps.es`) |
| `ODOO_DB` | `master` |
| `ODOO_USER` | — (required) |
| `ODOO_KEY` | — (required) |

```bash
eval $(auth.sh)
```

### 1.1.bis Alternative connection via the Odoo MCP (Cowork environment)

In Cowork there is **no** `auth.sh`/`search_records.sh` nor the `ODOO_*` variables. What
IS available is the **Odoo MCP** (generic `orm_search_read` / `orm_write` over any
model, running as **superuser**). Use it as the alternative path for STEP 1 (reading
`crm.lead` / `sale.order` / `sale.order.spreadsheet`) and STEP 3 (writing the spreadsheet).

Equivalences:

| auth.sh / search_records.sh | Odoo MCP |
|---|---|
| `search_records.sh <model> <domain> <fields>` | `orm_search_read(model, domain, fields)` |
| `update_record.sh <model> <ids> <vals>` | `orm_write(model, ids, vals)` |
| create record | `orm_create(model, vals)` |

**MCP proxy gotchas (document and keep in mind):**

1. **Does not return binaries.** `image_256`, `image_1920`, etc. come back as `false`; the proxy
   only exposes `_binary_url_fields`. → **Team photos CANNOT be pulled via MCP**:
   use `fetch_team.py` (RPC with API key) or leave `photo:""` (falls back to an initials avatar).
2. **`odoo_attachment_link` is blocked for `hr.employee`** ("Model hr.employee is not
   readable on this server"). Don't try to get the photo via attachment.
3. **Large fields: INLINE or PERSISTED — it's not deterministic.** The proxy decides based on the
   payload size (there's no flag to force it). `sale.order.spreadsheet.spreadsheet_data`
   (~45–50 KB) usually **persists to disk** (the tool result carries the PATH to a
   `{model,count,records:[...]}`); `crm.lead.xx_data_collection` (~45 KB) often
   comes back **inline** (the content in the tool result itself). **Never assume the flow will
   persist.** Golden rule: **the bytes never pass through the orchestrator's context** — the
   intake subagent (STEP 1) is the only one that touches them and normalizes them into a
   file verified with the `scripts/mcp_fetch.py` courier:

   ```bash
   # PERSISTED: the tool result carries a host path
   python3 scripts/mcp_fetch.py save --to <work>/spreadsheet.json --from-host "<ruta_host>"
   # INLINE: save the content (Write) to the file and validate
   python3 scripts/mcp_fetch.py verify --to <work>/toma.html
   ```

   `mcp_fetch.py` bridges host↔VM (mounts `.claude/`), does a **read-back verify** (sha256+bytes,
   catching truncated writes) and prints only a `{path,bytes,sha256}` manifest — never the
   content. **NOTE:** when the field persists, the file is the `{model,count,records}` wrapper;
   `spreadsheet_data` is the **internal JSON string** (`records[0]["spreadsheet_data"]`) → it has to be
   extracted before `json.loads`. The subagent does this and writes the internal value.

4. **Locating the spreadsheet.** It's located via `sale.order.spreadsheet` with
   `order_id == <sale_order_id>`. The `spreadsheet_data` field is a **JSON string** (you need to
   `json.loads` the field value, not the wrapper). The `crm.lead` is reached via
   `sale.order.opportunity_id`.

5. **Large chatter → paginate by `id asc` and deduplicate.** The `mail.message` records for a lead with
   a long history (ANRO: 74 messages, ~580 KB) **exceed the proxy's limit in a single
   call**. Paginate with `offset` in blocks (~20). **Sort by `id asc`, NOT by `date asc`:**
   with tied timestamps, sorting by date is unstable and duplicates/drops records at
   page boundaries. Even so, **deduplicate by `id`** when concatenating (belt and suspenders). The
   bodies of automatic marketing mailings (multi-KB templates, irrelevant) can
   be abbreviated to keep the file manageable.

### 1.2 Identify the opportunity

Ask for the ID or name of the `crm.lead`. If there are duplicates, show a list.

### 1.3 Extract lead data

```bash
# Intake data (primary source)
search_records.sh crm.lead '[["id","=",<LEAD_ID>]]' \
  '["name","xx_data_collection","description","partner_id","tag_ids","stage_id"]'

# Partner (company)
search_records.sh res.partner '[["id","=",<PARTNER_ID>]]' \
  '["name","website","industry_id","employee","street","city","country_id"]'

# Chatter (last 50 notes)
search_records.sh mail.message \
  '[["res_id","=",<LEAD_ID>],["model","=","crm.lead"],["message_type","in",["comment","note"]]]' \
  '["body","date","author_id"]' 50

# Relevant attachments
search_records.sh ir.attachment \
  '[["res_id","=",<LEAD_ID>],["res_model","=","crm.lead"]]' \
  '["name","mimetype","file_size"]'
```

**Source priority:**
1. `xx_data_collection` (HTML, primary source — generated by `lead-data-collection`)
2. Relevant attachments (PDF, DOCX)
3. Lead `description`
4. Chatter (internal notes)

### 1.4 Parse the intake data

```bash
echo '<html_content>' | python3 scripts/parse_toma_datos.py > parsed_data.json
```

Output:
```json
{
  "apps_detectadas": { "crm": {"nombre": "CRM", "base": 750, "razon": "..."} },
  "complementos_sugeridos": { "crm": [{"nombre": "...", "precio": 500}] },
  "metadata": { "num_usuarios": 25, "sector": "fabricación" },
  "resumen": { "num_apps": 7, "total_base": 10500, "total_estimado": 14250 }
}
```

**If there's no intake data:** ask the user for the data manually or run `sales-skills:lead-data-collection` first.

### 1.5 Detect existing quotes on the opportunity

BEFORE doing pricing, check whether the opportunity already has a `sale.order` and with which template:

```bash
search_records.sh sale.order '[["opportunity_id","=",<LEAD_ID>]]' \
  '["id","name","state","sale_order_template_id","amount_total"]'
```

- If one exists with `sale_order_template_id == 15` ("Implantación v2") → reuse it (STEP 3 fills that one).
- If one exists with ANOTHER template (e.g. id=1 "Propuesta de Implantación Odoo") → **decide with the sales rep**: create a new v2 order (recommended, doesn't touch the previous one) vs. converting the existing one. **Resolve this in STEP 1/2, not midway through the workflow.**
- If it doesn't exist → it will be created in STEP 3 (§3.0).

---

## STEP 3 — Writing the spreadsheet

### 3.0 Create the sale.order with the v2 template + spreadsheet (if it doesn't exist)

**Critical:** creating the `sale.order` via ORM with `sale_order_template_id=15` **does NOT** apply the template (the onchange is UI-only): the order ends up with no lines and no spreadsheet. Reliable recipe (via **odoo-pilot**, full RPC):

```bash
# 1. Create the order (it may come out empty)
#    create sale.order {partner_id, opportunity_id, sale_order_template_id: 15}

# 2. Create the 2 lines from the template (sale.order.template.line ids 5,6: Análisis + Fase de implantación)
search_records.sh sale.order.template.line '[["sale_order_template_id","=",15]]' \
  '["product_id","name","product_uom_qty"]'
#    create sale.order.line for each one with order_id = <NEW_ORDER_ID>

# 3. Create the spreadsheet by copying the CANONICAL BASE (order_id=false) — which ALREADY includes the
#    'Parámetros' sheet (prices editable per deal/country). Do NOT copy just any "recent"
#    base, nor the old id=1 (plain str): they might NOT carry 'Parámetros' and the
#    pricing override wouldn't be available.
#      - ES (db master): "Implantación v2 base (canónica)"  (order_id=false, id=72)
#      - EN:             "Value based (English)"             (order_id=false, id=16)
search_records.sh sale.order.spreadsheet '[["order_id","=",false],["name","ilike","canónica"]]' '["id","name"]'
#    read its spreadsheet_data (dict) and
#    create sale.order.spreadsheet {order_id: <NEW_ORDER_ID>, name: "Hoja de implantación base", spreadsheet_data: <canonical copy>}
#    Verify that the copied spreadsheet HAS the 'Parámetros' sheet (if it's missing, the
#    canonical base isn't correct; regenerate with generate_spreadsheet_template.py — see §Parámetros).
```

Verify at the end: the order has 2 lines and 1 linked `sale.order.spreadsheet`. If this can't be achieved, return `{"status":"cannot_create_spreadsheet"}` and ask the sales rep to create the quote from the opportunity by choosing "Implantación v2" (the UI does generate it).

### 3.1 Locate the sale.order's spreadsheet

### 3.1 Locate the sale.order's spreadsheet

The quote must have been created with the **"Implantación v2"** template (`sale.order.template` id=15).

```bash
search_records.sh sale.order.spreadsheet '[["order_id","=",<SALE_ORDER_ID>]]' \
  '["id","name","spreadsheet_data"]'
```

If there's no linked spreadsheet → wrong template. Ask the sales rep to select "Implantación v2".

### 3.3 Fill in the spreadsheet

```bash
# 1. Download the current spreadsheet
search_records.sh sale.order.spreadsheet '[["order_id","=",<SALE_ORDER_ID>]]' '["id","spreadsheet_data"]'

# 2. Save the spreadsheet and the quote JSON
cat > /tmp/current_spreadsheet.json << 'EOF'
<spreadsheet_data_json>
EOF
cat > /tmp/budget_data.json << 'EOF'
<budget_json>
EOF

# 3. Run fill_spreadsheet.py
python3 scripts/fill_spreadsheet.py /tmp/current_spreadsheet.json /tmp/budget_data.json /tmp/filled.json

# 4. Upload back
update_record.sh sale.order.spreadsheet '[<SPREADSHEET_ID>]' '{"spreadsheet_data": "<escaped_json>"}'
```

### 3.4 Update the sale.order lines

After filling in the spreadsheet, update the two `sale.order` lines with the calculated prices (not the product's generic unit prices):

| Line | `price_unit` | Justification |
|---|---|---|
| **Análisis** (`product_id` "Análisis de proyecto") | `ana_price` | Closed price for the functional analysis |
| **Fase de implantación** (`product_id` "Implantación ERP") | `impl_total_final` | Implementation TOTAL with ALL apps + support (10%) + developments from the spreadsheet. The fine-tuning (apps/devs checked) is done live by the sales rep on the single list; the sale.order stores the full reference total |

**Line re-sync (single source of truth) — ALWAYS, MCP-first.** In Odoo the spreadsheet and the `sale.order.line` records are independent: editing the sheet does NOT recalculate the lines. They must be rewritten from the sheet's current state **every time** (here in STEP 3.4 and again at the start of STEP 4, after re-reading the sheet that the sales rep may have touched). It's idempotent.

```bash
# 1) Both totals, calculated from the sheet with the SAME engine as the proposal
#    (this way sheet = order = proposal by construction). JSON output, no flooding.
python3 scripts/order_totals.py <spreadsheet.json> --locale es
#    -> {"ana_price":3680,"impl_total_final":51370,"soporte":4620,...}
```
```
# 2) Locate the two lines and rewrite price_unit (via MCP; the order is writable per line):
orm_search_read("sale.order.line", [["order_id","=",<SALE_ORDER_ID>]], ["id","name","product_id"])
orm_write("sale.order.line", [<ANA_LINE_ID>],  {"price_unit": <ana_price>})
orm_write("sale.order.line", [<IMPL_LINE_ID>], {"price_unit": <impl_total_final>})
```
**Análisis** line (`product_id` "Análisis de proyecto") = `ana_price` (sheet E10). **Fase de implantación** line (`product_id` "Implantación ERP") = `impl_total_final` (sheet E38): ALL the apps + Config + support + ALL the developments (the fine-tuning of apps/devs is done live by the sales rep on the single list; the order stores the full reference total).

`impl_total_final` = `config + Σapps(base+extras) + soporte_salida + Σdevs`, with `soporte_salida = calc_soporte_salida(ss, [config]+apps+devs, ana_price)` (10% of the HOURS, factor/rates from sheet B4/B5/B6 — see STEP 4/`proposal_data`). `order_totals.py` already calculates it; do NOT recalculate by hand.
> ⚠️ The old `round_to_10(apps_total × 0.10)` (10% of apps only) diverged from the sheet (~€2,190 in ANRO), and **the lines went stale** if they weren't rewritten after editing the sheet (real ANRO: lines 2,700/49,980 vs. sheet 3,680/51,370). This re-sync kills both causes. **STEP 4 runs it again and adds a guard**: it reads the two lines and warns/rewrites if they don't match `order_totals.py`.
**Added value / AI**, **future migration**, and **on-site presence** are NOT part of this line: they're optional and are checked live on the single list in the `live` HTML. **Training** is included in the exit support (2h per session).

### 3.5 Verify and STOP

**Machine check FIRST (before involving the sales rep)** — the sales rep reviews prices; structure is the machine's job. From the re-downloaded sheet, verify:
- 'Interna Comercial': analysis-sessions summary (B44) **> 0**, analysis weeks (B51) **> 0**, implementation weeks (B52) **> 0** — these are the "(skill rellena)" cells; if any is 0, STEP 3 did not complete `duracion`/`formacion` (typical cause: direct cell edits instead of `fill_spreadsheet.py`).
- 'Gantt' sheet: **painted cells present** (≥ 1 bar per active app + analysis + go-live). The template placeholders alone (3 labels, 0 painted cells) = incomplete.
- ⚠️ Matching totals do NOT prove completeness: in ANRO PR4035 the totals were correct and the Gantt was empty. If any check fails → **back to STEP 3** (re-run `fill_spreadsheet.py` and re-upload), do not present to the sales rep.

Then the sales rep must open the spreadsheet from the quote in Odoo and check:
1. Enabled extras and hidden inactive apps
2. Correct % adjustment
3. Analysis sessions and developments
4. Weeks and price/month

**STOP**: the skill does NOT continue to STEP 4 without explicit confirmation from the sales rep. The sales rep can adjust the spreadsheet directly in Odoo (quantities, prices, % adjustments). On the next iteration, the skill re-reads the updated state.

---

## STEP 4 — Generate web HTML + executive PDF (Dynapps brand)

**Pipeline order (v5, per rendered deliverable):** references cache fresh? → `match_cases.py` →
`fetch_case_images.py` (heroes + logos) → `generate_proposal.py` → `brand_lint.py --html` →
`verify_totals.py`. The two post-generation gates are **mandatory before ANY delivery**, on the
web HTML (STEP 4) and again on the PDF-source HTML (STEP 5.2).

### 4.0 Orchestrator sub-steps (full spec — SKILL.md carries only the index)

**ALWAYS ask at the start** (these change the deliverable): **(1) Mode** — `predemo` (no prices) or `live` (interactive, real prices, source of the PDF); **(2) Team** — sales rep + project manager + consultant names (for `fetch_team.py`). *(The brand is always Dynapps.)*

**Data injection (critical):** ALL corrections (photos, anchors, dev items) go **into `extra.json`** and must **always be re-injected before regenerating** — the generator only renders `extra_data`, it does NOT read auxiliary files (`team.json` etc.).

**🛑 Two-phase flow — do not generate the PDF until you have the client's selection.** STEP 4 generates **ONLY the web HTML** (`generate_proposal.py --only web`): the interactive `live` deliverable the sales rep uses to check/uncheck apps, custom dev items and add-ons in front of the client; it gets published and registered on the sale.order (STEP 5.1). **After the demo** (STEP 5.2), the sales rep copies the selection ("Copy selection" button → JSON), it is injected into `extra_data` and **ONLY the PDF** is generated (`--only pdf`, body = only what was checked).

Both deliverables follow `common-skills:dynapps-brand` and come from the **same `extra_data.json`** + `scripts/generate_proposal.py` (without `--only` it generates both — legacy/test usage): **`propuesta-pr<NUM>.html`** = self-contained interactive web in the **v5 client-first order** (hero [optional `hero_outcome` H1] → understanding → challenges max 4 → cases: logo wall + quote-first cards + industry pills → map → timeline → apps with ✓/+ bullets → Why Dynapps: 3 pillars + figures band 900+/#1 worldwide → team → **interactive single list with a live TOTAL** + add-ons/AI → calendar-together AFTER pricing → closing with rep photo, no phone + "Copy selection"); **`propuesta-pr<NUM>.pdf`** = static executive A4 that mirrors the web's order and visual language (methodology+timeline merged into "How we deliver it"; colour only on cover/why/closing/price/pills/timeline), **body = ONLY what was checked** (apps, custom dev, chosen AI), subtotals per block, rendered with headless Chrome from `-pdf.html`. `extra_data.mode` = `predemo` | `live`. Mode matrix, section structure, single-list and dynamic-10% detail, plus the mandatory `extra_data.json` fields, glossary, team, cases, recurring costs and command: `references/html_structure.md` → "STEP 4 — Execution detail".

1. **Re-download** the spreadsheet from Odoo (the sales rep may have made manual changes) + re-read the intake data. **Line re-sync + guard (single source of truth):** compute `python3 scripts/order_totals.py <ss.json> --locale <code>` → `{ana_price, impl_total_final}`; read the two `sale.order.line` records (`orm_search_read`) and, if `price_unit` doesn't match, **rewrite them** (`orm_write price_unit`) from those totals. This keeps sheet = order = proposal in sync (kills the drift: editing the sheet in Odoo does NOT recalculate the lines). See §3.4. **Completeness gate (ABORT):** if 'Interna Comercial' has analysis sessions (B44) = 0, or analysis/implementation weeks (B51/B52) = 0, or the 'Gantt' sheet has no painted cells → **ABORT STEP 4 and go back to STEP 3** (the sheet was filled with direct cell edits, not `fill_spreadsheet.py`; totals matching does NOT prove completeness — real ANRO PR4035 failure: correct totals, "0 weeks" hero and empty timeline). `generate_proposal.py` also warns on stderr. See §3.5.
2. **Write `extra_data.json` with MANDATORY personalized content** (`mode`, `understanding`, `executive_summary_html`, `cost_of_delay`, `anchor_dates`, `app_overrides.<APP>.{intro,fit,features}` — `features` is NOT optional, 1-2 sentences per feature in the client's context —, `dev_overrides`). The `features` KEYS are the exact spreadsheet names (one-liner in `references/html_structure.md`). Field-by-field detail in `html_structure.md` §4.2. **Non-ES:** content written in the target language + `extra_data.locale`; `dev_overrides[key].{title,description}` for a localized title (key stays in Spanish). See `references/localization.md`.
3. **Sector glossary** (`html_structure.md` §4.3.b), **project team** via `scripts/fetch_team.py` (§4.3.c) and **comparable cases** via `scripts/match_cases.py --locale <code>` (§4.3.d). ⚠️ **`match_cases.py` needs the `--locale` of the language the sector/toma text is written in**: without it the default resolves to `en` and a Spanish sector scores `sector=0` on every case → false `weak_match` (the scoring compares LOCALIZED sector tokens; "construcción" never matches "construction"). **Team photos via MCP** (no API key): `ir.attachment` of `res.partner` (`res_field='image_512'`) → `odoo_attachment_link(attachment_id)` → curl → `scripts/embed_team_photos.py <extra.json> "Name=/path"`. Full recipe in `references/mcp_photo_access.md`. Cases carry `url` (per-locale Dynapps case-studies link, `instance.cases_base_url` + slug), `image` (hero), and the v5 proof fields `industry_label`/`logo`/`quote`/`local` (quote ONLY if genuinely on the market site — **never invent one**; absent → factual headline). **References pipeline order: check `shared/references/<CC>.json` freshness (>90 days → stderr warning → refresh with `scripts/fetch_references.py --country <CC>`) → `match_cases.py` (v2 scoring: country-local +8 > exact industry +6 > adjacent +3) → `fetch_case_images.py` (embeds heroes AND logos) → generate.** **`metric` and `image` are MANDATORY per case: use the `match_cases.py` output AS-IS (never retype cases by hand) and ALWAYS run `python3 scripts/fetch_case_images.py [--width 640] <extra.json>` before generating** (on failure falls back to the URL). If `match_cases.py` warns `weak_match` (no case matches the sector): first re-check the `--locale`, then swap in a local case (content-collection) or get explicit sales-rep sign-off; `generate_proposal.py` warns on stderr about cases missing `image`/`metric`.
4. **Brand lint + validation:** run `python3 scripts/brand_lint.py [--locale <code>] <extra.json>` — v2 catches all-caps, brochure-speak/jargon, **DASH** (no em-dashes; en-dash only in numeric/date ranges), **MARCA** (always "Dynapps"), **IDIOMA** (language leaks), **REGISTRO** (nl je/jij, fr vous; es tú untouched), **PRESION** (no pressure phrasing — urgency = planning transparency) + warn-only EXCL/PASIVA; fix the findings. Then **show the `extra_data.json` to the sales rep and get their sign-off before generating the HTML**.
5. **Decide recurring costs** (real values from the `sale.order` or `RECURRENT_DEFAULTS`; overridable via `extra_data.costes_recurrentes`; rule 16) + selectable **future migration** (15% of checked apps+devs / 24 = €/month, **rounded UP to the next multiple of €10** — `ceil_to_10` in Python, `ceil10` in the live JS; "migration cycle renews every 2 years") — see `html_structure.md` §4.5.
6. **Pre-render rule gate (before spending a render):** `python3 scripts/pregate_extra_data.py <extra.json> --ss <ss.json> --toma <toma.html> [--locale <code>]` — the deterministic half of the rules of SKILL.md / `references/rules.md` in code: **FAIL (exit 1)** only for what is a wrong figure or already aborts later (per-user licence rate over the hard cap = PR4021, calendar dates under a country layer that forbids them = the legal gate, an unwritten prose marker = the M5 gate, `features` as a list = the render crashes), **WARN on stderr and continue** for everything else (unknown/missing feature keys, >4 challenges, `weak_match`, cases without metric/image, involvement out of range, hours in client prose). Verdict on stdout: `PREGATE: PASS|FAIL (<n> error(es), <m> aviso(s))`. `qb_run generate` runs it automatically (see below) and **never passes it `--strict`**; with prose still pending it is skipped — that case belongs to the M5 gate. It **replaces nothing**: the two post-generation gates below still run post-render.
7. **Generate ONLY the web HTML** with `scripts/generate_proposal.py [--locale <code>] [--allow-stub] --only web <ss.json> <toma.html> <out_basename> <extra.json>`. Built-in M5 gates: a pack with `pack_status:"stub"` won't generate without `--allow-stub` (today that includes `es`, flagged stub until its promotion; never use it to publish from fr/nl), and if the final HTML carries pack placeholders (`verify_from_odoo`, `<... — verify>`) the script writes nothing and exits with code 3. Check stderr WARNINGs. **The PDF is NOT generated here** — STEP 5.2 generates it with the demo selection. Detail in `html_structure.md` §4.6.
8. **Post-generation gates (MANDATORY before delivering ANY rendered HTML — this step and STEP 5.2):** `python3 scripts/brand_lint.py [--locale <code>] --html <rendered.html>` (rendered output clean, not just the JSON) **then** `python3 scripts/verify_totals.py --html <rendered.html> --extra-data <extra.json> [--locale <code>] [--ss <ss.json> --toma <toma.html>] [--strict]` — recomputes recurring totals with the same engine and hard-FAILS on rendered-summary vs detail drift (PR4021 bug class). Exit 1 → fix and regenerate; never deliver a failing file. Only then return the web HTML path to the sales rep.

**One local chain instead of eight turns (`scripts/qb_run.py`).** Every local, deterministic step
above can run in a **single invocation**. The latency of this skill is turns and emitted tokens, not
CPU (`--only web` renders in 0.20 s):

```bash
# prepare: order_totals -> match_cases -> fetch_case_images -> scaffold_extra_data -> brand_lint(json)
python3 scripts/qb_run.py prepare  /tmp/ss.json /tmp/toma.html /tmp/extra.json [--locale <code>] \
        [--prose /tmp/patch.json] [--no-images]
# generate: pregate_extra_data (PRE-render, FAIL cuts before rendering; WARN continues)
#           -> generate_proposal -> { brand_lint --html + verify_totals --html }  (the two gates, concurrently)
python3 scripts/qb_run.py generate /tmp/ss.json /tmp/toma.html /tmp/propuesta-pr<NUM> /tmp/extra.json --only web
python3 scripts/qb_run.py all      /tmp/ss.json /tmp/toma.html /tmp/propuesta-pr<NUM> /tmp/extra.json
```

Guarantees and deliberate limits: ONE resolved locale injected into every step (no drift); zero
reimplemented logic (each step is the same script as a subprocess, stderr inherited, exit codes
propagated verbatim — the M5 gate's 3 stays a 3); stops at the first failing step and delivers
nothing; **the two gates always run post-render** (PR4021 is a render-time bug class) and are never
weakened. It **does NOT talk to Odoo** (`fetch_team.py`, `fetch_references.py`, the `sale.order.line`
re-sync and publishing stay outside on purpose — `order_totals` is computed locally and printed so
the orchestrator rewrites the lines itself) and it **never crosses a human STOP**: the post-STEP-3
sheet review is assumed done (it reminds you on stderr) and `all` **refuses `--only pdf`** so the
post-demo selection stop cannot be skipped.

**Author only the prose (`scripts/scaffold_extra_data.py`).** A real `extra_data.json` is ~27 KB and
most of it is mechanical. The scaffold derives everything derivable — the exact **KEYS/structure** of
`app_overrides`/`dev_overrides` (spreadsheet feature names, in Spanish even for non-es locales),
`costes_recurrentes` (per-user licence rate, never the pre-multiplied total), `sector_glossary`,
`mode`, `locale`, `num_usuarios`, `num_usuarios_light` (0 = no Light User line), and `casos_exito`/`equipo` spliced **as-is** from
`match_cases.py` (`--cases`) / `fetch_team.py` (`--team`) — and never emits `anchor_dates` (the
engine derives them from the sheet, so the Belgian `_NO_CALENDAR_DATES_CC` gate stays the only
arbiter). Every mandatory prose field is left as a literal `[[WRITE: … — verify]]` marker, which the
**M5 placeholder gate catches** → a scaffold without prose cannot be published (exit 3, nothing
written). The model then writes only the prose patch and merges it: `--prose <patch.json>`
(deep-merge, patch wins, `null` deletes a key, `features`/`dev_overrides` accept a dict by key or a
positional list of the exact same length).

```bash
# 0. References cache (only when preparing cases): shared/references/<CC>.json is the scraped
#    market-site cache (ES/BE/CH; provenance + scraped_at). Older than 90 days => downstream
#    scripts warn on stderr; refresh with:
python3 scripts/fetch_references.py --country <CC>   # [--lang <seg>] [--site <url>] [--out <path>]

# TWO-STAGE FLOW (don't generate the PDF until you have the client's selection):

# STEP 4 — ONLY the web HTML (to build the quote live during the demo)
python3 scripts/generate_proposal.py [--locale <code>] [--allow-stub] --only web /tmp/ss.json /tmp/toma.html /tmp/propuesta-pr<NUM> /tmp/extra.json
#   -> /tmp/propuesta-pr<NUM>.html      (rich interactive web HTML: single list with live TOTAL + "Copiar selección" button)

# GATES (mandatory, in this order, after EVERY generation):
python3 scripts/brand_lint.py [--locale <code>] --html /tmp/propuesta-pr<NUM>.html
python3 scripts/verify_totals.py --html /tmp/propuesta-pr<NUM>.html --extra-data /tmp/extra.json \
        [--locale <code>] --ss /tmp/ss.json --toma /tmp/toma.html   # [--strict]
#   verify_totals recomputes the recurring totals with the SAME engine and hard-FAILS (exit 1)
#   on rendered-summary vs detail drift or an implausible per-user licence rate (PR4021 class).
#   FAIL => fix extra.json / regenerate; NEVER deliver a failing HTML.

# STEP 5.2 — ONLY the PDF, after injecting the selection copied from the demo into extra.json
python3 scripts/generate_proposal.py [--locale <code>] [--allow-stub] --only pdf /tmp/ss.json /tmp/toma.html /tmp/propuesta-pr<NUM> /tmp/extra.json
#   -> /tmp/propuesta-pr<NUM>-pdf.html  (source for the static PDF)
#   -> /tmp/propuesta-pr<NUM>.pdf       (executive PDF: body with ONLY what was checked in the demo, via headless Chrome)
# ... then re-run BOTH gates on /tmp/propuesta-pr<NUM>-pdf.html before uploading the PDF.

# (Without --only it generates both; legacy/test use.)
```

M5 gates (locales/README.md §4 and §6) integrated into the command:

- **pack_status gate**: a pack with `pack_status:"stub"` does NOT generate client-facing output
  (LocaleError, exit 1). `--allow-stub` is the deliberate override; today `--locale es` also needs it because the
  es pack is still flagged `stub` until its formal promotion (its byte-exact parity gate is already green). fr/nl are
  real stubs: do NOT generate client proposals with them.
- **Placeholder gate**: after rendering, the script scans the final HTML (web and pdf, predemo and live)
  looking for unfilled pack markers (`verify_from_odoo`, `<<VERIFY`, `<... — verify>`). If
  any appear, it does NOT write any file and exits with exit 3 listing each marker. Fields in
  `instance.json` still in placeholder state (local phone, partner manager) are OMITTED from the render instead of
  being made up (conditional render) — fill them in the pack so they appear.

`generate_proposal.py` follows `common-skills:dynapps-brand` (Readex Pro; pure brand palette with
extended digital tokens — Midnight Denim #050543, Unicorn Violet, Ferrarri Carmine, Sunset
Tangerine —; brand-guideline 45° gradient violet → carmine at 52% → orange #FF7759; Dynapps
logos; **no `text-transform: uppercase` anywhere**). It reuses the extraction from
`proposal_data.py` (same package) and renders the `context` into two templates:
`templates/propuesta_web.html` (web) and `templates/propuesta_pdf.html` (executive A4 PDF, clean page breaks).

Notes:
- **Requires Google Chrome / Chromium** for the PDF. If it's not present, the script warns and leaves the `-pdf.html`
  (printable to PDF from the browser). Asset override via `DYNAPPS_BRAND_ASSETS`.
- Both files are self-contained (fonts and logos embedded). Contact info configurable via
  `extra_data.contacto = {email, web, tel}`.
- `find_chrome()` searches, in this order: `QB_CHROME_BIN` (explicit override) → macOS paths →
  a `chromium` in the `PATH` (wrapper, see below) → the binary left by
  `playwright install chromium` under `PLAYWRIGHT_BROWSERS_PATH` (default `~/pw`).

### 4.bis Chrome in the Linux sandbox (no root, no pre-installed Chrome)

In the Cowork sandbox there's no Chrome/Chromium and no root (you can't `apt-get install` or
`playwright install-deps`). Procedure that **does** work:

```bash
# 1) Install the browser in a PERSISTENT path (not ~/.cache, which gets cleared between shells)
export PLAYWRIGHT_BROWSERS_PATH=~/pw
playwright install chromium    # chromium-headless-shell may fail; the full chrome does download

# 2) Resolve the missing .so files WITHOUT root: download the .deb packages and extract them to a local dir
mkdir -p ~/pwlibs && cd ~/pwlibs
apt-get download \
  libxdamage1 libxfixes3 libxrandr2 libxcomposite1 libxkbcommon0 libgbm1 libasound2 \
  libatk1.0-0 libatk-bridge2.0-0 libcups2 libatspi2.0-0 libnss3 libnspr4 libdrm2 \
  libxshmfence1 libpango-1.0-0 libpangocairo-1.0-0 libcairo2 libgdk-pixbuf-2.0-0 \
  libgtk-3-0 libx11-6 libxcb1 libxext6 libxrender1 libexpat1 libxcursor1 libxi6 libxtst6
for d in *.deb; do dpkg-deb -x "$d" .; done

# 3) `chromium` wrapper on the PATH that exports LD_LIBRARY_PATH and adds --no-sandbox,
#    so find_chrome() can locate it (exporting QB_CHROME_BIN to the wrapper also works)
mkdir -p ~/bin
CHROME=$(ls ~/pw/chromium-*/chrome-linux/chrome | head -1)
cat > ~/bin/chromium <<EOF
#!/bin/bash
export LD_LIBRARY_PATH="\$HOME/pwlibs/usr/lib/x86_64-linux-gnu:\$HOME/pwlibs/lib/x86_64-linux-gnu:\$LD_LIBRARY_PATH"
exec "$CHROME" --no-sandbox "\$@"
EOF
chmod +x ~/bin/chromium
export PATH=~/bin:$PATH
export QB_CHROME_BIN=~/bin/chromium   # optional but explicit
```

After this, `generate_proposal.py` finds the browser and generates the PDF without touching root.

The **web HTML** and the **executive PDF** are delivered to the sales rep; how to publish to the
client is decided case-by-case outside the workflow. Registering them on the sale.order is STEP 5.

## Render contracts (the detail behind SKILL.md §Document order and views)

`SKILL.md` states the triggers and the non-negotiables; this is the rest of each contract. The
section-order table and the involvement spec live in `references/html_structure.md`; the
app→process mapping rationale in `references/process_taxonomy.md`.

### Process view — the metro map

- **Auto-switch:** ≥6 scoped apps across ≥2 end-to-end processes. Per-deal override
  `extra_data.module_view = "process" | "flat"`. Guarded by `tests/process_view_smoke.py`.
- The map **REPLACES** the mindmap/solution-map section (same `#mapa` anchor, before the module
  breakdown). The mindmap survives **only** in flat view.
- **Hovering ANY station — lit or unlit — lights up its icon and its name.**
- The **FULL Odoo Enterprise network is always drawn**: 6 lines, no milestones and no terminus.
  Line identities are fixed: **Market-to-Lead** carmine feeding into CRM ◎ · **Lead-to-Cash**
  gradient · **Procure-to-Pay** violet · **Record-to-Report** midnight · **Hire-to-Retire** orange
  with the whole employee journey · **Project-to-Result** tangerine, born **VERTICALLY** at Ventas ◎
  ("de una venta nace un proyecto") and tunnelling under P2P/H2R.
- **ALL money lines converge into the Contabilidad final hub** (L2C via Facturación, P2P direct,
  H2R via Gastos) — the visual proof of Odoo's integration.
- **Station states:** lit = in scope and selected · dashed = in scope, not preselected ("Fase 2") ·
  greyed-out ghost = Odoo has it, this proposal doesn't (curated `flows.ghosts`, official icons —
  upsell without money).
- **NO prices on the map**: it sells processes, money lives in the single list.
- Lit stations are **two-way synced** with the single-list checkboxes (station tap = checkbox
  toggle), and single-list checkboxes take the **metro-line colour** of their process.
- The **L1 line renames itself** by where the lit flow starts: "Del clic al cobro" → "Del lead al
  cobro" → "Del presupuesto al cobro" (Quote-to-Cash).
- The **PDF gets the same map as a static SVG**; `predemo` gets a non-interactive map.
- Small deals keep the flat view **untouched (byte-parity, `tests/golden_check.py`)**.
- Network + mapping data: `shared/app_registry.json` (`apps[*].process`, `processes`, `flows`).

### Customer involvement per module

- Visible unit is **days/week** ("≈ 1 día a la semana"); the **% is internal** (20% = 1 day).
- Rendered as a **5-dot half-day scale (½–2½)**, plus a **price-weighted roll-up chip** per process
  header (`_g["involvement_pct"]`, weighted mean).
- **Source priority:** `app_overrides.<APP>.involvement_pct` > the app block's per-app sheet row
  'Dedicación cliente (días/sem)' > `involvement_default_pct` > `deal.involvement_dias`
  (`Parámetros`) > 1 day.
- Clamped to **20–50%** with a stderr `WARNING: involvement_pct … fuera del rango 20–50; se
  recorta.`; a non-numeric `deal.involvement_dias` is ignored with its own stderr warning.
- **Always on in process view**; in flat view **only when set explicitly** (byte-parity:
  `involvement_active = _inv_explicit or module_view == "process"`).

### Workbook tabs by process (v4.19+)

App tabs are grouped **BY PROCESS** (Order-to-Cash / Procure-to-Pay / Record-to-Report / Personas y
soporte); Config General and the construction vertical keep their own tabs. `fill_spreadsheet.py`
keeps a **per-app LEGACY fallback** for pre-v4.19 workbooks (old category tabs). Deployed base
templates need a **re-deploy** to gain the process tabs.

### Analysis gateway panel

Lists the deliverables explicitly ("Al terminar el Análisis, te llevas:" — blueprint,
implementation plan, methodology, timeline, **locked budget** highlighted) next to the fixed price
and the claim "Se contrata primero, por separado · precio fijo", **above** the two rights (right to
stop, right to the locked price). Web + PDF, on **every** `live` deal. Strings are pack-owned
(`deliv_*`, `gw_sep_chip`) and their coverage is checked by `scripts/validate_locale.py`.

### Construction vertical (Dynapps Construcción)

Scoped by `deal.vertical='inbuilt'` (`Parámetros`) or `extra_data.vertical`. Closed price **WITHOUT
hours** → it does **not** add to the 10% go-live support, but it **does** enter the future-migration
base. The pack **includes** the mobile App and the App can be sold standalone; post-demo selection
via `extra_data.vertical_sel = {pack, app}`. It renders as the `#vert-block` price tile on the web
and as `vert_block` in the PDF's value-added group, **plus** a first-class narrative app card.
Double-charge suppression, override schema and the migration state: `references/inbuilt_app.md`.

## STEP 5 — Publish the HTML and, after the demo, attach the PDF (full detail)

Idempotent: **if there's already a previous version on the sale.order, it gets UPDATED, not
duplicated.** Two moments, aligned with STEP 4's two-phase flow.

**5.1 — When delivering the HTML (before/for the demo). Does NOT touch the PDF.**

1. **Publish the HTML** (`utilities:publish` or the hoster's `create_upload_url` + `curl` on the
   file; to re-publish to the same destination use `upload_id`, or delete the previous one by
   looking it up by name `propuesta-pr<NUM>`). Returns the public URL.
2. **Post/update the note** on the `sale.order` (the order's id, NOT the crm.lead) via
   `orm_post_message(model="sale.order", res_id=<order_id>, subtype_xmlid="mail.mt_note",
   body_is_html=True, body=<HTML>)`. Internal note in **rich HTML**: the Dynapps connector's
   `orm_post_message` **preserves HTML** (the escaping bug was in `execute_kw message_post` /
   `orm_create`, NOT in this tool — verified 2026-07). Link the text **"HTML proposal"** as
   `<a href="<URL>">…</a>` (don't paste the raw URL). Idempotent: if there's already a
   "Proposal PR<NUM>" note, delete it (`orm_unlink`) and re-post, don't accumulate.

**5.2 — After the demo, with the client's selection.** The sales rep copies the selection using
the HTML's **"Copy selection"** button (JSON: `aplicaciones`, `desarrollos`, `ia_addons`,
`ia_plataforma`, `poc`, `jornadas_presenciales`, `num_usuarios`, `migracion_futura`,
`recurrentes`, `totales`).

1. **Inject the selection** into `extra.json`: `apps_recomendadas` = `aplicaciones`;
   `devs_recomendados` = `desarrollos`; `value_anadido` = `{mcp/starter_pack per ia_plataforma;
   field_service + ocr_* per ia_addons; poc; migracion = migracion_futura; num_usuarios;
   fs_usuarios}`; `num_usuarios` top-level. Only what was checked will appear in the static PDF.
   `num_usuarios_light` is not in the selection payload (the web has no Light selector): keep the
   value `extra.json` already carries, so the PDF prints the same Light User line as the web.
2. **Generate ONLY the PDF**: `generate_proposal.py [--locale <code>] [--allow-stub] --only pdf
   <ss.json> <toma.html> <out_basename> <extra.json>` → `propuesta-pr<NUM>.pdf` (headless
   Chrome). **Then run the two gates on the PDF source** (`brand_lint.py --html
   propuesta-pr<NUM>-pdf.html` + `verify_totals.py --html propuesta-pr<NUM>-pdf.html
   --extra-data <extra.json> [--ss/--toma]`) — a FAIL blocks the upload.
3. **Upload the PDF out-of-band** — `odoo_request_upload(filename="propuesta-pr<NUM>.pdf")`
   **WITHOUT pre-targeting `sale.order`** (⚠️ pre-targeting `model=sale.order` gives
   `"model sale.order is not writable"`) → `curl -F "file=@<path>.pdf;type=application/pdf"
   "<url>"` → `attachment_id`. The bytes never pass through the context. Alternative with an API
   key: odoo-pilot RPC.
4. **Update the note** (the same one from 5.1): delete the previous one and re-post in rich HTML
   with the "HTML proposal" link + `attachment_ids=[<attachment_id>]`. This keeps the PDF linked
   to the order via the message (the MCP proxy doesn't serve/read inline binaries — Gotcha
   9/11). Verified end-to-end (ANRO PR4021, 2026-07).

Return to the sales rep: in 5.1, the HTML's public URL + the note; in 5.2, confirmation of the
attached PDF and the updated note.

## Parameters — pricing per deal/country (`Parameters` sheet)

All the pricing that historically lived ONLY in the pack (`locales/<code>/pricing.json`) is
**overridable per deal** from the spreadsheet's **`Parámetros`** sheet, so you can set
**different prices per country** without editing code.

**Sheet format (layout v2, "settings page"):** four columns — `A = Parameter`
(human label), `B = Value` (**editable**, highlighted in yellow), `C = What it does` (description
for the user), `D = key` (dotted, machine — do not touch). The tab itself is `Parameters`
(`Parámetros` on anything deployed before v4.24; readers resolve both).

⚠️ **The header text is a CONTRACT, not decoration.** `sheet_overrides()` finds the key and value
columns by reading those headers. Translating them without updating
`proposal_data.PARAMS_KEY_HEADERS` makes the parser fall back to A/B, read the HUMAN LABELS as
technical keys and drop every per-deal override — 24 nonsense keys, zero `deal.*`, no error, a
plausible quote at pack defaults. Done by accident on 2026-09-01; now pinned by
`tests/params_sheet_contract_smoke.py` and backed by a structural fallback (the parser can find the
key column by recognising dotted keys, with no header at all).

**Where the text lives:** `strings.params.*` in the **`en` pack only**. The workbook is an internal
tool, so its chrome is English in every locale and the other packs inherit through the
display-string fallback chain — one source, no four-way drift. `build_params` owns the structure
(order, grouping, which value feeds each row) and raises if a row has no label or description. Grouped into blocks with section
headers (Deal data · Calendar dates · Support and adjustments · Future migration · AI and added
value · Recurring and licensing) + a legend at the bottom. The parser (`sheet_overrides`) **detects the
key column by the `clave` header** and falls back to the legacy layout `A=clave | B=valor | C=etiqueta`
if it can't find it — backward-compatible with older sheets. Prices are **numeric** (dot or
comma decimal); text is only valid in `deal.*` fields (country, locale, dates).

**Supported keys:**
- Flat / nested: `support.salida_pct`, `migration.pct`, `migration.months`, `poc.pct`,
  `adjust_band_pct`, `rates.onsite_day_rate`, `recurring_monthly.mcp`,
  `recurring_monthly.field_service_per_user`, `maintenance_plan.{base_monthly,included_hours,extra_hour}`.
- AI (lists per group/id): `ia_catalog.<group>.<id>.<field>` — e.g.
  `ia_catalog.exclusive.mcp.oneshot`, `ia_catalog.exclusive.mcp.monthly`,
  `ia_catalog.exclusive.starter_ia.oneshot`, `ia_catalog.addons.ocr_nominas.oneshot`,
  `ia_catalog.addons.field_service.{oneshot,monthly_per_user}`.
- Recurring items per id: `recurrent_lines.odoo_license.fallback_amount_per_user_year` (license €/user/year).
  `recurring_monthly.odoo_light_user` (Light User €/user/month) is overridable the same way, but is
  not pre-printed in the sheet template.
- Per deal (non-pricing): `deal.num_usuarios`, `deal.num_usuarios_light` (Light User count, own
  line, Gotcha 12b; not pre-printed, add the row by hand when needed), `deal.pais`, `deal.locale`.
- **Construction vertical (Dynapps Construcción):** `deal.vertical` = `inbuilt` **activates the "Vertical
  construcción" block** in the proposal (its own tile between Developments and AI); `deal.num_usuarios_app` =
  the App's on-site (construction-crew) users (≠ Odoo users, informational); `vertical.inbuilt.pack_oneshot`
  (€6,000, closed price WITHOUT hours, INCLUDES the App) and `vertical.inbuilt.app_oneshot` (€2,500,
  sellable separately). Live bundling: pack checked → the App switches to "Incluida en el pack" (no double
  billing). Checked one-shots add to the TOTAL and to the **future migration** base; **they do NOT add
  hours to the 10% support**. Post-demo selection → `extra_data.vertical_sel = {pack, app}` (the "Copiar selección"
  payload already carries `vertical`). PDF: card block only with what was checked.
- **Dates (deal.*, text dd/mm/yyyy):** `deal.fecha_propuesta` (document date; fallback if
  `extra_data` doesn't carry `fecha`/`fecha_completa`) and `deal.fecha_decision` — the **ANCHOR** of
  the sales calendar: the engine (`_derive_anchor_dates`) derives in a cascade the **team
  assignment** (decision + 3 business days), **kick-off** (1st Monday of the following month), **analysis**
  (kick-off + `semanas_analisis` from Interna Comercial), **implementation** (end of analysis +
  `semanas_impl`), and **support** ("after go-live"), with short localized labels ("25 jul",
  "ago–ene") from the pack's `month_names`. `deal.anchor_{asignacion,kickoff,analisis,
  implantacion,soporte}` (free text) **overrides a specific milestone** without breaking the cascade.
  **Full priority: `extra_data.anchor_dates` > sheet (anchor + overrides) > []** (hidden section).

**How it works:** `proposal_data.sheet_overrides(ss)` reads the sheet → `(pricing_override, deal_params)`;
`build_context` does `L = L.with_price_overrides(pricing_override)` (deep-merge over the pack; the
`ia_catalog`/`recurrent_lines` lists are merged by `id`). From that point on **every `L.price(...)`
call returns the sheet's value**. It's applied BEFORE `set_locale`, so the module-level
globals (`SOPORTE_SALIDA_PCT`) and `extract_costes_recurrentes` honor it too. **No `Parámetros` sheet →
empty override → pack verbatim** (ES byte-parity intact; the cached `Locale` is NEVER mutated: the
override lives in a copy).

**NOT made variable:** Odoo's `product_id` values (they're product references, not prices) and whatever
is already sheet-native (base prices for apps/add-ons in the category sheets; factor/rates in
`Interna Comercial` B4/B5/B6; recurring dyn_cloud/maintenance items in Interna Comercial rows 60-62).

**Templates per country:** `python3 scripts/generate_spreadsheet_template.py [out.json] --locale <code>`
emits the `Parameters` sheet pre-filled with the prices of THAT pack (country). Generating with a
country locale (e.g. `nl-BE`) produces its price table → **one template per country**. The sales rep starts from
their country's template and only tweaks the specifics (e.g. `deal.num_usuarios`).

### Rollout of the `Parameters` sheet to the V2 template in Odoo (per-country adoption)

Updating the code doesn't reach existing quotes: those copy a **base** `sale.order.spreadsheet`
(`order_id=false`) in Odoo. For a country to adopt the editable prices, its V2 base must carry the
`Parámetros` sheet. It's **additive** (only one sheet gets added; the existing 10 are untouched) and,
since it's pre-filled with the pack's prices, it's **numerically a no-op** (same total) — it only makes the
prices visible and editable.

**Real template↔sheet wiring (re-verified against db master 2026-08-05):**
`sale.order.template.spreadsheet_template_id` ("Calculadora de presupuestos") is the field Odoo
actually uses to copy the sheet when a consultant creates a quote from the UI. Current mapping:
- **Implantación Value Based** (id=15) → id=72 "Implantación v2 base (canónica)" (ES general).
- **Implantación Value Based - Construcción** (id=20) → id=76 "Implantación v2 — Construcción
  (canónica)". Consultants can pick it straight from Odoo's quotation-template dropdown.
- **Implantación Value Based - ENGLISH** (id=19) → id=16 "Value based (English)".
- **(ENG) Value based proposal** (id=11) → id=16 "Value based (English)" (older EN entry, same base).
- "Propuesta de Implantación Odoo" (id=1, deprecated legacy) → no sheet (`false`).

⚠️ **The templates were RENAMED and re-pointed at some point after 2026-07-10, and it broke
name-based discovery silently.** The old table above said `Implantación v2` (id=15) → base id=1, and
`Implantación v2 — Construcción` was id=18. Today no template contains "v2" in its name at all, and
id=15 points at base **72**, not 1. Because `odoo_discovery.resolve_template_id()` resolves BY NAME
(exact, then `ilike` fallback) from `instance.json.impl_template_name`, every pack pointing at
"Implantación v2" resolved to NOTHING — an `ilike '%Implantación v2%'` cannot match "Implantación
Value Based". Fixed 2026-08-05 in `locales/es/instance.json` (→ "Implantación Value Based") and
`locales/en/instance.json` (→ "Implantación Value Based - ENGLISH"). `fr`/`nl` still carry the old
Spanish name, which is harmless today only because those packs are stubs blocked from client
output — **fix them as part of promoting either country.**
**Lesson: a rename in Odoo is a silent breaking change for this skill.** `impl_template_id` (15) is
only a documented fallback, not what discovery uses. If a run reports "template not found", check
the real name in Odoo BEFORE assuming the connector is broken.

**Base id=1 "Hoja de implantación base" is now ORPHANED** — no `sale.order.template` points at it
(id=15 moved to 72). It is kept in sync anyway (it carries the 5 free complement slots) so an older
quote copied from it is not left behind, but STEP 3.0 must copy **72** (general) or **76**
(construction), never 1.

**master state (db master, updated 2026-07-10):** the bases carry the `Parámetros` v2 sheet
(nice layout + dates + vertical group): the **UI template** `Hoja de implantación base` (id=1,
the one Odoo copies when creating a quote with template 15 — it used to be the old plain-str format WITHOUT
Parámetros), the ES canonical base `Implantación v2 base (canónica)` (id=72), the EN base
`Value based (English)` (id=16), and the **construction base** `Implantación v2 — Construcción
(canónica)` (**id=76**, `order_id=false`, with `deal.vertical=inbuilt` pre-filled → the
"Dynapps Construcción" block comes out active). The ANRO deal (id=71, PR4021) has a retrofit with
`deal.num_usuarios=60` and `deal.fecha_decision=25/07/2026` (no vertical). **STEP 3.0: copy id=72
for general deals; id=76 for CONSTRUCTION deals** (signals in `references/inbuilt_app.md`:
construction sites, BC3, certifications, subcontractors…).

**Procedure for another country's/operator's Odoo** (each with THEIR OWN connector) — use the helper
`scripts/insert_params_sheet.py` (remaps the styles to the destination workbook, idempotent):
1. Back up the destination base's `spreadsheet_data` to a file.
2. `python3 scripts/insert_params_sheet.py <backup.json> <new.json> --locale <code>
   [--set deal.num_usuarios=N] [--set deal.fecha_decision=dd/mm/aaaa]` — inserts the sheet at
   index 1 (or replaces it if it already exists) WITH its formatting; `--plain` inserts it without styles.
3. Write the `spreadsheet_data` (~55-75 KB, large text field): plain `orm_write` CAN work at this
   size, but it's **not reliable** — depending on payload/encoding overhead it can hit the proxy's
   inline size limit and fail (or, if mis-handled, get written as an attachment value-ref instead
   of the real text — see **Gotcha 16**, never write `{'__ref__':'attachment',...}` into this
   field). If plain `orm_write` fails, fall back to Gotcha 16's recipe (`odoo_request_upload` +
   curl + temporary `ir.cron` that reads `attachment.raw.decode('utf-8')` server-side). Alternative
   with an API key: odoo-pilot RPC. **Either way: mandatory read-back verify** (re-read, `json.loads`,
   compare object equality against the file; if it doesn't match, restore the backup).
4. Verify: sheet count +1, the originals intact, `proposal_data.sheet_overrides` returns a
   non-empty override, and `order_totals.py` gives EXACTLY the same totals as before (no-op).
5. Set that operator's STEP 3.0 to copy that canonical base.

### Productivity rollout (done on db master, 2026-08-26)

`scripts/patch_productivity.py <in.json> <out.json> [--locale <code>]` retrofits a deployed base
so the productivity factor behaves as the management lever of gotcha 19: the `Parámetros` block
(factor + both base rates), B4/B5/B6 rewritten as formulas pointing at it, the support row flipped
(E from prices, F derived) and the "PRODUCTIVIDAD — OBJETIVO DEL EQUIPO" block. Idempotent; a
second run is a byte-identical no-op.

Applied to the three live bases on db master — wired through
`sale.order.template.spreadsheet_template_id`:

| Quotation template | Base sheet | Backup taken |
|---|---|---|
| 15 · Implantación Value Based | 72 | 138 |
| 19 · Implantación Value Based - ENGLISH (and legacy 11) | 16 | 139 |
| 20 · Implantación Value Based - Construcción | 76 | 140 |

Written with the gotcha 16 recipe (`odoo_request_upload` + `curl` + a temporary `ir.cron` doing
`attachment.raw.decode('utf-8')`), then read back, `json.loads`-ed and compared **byte for byte**
against the local patched file. Cron and temporary attachments deleted afterwards, and the deletion
confirmed. `order_totals.py` is unchanged on all three: the engine already computed price-based
support, so this is what makes the SHEET agree with it.

Sheet id 1 ("Hoja de implantación base") is no longer wired to any quotation template and id 111 is
an orphan duplicate — neither was patched. Worth a separate cleanup.

Re-deployed the same day with the reviewed layout ('Productividad' as its own tab, the lever in the
first rows of `Parámetros`, B4/B5/B6 linked and visibly locked). The patcher **self-heals** the
first cut: it removes the in-sheet block and lifts the rates rows to the top, rewriting the links.
Gate before every write: `scripts/verify_productivity.py <book.json> --locale <code>
[--expect-support N]` — 15 checks over STRUCTURE / WIRING / MODEL / RENDER, including the factor
sweep that requires a constant price and falling hours. All three bases plus PR4161's book passed,
and a second patcher pass is byte-identical.

## Test suite index (regression gates)

Run from the skill root. ALL must be green before committing engine/template/pack changes:

| Script | What it guards |
|---|---|
| `tests/materialize_fixtures.py` | rebuilds `tests/fixtures/` before the golden gates (run it first) |
| `python3 tests/golden_check.py` (**bare**) | **the ES byte-parity gate — what CI runs**: 4 fixtures × web/pdf/ctx vs `tests/golden/es/` (12 artifacts). Re-freeze ONLY deliberately via `tests/freeze_golden.py`, after reviewing every diff |
| `python3 tests/golden_check.py es en fr nl` | the same oracle for **every frozen pack** (~48 artifacts). Positionals are classified by name: a `locales/` code is a locale, a `tests/fixtures/` name is a fixture, and they can be mixed. Additive — it never changes what the bare invocation checks |
| `tests/golden_check_brand_lint.py` + `golden_check_helpers.py` | brand_lint output stability + helper CLIs |
| `python3 -m unittest scripts.test_generate_proposal` | 148 engine unit tests (frozen es quirks, pricing, JS contracts, formula robustness) |
| `python3 -m unittest scripts.test_verify_totals` | recurring-totals gate incl. PR4021 regression fixture |
| `tests/locale_smoke.py` | locale resolution + pack loading |
| `tests/composition_smoke.py` | language ⊕ country composition, BE/CH layers, stub gate |
| `tests/process_view_smoke.py` | metro map / process view + `module_view` override |
| `tests/references_smoke.py` | `match_cases` v2 scoring + per-locale case URLs |
| `tests/add_country_smoke.py` | add-country scaffold (throwaway pack, cleaned up) |
| `tests/complements_smoke.py` | add-on candidate detection (`complement_candidates.py`) |
| `tests/qb_run_smoke.py` | the `qb_run.py` chain: step order, exit-code propagation, both gates post-render, `all` refusing `--only pdf` |
| `tests/payload_smoke.py` | `spreadsheet_payload.py` measure/verify, value-ref rejection |
| `tests/scaffold_smoke.py` | `scaffold_extra_data.py` derivation + prose-patch merge + aborting markers |
| `tests/pregate_smoke.py` | `pregate_extra_data.py` pre-render checks: the golden fixtures must PASS, and only what already aborts later (prose marker, calendar dates under a country layer, licence rate over the cap, list-shaped `features`) may FAIL |
| `tests/render_samples.py` | full en/fr/nl renders (stubs must not crash) |
| `scripts/validate_locale.py` | pack schema + mechanical string lint (DASH/MARCA) |
