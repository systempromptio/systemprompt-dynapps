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

## Functional sweep

The [matrix](matrix.md) tracks 22 local BA/Salesforce fixture scenarios and 12
Atlassian scenarios. Each executed scenario starts in a fresh container with no
mounts or inherited Claude configuration, uses Haiku 4.5, and saves its prompt,
response, tool calls, artifacts, wall time and live audit reconciliation.
`REVIEW REQUIRED` is not a functional pass. Rubrics are reviewed against saved
artifacts and tool calls. Salesforce/GitHub live integration remains excluded.

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

The developer ledger carries the earlier **US$0.015379** admin control cost, so
its cumulative total is the task total; do not add the admin total a second time.
All attempts and retries share the **US$5** authorization. See [results](results.json).

```bash
INDIA_AUDIT_CONTAINER=<live-admin-control-container> \
python3 scripts/india/e2e-claude-code.py \
  --output demo/india-skills/<new-run> \
  --principal india-developer --expected-user <non-admin-user-id> \
  --pat-file /private/india-user.pat \
  --installer-file scripts/install-bridge.sh \
  --bridge-file bridge/target/release/astound-bridge
```

The private PAT never belongs in this folder. Binary hashes and bootstrap origin
are recorded per attempt. No recurring monitoring or live release is configured.
