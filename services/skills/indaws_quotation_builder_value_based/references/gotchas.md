# Gotchas — full detail

`SKILL.md` §Gotchas carries one line per gotcha. This file carries the rest: the incident that
taught the rule, the working recipe, the script that enforces it and the known gaps. **Numbering is
stable** (1-16) — code comments and tests reference it (e.g. `generate_proposal.py` "trampa 15.3",
`test_generate_proposal.py` "regla 15(4)"). Never renumber.

---

## 1. Empty intake data · the adjustment base

- `xx_data_collection` empty or missing → **do NOT parse and do not guess**. Ask the sales
  rep to run `sales-skills:lead-data-collection` first, then retry STEP 1. The STEP 1 subagent
  signals this as `{"status": "missing_toma"}`.
- The **adjustment %** applies over `base + add-ons`, never over the base alone:
  `(base + Σ add-ons × quantity) × (1 + adjustment%)`.
  Enforced by `proposal_data.calc_app_price_split()`; test `TestContext::test_app_price_split`.

## 2. Per-unit add-ons

Multi-warehouse, multi-company and extra pages are priced **× quantity**. Always ask the rep for the
quantity and pass it as `"Name:N"` inside `extras_activos` (e.g. `"Multi-compañía:5"` = 5 additional
companies, i.e. 6 in total). Parsed by `fill_spreadsheet.parse_extra_entry()`; the per-app extras
block totals **by RANGE** (`D20: =SUM(D16:D18)`, per-row `Dx: =Ax*Cx`, `D22: =D3+D20`).

## 3. Spreadsheet not found

An empty `sale.order.spreadsheet` means one of two things: the **wrong template**, or an order
**created via ORM** (the UI's `onchange` does not apply a template on `create`). Fix: create the
lines + the spreadsheet with the recipe in `workflow_detail.md` §3.0 (copy the canonical base — it
already carries the `Parámetros` sheet). **Never hand-craft `spreadsheet_data`.**

## 4. Invoicing vs. Accounting

They are **separate apps**. A client can have Invoicing without Accounting (sole traders / very
small companies). Don't bundle them by reflex; check the intake data.

## 5. Template slot limits

Max **10 analysis sessions** and **5 custom dev items** — a hard slot limit of the workbook
template, not a business rule (`fill_spreadsheet.py`: `sesiones[:10]` plus a blanking loop over
`range(len(sesiones), 10)`). If a deal genuinely needs more, extend the template first.

## 6. Templated `extra_data.json` = rejected proposal

`app_overrides.<APP>.features` (keys = the **exact** spreadsheet feature names) and `dev_overrides`
must carry real client context — number of users, warehouses, revenue, the actual pain points — in
both `intro` and every `feature`. If only `intro` is set and `features` is omitted, the features
render as plain text and the proposal reads generic. Extract the exact keys with the one-liner in
`html_structure.md`; `scripts/scaffold_extra_data.py` derives them for you and leaves
`[[WRITE: … — verify]]` markers that abort generation while prose is missing.

**`dev_overrides` reads `description`, NOT `desc`.** With `desc` the key still matches, so the
localized `title` renders and the row looks finished, while the body silently falls back to
`_get_dev_description()` — the generic catalogue line ("Custom development to cover specific…").
Same field name in the web accordion (`generate_proposal.py:3215`) and in the PDF page (`:4241`).
Cheap check before delivering: grep the render for the catalogue fallback; if it appears once per
dev, you wrote `desc`. (PR3951 EN, 2026-08-26.)

## 7. No all-caps anywhere (v5)

Everything in **sentence case** (first letter capitalized; proper nouns and acronyms like CIF / IVA
/ CRM aside). **Never** use capitals to emphasize, and **no `text-transform: uppercase` in the CSS
either** — the old "eyebrow caps via CSS" doctrine is retired. The brand is always **"Dynapps"**
(capital D) in prose; the all-lowercase form exists only inside the logo artwork.

`brand_lint.py` CAPS/MARCA enforce this **on visible text only**: `strip_html_visible()`
deliberately strips `<style>` before linting, so **the CSS half is NOT gated today**. Treat it as a
human check when editing templates.

## 8. Typography and breathing room

When editing templates, do not enlarge fonts (hero ≤ `clamp(40px,6vw,72px)`, section titles ≤
`clamp(23px,3vw,34px)`) or padding (~64px). Clean and tidy: **tighten before stretching**. Same rule
as `html_structure.md` #14.

## 9. Spreadsheet writes go through odoo-pilot

Creating or writing the spreadsheet **requires odoo-pilot** (RPC + API key). The generic MCP proxy
does not allow `describe_model`, does not run `onchange`, does not serve binaries and cannot create
the spreadsheet — it **can** read and write fields.

Two `spreadsheet_data` cell formats exist: `fill_spreadsheet.py` needs **dict `{content, style}`**
cells; the old plain-`str` workbooks fail. Fix by copying a recent "Hoja de implantación base"
(§3.0), never by rewriting cells by hand.

## 10. Chrome in the Linux sandbox (Cowork)

No Chrome and no root: you cannot `apt-get install` or `playwright install-deps`. `find_chrome()`
already checks `QB_CHROME_BIN` and `PLAYWRIGHT_BROWSERS_PATH` (default `~/pw`). The no-root recipe
(install playwright's chromium into a persistent path, pull the missing `.so` files with
`apt-get download` + `dpkg-deb -x`, wrap it with `LD_LIBRARY_PATH` + `--no-sandbox`) is in
`workflow_detail.md` §4.bis. Without Chrome the script warns
(`⚠️ Chrome no encontrado: PDF no generado.`) and leaves the printable `-pdf.html`.

## 11. Team photos

- **Broken photo (`b'iVBOR...'`)** — a base64 string carrying a Python bytes repr renders as a
  broken image; it must be clean ASCII. `fetch_team.py` and the renderer already sanitize it
  (`TestNewMejoras::test_photo_clean_strips_bytes_wrapper`). The **source of truth is
  `extra_data.equipo`**: re-inject after fixing anything.
- **Photo via MCP (no API key)** — the field itself is stripped, but the image is a field-backed
  `ir.attachment` row:
  `orm_search_read('ir.attachment', [res_model='res.partner'|'hr.employee', res_id=<id>, res_field='image_512'])`
  → `attachment_id` → `odoo_attachment_link(attachment_id)` → presigned URL → `curl` →
  `scripts/embed_team_photos.py`. The `odoo_attachment_link(model, res_id, field)` form **IS
  blocked** for `hr.employee`/`res.partner` (proxy model allowlist); by `attachment_id` it is not.
  Alternative: `fetch_team.py` (odoo-pilot RPC + API key). Full recipe and the standing feature
  request to the MCP team: `references/mcp_photo_access.md`.

## 12. Odoo license is a PER-USER rate — never inject the total (PR4021)

The Odoo License card carries the **per-user/year rate** and the engine multiplies it by
`num_usuarios` (live selector on the web, scaled in the PDF). The **reference rate** for proposals
is **Custom (Personalizado), yearly billing, list price: 334.80 €/user·year (27.90 €/month)**, the
`es`/`en` pack fallback. **Standard 178.80 €/user·year (14.90 €/month)** is the sanctioned
alternative. Never use Odoo's first-year discount (Standard 11.90 · Custom 22.90 €/month): better to
overestimate than to promise a discount that does not hold in year two. Monthly billing (Custom
33.90 · Standard 18.60 €/month) is not the reference either.

**NEVER inject the already-multiplied total into `price`.** 60 × 334.80 = 20,088; the engine
multiplies again → 20,088 × 60 = **1,205,280**. That is the PR4021 bug class.

Correct shape:

```json
{"title":"Licencia Odoo","id":"odoo_license","amount_per_user":334.80,
 "price":"334,80 € / año (27,90 € / mes)"}
```

plus `"num_usuarios": 60` → the engine renders €20,088/year (334.80 €/user·year × 60).

Guards: `generate_proposal.py` warns on stderr when the resolved per-user rate exceeds €500 (only
fires with more than 1 user), and `verify_totals.py` **hard-FAILS** on it
(`LICENCE_RATE_HARD_CAP = 500.0`, `STANDARD_LICENCE_RATES = (334.80, 178.80)`). If the user
count is `[ASK]`, **ask before generating**.

### 12b. Light User — its own line, its own count

Odoo's **Light User** licence (**4.90 €/user/month**, same price billed monthly or yearly =
**58.80 €/user·year**) is for employees with **no access to the core apps** (Sales, Purchases,
Projects, Accounting): HR record and limited employee self-service only. It is contracted
**through an Odoo Advisor**, and only the Spanish price is verified.

- Set **`num_usuarios_light`** (value_anadido flag > extra_data top level > `Parámetros`
  `deal.num_usuarios_light`, the same chain as `num_usuarios`). The engine adds an **annual fixed
  card** "Licencia Odoo · Light User" = rate × 12 × n (web + PDF, inside the annual fee). The
  `#recur-users` selector **never rescales it**. 0 (default) = no line anywhere.
- Rate from the pack: `pricing.recurring_monthly.odoo_light_user` (`es`/`en` 4.9; `fr`/`nl` `-1` =
  unverified). With `-1`/missing the engine **omits the line and warns on stderr**, never renders
  `-1`. `validate_locale.py` blocks `-1` in a live pack.
- **Never write it by hand in `costes_recurrentes`.** A line with id `odoo_license_light` or "light"
  in the title is NOT the Odoo licence (never × `num_usuarios`); it renders as a plain fixed card and
  the engine warns to use `num_usuarios_light` instead (both at once = charged twice).
- `verify_totals.py` expects the Light line exactly when it should render and FAILs if its total
  ≠ rate × 12 × n.

### 12c. odoo.com prices are geo-IP — never "verify" another country from here

The pricing page (`odoo.com/<lang>/pricing-configurator`) sets the pricelist by the **visitor's
IP**, not by the locale in the URL. From Spain, `fr_FR`, `nl_BE` or `en_US` all return the **ES
pricelist**. A figure read that way is the Spanish price with another language on top: it does not
verify `fr`, `nl`, BE or CH. Those packs keep the `-1` sentinel until someone confirms the price
from that market (that country's Odoo, a local Odoo Advisor or a visitor in that country).

## 13. Cropped team photo (not square) — known gap

`hr.employee`'s `image_512`/`image_256` usually comes portrait (e.g. 341×512); with
`object-fit:cover` in the render's circle the face gets cropped. **Always normalize** each photo to
a centred square with padding (neutral light background, e.g. `#EEEEF2`, resized to 512×512) before
injecting it into `extra_data.equipo` — `scripts/embed_team_photos.py` does it.

**This is a known unimplemented gap in `fetch_team.py`**, not a description of current behaviour: it
should do it by default and does not.

## 14. Text that sounds like AI = rejected proposal

The written content (`understanding`, `executive_summary_html`, `cost_of_delay`, `app_overrides`,
`dev_overrides`) has to read as human.

- No filler words ("empower", "leverage", "revolutionize", "a wide range of"…).
- No filler intros/outros ("in an ever-changing world…", "in conclusion").
- No **4+ consecutive sentences of the same length** (the robotic drone effect).
- Active voice, varied sentence length, the "café test".
- Talk about the client **2:1**; if any competitor could sign the sentence, delete it.

`brand_lint.py` flags these as **SLOP / FLUFF / RITMO** (literal tags; RITMO via `_monotony_run()`).
Fix the warnings **before** generating. Full writing rule: `html_structure.md` #15.

## 15. Multi-language traps

1. **`--locale` moves only the chrome.** It localizes the static strings and the apps'
   `display_name`; the **prose inside `extra_data` is per language**. Keep one `extra_<lang>.json`
   per language, each with a top-level `extra_data.locale` so it renders without a CLI flag. Lint in
   the right language (`brand_lint.py --locale es`, or bare for `en`).
2. **KEYS stay in Spanish.** `app_overrides.<app>` keys, `app_overrides.features` keys,
   `dev_overrides` keys, `data-app` / `data-app-name` attributes, checkbox `value`s, `req_map`,
   `COND_VARIANTS` and the copy-selection payload keys are the **machine contract** — the template
   JS and the spreadsheet matching branch on those Spanish literals. Only visible text is localized
   (`display_name`, `dev_overrides[k].title`). This is the highest-risk trap in the whole skill.
3. **Localized feature titles (gap closed in v5).** `app_overrides.features` values accept
   `{"title": "<localized bullet name>", "expl": "<explanation>"}` besides the legacy plain string.
   For non-`es` proposals **always use the object form** (the key stays the Spanish spreadsheet
   name; `title` is what the client sees). With the legacy string form the bullet name still reads
   the Spanish key. `brand_lint` IDIOMA flags leftovers.
4. **License is per user.** `extra_data.num_usuarios` + a per-user figure in `costes_recurrentes`
   (see Gotcha 12). The recurring block **does not render `desc`** — it builds its own subtitle, so
   any figure that must be visible has to live in a rendered field.
5. **Non-`es` recurring cards need a numeric `amount`.** `_recur_amount` falls back to
   `_parse_amount`, an **ES-format** regex ("." thousands, "," decimals). The `en` pack's own
   `_recurrent_defaults` emits `€1,080.00 / year` → **1.08**, `€320.00 / month` → **32,000** and
   `€334.80 / year (€27.90 / month)` → **33,480**, so copying the pack defaults straight into
   `costes_recurrentes` poisons the annual/monthly totals and the live user-selector recalc. Set
   `amount` on every card (`amount_per_user` on the license, Gotcha 12) and let the price string be
   cosmetic. `verify_totals.py` catches it: "initial annual total from the rendered rows = X but the
   engine expects Y".
6. **The "With Odoo" answer matches SPANISH substrings.** `PAIN_SOLUTIONS` keys on `consolidac`,
   `trazab`, `factura`, `client`, `isla`, `excel`…, so English `pain_points` fall through to
   `pain_sol_default` and **every** challenge renders the same generic answer — the visible bug in
   the PR3951 EN build of 2026-07. Write each English pain so it carries a token that also matches
   (`intercompany`, `stock`, `SII`/`verifactu`, `client`, `legacy`, `excel`), and remember the
   **first rule in list order wins** ("manual" hits `excel` before `stock`). Read the four rendered
   pairs before delivering.
7. **The hero's sector line and the companies count come ONLY from the toma.** Neither has an
   `extra_data` override (`ctx["sector"] = info.get("sector", "")`), so a Spanish intake puts a
   Spanish sentence in the hero of an English proposal. `n_sociedades` regexes `(\d+)\s*socied`
   and `multi_empresa` tests for the accented `"sí"`, so an unaccented intake ("SI — Davis España +
   futura filial Portugal") renders **1 company** and leaves `multi_empresa` False. Fix by
   rendering from a translated COPY of the toma — original untouched, only those cells corrected —
   and say so when handing the file over.

Golden gate: `python3 tests/golden_check.py` **bare** is the ES byte-parity gate (what CI runs);
`python3 tests/golden_check.py es en fr nl` checks all four packs. Never re-freeze to make a diff
go away.

## 16. Big Text-field writes — never use the attachment value-ref

`spreadsheet_data` (and any other big **Text** field, ~55-75 KB) is `ttype='text'`, **not**
Binary/Image. The generic MCP's `{'__ref__':'attachment','id':N}` value-ref convention is for
Binary/Image fields **ONLY**. Writing it into a Text field succeeds with no error and **silently
corrupts** the record: it stores the literal dict-repr as the field's text. The failure surfaces
client-side minutes or hours later as an opaque `OwlError` /
`SyntaxError: Expected property name or '}' in JSON`. Both `spreadsheet_payload.normalize()` and
`compact_dumps()` reject a value-ref, in its JSON form and in its Python-repr form.

**Choose the write path by measurement, not intuition:**
`python3 scripts/spreadsheet_payload.py measure <workbook.json> --payload <out.txt>` returns
`write_path` = `orm_write` (inline fast path) or `ir.cron`. See `spreadsheet_filling.md` §Write path.

**When it does not fit inline — the 7-step recipe:**

1. `odoo_request_upload` **with no pre-target**.
2. `curl` the **RAW text** content (the real JSON string, not a wrapper) to the presigned URL →
   `attachment_id`.
3. Create a **temporary** `ir.cron` (confirm `model_id` for the target model) whose code does
   `attachment = env['ir.attachment'].browse(<attachment_id>); env['<model>'].browse(<record_id>).write({'<field>': attachment.raw.decode('utf-8')})`
   — this is what actually moves the bytes into the Text field, server-side.
4. Trigger it with an `ir.cron.trigger` record (`method_direct_trigger` and
   `ir.actions.server.run()` are **both blocked** by this proxy).
5. Poll until `lastcall` is populated.
6. **Mandatory read-back verify** — re-read the field, `json.loads()` it, confirm real structure
   (e.g. a `sheets` list). `python3 scripts/spreadsheet_payload.py verify <payload.txt> <readback.json>`
   does the deep-equality comparison.
7. Clean up the temporary `ir.cron` (its trigger cascades) and the temporary `ir.attachment`(s).

**Always do the read-back + `json.loads()` verify after ANY write to this field**, regardless of
which method wrote it — plain `orm_write` included. See `workflow_detail.md` §Parámetros Rollout
step 3.

---

## 17. The deal's spreadsheet silently replaced by the canonical base

Deploying "Implantación v2 base (canónica)" onto an existing order creates a **new**
`sale.order.spreadsheet` and the deal's numbers are gone. Nothing to recover: the new record has
`spreadsheet_revision_ids: []`, `current_revision_uuid: "START_REVISION"` and an empty
`spreadsheet_snapshot`. Happened to PR3951 (Davis) on 2026-08-05, found on 2026-08-26 when the
proposal was regenerated.

**The tell:** `order_totals.py` returns an `apps_total` equal to the sum of **every** base price in
the catalogue (16,250 € for the 13 apps) with `devs_total: 0`, and the completeness gate fires on
B44/B51/B52 = 0 plus an unpainted Gantt. Matching totals never prove the sheet is *this deal's*
sheet — check that the ACTIVE apps are the ones the deal scoped, not all of them.

**Rebuilding from the last delivered HTML.** It is the only surviving record of the budget and it
carries everything needed: per-app closed prices, the extras (per-app price − base price), the dev
list with amounts, `num_usuarios` (license annual total ÷ per-user rate) and the Gantt (each bar's
`left`/`width` % over the axis = start week and weeks per app; `duracion.semanas_implantacion`
includes go-live, which `phase_flow_html` splits back out as `impl − golive_weeks`). Reconstruct
`budget_json`, run `fill_spreadsheet.py`, and require the rebuild to reproduce the delivered
`ana_price` / `apps_total` / `soporte` / `devs_total` **exactly** before generating anything.

**So it never costs that again:** archive `extra_data.json` next to every delivered file in STEP 5.
The reconstruction above took a full session precisely because PR3951's was never archived.

## 18. Sheet references in formulas must ALWAYS be quoted

`generate_spreadsheet_template.py` built cross-sheet references with
`safe = f"'{sn}'" if " " in sn else sn` — it quoted by **space**, not by need. Since v4.19 the
workbook groups apps by end-to-end process and three of those tabs are **hyphenated**:
`Order-to-Cash`, `Procure-to-Pay`, `Record-to-Report`. Bare, o-spreadsheet parses the hyphen as
**subtraction**: `=Order-to-Cash!D30` reads as `Order - to - Cash!D30` → `#BAD_EXPR`.

`Personas y soporte`, `Config General` and `Análisis Funcional` survived on their space alone —
which is why **Proyecto was the only app showing a number** and the bug read as a one-off.

**Blast radius.** 10 of the 14 apps in `app_to_sheet_group` land on a hyphenated tab (CRM, Ventas,
Facturación, Helpdesk, Website, Marketing → O2C; Compras, Inventario, Fabricación → P2P;
Contabilidad → R2R). In 'Interna Comercial' that takes out Base, Extras, Total App and Horas int.
per app, and in cascade the 10% go-live support (E36), Subtotal Implantación (E38), TOTAL PROYECTO
(E40) and the duration block (C51-C53, D51-D53).

**Why nobody saw it for weeks.** The proposal is right. `proposal_data.py` reads the app tabs
directly (`_sheet`/`_gc`) and recomputes every number in Python — it never evaluates a formula. So
the HTML and the PDF are correct while the workbook the rep co-signs in Odoo is `#BAD_EXPR` from
top to bottom. The STEP 3 completeness gate does not catch it either: B44/B51/B52 are literal
numbers, so they pass.

**Fix:** `sheet_ref(name)` quotes unconditionally and doubles internal single quotes, Excel-style.
Quoting is valid for any name and also covers `+`, parentheses and accents. Gate:
`tests/sheet_ref_quoting_smoke.py` (proven red by reverting the call site, not the helper — the
helper-and-all revert only produces an ImportError, which tests nothing).

**Already-deployed workbooks.** Canonical bases 1 / 16 / 72 / 76 were repaired in the DATA on
2026-08-05 17:34 — the code was not, so any regeneration would have reintroduced it. Workbooks
created **before** that repair still carry the broken formulas (found on PR4086, Benítez Bittar,
sheet created 2026-07-29). Repair them **in place**: never redeploy the base onto a live order
(gotcha 17 — it wipes the deal's sheet). `spreadsheet_data` is a non-stored computed field, so it
cannot be searched in SQL either; audit and repair server-side with a temporary `ir.cron`
(`model_id` 3403), replacing `"=<Tab>!` with `"='<Tab>'!` in the raw JSON text — single quotes need
no JSON escaping, so the whole repair is a string replace. Then **read back and `json.loads()`**
(gotcha 16).

**The golden defended the bug.** `tests/golden/es/helpers/template.norm.json` had the bare
formulas frozen in, so `golden_check_helpers.py` failed on the *fix* — a byte-parity oracle can
only pin whatever the engine did when it was frozen, bug included. Re-frozen with the fix: 20
formulas, +40 lines, quotes only, nothing else moved. Leftover to be aware of:
`tests/fixtures/helpers/template_raw.json` is a frozen *input* fixture and still carries the bare
formulas — harmless (`fill_spreadsheet` never touches cross-sheet refs, which is why `filled.json`
kept passing), but do not read it as a reference for correct output.

## 19. The productivity factor never touches the price

`rates.productivity_factor` (`Parámetros` sheet, 0.92 by default) is the lever **management** uses
to decide how much of the work product and AI absorb instead of consulting. The contract is one line:

> lowering it cuts the **internal hours** and raises the **effective rate**
> (`base_rate ÷ factor`); the **price does not move**.

The productivity gain is **Dynapps margin**. Never a discount to the client, and never a shorter
schedule either: weeks are computed from UNFACTORED hours (`timeline_rules.md`).

**What was broken.** The "Salida a producción" line was 10 % of the **hours**, and the hours carry
the factor. So lowering the factor lowered the price. Measured on PR4086, 0.92 -> 0.70 discounted
**203 €** (-2.9 % of the implementation, roughly 9 € per hundredth) — the exact opposite of what the
lever is for. The line now comes from the **price** (`10 % × (analysis + Config + apps + checked
devs)`) and is invariant; its own hours do follow the factor. In the sheet the order flips:
**E36 (€) leads and F36 (hours) is derived**.

**The override sat there inert, and silent.** `sheet_overrides()` picked up
`rates.productivity_factor` from `Parámetros`, but `soporte_rates()` read only
`Interna Comercial!B4`: management could write 0.70 and the engine kept computing with 0.92 without
a word. Priority is now **a LITERAL in B4/B5/B6 > `Parámetros` > pack defaults**, and new workbooks emit
B4/B5/B6 as FORMULAS pointing at `Parámetros`, so management governs by default. The literal wins
when present because typing a number into the cell is a deliberate act — ignoring it would be the
same silent-override bug mirrored. Legacy workbooks (literals, no `Parámetros` sheet) keep working
unchanged, which is also why the deployment patcher must convert B4/B5/B6 to formulas: otherwise
the old literal would keep beating management's value.

**IA & Product Value.** `price × (1 − factor)`: the value delivered without consulting hours. At
0.92 that is 8 % of the project (810 € on PR4086). It lives on its **own 'Productividad' tab**,
grouped as RATES (what the team must reach) / HOURS (the capacity budget) / WHAT PRODUCT AND AI
CONTRIBUTE, each row explaining itself in column C. Everything on it is derived by formula from the
TOTAL row of `Interna Comercial`, so it can never disagree with the rest of the workbook.
**Internal only**: not shown to the client, not written to any `sale.order` field (management
decision, 2026-08-26).

**Where each thing lives, and why.** The lever (`rates.productivity_factor` + both base rates) is
the **FIRST group of `Parámetros`**, right under the header — buried under the footnotes it may as
well not exist. `Interna Comercial` B4/B5/B6 stay, because B4 feeds 21 formulas and B5 feeds 20 (the
hours of every line and the go-live support), but they are **formulas** pointing at `Parámetros`,
carry **style 18 (linked, not editable)** and a note sending the reader there. That styling is not
cosmetic: the read priority is LITERAL > Parámetros, so a cell that invites typing is a cell that
silently beats management.

**Gate:** `tests/productivity_factor_smoke.py` sweeps the factor from 1.00 down to 0.70 and requires
a constant price, strictly decreasing hours, `effective_rate == base/factor`, and `Parámetros`
beating B4. Proven red by putting `calc_soporte_salida` back on an hours base: the price slides
14,775 -> 14,535 and 10 assertions fire.

**When deploying:** the three Odoo base workbooks (72 ES · 16 EN · 76 Construction, wired through
`sale.order.template.spreadsheet_template_id`) must be patched so they emit the new formulas.
Quotes already created are NOT recalculated — they keep their price.

## 20. Readers must resolve the workbook by ALIAS, never by a literal or by the active locale

**The wb16 incident (2026-09-01).** `Value based (English)` (`sale.order.spreadsheet` id 16) is a
real deployed base whose chrome is English. Read by the same engine that reads the Spanish bases it
returned, with no error at all:

| | wb72 (ES base) | wb16 (EN base) |
|---|---|---|
| a 4.500 € development typed by the rep | `[{'name': ..., 'price': 4500}]` | `[]` |
| `calc_analisis_price` | `800` | `0` |

The quote just came out cheaper. Same failure class as ANRO PR4035.

**Two independent causes, both "match the workbook against a Spanish literal":**

1. `calc_dev_prices` located the block by `"Funcionalidades Adicionales"`; the English workbook says
   `Additional Features`, so the section was never found and every custom development was dropped.
   Its row filter was worse: `"salida" not in name.lower()` was meant to skip the go-live line, and
   `Go-live support` walks straight through it.
2. `calc_analisis_price` / `get_analisis_sessions` asked for the sheet `"Análisis Funcional"`. That
   tab went English in v4.24 — so from that commit on the analysis price read **0 in every locale**,
   ES included, on every newly generated workbook. Only workbooks deployed earlier still worked.

**The rule.** A reader is handed a WORKBOOK, not a locale. The deal may be `es` while the workbook
came off the English base (an ES base worked by a Belgian consultant is the same situation), so
resolving by `L.s(...)` looks up the wrong string and — every call site degrading gracefully — fails
silently. Resolve by ALIAS: `INTERNA_SHEET_NAMES`, `PARAMS_SHEET_NAMES`, `ANALISIS_SHEET_NAMES`,
`PRODUCTIVITY_SHEET_NAMES`, `ADDITIONAL_FEATURES_LABELS`, `SUBTOTAL_EXTRAS_LABELS`,
`CAT_LABEL_TO_KEY`.

**Those tuples are built from the packs on disk**, not typed out by hand: the union over
`locales/*/strings.json` IS the alias list, so adding a locale widens every reader for free. Only
LEGACY names — the ones frozen inside already-deployed workbooks and emitted by no pack any more —
are listed explicitly, and `validate_locale.check_sheet_tab_aliases` fails if one of them is ever
dropped (proven red by removing `Análisis Funcional`: 4 errors, one per pack).

**Prefer structure over text where structure exists.** A development slot holds a LITERAL price the
rep typed; the go-live / subtotal / total rows that close the block hold FORMULAS. That distinction
is language-free and needs no alias at all. Text matching stays only as the second layer, because
the minimal fixtures in `tests/fixtures/` shorten the labels ("Subtotal") and word order moves
between languages ("Subtotal Análisis" vs "Analysis Subtotal") — no first-word rule survives that.

**Gate:** `tests/workbook_language_agnostic_smoke.py` generates the same workbook in every pack,
injects the identical development and analysis sessions into each, and requires the readers to
AGREE — value for value, not merely "does not crash". It is built from the live packs rather than
from a frozen fixture on purpose: a fixture pins the chrome of the day it was captured, so a pack
that renames a label would ship green and break in production. Proven red before the fix: 11
assertions, including `en: calc_dev_prices -> []`.

## 21. The parameters sheet's column HEADERS are a parser contract

`sheet_overrides()` does not know where the technical keys are: it finds the key and value
columns by reading their HEADER text. That header is emitted from the pack (`strings.params.hdr`)
and matched in `proposal_data.PARAMS_KEY_HEADERS` — one contract living in two files, with nothing
but `tests/params_sheet_contract_smoke.py` holding them together.

Translating the header from `clave` to `key` without touching the parser was done on 2026-09-01
while localising the sheet, and produced, in silence:

```
override keys : ['AI Starter Pack (€ one-shot)', 'Analysis rate (€/h base)', ... 24 of them]
deal keys     : (none)
```

The parser had fallen back to A/B and read the HUMAN LABELS as technical keys. Every per-deal
parameter — rates, productivity factor, licence, OCR add-ons, the whole date anchor cascade — was
gone, the quote silently reverted to pack defaults, and nothing was raised.

**Three detection layers now, in order.** (1) By header text, across every pack plus the legacy
`clave`/`valor`. (2) **Structurally**: the column that actually contains dotted technical keys
(`^[a-z][a-z0-9_]*(\.[a-z0-9_]+)+$`), which needs no header at all and is the layer that would have
caught this on its own. (3) A/B, the pre-v2 `clave | valor | etiqueta` layout. All three are
exercised by the smoke, including with a header no pack has ever emitted.

**If you change `params.hdr`,** run `python3 tests/params_sheet_contract_smoke.py`. Sabotage-proven:
putting the parser back on the `clave` literal and killing the structural layer turns 14 assertions
red.

**Where the text lives.** `strings.params.*` is in the **`en` pack only** — the workbook is an
internal tool, so its chrome is English in every locale and the other packs inherit it through the
display-string fallback chain. `INHERITED_NAMESPACES` in `validate_locale.py` exempts it from
key-parity; a country layer can still override a single label. `build_params` keeps the structure
(order, grouping, which value feeds each row) and raises if a row is missing its label or
description; `check_params_block` catches the half-edit at lint time instead.

## 22. Workbook CHROME is inherited English; the client-facing strings are not

The workbook never reaches a client, so all of its chrome — tab names, column headers, row
labels, hints, the whole parameters page — is **English in every locale**. `strings.sheet.*` and
`strings.params.*` live in the **`en` pack alone** and every other pack inherits them through the
display-string fallback chain (`Locale.s` → fallback → `en`). That removed ~240 duplicated strings
and, with them, four chances to drift.

**But one `sheet.*` key is not chrome.** `sheet.formacion_line` (`"{n} sesiones formación —
{name}"`) renders a base feature that the CLIENT reads. Letting `es` inherit it produced, in a
Spanish proposal:

```
base_features[formaciones] = ['2 training sessions — Equipo comercial']
```

Spanglish, to the customer — and the **ES byte-parity gate stayed green throughout**, because its
fixtures are minimal hand-built workbooks that carry none of the generated training rows. So `es`,
`fr` and `nl` each keep their own `formacion_line` and nothing else. The list of such keys is
`CLIENT_FACING_SHEET_KEYS` in `tests/workbook_chrome_boundary_smoke.py`; add to it the moment a new
sheet string starts reaching client output.

**A NOTE ON THE FIRST VERSION OF THAT TEST**, because the mistake is easy to repeat: it rendered
the workbook in a locale and checked the training lines against *that pack's own* template. With
the key inherited, pack and workbook were both English — so they matched, and the sabotage sailed
through. A test that asks a thing to agree with itself proves nothing. It now checks the pack AS
WRITTEN (`raw_pack`, no fallback chain): the client-facing keys must be declared locally, the
chrome must not be, and the rendered lines must actually differ from English. Both directions are
sabotage-proven.

**Guard rails, and which one owns what.** `INHERITED_NAMESPACES` in `validate_locale.py` exempts
the two namespaces from key-parity, and compares a partially-declared namespace only on the keys
the pack actually declares. Because the sub-schemas can no longer require keys of every pack, that
demand moved to the reference: `x-reference-required` in the schema, enforced by
`check_reference_carries_inherited`. One hole is deliberate — an EXTRA key in an inherited
namespace is not a parity error; the boundary smoke catches it as stray chrome instead.

## 23. Migrating a deployed workbook: the rename is the easy half

`scripts/migrate_workbook_chrome.py` brings a workbook that is already live in Odoo to English
chrome. The bar is not "it ran": it is that the quote engine reads the SAME numbers, the SAME
developments and the SAME client-facing features out of it afterwards. Verified on all four
canonical bases (1 · 16 · 72 · 76) pulled from Odoo — identical engine reads, no dangling
references — and pinned by `tests/migrate_chrome_smoke.py`.

**Four things that a naive rename gets wrong, all of them silent:**

1. **The 36 cross-sheet formulas.** `='Parámetros'!B4` does not follow a tab rename in a raw JSON
   rewrite. o-spreadsheet shows `#REF` and the quote reads as zero.
2. **The tab name inside PROSE.** "Factor and rates are edited on the 'Parámetros' sheet" is a
   cross-reference too, just not one a formula scanner sees. Rewriting these BEFORE the chrome
   table also does a second job: strings that differ from the pack only by that name then match
   exactly, which is what fixed the Productivity descriptions in one move instead of a lookup
   table.
3. **`Concepto`.** The old pack rendered it as `Concept` on Internal and `Item` on Productivity.
   Globally ambiguous, so it is resolved per sheet or left alone — never guessed.
4. **The parameters GROUP headers** ("Datos del deal", "Fechas del calendario", …) were hardcoded
   in the old generator and appear in NO pack, so no chrome table can reach them. Mapped
   explicitly, old label → group id.

**The parameters sheet is migrated by TECHNICAL KEY, not by text.** Column D is the row's identity,
so label and description are rewritten from the pack with nothing to mis-hit. A key the pack does
not recognise is left exactly as it is and reported in the manifest — an unknown key is either an
old parameter or one somebody added by hand, and guessing at it is how a migration quietly deletes
someone's work.

**What it must never touch:** app names, feature names, development names and the training lines.
Those are client-facing and stay in the deal's language (gotcha 22). The audit in the smoke
excludes them explicitly rather than letting a broad "is this Spanish?" rule decide.

**It is idempotent** — a second pass reports zero changes — so a partially-applied migration can
simply be run again.

## 24. The Gantt sheet is ES-authored BY DESIGN — and `get_duration` reads Spanish

Two client-facing defects found by A/B-ing a real deal (PR4035 / ANRO, 14 apps, 4 developments,
the pre-v4.19 layout) through old code + original workbook against new code + migrated workbook.
Neither was caught by the byte-parity gate, the locale validator or 18 green smokes — only by
rendering the actual proposal and diffing it.

**1. The Gantt sheet must never be translated.** Column A is ES-authored on purpose and doubles as
the LOOKUP KEY of `generate_proposal._localize_gantt_label`, which translates it per locale on the
way into the proposal. The chrome migration translated it, so base 72 — the base every Spanish
quote copies — rendered a Spanish client timeline as *"Functional Analysis / Custom developments /
Go-live"*. Same reason `sheet.gantt_template_rows` (which authors those labels) and
`sheet.gantt_row_labels` (the ES→local table) are per-pack, never inherited. The migrator now skips
the sheet outright: `UNTRANSLATABLE_SHEETS`.

**2. `get_duration` matched two Spanish literals** — and printed `0` when it missed, which the
client reads as *"0 semanas de proyecto"* in the hero and in the timeline. It survived the first
alias sweep because the literals were written ESCAPED (`"Análisis"`), invisible to a grep of
the accented form. **When sweeping for hardcoded strings, grep the escaped form too** — better,
walk the AST for comparisons against string constants, which is how it was finally found.

**The deeper lesson is about the probe, not the code.** The first A/B compared seven readers and
came back byte-identical, which read as proof. It was not: `get_duration` and `extract_gantt_data`
were simply not in it, and a probe silently proves nothing about what it does not call. Rendering
the whole client HTML is what exposed both, because it exercises every reader whether or not you
remembered it existed. **A/B the deliverable, not a list of functions you thought of.**

Both are pinned now: `get_duration` in `workbook_language_agnostic_smoke.py` (sabotage → 4 red),
the Gantt exclusion in `migrate_chrome_smoke.py` (sabotage → 1 red, naming the three cells).
