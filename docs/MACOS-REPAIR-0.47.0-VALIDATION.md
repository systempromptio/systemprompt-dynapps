# v0.47.0 macOS repair: validation checkpoint

Date: 2026-09-07, updated 2026-09-08. Current status: signed DMG verified on the sys domain and end-to-end on a clean Mac against sp-dev; sp-dev still serves an unsigned DMG and Claude Code inference is not gateway-routed on macOS.
verified; full-system acceptance is not green.

## 2026-09-08: clean-slate native Mac end-to-end against sp-dev

This section supersedes the 2026-09-07 deployment section below for the
customer endpoint question. Verified on this Intel Mac from a wiped state.

### Published binary state

| Source | SHA-256 | Signature | Gatekeeper |
| --- | --- | --- | --- |
| `https://sp-dev.astound.digital/files/downloads/astound-bridge-macos.dmg` | `cb3e69cdd68e6a836969e4a6ea54570edea64b75876def1c85ee2ba0f9e0ec66` | ad-hoc, `TeamIdentifier=not set`, no ticket | **rejected** |
| `https://astound.systemprompt.io/files/downloads/astound-bridge-macos.dmg` | `f7cd861a24e422fc6ed29242f53d903c77ed47d4fbf40c3cf11bae25b6dbb4d7` | Developer ID `7FSAPLA7RX`, app + DMG notarized and stapled | accepted, `Notarized Developer ID` |
| GitHub release `v0.47.0` asset | `e5341e3b2cec47e51ca7c10f1e49156de3e7b85e5f4a2c6b6a4389df41d4e693` | (per 2026-09-07 publication update) | — |

`sp-dev.astound.digital` resolves to `92.5.46.11` behind nginx and is not the
Fly app; it is a customer-operated mirror that this repository cannot deploy
to. Its sha256 sidecar matches its own unsigned DMG, so nothing on the page
warns. The sidecar on `astound.systemprompt.io` matches and
`scripts/verify-bridge-macos.sh` passes on the download. **Action for the
sp-dev operator: replace the served DMG and sidecar with the
`astound.systemprompt.io` artifact.** The three hashes above are three
different builds; the sys domain and the GitHub release should converge on the
next release.

### Clean slate

Removed before the test: `/Applications/Systemprompt Internal Bridge.app`
(0.45.0, different product), `~/Library/Application Support/astound/`,
`~/Library/Application Support/astound-bridge/`,
`~/Library/Application Support/systemprompt/bridge-install.id`,
`~/Library/Caches/astound-bridge/`, and the keychain services
`astound-bridge.oauth-client` and `systemprompt-internal-bridge.oauth-client`.
`~/.claude/settings.json` carried no gateway keys, no plugins and no
marketplaces; Claude Desktop had no managed preferences and no org-plugins.

### Install and sign-in (sp-dev, default gateway, no override)

- Installed from the `astound.systemprompt.io` DMG: copied to
  `/Applications/AstoundBridge.app`, DMG ejected, `spctl` accepted, launched
  with no Gatekeeper prompt. `astound-bridge 0.1.1 (c0fafb3, 2026-09-07)`.
- Note: `open -a AstoundBridge` can launch a stale dev build from
  `bridge/target/release/` on a developer Mac; launch `/Applications/AstoundBridge.app`
  by path when validating.
- `astound-bridge login` requires a terminal (device link); unattended it
  exits 1 with a clear message. Interactive device-link sign-in against
  `https://sp-dev.astound.digital` succeeded via ADFS
  (`e.burton@astound.local`, roles admin/platform_admin/user, token audiences
  bridge+mcp, issuer sp-dev).
- `doctor` after login: 18 ok, 2 warn (install record, lazy hook OAuth client),
  0 fail. Sync pulled marketplace `enterprise-demo` with `astound-admin` and
  `astound-dev`, pinned pubkey `AWe5jjKyOykf/kXPiZnMjHwlWISBVkrfXOa0rV7Ykz4=`.
- `install --apply` wrote `/Library/Managed Preferences/com.anthropic.claudefordesktop.plist`
  (and the per-user copy): `inferenceProvider=gateway`,
  `inferenceGatewayBaseUrl=http://127.0.0.1:48217`, bearer = loopback secret,
  models opus-5/sonnet-5/haiku-4-5, managed MCP servers `systemprompt` and
  `knowledge-bank` through the proxy. `--host claude-desktop` additionally
  offers a configuration profile under System Settings › General › Device
  Management that the user must approve.

### Proven through the proxy against sp-dev

- `POST http://127.0.0.1:48217/v1/messages` with the loopback secret,
  `claude-haiku-4-5`, returned `PONG`; audited on sp-dev as request
  `ab97151f-2ce7-46b3-8f4b-83a0e55c823f` (14/6 tokens, $0.000044) under user
  `9e7a44ac-7494-489b-b4d2-fcbb4191c983`.
- `/mcp/systemprompt` through the proxy: initialize + `tools/list` return the
  `systemprompt` CLI tool (server 0.47.0); `tools/call` executes
  `infra logs …` with admin scope.
- Native Claude Code 2.1.263, `claude mcp list` shows
  `plugin:astound-admin:systemprompt` connected via the proxy. A `claude -p`
  run made a real `mcp__systemprompt__systemprompt` call (`core skills list`,
  answer `admin_activity_report`); sp-dev trace
  `147a25b8-83ff-452f-a552-4b669a3d15ef` records it (MCP success, 27 ms).
- A request `34a6be2a-22e1-4c4f-8ee5-a87ff23c81f3` (user ".", assistant
  "Hello", haiku, 8/1) was audited at 08:25:25 UTC, coinciding with the first
  Claude Desktop launch under the managed policy — consistent with Desktop
  probing the gateway. Not yet confirmed by an interactive Desktop message.

### Non-Anthropic models through Claude Code (local gateway, 2026-09-08)

Run against a local server built from the vendored core (`SQLX_OFFLINE=true
just build --release`; the knowledge-bank crate can only build offline), with
provider keys copied into the local profile. Three defects, all fixed in core
`next` and covered by tests:

1. **Gemini rejected Claude Code's tool schemas** (`prefixItems` tuple nested
   under `items`; the audited failure on sp-dev was `12c7fd90`). The schema
   sanitizer now flattens tuple arrays for providers without them
   (`SchemaFeatures.tuple_items`), recurses into kept tuples, drops `$comment`.
2. **Vertex AI rejected `enum` without `type`** ("schema didn't specify the
   schema type field") after the const→enum conversion of `mode: {const: …}`.
   The sanitizer infers the type from the enum values when they agree.
3. **Every tool of the `systemprompt` MCP server vanished from Claude Code**:
   `admin_report`'s input is an internally tagged enum, which schemars renders
   as a bare `oneOf` with no root `type`; Claude Code validates
   `inputSchema.type == "object"` per tool and, on one failure, drops the
   server's whole list ("tools fetch failed — Invalid input (at
   tools.1.inputSchema.type)"). Core's `tool_definition` now sets the root
   type when a handler's schema lacks one; a repo unit test asserts every
   systemprompt tool has an object root.

Proof, one real `claude -p` turn per model through an isolated bridge proxy
(inference) and the gateway MCP route (bridge JWT), verified in Postgres
(`mcp_tool_executions` success + `ai_request_tool_calls` joined to an
`ai_requests` row on that model):

| model | verdict | note |
| --- | --- | --- |
| claude-haiku-4-5 | PASS | answer `admin_activity_report` |
| gemini-2.5-flash | PASS | |
| gemini-2.5-pro | PASS | |
| gpt-4.1-mini | PASS | |
| o4-mini | PASS | |
| vertex-gemini-2.5-flash | PASS | after fix 2 |
| gpt-4.1 | no tool call | answered `dataviz` from memory; request completed, no wire fault |

The live wire matrix (`scripts/e2e-wires.sh`, now sending Claude Code's real
schema shape on all three inbound surfaces) passed 45 cells across
Claude, Gemini, OpenAI chat/responses, Cerebras and Vertex-MaaS Qwen before
fix 2; the remaining failures were the Vertex Gemini enum type (fixed) and
DeepSeek upstream throttling (`RESOURCE_EXHAUSTED`), plus one DeepSeek
streaming cell on `/v1/responses` returning a tool call with no terminal
reason — still open.

Harness notes for macOS: `scripts/e2e-claude-code.sh` host mode is
Linux-only (`ldd`, `/proc`); the per-model cells above were run with an
equivalent shell runner. `psql` is not installed on this Mac; a shim that
runs `psql` inside the local Postgres container served the harnesses.

### Open defects found

1. **Claude Code inference bypassed the gateway on macOS — fixed in core.**
   `install --apply --host claude-code` reported `[ok] Claude Code — governed
   through the gateway` but wrote nothing for the terminal CLI (no
   `env.ANTHROPIC_BASE_URL` / `apiKeyHelper` in `~/.claude/settings.json`,
   `/Library/Application Support/ClaudeCode/managed-settings.json` still `{}`),
   so a plain `claude -p` call (answer `PONG-CC`, $0.018, 08:26:50 UTC) never
   appeared in sp-dev's `infra logs request list`; only MCP/hook traffic was
   governed. Core `next` now shares the Linux settings writer with macOS
   (`install::mdm::claude_code_settings`), merging the gateway keys into the
   user's existing `~/.claude/settings.json` and removing only those keys on
   `uninstall --host claude-code`. A second defect surfaced on the first
   attempt: Claude Code runs `apiKeyHelper` through `/bin/sh` unquoted, and
   the macOS helper path contains a space, so it exited 127 and broke every
   `claude` session on the machine until the value was shell-quoted. With the
   rebuilt bridge, `claude -p --model claude-haiku-4-5` (answer
   `PONG-CC-ROUTED`) produced four audited requests on sp-dev at 08:43 UTC
   (`01a50816`, `2e190f0c`, `30bc0708`, `ef269ebd`) under the same user id.
   Side effect to document for users: with the gateway credential active,
   Claude Code disables claude.ai connectors ("another auth source is set").
   Follow-up in core `0df08b16`: `install --apply` on macOS no longer merges
   into the user's settings; it writes
   `~/Library/Application Support/astound/claude-code-settings.json`, which
   `claude --settings <file>` applies per session (verified: `PONG-TOGGLE`,
   audited on sp-dev at 08:51 UTC with the global keys absent). The merge is
   now only `install --apply --host claude-code`, undone by
   `uninstall --host claude-code`. Documented on the new
   `/documentation/install-macos` page. `bridge/CORE_REF` points at that core
   commit; the DMG must be rebuilt and republished for users to get it.
2. `update --check` still fails: `/v1/bridge/latest` 404 on sp-dev (no
   `gateway.bridge_releases` feed).
3. `/bridge-auth/setup` offered no macOS download and described a token-paste
   sign-in; fixed in this repository alongside these notes
   (`bridge-setup.hbs`, `admin-bridge-setup.js`, `downloads.md`,
   `fetch-remote-downloads.sh`). sp-dev will keep showing the old page until
   its operator redeploys.
4. The bridge CLI has no per-command `--help`: `astound-bridge uninstall --help`
   executes `uninstall`.
5. The gateway's `/v1/models` returned `display_name` derived from the id
   ("Claude Haiku 4 5") and `created_at` of the Unix epoch. Fixed in core
   (`Claude Haiku 4.5`, real dates for dated ids); "From gateway" is Claude
   Code's own label for discovered models.
6. **Gemini from Claude Code.** Two defects, both fixed in core `next`:
   (a) Claude Code's discovery only lists ids containing `claude`/`anthropic`,
   so the Gemini/Vertex models the gateway advertises never reached `/model`;
   the bridge now writes `modelPicker` rows for them on every sync (Claude Code
   only — Claude Desktop's `inferenceModels` is filtered to Anthropic ids with
   a unit test guarding it, because Desktop breaks on other families).
   (b) `claude --model gemini-2.5-flash` failed with Gemini's
   `Unknown name "prefixItems" … function_declarations[1].parameters…items`;
   the request was audited on sp-dev as `12c7fd90` (failed, 0/0). The schema
   sanitizer now flattens draft-2020-12 tuple arrays (`prefixItems`,
   `additionalItems`, `unevaluatedItems`) to a plain `items` schema for
   providers without tuple support (Gemini), recurses into kept tuples, and
   drops `$comment`.

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

## 2026-09-08 08:39 UTC — Claude Code authentication recovered

The local experimental installer added an unquoted `apiKeyHelper` command to
`~/.claude/settings.json`. Its path contains `Application Support`; Claude Code
executes the command through a shell, which split the path and exited 127.
Comparison with the pre-install settings backup found only `env` and
`apiKeyHelper` changed. This was an authentication configuration failure.

Local recovery preserved a backup, removed those newly added gateway settings
from the user file, and placed them in
`~/Library/Application Support/astound/claude-code-gateway.settings.json`, with
the helper command shell-quoted. The helper exits zero and returns the current
loopback key. `claude auth status` again reports the existing claude.ai Max login.

On this recovered Mac, choose routing when starting a new session:

```bash
# Existing personal authentication
claude

# Bridge gateway authentication for this session
claude --settings "$HOME/Library/Application Support/astound/claude-code-gateway.settings.json"
```

The extra settings file is a local recovery artifact, not yet a file generated
by the released installer. Claude documents session settings and their merge
precedence at https://code.claude.com/docs/en/settings.
This toggles inference authentication; installed MCP servers and plugins retain
their existing configuration.

A real Claude Code inference test using the separate file and
`claude-haiku-4-5-20251001` returned `ASTOUND-GATEWAY-RECOVERY-OK`, exit zero,
session `a77727de-f614-4391-80d0-8485e7a4512a`. Tools and MCP were disabled for
this focused inference test. Server-side request listing through the bridge MCP
confirmed completed sp-dev requests at 08:39:42 UTC:

- `667f043c-1747-42ce-83f3-a63c15996c07`
- `7af77aee-c117-42bf-b5d8-f868ac324911`

Both belong to user `9e7a44ac-7494-489b-b4d2-fcbb4191c983`.
Bridge doctor: 19 OK, one warning, zero failures. The warning is that the
installation record references the local development executable while the
checked executable is the signed app under /Applications.

Outstanding product work: correct installer command quoting and implement an
explicit settings-file enrollment mode before shipping the experimental core
changes. Re-running that experimental installer can reintroduce the global
settings problem. This recovery does not validate its release readiness.
