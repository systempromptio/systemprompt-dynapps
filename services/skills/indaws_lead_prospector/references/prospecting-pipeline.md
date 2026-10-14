# Prospecting Pipeline — Clay MCP Step-by-Step

## Step 1: Parse User Request

Extract prospecting parameters from the user's input:
- Company identifier (domain) or company characteristics (industry, size)
- Contact filters (titles, locations, seniority)
- Enrichment requirements (email, tech stack, etc.)
- Quantity desired

## Step 2: Search for Contacts at Company

Use `find-and-enrich-contacts-at-company` with:
- `companyIdentifier`: Company domain (e.g., "empresa.es")
- `contactFilters`:
  - `job_title_keywords`: Target titles from ICP (e.g., ["CEO", "Director General", "CFO", "Director Financiero", "COO", "CTO", "Director de IT"])
  - `locations`: ["Spain"] or more specific (["Valencia", "Alicante", "Castellon"])
  - `job_title_exclude_keywords`: ["Intern", "Becario", "Junior"] (optional, to focus on decision makers)
- `dataPoints` (optional, can add later):
  - `contactDataPoints`: [{"type": "Email"}]
  - `companyDataPoints`: [{"type": "Tech Stack"}, {"type": "Annual Revenue"}]

**Save the returned `taskId`** — every subsequent enrichment step uses the same `taskId`.

## Step 3: Enrich Companies

Use `add-company-data-points` with:
- `taskId`: From Step 2
- `dataPoints`: Select based on user needs:
  - `{"type": "Tech Stack"}` — Critical: identifies current ERP
  - `{"type": "Annual Revenue"}` — Validates ICP revenue range
  - `{"type": "Headcount Growth"}` — Growth signal
  - `{"type": "Recent News"}` — Buying signals, leadership changes
  - `{"type": "Open Jobs"}` — Hiring for ERP roles = buying signal
- `entityIds`: (optional) Specific company IDs to enrich

## Step 4: Find Decision Makers

If Step 2 did not include contact filters, or to expand the search:
- Use `find-and-enrich-contacts-at-company` again with refined `contactFilters`
- Spanish C-level title variations:
  - CEO: ["CEO", "Director General", "Consejero Delegado", "Gerente"]
  - CFO: ["CFO", "Director Financiero", "Director de Finanzas", "Responsable Financiero"]
  - COO: ["COO", "Director de Operaciones", "Director de Produccion"]
  - CTO: ["CTO", "Director de IT", "Director de Sistemas", "Director de Tecnologia", "Responsable de IT"]
  - HR: ["Director de RRHH", "Director de Recursos Humanos", "Responsable de RRHH"]

## Step 5: Enrich Contacts

Use `add-contact-data-points` with:
- `taskId`: From Step 2
- `dataPoints`:
  - `{"type": "Email"}` — Required for outreach
  - `{"type": "Summarize Work History"}` — Context for personalization
  - `{"type": "Find Thought Leadership"}` — LinkedIn activity for warm outreach hooks
- `entityIds`: (optional) Specific contact IDs to enrich

## Step 6: Poll for Results

Use `get-existing-search` with:
- `taskId`: From Step 2
- Wait approximately 5 seconds between polls
- Check each enrichment's `state` field:
  - `"in-progress"` — Still processing, wait and poll again
  - `"completed"` — Results available in `value` field
- Poll up to 5 times (total ~25 seconds max wait)

## Step 7: Apply Quality Gate Scoring

For each lead returned, apply the scoring system from `references/quality-gate.md`:

1. Start with base score of 50
2. Apply each check from the scoring table
3. Classify: PASS (>=60), WARN (>=35), FAIL (<35)
4. Record all score impacts for transparency

## Step 8: Present Ranked Lead Table

See `references/output-formats.md` for the full table and profile formats.

---

## Alternative: Company-First Research

When the user provides a specific company domain or name, use the single-company deep-dive flow.

### Single Company Lookup

Use `find-and-enrich-company` with:
- `companyIdentifier`: Company domain (e.g., "target-company.es")
- `companyDataPoints`: Select relevant enrichments:
  - `{"type": "Tech Stack"}` — What ERP/tools they currently use
  - `{"type": "Annual Revenue"}` — Revenue range
  - `{"type": "Headcount Growth"}` — Growing or shrinking
  - `{"type": "Recent News"}` — Latest company news
  - `{"type": "Open Jobs"}` — Hiring activity
  - `{"type": "Company Competitors"}` — Competitive landscape
  - `{"type": "Company Customers"}` — Their customer base

### Custom Research Questions

Use Custom data points for specific research needs:
- `{"type": "Custom", "dataPointName": "Current ERP system", "dataPointDescription": "What ERP or business management software does this company currently use?"}`
- `{"type": "Custom", "dataPointName": "Digital transformation status", "dataPointDescription": "Evidence of digital transformation initiatives or recent technology investments"}`
- `{"type": "Custom", "dataPointName": "Recent funding or growth", "dataPointDescription": "Any recent funding rounds, acquisitions, or significant growth indicators"}`

### Then Find Decision Makers

After company research, use `find-and-enrich-contacts-at-company` to find relevant contacts at that specific company.
