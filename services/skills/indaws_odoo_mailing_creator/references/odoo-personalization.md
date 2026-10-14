# Odoo personalization (per-recipient dynamic content)

How to insert real per-recipient values (recipient name, company, unsubscribe link…) so Odoo fills them at
send time. This is the doc `html-input-rules.md §4` points to. Verified against Odoo 19 source
(`mass_mailing/models/mailing.py`).

## Two render engines — use the right syntax per field
Odoo renders `mailing.mailing` fields with DIFFERENT engines:

| Field | `render_engine` | Personalization syntax |
|---|---|---|
| `body_html` (what is sent) | **`qweb`** | `<t t-out="object.field">fallback</t>` |
| `preview` (preheader) | **`inline_template`** | `{{ object.field }}` |
| `subject` | `inline_template` (default) | `{{ object.field }}` |

`body_arch` (the editor copy) is NOT render-engine-tagged — `body_html` is the rendered one. We write both, so
put the dynamic tags in the source that becomes both (they pass through juice/minify untouched if well-formed).

## In the BODY → QWeb (`t-out`)
`object` is the current recipient record (the mailing's target model — often `res.partner` / a mailing
contact). Always give **fallback text** inside the tag so it's never blank:
```html
<t t-out="object.name">cliente</t>
<t t-out="object.partner_id.company_name or 'tu empresa'">tu empresa</t>
```
- QWeb leaves unknown `{{ }}` **as literal text** (it does NOT process them) → a stray `{{ NAME }}` would
  reach the inbox verbatim. So never use `{{ }}` in the body.
- Keep expressions simple field reads; avoid logic in the template.

## In the PREVIEW / SUBJECT → inline_template (`{{ }}`)
```
{{ object.name }}
```
Here `{{ }}` IS the right syntax (inline_template engine), unlike the body.

## Static campaign placeholders (filled by the marketing team, NOT per-recipient)
Things like `FECHA_MIGRACION`, `HORA_INICIO`, `URL_DYNCLOUD` are NOT personalization — the team edits them to a
fixed value before sending. Keep them as **plain UPPERCASE words** (they pass through MJML/juice/qweb
untouched). Do NOT wrap them in `{{ }}` or `t-out` — that would make Odoo try to resolve them and leave them
blank/literal. The skill's step-2 validation flags any leftover `{{ }}` precisely to avoid this confusion.

## Rule of thumb
- Per-recipient value in the **body** → `<t t-out="object.x">fallback</t>`.
- Per-recipient value in **preview/subject** → `{{ object.x }}`.
- Fixed value the team types in → plain UPPERCASE placeholder, no braces, no tags.

When in doubt, the Odoo editor's own "dynamic placeholder" button inserts the correct syntax for each field —
match what it produces.
