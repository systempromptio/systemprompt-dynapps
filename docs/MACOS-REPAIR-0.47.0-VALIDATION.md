# v0.47.0 macOS repair: validation checkpoint

Date: 2026-09-07. Current status: published repair and test-environment deployment
verified; full-system acceptance is not green.

## Latest core and test-environment deployment

This section supersedes the historical checkpoints below. The environment
`https://astound.systemprompt.io` is our test environment, named `production`
in the local profile. It is **not** `https://sp-dev.astound.digital`, the
customer endpoint. No deployment to that customer endpoint was performed.

- Core `next`: `c0fafb36dcecfee02f44a47f4cc7de9abf913905`, vendored locally,
  with matching 0.47.0 pins and active patches in both Cargo workspaces.
- Deployed runtime source: `4aa531c8`; subsequent `431245d2` changes test
  fixtures only. Fly app: `sp-a2f658d8bc5f`.
- Deployed image digest:
  `sha256:35487d61f0c2c2d24049faa643dfbbc55896b39fe84227ad2a2af289bd6b9f57`.
- Actual served/downloaded DMG SHA-256:
  `f7cd861a24e422fc6ed29242f53d903c77ed47d4fbf40c3cf11bae25b6dbb4d7`.
- App notarization: `1925b445-a914-48c2-9e7a-6fb609cc22b9` (Accepted).
- DMG notarization: `a6a1a9be-de30-4193-9321-ec93c7f727a3` (Accepted).

The Linux server and both MCP binaries were built from the vendored source.
Deployment reused that built image with a correction to the runtime profile's
ownership; the retained full-build Dockerfile includes that correction and
asserts readability as the unprivileged runtime user. The resulting image
contains no PEM files, secrets.json or credentials.json under `/app`.
The downloaded credential archive contained a stale standalone signing key;
the local profile now uses the existing server key from its secret store,
verified against the public JWKS. This did not rotate the server key.

After deployment, `/readyz` returned HTTP 200 with status ready. The actual
DMG downloaded from the test environment passed `tests/release/macos-smoke.sh`
on this Intel Mac: Developer ID/team, hardened runtime, universal architecture,
app and DMG notarization/staples, Gatekeeper, and native CLI/login URL checks.
The app was copied from the mounted DMG and tested after ejecting it.

### Native Claude Code proof

The live-downloaded app enrolled against the deployed gateway, ran its native
loopback proxy, and minted gateway credentials. Native terminal Claude Code
2.1.260 made a real `mcp__systemprompt__systemprompt` call with command
`core skills list`, received a 13,969-byte tool result, and returned the first
skill, `admin_activity_report`. PreToolUse governance and PostToolUse tracking
HTTP hooks both returned 200. The call cost $0.0472476.
The server independently recorded three completed Anthropic requests with
input/output/cache token counts and a total of 47,248 microdollars (the same
cost rounded to its storage precision). Final request:
`966c37db-d001-45af-8924-b2a375df3d53`.

The server audit independently confirms the call at 2026-09-07 14:48:31 UTC:

- Hook decision `9e53bbbc-fb9d-43e8-b9fb-34d97757b749`: allow/default_allow,
  plugin `astound-dev`, tool `mcp__systemprompt__systemprompt`.
- MCP authorization trace `6412f95d-5164-4b1b-8364-b39ff7560fb4`, including
  decision `7d61e168-cadd-47a3-9bcb-2bf44e0003ab`: allow/authz.

These are real calls, not browser-seeded audit rows. Tests used isolated XDG
and Claude configuration directories with explicit MCP/hook settings; no
global Claude bridge install/sync was performed. SessionEnd and external
connector OAuth flows are not covered by this proof.

Normal `admin bridge issue-code --user-id ...` succeeded for the owner's
account. The `--json` variant fails: core reconstructs global flags before
the subcommand, while `/api/v1/admin/cli` requires a lowercase subcommand as
the first argument. A direct request with leading `--json` returned HTTP 400
with that validation error. The authenticated profile bridge-code endpoint
also succeeded and was used for the native enrollment above.

The live-downloaded app's `update --check` exited 3: `/v1/bridge/latest`
returned 404. The deployment has no `gateway.bridge_releases` feed configured;
core intentionally returns 404 without that feed. Automatic update acceptance
therefore remains open, independently of the verified direct download.

### Full-suite checkpoint and disk cleanup

The full local Chromium run against the latest core completed with **761
passed, 42 failed, 20 skipped**. Two test-harness failures were then fixed
(indentless YAML list parsing and expectations for entitled but unconnected
external connectors); the focused regression run passed all **21 tests**.
The other **40 local failures remain unresolved**, including sorting/filtering,
membership removal, KPI/counts, catalog presentation, layout and accessibility.
Snapshot comparisons were disabled because these are native Mac runs against
Linux baselines; this is not visual-regression approval.

The full remote Chromium run against `astound.systemprompt.io` completed in
12.5 minutes: **761 passed, 41 failed, 20 skipped, 4 did not run** (826 total).
This includes failures in the mock-provider flows: the deployed server cannot
reach that test upstream and returns 502, unlike the successful real Anthropic
call above. The warn-mode serial suite's four following tests did not run
after its first failure. UI and requirements failures also remain; the remote
suite is not green. CSV export presentation, manual group-member removal,
sorting/filtering and user-page accessibility are examples requiring follow-up.

Rust CI run `34132973907` for `431245d2` **passed all four jobs**: static,
lint, tests and coverage (floor + ratchet). **861 unit, 794 integration and
107 admin-contract tests passed**, with zero skipped in those three suites.
The previous integration compile failure against the newly fallible governance
constructor is fixed in `431245d2`. This green Rust gate does not waive the
separate browser acceptance failures above.

No external connector was granted access merely to make acceptance tests pass.

Removed regenerable root/test debug build artifacts using Cargo cleanup
(50.8 GiB and 29.4 GiB reported respectively); release binaries, source,
credentials and databases were preserved. After rebuilding/deploying, the
disk has approximately 56 GiB free. Local service startup and migrations
passed with the new core. Knowledge-bank uses a dedicated local pgvector
container; the existing PostgreSQL and unrelated Docker workloads were not
replaced.

Current private evidence workspace: `/tmp/astound-deploy-test-bootstrap`.
Deployment log: `deploy-permission-repair.log`; native downloaded-app evidence:
`live-client/`; full local browser artifacts: `local-e2e-full-report/` and
`local-e2e-full-results/`; remote browser log: `remote-e2e.log`.
These temporary files can contain credentials and must not be uploaded.

## Publication update

The owner requested integration into `next`, branch cleanup and correction of
the production binary. All repair work was merged into `next` at `40d92c44`,
preserving concurrent CI improvements. PR #3 is merged and the temporary branch
was deleted locally and remotely. The direct-`next` working agreement is now
recorded in `AGENTS.md`.

Publication run
https://github.com/Astound-Digital/systemprompt-astound/actions/runs/34124758728
passed all three repair jobs. GitHub release v0.47.0 now contains the candidate
DMG below; GHCR `0.47.0` and `latest` both resolve to the candidate digest below.
The published GitHub DMG was downloaded again and passed Intel packaging and
CLI smoke tests; the same publication run passed native Apple Silicon tests.
Original assets are retained in draft release
`repair-backup-v0.47.0-e5341e3b2cec`, and original image digests are pinned by
`backup-0.47.0-e5341e3b2cec` and `backup-latest-e5341e3b2cec`.

CI run 34120631174 ultimately passed all four jobs, including coverage. The
newly merged `next` has its own gate run; the previous result is not a claim
that the merge run has completed.

The live endpoint became reachable and reports healthy v0.47.0. However, an
actual download from it still hashes to
`cb3e69cdd68e6a836969e4a6ea54570edea64b75876def1c85ee2ba0f9e0ec66`
and fails `verify-bridge-macos.sh`. Production must be redeployed using the
repaired image, and its served download must match the candidate SHA below.
Deployment host/user or orchestration access is still needed; no production
deployment has been performed. The UI/MCP acceptance gaps below remain open.

The remainder records the earlier pre-publication checkpoint; its hold and
network-timeout statements are historical, superseded by this update.

## Reproducible signing

All eight Apple signing/notarization secrets are provisioned in the Astound
GitHub repository. `scripts/provision-apple-signing.sh` validates the identity,
private-key match, Apple API credentials and macOS PKCS12 import before upload.
No credential values belong in this report or the repository.

The shared Apple action requires credentials; production packaging refuses an
unsigned or unnotarized download. Both the universal app and DMG are notarized
and stapled. Existing-version image builds reuse published downloads so their
bytes do not diverge from the GitHub release.

- Candidate build: https://github.com/Astound-Digital/systemprompt-astound/actions/runs/34120899194
- Exact-artifact reuse and native ARM smoke: https://github.com/Astound-Digital/systemprompt-astound/actions/runs/34121710915
- Build source: `e6e07890`; pinned core: `74a90fbef2817c885caef82aa2880d39e9b68267`.
- DMG SHA-256: `e5341e3b2cec47e51ca7c10f1e49156de3e7b85e5f4a2c6b6a4389df41d4e693`.
- Candidate image: `ghcr.io/astound-digital/systemprompt-astound:repair-0.47.0-34120899194`.
- Candidate digest: `sha256:91ce3dd4c3f53e9dcf272ac8e5b48798ec5b85a8d88b2a71b6741b8a4a4be01d`.
- Original image digest: `sha256:c23551612ff0e16f4a4eadf33f49d56850e249a3398a404757a86be33da9190f`.

The existing v0.47.0 release assets and `0.47.0`/`latest` image tags have NOT
been replaced. Publication uses the tested candidate without rebuilding, and
backs up original release assets plus both old image tags.

## Completed tests

| Surface | Result |
| --- | --- |
| Downloaded CI DMG, Intel Mac | Signature/team, hardened runtime, both architecture slices, notarization tickets, Gatekeeper, copied app after eject, native CLI smoke passed |
| Same DMG, Apple Silicon CI | Native CLI smoke and packaging verification passed |
| Login URL selection | Default gateway, explicit override, saved configuration, override precedence and failed-login config preservation passed |
| Isolated published-image harness | 60 passed, 0 failed, 2 skipped |
| Targeted Chromium auth/enrollment/manifest/access/KB suite | 41 passed, 4 skipped |
| Full Linux Chromium suite | 754 passed, 48 failed, 21 skipped |
| Real Claude Code and OpenCode inference | Both returned PONG through the candidate gateway |
| Rust/static CI run 34120631174 | Static, lint and unit/integration/admin-contract jobs passed; coverage still running at this checkpoint |
| Latest workflow/script edits | actionlint, shellcheck and git diff whitespace checks passed |

The isolated image harness checked a three-node TLS gateway, primary/replica
PostgreSQL, migration coordination, cross-node PAT inference, real Anthropic
usage/cost auditing, restart persistence, exact download hashes, server
tarballs and a downloaded clean Linux bridge installation. Its two skips were
bridge-to-gateway self-signed TLS trust and native Windows execution. The four
targeted browser skips are explicitly unimplemented invitation tests.

## Outstanding acceptance failures

The 48 full-suite failures comprise 29 design/accessibility assertions, 12
page-behaviour assertions and 7 requirements assertions. They are not waived.
Examples include sorting/filtering, manual group-member removal, KPI/count
expectations, context-detail accessibility, and catalog/scope presentation.
The plugin-catalog failure expects a `.sp-catalog-card` that is absent from the
rendered page; the MCP detail failure expects visible `Scopes` text that is
absent. These findings require triage, not blind snapshot updates. The image
under test retains the original server; only its macOS download was replaced.

Actual bridge MCP calls and SessionEnd hooks have NOT passed end-to-end:
the local TLS fixture uses a self-signed CA, and bridge upstream calls fail
certificate trust. A separate HTTP entry point still received advertised
`https://sp.local` MCP URLs. Real inference success does not prove tool or hook
success. Browser-seeded audit rows are also not evidence of real tool calls.

Live `https://sp-dev.astound.digital` resolves to private IP `10.13.1.35` and
HTTPS connections time out from this machine. A working Astound network/VPN
route and the deployment method are needed before live SSO, enrollment,
download, MCP/hook and deployment acceptance can be completed. The old Fly
production profile is not this endpoint and must not be used to deploy it.

## Retained local evidence

Temporary workspace: `/tmp/astound-release-repair.KhhX7n`.
The `ci-candidate/` directory contains the downloaded Actions artifact;
`original-assets/` contains the original release backup. The browser report
and failure traces are under `browser-repo/playwright/playwright-report/`
and `browser-repo/playwright/test-results/`. The isolated stack is retained
under compose project `e2e243708` for investigation. These local artifacts
are temporary, and runtime profile files contain secrets: do not upload them.

Before publication, resolve acceptance findings, verify the remaining CI
coverage result, and use the explicit candidate-reuse publication procedure
in `docs/RELEASING.md`. Do not describe this checkpoint as a fully functional
or deployed release.
