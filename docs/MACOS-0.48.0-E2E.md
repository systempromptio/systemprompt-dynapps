# macOS 0.48.0 acceptance run — 2026-09-09

## Decision

**Not a complete deployment acceptance pass.** The customer endpoint was
updated by another actor during this run and now serves the signed 0.48.0
release download. Local inference and tool execution work, but the released
picker defect and the non-green browser suite remain. No remote deployment
was made by this agent.

## Provenance

- Synced Astound `main`/`next` to `bf40988616cc24e77dee9301c05964a33a6a63ba`.
- Synced core `main` to release `v0.48.0`,
  `68849c4e911f20afc95492f915386865ea956f62`.
- Tested the actual GitHub release DMG, not a locally rebuilt substitute:
  SHA-256 `cf76afbd7e59498363ef39fbfb4afd41786c44d0d4d0a20451b6948f17254c2c`.
- Local gateway was built through Astound against published core 0.48.0.
  It served `http://localhost:8080`; live provider credentials were used for
  inference. Slack, Teams and Salesforce credentials were excluded from the
  private test profile. Browser fixtures touched the local database only.
- Picker repair is on core `next`: semantic fix `21b64e28d`, formatting-only
  follow-up `9f1ea9e55`, module-layout follow-up `18fedfe56`. Astound pins the
  latter and activates both Cargo patch
  blocks together. Neither commit is a new published core release.
- Intel Mac; native Claude Code 2.1.263 for interactive picker testing.
  ARM64 slice/signing checked, but no physical Apple Silicon execution.

## Customer endpoint

Initially, `https://sp-dev.astound.digital/readyz` returned 200 with version **0.47.0**.
Its actual downloaded DMG had SHA-256
`cb3e69cdd68e6a836969e4a6ea54570edea64b75876def1c85ee2ba0f9e0ec66`.
It was ad-hoc signed, had no Developer ID team and was rejected by Gatekeeper.
The verifier now prints an explicit team-identity failure for this case.

On recheck later in the same run, readiness returned **0.48.0**, instance
`3de121fad08d`. A fresh download had SHA-256
`cf76afbd7e59498363ef39fbfb4afd41786c44d0d4d0a20451b6948f17254c2c`, exactly
matching the tested GitHub release. Its app and DMG passed Developer ID,
notarization/stapling, hardened-runtime, universal-slice and Gatekeeper checks.
The endpoint-downloaded asset also passed the full native Intel CLI smoke,
including launch after ejecting and gateway selection/empty-login rejection.
The earlier rejected asset is historical evidence, **not the final live state**.

No usable customer credential was available. The installed app's customer
sign-in still requires the user's browser approval. Credentials belonging to
`astound.systemprompt.io` were not reused against the customer endpoint.
That older test environment was not deployed or used as a substitute for
customer acceptance.

Pushing `next` does not replace the immutable GitHub `v0.48.0` asset or deploy
the customer endpoint. A versioned release and an explicit customer deployment
are still required; verify the endpoint's download again afterwards. The
locally repaired 0.48.0 candidate is not evidence of an automatic update from
an existing 0.48.0 installation.

## Published Mac asset

`bash tests/release/macos-smoke.sh <downloaded-dmg> <sha256-string>` passed:

- Developer ID team `7FSAPLA7RX`, hardened runtime, universal binary.
- App and DMG notarized and stapled; Gatekeeper accepted both.
- Copied app executed after ejecting the image, on native Intel macOS.
- Default/override gateway selection and rejection of empty login passed.
- Installed the released app at `/Applications/AstoundBridge.app`.

The released bridge **does contain a picker defect**: it writes an array of
`{id,label}` rows. Claude Code requires
`{"modelPicker":{"options":[{"model":"…","label":"…"}]}}`.
The native terminal displayed the settings warning and ignored the old field.

## Locally repaired candidate

Built both architectures through `just bridge-package-macos` against core
`18fedfe56d287590c3bd0e8277ea5e7c8f397c5a`, using the existing Developer ID
identity and Apple notary credential profile. No signing secrets were changed.
Candidate SHA-256:
`134090fe90e2423ad962a7574e5d8a22f86f46c8d34600997ceab54c3c24cdc1`.

Downloaded this candidate from the **local gateway's actual download route**
and ran the complete Mac smoke again: PASS, including strict copied-app
verification after ejecting. It reports `0.48.0 (18fedfe, 2026-09-09)`.
Installed that downloaded app in `/Applications`, verified strict signing and
Gatekeeper there, and restarted the proxy from it. This is a local candidate,
not the current customer download or an overwritten GitHub release.
After installation, normal sync succeeded and both Gemini Flash and Vertex
Flash completed real synced-plugin tool calls through that installed app's
proxy. Final checks found 18 correctly shaped picker options and no personal
always-on Claude gateway override.

The machine's privacy controls denied `hdiutil create -srcfolder` access to
its intermediate `/Volumes` path. Packaging now has a checked fallback: create
a blank writable HFS+ image, mount at a private temporary path, copy and verify
the signed app, detach and compress, then sign/notarize the DMG and run the
same full verifier. A trial using `makehybrid` was rejected by that verifier
because it injected FinderInfo metadata; that approach was removed and no
invalid candidate was staged. Gatekeeper and signature checks were not relaxed.

## Local bridge and Claude Code

Admin-issued one-time code login, JWT minting, authenticated identity,
inference, MCP and sync were exercised with the released bridge.
Reusing a consumed code failed without changing the stored credential.
Restarting the proxy required no re-enrolment and another prompt succeeded.

Fresh sync correctly refused an unpinned manifest key. Explicit TOFU was used
**only for the localhost test**; subsequent normal sync succeeded with the
pinned key. Synced 2 plugins, 11 skills, 1 agent and 2 MCP servers. The first
macOS managed-plugin write required administrator approval.

Each model below completed actual gateway `core skills list` and a local MCP
fixture exercising nested `prefixItems`/`const` schemas. Tool responses
included `admin_activity_report` and `VERIFIED_MAC_GEMINI_048`:

| Model | Inference + both tools |
|---|---|
| `gemini-2.5-flash` | PASS |
| `gemini-2.5-flash-lite` | PASS |
| `gemini-2.5-pro` | PASS |
| `vertex-gemini-2.5-flash` | PASS |
| `vertex-gemini-2.5-pro` | PASS |
| `claude-sonnet-5` | PASS |

Also tested the actual synced `astound-admin` plugin, without a replacement
MCP configuration: fresh Gemini Flash and Vertex Flash sessions both returned
the first skill from a real tool call.

Always-routed Claude settings were tested then uninstalled. The final intended
mode is **per-session `--settings`**, not a personal always-on gateway override.
The test configuration path contains `Application Support`; the generated
quoted API-key helper worked with spaces in its path.

After rebuilding the picker repair through Astound, real bridge sync migrated
the settings. Native Claude Code no longer warned, `/model` displayed Gemini
and Vertex, and session-only Gemini selection passed PONG and the synced MCP
tool call. Regression tests cover modern output, legacy migration, preserving
unrelated rows/metadata, repeated sync and ownership-scoped uninstall.

Switching that tool-bearing conversation from Gemini to Vertex produced one
HTTP 400 (`Request contains an invalid argument`). Fresh Vertex succeeded;
subsequent exact request replays, including streaming, succeeded. This
observation is not explained away or recorded as a clean switching pass.
Resuming the original Claude conversation on Vertex and retrying the real
MCP tool also succeeded. No speculative gateway change was made for this
non-reproducing upstream error.

## Browser and audit checks

- Two bridge JavaScript test files: **6 passed**.
- Full Chromium suite: **747 passed, 66 failed, 29 skipped** (842 tests).
  Linux visual snapshots were excluded on this Mac; no visual approval claimed.
- Two failures were obsolete connection-code selectors after the profile
  redesign. Updated the page object and requirement test; both targeted tests
  then passed, including a successful code-mint response and visible output.
  This was not a full-suite rerun: the other 64 failures remain untriaged,
  spanning layout/accessibility, fixtures/catalogs, sessions and audit pages.
- Gateway audit rows confirmed the tested providers, models, completed requests,
  nonzero tokens/cost and real MCP executions. Explicit diagnostic PreToolUse
  and PostToolUse HTTP hooks returned 200.
- Do not equate Claude's local cost estimates with gateway provider accounting.
  The inspected AI-tool/MCP foreign-key links were null, so this run does not
  establish a fully joined identity-to-tool audit trace. Some auxiliary requests
  remained pending; their lifecycle was not accepted as complete.

## Cleanup and evidence

Old bridge app/support state was backed up and moved to recoverable Trash;
old bridge-managed Claude integrations were removed before the clean install.
Personal Claude conversations, authentication, permissions and unrelated
settings were preserved. Stale Hermes/OpenCode profiles were found during
doctor checks and backed up separately before bridge-scoped cleanup.
Hermes inference settings and old OpenCode user-level MCP entries were removed;
removing OpenCode's machine-managed provider file still awaits the native
administrator prompt. Until approved, local doctor reports **17 OK, 3 warnings,
1 failure** for that stale OpenCode proxy port, not a clean zero-failure result.
Deleted the old `fix/macos-statvfs-build` branch only after proving its commit
is an ancestor of core `next`; its code remains in history.
Debug build caches were cleaned; approximately 48 GiB was free afterwards.
Sources, databases, signing keys and deployment credentials were preserved.

Private local evidence is under `/tmp/astound-macos-048.NgXGGA` (mode 0700):
DMGs, checksums, signing/build logs, model transcripts, audit summaries and
backups. It contains test credentials: **do not publish this directory**.
Only this sanitized report is committed. Native administrator approvals and
customer authentication are not inferred from CI or a successful registry push.
