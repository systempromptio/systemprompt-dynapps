# India skills demo — 9 September 2026

**Live functional testing is blocked on the owner's admin login.** No skill
execution, authenticated installation or Atlassian write has been claimed as a
pass. All 34 starting prompts and acceptance rubrics are prepared in the
[testing matrix](matrix.md).

## Observed results

| Check | Result |
|---|---|
| Clean Docker image | Built; Claude Code 2.1.266; no host mounts or inherited client configuration |
| Live model catalog | Haiku 4.5 dated model advertised |
| Published installer default | **FAIL:** defaults to `astound.systemprompt.io`, although downloaded from `sp-dev.astound.digital` |
| Explicit live endpoint download | **PASS:** archive checksum verified by the published installer |
| Downloaded bridge | `0.48.0`, commit `68849c4`, build date 2026-09-08 |
| Public setup duration | 5.351 seconds through the authentication gate, including container lifecycle; image build excluded |
| Admin login and installed grants | **BLOCKED:** no live admin credential supplied |
| 34 Haiku skill scenarios | **NOT EXECUTED:** authentication prerequisite missing |
| Ordinary India developer isolation | **UNVERIFIED:** admin-only scope chosen |
| Salesforce/GitHub live integrations | Excluded; Salesforce local fixture scenarios prepared |
| Model spend | **US$0:** no inference requests issued; per-skill costs and speeds unknown |
| Atlassian changes | None; [remote inventory](remote-items.json) is empty |

The [public onboarding evidence](onboarding-public.json) contains image and binary
hashes. The [installer output](onboarding-public.txt) shows successful download
and checksum verification followed by sign-in failure without a terminal/login.
This is a public bootstrap check, not evidence of a completed login.

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

Provide a private PAT file for the owner's admin account on the live endpoint,
then resume the same ledger:

```bash
python3 scripts/india/e2e-claude-code.py \
  --output demo/india-skills/20260909-admin-haiku \
  --pat-file /private/live-admin.pat
```

A fresh connect code can instead be supplied via `--code-file`. The full
[runbook](../../../scripts/india/README.md) explains scenario selection, reviewed
Atlassian writes, audit-database access, evidence review, local patched retests,
cleanup and the shared **US$5** limit. Relative paths in the command assume the
repository root. No recurrence or deployment is configured by this task.
