# Spreadsheet Filling — JSON Structure and Rules

## Structure of the quote's data JSON

Input expected by `fill_spreadsheet.py` as its second argument:

```json
{
  "apps": {
    "crm": {
      "activa": true,
      "extras_activos": ["4-10 comerciales", "Migración oportunidades"],
      "notas": "5 comerciales. Ciclo corto."
    },
    "fabricacion": {
      "activa": false,
      "notas": "App propia — integración API."
    }
  },
  "analisis": {
    "sesiones": [
      {"area": "Config + CRM", "num": 1, "descripcion": "Dirección + comercial"},
      {"area": "Ventas + Compras", "num": 2, "descripcion": "Administración"}
    ]
  },
  "desarrollos": [
    {"nombre": "Integración API fabricación", "precio": 2625, "notas": "3 endpoints"},
    {"nombre": "Conector Shopify", "precio": 1500, "notas": "Solo facturación"}
  ],
  "duracion": {
    "semanas_analisis": 7,
    "semanas_implantacion": 19
  },
  "formacion": {
    "sesiones_analisis": 9,
    "sesiones_extra": 0
  },
  "gantt": {
    "semanas_por_app": {"config_general": 1, "ventas": 2}
  }
}
```

## Construction rules

| Field | Rule |
|-------|-------|
| `apps.{id}.activa` | `false` → inactive app (left at 0 in the spreadsheet) |
| `apps.{id}.extras_activos` | List of extra names (matched by substring against the template) |
| `analisis.sesiones` | **Maximum 10** (template slots) |
| `desarrollos` | **Maximum 5** (template slots) |
| `duracion` | **MANDATORY, both keys > 0** (`semanas_analisis`, `semanas_implantacion`). Weeks used for the price/month calculation and the "(skill rellena)" cells of 'Interna Comercial' (B51/B52). If omitted, the script writes **0 without error** (it only warns on stderr) and the proposal shows "0 weeks" |
| `formacion` | **MANDATORY: `sesiones_analisis` > 0** (must equal the number of `analisis.sesiones`); `sesiones_extra` may be 0. Feeds the 'Interna Comercial' summary (B44/B46). Same silent-zero risk as `duracion` |
| `gantt.semanas_por_app` | Weeks per app for the Gantt. If not specified, automatic defaults apply |

## What the script modifies

| Sheet | What gets modified |
|------|-----------------|
| App sheets (Config General, Comercial, etc.) | 1/0 flags on extras based on `extras_activos` |
| Análisis Funcional | Sessions (area, no., description). Price auto-calculated. Empty slots removed |
| Interna Comercial | Developments (name, price, notes), duration in weeks, training sessions, notes per app |
| Gantt | Bars per task: Analysis, 1 row per active app, 1 row per development, go-live |

## What it cleans up

| What it removes | Detail |
|-------------|---------|
| Inactive app sections | If `activa: false` → removes base + extras + total |
| Empty category sheets | If all apps in a category are inactive |
| Inactive app rows in Interna | Removes the row and rebuilds the subtotal and total formulas |
| Empty development slots | If there are fewer than 5 developments, removes the leftover ones |

## Formula reconstruction

After cleanup, the script rebuilds:
- **Go-live (10% of hours)** → active apps + devs only
- **Implementation subtotal** → active apps + devs + go-live
- **Project total** → Analysis + Implementation subtotal (no circular reference)

## Write path — uploading `filled.json` back into `spreadsheet_data`

The **only** write path for the sheet's content is `fill_spreadsheet.py` → upload the **WHOLE**
resulting JSON. Per-cell `orm_write` patches are FORBIDDEN (they skip duration, training summary,
Gantt, style remaps and inactive-app cleanup → "0 weeks" + empty timeline, the real ANRO PR4035
failure).

> ⚠️ **DANGER (Gotcha 16).** `spreadsheet_data` is `ttype='text'`, NOT Binary/Image. The MCP
> value-ref `{'__ref__': 'attachment', 'id': N}` is for Binary/Image fields ONLY. Writing it here
> raises **no error** and **silently corrupts** the record: it stores the literal dict-repr as the
> field's text, and the failure surfaces client-side hours later as an opaque `OwlError` /
> `SyntaxError: Expected property name or '}' in JSON`. `spreadsheet_payload.py` refuses a value-ref
> as payload — in its JSON form *and* in the Python-repr form you read back from an
> already-corrupted record — so this mistake fails locally instead of reaching the client.

### Decision procedure (measure, don't guess)

```bash
# 0. BACK UP the current field to disk first — this is what step 4 restores from.
python3 scripts/mcp_fetch.py save --to work/backup.json --from-host <hostpath>   # or --stdin

# 1. Fill (never edit cells by hand).
python3 scripts/fill_spreadsheet.py work/current.json work/budget.json work/filled.json --locale es

# 2. MEASURE and emit the exact bytes to upload (compact separators, ensure_ascii=False,
#    no trailing newline). Read `write_path` from the manifest — it is the routing decision.
python3 scripts/spreadsheet_payload.py measure work/filled.json --payload work/payload.txt
```

The manifest's decision metric is **`wire_bytes`**, not the file size: the payload travels **escaped
as a JSON string inside the MCP args**, and the workbook carries thousands of `"`, each costing 2
characters on the wire. `inline_ok` = `wire_bytes + 512 B` (args envelope) fits in `--threshold`
(default `65536`, the proxy's args budget). Add `--require-inline` to make an oversized payload exit
`3` instead of just reporting it.

Measured on workbooks **in this repo**, so the figures are reproducible from a clone —
`python3 scripts/spreadsheet_payload.py measure <file>`:

| workbook | baseline | compacted | wire (escaped in args) | route |
|---|---|---|---|---|
| `tests/golden/es/helpers/filled.json` — filled per-deal (8 sheets, 892 cells) | 43,845 B | **39,453 B** (−10.0%) | 46,647 B → 18,377 B of headroom | **`orm_write`** |
| `tests/fixtures/helpers/template_raw.json` — unfilled base template (`Parámetros` rollout) | 62,413 B | 56,062 B (−10.2%) | 66,408 B → **1,384 B over budget** | `ir.cron` |

That contrast is the whole decision procedure in one table: a **filled per-deal** workbook fits
inline with room to spare, while the **unfilled base template** — nothing removed yet — does not.
So the fast path is the norm for per-deal writes and the cron recipe stays mandatory for rollouts.
Do not generalize from one number: **always measure**, because a workbook with many active apps
approaches the ceiling (a real 5-app ES deal measures ~43.8 KB compacted, ~13.5 KB of headroom).

`ensure_ascii=False` is the smaller form with real content (accents in 'Análisis Funcional',
'Parámetros'…): 39,453 B vs 40,252 B ascii-escaped on the same file.

**2A. FAST PATH — `write_path: "orm_write"`** (1 round-trip instead of ~7 + the cron-scheduler wait):

```
orm_write(model="sale.order.spreadsheet", ids=[<id>],
          values={"spreadsheet_data": <the verbatim contents of work/payload.txt>})
```

- Pass the payload **as text**. Never an attachment value-ref (see the DANGER box).
- **If Bash + an odoo-pilot API key are available, prefer the RPC variant that reads
  `work/payload.txt` from disk** (odoo-pilot `update_record.sh`, or a short `xmlrpc`/JSON-RPC
  snippet): same single round-trip, and the payload never has to be re-emitted token by token —
  ~45 KB of args is ~11k output tokens (minutes of generation) when the agent must type it out.
- If the write errors (size limit, proxy refusal), do **not** retry it: go to 2B.

**2B. CRON PATH — `write_path: "ir.cron"`, or after any fast-path failure**: the full Gotcha 16
recipe (`SKILL.md` #16) — (1) `odoo_request_upload` with no pre-target; (2) `curl` the RAW
`work/payload.txt` to the pre-signed URL → `attachment_id`; (3) a **temporary** `ir.cron` whose code
is

```python
attachment = env['ir.attachment'].browse(<attachment_id>)
env['sale.order.spreadsheet'].browse(<id>).write(
    {'spreadsheet_data': attachment.raw.decode('utf-8')})
```

(4) trigger it with an `ir.cron.trigger` record (`method_direct_trigger` and
`ir.actions.server.run()` are both blocked by this proxy); (5) poll until `lastcall` is populated;
(6) verify (step 3); (7) delete the temporary cron (its trigger cascades) and the attachment(s).
This path never puts the payload in the agent's output — `curl` reads it from disk.

**3. MANDATORY read-back verify — BOTH paths, no exceptions.** Re-read `spreadsheet_data` to a file
(`mcp_fetch.py save …`, an `orm_search_read` dump, or the raw field text — all three shapes are
accepted) and:

```bash
python3 scripts/spreadsheet_payload.py verify work/payload.txt work/readback.json
```

Deep equality after `json.loads` on both sides, so Odoo re-serializing (key order, separators,
indentation) is fine and a single changed cell is not. Exit `0` = `pass`; exit `1` = `fail` with the
**path of the first mismatch** (e.g. `$.sheets[0].cells.A3.content`); exit `2` = the payload/read-back
isn't a workbook at all (value-ref → corrupted record). Tie it back to what you uploaded via
`sent_sha256` (identical to `measure`'s `sha256`).

**4. On `fail` — restore the backup**, do not "fix forward": write `work/backup.json` by the same
procedure, verify *that*, and only then re-attempt. A record left mismatched looks fine in the API
and breaks in the client's face.

## Troubleshooting

- **`measure` says `write_path: "ir.cron"` on a per-deal sheet:** the workbook still carries
  inactive apps/sheets (they are what removes bulk). Check `fill_spreadsheet.py` actually got
  `activa: false` for the unsold apps before reaching for the long recipe.
- **`verify` exits 2 with "value-ref":** the record was written with
  `{'__ref__':'attachment',...}` — it is corrupted, not merely different. Restore the backup.
- **An extra doesn't activate:** check that the name in `extras_activos` matches (by substring) the name in the template.
- **You need more than 10 sessions or more than 5 developments:** extend the template in Odoo — the script doesn't support more slots.
- **Two `spreadsheet_data` formats:** `fill_spreadsheet.py` requires cells in `dict {content, style}` format. Older spreadsheets (e.g., the one with the lowest `id`) store cells as plain `str` and **fail**. When copying a base for a new order (workflow §3.0), always use a RECENT "base implementation sheet" in dict format.
