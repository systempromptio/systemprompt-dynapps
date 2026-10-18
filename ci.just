# Shared CI recipes, byte-identical in systemprompt-astound and
# systemprompt-dynapps (imported by the justfile). Brand values come from
# .github/ci-params.env; nothing in this file may name a brand.
# scripts/ci-sync-check.sh fails the build when this file drifts from the
# manifest in .github/ci-shared.sha256.

ci_repo := `sed -n 's/^CI_REPO_NAME=//p' .github/ci-params.env`
ci_github_repo := `sed -n 's/^CI_GITHUB_REPO=//p' .github/ci-params.env`


# ══════════════════════════════════════════════════════════════════════════════
# PREFLIGHT (local stand-in for CI — tiered, cheapest first)
# ══════════════════════════════════════════════════════════════════════════════

# Everything: static gates → lint/doc/msrv → tests → coverage floor+ratchet.
# The mandatory gate before landing next -> main; CI re-runs the first three
# tiers on the merge commit, coverage stays local.
preflight: preflight-static preflight-lint test coverage-check

# Tier 0 — seconds. Formatting, sqlx cache freshness, and the source gates.
# The gates run UNCOORDINATED here on purpose: they are read-only shell
# checks, so queueing them on the build lock only made pre-push hang behind
# whoever was mid test run. `just lint-gates` stays coordinated for callers
# that want dedupe (clippy's dependency).
#
# Every check runs even after one fails, and the recipe fails if any did: a
# formatting slip must not hide thirty-six source gates until the next round.
preflight-static:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    check() { echo "==> $1"; shift; "$@" || failed+=("$*"); }
    check "fmt (root)" cargo fmt --all -- --check
    check "fmt (tests)" cargo fmt --manifest-path tests/Cargo.toml --all -- --check
    check "fmt (bridge)" cargo fmt --manifest-path bridge/Cargo.toml --all -- --check
    check "sqlx cache" bash scripts/check-sqlx-cache.sh
    check "core crate versions" bash scripts/check-core-crate-versions.sh
    check "build coordinator" bash tests/scripts/build-coordinator.sh
    check "deploy-release recipe present" bash -c '{{just_executable()}} --summary | tr " " "\n" | grep -qx deploy-release'
    check "source gates" {{just_executable()}} _lint-gates-uncoordinated
    if [ "${#failed[@]}" -gt 0 ]; then printf '::error::preflight-static failed: %s\n' "${failed[@]}"; exit 1; fi

# Tier 1 — compilers. Clippy (both workspaces), rustdoc as errors, MSRV.
# Each runs even after one fails, as in preflight-static.
preflight-lint:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    for recipe in clippy doc-check msrv-check; do
        echo "==> $recipe"
        {{just_executable()}} "$recipe" || failed+=("$recipe")
    done
    if [ "${#failed[@]}" -gt 0 ]; then printf '::error::preflight-lint failed: %s\n' "${failed[@]}"; exit 1; fi

# Weekly deep pass: preflight plus the network-touching supply-chain gates.
preflight-full: preflight deny audit machete hack

# Rustdoc with warnings denied, across all three workspaces (root, tests/,
# bridge/) — mirrors core's quality.yml docs job. Single-flight coordinated.
doc-check:
    @scripts/build-coordinator.sh run doc-check "" -- {{just_executable()}} _doc-check-uncoordinated

_doc-check-uncoordinated:
    SQLX_OFFLINE=true RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
    SQLX_OFFLINE=true RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path tests/Cargo.toml --workspace --no-deps
    RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path bridge/Cargo.toml --no-deps

# The workspace must build on the declared minimum supported Rust version
# (rust-version in Cargo.toml). Requires: rustup toolchain install 1.96.0
msrv-check:
    @scripts/build-coordinator.sh run msrv-check "" -- {{just_executable()}} _msrv-check-uncoordinated

_msrv-check-uncoordinated:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! rustup toolchain list | grep -q '^1\.96\.0'; then
        echo "msrv-check: toolchain 1.96.0 missing — run: rustup toolchain install 1.96.0" >&2
        exit 1
    fi
    SQLX_OFFLINE=true cargo +1.96.0 check --workspace

# ══════════════════════════════════════════════════════════════════════════════
# COVERAGE (raw llvm-cov; floor + ratchet vs tracked coverage/baseline.json)
# ══════════════════════════════════════════════════════════════════════════════

# Instrumented test run over all three workspaces; writes coverage-report/.
# See scripts/coverage.sh for the sccache/mold neutralisation notes.
coverage:
    @scripts/build-coordinator.sh run coverage "" -- bash scripts/coverage.sh

# Enforce the floor and ratchet recorded in coverage/baseline.json.
coverage-check: coverage
    bash scripts/coverage-check.sh

# Re-record coverage/baseline.json at the measured value (deliberate act —
# commit the result). Raise the "floor" field by hand as milestones land.
coverage-baseline: coverage
    UPDATE_BASELINE=1 bash scripts/coverage-check.sh

# Record tests/fixtures/schema/release-baseline-<version>.sql: the schema a
# fresh install of a release produces, plus its extension_migrations rows,
# dumped by the local Postgres container's own pg_dump (always the server's
# major — the host client may be older and refuse). With no argument the
# current tree is installed under the workspace version — run it after every
# version bump (scripts/check-schema-baseline.sh enforces that). With a
# version, that release's server tarball is fetched and ITS binary does the
# install, so a rung can be added for a release that shipped before the
# ladder existed. The upgrade test restores every rung into an empty database
# and runs the current installer over it; the ladder is append-only.
schema-baseline VERSION="":
    #!/usr/bin/env bash
    set -euo pipefail
    version="{{VERSION}}"
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
    if [ -n "$version" ]; then
        [ "$(uname -s)" = "Linux" ] || { echo "schema-baseline: release tarballs are Linux-only" >&2; exit 1; }
        case "$(uname -m)" in x86_64) arch=amd64 ;; aarch64|arm64) arch=arm64 ;; *) echo "schema-baseline: no tarball for $(uname -m)" >&2; exit 1 ;; esac
        name="{{ci_repo}}-server-$version-linux-$arch.tar.gz"
        echo "schema-baseline: downloading $name"
        gh release download "v$version" -R "{{ci_github_repo}}" -p "$name" -p SHA256SUMS -D "$tmp"
        (cd "$tmp" && grep " $name\$" SHA256SUMS | sha256sum -c - >/dev/null)
        tar xzf "$tmp/$name" -C "$tmp"
        cli="$tmp/${name%.tar.gz}/bin/systemprompt"
        core_ref="v$version"
    else
        if [ -z "${SCHEMA_BASELINE_CLI:-}" ]; then just build; fi
        version="$(awk '/^\[workspace\.package\]/{p=1;next}/^\[/{p=0}p&&/^version[[:space:]]*=/{gsub(/[[:space:]"]/,""); sub(/^version=/,""); print; exit}' Cargo.toml)"
        core_ref="$(tr -d '[:space:]' < bridge/CORE_REF)"
        cli="${SCHEMA_BASELINE_CLI:-{{CLI}}}"
        if [ -n "${SCHEMA_BASELINE_CLI:-}" ]; then
            [ "$("$cli" --version)" = "systemprompt ${core_ref#v}" ] || { echo "schema-baseline: CLI core version differs from pinned release" >&2; exit 1; }
            git diff --quiet "v${core_ref#v}" -- extensions/web/schema || { echo "schema-baseline: overridden CLI requires unchanged released schema" >&2; exit 1; }
        fi
    fi
    container="${SCHEMA_BASELINE_CONTAINER:-$(docker compose -p "$(just _project_name local)" -f .systemprompt/docker/local.yaml ps -q postgres)}"
    [ -n "$container" ] || { echo "schema-baseline: local Postgres is not running (just db-up)" >&2; exit 1; }
    base_url="$(jq -r '.database_url' .systemprompt/profiles/local/secrets.json)"
    scratch="sp_schema_baseline_$$"
    scratch_url="${base_url%/*}/$scratch"
    fixture="tests/fixtures/schema/release-baseline-$version.sql"
    mkdir -p "$(dirname "$fixture")"
    cleanup() { docker exec "$container" psql -U systemprompt -d postgres -qc "DROP DATABASE IF EXISTS \"$scratch\" WITH (FORCE)" >/dev/null 2>&1 || true; rm -rf "$tmp"; }
    trap cleanup EXIT
    docker exec "$container" psql -U systemprompt -d postgres -qc "CREATE DATABASE \"$scratch\""
    echo "schema-baseline: fresh install of $version (core $core_ref) into $scratch"
    # Why: core 0.65 ignores SYSTEMPROMPT_DATABASE_URL once a profile is named and
    # installs into the profile's own database; the flag is the override that bypasses it.
    if ! log="$("$cli" infra db migrate --profile local --database-url "$scratch_url" 2>&1)"; then
        echo "$log" | tail -30; echo "schema-baseline: fresh install failed" >&2; exit 1
    fi
    {
        echo "-- {{ci_repo}} release-baseline: $version (core $core_ref)"
        echo "-- Recorded by 'just schema-baseline' from a fresh install; the upgrade test"
        echo "-- restores it and migrates forward. Re-record after every version bump."
        docker exec "$container" pg_dump -U systemprompt --schema-only --no-owner --no-privileges --no-comments "$scratch"
        docker exec "$container" pg_dump -U systemprompt --data-only --inserts --no-owner --table=extension_migrations "$scratch"
    } | grep -v -e '^\\' -e "set_config('search_path'" > "$fixture"
    echo "schema-baseline: wrote $fixture ($(wc -l < "$fixture") lines)"
    git diff --stat -- "$fixture" | tail -1
    bash scripts/check-schema-baseline.sh

# Rewrite the README's coverage badge from coverage/baseline.json. Run it
# after `just coverage-baseline`; the `coverage-badge.sh --check` gate fails
# the build if the two disagree.
coverage-badge:
    bash scripts/coverage-badge.sh --write

# Browsable HTML tree from the last `just coverage` run.
coverage-html:
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="$(pwd)"
    if [ ! -f "$ROOT/coverage-report/tests.profdata" ]; then
        echo "Run 'just coverage' first" >&2
        exit 1
    fi
    TOOLDIR="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"
    TBASE="${COVERAGE_TARGET_DIR:-$ROOT/coverage-report/target}"
    BINS=$(for t in "$TBASE-root" "$TBASE-tests" "$TBASE-bridge"; do \
        find "$t/debug/deps" -maxdepth 1 -executable -type f ! -name '*.d' ! -name '*.so' -printf '%T@ %p\n' 2>/dev/null; \
    done | sort -rn | awk '{ base=$2; sub(".*/", "", base); sub(/-[0-9a-f]+$/, "", base); if (!seen[base]++) print $2 }')
    OBJ_ARGS=""
    for b in $BINS; do OBJ_ARGS="$OBJ_ARGS --object $b"; done
    mkdir -p "$ROOT/coverage-report/html"
    "$TOOLDIR/llvm-cov" show \
        --instr-profile="$ROOT/coverage-report/tests.profdata" \
        $OBJ_ARGS \
        --ignore-filename-regex="(\.cargo|/rustc/|/registry/|/debug/build/|/tests/|/target/|systemprompt-core/|{{ci_repo}}/src/(main|lib)\.rs|bridge/src/main\.rs|extensions/(cli|mcp)/[^/]+/src/main\.rs|extensions/cli/[^/]+/src/commands/|extensions/.*/extension\.rs|build\.rs)" \
        --format=html \
        --output-dir="$ROOT/coverage-report/html"
    echo "Coverage report: coverage-report/html/index.html"

# Remove all coverage artifacts (instrumented target dirs included).
# Refuses while a coordinated run holds the lock: coverage-report/ carries the
# instrumented test binaries, so deleting it mid-run makes every remaining test
# fail to exec ("No such file or directory", nextest exit 70) and the report
# come out at 0.00% — a failure that looks like a code regression and is not.
coverage-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    LOCK="${COORD_STATE_DIR:-{{ justfile_directory() }}/.build}/lock"
    if [ -d "$LOCK" ]; then
        PID="$(cat "$LOCK/pid" 2>/dev/null || echo)"
        if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
            echo "refusing: '$(cat "$LOCK/recipe" 2>/dev/null || echo run)' is running (pid $PID)." >&2
            echo "  Deleting coverage-report/ now would pull the instrumented binaries" >&2
            echo "  out from under it. Wait for it, or override with COVERAGE_CLEAN_FORCE=1." >&2
            [ "${COVERAGE_CLEAN_FORCE:-0}" = "1" ] || exit 1
        fi
    fi
    rm -rf coverage-report/

# Point git at the tracked hooks (pre-commit patch-marker guard + fast
# gates). There is deliberately NO pre-push hook: pushes to next are gated by
# CI (gates.yml), and `just preflight` is run manually when landing
# next -> main. Run once per clone.
init-hooks:
    git config core.hooksPath .githooks
    @echo "git hooks now sourced from .githooks/"

# Cross-file referential integrity for services/ (ACL entity ids, MCP ports)
validate:
    bash scripts/validate-services.sh

# Verify every production extension source has a `//!` module head
check-headers:
    bash scripts/check-file-headers.sh

# Observational Rust-standards audit — appends to ISSUE.md, never blocks
audit-standards:
    bash scripts/audit-rust-standards.sh

# 300-line ceiling on extension sources (same script CI runs)
file-size:
    bash scripts/check-file-size.sh

# Every Cargo workspace in the repo. `tests/` and `bridge/` are excluded from
# the root workspace, so a bare root-level scan silently skips their lockfiles.
# Keep in sync with `git ls-files '*Cargo.lock'`.
workspaces := ". tests bridge"

# Detect unused dependencies across every workspace
machete:
    #!/usr/bin/env bash
    set -euo pipefail
    for w in {{ workspaces }}; do
        echo "==> cargo machete: $w"
        (cd "$w" && cargo machete)
    done

# Supply-chain gates across every workspace: cargo-deny (licenses/bans/
# advisories, root deny.toml discovered via --manifest-path) and cargo-audit
deny:
    #!/usr/bin/env bash
    set -euo pipefail
    for w in {{ workspaces }}; do
        echo "==> cargo deny: $w"
        cargo deny --manifest-path "${w%/}/Cargo.toml" check
    done

check-bans:
    cargo deny check bans

audit:
    #!/usr/bin/env bash
    set -euo pipefail
    for w in {{ workspaces }}; do
        echo "==> cargo audit: $w"
        cargo audit --file "${w%/}/Cargo.lock"
    done

# Build every feature powerset (catches feature-flag drift); weekly tier only
hack:
    cargo hack --workspace --feature-powerset --depth 2 check

# The upgrade ladder's two newest rungs on a database of their own, the
# downstream mirror of core's schema-ladder.yml. tests/integration/schema-upgrade
# restores each recorded release (tests/fixtures/schema/release-baseline-*.sql),
# seeds it, runs the current installer and diffs against a fresh install;
# `just test-integration` runs every rung, this isolates the two a deployed
# instance is most likely to be on, so a migration that only breaks an
# established database is its own red context instead of one failure among
# the integration tier.
schema-ladder RUNGS="2":
    #!/usr/bin/env bash
    set -euo pipefail
    : "${SYSTEMPROMPT_TEST_DATABASE_URL:?schema-ladder needs SYSTEMPROMPT_TEST_DATABASE_URL}"
    mapfile -t rungs < <(ls tests/fixtures/schema/release-baseline-*.sql \
        | sed 's#.*/release-baseline-\(.*\)\.sql#\1#' | sort -V | tail -n "{{RUNGS}}")
    [ "${#rungs[@]}" -gt 0 ] || { echo "schema-ladder: no release-baseline rungs" >&2; exit 1; }
    filter=""
    for v in "${rungs[@]}"; do
        filter="${filter:+$filter | }test(=release_${v//./_}_upgrades_to_a_fresh_install)"
    done
    echo "schema-ladder: upgrading from ${rungs[*]}"
    cargo nextest run --no-fail-fast --manifest-path tests/Cargo.toml \
        -p schema-upgrade-tests -E "$filter"

# Shared CI files must match .github/ci-shared.sha256 (and, with a sibling
# checkout, the sibling's copies). Record a deliberate change with
# `scripts/ci-sync-check.sh --update` and land it in both repositories.
ci-sync-check *ARGS:
    bash scripts/ci-sync-check.sh "$@"

# CI's supply-chain tier: cargo-deny and cargo-audit over every workspace.
# The fetches run under scripts/ci-retry.sh and the checks run offline after
# them, so a network blip is retried and a real finding fails exactly once.
supply-chain:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    for w in {{ workspaces }}; do
        m="${w%/}/Cargo.toml"
        echo "==> cargo deny: $w"
        if bash scripts/ci-retry.sh 3 cargo deny --manifest-path "$m" fetch; then
            cargo deny --manifest-path "$m" check --disable-fetch || failed+=("deny $w")
        else
            failed+=("deny fetch $w")
        fi
    done
    # cargo-audit has no fetch-only command; clone the database it reads
    # (its default path) under the retry, then audit with --no-fetch.
    bash scripts/ci-retry.sh 3 bash -c 'rm -rf "$HOME/.cargo/advisory-db" && git clone --quiet --depth 1 https://github.com/rustsec/advisory-db.git "$HOME/.cargo/advisory-db"' \
        || failed+=("audit fetch")
    for w in {{ workspaces }}; do
        echo "==> cargo audit: $w"
        cargo audit --no-fetch --file "${w%/}/Cargo.lock" || failed+=("audit $w")
    done
    if [ "${#failed[@]}" -gt 0 ]; then printf '::error::supply-chain failed: %s\n' "${failed[@]}"; exit 1; fi
