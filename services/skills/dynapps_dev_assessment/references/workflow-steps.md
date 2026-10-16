# Workflow Steps — Detailed Reference

> Instructions are English. **Every artefact produced goes out in the reader's language** (the
> client validation document in the client's language). See SKILL.md `## Critical`.

## STEP 0: Information Gathering [STOP]

**NEVER continue without gathering ALL the necessary information.**

### 0.0 Detect the mode (A or B)

- **Mode A — new request**: nothing has been sketched yet. This is the default mode.
- **Mode B — promotion**: a Studio prototype already exists (typically in staging) that someone
  wants to push to production.

Deduce it, do not ask if it is obvious: if there is a Studio change in staging that is not in
production, it is **mode B**. Detail and the bias of each mode:
[escalation-and-modes.md](escalation-and-modes.md).

**If there are two environments that differ, that is two assessments, not one.**

### 0.1 If the argument is an Odoo record ID (lead / task / ticket)

When the skill receives a number (e.g. `32676`), that is an Odoo record ID, NOT a description.
Resolve the context via the **Odoo MCP** (ORM proxy: `odoo_whoami`, `orm_search_read`,
`orm_execute_kw`, `odoo_attachment_link`) in **one single parallel batch of tool calls**:

1. `odoo_whoami` → version and edition (`odoo.version`, `odoo.edition`). **Do not ask for the
   version if the MCP already gives it.**
2. Resolve the record's **model generically, in parallel** (not cascaded): try `crm.lead`,
   `project.task`, `helpdesk.ticket`, `sale.order` with `orm_search_read` in the same batch.
3. Read from the record: `name`, `description`, `partner_id`, `user_id`, and its
   `message_ids` / `attachment_ids`.

### 0.2 ALWAYS read the attachments (critical accuracy rule)

**An attachment may contain a partner proposal, technical requirements, or a solution that
invalidates an analysis based on the description alone.** Reading them is not optional. Read
cheapest-first and stop at the first useful text:

1. `orm_execute_kw(ir.attachment, read, [ids], [["index_content"]])` → text Odoo already
   extracted. Cheap, fast, no binary through the context. **Default route.**
   **Warning:** `index_content` **DISCARDS non-ASCII characters** — a flawless UTF-8 CSV reads as
   `Preparaci n`, `sin m quina`. It serves to **decide** (what the file says), not to
   **diagnose**: before claiming anything about encoding, accents or odd characters, download the
   binary and check. Verified 21-Aug-2026 with the CSVs of task 21612.
2. Only if `index_content` comes back empty (image, scanned PDF) → `odoo_attachment_link` to
   download, then `Read` / OCR.
3. **NEVER** request `datas` / binaries via `orm_search_read` / `read`: the MCP returns `false`.
   Guaranteed wasted calls.

### 0.3 Subagent by volume only

Read attachments **inline** in the common case (one small attachment) — cheaper than a subagent.
Delegate to an `Explore` subagent (structured summary) **only if** there are ≥3 attachments or
>~50 KB of extracted text. Fan-out may be disabled; if so, read inline and say so.

### 0.4 Detect what is missing, then ask only for that

After reading the record and attachments, check the request against the completeness list
below. **Ask only for what is actually missing** — never re-ask what the record or the user
already provided. `AskUserQuestion` is for gaps, not for confirmation.

**Blocking — a gap here means STOP and ask.** The assessment cannot proceed without it:

1. **Odoo version** (14.0–19.0). Greenfield + no `odoo_whoami` → default to the latest for new
   implementations.
2. **Edition** (Community / Enterprise). Often deducible (Studio, real-time consolidation,
   advanced multi-company and e-invoicing → Enterprise), but Studio is not available on Community,
   so it blocks the verdict.
3. **What the request is** — the concrete gap or desired behaviour. "Improve reporting" is not
   assessable; "add column X to report Y" is.
4. **Who / which flow is affected** — when it changes what "standard covers this" means (a portal
   user and an internal user give different answers).

**Non-blocking — record it as missing, do not ask now.** These matter to the developer later,
not to the classification. If absent, carry them into the transfer package as open questions:

- Acceptance criteria (how you know it is done)
- Volume / affected records
- Deadline or priority
- Dependencies (other tasks, modules, integrations)

If everything needed is present, do not ask. **STOP only when a blocking item is missing**; a
non-blocking gap does not stop the assessment — it is recorded and passed on.

---

## STEP 1: Search Standard Odoo

### 1.1 Consult the changelogs
For Odoo changelogs, delegate to the `content-collection` skill. **Do not load the whole
changelog with `Read`**: filter with `grep -niE "kw1|kw2"` on the requirement's keywords and read
only the matches (context with `sed -n`). Search the client's version AND later ones.

### 1.2 LLM knowledge
Evaluate: standard functionality, little-known advanced configuration, module combinations.

Frequently unknown functionality: server actions, no-code automations, Studio computed fields,
multi-level price lists, customizable portal, Studio approvals (group rules since v14; named
approvers and delegation since v18), electronic signature (Enterprise).

### 1.3 Web search (only if needed)
Only on genuine doubt: `site:odoo.com/documentation/{version} {functionality}`

### 1.4 Outcome
- **COVERED**: exists in standard → END
- **PARTIALLY**: something is missing → STEP 2
- **NOT COVERED** → STEP 2

---

## STEP 2: Search Dynapps Core

### 2.1 Consult the indexes (triage, not the answer)
For the Dynapps Core module indexes (123 core + 8 enterprise + 15 vertical), delegate to the
`content-collection` skill.

**Do not load the whole index with `Read`** (hundreds of modules): filter with `grep -hiE "kw"`
or a `python3 -c` that walks the index and prints only modules whose `technical_name` /
`description` / `depends` match the keywords. Search by technical name, description,
dependencies, category, version availability.

> **The index tells you the name and description, not whether it does the work.** A match can
> still be a module that does not cover the case (a Core module that matched by keyword and
> turned out to have 0 views), and a miss can hide a module whose description does not use the
> request's words.

### 2.2 Read the code when the decision matters
If the request is non-trivial or the domain smells of Core (finance, sales, project, HR, stock,
FSM, EDI, contracts, connectors, AI/OCR, migrations), the index is not enough — **read the code**.
Cheapest first:

1. **Data model via the live environment (no clone).** If a client environment is reachable
   (STEP 0), read `ir.model`, `ir.model.fields`, `ir.ui.view` filtered by the candidate module:
   does it bring the models/fields/views the request needs? This answers "does it cover this" in
   most cases.
2. **Clone the shared repo and read the source** when the *logic* is what matters (does it
   compute/validate correctly), not just the shape. Shared repos, branch `master`:

   | Repo | Indexed? | Notes |
   |---|---|---|
   | `indaws-odoo` | yes (123) | consolidated Core |
   | `indaws-odoo-enterprise` | yes (8) | enterprise |
   | `vertical-odoo-inbuilt` | yes (15) | verticals |
   | `indaws-contract` | no | contracts |
   | `indaws-edi` | no | EDI |
   | `indaws-tools` | no | shared tools |
   | `indaws-migration-kit` | no | migrations |
   | `ai-odoo`, `ai-odoo-ocr` | no | AI / document extraction |
   | `connector-delivery-odoo` | no | delivery connectors |

   Use the same clone routes as STEP 4.0 (`gh` / SSH / ZIP, always `--depth=1`).

   **Never clone the legacy family** — `indaws_core`, `indaws_account`, `indaws_conectors`,
   `indaws_apps`, `indaws`, etc. (2022-2023, consolidated into `indaws-odoo` in Dec 2024;
   `indaws_sales` is DMCA-blocked). That is dead code and would mislead the verdict.

### 2.3 If you cannot read the code, do not guess
If the verdict hinges on code you cannot read (no GitHub access, no live environment), the
outcome is **not** a verdict:

- **Escalate to a dev.** Say so in the report: "the Core decision needs a dev to read
  `{repo}/{module}`".
- **Request repo access.** If `gh auth status` fails or the clone is refused, tell the user to
  ask for access to the `indawsodoo` org and stop — do not proceed blind.

This is the escalation gate applied to the Core decision: the skill never invents a verdict it
could not verify.

### 2.4 Outcome
- **COVERED BY CORE** → END
- **PARTIALLY** → STEP 3
- **NOT COVERED** → STEP 3

---

## STEP 3: Studio vs Custom Evaluation [STOP]

For classification, delegate to the `odoo-classification-guide` skill — the source of truth for:
- Golden rules — mandatory, evaluate ALWAYS first
- The full classification decision tree
- Studio vs Custom thresholds per version
- Complexity thresholds — Minor / Standard / Complex
- Its **`ai-impact-studio-vs-custom.md`** reference — the updated criterion: for non-trivial
  logic, a governed module beats ungovernable Studio. Do not classify as Studio to "avoid code".

### Classification
- STUDIO: solvable with Studio → generate instructions, END
- CUSTOM MINOR: <2 days
- CUSTOM STANDARD: 2–5 days
- CUSTOM COMPLEX: >5 days

**Before the verdict**, if there is a live client environment → run **STEP 3.5** (it can
reclassify a "Studio" into a module because of a collision).

**STOP:** present the result of steps 1–3. If CUSTOM, continue to STEP 4.

---

## STEP 3.5: Studio governance check (collision radar) — optional

**When it applies:** there is a **live client environment** (the MCP already connected in
STEP 0). This covers the gap the ecosystem acknowledges: there is no Studio auditor. Two
situations: a new request leaning towards Studio, or a **staging→production promotion** of a
change already built in Studio (the consultant's note is the main input).

**No environment access (greenfield, description only): SKIP this step.**

Full detail and MCP recipes: [studio-governance-check.md](studio-governance-check.md). Summary:

1. Classify installed modules **by author** (standard = `Odoo S.A.`; non-standard = Indaws / OCA /
   client custom + `studio_customization`).
2. Map which **fields** each non-standard module touches (`ir.model.data` → `ir.model.fields`);
   group by zone.
3. Identify the **zone** the request lands in. There is a **collision** only if that zone is
   already managed by an existing non-standard module or Studio (by zone, NOT by whole model).
4. **Collision → force a module** (extend the piece that already owns the zone, `indaws_*` Core if
   reusable). Free zone + trivial → Studio still valid. Genuine doubt → a developer decides.
5. **Promotion mode:** diff `studio_customization` staging vs production to isolate what is new;
   the consultant confirms the short list. A human always performs the production push; fields
   must be clean (no `x_studio_*`); staging data is not migrated.

---

## STEP 4: Technical Approach

### 4.0 Get the code before scanning

`odoo-module-scanner` needs the module **locally**. If you do not have it, obtain it in this
order (stop at the first one that works) and **write down in the task which route worked**:

1. **Already local** — look first: `ls ~/addons-{client}` or the path declared in CLAUDE.md.
   Do not clone what you already have. If you do not know the repo:
   `gh search repos --owner indawsodoo {client}`.
2. **GitHub CLI** (if `gh auth status` answers OK):
   `gh repo clone indawsodoo/{repo} /tmp/{repo} -- --depth=1`
3. **SSH** (requires a key registered on GitHub):
   `git clone --depth=1 git@github.com:indawsodoo/{repo}.git`
4. **ZIP by URL** — for anyone with neither `gh` nor an SSH key:
   `https://github.com/indawsodoo/{repo}/archive/refs/heads/{branch}.zip`

Always clone with `--depth=1`: you need the code, not the history. Work in `/tmp` or the
scratchpad, **never on the client's working repo**.

Note the **module version** (`__manifest__.py`) alongside the route: the transfer package
requires it.

### 4.1 Architecture
Modules to create (`indaws_*`), new/extended models, views, security, dependencies, demo data.

### 4.2 Estimation

Delegate to the `odoo-effort-estimator` skill: it is the single source for hours per component
and for the AI factors. Do not duplicate them here.

### 4.3 Risks
Migratability, critical dependencies, performance, testing complexity, alternatives dismissed.
