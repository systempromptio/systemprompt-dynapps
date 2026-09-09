# issues.md — the class of defect behind the bridge failures of 2026-09-08

Four symptoms were reported against the downloaded Windows bridge pointed at a
local instance: sync refused with a signature error, the footer read `v0.1.1`,
the registry was only written when the app ran as Administrator (and said
nothing when it was not), and the window flashed every 30 seconds. None of
these is a one-off. Every one is an instance of the same class: **a component
that keeps working, reports success, and is wrong.** This file names the class,
records the instances found, gives the procedure for finding the rest, and sets
the rules that keep them out.

Silent failure and tech debt are not permitted in this project. A path that
cannot do its job stops and says so. A number that must agree with another
number is derived from it or checked against it. A write that must land is read
back. Anything else is a defect, whether or not a user has hit it yet.

## 1. The class, in five shapes

### 1.1 Success without evidence

Code reports "done" from the absence of an error rather than the presence of a
result.

- `install/elevated_job.rs::finish` (core bridge): the elevated child exited 0
  and the parent could not read its result file, so the parent returned `Ok`.
  The child itself swallowed the failure to write that file and still exited 0.
  Two swallowed errors composed into a green sync with nothing written.
- No registry write was ever read back. `RegSetValueExW` returning 0 was the
  whole proof.
- `bridge_version_is_supported` returned `true` when it could not parse the
  version. The gate passed precisely when it had no idea.

### 1.2 Short-circuits that read the wrong thing

"Already in step, nothing to do" is a write that did not happen. It is only
honest if it reads exactly what the write would have produced.

- `WritePlan::drifted()` read HKLM **then** HKCU and accepted whichever had a
  value. A stale per-user copy made the machine-wide write unnecessary in the
  code's eyes, so the HKLM write was skipped and the caller was told it had
  landed.
- `current_value()` did the same for `managedMcpServers` after discarding the
  result of the HKCU delete it had just issued.

### 1.3 State bound to a peer without recording the peer

A value learned from one counterpart, then applied to another.

- The manifest pubkey was learned by trust-on-first-use from the staging
  gateway and stored as a bare string. Changing `gateway_url` kept the key. The
  error text blamed tampering. There was no path back except editing the TOML
  by hand, and the registry copy written by an earlier `install --apply`
  silently outranked even that.

### 1.4 Numbers that must agree but are not derived from each other

- Displayed bridge version: astound `bridge/Cargo.toml` `0.1.1`. Wire version:
  core `bin/bridge/Cargo.toml` `0.38.0`. Core release: `0.47.0`. The release
  workflow printed two of them side by side and the admin Devices page ranked a
  third by string comparison, where `0.9.0` beats `0.47.0`. The core changelog
  records an earlier fix that moved the wire off the brand version because the
  brand's `0.1.1` was rejected by the gateway floor — the symptom was fixed,
  the drift was kept.

### 1.5 Discarded results and background work nobody owns

- `_ = reg_command().status()` on the HKCU and legacy-key deletes.
- `bridge/web/js/components/profile-connections.js` ran a window-wide 30 s
  timer that fetched three gateway endpoints from every tab and could start a
  sync from the Marketplace pane. Nothing in the UI could attribute the
  resulting repaint to it.
- The Library pane refetched its listing on every snapshot once a sync had
  failed, because its "unchanged" check was keyed on `state == "ok"` instead of
  on the listing.

## 2. What changed (2026-09-08)

| instance | fix | where |
|---|---|---|
| pin without gateway | pin stored with `pinned_pubkey_gateway`; mismatch ⇒ stale ⇒ re-learned; error names source + gateway + fix | core `bin/bridge/src/config/profile.rs`, `sync/manifest.rs`, `sync/error.rs` |
| HKLM hard-coded | unelevated ⇒ HKCU; `HiveConflict` when HKLM would shadow; UAC job only for org-plugins and admin files | core `install/mdm/windows*.rs`, `integration/claude_desktop/windows.rs`, `config/store/mod.rs` |
| no read-back | every write read back; `VerifyMismatch { hive, subkey, name }` | core `config/store/windows_registry_write.rs` |
| union read before write | `drifted()` / `current_value()` read the target hive only | core `install/mdm/windows_policy.rs` |
| child result optional | missing/unparseable result file is an error; child writes `started` first and exits non-zero if it cannot | core `install/elevated_job.rs` |
| host id only in toast | first line of each failure beside the host id | core `gui/handlers/sync.rs` |
| fail-open version parse | unparseable ⇒ unsupported; test inverted with the reason | core `crates/shared/models/src/bridge/manifest.rs` |
| three versions | core bridge crate and astound bridge crate both `0.47.0`; `sync-release-version.sh` pins it; release workflow fails on disagreement; `doctor` and a start-up warning report drift; Devices page compares numerically | `bridge/Cargo.toml`, `scripts/sync-release-version.sh`, `.github/workflows/release.yml`, `extensions/web/admin/src/handlers/ssr/devices/view.rs` |
| app-wide poll | poll only while the Account tab is mounted | `bridge/web/js/components/profile-connections.js` |
| refetch on error state | marker keyed on the listing; existing listing stays painted while refreshing | core `web/js/services/marketplace-service.js` |

The 30-second flash is not closed by this. Two of its three candidate causes
are removed above; the third (a snapshot field changing per probe and defeating
the `state.changed` hash) needs one instrumented run on Windows. See §4.

## 3. How to find the rest

Each shape has a grep. Run them over `extensions/`, `bridge/src/` and
`../systemprompt-core/bin/bridge/src/`, exclude `tests/`, and read every hit
against the question in the right-hand column. The current counts are the
backlog; the target for each is zero un-annotated hits.

| shape | grep | question to ask at each hit |
|---|---|---|
| discarded result | `grep -rnE '^\s*(let )?_ = \|\.ok\(\);\s*$'` | What would the caller do differently if this failed? If "nothing", why is the call made? |
| swallowed error | `grep -rnE 'if let Ok\(|let Ok\(.*\) = .* else \{|unwrap_or_default\(\)|unwrap_or\(false\)|\.unwrap_or\(true\)'` | Does the `else`/default branch produce the same observable output as success? |
| warn-and-continue | `grep -rn 'tracing::warn!' \| grep -v 'return\|Err('` | Is the warning the only record that something failed? Who reads the log? |
| fail-open guard | `grep -rnE '_ => true,|\.is_none_or\(|\.map_or\(true'` | Does the guard pass when it cannot evaluate? |
| union read before a targeted write | `grep -rnE 'then|fallback|or_else' config/store install/mdm` | Does the read see the same scope the write targets? |
| short-circuit "already" | `grep -rn 'already\|nothing to write\|in step'` | Is the comparison against the exact target and value the write would produce? |
| peer-bound state | `grep -rn 'pinned\|cached\|remembered\|last_' config/` | Does the stored value carry the identity of the peer it came from? |
| independent version numbers | `grep -rn '^version = ' --include=Cargo.toml` + `grep -rn 'CARGO_PKG_VERSION'` | Is each number derived from one source or checked against it in CI? |
| window-scoped timers | `grep -rn 'setInterval\|setTimeout' web/js` outside a component's `onConnect`/`onDisconnect` pair | Who cancels it? Which pane does it belong to? |
| string-compared versions | `grep -rn '\.max()\|\.sort()' \| grep -i version` | Is `0.9.0 > 0.47.0` acceptable here? |

Counts at the time of writing (statement-level discards only):

| tree | hits |
|---|---|
| `extensions/` + `bridge/src/` | 19 |
| `../systemprompt-core/bin/bridge/src/` | 139 |

Work them file by file. For each hit either propagate the error, replace the
discard with a typed outcome the caller inspects, or annotate the line with
`// Why: discard-ok: <reason>` so the decision is visible in review. An
annotation is for a result that genuinely carries no information (a best-effort
`remove_file` on a temp path), not for one that is inconvenient to handle.

## 4. Reproducing what cannot be read

Some of this class only shows at runtime. The 30 s flash is the current
example. The procedure:

1. Build the bridge (`just bridge-build`) and run it with the WebView2 devtools
   open (`--dev-preview` / `F12` on Windows).
2. In the console, count events per channel for two minutes:
   `state.changed`, `host.changed`, `gateway.changed`, and every `bridge.*`
   IPC reply. Diff consecutive `state.changed` payloads; any key that changes
   per tick and is not in `VOLATILE_KEYS` (`gui/emit.rs`) is the defeat of the
   hash dedup.
3. Watch `document.body.classList` for `is-setup-mode` toggles and
   `sp-marketplace` for `listingState` transitions.
4. Fix at the source of the event, never by throttling the render.

Record the transcript of that run in the ignored `docs-internal/bugs/` directory beside the fix.

## 5. Rules (enforced, or to be enforced)

These are the invariants, in the form a gate can check.

1. **No success without a result.** A function that performs a side effect
   returns a typed outcome that names what it did (`PolicyWrite::Written(hive)`,
   `SatisfiedByMachine`), never `()` or `bool` for "probably fine". The elevated
   job protocol requires a result file; its absence is an error.
2. **Every write is read back** through the same store it was written to, and a
   mismatch is a typed error naming the location. Registry, managed files,
   config TOML.
3. **A short-circuit reads exactly what the write would produce**, in the scope
   the write targets. No hive union, no "any of these locations".
4. **State learned from a peer records the peer.** A pin, a cached token, a
   remembered port carries the origin it belongs to and is void elsewhere.
5. **One number, one source.** The astound bridge version is the core release
   version; `scripts/sync-release-version.sh` writes it, `--check` verifies it,
   `release.yml` refuses a release where they differ, and the bridge warns at
   start-up and in `doctor` if the brand and wire versions disagree.
6. **Fail closed.** A guard that cannot evaluate its input returns "not
   allowed", and the test that pins the behaviour says why.
7. **Discarded results are annotated or gone.** `_ =`, `.ok();`,
   `unwrap_or_default()` on a fallible side effect need `// Why: discard-ok:`.
8. **Errors reach the surface with their detail.** A toast, summary line or
   CLI check names the hive, path, key or gateway, not a host id or a
   category. The first line of the error is the minimum.
9. **Timers belong to components.** Created in `onConnect`, cancelled in
   `onDisconnect`, never module-level.

### Gates to add

`lint-silent-skips.sh` already covers tests that return early. The production
side needs its sibling:

- `scripts/check-discarded-results.sh` — the first grep in §3 over
  `extensions/` and `bridge/src/`, failing on any hit without a
  `discard-ok:` annotation on the statement. Wire it into the `gates=()` array
  in the `justfile`. Land it with the 19 existing hits resolved in the same
  change, not with an exemption list.
- `scripts/check-fail-open.sh` — `_ => true` / `.is_none_or(` / `.map_or(true`
  in any function whose name starts with `is_`, `check_`, `verify_`,
  `supported`, `allowed`; same annotation escape.
- A cargo test in `tests/unit/web/src/` asserting `bridge/Cargo.toml` version
  equals the workspace version, so the parity does not depend on remembering to
  run the sync script.
- The same three in core's `scripts/` for `bin/bridge/src`, where 139 hits
  wait.

## 6. Review checklist

Before approving a change that touches a side effect, ask in order:

1. If this write fails, what does the caller see? (Not "a log line".)
2. If the write is skipped as unnecessary, what was read to decide that, and is
   it the same location the write targets?
3. If this value was learned from somewhere, is the somewhere stored with it?
4. If this number must match another number, where is the check that fails
   when it does not?
5. If this returns `true` on an error path, why is that safe?

A change that cannot answer all five is not done.
