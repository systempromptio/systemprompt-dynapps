# v0.47.0 macOS repair: validation checkpoint

Date: 2026-09-07. Current status: published repair; live deployment outstanding.

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
