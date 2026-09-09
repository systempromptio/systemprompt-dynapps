# India skills demo — 9 September 2026

**Live admin login works; India skill execution is blocked by workspace access.**
The supplied connection code was redeemed successfully for `e.burton@astound.local`.
The live manifest grants 11 Platform/Admin skills and none of the 34 India skills.
A temporary India workspace grant is awaiting owner approval; no access was changed.
All starting prompts and acceptance rubrics are in the [testing matrix](matrix.md).

A clean-client Haiku control returned **READY in 3.714 seconds**, with live audited
cost **US$0.015379**. See [metrics](connection-control/metrics.json),
[output](connection-control/output.md) and [audit](connection-control/audit.json).
This verifies authentication, model routing and billing, not skill functionality.

## Observed results

| Check | Result |
|---|---|
| Clean Docker image | Built; Claude Code 2.1.266; no host mounts or inherited client configuration |
| Live model catalog | Haiku 4.5 dated model advertised |
| Published installer default | **FAIL:** defaults to `astound.systemprompt.io`, although downloaded from `sp-dev.astound.digital` |
| Explicit live endpoint download | **PASS:** archive checksum verified by the published installer |
| Downloaded bridge | `0.48.0`, commit `68849c4`, build date 2026-09-08 |
| Public setup duration | 5.351 seconds through the authentication gate, including container lifecycle; image build excluded |
| Admin login and installed grants | **PASS login:** 11 Platform/Admin skills; **0/34 India skills granted** |
| 34 Haiku skill scenarios | **NOT EXECUTED:** India workspace access missing |
| Ordinary India developer isolation | **UNVERIFIED:** admin-only scope chosen |
| Salesforce/GitHub live integrations | Excluded; Salesforce local fixture scenarios prepared |
| Model spend | **US$0.015379:** Haiku connection control only; per-skill costs and speeds remain unknown |
| Atlassian changes | None; [remote inventory](remote-items.json) is empty |

The [public onboarding evidence](onboarding-public.json) contains image and binary
hashes. The [installer output](onboarding-public.txt) shows successful download
and checksum verification followed by sign-in failure without a terminal/login.
This is a public bootstrap check, not evidence of a completed login.

The authenticated published installer also exits early when `systemctl --user`
is unavailable in Docker. A local installer fix skips timer setup and starts a
session proxy; a second fresh container confirmed setup completes with that fix.
This fix is not deployed. The [installation comparison](installation-before-india-grant.json)
also records an admin MCP configuration mismatch; the advertised Atlassian route
returned 404. These are recorded defects, not successful India skill tests.

## Review findings and local fixes

- Four Salesforce skill bodies contained six obsolete `.cursor/skills/...` links;
  one also named an agent that the marketplace does not install. These now
  reference installed skills.
- Bundled project rules described an upstream Cursor agent roster and mandatory
  branch/deploy workflow. They now explicitly defer to current project/user
  instructions and identify the roster as review perspectives, not installed
  agents. Historical setup documents are labelled as source templates.
- Notification, scheduled-reporting and transcript descriptions still promised
  sending, job operations or BA-platform persistence. Descriptions now match
  unsent drafts, report/schedule proposals and workspace Markdown. Reporting no
  longer instructs the client to inspect nonexistent job history.
- Jira instructions claimed automatic ADF conversion and bulk writes in a single
  call. They now require the advertised content schema and individual approved
  updates with partial-failure reporting.
- The India marketplace documentation incorrectly claimed deny rules excluded
  Europe and Cowork universally. Current project/role grants can legitimately
  add these workspaces; an admin's catalog is not the ordinary developer catalog.

The [original source review](source-review.json) and
[post-fix source review](source-review-after.json) preserve the before/after
hashes. The import script retains these adaptations on future imports, and the
inventory now records supporting-reference hashes. **These fixes are local;
they have not been deployed or functionally verified against the live account.**
The published installer's embedded staging default remains a live release issue;
the harness explicitly pins both gateway and download URL to the requested host.

## Validation and continuation

Repository unit tests: **907 passed**. Source lint gates: **33 passed**. Focused
Python harness checks: see [validation results](validation.json).
These checks do not establish that the 34 skills work under Haiku.

The redeemed login is retained privately; another connection code is unnecessary.
Continuation requires approval for a temporary India workspace grant, then fresh
container installation and the 34 scenarios. The [runbook](../../../scripts/india/README.md)
describes reviewed Atlassian writes, audit correlation, local patched retests,
cleanup and the shared **US$5** cap. No deployment or recurring monitor is configured.
