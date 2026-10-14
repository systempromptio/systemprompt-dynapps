# Lead Data Collection — Workflow Detail

Full workflow steps, rules, validations and success criteria.

> **Language.** This reference is English. The **form and the chatter note are rendered in the
> user's language** (default: the language of the CRM record). The markers `[PREFILLED - source]`
> and `[ASK]` stay verbatim English in the working draft; the visible pending label is localised
> only in the final HTML (Step 3a). See SKILL.md `## Critical`.

---

## Step 0: Data Collection (CRITICAL)

Before doing anything, **ALWAYS** gather:

1. **Corporate Odoo credentials** (if not already configured): URL, database, API key, user
2. **Opportunity identifier**: the numeric ID or the opportunity name in the CRM
3. **Additional context** (optional): anything the rep wants to add (e.g. "the client already
   runs SAP", "mainly interested in inventory")

**STOP** and wait for the user's answer before continuing.

---

## Step 1: Data Extraction from Odoo

```bash
# Corporate credentials
export ODOO_URL="<url>"
export ODOO_DB="<db>"
export ODOO_KEY="<api_key>"
export ODOO_USER="<user>"

# Extract by ID
./scripts/fetch_opportunity_data.sh <lead_id> --pilot-dir /path/to/odoo-pilot/scripts > /tmp/ldc_opportunity.json

# Or by name (ilike search):
./scripts/fetch_opportunity_data.sh "Client name" --pilot-dir /path/to/odoo-pilot/scripts > /tmp/ldc_opportunity.json
```

**What the script returns:**

- `opportunity`: crm.lead data (name, description, expected_revenue, stage_id, tags,
  **xx_data_collection**)
- `partner`: contact data (name, industry, website, city, country, employee count)
- `tags`: the lead's tag names
- `stage`: current stage name
- `chatter_notes`: **the last 50 notes/emails** on the lead (wider than other skills)
- `existing_toma_datos`: previous content of `xx_data_collection`, if any
- `attachments`: attached files (id, name, mimetype, file_size)
- `summary`: useful flags (has_website, has_relevant_attachments, has_existing_toma_datos, …)

**Post-extraction checks:**

1. **If `summary.has_existing_toma_datos` is true**: **STOP** and ask — "This opportunity already
   has a data collection form. Overwrite, complement, or cancel?" Show an excerpt of the existing
   content.
2. **If `summary.description_empty` is true**: warn the rep. The lead description is empty; work
   from tags, chatter, attachments and the website instead.

---

## Step 1.5: Supplementary Research

### (A) Attachment analysis (CONDITIONAL)

If `summary.has_relevant_attachments` is true:

- List the relevant attachments (PDF, DOCX, XLSX) for the rep
- Download them with the local script. It reads `datas` via JSON2 bearer (API key) in **a single
  HTTP call** for the whole batch and **decodes the base64 to disk inside the script** (base64
  never enters the context). Prefer batch mode, passing the `attachments` array from the
  opportunity JSON:

  ```bash
  # Batch (recommended): one call for every attachment
  echo "$ATTACHMENTS_JSON" | scripts/download_attachment.sh --batch-json -
  # or from a file:  scripts/download_attachment.sh --batch-json attachments.json
  # Single:          scripts/download_attachment.sh <id>
  ```

  It returns a JSON array of `{id, name, path, text_path, mimetype, bytes}` per attachment, and
  extracts plain text: PDF via `pdftotext`, DOCX/DOC/RTF/ODT via `textutil`, XLSX via a stdlib
  parser.

- **Reading an attachment — order of preference (token efficiency):**
  1. `text_path` != null (text already extracted) → `Read` THAT `.txt`. Do **NOT** read the
     binary. It is ~10–50× cheaper.
  2. `text_path` null and it is a PDF (scanned / no text layer) → `Read` the `path` (visual render).
  3. CSV/TXT → `Read` the `path` (already text).
  4. DOCX/XLSX with `text_path` null (tool missing or legacy `.xls`) → `Read` does NOT parse them.
     Convert first or tell the rep; never dump the binary.
- **NEVER** read the `ir.attachment.datas` field via MCP or `Read` (base64 into the context).
  Always go through the script, which decodes to disk.
- Extract: client needs, company structure, requirements, volumes, integrations mentioned,
  current software
- Feed everything extracted into the pre-fill as a source

**Attachment types to review** (by mimetype):

- `application/pdf` — presentations, proposals, requirement documents
- `application/vnd.openxmlformats*` — Word, Excel, PowerPoint
- `application/vnd.ms-*` — Word, Excel, PowerPoint (legacy)
- `text/csv` — exported data

**NEVER ignore attachments** — they frequently hold the most detailed client information.

### (B) Client website research (CONDITIONAL)

If `summary.has_website` is true:

- Use `WebFetch` on the client's site (`summary.website`)
- Extract: what the company does, products/services, sectors served, structure (subsidiaries,
  sites), approximate size
- Try key pages where available:
  - `/about`, `/about-us`, `/company` — corporate information
  - `/products`, `/services`, `/catalogue` — what they sell
  - `/contact` — sites and locations
- Feed this into the general profile, sector/activity and module-detection fields

If there is no website but there is a company name:

- Try `WebSearch` to find the site
- If found, proceed as above

---

## Step 2: Intelligent Pre-fill of the Form (LLM WORK)

**This step is entirely the LLM's.** Read and analyse:

1. `/tmp/ldc_opportunity.json`
2. `./references/questions_by_module.md`
3. `./references/budget_factors.md`
4. `./templates/data_collection_form.md`

### LLM tasks

a) **Analyse ALL available information**: opportunity description; chatter (50 messages — look for
   needs, current software, volumes, dates); attachments downloaded in Step 1.5a; the client
   website (Step 1.5b); partner data (industry, employees, city, comments); lead tags.

b) **Pre-fill the form** with what was found. Every pre-filled field states its source:
   - `[PREFILLED - opportunity]` — from the lead's description field
   - `[PREFILLED - chatter]` — from the message history
   - `[PREFILLED - attachment: name.pdf]` — from a specific attachment
   - `[PREFILLED - website]` — from the client's site
   - `[PREFILLED - partner]` — from contact data (industry, employees, …)

   Fields without confirmed information are marked `[ASK]` with a specific question, e.g.
   `[ASK: how many users will need access?]`.

c) **Detect the relevant modules** for Section 3:
   - **Part A — detected modules**: those the analysis says are relevant, each with a short
     explanation of the signal that triggered it. Detection rules live in
     `./templates/data_collection_form.md`.
   - **Part B — other modules**: every remaining Odoo module with its questions. The rep can
     delete whatever does not apply.

d) **Module detection rules** (for Part A):
   - Sales process, sales team, pipeline mentioned → CRM
   - Quotations, orders, price lists, sales process → Sales
   - They sell physical products → Inventory, Purchase
   - They manufacture → Manufacturing
   - Online shop / eCommerce → Website / eCommerce
   - Accounting or invoicing mentioned → Invoicing and/or Accounting
   - Employees or payroll mentioned → HR
   - Projects or services mentioned → Project
   - Support, incidents, tickets mentioned → Helpdesk
   - Online presence / corporate site → Website
   - Physical stores mentioned → Point of Sale

e) **Cap the active questions**: at most ~50 `[ASK]` fields in Part A. A one-hour call does not
   fit more. Prioritise by budget impact (see `budget_factors.md`).

---

## Step 2.5: Validation of the Pre-filled Form (MANDATORY)

**STOP** and present to the user:

1. **Summary of sources analysed**: what came from the opportunity; which attachments were
   reviewed and what they yielded; what the website gave; how many chatter messages were analysed
2. **The complete form** (the filled Markdown template)
3. **Detection summary**: modules in Part A and why; count of pre-filled fields vs `[ASK]` fields;
   a reminder that Part B holds the remaining modules
4. **Ask for revisions**: move modules between Part A and Part B; correct pre-filled information;
   delete Part B modules that clearly do not apply; add pre-call notes

**Wait for the user:**

- **Approve** → continue to Step 3
- **Modify** → adjust and present again
- **Cancel** → write nothing to Odoo

---

## Step 3: Save to Odoo

### 3a. Generate the clean HTML version

Take the approved form and:

- **Remove** the `[PREFILLED - …]` markers, leaving only the value
- **Keep** the pending rows with their specific question — the rep fills them in during the call —
  and **localise the visible marker** to the output language (`[PREGUNTAR]`, `[TE VRAGEN]`,
  `[ASK]`, …)
- Generate HTML using `./templates/data_collection_form_html.md`:
  - Inline styles are mandatory
  - Include the colour legend at the top (white = confirmed, yellow = pending)
  - Confirmed rows: white background on both cells (label + value)
  - Pending rows: yellow (`background-color: #fff3cd`) on BOTH cells (label + value), not just
    the value cell
  - Main colour `#714B67` for headers
  - Every visible string in the output language

### 3b. Save the form on the opportunity

```bash
$PILOT_DIR/update_record.sh "crm.lead" '[<lead_id>]' '{
  "xx_data_collection": "<form_html>"
}'
```

**If `xx_data_collection` does not exist** (the `xx_crm_fields` module is not installed on that
instance — on production it always is):

- Warn the user: "The field xx_data_collection does not exist on crm.lead — the xx_crm_fields
  module is not installed on this instance. Saving as a local file."
- Save as `data_collection_{client_name}_{date}.html`

### 3c. Create the internal chatter note

```bash
$PILOT_DIR/create_record.sh "mail.message" '{
  "model": "crm.lead",
  "res_id": <lead_id>,
  "body": "<summary_html>",
  "message_type": "comment",
  "subtype_id": 2,
  "subject": "Data collection prepared - <client_name> - <date>"
}'
```

Note: `subtype_id: 2` maps to `mail.mt_note` (internal note, not sent to the client). The note
body is written in the output language and includes: generation date; sources analysed
(opportunity, attachments, website, chatter); detected modules (Part A); count of pre-filled vs
pending fields.

**Present the result**: confirmation of the records created/updated.

---

## Rules and Guidelines

### General

- **ALWAYS** ask for the opportunity ID before starting (Step 0)
- **MANDATORY**: present the pre-filled form to the user (Step 2.5) and wait for approval
- **NEVER** create records in Odoo without user validation
- **NEVER** invent client data, make assumptions, or include hypotheses — only 100% verified
  information
- **NEVER** leave a field blank and unmarked — it either has a confirmed value (white background)
  or an `[ASK]` with a specific question (yellow across the whole row)
- **NEVER** pre-fill a field with deduced or assumed information — if it is not explicitly
  confirmed in the sources, it goes out as `[ASK]`
- **ALWAYS** state the source of pre-filled data (opportunity, attachment, website, chatter,
  partner)
- **ALWAYS** review the opportunity's attachments — they hold the most detailed information
- **ALWAYS** research the client website when a URL is available
- If `xx_data_collection` already has content, **ALWAYS** ask before overwriting

### Pre-fill

- Pre-fill only with 100% VERIFIED information from real sources — no exceptions
- State the source in parentheses after each pre-filled value
- If the information is ambiguous, unreliable or deduced, mark it `[ASK]` with a specific
  question — NEVER assume or infer
- At most ~50 active `[ASK]` questions in Part A
- Prioritise questions by budget impact (see `budget_factors.md`)

### Module detection

- EVERY module is detected from signals in the analysed information — there are no "always
  included" modules
- When a module is detected, briefly explain WHY
- Part B holds ALL undetected modules — the rep decides whether to delete or use them
- If 8+ modules land in Part A, flag that the project is probably complex

### HTML form

- Inline styles mandatory (Odoo does not process external CSS)
- Main colour: `#714B67` (Odoo violet)
- Colour legend at the top of the form (white = confirmed, yellow = pending)
- Pending rows: yellow (`#fff3cd`) on BOTH cells (label + value), NOT just the value cell
- The HTML version is CLEAN: no source markers, values only
- It must be readable directly in the Odoo crm.lead form
- Every visible string in the output language, one language per form

### Skill scope

- The skill ENDS before the call — its job is to PREPARE, not to accompany
- There are no post-call steps
- The rep uses and edits the form manually during the call, in Odoo

### Odoo connection

- Use odoo-pilot to read and write records
- If `update_record.sh` fails, print the HTML so the user can paste it manually
- Verify `xx_data_collection` exists before writing (it does wherever `xx_crm_fields` is installed)
- `mail.message` with `subtype_id: 2` for an internal note (NOT sent to the client)

---

## Validations

**Interactive (mandatory):**

1. **Pre-filled form validation (Step 2.5)**: the user approves the complete form before it is
   saved to Odoo

**Conditional:**

2. **Existing form (Step 1)**: if `xx_data_collection` already has content, the user
   decides overwrite / complement / cancel
3. **Name search (Step 1)**: if the name search returns several results, the user selects one
4. **Relevant attachments (Step 1.5a)**: if there are PDFs/documents, download and review them
   BEFORE pre-filling

**Automatic:**

5. Opportunity JSON extracted correctly with the required fields
6. At least one data source available (description, attachments, chatter or website)
7. Relevant attachments reviewed, if any exist
8. Every Part A module carries its detection justification
9. At most ~50 active `[ASK]` questions in Part A
10. The form HTML uses inline styles
11. Pending rows highlighted yellow (`#fff3cd`) on BOTH cells in the HTML version
12. Internal chatter note created correctly
13. The form is in a single language, and it is the output language requested

---

## Success Criteria

- Opportunity data extracted correctly (50 chatter messages)
- Relevant attachments downloaded, read and used as a source
- Partner website researched and business information extracted, if a URL existed
- Form pre-filled with everything available, each datum carrying its source
- Fields without information marked `[ASK]`
- Modules detected correctly with an explanation of why (no "always included" modules)
- All remaining modules available in Part B
- **The user validated the form** (Step 2.5)
- Clean HTML form saved to the lead's `xx_data_collection` (or a local file if `xx_crm_fields` is
  not installed on the instance)
- Internal chatter note created on the lead summarising what was prepared
- Pending rows highlighted yellow (`#fff3cd`) on BOTH cells in the Odoo HTML version
- Colour legend included at the top of the HTML form
- Nothing invented, assumed or inferred — every datum from a 100% verified source
- Form and chatter note rendered in the output language, consistently
