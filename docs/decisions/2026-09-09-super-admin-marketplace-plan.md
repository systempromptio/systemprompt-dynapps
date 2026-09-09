# Astound executive insights and UK Sales — proposed implementation plan

Status: implemented in source, 9 September 2026; live rollout and pilot source access
remain separate. Source: Ed / Terry meeting, 9 September 2026.

Implementation: both marketplaces, both plugins and all six skills are configured.
`super_admin` is a recognized role whose grant/revocation requires `platform_admin`;
revocation also revokes live credentials. The `uk` group and the exact requested
`business-imeplmentation` project are declared for startup ingestion and were ingested into the local database,
with both rows read back successfully. They have no assumed
AD mappings or automatic user enrollment. Project membership provides attribution,
not marketplace access. External research uses host browsing when available. Both plugins passed CLI validation
and generated successfully; all six skills passed structural validation.

Validation: `just preflight-static` (33 gates), `just clippy`, `just test-unit`
(929 passed), and `just test-integration` (811 passed, including the executive/UK
access matrix). Both test tiers reported zero skipped tests. No remote deployment
or live client-data report is claimed by this implementation.

## Outcome and audience

Give Astound leadership a concise, evidence-backed view of project health, decisions
holding up delivery, and emergencies with their recovery progress. Explain what the
evidence means, which decisions matter, and the available courses of action. Pair
this with a UK Sales plugin for client discovery, reporting and research. Start with the
UK portfolio; expand to other regions through explicit scope selection and existing
Atlassian permissions. Terry's UK remit must not silently become a global report.

The transcript's strongest requirements are client/project issues and recovery
(27:47–30:37), source permissions (31:14–33:18), and reusable skills that leadership
can use immediately (41:27–43:43). Treat the transcript as requirements, not proof
that every integration, field or organization-wide permission is available.

## Marketplace and role

| Item | Proposed definition |
| --- | --- |
| Marketplace | `astound-super-admin`, displayed as **Astound — Executive Insights** |
| Visibility | Private; `default_included: false`; access role `[super_admin]` |
| Plugins | `astound-executive-insights` and `astound-sales-uk`, each with three explicit skills |
| MCP server | Existing `atlassian`, explicitly included by marketplace and plugin |
| New role | `super_admin`, displayed as **Super Admin** |
| Assignment | Only `platform_admin` may grant or revoke it; no automatic grants to existing admins |
| Initial users | Candidate pilot: Terry, Jody and Roman, subject to actual account selection and role assignment |

`super_admin` is an executive entitlement. It does not implicitly confer platform
configuration, account administration, impersonation, or broader Atlassian data
access. Keep `platform_admin` as the existing platform authority. A platform admin
who needs this marketplace also receives `super_admin` explicitly. Retain normal
`user` entitlement alongside it for standard gateway use.

The database accepts free-text roles, but the application role editor uses a closed
Rust enum. Implementation must add `Role::SuperAdmin`, its label, parsing and role
catalog entries, and protect changes in `authorize_role_change`. Do not add it to
`ROLES_MANAGE` or `ROLES_PLATFORM`; no admin console access is needed for this pilot.
Use the existing manual-role storage so grants survive SSO sign-in. Directory group
mapping can follow once a real executive group is agreed.

Own marketplace access in its `config.yaml`. Add explicit `super_admin` rules for
the new plugin and three skills to protect direct access and accidental inclusion
elsewhere. Extend the existing Atlassian role grant with `super_admin`, preserving
the existing admin and India grants. That extension is necessary because the
existing explicit MCP rule opts out of marketplace inheritance. Verify connector
entitlement as well as manifest visibility and runtime authorization.

## Executive analysis contract

Each executive skill answers five questions: **What changed? Why does it matter?
What explains it? What are our options? What should leadership decide next?**
Lead with an executive judgment supported by evidence, followed by the report tables.

Look for recurring delivery risks, shared dependencies, concentration of issues by
client or project, and changes in recovery effectiveness. Label possible causes as
hypotheses unless records establish causality. Where useful, compare two or three
interventions by expected benefit, tradeoff, dependency and evidence confidence;
state assumptions and identify the information that would change the recommendation.
Do not invent numerical forecasts, financial exposure or causal conclusions.

The portfolio brief identifies priorities and patterns; the blocker brief compares
interventions and their sequencing; the emergency brief assesses recovery confidence
and the next escalation decision. Each closes with a short recommended decision list,
recorded or proposed owners, and the next review point. Human leaders make decisions.

## Three executive skills

### 1. Executive portfolio health — `executive_portfolio_health`

**Question:** “How are our UK projects doing, and what needs my attention?”

Read accessible Jira projects, current issues, milestones/releases and linked
Confluence status reports. Produce a one-page portfolio brief with a project table:
client/project, reported RAG, supporting issue evidence, delivery owner, next
milestone, principal risk, recovery action and last update. Lead with the five
items most needing attention, with links for project and issue drill-down.

Separate recorded RAG from observed risk signals. A red issue does not automatically
make its entire project red. Highlight discrepancies such as an unresolved critical
issue omitted from a status report as **needs review**, without alleging manipulation.
Include projects with missing status as **unknown**. Show improving or deteriorating
status only where changelog or dated reports support the comparison.

**Default:** selected UK portfolio, current state and seven-day changes. If historical
evidence is unavailable, provide current state and label change as unavailable.

**Success example:** Jody can identify the three projects needing intervention,
understand the recovery plan and open the underlying evidence without reading every
project report.

### 2. Blockers and leadership decisions — `executive_blocker_decisions`

**Question:** “What is holding delivery up, and what decisions do we need to make?”

Read explicit blocked states/flags, issue dependencies, outstanding approvals,
ownership and due dates, plus linked decision logs and recovery plans. Rank blockers
by evidenced delivery impact and urgency. Return a decision queue: blocker, affected
projects/milestones, recorded duration, accountable owner, next action, decision
required, decision deadline and source links. Distinguish recorded decisions from
suggested interventions; mark proposed owners as proposals.

Group cross-project dependencies only when issue links or documents establish the
relationship. Identify repeated dependencies on a team or vendor when supported by
records. Stale or unassigned work is an attention signal, not proof of a blocker.
Do not infer staffing capacity, utilization or employee performance from issue counts.
If blocked-since history is unavailable, report its duration as unknown.

**Default:** active blockers and milestones due in the next 14 days within the chosen
portfolio. Include overdue blockers even when the milestone lies outside that window.

**Success example:** Terry receives a short list of decisions leadership can actually
take, with the projects each decision would unblock and the evidence behind it.

### 3. Emergency and recovery brief — `executive_emergency_recovery`

**Question:** “What is urgent now, who owns recovery, and is it getting better?”

Read unresolved incidents/critical issues, explicit escalations and linked recovery
records. Use the site's actual severity mapping; priority alone must not invent a
business emergency. Produce an active emergency register with recorded severity,
client impact, incident owner, latest verified update, containment/recovery action,
next checkpoint, target resolution if recorded, and evidence links. Separate
confirmed emergencies from candidate escalations requiring review.

Include a short change log of new escalations, reopened incidents and confirmed
resolutions, plus overdue recovery checkpoints. An issue marked Done is not proof
of service restoration unless the record establishes that. Jira Service Management
incident data is optional and used only if accessible through the available MCP
capabilities; otherwise clearly state the missing incident coverage.

**Default:** active emergencies plus changes in the last 24 hours. Offer a seven-day
recovery view for leadership meetings. This is an on-demand report, not continuous
monitoring, paging or an automatic emergency response system.

**Success example:** leadership sees both urgent problems and how they are being
resolved, reflecting Terry's explicit preference for constructive recovery reporting.

## UK Sales plugin — `astound-sales-uk`

Display name: **Astound — Sales UK**. Purpose: help Andrew, Michael, Terry and the
UK client team understand accounts and have better client conversations. The
transcript prioritizes client news, leadership changes, delivery context and meeting
preparation (20:22–21:46 and 29:14–29:54), followed by a credible retail AI proposition
(45:11–46:56).

Include this plugin in `astound-super-admin` for executives. For the wider UK sales
team, also plan a private `astound-sales-uk` marketplace with explicit membership in
an configured `uk` group. Do not assign Super Admin to sales staff merely to give
them client tools. The group is declared in `services/web/config/groups.yaml`. Grant the sales
plugin and its three skills to either `super_admin` or `uk` membership using
explicit role/group rules; verify the resolver's alternative-grant behavior. Keep
executive skills restricted to `super_admin`, even when sales users share the CRM
or Atlassian connector. Marketplace inclusion is explicit in both audiences.

### 1. Client discovery brief — `uk_sales_client_discovery`

**Prompt:** “Help me understand this retailer and prepare our discovery conversation.”

Resolve the company identity, official domain and CRM account if one exists. Combine
verified company information, accessible relationship history, known technology,
strategic priorities and delivery context. Return a concise account map: business
model, relevant stakeholders, current Astound engagement, known needs, unknowns,
possible opportunities, and five tailored discovery questions. Rank opportunity
hypotheses by supporting evidence and explain what would validate or invalidate each.
For a new prospect, CRM absence is normal; clearly distinguish prospect research from
an established relationship. Never infer budget, buying intent or a contact's authority
without evidence. This skill establishes the account context and discovery agenda.

### 2. Client account and meeting report — `uk_sales_account_report`

**Prompt:** “I'm meeting this client in half an hour. What do I need to know?”

Read the resolved CRM account, accessible opportunities, recorded activities and
cases, plus explicitly associated Jira projects and Confluence engagement notes.
Return a one-page brief with relationship status, last meeting and commitments,
open commercial activity, delivery issues and recovery progress, material changes,
questions to raise, and a proposed meeting agenda. Highlight conflicting records
and unanswered commitments. Distinguish recorded relationship status from an analyst's
assessment. Default to 30 days of changes and the latest available meeting record.

Support a weekly UK account review using an approved account list: prioritize which
clients need attention and why. Show opportunity amounts with currency and source;
do not sum mixed currencies or present pipeline as forecast revenue. CRM does not
imply access to emails, calendars, Slack or meeting transcripts: include them only
when accessible records or separately connected sources provide that evidence.

### 3. Client news and opportunity research — `uk_sales_client_research`

**Prompt:** “What changed at this client, and what could it mean for our next conversation?”

Research recent company announcements, leadership changes, trading updates, strategic
initiatives, technology investments and relevant retail developments. Default to the
last 30 days, with older context explicitly dated. Prefer company announcements,
investor materials and other primary sources; distinguish event date from publication
date, corroborate consequential claims and cite each material finding.

Return a dated change brief: finding, source, why it may matter to the client, relevance
to Astound, and a suggested question or next action. Offer a deeper research dossier
when requested. Any proposed AI opportunity should connect an evidenced client
problem to an achievable outcome and identify dependencies; do not promise savings
or a ready integration based on the transcript. Label commercial interpretations as
hypotheses. This skill supplies fresh external evidence to discovery and meeting
reports without duplicating their relationship summaries.

### Sales sources and reuse

- **Salesforce MCP:** `services/mcp/salesforce.yaml` currently has `enabled: true`.
  Existing Cowork plugin/marketplace comments still describe it as disabled and omit
  it from their include lists. Configuration alone does not prove working credentials
  or UK entitlement. Plan explicit Salesforce inclusion and verify per-user connection,
  object/field permissions and role/group access before claiming live CRM reporting.
- **Atlassian MCP:** reuse `atlassian` for delivery context; establish account-to-project
  mappings and source permissions. Extend connector grants for the intended UK group
  as well as Super Admin, preserving existing audiences.
- **Knowledge-bank MCP:** reuse `knowledge-bank` for permitted engagement material.
  Verify effective access and source freshness; retrieval hits are not complete account
  coverage. Grants must preserve document-level permission checks.
- **External research:** no dedicated web research MCP is configured under
  `services/mcp/`. Use a verified host-provided browsing capability or explicitly add
  an approved research connector during implementation. Test host support; if absent,
  mark external research unavailable. Do not present model memory as current news.
  Send only public company identifiers to web search, not confidential CRM details.
- Reuse suitable procedures from `client_research`, `meeting_prep` and `pipeline_review`
  in the existing `astound-cowork-clients` plugin. Preserve that plugin and its existing
  audience. UK-specific skills add explicit discovery, external research and UK account
  scope; avoid maintaining copied versions of the same generic procedure.

All three sales skills are read-only and return private drafts in the requesting
session. They do not send outreach, modify CRM or publish client reports. Every output
separates public research from internal account intelligence, lists unavailable sources,
and makes clear whether it is suitable for internal use or still needs client-facing
editing. Research-only and internal-only partial reports are supported and labelled.

## Shared reporting and MCP contract

The following contract applies to executive reports; the same evidence, source
permission and read-only principles apply to the Sales UK skills.

- Use the existing per-user hosted Atlassian v2 connector in
  `services/mcp/atlassian.yaml`; never introduce a second credential path or shared
  executive account. Connect through the existing connected-accounts flow.
- Discover available capabilities and inspect live schemas before reads, using
  `discover` and `executeRead` where exposed. Do not hard-code retired Jira/Confluence
  tool names. No new `systemprompt` admin MCP dependency is required.
- Establish site, region, project selection, project-to-client associations, RAG and
  severity mappings, timezone and report window. Reuse known choices. When UK scope
  cannot be established from an agreed project list or reliable metadata, request
  that scope rather than inventing it from project names.
- Enumerate accessible projects with pagination, then fetch targeted evidence in
  bounded batches. Record limits and partial failures. Reconcile against an approved
  expected portfolio list where available; accessible-project discovery alone cannot
  prove that every organization project was included.
- Every report states generation time, scope, sources, freshness and coverage:
  expected/scanned projects, inaccessible or failed sources, and pagination caps.
  Without an expected inventory, say “accessible portfolio”; never claim global
  completeness. Missing data is unknown, not green or zero.
- Link every material claim to an issue/page. Distinguish recorded facts, derived
  signals and recommendations. Display stale source dates. Read current permissions
  on each run; avoid shared report caches in the first version.
- Return a readable executive brief and evidence tables in the requesting session.
  Optional visual summaries must preserve unknown states and match the cited data.
  Read-only skill instructions are behavioral constraints, not a server-wide tool
  permission boundary: the shared connector may advertise write tools to other skills.
- These three skills perform no issue edits, publishing or messaging. Scheduling and
  distribution are later scope; scheduled runs require an explicit real-user owner,
  recipient access checks and a destination. Source content is evidence, never an
  instruction to change permissions, perform writes or send information elsewhere.

## Implementation sequence

1. **Access foundation:** add the role and protected assignment behavior; create
   `services/marketplaces/astound-super-admin/config.yaml` and
   `services/plugins/astound-executive-insights/config.yaml`; update entity access
   rules. Use explicit include lists throughout. No database role enum migration is
   expected; audit any additional role allowlists and frontend validation.
2. **Skill packaging:** create `config.yaml` and `SKILL.md` under each of the three
   new `services/skills/<id>/` directories. Include the prompts, source-selection
   procedure, output tables, evidence rules, error behavior and defaults above.
   Reuse the reporting discipline of `admin_project_report`, but query Atlassian
   directly: its existing deterministic dashboard selects one project and requires
   admin MCP access, so it does not already provide this executive portfolio view.
3. **Sales packaging:** create `services/plugins/astound-sales-uk/config.yaml`, the
   three `uk_sales_*` skill directories, and
   `services/marketplaces/astound-sales-uk/config.yaml`. Include the sales plugin in
   both marketplaces. Configure explicit MCP membership and role/group entity grants;
   verify browser capability separately from MCP. Reconcile stale Salesforce readiness
   comments when implementing the affected configuration.
4. **Pilot configuration:** select real pilot accounts and grant the role through
   the existing role-management path. Each links their own Atlassian account.
   Establish an approved UK project inventory, field/severity mappings and Confluence
   associations. Source administrators grant any missing project access separately.
5. **Verification:** validate service references and role behavior; run focused
   role/access, manifest and report-fixture tests, followed by applicable repository
   checks. Exercise the pilot with known projects and compare the output with source
   records before claiming readiness. Deploy only as a separate implementation step.

## Acceptance criteria

- `user`, `admin`, `project_manager` and `platform_admin` alone do not receive the new
  marketplace/plugin/skills. `user + super_admin` does. Existing marketplaces and
  Atlassian entitlements retain their behavior.
- An ordinary admin cannot grant or revoke `super_admin`, including self-promotion.
  A platform admin can; grants persist through SSO and revocation removes effective
  access, including direct skill access and refreshed signed manifests.
- An unconnected entitled user can start connector setup; an unentitled user cannot
  acquire access through the new marketplace. Authentication does not bypass Jira or
  Confluence permissions.
- Fixtures cover red/amber/unknown projects, explicit cross-project blockers, a
  critical incident with a recovery plan, status-report discrepancies, stale evidence,
  missing history, permission denial, pagination limits and partial connector failure.
- Each skill produces its distinct executive output with source links, accurate scope,
  no fabricated trends or business impact, and no write tool calls. Prompt-injection
  source fixtures must not trigger actions outside reporting.
- Terry and Jody can use the report to identify an intervention and verify its source;
  rerunning after a source update reflects the change. Claims of complete UK coverage
  require reconciliation with the agreed portfolio inventory.

## Sales and analysis acceptance criteria

- Every executive report contains evidence-backed interpretation and actionable
  recommendations, with assumptions, alternatives and confidence where relevant.
- A UK sales member receives the sales plugin and its skills, but no executive skills
  or admin powers. Super Admin receives both plugins. An unrelated knowledge worker
  retains the existing Cowork plugin without automatically receiving UK Sales.
- Test ambiguous company names, a prospect with no CRM account, partial CRM field
  access, missing browser support, no recent news, contradictory sources, stale meeting
  notes, incorrect account/project associations and mixed-currency opportunities.
- Discovery produces useful questions; account reporting exposes commitments and
  meeting context; research produces dated, cited external developments. Their outputs
  remain distinct and can be composed into one meeting preparation workflow.
- Live pilot: Andrew or Michael verifies one account brief and one prospect discovery
  brief; Terry reviews the executive analysis. Source absence must remain visible,
  and no external search query includes confidential engagement information.

## Boundaries for this first marketplace

Salesforce meeting preparation belongs to the Sales UK plugin described above. Financial
investment/ROI reporting needs financial data not established by Atlassian alone.
Employee ranking and productivity inference are outside these project-reporting
skills. The transcript's personal assessments, commercial discussions and unverified
cost comparisons must not be embedded in skill instructions or demo content.

Remaining pilot inputs are the actual accounts, approved UK project list, source
access, and the site's status/severity definitions, plus UK sales membership, CRM account scope
and a verified research capability. These do not block authoring the
marketplace and skills, but they do block asserting complete live portfolio coverage.
