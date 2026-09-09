# India skills in a clean Claude Code container

This harness targets **only** `https://sp-dev.astound.digital`. It never uses the
local database, localhost fixture accounts, the host's Claude configuration, or
the staging `production` profile. A host PAT is read only for authentication;
provider tokens remain managed by the bridge.

Build the isolated image and prepare the complete 34-skill matrix:

```bash
docker build --file deploy/clean-client/india.Dockerfile \
  --tag astound-india-clean:2.1.266 deploy/clean-client
python3 scripts/india/e2e-claude-code.py \
  --output demo/india-skills/<run-id> --prepare-only --public-probe
```

The public probe runs the installer up to authentication without making model
calls. It records image identity, empty configuration, absence of mounts,
installer checksum, the default endpoint embedded in the installer, downloaded
bridge version/checksum, Claude version and elapsed time. It is not a login test.

## Authenticated runs

Use a live credential through a private file. Admin control runs use the default
`--principal admin`. To test ordinary India installation, select
`--principal india-developer --expected-user <user-id>`. This rejects admin or
extra roles and requires exactly the India marketplace, two plugins and 34 skills.
A disposable account on the live gateway is valid for this test; it must be
explicitly selected and its membership provenance recorded. PAT enrollment does
not exercise the corporate SSO assertion itself. Alternatively,
`--code-file` accepts a fresh bridge connect code, exchanges it once, and keeps
the returned PAT under `.build/india-private/<run-id>.pat` (mode 0600).
The authenticated manifest must match the selected principal and expected user.
Do not use `playwright/.auth` accounts: they are localhost test fixtures.

```bash
python3 scripts/india/e2e-claude-code.py \
  --output demo/india-skills/<run-id> --pat-file /private/live-admin.pat
```

Use the **same output folder** throughout one sweep: its ledger includes initial
turns, failures, follow-ups and retries. The authorized cap is US$5. Each local
turn has a US$0.10 client budget; Atlassian turns have US$0.20. The ledger reserves
US$0.50 for an in-flight Haiku request, limits output to 1536 tokens and stops
launching calls when accounting is missing. For longer local reviews, use
`--max-turns 20 --max-output-tokens 4096`; both values are recorded per attempt
and the shared spend cap still applies. It pins Haiku 4.5, disables subagent
and shell tools, and checks the reported resolved models. These small budgets
can interrupt a case; an interrupted case is not a pass. The CLI checks a
budget between requests, so the reservation matters. See the
[Claude Code CLI reference](https://code.claude.com/docs/en/cli-reference).

For server reconciliation set `INDIA_AUDIT_DATABASE_URL` securely to the **live
Astound audit database**. Queries are read-only and correlate request/session IDs,
MCP execution IDs, traces, model, cost and tokens. There is deliberately no local
or staging fallback. Without this evidence a live E2E pass cannot be recorded.
Client timings and cost reports are still retained separately.

Every independent case gets a new Docker container with an empty home. The
installer must automatically provision the skills; the first run never copies
skills from the repository. Installed marketplaces, plugin registry, enablement,
skill bodies, supporting references and active cache are compared with the
server manifest. Extra admin grants are reported; ordinary developer isolation
is checked by `--principal india-developer`; an admin control cannot pass that check.

## Reviewed Atlassian writes and stateful scenarios

First use `--only <skill-id> --keep-clients` to collect a proposal. Review the
actual proposed fields and target against the scenario rubric. Write the exact
approval and next action into a local text file, then resume that same scenario:

```bash
python3 scripts/india/e2e-claude-code.py \
  --output demo/india-skills/<run-id> --pat-file /private/live-admin.pat \
  --only ba_confluence_management --keep-clients

python3 scripts/india/e2e-claude-code.py \
  --output demo/india-skills/<run-id> --pat-file /private/live-admin.pat \
  --only ba_confluence_management --followup-file /private/reviewed-turn.txt \
  --enable-reviewed-writes --keep-clients
```

The follow-up must identify the reviewed space, parent page and page title (or
Jira issue fields). Use existing `EAICOMM` projects/spaces, confirmed through live
MCP discovery. Only new items carrying `india-skills-demo` and the exact run
marker may be changed. Never alter existing unrelated items or project settings.
The first turn denies `executeWrite`; reviewed follow-ups enable that capability.
Actual MCP tool names must match the installed Atlassian v2 discover/execute
schema; other provider capabilities are not assumed.

For dependent BA workflows, first create and read back the run-labelled FSD,
then an epic, then stories/bugs. Use the confirmed current Atlassian account for
any stakeholder fixture; do not invent an approver. Requirements tracing, impact,
CR, sprint and release checks read this remote evidence. Missing FSD, stakeholder,
board, issue type or connector capability is a blocked case, not permission to
substitute a local mock for remote evidence. Use extra follow-up turns for
changes, read-back, sequential RTM updates and cleanup as the rubric requires.
The harness prepares prompts; it does not automatically approve model proposals.

Session memory uses a follow-up without `--enable-reviewed-writes`: explicitly
read its saved correction in a later turn and verify the value. Notifications
produce unsent drafts; reporting produces a proposal, not an installed schedule.
Salesforce scenarios use labelled local fixtures and never establish actual org
compilation, deployment or runtime behavior.

Capture remote IDs, URLs, fields, read-back and cleanup outcomes in each attempt's
`output/remote-items.json`. Delete only this run's created items when the advertised
connector supports deletion; otherwise record retained demo URLs. Remove retained
clients after exporting evidence:

```bash
python3 scripts/india/e2e-claude-code.py --output demo/india-skills/<run-id> --cleanup
```

Retained containers hold credentials for the scenario. They are never reused by
another case. Only containers labelled with this exact run ID are removed.

## Review, failures and local fixes

A successful process exit, model narration or keyword match does not establish
functional correctness. Each case includes three explicit rubric criteria;
mechanical checks catch file corruption and missing evidence, and a reviewer
checks semantic correctness. Initial execution yields `REVIEW REQUIRED`, never
an automatic full pass.

Use `--review-file` with a JSON object keyed by skill ID. Each entry names the
latest `attempt` number and `checks` in the exact order of `acceptance.json`'s
`rubric`: each check includes `criterion`, Boolean `passed`, and an `evidence`
array of file paths relative to that attempt. Include reviewer identity and
notes. All evidence files must exist in the attempt. This cannot convert an
unrun/failed execution into a pass. A live pass also requires correlated successful
Atlassian server executions. Failed criteria remain failures.

Use `--retry --only <skill-id>` for a fresh live-installed rerun. Use
`--retry --local-skill-files --only <skill-id>` to test a local patch **after a
preserved live attempt**. The patch is overlaid only after live installation
proof is saved, and results are labelled `LOCAL PATCH`; this is never deployment
proof. No automatic deployment is performed.

Sanitized evidence lives in the output folder; private diagnostics and credentials
stay in `.build/india-private/`. Scan/review exported provider content before
committing it. The current demo contains no authenticated provider content.

## Harness checks

```bash
python3 -m unittest discover -s tests/unit/india_harness -v
just test-unit
just lint-gates
```

Python checks cover accounting, grant mismatches, redaction, invocation evidence,
CSV preservation, scenario completeness and import consistency. Repository unit
tests also reject obsolete imported tool names and broken Cursor skill paths.

## Local bridge fixes and live audit

`--installer-file scripts/install-bridge.sh` tests the container timer fix while
still downloading the published bridge. `--bridge-file <binary>` then replaces
that binary inside each isolated client and resyncs from the live gateway.
Evidence records the installed binary hash and labels reviewed results as a local
bridge patch. Neither option deploys a release.

Instead of a database credential, set `INDIA_AUDIT_CONTAINER` to an enrolled
admin Docker client scoped to the live gateway. Audit SQL runs read-only through
the governed systemprompt MCP. The developer client obtains its own proxy session
ID from a free `knowledge-bank/list_knowledge_sources` call; it never receives
admin tools or credentials. This avoids assuming Claude's session UUID matches
the gateway's session ID.

Keep one shared budget ledger per identity and explicitly carry prior task spend
when changing principals. The September developer run includes the earlier admin
control cost in its US$5 cap. Do not sum that carried cost twice.

On hosts where `docker` resolves to a WSL/Windows shim, use
`INDIA_DOCKER=/usr/bin/docker` to select the native client explicitly. This run
found that the shim changed fixture-copy semantics and intermittently failed
before model execution. Those failed attempts remain in the ledger.

The matrix totals model costs and model wall time across retries.
`e2e_wall_seconds` measures enrollment through evidence collection, excluding
image build and cleanup; older attempts have no full-run timing. A Salesforce
`LOCAL ARTIFACT PASS` means the saved proposal met its rubric, not that it
compiled, deployed or passed tests in Salesforce.
