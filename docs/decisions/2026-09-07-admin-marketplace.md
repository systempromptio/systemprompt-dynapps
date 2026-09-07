# Astound Admin marketplace and reporting

The private `systemprompt-admin` marketplace owns the existing `astound-admin`
plugin. Both admin roles receive it, including admins in `india-devs`. The general
`enterprise-demo` marketplace retains everyday development resources. Atlassian
remains per-user authenticated; granting its connector never grants Atlassian data.

Migration 047 removes the five former India admin exclusions that the bootstrap
loader persisted. It matches their original bootstrap justification and preserves
operator-authored overrides, role grants, and the knowledge-bank exclusion.

Five workflows cover delivery, cost/adoption, governance investigation, user access,
and AI operations. Reporting is read-only; operational skills perform requested
changes and verify the target afterward. The existing Slack agent remains attached.

`admin_report` returns structured source-labelled report tables and a deterministic
MCP Apps artifact. `admin_project_catalog` populates the project artifact's selectors.
The same maintained HTML is staged into Cowork. Design styles and the installation
workflow were migrated from the sibling systemprompt-internal control plane; Odoo
and approval tools were not imported. Existing artifact IDs remain stable.

The Atlassian connector uses the vendor-supported `?tools=all` endpoint so the
gateway advertises a complete, paginated catalog. The deterministic adapter calls
only known read tools present in that catalog; it does not guess deferred execution
schemas. See [Atlassian's tool exposure contract](https://developer.atlassian.com/cloud/rovo-mcp/guides/supported-tools/).

Only `systemprompt_setup_admin` installs Cowork library artifacts. Its `hosts: [cowork]`
restriction must be enforced by host projection, including Claude Code's copied
plugin trees, not merely described in the skill text.

Report limits are explicit: directory/request/model pages are bounded; Jira collects
at most 1,000 issues, with incomplete coverage reported. Confluence shows the selected
space's pages, not inferred project associations. Project metrics are deterministic
facts; no AI-cost-per-project or inferred productivity metric is available here.

Validation: `just preflight-static`, `just clippy`, `just test-unit`, relevant manifest
integration tests, and `cd playwright && npx playwright test -c admin-artifacts.config.ts`.
The isolated artifact browser tests need no database, authentication or live Atlassian.

| High-value action | Evidence and outcome |
| --- | --- |
| Project delivery review | Jira status, overdue and unassigned work, stale issues, completions, and optional Confluence space pages; scope selectors and source links live in the artifact. |
| AI cost and adoption review | Spend summary, model usage, cost trends, and session statistics; distinguish aggregate statistics from bounded samples. |
| Governance investigation | Request activity and statistics, followed by trace-specific CLI investigation; explain denied or failed activity using recorded evidence. |
| User and access review | Paginated account directory and account statistics, followed by inspection and verified changes to a specifically requested account or grant. |
| AI operations review | Providers, routes, rate limits, agents, MCP health, jobs, and evaluation runs; distinguish configured state from effective runtime behavior. |

Changes are performed by the workflow skills through the authenticated CLI or
Atlassian tools only when requested. Artifact controls remain read-only. Neither
setup nor reporting requires paid model calls; evaluation execution and publishing
have their own user-requested scope.

After rollout, restart the service so startup migration and configuration ingestion
run, then sync the bridge. Admins receive the separate marketplace automatically.
In Cowork, run `systemprompt_setup_admin` to reconcile the Artifacts library. Other
MCP Apps hosts use the artifacts returned directly by `admin_report`, for example
`{"report":"costs","days":7}`. Project reports require the caller to link Atlassian
and select their own site/project; optional Confluence space selection is explicit.

Local verification on 2026-09-07 covered 873 unit tests, 797 integration tests,
44 marketplace/manifest browser tests and four isolated artifact browser tests.
The live local server returned activity, cost (including trends), user and operations
reports with MCP UI resources. Fresh admin and platform-admin principals received
the signed marketplace both inside and outside India; an India user did not.
Non-admin report calls were denied, and an unlinked Atlassian account produced an
explicit connector failure. No local Atlassian accounts were linked, so live vendor
content was not verified; transport, pagination and project metric behavior were
verified against structured MCP fixtures. Core CI passed at `1ddfc17b6`.
