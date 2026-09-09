# Ordinary India developer — live gateway, clean Claude Code

This run corrects the earlier admin-only test. A disposable **non-admin** account
was created on `https://sp-dev.astound.digital` through the owner's authenticated
admin session, with roles `developer,user` and manual membership of `india-devs`.
The owner's roles, group membership and connector credentials were not changed.

## Installation result

The published bridge installed **exactly 34 India skills**, the two India plugins,
and all bundled references. No Platform/Admin skills appeared. Skills matched the
live manifest and the active Claude cache. See [onboarding](onboarding.json),
[principal checks](principal-checks.json) and [identity](whoami.json).

The complete published installation was **not a pass**: plugin MCP files included
Atlassian, Salesforce and GitHub even though the user manifest withheld those
unconnected providers, and omitted the granted shared knowledge-bank connector.
The published Linux installer also required a local fix to tolerate containers
without a user systemd instance.

A locally built bridge patch projects MCP configuration from the user's signed
manifest and clears stale connector entries. A fresh-client retest passed every
installation check, including MCP configuration, skill bodies and active caches:
[comparison](skills/ba_tracker_management/attempt-02/installation.json).
**This is a local bridge patch, not a deployment to the live download endpoint.**

The [admin route denial](admin-route-denial.json) also returned HTTP 404 for the
ordinary account. The [membership removal test](revocation.json) passed: after
removing only the disposable user's manual India membership, the manifest,
marketplace mirror, enabled registry and active cache all removed India skills.
Only `code_review`, `git_commit` and `systematic_debugging` remained. The observed
61.12-second refresh includes the normal 60-second cache TTL for direct SQL changes.

## Functional sweep

The [matrix](matrix.md) tracks 22 local BA/Salesforce fixture scenarios and 12
Atlassian scenarios. Each executed scenario starts in a fresh container with no
mounts or inherited Claude configuration, uses Haiku 4.5, and saves its prompt,
response, tool calls, artifacts, wall time and live audit reconciliation.
Final reviewed outcomes: **4 local BA functional passes, 12 Salesforce artifact
passes, 6 rubric failures, and 12 Atlassian cases blocked before model execution**.
Salesforce artifact passes cover proposed files only; no Salesforce code was
compiled, deployed or run in an org. Salesforce/GitHub live integration remains excluded.

The six remaining failures are:

| Skill | Remaining defect in Haiku output |
|---|---|
| sf_bypass_patterns | Missing required QA cases and separate production recursion guard |
| sf_custom_metadata_driven_config | Missing invalid-configuration test |
| sf_profile_permissionset_patterns | Unrequested Create and AllowViewKnowledge grants |
| sf_email_notification_pattern | Incoherent proposed send-action configuration |
| sf_integration | Failed-batch retry drops remaining leads; missing timeout/duplicate mocks and broken test calls |
| sf_ui_framework | Unsupplied Name field, unreachable loading state, unsupported accessibility/test claims |

See [targeted reviews](retest-reviews.json) and [final reviews](final-reviews.json).
Instruction patches improved six targeted artifact results, but the remaining
failures mean the skill set is **not fully working with Haiku**.

The first tracker attempt exposed a harness fixture-placement defect: Docker
copied the temporary directory into the work root. Haiku found and edited those
files, but collection looked in the intended paths. The failed attempt is kept;
the corrected fresh run passed the mechanical checks. Its cost is still counted.

Skill output review found missing profile-retrieval guidance, an incorrect
production recursion guard and inconsistent metadata-rule priority handling.
Local instruction fixes are preserved separately from the delivered skill bodies.

## Identity and provider limits

This tests real live PAT enrollment for an explicitly provisioned ordinary India
member; it does not replay a corporate SSO assertion. Live directory evidence
shows four existing users mapped through `Systemprompt-Core` into `india-devs`:
[directory cohort](directory-cohort.json). The test account's manual membership
must not be presented as an ADFS login result.

The identity endpoint reports Atlassian **configured and entitled**, but
**not_connected**. No remote Jira/Confluence write has been made. The ordinary
India cohort has no connected provider account in the live account-state table;
the owner's admin account also has no connector-state row. Provider credentials
are not copied between identities. Existing Atlassian dev projects remain the
required targets when a linked test account is available.

## Cost and reproduction

**Total measured task spend: US$1.531400**, including all failed attempts,
follow-ups, retries and the admin control. The latest attempt of each of the
22 local cases totals US$0.926318; that is not a clean first-pass sweep estimate.
There are 40 preserved developer attempts (including pre-model failures).
Recorded model wall time totals 1,481.47 seconds across attempts. The 11 recent
runs with complete enrollment-to-evidence timing range from **50.37–103.29 seconds**,
with median **58.79 seconds**. Earlier attempts retain model timing only.
Per-skill costs, model seconds and available full-run seconds are in the matrix.

The developer ledger carries the earlier **US$0.015379** admin control cost, so
its cumulative total is the task total; do not add the admin total a second time.
All attempts and retries share the **US$5** authorization. See [results](results.json).

```bash
INDIA_DOCKER=/usr/bin/docker \
INDIA_AUDIT_CONTAINER=<live-admin-control-container> \
python3 scripts/india/e2e-claude-code.py \
  --output demo/india-skills/<new-run> \
  --principal india-developer --expected-user <non-admin-user-id> \
  --pat-file /private/india-user.pat \
  --installer-file scripts/install-bridge.sh \
  --bridge-file bridge/target/release/astound-bridge
```

The private PAT never belongs in this folder. Binary hashes and bootstrap origin
are recorded per attempt. Monitoring during the sweep reconciled model usage with live gateway audit
records; no recurring paid schedule or live release is configured. Published
onboarding still needs the reviewed installer/bridge changes released before it
can be called fixed out of the box.

## Cleanup

Both disposable PATs were revoked, the test account was marked inactive, and its
India membership was removed. A new token exchange returned HTTP 401. All run-owned
containers were removed after evidence export. The owner's account was unchanged;
no Atlassian items or provider configuration were changed. See [cleanup](cleanup.json).

## Demo and validation

Start with the tracker [prompt](skills/ba_tracker_management/attempt-02/prompt.md),
[response](skills/ba_tracker_management/attempt-02/output.md) and
[saved artifacts](skills/ba_tracker_management/attempt-02/artifacts/).
The utility-bar [retest](skills/sf_app_utilitybar_pattern/attempt-02/output.md)
cost US$0.033950 and took 50.37 seconds including setup/evidence collection.
Every matrix row links its starting prompt; attempt folders retain outputs and metrics.

[Monitoring summary](monitoring-summary.json) records per-attempt costs and
correlation status. [Validation](validation.json): 17 Python tests passed, the
local bridge built, and core CI bridge regression plus Quality checks passed.
Repository lint gates currently fail in two concurrent, unrelated admin source
changes; those failures are recorded and those files were left untouched.
