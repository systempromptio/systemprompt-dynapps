---
name: dynapps-lead-data-collection
description: >-
  Prepares a pre-filled discovery data-collection form for calls with Odoo implementation leads.
  Pulls the CRM opportunity (description, chatter, PDF/DOCX/XLSX attachments, client website),
  marks each field '[PREFILLED - source]' or '[ASK]', detects the relevant Odoo modules from
  signals, and saves it to xx_data_collection with an internal chatter note. Triggers:
  "prepare the data collection for lead X", "discovery form", "pre-fill the discovery
  questionnaire", preparing a discovery call. Renders the form in the rep's or client's language.
version: 1.5.0
model: opus
effort: medium
---

# Lead Data Collection — Discovery Call Preparation

Sales assistant that extracts information from a CRM opportunity (description, chatter,
attachments, website) and writes a pre-filled discovery form into the lead's
`xx_data_collection` field.

## Critical

1. **Output language follows the user.** This skill and all its references and templates are
   written in English; the **form, the chatter note and every field label are rendered in the
   language the user asks for**. Default when unstated: the language of the CRM record and its
   chatter (typically the rep's own language). Never mix languages inside one form — labels,
   questions and the colour legend all go in the same language.
2. **NEVER invent, assume or infer client data.** Only 100% verified information from real
   sources. Anything unconfirmed goes out as `[ASK]` with a specific question.
3. **NEVER write to Odoo without user approval** — Step 2.5 is mandatory.
4. **NEVER read `ir.attachment.datas` through MCP or `Read`** — base64 (+33%) floods the context.
   Always go through `scripts/download_attachment.sh` (see GOTCHA 8).

---

## Workflow

> Full detail with code blocks, sub-steps and templates: `references/workflow_detail.md`

| Step | Action | Gate |
|------|--------|------|
| **0** | Collect Odoo credentials + opportunity ID + optional context. See references/shared/odoo-connection.md for the connection pattern. | **[STOP]** — wait for the user |
| **1** | Run `fetch_opportunity_data.sh` → `/tmp/ldc_opportunity.json` | Existing form → ask overwrite / complement / cancel |
| **1.5a** | Download and analyse relevant attachments (PDF, DOCX, XLSX) | Conditional: `has_relevant_attachments` |
| **1.5b** | Research the client website with WebFetch (about, products, contact) | Conditional: `has_website` |
| **2** | Analyse every source, pre-fill the form with `[PREFILLED - source]` or `[ASK]`, detect modules (Part A detected + Part B the rest) | Max ~50 active questions |
| **2.5** | Present the complete form + a source summary to the user | **[STOP]** — wait for approval |
| **3** | Generate clean HTML, save to `xx_data_collection`, add an internal chatter note (`subtype_id: 2`) | No `xx_crm_fields` on the instance → save as a local file |

---

## RULES

1. **NEVER invent data** — only 100% verified information from real sources; anything unconfirmed
   goes out as `[ASK]`
2. **ALWAYS state the source** of each pre-filled field: opportunity, chatter, attachment,
   website, partner
3. **NEVER write to Odoo without approval** (Step 2.5 is mandatory)
4. **ALWAYS review the attachments** — they carry the most detailed client information
5. **ALWAYS research the website** when a URL is available
6. **Commercial language** — the reader is a sales rep, not a functional consultant. No technical
   jargon
7. **Modules by signal** — there are no "always included" modules; each one is justified
8. **Inline HTML** — inline styles are mandatory, main colour `#714B67`, `[ASK]` rows in yellow
   `#fff3cd` on BOTH cells
9. **Colour legend** at the top of the HTML form (white = confirmed, yellow = pending)
10. **The skill ENDS before the call** — its job is to PREPARE, not to accompany
11. **One language per form** — render labels, questions, legend and chatter note in the same
    language (see `## Critical` 1)

---

## GOTCHAS

1. `xx_data_collection` is a **module field** (`xx_crm_fields`), not a Studio field: it is present
   on every instance where that module is installed, so on production it is always there. Keep the
   local-file fallback only for instances without `xx_crm_fields`.
2. If it already has content, ask BEFORE overwriting (never overwrite silently).
3. Odoo ignores external CSS in HTML fields — everything must be inline.
4. `[ASK]` rows need the yellow background on BOTH cells (label + value), not just the value.
5. `mail.message` with `subtype_id: 2` = internal note; without the 2 an email goes to the client.
6. Searching by name can return multiple results — present the list to the user.
7. Attachments frequently hold the richest information — NEVER skip them.
8. **Attachments: NEVER read `ir.attachment.datas` via MCP or `Read`** — the base64 (+33%)
   floods the context. Use `scripts/download_attachment.sh --batch-json -` (or `<id>`): it reads
   `datas` via JSON2 bearer, decodes to disk (base64 never touches the context) and extracts
   text. Read the `.txt` at `text_path`; if null, `Read` the `path`.
9. **With an API key only JSON2 bearer works for attachments** — in Odoo 19
   `/web/session/authenticate` rejects the API key (Access Denied) and `/web/content/<id>`
   returns 404 without a cookie. Do not retry the relative `/web/content` URL; the script already
   uses the only valid route.
10. **`Read` does NOT parse DOCX/XLSX** (binary zips). The script extracts text: PDF→`pdftotext`,
    DOCX/DOC/RTF/ODT→`textutil` (macOS), XLSX→stdlib parser. If the tool is missing or the `.txt`
    comes out empty (scanned PDF, legacy `.xls`) → `text_path` is null; do not `Read` the binary.
11. **Markers are English in the draft, localised in the final HTML.** `[ASK]` and
    `[PREFILLED - source]` are machine markers: keep them verbatim in the working draft so Step 3a
    can strip `[PREFILLED …]` and highlight `[ASK]` rows reliably. In the HTML that lands in Odoo,
    render the visible pending label in the output language (`[PREGUNTAR]` in Spanish,
    `[TE VRAGEN]` in Dutch, `[ASK]` in English) — the rep reads it, so it must be their language.

---

## Agent Routing

**opus** — evidence synthesis across heterogeneous sources (description, chatter, attachments,
website) plus judgment on which modules the signals justify.

## Delegates to

- **odoo-pilot** (common) — CRUD on crm.lead, mail.message, ir.attachment

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| demo-script-generator | sales (optional) | Uses the generated form as input for demo scripts |

---

## References

| File | Content |
|------|---------|
| `references/workflow_detail.md` | Full workflow steps (0–3), sub-steps, code blocks, rules, validations, success criteria |
| `references/questions_by_module.md` | Budget-driving questions organised by Odoo module |
| `references/budget_factors.md` | Factors that move the implementation budget, question priority |
| `templates/data_collection_form.md` | Markdown template of the discovery form |
| `templates/data_collection_form_html.md` | HTML template with inline styles for Odoo |
| `scripts/fetch_opportunity_data.sh` | Opportunity data extraction script |
| `scripts/download_attachment.sh` | Downloads attachments via JSON2 bearer (1 call per batch), decodes base64 to disk (never to the context) and extracts text: PDF→pdftotext, DOCX→textutil, XLSX→stdlib parser |
