#!/usr/bin/env bash
# Line-coverage run for all three workspaces (root, tests/, bridge/), ported
# from systemprompt-core's `just coverage` recipe. Raw llvm-cov rather than
# cargo-llvm-cov, for the same host-toolchain reasons core documents:
#
#   1. sccache via [build] rustc-wrapper in ~/.cargo/config.toml returns
#      cached uninstrumented rlibs — neutralised by CARGO_BUILD_RUSTC_WRAPPER="".
#   2. A mold linker pinned by target.<triple>.rustflags strips the
#      profile-runtime constructors, silently producing zero profraw files.
#      Setting the RUSTFLAGS env replaces target rustflags entirely (cargo's
#      flag-resolution order), so the default linker links. cargo-llvm-cov
#      MERGES target rustflags back in, which is why it cannot be used.
#
# Builds only in dedicated target dirs under coverage-report/ so concurrent
# agents sharing this checkout are unaffected. Profraw naming is per platform:
# Linux uses %m%c (continuous mode with --runtime-counter-relocation); on
# macOS the nightly LLVM runtime rejects continuous-mode counters ("Counters
# section not page-aligned") and records nothing, so Darwin uses %m%p. %m makes
# the runtime merge into an existing file under a lock, so PID reuse across
# many test processes cannot overwrite a profile in either mode.
#
# DB-backed suites (mcp-integration, admin-contract) manage their own
# throwaway *_test databases via SYSTEMPROMPT_TEST_DATABASE_URL, same
# derivation as `just test-integration` — no dedicated coverage database is
# needed here, unlike core's shared-dev-DB situation.
#
# Outputs (all under coverage-report/, gitignored):
#   summary.json   llvm-cov export --summary-only (totals + per-file)
#   report.txt     human-readable llvm-cov report
#   lcov.info      lcov export for external tooling
#   tests.profdata merged profile (input to `just coverage-html`)
#   objects.txt    the deduplicated objects reported over (same)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Brand values (the repo name in the ignore regex) live in the one per-repo CI
# file, so this script is byte-identical across the downstream repos.
CI_REPO_NAME="$(sed -n 's/^CI_REPO_NAME=//p' "$ROOT/.github/ci-params.env")"
[ -n "$CI_REPO_NAME" ] || { echo "error: CI_REPO_NAME missing from .github/ci-params.env" >&2; exit 1; }

# Peak memory is several instrumented binaries linking at once; a runner out
# of memory ends the job with a bare SIGTERM. Lower it where the machine is
# small; the default suits a workstation.
BUILD_JOBS="${COVERAGE_BUILD_JOBS:-4}"

# Counts phases that failed to build or ran a red test; reported at the end so
# the exit status says whether the number rests on every workspace.
PHASE_FAILURES=0
cd "$ROOT"
PROFDIR="$ROOT/coverage-report/profraw"
TBASE="${COVERAGE_TARGET_DIR:-$ROOT/coverage-report/target}"

# COVERAGE_PHASES picks a subset of root, tests, bridge and report, so CI can
# build the root workspace on one runner while another builds and runs the
# rest, and a third reports over both. Any build phase starts a fresh profile;
# a report-only run reads the profraw and objects it was handed.
PHASES=" ${COVERAGE_PHASES:-root tests bridge report} "
has_phase() { [[ "$PHASES" == *" $1 "* ]]; }

if has_phase root || has_phase tests || has_phase bridge; then
    rm -rf "$PROFDIR" "$ROOT/coverage-report/tests.profdata"
fi
mkdir -p "$PROFDIR"

if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ] && [ -f .systemprompt/profiles/local/secrets.json ]; then
    SYSTEMPROMPT_TEST_DATABASE_URL=$(python3 -c "
import json, urllib.parse as up
u = up.urlsplit(json.load(open('.systemprompt/profiles/local/secrets.json'))['database_url'])
print(up.urlunsplit((u.scheme, u.netloc, '/postgres', '', '')))")
    export SYSTEMPROMPT_TEST_DATABASE_URL
fi

# One invocation per workspace; each gets its own target dir so a plain build
# never poisons an instrumented one (or vice versa). nextest, not cargo test:
# process-per-test isolates the OnceLock-global fixtures (Config::install)
# that collide inside one test process, honours the serial DB test-groups,
# and --no-fail-fast keeps one red package from silently dropping every
# later package out of the denominator.
if [ "$(uname -s)" = "Darwin" ]; then
    PROFRAW_PATTERN="%m%p"
    COVERAGE_RUSTFLAGS="-C instrument-coverage"
else
    PROFRAW_PATTERN="%m%c"
    COVERAGE_RUSTFLAGS="-C instrument-coverage -C llvm-args=--runtime-counter-relocation"
fi

run_instrumented() {
    local tdir="$1"; shift
    # Why: the export below picks the newest binary per crate basename, so a
    # test executable left behind by a crate that no longer exists (or was
    # renamed) is never superseded and keeps feeding stale coverage mappings —
    # files that were deleted count as 0% and functions whose hashes changed
    # count as uncovered. Dropping every executable first costs one relink per
    # test binary; the rlibs stay cached.
    find "$tdir/debug/deps" -maxdepth 1 -type f -executable \
        ! -name '*.d' ! -name '*.so' ! -name '*.rlib' -delete 2>/dev/null || true
    # `if` rather than a bare call: this script runs under `set -e`, and the
    # phases below survive a red workspace while keeping its exit status.
    local status=0
    if CARGO_BUILD_RUSTC_WRAPPER="" \
        RUSTC_WRAPPER="" \
        CARGO_TARGET_DIR="$tdir" \
        LLVM_PROFILE_FILE="$PROFDIR/$PROFRAW_PATTERN.profraw" \
        RUSTFLAGS="$COVERAGE_RUSTFLAGS" \
        SQLX_OFFLINE=true \
        cargo nextest run --no-fail-fast --build-jobs "$BUILD_JOBS" "$@"
    then
        status=0
    else
        status=$?
    fi
    # nextest exit codes, treated by consequence rather than lumped together:
    #   0  everything ran
    #   4  the workspace defines no tests. True of the root and bridge
    #      workspaces here — they are instrumented for their objects, not for
    #      tests — so it is the expected answer, not a failure.
    #   *  a build failure or a red test. The report is still produced, since a
    #      partial number beats none while iterating, but the run exits non-zero
    #      so a workspace that did not compile never passes as a healthy number.
    case "$status" in
        0|4) ;;
        *)
            echo "warning: workspace failed (nextest exit $status) — continuing to coverage report" >&2
            PHASE_FAILURES=$((PHASE_FAILURES + 1))
            ;;
    esac
}

# One target dir per workspace. A shared root+tests dir saved no compile:
# the two workspaces resolve the dependency graph under different feature
# sets, so each phase rebuilt what the other had just written, and CI runs
# the phases on separate runners anyway.
# Why --no-tests=pass: the root and bridge workspaces are instrumented for
# their objects and define no tests; nextest's default for that is exit 4.
if has_phase root; then
    echo "==> [1/3] Instrumented tests: root workspace"
    run_instrumented "$TBASE-root" --workspace --tests --no-tests=pass
fi

if has_phase tests; then
    echo "==> [2/3] Instrumented tests: tests/ workspace"
    # A repo whose tests/.config/nextest.toml declares a `coverage` profile
    # (longer slow-timeouts under instrumentation) gets it; nextest refuses a
    # profile its config does not name, so it is never passed blind.
    profile=()
    if grep -q '^\[profile\.coverage\]' tests/.config/nextest.toml 2>/dev/null; then
        profile=(--profile coverage)
    fi
    run_instrumented "$TBASE-tests" --manifest-path tests/Cargo.toml --workspace "${profile[@]}"
fi

if has_phase bridge; then
    echo "==> [3/3] Instrumented tests: bridge/ workspace"
    run_instrumented "$TBASE-bridge" --manifest-path bridge/Cargo.toml --workspace --no-tests=pass
fi

if ! has_phase report; then
    if [ "$PHASE_FAILURES" -gt 0 ]; then
        echo "error: $PHASE_FAILURES workspace(s) did not run cleanly" >&2
        exit 1
    fi
    exit 0
fi

PROFRAW_COUNT=$(find "$PROFDIR" -name '*.profraw' | wc -l)
echo "==> Generated $PROFRAW_COUNT profraw files"
if [ "$PROFRAW_COUNT" -eq 0 ]; then
    echo "error: zero profraw files — the sccache/mold overrides are not taking effect" >&2
    exit 1
fi

TOOLDIR="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"
LLVM_PROFDATA="$TOOLDIR/llvm-profdata"
LLVM_COV="$TOOLDIR/llvm-cov"

echo "==> Merging profile data"
find "$PROFDIR" -name '*.profraw' > "$ROOT/coverage-report/profraw-list.txt"
"$LLVM_PROFDATA" merge -sparse -f "$ROOT/coverage-report/profraw-list.txt" \
    -o "$ROOT/coverage-report/tests.profdata"

# Test binaries land in <target>/debug/deps. Emit candidates from both
# instrumented dirs, then dedupe the combined stream by crate basename,
# keeping the newest build of each. Dedupe must be global: per target directory it
# feeds llvm-cov multiple feature/build variants of the same crate. Their
# function names overlap but their mapping hashes differ, so llvm-cov discards
# profile records with "functions have mismatched data" and under-reports the
# very tests this job is meant to measure.
# Why: BSD find (macOS) has no -printf, so the mtime is read with stat on
# either platform instead; GNU stat and BSD stat spell the format differently.
# The flavour is probed once: GNU `stat -f` is not an error but --file-system,
# whose "Total:" output would reach llvm-cov as an object path.
if stat -c '%Y' / >/dev/null 2>&1; then
    STAT_MTIME=(stat -c '%Y %n')
else
    STAT_MTIME=(stat -f '%m %N')
fi
collect_bin_candidates() {
    find "$1/debug/deps" -maxdepth 1 -type f -perm -u+x ! -name '*.d' ! -name '*.so' \
        ! -name '*.dylib' ! -name '*.rlib' ! -name '*.rmeta' 2>/dev/null \
        | while IFS= read -r bin; do
            if [ -f "$bin" ]; then "${STAT_MTIME[@]}" "$bin"; fi
          done | sort -rn
}
BINS="$(
    {
        collect_bin_candidates "$TBASE-root"
        collect_bin_candidates "$TBASE-tests"
        collect_bin_candidates "$TBASE-bridge"
    } | awk '{ base=$2; sub(".*/", "", base); sub(/-[0-9a-f]+$/, "", base); if (!seen[base]++) print $2 }'
)"
BIN_COUNT=$(printf '%s\n' "$BINS" | sed '/^$/d' | wc -l)
if [ "$BIN_COUNT" -eq 0 ]; then
    echo "error: no instrumented test objects found" >&2
    exit 1
fi
echo "==> Reporting over $BIN_COUNT globally deduplicated test objects"
printf '%s\n' "$BINS" | sed '/^$/d' > "$ROOT/coverage-report/objects.txt"
OBJ_ARGS=()
for b in $BINS; do
    if [ -f "$b" ]; then OBJ_ARGS+=(--object "$b"); fi
done

# Denominator exclusions, kept explicit per-file (per-directory only for test
# code) so ordinary testable code added alongside them still counts:
#   src/main.rs, src/lib.rs        — process entry / pure re-export shims
#   extensions/**/extension.rs     — inventory registration glue, no logic
#   */build.rs                     — build scripts
# Keep this regex in sync between coverage.sh, coverage-check.sh docs, and
# `just coverage-html`.
# systemprompt-core/ is the sibling checkout pulled in via [patch.crates-io];
# it has its own coverage CI and must not dilute this repo's denominator.
# bridge/src/main.rs is a process supervisor (brand consts + run_with_brand
# delegation); on Linux it is the crate's only compiled file, mirroring core's
# explicit process-entry exclusions.
# extensions/mcp/*/src/main.rs are `tokio::main` shells that build a server and
# serve it over stdio — the same kind of process entry as the CLI mains beside
# them. Whether their object is picked up also depends on which binaries a run
# builds, so counting them would move a crate's number with unrelated test
# dependencies.
IGNORE_RE="(\.cargo|/rustc/|/registry/|/debug/build/|/tests/|/target/|systemprompt-core/|${CI_REPO_NAME}/src/(main|lib)\.rs|bridge/src/main\.rs|extensions/(cli|mcp)/[^/]+/src/main\.rs|extensions/cli/[^/]+/src/commands/|extensions/.*/extension\.rs|build\.rs)"

echo "==> Coverage report"
WARNINGS="$ROOT/coverage-report/llvm-cov-warnings.txt"
"$LLVM_COV" report \
    --instr-profile="$ROOT/coverage-report/tests.profdata" \
    "${OBJ_ARGS[@]}" \
    --ignore-filename-regex="$IGNORE_RE" \
    2> >(tee "$WARNINGS" >&2) \
    | tee "$ROOT/coverage-report/report.txt"

# Why: each workspace builds into its own target dir, so a crate two of them
# compile (core, via the path patch) can leave a stray function or two with
# differing hashes. That is noise; a broken dedupe drops hundreds.
MISMATCHED=$(sed -n 's/^.*warning: \([0-9][0-9]*\) functions have mismatched data.*$/\1/p' "$WARNINGS" | head -n 1)
MISMATCH_LIMIT="${COVERAGE_MISMATCH_LIMIT:-10}"
if [ -n "$MISMATCHED" ] && [ "$MISMATCHED" -gt "$MISMATCH_LIMIT" ]; then
    echo "error: llvm-cov rejected profile data for $MISMATCHED functions (limit $MISMATCH_LIMIT); duplicate or incompatible instrumented objects remain" >&2
    echo "       inspect coverage-report/llvm-cov-warnings.txt" >&2
    exit 1
fi

"$LLVM_COV" export \
    --instr-profile="$ROOT/coverage-report/tests.profdata" \
    "${OBJ_ARGS[@]}" \
    --ignore-filename-regex="$IGNORE_RE" \
    --summary-only \
    > "$ROOT/coverage-report/summary.json"

"$LLVM_COV" export \
    --instr-profile="$ROOT/coverage-report/tests.profdata" \
    "${OBJ_ARGS[@]}" \
    --ignore-filename-regex="$IGNORE_RE" \
    --format=lcov \
    > "$ROOT/coverage-report/lcov.info"

TOTAL=$(jq -r '.data[0].totals.lines.percent' "$ROOT/coverage-report/summary.json")
printf '==> Total line coverage: %.2f%%\n' "$TOTAL"
echo "Reports: coverage-report/{report.txt,summary.json,lcov.info}"
echo "For HTML: just coverage-html"

if [ "$PHASE_FAILURES" -gt 0 ]; then
    echo "error: $PHASE_FAILURES workspace(s) did not run cleanly — the figure above is measured over a partial build" >&2
    exit 1
fi
exit 0
