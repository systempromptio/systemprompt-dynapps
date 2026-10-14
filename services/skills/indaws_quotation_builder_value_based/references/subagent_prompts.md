# Subagent Prompts — STEP 1 and STEP 3

Self-contained prompts for the `Agent(model: sonnet)` calls that run the mechanical steps of the workflow. Copy verbatim, substituting the variables injected between `{}`.

---

## STEP 1 — CRM extraction

**Variables to inject:** `lead_input` (numeric ID or name) and `workdir` (working folder for the files, e.g. `<outputs>/<sale_order_id>`).

> You are the INTAKE subagent (STEP 1). MECHANICAL work: fetch the large fields of an Odoo `crm.lead` (db `master`) via **MCP**, dump them to VERIFIED files with the courier, parse the intake data + chatter, and return ONLY a structured JSON. **The bytes do NOT pass through the orchestrator's context** — you return a manifest + parsed data, never the raw HTML/JSON.
>
> **Connection: MCP-only** (no `auth.sh`/API). Load the tool with `ToolSearch query "select:mcp__88fb5415-35bd-4791-a827-814b99059299__orm_search_read"` and use it for all reads. Courier helper: `scripts/mcp_fetch.py`.
>
> 1. **Resolve the `crm.lead`** from `{lead_input}` (ID or name; if there are duplicates, return the list and stop). Read only metadata: `name`, `partner_id`, `stage_id`, `tag_ids` (do NOT read the intake data here).
> 2. **Resolve `sale.order` + spreadsheet:** `orm_search_read sale.order [["opportunity_id","=",<LEAD_ID>]] ["id","name","state","opportunity_id"]`; then `sale.order.spreadsheet [["order_id","=",<SALE_ORDER_ID>]] ["id"]`. Create `{workdir}` if it doesn't exist.
> 3. **Fetch the 3 large fields to verified files** (courier pattern — the proxy returns them either INLINE or PERSISTED, it's not deterministic; the courier normalizes both):
>    - `spreadsheet_data` (from `sale.order.spreadsheet`) → `{workdir}/spreadsheet.json`. If persisted, the file is the `{model,count,records}` wrapper; extract `records[0]["spreadsheet_data"]` (internal JSON string) and that is the content to save.
>    - `crm.lead.xx_data_collection` → `{workdir}/toma.html`.
>    - `mail.message` for the lead → `{workdir}/chatter.json`. **Paginate by `id asc`** (batches of ~20 with `offset`; NOT `date asc` — with tied dates it duplicates at the boundaries) and **deduplicate by `id`**. Fields: `id,date,author_id,subject,body,message_type`. Bodies of automated mailings can be abbreviated.
>    - PERSISTED → `python3 scripts/mcp_fetch.py save --to <f> --from-host "<path>"`. INLINE → save with Write and `python3 scripts/mcp_fetch.py verify --to <f>`. If `bytes==0` → error.
> 4. **Parse** `toma.html` with `scripts/parse_toma_datos.py` (HTML→JSON) + read `chatter.json` for milestones/decisions. If the intake data is empty → return `{"status":"missing_toma","next":"lead-data-collection"}` and stop.
> 5. **Detect existing quotes** in `existing_orders` (flag `is_v2` based on `sale_order_template_id`).
>
> **Expected output (structured JSON, NOT prose; NEVER the raw content):**
> ```
> {
>   "lead_id": <int>, "lead_name": "<str>", "partner": {...},
>   "manifest": {"spreadsheet":{path,bytes,sha256}, "toma":{...}, "chatter":{...}},
>   "parsed": {sector, cnae, sizing, sistema_actual, apps_detectadas, complementos_sugeridos, pains, presupuesto, desarrollos_mencionados, chatter_highlights, resumen},
>   "existing_orders": [{"id","name","template_id","is_v2": <bool>}],
>   "notas_intake": ["issues: empty/incomplete intake data, contradictions, etc."]
> }
> ```
>
> **Do not do:** pricing decisions, selecting complements, validation with the sales rep, writing to Odoo. Only read, dump to file and return.

**Handling the output:** if it returns `missing_toma` → run `sales-skills:lead-data-collection` and retry STEP 1.

---

## STEP 3 — Fill in the sale.order spreadsheet

**Variables to inject:** `sale_order_id`, `budget_json`, `ana_price`, `qs_base_total`, `extras_total`, `devs_total` (all decisions already closed in STEP 2).

> Mechanical work to write the spreadsheet of a `sale.order` (template "Implantación v2", `sale.order.template` id=15) in the Dynapps corporate Odoo.
>
> **Injected input:**
> - `sale_order_id`: <int>
> - `budget_json`: <JSON with apps, analisis, desarrollos, duracion, formacion, gantt — structure in `references/spreadsheet_filling.md`>
> - `ana_price`, `qs_base_total`, `extras_total`, `devs_total`: <pricing decisions already made by the orchestrator>
>
> **Connection (two paths, same contract):** CREATING the spreadsheet, onchange and binaries require `common-skills:odoo-pilot` (full RPC, `auth.sh`). READING/WRITING the `spreadsheet_data` field is also valid via the generic Odoo MCP (Cowork without `auth.sh`): download with the courier (`scripts/mcp_fetch.py`, keyed by `order_id` — see `workflow_detail.md` §1.1.bis) and upload with `orm_write` followed by a **mandatory read-back verify** (re-read the field, `json.loads()` it, confirm a real `sheets` list — Gotcha 16). If the spreadsheet must be CREATED and you only have MCP → `{"status": "needs_odoo_pilot"}` and stop.
>
> **The ONLY write path for the spreadsheet content is `fill_spreadsheet.py` → upload the COMPLETE resulting JSON. Direct cell edits (orm_write of hand-patched cells) are FORBIDDEN**: they skip duration, training summary, Gantt, style remaps and inactive-app cleanup, and produce a proposal with "0 weeks" and an empty timeline (real ANRO PR4035 failure). Filling `duracion` + `formacion` + `gantt` is part of THIS step's contract — the orchestrator cannot recover it later.
>
> **Steps:**
> 1. Authenticate with `common-skills:odoo-pilot` (`auth.sh`) — or confirm the MCP path above.
> 2. Locate the `sale.order.spreadsheet` with `order_id == sale_order_id`.
>    - If it does NOT exist but the order uses template 15 (created via ORM) → **create it** following `references/workflow_detail.md` §3.0 (create the 2 lines + copy `spreadsheet_data` from a RECENT base in dict format, never the old one in str format). Requires odoo-pilot. Return `spreadsheet_created: true`.
>    - If the order uses another template and you can't create it → `{"status": "wrong_template"}` and stop.
> 3. Download `spreadsheet_data`, write `budget_json` to `/tmp/budget.json`, run `scripts/fill_spreadsheet.py /tmp/current_spreadsheet.json /tmp/budget.json /tmp/filled.json`. **Fix any WARNING it prints on stderr (duracion/formacion missing) before continuing** — those fields are mandatory `> 0` in `budget_json` (`references/spreadsheet_filling.md`).
> 4. Upload the result (`update_record.sh sale.order.spreadsheet`, or `orm_write` + read-back verify on the MCP path). Upload the WHOLE `/tmp/filled.json` — never per-cell patches.
> 5. Calculate: `qs_total_final = round_to_10(qs_base_total × 1.10)`, `adv_total_final = round_to_10((qs_base_total + extras_total) × 1.10)`, `formacion_extra = nº_sesiones_extra × 200`, `custom_total_final = adv_total_final + devs_total + formacion_extra`.
> 6. Update lines: "Análisis" line → `price_unit = ana_price`; "Fase de implantación" line → `price_unit = custom_total_final`. See `references/workflow_detail.md` §3.4.
> 7. **Completeness checklist (mandatory, from the re-downloaded sheet):** in 'Interna Comercial', the analysis-sessions summary (label 'Sesiones análisis…', cell B44 in the standard template) > 0, analysis weeks (B51) > 0 and implementation weeks (B52) > 0; the 'Gantt' sheet has painted cells (≥ 1 bar per active app + analysis + go-live). Report these numbers in the output — totals alone do NOT prove the sheet is complete.
>
> **Expected output:**
> ```
> {"spreadsheet_id": <int>, "spreadsheet_created": <bool>, "qs_total_final": <int>, "adv_total_final": <int>, "custom_total_final": <int>, "ana_price": <int>,
>  "completeness": {"sesiones_analisis": <int>, "semanas_analisis": <int>, "semanas_impl": <int>, "gantt_bars": <int>, "check": "pass|fail"}}
> ```
>
> **Do not do:** draft text, generate the PDF, decide new pricing, modify `budget_json` (it is read-only), edit spreadsheet cells directly.

**STOP REQUIRED before STEP 4 — output handling:**

0. **If `completeness.check` is `fail` (or missing), do NOT present to the sales rep: relaunch STEP 3.** Matching totals do not prove completeness — the ANRO PR4035 sheet had correct totals and an empty Gantt.
1. Present the sales rep with the summary returned by the subagent: pricing (qs/adv/custom/total) + active apps + completeness (sessions/weeks/Gantt bars) + spreadsheet URL in Odoo (build it from the pack's `instance.json` `sales_url_pattern`: `{odoo_url}/odoo/sales/<sale_order_id>`).
2. **Ask the sales rep to open the spreadsheet in Odoo and review it** (active apps, extras, quantities, devs, prices). The sales rep can adjust anything they see directly in Odoo.
3. **Do NOT continue to STEP 4 until the sales rep literally says "ok, go ahead" / "OK" / "validated" or equivalent.** Silence is NOT validation. A response that doesn't explicitly confirm is NOT validation.
4. Once confirmed, STEP 4 downloads the spreadsheet again (it may have changed due to manual editing by the sales rep) before drafting content.
