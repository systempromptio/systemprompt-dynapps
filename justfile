# systemprompt-astound
set dotenv-load
# Without this, `just cli ... --full-name "Test User"` word-splits the quoted
# value into two arguments before the CLI ever parses it.
set positional-arguments

# check-dead-repository-code can consult a sibling checkout so a symbol only
# it consumes is not reported dead. Optional; an already-set value wins.
export SIBLING_REPO := env("SIBLING_REPO", if path_exists("../systemprompt-template") == "true" { "../systemprompt-template" } else { "" })

CLI_RELEASE := "target/release/systemprompt"

# Cloud profile every deploy targets: tenant a2f658d8bc5f, Fly app
# sp-a2f658d8bc5f, served at https://astound.systemprompt.io.
# See .systemprompt/profiles/production/.
DEPLOY_PROFILE := "production"
# Where .github/workflows/release.yml publishes the gateway image.
IMAGE := "ghcr.io/astound-digital/systemprompt-astound"

# Use newest binary (release vs debug, whichever is most recent)
CLI := if path_exists("target/release/systemprompt") == "true" { \
    if path_exists("target/debug/systemprompt") == "true" { \
        `[ target/release/systemprompt -nt target/debug/systemprompt ] && echo target/release/systemprompt || echo target/debug/systemprompt` \
    } else { \
        "target/release/systemprompt" \
    } \
} else if path_exists("target/debug/systemprompt") == "true" { \
    "target/debug/systemprompt" \
} else { \
    "echo 'ERROR: No CLI binary found. Run: just build' && exit 1" \
}

# Default: run CLI with any arguments
default *ARGS:
    {{CLI}} "$@"

# Run CLI with full session context (profile + auth token)
cli *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    SESSION_FILE="{{justfile_directory()}}/.systemprompt/sessions/index.json"
    if [ -f "$SESSION_FILE" ]; then
        ACTIVE_KEY=$(jq -r '.active_key // "local"' "$SESSION_FILE")
        export SYSTEMPROMPT_PROFILE=$(jq -r ".sessions[\"$ACTIVE_KEY\"].profile_path // empty" "$SESSION_FILE")
        export SYSTEMPROMPT_AUTH_TOKEN=$(jq -r ".sessions[\"$ACTIVE_KEY\"].session_token // empty" "$SESSION_FILE")
    fi
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    exec {{CLI}} "$@"

# Print a single-use /admin login link for USER (development profile only)
dev-login USER:
    #!/usr/bin/env bash
    set -euo pipefail
    export SYSTEMPROMPT_PROFILE="${SYSTEMPROMPT_PROFILE:-{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml}"
    exec {{CLI}} plugins run dev-login "{{USER}}"

# Get DATABASE_URL from profile secrets (for sqlx compile-time checks)
_db-url:
    @PROFILE="${SYSTEMPROMPT_PROFILE:-}"; \
    [ -n "$PROFILE" ] || PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"; \
    if [ -f "$PROFILE" ]; then \
        PROFILE_DIR="$(dirname "$PROFILE")"; \
        SECRETS_PATH="$(yq -r '.secrets.secrets_path // "./secrets.json"' "$PROFILE")"; \
        if [ "${SECRETS_PATH#/}" = "$SECRETS_PATH" ]; then \
            SECRETS_FILE="$PROFILE_DIR/$SECRETS_PATH"; \
        else \
            SECRETS_FILE="$SECRETS_PATH"; \
        fi; \
        if [ -f "$SECRETS_FILE" ]; then \
            jq -r '.database_url' "$SECRETS_FILE"; \
        else \
            echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"; \
        fi; \
    else \
        cat .systemprompt/tenants.json 2>/dev/null | jq -r '.tenants[] | select(.tenant_type == "local") | .database_url' | head -1 || echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"; \
    fi

# ══════════════════════════════════════════════════════════════════════════════
# BUILD & CHECK
# ══════════════════════════════════════════════════════════════════════════════

# Build (Windows) - always uses offline mode
[windows]
build *FLAGS:
    $env:SQLX_OFFLINE="true"; cargo build --workspace {{FLAGS}}

# Build (Unix) - single-flight: dedupes concurrent identical builds across agents
[unix]
build *FLAGS:
    @scripts/build-coordinator.sh run build "{{FLAGS}}" -- {{just_executable()}} _build-uncoordinated {{FLAGS}}

# What is the build/lint/test state right now? Read this before running anything.
[unix]
build-status *RECIPE:
    @scripts/build-coordinator.sh status {{RECIPE}}

# Re-run even if the coordinator considers this source tree already green
[unix]
build-force *FLAGS:
    @BUILD_FORCE=1 scripts/build-coordinator.sh run build "{{FLAGS}}" -- {{just_executable()}} _build-uncoordinated {{FLAGS}}

# The real build. Call `just build` instead - this one skips coordination.
[unix]
_build-uncoordinated *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    # Default to the `local` profile when one is set up but no SYSTEMPROMPT_PROFILE
    # is explicitly exported — keeps the in-build migrate step from failing
    # with "Profile '' not found" on a fresh clone where setup-local writes
    # secrets.json before invoking `just build`.
    SECRETS_FILE_DEFAULT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ] && [ -f "$SECRETS_FILE_DEFAULT_PROFILE" ]; then
        export SYSTEMPROMPT_PROFILE="local"
    else
        export SYSTEMPROMPT_PROFILE="${SYSTEMPROMPT_PROFILE:-}"
    fi
    # CC=clang comes from .cargo/config.toml so every recipe fingerprints alike;
    # this only keeps the historical guard for a box without clang.
    if command -v clang >/dev/null 2>&1; then
        export CC="${CC:-clang}"
        export CXX="${CXX:-clang++}"
    fi
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false
    # Why: an explicit SQLX_OFFLINE=true wins over the live-DB probe below.
    # The online path verifies every query against the running database, and
    # some crates deliberately query tables that database will never have:
    # knowledge-bank's schema is unregistered on purpose (it opens with
    # CREATE EXTENSION vector, which the deployment target does not ship, and
    # registering it crash-loops the platform on boot). Those queries are
    # covered by the committed .sqlx/ cache and can only ever build offline, so
    # first-run setup and anyone with a fresh database needs this escape hatch.
    if [ "${SQLX_OFFLINE:-}" = "true" ]; then
        echo "SQLX_OFFLINE=true set explicitly, using offline mode"
        USE_OFFLINE=true
    fi
    db_reachable() {
        local url="$1"
        local pgcmd=""
        if command -v pg_isready >/dev/null 2>&1; then pgcmd="pg_isready"
        elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then pgcmd="/opt/homebrew/opt/libpq/bin/pg_isready"
        elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then pgcmd="/usr/local/opt/libpq/bin/pg_isready"
        fi
        if [ -n "$pgcmd" ]; then
            "$pgcmd" -d "$url" -t 2 >/dev/null 2>&1 && return 0 || return 1
        fi
        local hostport="${url#*@}"; hostport="${hostport%%/*}"
        local host="${hostport%:*}"; local port="${hostport##*:}"
        [ "$port" = "$host" ] && port=5432
        (exec 3<>/dev/tcp/"$host"/"$port") >/dev/null 2>&1 && { exec 3<&-; exec 3>&-; return 0; } || return 1
    }
    if [ "$USE_OFFLINE" = "false" ] && [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SECRETS_FILE" 2>/dev/null | head -1)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
            if db_reachable "$DB_URL"; then
                export DATABASE_URL="$DB_URL"
                echo "Using database: $DB_URL"
            else
                echo "Database not reachable, using offline mode"
                USE_OFFLINE=true
            fi
        else
            echo "No database_url in secrets, using offline mode"
            USE_OFFLINE=true
        fi
    elif [ "$USE_OFFLINE" = "false" ]; then
        echo "No local profile secrets found, using offline mode"
        USE_OFFLINE=true
    fi
    # Sync DATABASE_URL to MCP extension directories for sqlx compile-time checks
    if [ "$USE_OFFLINE" = "false" ]; then
        for dir in extensions/mcp/*/; do
            if [ -f "$dir/Cargo.toml" ]; then
                echo "DATABASE_URL=$DATABASE_URL" > "$dir/.env"
            fi
        done
    fi
    cargo update systemprompt --quiet 2>/dev/null || true
    if [ "$USE_OFFLINE" = "true" ]; then
        SQLX_OFFLINE=true cargo build --workspace {{FLAGS}}
    else
        # Apply pending schema migrations before the online sqlx compile-time
        # check sees the live DB. Build the CLI in offline mode first so
        # drift between checked-in `.sqlx/` and the unmigrated live schema
        # can't deadlock the bootstrap.
        echo "Applying pending migrations before online build..."
        SQLX_OFFLINE=true cargo build --bin systemprompt --quiet
        target/debug/systemprompt infra db migrate
        SQLX_OFFLINE=false cargo build --workspace {{FLAGS}}
    fi

# Clippy (Windows) - always uses offline mode
[windows]
clippy *FLAGS: lint-no-synthesis lint-no-untyped-admin lint-gates
    $env:SQLX_OFFLINE="true"; cargo clippy --workspace {{FLAGS}} -- -D warnings

# Clippy (Unix) - single-flight, same coordinator as `just build`
[unix]
clippy *FLAGS:
    @scripts/build-coordinator.sh run clippy "{{FLAGS}}" -- {{just_executable()}} _clippy-uncoordinated {{FLAGS}}

# The real clippy. Call `just clippy` instead - this one skips coordination.
[unix]
_clippy-uncoordinated *FLAGS: lint-no-synthesis lint-no-untyped-admin lint-gates
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false
    # Why: same escape hatch as `build`. knowledge-bank's queries name tables the
    # main database never has (its schema is unregistered on purpose), so the
    # online probe fails them; the committed .sqlx/ cache is their only check.
    if [ "${SQLX_OFFLINE:-}" = "true" ]; then
        echo "SQLX_OFFLINE=true set explicitly, using offline mode"
        USE_OFFLINE=true
    fi
    db_reachable() {
        local url="$1"
        local pgcmd=""
        if command -v pg_isready >/dev/null 2>&1; then pgcmd="pg_isready"
        elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then pgcmd="/opt/homebrew/opt/libpq/bin/pg_isready"
        elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then pgcmd="/usr/local/opt/libpq/bin/pg_isready"
        fi
        if [ -n "$pgcmd" ]; then
            "$pgcmd" -d "$url" -t 2 >/dev/null 2>&1 && return 0 || return 1
        fi
        local hostport="${url#*@}"; hostport="${hostport%%/*}"
        local host="${hostport%:*}"; local port="${hostport##*:}"
        [ "$port" = "$host" ] && port=5432
        (exec 3<>/dev/tcp/"$host"/"$port") >/dev/null 2>&1 && { exec 3<&-; exec 3>&-; return 0; } || return 1
    }
    if [ "$USE_OFFLINE" = "false" ] && [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SECRETS_FILE" 2>/dev/null | head -1)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
            if db_reachable "$DB_URL"; then
                export DATABASE_URL="$DB_URL"
            else
                USE_OFFLINE=true
            fi
        else
            USE_OFFLINE=true
        fi
    elif [ "$USE_OFFLINE" = "false" ]; then
        USE_OFFLINE=true
    fi
    if [ "$USE_OFFLINE" = "true" ]; then
        SQLX_OFFLINE=true cargo clippy --workspace {{FLAGS}} -- -D warnings
    else
        SQLX_OFFLINE=false cargo clippy --workspace {{FLAGS}} -- -D warnings
    fi
    # bridge/ is a standalone workspace and is not covered by --workspace
    cargo clippy --manifest-path bridge/Cargo.toml --all-targets {{FLAGS}} -- -D warnings

# Unit tests: extensions/web/admin (main workspace) + the tests/ workspace.
# If sqlx offline errors appear, run `just prepare` first to refresh .sqlx.
test-unit:
    @scripts/build-coordinator.sh run test-unit "" -- {{just_executable()}} _test-unit-uncoordinated

_test-unit-uncoordinated:
    cargo nextest run --manifest-path tests/Cargo.toml -p mcp-unit-tests -p web-unit-tests

# DB-backed integration tests. Creates/drops throwaway mcp_ext_test_*
# databases on the maintenance DB; the harness guard refuses any database
# name that is not 'test', 'postgres', or '*_test'. Falls back to the local
# profile's server with the database swapped to 'postgres'.
test-integration:
    @scripts/build-coordinator.sh run test-integration "" -- {{just_executable()}} _test-integration-uncoordinated

_test-integration-uncoordinated:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ] && [ -f .systemprompt/profiles/local/secrets.json ]; then
        SYSTEMPROMPT_TEST_DATABASE_URL=$(python3 -c "
    import json, urllib.parse as up
    u = up.urlsplit(json.load(open('.systemprompt/profiles/local/secrets.json'))['database_url'])
    print(up.urlunsplit((u.scheme, u.netloc, '/postgres', '', '')))")
        export SYSTEMPROMPT_TEST_DATABASE_URL
    fi
    # Why: falling through with no URL hands the suites a reason to skip, and a
    # skipped DB tier reports the same green as one that ran every assertion.
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ]; then
        echo "No test database. Set SYSTEMPROMPT_TEST_DATABASE_URL, or run \`just setup-local\`" >&2
        echo "so .systemprompt/profiles/local/secrets.json carries a database_url." >&2
        exit 1
    fi
    cargo nextest run --manifest-path tests/Cargo.toml -p mcp-integration-tests -p web-integration-tests -p admin-db-core-tests -p admin-db-config-tests -p gateway-integration-tests

# HTTP contract suite: drives every admin route under three principals and
# diffs the result against tests/contract/admin/baseline.txt. Same throwaway-
# database convention as test-integration. A status change fails the run; if
# it is deliberate, re-record with UPDATE_CONTRACT_BASELINE=1 and list it in
# the PR.
test-contract:
    @scripts/build-coordinator.sh run test-contract "" -- {{just_executable()}} _test-contract-uncoordinated

_test-contract-uncoordinated:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ] && [ -f .systemprompt/profiles/local/secrets.json ]; then
        SYSTEMPROMPT_TEST_DATABASE_URL=$(python3 -c "
    import json, urllib.parse as up
    u = up.urlsplit(json.load(open('.systemprompt/profiles/local/secrets.json'))['database_url'])
    print(up.urlunsplit((u.scheme, u.netloc, '/postgres', '', '')))")
        export SYSTEMPROMPT_TEST_DATABASE_URL
    fi
    # Why: falling through with no URL hands the suites a reason to skip, and a
    # skipped DB tier reports the same green as one that ran every assertion.
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ]; then
        echo "No test database. Set SYSTEMPROMPT_TEST_DATABASE_URL, or run \`just setup-local\`" >&2
        echo "so .systemprompt/profiles/local/secrets.json carries a database_url." >&2
        exit 1
    fi
    # Why: the contract suite self-skips when no database is reachable, and it
    # carries the governance privilege-escalation check. This turns that skip
    # into a failure, so a missing database can never be read as a pass.
    export SYSTEMPROMPT_REQUIRE_DB=1
    cargo nextest run --manifest-path tests/Cargo.toml -p admin-contract-tests

# All tests
test: test-unit test-integration test-contract

# Source gates ported from systemprompt-core (scripts/*.sh)
# Reject tests that return early on a missing prerequisite without saying so
lint-silent-skips:
    ./scripts/lint-silent-skips.sh tests

lint-gates:
    @scripts/build-coordinator.sh run lint-gates "" -- {{just_executable()}} _lint-gates-uncoordinated

# Gates are independent read-only checks; they run concurrently and every
# failure is reported, so one red gate cannot hide the rest.
_lint-gates-uncoordinated:
    #!/usr/bin/env bash
    set -uo pipefail
    gates=(
        check-bridge-lifecycle.sh
        check-discarded-results.sh
        check-fail-open.sh
        lint-schema.sh
        lint-extensions.sh
        check-migration-numbers.sh
        lint-layers.sh
        lint-repo-construction.sh
        check-json-value.sh
        check-sqlx.sh
        check-http-errors.sh
        check-test-value.sh
        lint-silent-skips.sh
        lint-raw-ids.sh
        check-glob-reexports.sh
        check-comments.sh
        lint-inline-comments.sh
        check-duplicate-types.sh
        check-repository-naming.sh
        check-admin-template-links.sh
        check-admin-template-assets.sh
        # admin-css-classes + frontend-standards now run as cargo tests in
        # extensions/web/tests/ (admin_css_classes.rs, frontend_standards.rs).
        check-dead-repository-code.sh
        check-file-headers.sh
        check-file-size.sh
        check-asset-reachability.sh
        check-workspace-deps.sh
        validate-services.sh
        check-core-ref.sh
        coverage-badge.sh
        check-docs-version.sh
        check-spec-shape.sh
        check-req-tags.sh
        check-template-fields.sh
    )
    logdir=$(mktemp -d)
    trap 'rm -rf "$logdir"' EXIT
    pids=()
    for gate in "${gates[@]}"; do
        bash "scripts/$gate" >"$logdir/$gate.log" 2>&1 &
        pids+=("$!:$gate")
    done
    failed=()
    for entry in "${pids[@]}"; do
        pid=${entry%%:*}
        gate=${entry#*:}
        if ! wait "$pid"; then
            failed+=("$gate")
        fi
    done
    if [ ${#failed[@]} -gt 0 ]; then
        for gate in "${failed[@]}"; do
            echo "==== FAILED: $gate ===="
            cat "$logdir/$gate.log"
        done
        echo "lint gates failed: ${failed[*]}"
        exit 1
    fi
    echo "all ${#gates[@]} lint gates passed"

# The whole gate, in one command — exactly what .github/workflows/gates.yml
# runs on every push to next and, via release.yml, on every push to main. Run
# it before you push so CI is confirmation, not discovery. `preflight` adds
# the coverage floor/ratchet on top; use it before landing next -> main.
verify: preflight-static preflight-lint test
    @echo "verify: format, sqlx cache, lint gates, clippy, docs, msrv, and tests all pass"

# ══════════════════════════════════════════════════════════════════════════════
# PREFLIGHT (local stand-in for CI — tiered, cheapest first)
# ══════════════════════════════════════════════════════════════════════════════

# Everything: static gates → lint/doc/msrv → tests → coverage floor+ratchet.
# The mandatory gate before landing next -> main; CI re-runs the first three
# tiers on the merge commit, coverage stays local.
preflight: preflight-static preflight-lint test coverage-check e2e-gate

# The browser tier of preflight. It needs a live stack, which `just preflight`
# cannot start for you (the server on this clone may be shared), so this fails
# loudly with the command to run rather than skipping quietly — a preflight that
# silently omits the e2e suite is worse than one that stops.
e2e-gate:
    #!/usr/bin/env bash
    set -euo pipefail
    URL="${GATEWAY_URL:-http://localhost:8080}"
    if ! curl -fsS -o /dev/null "$URL/health"; then
        echo "preflight: the e2e tier needs a running stack at $URL." >&2
        echo "           Run 'just start' (or set GATEWAY_URL) and re-run preflight." >&2
        exit 1
    fi
    {{just_executable()}} e2e --project chromium

# Tier 0 — seconds. Formatting, sqlx cache freshness, and the source gates.
# The gates run UNCOORDINATED here on purpose: they are read-only shell
# checks, so queueing them on the build lock only made pre-push hang behind
# whoever was mid test run. `just lint-gates` stays coordinated for callers
# that want dedupe (clippy's dependency).
preflight-static:
    cargo fmt --all -- --check
    cargo fmt --manifest-path tests/Cargo.toml --all -- --check
    cargo fmt --manifest-path bridge/Cargo.toml --all -- --check
    bash scripts/check-sqlx-cache.sh
    {{just_executable()}} _lint-gates-uncoordinated

# Tier 1 — compilers. Clippy (both workspaces), rustdoc as errors, MSRV.
preflight-lint:
    {{just_executable()}} clippy
    {{just_executable()}} doc-check
    {{just_executable()}} msrv-check

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
        --ignore-filename-regex="(\.cargo|/rustc/|/registry/|/debug/build/|/tests/|/target/|systemprompt-core/|systemprompt-astound/src/(main|lib)\.rs|bridge/src/main\.rs|extensions/(cli|mcp)/[^/]+/src/main\.rs|extensions/cli/[^/]+/src/commands/|extensions/.*/extension\.rs|build\.rs)" \
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

# Structural guard: `UserId::admin()` is banned outside sanctioned call sites.
# The allowlist is empty by design — this repo has no sanctioned site; adding
# one requires justification in review.
lint-no-untyped-admin:
    #!/usr/bin/env bash
    set -euo pipefail
    hits=$(grep -rn 'UserId::admin()' extensions/ src/ bridge/src/ --include='*.rs' 2>/dev/null \
        | grep -v '/tests/' \
        || true)
    if [ -n "$hits" ]; then
        echo "lint-no-untyped-admin: untyped UserId::admin() outside the sanctioned call sites:"
        echo "$hits"
        exit 1
    fi

# Structural guard: no string-literal `UserId::new("...")` in extension code.
# String literals are how principal synthesis sneaks in — every legitimate
# UserId::new call takes a validated identifier as a variable, never a literal.
# Allowlisted: test code (regression tests intentionally construct ids) and
# any future bootstrap/provisioning module.
lint-no-synthesis:
    #!/usr/bin/env bash
    set -euo pipefail
    hits=$(grep -rEn 'UserId::new\("' extensions/ \
        --include='*.rs' \
        --exclude-dir=tests \
        --exclude-dir=bootstrap \
        || true)
    if [ -n "$hits" ]; then
        echo "error: forbidden synthesized principal — UserId::new with string literal"
        echo "$hits"
        echo
        echo "UserId::new must take a validated identifier (from cookie, query,"
        echo "JWT claim, or DB row), never a hard-coded literal. If this is"
        echo "legitimate bootstrap code, move it to extensions/**/bootstrap/."
        exit 1
    fi

# Prepare SQLx offline query cache (requires running database)
prepare:
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    if [ ! -f "$SECRETS_FILE" ]; then
        echo "Error: No local profile secrets found at $SECRETS_FILE"
        echo "Run 'just db-up' first to start the database"
        exit 1
    fi
    DB_URL=$(jq -r '.database_url // empty' "$SECRETS_FILE" 2>/dev/null)
    if [ -z "$DB_URL" ] || [ "$DB_URL" = "null" ]; then
        echo "Error: No database_url in secrets"
        exit 1
    fi
    PG_ISREADY=""
    if command -v pg_isready >/dev/null 2>&1; then PG_ISREADY="pg_isready"
    elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then PG_ISREADY="/opt/homebrew/opt/libpq/bin/pg_isready"
    elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then PG_ISREADY="/usr/local/opt/libpq/bin/pg_isready"
    fi
    if [ -z "$PG_ISREADY" ] || ! "$PG_ISREADY" -d "$DB_URL" -t 2 >/dev/null 2>&1; then
        echo "Error: Database not reachable at $DB_URL"
        echo "Run 'just db-up' first to start the database"
        exit 1
    fi
    # Apply pending migrations before sqlx prepare — otherwise the macros
    # see a schema older than the code references and fail with
    # "relation ... does not exist". Skipped if no CLI binary exists yet
    # (first-time bootstrap before any build).
    if [ -x "{{CLI}}" ]; then
        echo "Applying pending migrations..."
        {{CLI}} infra db migrate --profile local
    else
        echo "Warning: no systemprompt binary yet; skipping migrate step."
        echo "  If sqlx prepare fails with 'relation does not exist',"
        echo "  build first ('just build') then re-run 'just prepare'."
    fi
    # The knowledge-bank tables are not migrations: the binary applies its own
    # schema at boot. Apply it here too, or its queries have nothing to verify
    # against and prepare prunes them from the cache.
    PSQL=""
    if command -v psql >/dev/null 2>&1; then PSQL="psql"
    elif [ -x /opt/homebrew/opt/libpq/bin/psql ]; then PSQL="/opt/homebrew/opt/libpq/bin/psql"
    elif [ -x /usr/local/opt/libpq/bin/psql ]; then PSQL="/usr/local/opt/libpq/bin/psql"
    fi
    if [ -n "$PSQL" ]; then
        echo "Applying the knowledge-bank schema..."
        "$PSQL" -q -v ON_ERROR_STOP=1 "$DB_URL" -f extensions/mcp/knowledge-bank/schema/knowledge_bank.sql
        "$PSQL" -q -v ON_ERROR_STOP=1 "$DB_URL" -f extensions/mcp/knowledge-bank/schema/google_drive.sql
    else
        echo "Warning: psql not found; knowledge-bank queries verify only if its tables already exist."
    fi
    echo "Preparing SQLx offline cache..."
    export DATABASE_URL="$DB_URL"
    ROOT="{{justfile_directory()}}"
    # Crates whose own .sqlx cache this recipe creates as scratch. Declared up
    # here because the failure path has to clean them too.
    EXTENSION_DIRS="extensions/web extensions/mcp/shared extensions/mcp/systemprompt extensions/mcp/knowledge-bank"
    # `cargo sqlx prepare` truncates and rewrites .sqlx as it goes, and this
    # recipe runs it three times. A pass that fails part-way therefore leaves
    # the cache worse than it found it — observed once as 55 tracked entries
    # deleted before the error, recoverable only by `git checkout -- .sqlx`,
    # which is not obvious mid-incident. Snapshot everything this recipe can
    # write, and put it all back unless every pass succeeded.
    SNAP="$(mktemp -d)"
    HAD_ROOT_CACHE=0
    if [ -d "$ROOT/.sqlx" ]; then
        HAD_ROOT_CACHE=1
        mkdir -p "$SNAP/root"
        cp -a "$ROOT/.sqlx/." "$SNAP/root/"
    fi
    SNAPSHOTTED_EXT_DIRS=""
    for dir in $EXTENSION_DIRS; do
        if [ -d "$ROOT/$dir/.sqlx" ]; then
            slug=$(echo "$dir" | tr '/' '_')
            mkdir -p "$SNAP/ext/$slug"
            cp -a "$ROOT/$dir/.sqlx/." "$SNAP/ext/$slug/"
            SNAPSHOTTED_EXT_DIRS="$SNAPSHOTTED_EXT_DIRS $dir"
        fi
    done
    PREPARE_OK=0
    restore_sqlx_cache() {
        rc=$?
        rm -rf "$ROOT/.sqlx.default-pass"
        for dir in $EXTENSION_DIRS; do
            rm -rf "$ROOT/$dir/.sqlx"
        done
        if [ "$PREPARE_OK" -eq 1 ]; then
            rm -rf "$SNAP"
            exit $rc
        fi
        rm -rf "$ROOT/.sqlx"
        if [ "$HAD_ROOT_CACHE" -eq 1 ]; then
            mkdir -p "$ROOT/.sqlx"
            cp -a "$SNAP/root/." "$ROOT/.sqlx/"
        fi
        for dir in $SNAPSHOTTED_EXT_DIRS; do
            slug=$(echo "$dir" | tr '/' '_')
            mkdir -p "$ROOT/$dir/.sqlx"
            cp -a "$SNAP/ext/$slug/." "$ROOT/$dir/.sqlx/"
        done
        rm -rf "$SNAP"
        echo ""
        echo "prepare FAILED — the .sqlx cache was restored to exactly what it was"
        echo "before this run. No entries were added, removed, or rewritten."
        exit $rc
    }
    trap restore_sqlx_cache EXIT
    # Drop the incremental artifacts of every crate that uses sqlx, so each
    # query macro re-expands against the freshly-migrated schema.
    #
    # This has to be all of them, not just the crate whose schema changed.
    # `cargo sqlx prepare` collects query data emitted by macro expansion, so
    # a crate cargo considers fresh contributes nothing to the run and its
    # queries are pruned from .sqlx as though they no longer existed. That is
    # what made prepare non-deterministic: a cold cache re-expanded everything
    # and kept the full set, while a warm one silently dropped whatever it did
    # not rebuild (the event_outbox queries from systemprompt-events being the
    # usual casualty). The emitted set must not depend on target/ state.
    #
    # Dependencies count too, not just workspace members — their queries land
    # in the workspace cache the same way.
    SQLX_PKGS=$(cargo metadata --format-version 1 2>/dev/null \
        | jq -r '.packages[] | select(.dependencies[]?.name == "sqlx") | .name' \
        | sort -u)
    if [ -z "$SQLX_PKGS" ]; then
        echo "Error: could not resolve the sqlx-dependent package list."
        echo "Without it, prepare would prune queries it simply did not rebuild."
        exit 1
    fi
    for pkg in $SQLX_PKGS; do
        cargo clean -p "$pkg" 2>/dev/null || true
    done
    # Workspace-level prepare (catches lib crates)
    cargo sqlx prepare --workspace
    # Second pass: the tests workspace builds systemprompt-web-admin with
    # `governance-ssr`, whose queries a default-feature prepare never expands
    # (and therefore prunes). prepare regenerates .sqlx wholesale, so the two
    # passes are unioned by hand — query filenames are content hashes, so a
    # plain copy merge cannot collide.
    cp -a .sqlx .sqlx.default-pass
    cargo sqlx prepare --workspace -- --package systemprompt-web-admin --features systemprompt-web-admin/governance-ssr
    cp -n .sqlx.default-pass/query-*.json .sqlx/ 2>/dev/null || true
    rm -rf .sqlx.default-pass
    # Per-crate prepare for binary/extension crates that cargo sqlx skips.
    #
    # These caches are SCRATCH, not artefacts: each crate's queries are copied
    # into the root .sqlx below and the per-crate directory is then removed.
    # Every crate here is a root workspace member, so the root cache is the one
    # the macros resolve against — `extensions/mcp/systemprompt` is a member
    # that has never had its own cache and builds fine, and a workspace-wide
    # `SQLX_OFFLINE=true cargo check --all-targets` passes with all of these
    # deleted. Leaving them on disk committed 1,703 duplicate files (690 for
    # knowledge-bank, 927 for web, 86 for mcp/shared), which is why a routine
    # prepare showed up as a 1,374-file diff.
    for dir in $EXTENSION_DIRS; do
        if [ -f "{{justfile_directory()}}/$dir/Cargo.toml" ]; then
            # Skip crates with no sqlx dependency — prepare would only
            # resurrect an orphaned .sqlx cache.
            if ! grep -qE '^sqlx' "{{justfile_directory()}}/$dir/Cargo.toml"; then
                continue
            fi
            echo "  Preparing $dir..."
            # Why: `... | tail -1` made the pipeline's status tail's, and the
            # trailing `|| true` discarded even that. A per-crate pass could
            # fail and its queries silently never reach the root cache — the
            # same shape as the bug this recipe exists to prevent.
            if ! ext_log=$(cd "{{justfile_directory()}}/$dir" && cargo sqlx prepare 2>&1); then
                echo "$ext_log" | tail -20
                echo "Error: cargo sqlx prepare failed in $dir"
                exit 1
            fi
            echo "$ext_log" | tail -1
            if ls "{{justfile_directory()}}/$dir/.sqlx/"*.json >/dev/null 2>&1; then
                cp "{{justfile_directory()}}/$dir/.sqlx/"*.json "{{justfile_directory()}}/.sqlx/"
            fi
            # Scratch, not an artefact — the queries now live in the root cache.
            rm -rf "{{justfile_directory()}}/$dir/.sqlx"
        fi
    done
    PREPARE_OK=1
    echo "SQLx cache prepared successfully ($(ls {{justfile_directory()}}/.sqlx/ | wc -l) queries cached)"

# ══════════════════════════════════════════════════════════════════════════════
# SERVICES & DATABASE
# ══════════════════════════════════════════════════════════════════════════════

# Start server (always uses local profile)
start:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f .systemprompt/docker/local.yaml ]; then
        just db-up local
    fi
    exec {{CLI}} infra services start --profile local

# Optional: running server + binary provenance + recent build/lint/test results
[unix]
server-status:
    @scripts/server-state.sh report

# Start server with release binary
start-release:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f .systemprompt/docker/local.yaml ]; then
        just db-up local
    fi
    exec {{CLI_RELEASE}} infra services start --profile local

# Stop this clone's services
stop:
    {{CLI}} infra services stop --all

# Run migrations
migrate:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    {{CLI}} infra db migrate

# When an already-applied migration file is edited (e.g. a seed fix), its
# stored checksum stops matching the file and `migrate` / `start` refuse to
# proceed. `infra db migrate-repair` re-aligns the tracking table by dropping
# the drifted rows and re-applying those migrations — every migration is
# idempotent (guarded seeds or CREATE ... IF NOT EXISTS), so re-running them
# re-records the current checksum without touching your data.
# Repair migration checksum drift in place — no data loss, no destructive reset.
repair-migrations:
    {{CLI}} infra db migrate-repair --apply

# Per-clone docker compose project name. Derived from the absolute justfile directory
# so a second clone on the same host gets its own containers and volumes.
_project_name TENANT:
    #!/usr/bin/env bash
    set -euo pipefail
    HASH=$(printf '%s' "{{justfile_directory()}}" | { sha256sum 2>/dev/null || shasum -a 256; } | cut -c1-8)
    LEAF=$(basename "{{justfile_directory()}}" | tr '_' '-' | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]/-/g')
    printf 'sp-%s-%s-%s\n' "$LEAF" "$HASH" "{{TENANT}}"

# Start PostgreSQL for a specific tenant (default: local)
db-up TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml up -d

# Stop PostgreSQL for a specific tenant
db-down TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml down

# Show PostgreSQL logs for a specific tenant
db-logs TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml logs -f

# List all tenant databases
db-list:
    @ls -1 .systemprompt/docker/*.yaml 2>/dev/null | xargs -I {} basename {} .yaml || echo "No tenant databases found"

# ══════════════════════════════════════════════════════════════════════════════
# AUTH & TENANT & PROFILE
# ══════════════════════════════════════════════════════════════════════════════

# Authenticate with SystemPrompt Cloud
login ENV="production":
    {{CLI}} cloud auth login {{ENV}}

# Clear saved credentials
logout:
    {{CLI}} cloud auth logout

# Show current user and tenant
whoami:
    {{CLI}} cloud auth whoami

# Tenant operations (interactive menu)
tenant:
    {{CLI}} cloud tenant

# Set up a local-only profile + Docker Postgres (no cloud, no login required).
# Pass keys as positional args, or leave blank to be prompted interactively:
#   just setup-local sk-ant-... sk-... AIza...
# Port and Postgres port can be overridden for running multiple clones on one host:
#   just setup-local sk-ant-... "" "" 8081 5433
# A bare re-run preserves the ports chosen at first setup (read back from the
# profile/compose files), so it never reverts an 8081/5436 install to 8080/5432.
# ADMIN_EMAIL defaults to `git config user.email`.
setup-local ANTHROPIC_KEY="" OPENAI_KEY="" GEMINI_KEY="" HTTP_PORT="8080" PG_PORT="5432" ADMIN_EMAIL="":
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{justfile_directory()}}"
    PROFILE_DIR="$ROOT/.systemprompt/profiles/local"
    DOCKER_DIR="$ROOT/.systemprompt/docker"
    ANTHROPIC_KEY="{{ANTHROPIC_KEY}}"
    OPENAI_KEY="{{OPENAI_KEY}}"
    GEMINI_KEY="{{GEMINI_KEY}}"
    HTTP_PORT="{{HTTP_PORT}}"
    PG_PORT="{{PG_PORT}}"
    # Why: `admin setup` requires --admin-email (core 0.42.0 made it mandatory:
    # it names the platform admin on sign-in and consent screens). Both the
    # setup call below and `admin bootstrap` at the end take this same value, so
    # a fresh clone never stops to ask.
    ADMIN_EMAIL="{{ADMIN_EMAIL}}"
    if [ -z "$ADMIN_EMAIL" ]; then
        ADMIN_EMAIL="$(git config user.email 2>/dev/null || true)"
    fi
    if [ -z "$ADMIN_EMAIL" ]; then
        ADMIN_EMAIL="admin@localhost.localdomain"
    fi
    export SYSTEMPROMPT_PROFILE="$PROFILE_DIR/profile.yaml"
    # Default ports on a re-run mean "keep what I had", not "move me back to
    # 8080/5432": read the original choice back from the files setup wrote.
    if [ "$HTTP_PORT" = "8080" ] && [ -f "$PROFILE_DIR/profile.yaml" ]; then
        SAVED_HTTP="$(sed -n 's/^ *port: //p' "$PROFILE_DIR/profile.yaml" | head -1)"
        if [ -n "$SAVED_HTTP" ]; then
            HTTP_PORT="$SAVED_HTTP"
        fi
    fi
    if [ "$PG_PORT" = "5432" ] && [ -f "$DOCKER_DIR/local.yaml" ]; then
        SAVED_PG="$(sed -n 's/.*"\([0-9][0-9]*\):5432".*/\1/p' "$DOCKER_DIR/local.yaml" | head -1)"
        if [ -n "$SAVED_PG" ]; then
            PG_PORT="$SAVED_PG"
        fi
    fi
    # Whether a key was passed as a positional arg. When none is and there is
    # nothing to preserve, generation still needs a provider: on a TTY we let
    # `admin setup` drive its own "Select your AI provider" menu (the CLI owns
    # the prompt); off a TTY we cannot prompt, so keys must come as args. A
    # developer who keeps .systemprompt/ across reclones re-runs with no args
    # and is never asked again (the profile.yaml guard below skips generation).
    HAS_KEY=false
    if [ -n "$ANTHROPIC_KEY" ] || [ -n "$OPENAI_KEY" ] || [ -n "$GEMINI_KEY" ]; then
        HAS_KEY=true
    fi
    if [ "$HAS_KEY" = false ] && [ ! -f "$PROFILE_DIR/secrets.json" ] && [ ! -t 0 ]; then
        echo ""
        echo "================================================================"
        echo "  setup-local needs an AI provider API key"
        echo "================================================================"
        echo ""
        echo "  Not running on a TTY, so the provider menu can't be shown."
        echo "  Pass a key as an argument (one of Anthropic, OpenAI, Gemini):"
        echo "    just setup-local <anthropic_key> [openai_key] [gemini_key]"
        echo ""
        exit 1
    fi
    if [ ! -x target/debug/systemprompt ] && [ ! -x target/release/systemprompt ]; then
        # No toolchain needed when a release exists for this version: the
        # server + MCP binaries come from the GitHub Release tarball. Any
        # failure there (no release yet, no gh auth, macOS) falls through to
        # the build; SETUP_LOCAL_BUILD=1 skips the download outright.
        if [ "${SETUP_LOCAL_BUILD:-0}" != "1" ] && just fetch-release; then
            echo "Using the released binaries; skipping the build."
        else
        echo "Building debug binary..."
        # Why: offline. The database this just created is empty, and the online
        # build would verify every sqlx query against it — including
        # knowledge-bank's, whose tables the binary creates for itself at its
        # first boot. The committed .sqlx/ cache is the source of truth at
        # first run, so a clean clone does not need a populated database to
        # bootstrap.
        SQLX_OFFLINE=true just build
        fi
    fi
    # Resolve the binary at runtime: the {{CLI}} variable is evaluated by `just`
    # at parse time, so on a cold clone (no binary yet) it expands to an error
    # stub — useless for the bootstrap/keygen calls below, which run only after
    # the build above has produced the binary.
    if [ -x target/release/systemprompt ]; then
        BIN="$ROOT/target/release/systemprompt"
    else
        BIN="$ROOT/target/debug/systemprompt"
    fi
    mkdir -p "$PROFILE_DIR" "$DOCKER_DIR"
    # Rewrite the compose file when it exists but pins a different host port.
    # Guarding only on existence meant a re-run with a new PG_PORT kept the old
    # mapping, brought Postgres up on the old port, and then waited 60s for the
    # new one before dying on "Postgres did not become ready" — which names the
    # symptom and hides the cause.
    if [ -f "$DOCKER_DIR/local.yaml" ] \
        && ! grep -q "\"${PG_PORT}:5432\"" "$DOCKER_DIR/local.yaml"; then
        echo "Docker compose pins a different host port; rewriting for $PG_PORT."
        echo "Recreating the container so the new mapping takes effect..."
        docker compose -p "$(just _project_name local)" -f "$DOCKER_DIR/local.yaml" down 2>/dev/null || true
        rm -f "$DOCKER_DIR/local.yaml"
    fi
    if [ ! -f "$DOCKER_DIR/local.yaml" ]; then
        echo "Writing Docker compose for local Postgres (host port $PG_PORT)..."
        cat > "$DOCKER_DIR/local.yaml" <<YAML
    services:
      postgres:
        image: pgvector/pgvector:pg18
        restart: unless-stopped
        environment:
          POSTGRES_USER: systemprompt
          POSTGRES_PASSWORD: 123
          POSTGRES_DB: systemprompt
        ports:
          - "${PG_PORT}:5432"
        volumes:
          - postgres_data:/var/lib/postgresql
        healthcheck:
          test: ["CMD-SHELL", "pg_isready -U systemprompt -d systemprompt"]
          interval: 5s
          timeout: 5s
          retries: 5
    volumes:
      postgres_data: {}
    YAML
    fi
    echo "Starting local Postgres via Docker..."
    just db-up local
    echo "Waiting for Postgres to accept connections on localhost:${PG_PORT}..."
    for i in $(seq 1 60); do
        if (exec 3<>/dev/tcp/127.0.0.1/${PG_PORT}) 2>/dev/null; then
            exec 3<&- 3>&-
            # Also confirm the server actually answers pg_isready, not just a half-open socket.
            CONTAINER=$(docker compose -p "$(just _project_name local)" -f .systemprompt/docker/local.yaml ps -q postgres)
            if [ -n "$CONTAINER" ] && docker exec "$CONTAINER" pg_isready -U systemprompt -d systemprompt >/dev/null 2>&1; then
                echo "Postgres is ready."
                break
            fi
        fi
        if [ "$i" = "60" ]; then
            echo "ERROR: Postgres did not become ready within 60s." >&2
            exit 1
        fi
        sleep 1
    done
    if [ ! -f "$PROFILE_DIR/profile.yaml" ]; then
        echo "Generating profile + provider registry + secrets via 'admin setup'..."
        if [ "$HAS_KEY" = true ]; then
            # Keys supplied as args: fully non-interactive. The default provider
            # is the first key given, so the generated config (the providers
            # registry, gateway default, ai/config.yaml) is consistent with the
            # single key.
            KEY_ARGS=()
            DEFAULT_PROVIDER=""
            if [ -n "$ANTHROPIC_KEY" ]; then KEY_ARGS+=(--anthropic-key "$ANTHROPIC_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=anthropic; fi
            if [ -n "$OPENAI_KEY" ]; then KEY_ARGS+=(--openai-key "$OPENAI_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=openai; fi
            if [ -n "$GEMINI_KEY" ]; then KEY_ARGS+=(--gemini-key "$GEMINI_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=gemini; fi
            "$BIN" admin setup --yes --no-migrate --environment local \
                --db-host localhost --db-port "$PG_PORT" \
                --db-user systemprompt --db-password 123 --db-name systemprompt \
                --admin-email "$ADMIN_EMAIL" \
                --default-provider "$DEFAULT_PROVIDER" \
                "${KEY_ARGS[@]}"
        else
            # No key arg: let the CLI prompt for which provider to use. DB,
            # environment, and migrations stay non-interactive (flags + env);
            # only the provider selection is interactive, and the chosen
            # provider becomes the default.
            SYSTEMPROMPT_NON_INTERACTIVE=1 "$BIN" admin setup --no-migrate --environment local \
                --db-host localhost --db-port "$PG_PORT" \
                --db-user systemprompt --db-password 123 --db-name systemprompt \
                --admin-email "$ADMIN_EMAIL"
        fi
        if [ "$HTTP_PORT" != "8080" ]; then
            "$BIN" admin config server set --port "$HTTP_PORT" \
                --api-server-url "http://localhost:${HTTP_PORT}" \
                --api-internal-url "http://localhost:${HTTP_PORT}" \
                --api-external-url "http://localhost:${HTTP_PORT}"
            # The authz hook URL is an absolute webhook target baked at
            # `admin setup` time on the default port; re-point it at the
            # chosen port so the gateway's govern callback reaches this server.
            "$BIN" admin config governance set --mode webhook \
                --url "http://localhost:${HTTP_PORT}/api/public/govern/authz"
            # jwt_issuer is baked on the default port too, and it is not
            # cosmetic: it is the base a client resolves
            # `{iss}/.well-known/jwks.json` against. Left at :8080 on a
            # non-default port, every external verifier (the bridge, Claude
            # Code) fetches the signing keys of whatever else is on :8080 —
            # or nothing — and rejects this server's tokens as minted under an
            # unknown authority.
            "$BIN" admin config security set \
                --jwt-issuer "http://localhost:${HTTP_PORT}"
            # Same for CORS: the seeded origins name :8080, so the admin UI
            # served from the chosen port is refused by its own API.
            "$BIN" admin config server cors add "http://localhost:${HTTP_PORT}" || true
            "$BIN" admin config server cors add "http://127.0.0.1:${HTTP_PORT}" || true
            "$BIN" admin config server cors remove "http://localhost:8080" || true
            "$BIN" admin config server cors remove "http://127.0.0.1:8080" || true
        fi
        # The generator binds 127.0.0.1, which no container can route to.
        # Containerized clients (`just claude`, clean-client) need the gateway
        # on 0.0.0.0 from the first start, not after a start-time remediation.
        "$BIN" admin config server set --host 0.0.0.0
    elif [ "$HAS_KEY" = true ]; then
        # Profile generation is one-shot, guarded on profile.yaml. `just db-down`
        # drops the database but leaves the profile, so a re-run with different
        # keys would silently keep the old provider registry. Say so loudly and
        # point at the one command that actually re-provisions.
        echo ""
        echo "================================================================"
        echo "  Existing profile reused — supplied keys were NOT applied"
        echo "================================================================"
        echo ""
        echo "  $PROFILE_DIR/profile.yaml already exists, so 'admin setup' was"
        echo "  skipped and the provider registry/keys are unchanged."
        echo "  To re-provision from the keys you just passed:"
        echo ""
        echo "    rm -rf \"$PROFILE_DIR\" && just setup-local <keys...> $HTTP_PORT $PG_PORT"
        echo ""
    fi
    mkdir -p "$ROOT/web/dist"
    echo "Building binaries (release, full workspace)..."
    # Why: offline, for the same reason as the debug build above — the live
    # database has no knowledge-bank tables until the knowledge-bank binary
    # first boots, so the online sqlx check cannot pass on a fresh install.
    # `just migrate` below still applies every registered migration; this only
    # decides where the compile-time query check reads its schema from.
    SQLX_OFFLINE=true just build --release
    echo "Running database migrations..."
    just migrate
    echo "Ensuring bootstrap admin user ($ADMIN_EMAIL)..."
    "$BIN" admin bootstrap --email "$ADMIN_EMAIL"
    # Why: `admin setup` stores the identity signing key in secrets.json and
    # the server prefers that over signing_key_path, so a key generated here
    # would be one the server never verifies with (the e2e suite signs with
    # this file and got InvalidSignature). Materialise the secret's key as the
    # file; generate one only when the profile predates identity secrets.
    if [ ! -f "$ROOT/signing_key.pem" ]; then
        if python3 -c 'import base64,json,sys; p=json.load(open(sys.argv[1])).get("signing_key_pem"); sys.exit(1) if not p else open(sys.argv[2],"w").write(base64.b64decode(p).decode())' "$PROFILE_DIR/secrets.json" "$ROOT/signing_key.pem"; then
            echo "Wrote signing_key.pem from the profile's identity secret."
        else
            echo "Generating JWT signing key..."
            "$BIN" admin keys generate --output "$ROOT/signing_key.pem"
        fi
    fi
    echo "Publishing assets..."
    just publish
    echo ""
    echo "Local setup complete. Run: just start"

# List all tenants
tenants:
    {{CLI}} cloud tenant list

# Profile operations (interactive menu)
profile:
    {{CLI}} cloud profile

# List all profiles
profiles:
    {{CLI}} cloud profile list

# ══════════════════════════════════════════════════════════════════════════════
# SYNC
# ══════════════════════════════════════════════════════════════════════════════

# Content and skills are ingested from services/ at server startup and by
# `just publish` (publish_pipeline job); there is no separate local sync command.

# Core 0.29.0 removed `cloud sync`. Pushing is `just deploy` (cloud deploy),
# and pulling is `cloud backup`, which downloads the tenant's runtime services/
# tree. The old sync-push / sync-pull recipes called a command that no longer
# exists, so they are gone rather than aliased to something they never were.

# Download the tenant's runtime services/ tree (--list to inspect first)
backup *ARGS:
    {{CLI}} cloud backup "$@"

# ══════════════════════════════════════════════════════════════════════════════
# DEPLOY
# ══════════════════════════════════════════════════════════════════════════════

# Build everything and deploy to cloud — one command, no preceding build step.
# Note: publish_pipeline runs automatically on server startup with correct profile URLs
# Pinned to the `production` profile so a deploy never follows whichever profile
# the CLI session happens to be switched to.
deploy *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    for flag in "$@"; do
        if [ "$flag" = "--check" ]; then
            exec {{CLI}} cloud deploy --profile {{DEPLOY_PROFILE}} "$@"
        fi
    done
    {{just_executable()}} deploy-check
    {{just_executable()}} build-all
    exec {{CLI_RELEASE}} cloud deploy --profile {{DEPLOY_PROFILE}} "$@"

# Copy the sibling core checkout into .vendor/ (gitignored, never pushed) so
# the active [patch.crates-io] paths resolve both locally and inside the
# Docker build context — `COPY . /src` cannot see ../systemprompt-core.
# Re-run after every core edit; `deploy` depends on it while the patch is on.
vendor-core:
    #!/usr/bin/env bash
    set -euo pipefail
    # Why: vendoring is only meaningful while [patch.crates-io] is active. With
    # the patch commented, .vendor resolves nothing, so a vendor+deploy would
    # quietly ship a registry build while looking like it shipped the sibling —
    # or, worse, be read as pinning a core that was never pinned. Refuse instead.
    if ! grep -qE '^\[patch\.crates-io\]' Cargo.toml; then
        echo "vendor-core: [patch.crates-io] is commented — this branch builds against the" >&2
        echo "  published crates. Uncomment the patch blocks in Cargo.toml AND tests/Cargo.toml" >&2
        echo "  before vendoring, or use 'just deploy' which needs no vendoring." >&2
        exit 1
    fi
    src="${CORE_REPO:-../systemprompt-core}"
    [ -d "$src" ] || { echo "vendor-core: no core checkout at $src" >&2; exit 1; }
    mkdir -p .vendor
    rsync -a --delete \
        --exclude '.git' --exclude 'target' --exclude 'node_modules' \
        --exclude 'crates/tests/target' \
        "$src"/ .vendor/systemprompt-core/
    echo "vendored $src -> .vendor/systemprompt-core ($(git -C "$src" rev-parse --short HEAD 2>/dev/null || echo unknown))"

# Deploy the patch-active `next` branch: vendor the core checkout first so the
# image build compiles against it. Only for an active [patch.crates-io];
# `vendor-core` refuses otherwise, so plain `just deploy` is the released path.
deploy-next *FLAGS: vendor-core (deploy FLAGS)

# ── Knowledge database (dedicated Fly Postgres with pgvector) ────────────────
# The knowledge bank's index lives on its own Fly Postgres app: the shared
# systemprompt-db-prod cluster has no `vector` extension and the tenant role
# cannot CREATE DATABASE. Fly Postgres is not Fly-Launch managed, so these are
# plain flyctl steps; procedure and fallback image: deploy/fly/knowledge-db/.
KNOWLEDGE_DB_APP := "sp-a2f658d8bc5f-knowledge"
KNOWLEDGE_DB_REGION := "fra"
# Every recipe below reaches the database through `flyctl proxy` + local psql
# (deploy/fly/knowledge-db/psql.sh): `flyctl postgres connect` runs psql on
# the Fly machine and cannot see local SQL files.
KNOWLEDGE_DB_PSQL := "KNOWLEDGE_DB_APP=" + KNOWLEDGE_DB_APP + " deploy/fly/knowledge-db/psql.sh"

# Create the knowledge Postgres app (once). FLY_ORG defaults to the app's org.
# Keep the `postgres` password it prints: provisioning needs it as
# KNOWLEDGE_DB_SUPERUSER_PASSWORD.
knowledge-db-create:
    #!/usr/bin/env bash
    set -euo pipefail
    org="${FLY_ORG:-$(flyctl apps list --json | python3 -c 'import json,sys; print(next(a["Organization"]["Slug"] for a in json.load(sys.stdin) if a["Name"]=="sp-a2f658d8bc5f"))')}"
    flyctl postgres create --name {{KNOWLEDGE_DB_APP}} --region {{KNOWLEDGE_DB_REGION}} \
        --initial-cluster-size 1 --vm-size shared-cpu-1x --volume-size 10 --org "$org"
    echo "Now: KNOWLEDGE_DB_SUPERUSER_PASSWORD=<printed above> KNOWLEDGE_DB_PASSWORD=<choose> just knowledge-db-provision"

# Create the `knowledge` role + database and CREATE EXTENSION vector.
# Needs KNOWLEDGE_DB_SUPERUSER_PASSWORD and KNOWLEDGE_DB_PASSWORD; fails loudly
# if the image has no pgvector (then deploy deploy/fly/knowledge-db/Dockerfile
# and re-run).
knowledge-db-provision:
    #!/usr/bin/env bash
    set -euo pipefail
    : "${KNOWLEDGE_DB_PASSWORD:?set KNOWLEDGE_DB_PASSWORD to the password for role knowledge}"
    {{KNOWLEDGE_DB_PSQL}} --admin -v pw="$KNOWLEDGE_DB_PASSWORD" -f deploy/fly/knowledge-db/provision.sql
    echo "Now set the secret on the production profile:"
    echo "  systemprompt admin config secret set knowledge_database_url \\"
    echo "    'postgres://knowledge:<pw>@{{KNOWLEDGE_DB_APP}}.flycast:5432/knowledge' --profile {{DEPLOY_PROFILE}}"

# Interactive psql on the knowledge database (needs KNOWLEDGE_DB_PASSWORD)
knowledge-db-psql:
    {{KNOWLEDGE_DB_PSQL}}

# Size, documents, chunks, grants, syncing users, cache rows, per-user syncs
knowledge-db-report:
    {{KNOWLEDGE_DB_PSQL}} -f deploy/fly/knowledge-db/report.sql

# The prune_knowledge SQL against the Fly database. Dry run by default;
# DRY_RUN=0 commits. STALE_DAYS bounds both stale users and cache age.
knowledge-db-prune STALE_DAYS="90" DRY_RUN="1":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{DRY_RUN}}" = "0" ]; then tail="COMMIT;"; else tail="ROLLBACK;"; fi
    { cat deploy/fly/knowledge-db/prune.sql; echo "$tail"; } | \
        {{KNOWLEDGE_DB_PSQL}} -v stale_days={{STALE_DAYS}} -f -
    [ "$tail" = "COMMIT;" ] && echo "pruned" || echo "dry run (rolled back); DRY_RUN=0 to apply"

# Seed the knowledge bank of a running local stack (`just start`, `just
# e2e-seed` for the e2e admin): drives the MCP server over streamable HTTP as
# the e2e admin — sync_knowledge, then polls list_knowledge_sources until the
# fixture categories under services/knowledge-bank/fixtures/ report documents.
# Free with the shipped `local` model provider; not part of the default e2e seed.
knowledge-seed:
    cd playwright && npx tsx setup/seed/knowledge.ts

# Seed the clearly labelled `[KB DEMO]` workspace in Atlassian (EAICOMM space
# and project) THROUGH the hosted Atlassian MCP as the connected account of
# the user whose PAT is in KB_E2E_PAT — never with an Atlassian credential of
# ours. Idempotent: a rerun finds the parent page and reuses it. Needs `just
# start` and a grant carrying the connector's write scopes.
knowledge-demo:
    cd playwright && KB_E2E_PAT="${KB_E2E_PAT:?set KB_E2E_PAT to a PAT of a user with Atlassian connected}" npx tsx setup/seed/atlassian-demo.ts

# The knowledge bank end to end over the connected Atlassian account: seed
# the demo workspace, sync, search, grants, degradation for an unlinked user,
# audit rows. Needs `just start`, `just e2e-seed` (for the unlinked principal)
# and KB_E2E_PAT.
e2e-knowledge:
    cd playwright && KB_E2E_PAT="${KB_E2E_PAT:?set KB_E2E_PAT to a PAT of a user with Atlassian connected}" npx playwright test tests/knowledge-atlassian-e2e.spec.ts

# Pre-deploy preflight only — no build, no push
deploy-check:
    {{CLI}} cloud doctor --profile {{DEPLOY_PROFILE}} --distributed

# Check deployment status
status:
    {{CLI}} cloud status --profile {{DEPLOY_PROFILE}}

# ── Profile-aware deploys (any target, not just our pinned production) ───────
# `deploy` stays pinned to production on purpose. Operators deploying the same
# repo to their own infra (e.g. an Oracle VM with a remote Postgres) work with
# their own profile dir under .systemprompt/profiles/<name>/ and use these.

# Validate any profile without a running server: files, secrets keys, URLs,
# database reachability from this machine, and (with a built CLI) that the
# profile deserializes — the loader rejects unknown YAML keys. Add --live to
# also probe <api_external_url>/api/v1/health.
profile-check PROFILE="local" *FLAGS:
    @bash scripts/profile-check.sh {{PROFILE}} {{FLAGS}}

# Deploy to a cloud tenant under a non-default profile. Self-host targets do
# not cloud-deploy — use `just bundle` instead.
deploy-to PROFILE *FLAGS: build-all
    {{CLI_RELEASE}} cloud deploy --profile {{PROFILE}} {{FLAGS}}

# Package everything a self-host deployment needs (the same manifest the
# production Dockerfile ships) into dist/systemprompt-<profile>.tar.gz,
# including an UNPACK.md with the first-run steps (migrate, serve, probe).
bundle PROFILE:
    @bash scripts/bundle-profile.sh {{PROFILE}}

# Run the server in the foreground under any profile (self-host style).
serve PROFILE="local":
    SYSTEMPROMPT_PROFILE=.systemprompt/profiles/{{PROFILE}}/profile.yaml {{CLI}} infra services serve --foreground

# ══════════════════════════════════════════════════════════════════════════════
# MCP & BUILD ALL
# ══════════════════════════════════════════════════════════════════════════════

# Build all MCP servers (reads from manifest.yaml files) — single-flight and
# fingerprint-skipped, so a tree whose MCP servers already built returns
# immediately instead of re-paying the per-package rebuild.
build-mcp:
    @scripts/build-coordinator.sh run build-mcp "" -- {{just_executable()}} _build-mcp-uncoordinated

_build-mcp-uncoordinated:
    DATABASE_URL="$(just _db-url)" {{CLI}} build mcp --release

# The workspace build includes both MCP binaries. publish_pipeline includes
# bundle_admin_css and copy_extension_assets; run each producer once.
build-all:
    just build --release
    just bridge-package-host
    just downloads-fetch
    {{CLI_RELEASE}} infra jobs run publish_pipeline
    @echo "All components built"

# ══════════════════════════════════════════════════════════════════════════════
# WEB ASSETS & PUBLISHING
# ══════════════════════════════════════════════════════════════════════════════

# Copy web assets to dist (CSS, JS, images)
web-assets:
    {{CLI}} infra jobs run copy_extension_assets

# Publish: compile templates, bundle CSS/JS, copy assets, prerender content
publish:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    {{CLI}} infra jobs run publish_pipeline

# Stage setup-skill dashboard assets (skill assets ride into the Cowork VM)
cowork-setup-assets *ARGS:
    scripts/generate-cowork-setup-assets.py {{ARGS}}

# Build web assets only (templates + CSS + JS + copy to dist)
web-build:
    {{CLI}} infra jobs run bundle_admin_css
    {{CLI}} infra jobs run copy_extension_assets

# ══════════════════════════════════════════════════════════════════════════════
# DOCKER
# ══════════════════════════════════════════════════════════════════════════════

# Build the gateway image locally, named as CI publishes it. CI stages the
# bridge binaries into storage/files/downloads/ first; do the same
# (`just bridge-package-host`, `just downloads-fetch`) or the image 404s them.
docker-build TAG="local":
    docker build -f Dockerfile --build-arg SYSTEMPROMPT_VERSION={{TAG}} -t {{IMAGE}}:{{TAG}} .

# Run a local or published image against .env
docker-run TAG="local":
    docker run -p 8080:8080 --env-file .env {{IMAGE}}:{{TAG}}

# The colleague install: pull the published image and start (docs/install/docker.md).
compose-up:
    docker compose pull && docker compose up -d

# Build the branded bridge. Its own standalone workspace, NOT the server
# workspace — `just build` does not touch it, and a bare `cargo build` from the
# repo root silently builds the server instead.
bridge-build *ARGS: core-checkout
    cd {{justfile_directory()}}/bridge && cargo build --release {{ARGS}}

# The client depends on systemprompt-bridge by relative path, and that crate is
# not published — so unlike the server, building the client needs the core
# repository checked out beside this one. Clones it when absent, fast-forwards
# it when present, and leaves local work alone.
core-checkout:
    #!/usr/bin/env bash
    set -euo pipefail
    CORE="{{justfile_directory()}}/../systemprompt-core"
    if [ -d "$CORE/.git" ]; then
        if [ -n "$(git -C "$CORE" status --porcelain)" ]; then
            echo "core checkout has local changes — leaving it as it is."
        else
            echo "Using existing core checkout at $(git -C "$CORE" rev-parse --short HEAD)"
            # Updating here can mix a vendored server with a newer bridge mid-deploy.
            expected="$(tr -d '\r\n' < "{{justfile_directory()}}/bridge/CORE_REF")"
            [ "$(git -C "$CORE" rev-parse HEAD)" = "$(git -C "$CORE" rev-parse "$expected^{commit}")" ] || {
                echo "core checkout differs from bridge/CORE_REF; update and vendor/pin core together before building." >&2
                exit 1
            }
        fi
    else
        echo "Cloning systemprompt-core beside this repo (the client needs it)."
        git clone --quiet https://github.com/systempromptio/systemprompt-core "$CORE"
    fi

# Package the branded bridge as a Linux release tarball into dist/
# Coordinated: bridge/ and the core sibling are both in the fingerprint, so a
# failed deploy retried on the same tree skips straight past this step.
bridge-package-linux:
    @scripts/build-coordinator.sh run bridge-package "" -- scripts/package-bridge-linux.sh

# Cross-compile the Windows bridge exe (x86_64-pc-windows-msvc via cargo-xwin —
# msvc is required: it statically links WebView2Loader, a -gnu build ships a
# bare exe that dies at start on "WebView2Loader.dll was not found") and stage
# it into storage/files/downloads/. Follow with `just publish`.
bridge-package-windows: core-checkout
    @scripts/build-coordinator.sh run bridge-package-windows "" -- scripts/package-bridge-windows.sh

# Build, sign, notarize, and stage the macOS bridge DMG (universal arm64 +
# x86_64) into storage/files/downloads/. macOS only — needs a "Developer ID
# Application" cert in the keychain plus a notarytool credential profile; the
# script header has the one-time store-credentials command. Smoke builds with
# UNSIGNED=1 or SKIP_NOTARIZE=1 never stage assets and exit 2 (not shippable).
[macos]
bridge-package-macos: core-checkout
    @scripts/build-coordinator.sh run bridge-package-macos "" -- scripts/package-bridge-macos.sh
    @bash scripts/verify-bridge-macos.sh storage/files/downloads/astound-bridge-macos.dmg

# Backfill storage/files/downloads/ from the deployed host. The dir is
# gitignored and each platform's bridge binary can only be built on that
# platform's toolchain, so before a deploy every asset another machine produced
# must be fetched back — otherwise the freshly baked image 404s download links
# the site already serves. Local files always win over the remote copy.
#
# An asset that exists neither locally nor remotely WARNS and the deploy
# proceeds: no host can build every platform, so a hard failure here blocked
# every Linux deploy on the macOS DMG. `DOWNLOADS_STRICT=1 just deploy` makes
# it fatal again — do that when cutting a release.
downloads-fetch:
    @scripts/fetch-remote-downloads.sh

# Install the compiled server + MCP binaries from a GitHub Release instead of
# building them: `systemprompt-astound-server-<version>-linux-<arch>.tar.gz`
# holds bin/systemprompt and every bin/systemprompt-mcp-* the manifests name,
# built by the same `cargo build --release --workspace` as the image. The
# binaries land in target/release/, which is where `just start`, `setup-local`
# and the MCP validator already look — so a clone never needs a Rust toolchain.
# Linux only (the tarball is built in the Linux image); verified against the
# release's SHA256SUMS. Default version = the workspace version in Cargo.toml.
fetch-release VERSION="":
    #!/usr/bin/env bash
    set -euo pipefail
    v="{{VERSION}}"
    [ -n "$v" ] || v=$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' Cargo.toml | head -1)
    [ "$(uname -s)" = "Linux" ] || { echo "fetch-release: the server tarball is Linux-only; on $(uname -s) use the image (docs/install/docker.md) or 'just build --release'." >&2; exit 1; }
    case "$(uname -m)" in x86_64) arch=amd64 ;; aarch64|arm64) arch=arm64 ;; *) echo "fetch-release: no tarball for $(uname -m)" >&2; exit 1 ;; esac
    name="systemprompt-astound-server-$v-linux-$arch.tar.gz"
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
    echo "==> downloading $name from release v$v"
    gh release download "v$v" -p "$name" -p SHA256SUMS -D "$tmp" \
        || { echo "fetch-release: release v$v has no $name (gh auth, or the release does not exist yet)." >&2; exit 1; }
    (cd "$tmp" && grep " $name\$" SHA256SUMS | sha256sum -c -)
    tar xzf "$tmp/$name" -C "$tmp"
    mkdir -p target/release
    install -m 0755 "$tmp/${name%.tar.gz}"/bin/* target/release/
    echo "==> installed into target/release/:"; ls -1 target/release/systemprompt target/release/systemprompt-mcp-* | sed 's/^/    /'
    target/release/systemprompt --version

# Package whatever bridge this host's toolchain can build (build-all calls
# this; downloads-fetch then backfills the platforms this host cannot build).
[linux]
bridge-package-host: bridge-package-linux

[macos]
bridge-package-host: bridge-package-macos

[windows]
bridge-package-host:
    @echo "warn: no bridge packaging runs on Windows hosts - relying on downloads-fetch."

# Installs the client if it is not there yet; re-running it with a fresh code
# re-binds the machine to whoever that code belongs to.
# Point Claude Code on THIS host at the gateway (CODE comes from /admin/profile)
connect CODE GATEWAY="http://localhost:8080":
    curl -fsSL {{GATEWAY}}/files/downloads/install.sh | sh -s -- \
        --download-base {{GATEWAY}}/files/downloads --code {{CODE}}

# A code is needed the first time only: the credential it is exchanged for
# persists in a per-gateway volume, so later runs are just `just
# claude`. `just claude-reset` signs out and makes a code necessary again.
# Claude Code, connected, in a container (CODE from /admin/profile, first run only)
claude CODE="" GATEWAY="http://localhost:8080":
    #!/usr/bin/env bash
    set -euo pipefail

    # The page prints the gateway as a browser sees it. Inside a container
    # localhost is the container, so rewrite it to the host alias. Keep the
    # original too: reaching it from the host is what separates "the gateway is
    # down" from "the gateway is up but the container cannot route to it".
    HOST_GATEWAY="{{GATEWAY}}"
    GATEWAY="{{GATEWAY}}"
    GATEWAY="${GATEWAY//localhost/host.docker.internal}"
    GATEWAY="${GATEWAY//127.0.0.1/host.docker.internal}"

    # Scope the session to this clone AND its gateway, the same way the Docker
    # Postgres project name is scoped, so several checkouts coexist.
    #
    # Two things force it. A credential is only valid for the gateway that
    # issued it, so sharing one home makes a second gateway look like a broken
    # sign-in: the PAT is found, whoami fails against the wrong host, and
    # bootstrap drops to asking for a code. And a fixed container name would
    # make a second clone attach to the first clone's session instead of
    # starting its own.
    REPO_SLUG="$(basename "{{justfile_directory()}}" | sed 's|[^A-Za-z0-9]|-|g')"
    REPO_HASH="$(printf '%s' "{{justfile_directory()}}" | sha256sum | cut -c1-8)"
    GW_SLUG="$(printf '%s' "$GATEWAY" | sed -e 's|^https\?://||' -e 's|[^A-Za-z0-9]|-|g')"
    SCOPE="${REPO_SLUG}-${REPO_HASH}-${GW_SLUG}"
    VOL="astound-claude-${SCOPE}"
    NAME="astound-claude-${SCOPE}"

    if [ "$(docker inspect -f '{{{{.State.Running}}}}' "$NAME" 2>/dev/null)" = "true" ]; then
        echo "Already running for this repo — opening another session in it."
        exec docker exec -it "$NAME" bash -lc claude
    fi
    docker rm -f "$NAME" >/dev/null 2>&1 || true

    if ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    # The client is a separate workspace and depends on systemprompt-bridge by
    # relative path into a sibling core checkout, so building it needs that
    # checkout present. `bridge-build` fetches it. Do this before the code is
    # spent: a first-run compile can outlive the code's 10-minute TTL, so
    # `just bridge-build` belongs in setup rather than here.
    BRIDGE="{{justfile_directory()}}/bridge/target/release/astound-bridge"
    if [ ! -f "$BRIDGE" ] || [ ! -x "$BRIDGE" ]; then
        echo "Client not built yet — fetching core and building it."
        echo "warn: a first build takes minutes and the code expires in 10." >&2
        echo "      If it is rejected as expired, issue a fresh one and re-run." >&2
        just bridge-build
    fi

    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi

    # Prove the container can reach the gateway BEFORE the code is spent —
    # otherwise an unroutable gateway surfaces as a connect error with the code
    # already consumed. --entrypoint is required: the image's entrypoint would
    # otherwise swallow this as its own arguments, print its banner, and exit 0,
    # so the check would pass without making a request.
    gateway_reachable() {
        docker run --rm --entrypoint curl \
            --add-host host.docker.internal:host-gateway \
            astound-clean-client:local \
            -sf --max-time 5 "$GATEWAY/health" >/dev/null 2>&1
    }

    if ! gateway_reachable; then
        # Separate the two causes before touching anything. If the gateway does
        # not answer on the host either, it is down or on another port, and
        # rebinding it would be treating the wrong illness — Docker Desktop and
        # WSL2 proxy host.docker.internal to host loopback, so a 127.0.0.1 bind
        # is genuinely reachable there and the bind is often not the problem.
        if ! curl -sf --max-time 5 "$HOST_GATEWAY/health" >/dev/null 2>&1; then
            echo "ERROR: the gateway is not answering at $HOST_GATEWAY" >&2
            echo "" >&2
            echo "  It is not reachable from this host either, so it is down or" >&2
            echo "  listening on another port — not a container routing problem." >&2
            echo "  Start it, then re-run:" >&2
            echo "" >&2
            echo "      just start" >&2
            echo "      just server-status" >&2
            echo "" >&2
            echo "  Your code has not been used." >&2
            exit 1
        fi

        # The gateway is up on the host but the container cannot route to it.
        # On native Linux Docker that is exactly what a loopback bind causes,
        # and widening it is the documented remedy. Any other bind is left
        # alone rather than guessed at.
        if [ -x target/release/systemprompt ]; then
            SP="{{ justfile_directory() }}/target/release/systemprompt"
        else
            SP="{{ justfile_directory() }}/target/debug/systemprompt"
        fi
        # `show` prints its labels on stderr and its values on stdout, so the
        # human-readable form cannot be parsed by name. The JSON artifact can.
        # A parse failure yields an empty host, which matches nothing below and
        # so declines to remediate rather than guessing.
        BOUND_HOST="$("$SP" --json admin config server show --profile local 2>/dev/null \
            | python3 -c "import json,sys; d=json.load(sys.stdin); print(next((s.get('content','') for s in d.get('sections',[]) if s.get('heading')=='host'),''))" \
            2>/dev/null || true)"

        if [ "$BOUND_HOST" = "127.0.0.1" ] || [ "$BOUND_HOST" = "localhost" ]; then
            echo "notice: the gateway is bound to $BOUND_HOST, which no container can" >&2
            echo "        route to. Rebinding it to 0.0.0.0 and restarting." >&2
            echo "        This widens the gateway to your LAN — revert with:" >&2
            echo "            systemprompt admin config server set --host 127.0.0.1" >&2
            # Two things about this restart. `restart` needs an explicit target
            # — a bare `restart` exits 1 with "Must specify target (api, agent,
            # mcp)" — and the bind only affects the API listener, so restarting
            # `api` leaves the MCP servers up. And it SERVES in the foreground
            # rather than returning, so it has to be detached or it would hang
            # here forever; readiness is established by polling below, not by
            # the command exiting.
            "$SP" admin config server set --host 0.0.0.0 --profile local >/dev/null
            nohup "$SP" infra services restart api --profile local \
                >/dev/null 2>&1 &

            READY=0
            for _ in $(seq 1 45); do
                if gateway_reachable; then READY=1; break; fi
                sleep 1
            done
            if [ "$READY" -eq 0 ]; then
                echo "warn: the API server did not come back within 45s." >&2
                echo "      Check it with: just server-status" >&2
            fi
        fi
    fi

    if ! gateway_reachable; then
        echo "ERROR: the container cannot reach the gateway at $GATEWAY" >&2
        echo "" >&2
        echo "  The gateway answers on this host, so it is running — but the" >&2
        echo "  container cannot route to it, and it is bound to" >&2
        echo "  ${BOUND_HOST:-an unspecified host}, which rebinding did not fix." >&2
        echo "  Something between the two is in the way: a firewall on the" >&2
        echo "  docker bridge, or a docker network without host-gateway." >&2
        echo "" >&2
        echo "      just server-status" >&2
        echo "      curl -sf $HOST_GATEWAY/health      # from this host" >&2
        echo "" >&2
        echo "  Your code has not been used. Re-run this once the container can" >&2
        echo "  reach the gateway." >&2
        exit 1
    fi

    # No code on a repeat run: the stored credential is reused, and bootstrap
    # only falls back to asking when there is neither. Passing an empty value
    # would look like "a code was supplied" to that check.
    # Look for the credential itself, not merely the volume: a volume left over
    # from a run that never completed sign-in holds none, and bootstrap would
    # then drop to an interactive code prompt — which is what a non-interactive
    # caller sees as a hang.
    CODE_ENV=()
    if [ -n "{{CODE}}" ]; then
        CODE_ENV=(-e ASTOUND_BRIDGE_CODE="{{CODE}}")
    elif ! docker run --rm --entrypoint test \
            -v "$VOL":/home/tester \
            astound-clean-client:local \
            -f /home/tester/.config/astound/astound-bridge.pat >/dev/null 2>&1; then
        echo "ERROR: not signed in yet, and no code was given." >&2
        echo "" >&2
        echo "  A code is needed the first time. Take one from /admin/profile:" >&2
        echo "" >&2
        echo "      just claude <code>" >&2
        echo "" >&2
        echo "  Later runs need no code — the credential is kept." >&2
        exit 1
    fi

    exec docker run -it --rm \
        --name "$NAME" \
        --hostname "${REPO_SLUG:0:24}" \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="$GATEWAY" \
        "${CODE_ENV[@]}" \
        -e CLEAN_CLIENT_EXEC_CLAUDE=1 \
        -e CLEAN_CLIENT_ALLOW_STATE=1 \
        -v "$VOL":/home/tester \
        -v "$BRIDGE:/usr/local/bin/astound-bridge:ro" \
        -v "{{justfile_directory()}}/deploy/clean-client/bootstrap.sh:/usr/local/bin/bootstrap.sh:ro" \
        "${PORTS[@]}" \
        astound-clean-client:local /usr/local/bin/bootstrap.sh

# ──────────────────────────────────────────────────────────────────────────────
# CLEAN CLIENT — a config-free Linux box for testing the Claude Code + bridge
# integration. See deploy/clean-client/README.md.
# ──────────────────────────────────────────────────────────────────────────────

# Build the clean-client image (context is deploy/clean-client only — no repo state)
clean-client-build *ARGS:
    docker build {{ARGS}} -f deploy/clean-client/Dockerfile -t astound-clean-client:local deploy/clean-client

# Shell into a clean client. PERSIST=1 keeps the login across runs.
# GATEWAY overrides the gateway URL (default: this WSL host on :8080).
clean-client PERSIST="0" GATEWAY="http://host.docker.internal:8080":
    #!/usr/bin/env bash
    set -euo pipefail
    if ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    # Mount the bridge you just built, read-only. Absent is not fatal: you can
    # still test Claude Code against the gateway without the bridge installed.
    BRIDGE="{{justfile_directory()}}/bridge/target/release/astound-bridge"
    MOUNTS=()
    # -f as well as -x: a bare `docker run -v` against a missing path makes
    # docker create a root-owned DIRECTORY there, and `[ -x dir ]` is true, so
    # an -x-only test would happily mount the directory and report a bridge that
    # is not there.
    if [ -d "$BRIDGE" ]; then
        echo "ERROR: $BRIDGE is a directory (docker created it from a stale -v mount)." >&2
        echo "       Remove it with: sudo rmdir '$BRIDGE'" >&2
        exit 1
    fi
    if [ -f "$BRIDGE" ] && [ -x "$BRIDGE" ]; then
        MOUNTS+=(-v "$BRIDGE:/usr/local/bin/astound-bridge:ro")
    else
        echo "warn: $BRIDGE not found — run 'cd bridge && cargo build --release' for the full flow." >&2
    fi

    # PERSIST keeps ~/.config/astound and ~/.claude in a named volume so a PAT
    # survives a restart. Off by default: a throwaway HOME is the point.
    if [ "{{PERSIST}}" = "1" ]; then
        MOUNTS+=(-v astound-clean-home:/home/tester -e CLEAN_CLIENT_ALLOW_STATE=1)
        echo "State persists in volume 'astound-clean-home' — 'just clean-client-reset' wipes it."
    fi

    # 8767 is the bridge's plugin-OAuth loopback port; it must be reachable from
    # a Windows browser. It is NOT published if your primary distro already
    # holds it, since that bind would just fail.
    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it. Plugin OAuth loopback will not work." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi

    # Note what is deliberately NOT here: no --env-file, no $HOME mounts, no
    # repo mount. The container must start from nothing.
    exec docker run -it --rm \
        --name astound-clean-client \
        --hostname clean-client \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        "${MOUNTS[@]}" "${PORTS[@]}" \
        astound-clean-client:local

alias cc := clean-client-ready

# Clean client, signed in and ready: paste one code, then type `claude`.
clean-client-ready GATEWAY="http://host.docker.internal:8080":
    #!/usr/bin/env bash
    set -euo pipefail

    # Already running (another terminal has it): open a second shell in it
    # rather than failing on the name conflict.
    if [ "$(docker inspect -f '{{{{.State.Running}}}}' astound-clean-client 2>/dev/null)" = "true" ]; then
        echo "Container 'astound-clean-client' is already running — opening a shell in it."
        exec docker exec -it astound-clean-client bash -l
    fi

    # Stopped leftover (a crashed run, or one started without --rm): restart it
    # and shell in, so anything written outside the /home/tester volume survives.
    # Only if it refuses to start do we reclaim the name.
    if docker inspect astound-clean-client >/dev/null 2>&1; then
        echo "Container 'astound-clean-client' is stopped — restarting it."
        if docker start astound-clean-client >/dev/null 2>&1; then
            exec docker exec -it astound-clean-client bash -l
        fi
        echo "warn: it would not restart — removing it and starting fresh." >&2
        docker rm -f astound-clean-client >/dev/null
    fi

    if ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    BRIDGE="{{justfile_directory()}}/bridge/target/release/astound-bridge"
    if [ ! -f "$BRIDGE" ] || [ ! -x "$BRIDGE" ]; then
        echo "ERROR: $BRIDGE not found — run 'just bridge-build' first." >&2
        exit 1
    fi

    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi

    # State persists so a second run reuses the PAT instead of burning a code;
    # CLEAN_CLIENT_ALLOW_STATE tells the entrypoint that reuse is deliberate
    # here rather than host config leaking in.
    exec docker run -it --rm \
        --name astound-clean-client \
        --hostname clean-client \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        -e CLEAN_CLIENT_ALLOW_STATE=1 \
        -v astound-clean-home:/home/tester \
        -v "$BRIDGE:/usr/local/bin/astound-bridge:ro" \
        -v "{{justfile_directory()}}/deploy/clean-client/bootstrap.sh:/usr/local/bin/bootstrap.sh:ro" \
        "${PORTS[@]}" \
        astound-clean-client:local /usr/local/bin/bootstrap.sh

# End-to-end: run the published installer with a PAT and assert managed MCP (see script header for how to mint the PAT)
clean-client-install PAT GATEWAY="http://host.docker.internal:8080":
    GATEWAY="{{GATEWAY}}" scripts/clean-client-install.sh "{{PAT}}"

# The three client matrices. Each drives a real client through the gateway once
# per model /v1/models advertises, and asserts the pass against the server's own
# audit rows rather than the client's transcript — a transcript grep once went
# green while the model shelled out to `echo`. Pass --list to print the models
# without running anything; MODELS=a,b narrows the sweep.
#
# All three need a running server and an admin PAT:
#   systemprompt admin users api-key issue --user <uuid> --name e2e

# Claude Code agent turn, per model, proven server-side
e2e-claude-code *ARGS:
    scripts/e2e-claude-code.sh {{ARGS}}

# OpenCode agent turn, per model, proven server-side
e2e-opencode *ARGS:
    scripts/e2e-opencode.sh {{ARGS}}

# The entitlement matrix through real clients: every seeded bridge principal
# (playwright/.auth/pats.json, from `just e2e-seed`) enrolled on Claude Code and
# OpenCode in turn, host mode only (E2E_CLIENT=host — one loopback proxy port,
# so cells are serial under a throwaway HOME each). Per cell it asserts, as
# exact sets against playwright/setup/seed/entitlements.json: the marketplace
# dirs, installed_plugins keys and per-plugin .mcp.json servers (Claude Code) or
# the skill dirs, opencode.json mcp keys and sidecar (OpenCode); that the
# knowledge-bank tool is callable over the proxy for entitled principals and
# refused for the rest; and a real model turn calling search_knowledge, proven
# by ai_request_tool_calls + mcp_tool_executions for that user. --list prints
# the cells without a server; --principals a,b / --hosts h / --skip-model narrow
# the sweep; MODELS=... picks the model (default claude-sonnet-5). Needs
# `just knowledge-seed` for the sentinel document.
# Marketplace entitlements per principal × host, through real clients
e2e-marketplaces *ARGS:
    E2E_CLIENT="${E2E_CLIENT:-host}" scripts/e2e-marketplaces.sh {{ARGS}}

# Raw gateway wire behaviour, per model
e2e-wires *ARGS:
    scripts/e2e-wires.sh {{ARGS}}

# Drops this clone's sessions and the credentials they stored, so the next run
# redeems a fresh code. ALL=1 signs out every clone on this host.
# Sign out of `just claude` (this repo; ALL=1 for every repo)
claude-reset ALL="0":
    #!/usr/bin/env bash
    set -euo pipefail
    # Scoped to this clone by default: signing every other checkout out because
    # one of them wanted a clean slate is not what anyone means by "reset".
    if [ "{{ALL}}" = "1" ]; then
        FILTER='name=^astound-claude-'
        echo "Signing out every repo on this host."
    else
        REPO_SLUG="$(basename "{{justfile_directory()}}" | sed 's|[^A-Za-z0-9]|-|g')"
        REPO_HASH="$(printf '%s' "{{justfile_directory()}}" | sha256sum | cut -c1-8)"
        FILTER="name=^astound-claude-${REPO_SLUG}-${REPO_HASH}-"
    fi
    docker ps -aq --filter "$FILTER" | while read -r c; do
        docker rm -f "$c" >/dev/null 2>&1 && echo "removed container $c"
    done
    FOUND=0
    for v in $(docker volume ls -q --filter "$FILTER"); do
        docker volume rm "$v" >/dev/null 2>&1 && { echo "removed $v"; FOUND=1; }
    done
    [ "$FOUND" = "1" ] || echo "Nothing to sign out of."
    echo "'just claude <code>' will start from nothing."

# Wipe the persisted clean-client state volume
clean-client-reset:
    -docker rm -f astound-clean-client 2>/dev/null
    -docker volume rm astound-clean-home
    @echo "Clean-client state wiped."

# Isolated dev sandbox on a real project: clean client + the repo mounted at
# /workspace/project. HOME stays virgin (device-link auth as usual); only the
# project directory crosses into the container. The image ships Playwright +
# Chromium so the dev_test skill works against the mounted project.
dev-sandbox REPO PERSIST="0" GATEWAY="http://host.docker.internal:8080":
    #!/usr/bin/env bash
    set -euo pipefail
    REPO_ABS="$(readlink -f "{{REPO}}")"
    if [ ! -d "$REPO_ABS" ]; then
        echo "ERROR: {{REPO}} is not a directory" >&2
        exit 1
    fi
    if ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi
    BRIDGE="{{justfile_directory()}}/bridge/target/release/astound-bridge"
    MOUNTS=(-v "$REPO_ABS:/workspace/project")
    if [ -d "$BRIDGE" ]; then
        echo "ERROR: $BRIDGE is a directory (docker created it from a stale -v mount)." >&2
        echo "       Remove it with: sudo rmdir '$BRIDGE'" >&2
        exit 1
    fi
    if [ -f "$BRIDGE" ] && [ -x "$BRIDGE" ]; then
        MOUNTS+=(-v "$BRIDGE:/usr/local/bin/astound-bridge:ro")
    else
        echo "warn: $BRIDGE not found — run 'cd bridge && cargo build --release' for the full flow." >&2
    fi
    if [ "{{PERSIST}}" = "1" ]; then
        MOUNTS+=(-v astound-clean-home:/home/tester -e CLEAN_CLIENT_ALLOW_STATE=1)
        echo "State persists in volume 'astound-clean-home' — 'just clean-client-reset' wipes it."
    fi
    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it. Plugin OAuth loopback will not work." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi
    exec docker run -it --rm \
        --name astound-dev-sandbox \
        --hostname dev-sandbox \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        "${MOUNTS[@]}" "${PORTS[@]}" \
        astound-clean-client:local

# Prove a published release deploys the way a customer deploys it: pull +
# cosign-verify the GHCR image, verify every GitHub Release asset, stand up
# PostgreSQL 16 primary + replica, three gateway nodes behind a TLS load
# balancer from deploy/self-host/, and assert the lot (report lands in
# deploy/self-host/E2E-REPORT-<version>.md). Needs .e2e.env with
# ANTHROPIC_API_KEY + BEDROCK_API_KEY, docker, gh, and cosign on PATH.
e2e-release VERSION:
    scripts/e2e-published-release.sh {{VERSION}}

# Install the Playwright e2e suite's dependencies (playwright/ directory)
e2e-install:
    cd playwright && npm ci && npx playwright install chromium

# Run the Playwright e2e suite against a running gateway (GATEWAY_URL env
# overrides the default http://localhost:8080). Not part of `just validate` —
# it needs a live stack: `just start` first.
e2e *ARGS:
    bash scripts/check-spec-shape.sh
    bash scripts/check-req-tags.sh
    cd playwright && npx playwright test "$@"

# Seed deterministic e2e principals + analytics trail (idempotent; touches only
# e2e-*/@e2e.local rows). `--reset` deletes and re-creates exactly those rows.
e2e-seed *ARGS:
    cd playwright && npx tsx setup/seed.ts {{ARGS}}

# Build the demo book: re-seed from scratch, walk every section of the console
# with the narrated `demo` project, and render playwright/demo/index.html from
# the steps it recorded. Needs `just start`. The book and its shots are
# gitignored — they are a deliverable, not a source.
e2e-demo:
    rm -rf playwright/demo
    cd playwright && npx tsx setup/seed.ts --reset
    cd playwright && npx playwright test --project demo --workers 1
    cd playwright && npx tsx scripts/demo-book.ts

# Capture the requirements evidence pack -- one named screenshot per REQ row,
# plus an index.md recording the URL and principal each came from -- into the
# tracked storage/files/images/evidence/ directory the documentation pages
# embed (needs `just start`).
e2e-screens:
    cd playwright && npx tsx scripts/capture.ts

# Capture the groups / projects / marketplace-access review pack into the
# gitignored playwright/screenshots/groups/ (18 shots + index.md), then build
# the self-contained review.html that embeds them. Needs `just start`.
e2e-screens-groups:
    cd playwright && npx tsx scripts/capture-groups.ts
    cd playwright && npx tsx scripts/review-html.ts

# Prove one requirements-register row end to end (the xlsx register is the
# master). Needs `just start` and `just e2e-seed`.
e2e-req REQ:
    cd playwright && npx playwright test --grep "@{{REQ}}"

# Test build without pushing
docker-test:
    just build-all
    just docker-build test
    @echo "Docker build successful! Image: systemprompt-template:test"

# ══════════════════════════════════════════════════════════════════════════════
# ADMIN & PLUGINS
# ══════════════════════════════════════════════════════════════════════════════

# Generate WebAuthn setup token for admin user
webauthn-admin EMAIL:
    {{CLI}} admin users webauthn generate-setup-token --email "{{EMAIL}}"

# Update Anthropic official plugins from vendor submodule and reimport
update-anthropic-plugins:
    git submodule update --remote vendor/knowledge-work-plugins
    {{CLI}} infra jobs run import_anthropic_plugins

# ══════════════════════════════════════════════════════════════════════════════
# BENCHMARKS
# ══════════════════════════════════════════════════════════════════════════════

# Benchmark governance endpoint. Downloads `hey` for the host OS/arch on first run.
benchmark REQUESTS="200" CONCURRENCY="100":
    #!/usr/bin/env bash
    set -e
    # Use system hey if available, else /tmp/hey
    if command -v hey >/dev/null 2>&1; then
        HEY="$(command -v hey)"
    else
        HEY="/tmp/hey"
    fi
    # Re-download if the cached binary can't execute here (e.g. Linux hey on a Mac).
    if ! { [[ -x "$HEY" ]] && "$HEY" --help >/dev/null 2>&1; }; then
        rm -f "$HEY"
        HEY="/tmp/hey"
        OS_ARCH="$(uname -s)/$(uname -m)"
        case "$OS_ARCH" in
            Darwin/*)
                HEY_URL="https://hey-release.s3.us-east-2.amazonaws.com/hey_darwin_amd64"
                echo "Installing hey from $HEY_URL..."
                curl -fsSL "$HEY_URL" -o "$HEY" && chmod +x "$HEY"
                ;;
            Linux/x86_64|Linux/amd64)
                HEY_URL="https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64"
                echo "Installing hey from $HEY_URL..."
                if ! curl -fsSL "$HEY_URL" -o "$HEY"; then
                    echo "ERROR: failed to download hey. Run: sudo apt-get install hey" >&2; exit 1
                fi
                chmod +x "$HEY"
                ;;
            *) echo "ERROR: no prebuilt hey for $OS_ARCH. Install with 'brew install hey' or 'go install github.com/rakyll/hey@latest'." >&2; exit 1 ;;
        esac
        if ! "$HEY" --help >/dev/null 2>&1; then
            echo "ERROR: hey won't run on $OS_ARCH." >&2
            if [[ "$OS_ARCH" == "Darwin/arm64" ]]; then
                echo "       Apple Silicon: 'softwareupdate --install-rosetta' or 'brew install hey'." >&2
            else
                echo "       Install manually: 'sudo apt-get install hey' or 'go install github.com/rakyll/hey@latest'." >&2
            fi
            rm -f "$HEY"; exit 1
        fi
    fi
    TOKEN_FILE="demo/.token"
    if [[ ! -f "$TOKEN_FILE" ]]; then
        echo "ERROR: No token. Run: ./demo/00-preflight.sh" >&2
        exit 1
    fi
    TOKEN=$(cat "$TOKEN_FILE")
    echo "Governance endpoint: {{REQUESTS}} requests, {{CONCURRENCY}} concurrent"
    echo ""
    "$HEY" -n {{REQUESTS}} -c {{CONCURRENCY}} -m POST \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"bench","tool_input":{"file_path":"/src/main.rs"}}' \
        "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo"


# --- Release ------------------------------------------------------------

# Step A of a release (on `next`, patch blocks re-commented): bump every pin to
# the new published core, point bridge/CORE_REF at its tag, and gate locally
# (migrate + build + clippy). Then `just preflight` and `just release`.
# See docs/RELEASING.md.
core-bump version:
    @! grep -q '^\[patch\.crates-io\]' Cargo.toml || (echo "ERROR: [patch.crates-io] is active — publish core and re-comment it first" && exit 1)
    scripts/sync-release-version.sh {{version}}
    @printf 'v{{version}}\n' > bridge/CORE_REF
    cargo update -w
    just db-up
    cargo run --bin systemprompt -- infra db migrate || true
    just build
    just clippy
    @echo "core-bump {{version}} complete — review the diff, then: just preflight && just release {{version}}"

# Pin bridge/CORE_REF to a core `next` commit while [patch.crates-io] is
# active (the vendored tree's HEAD by default). CI checks that ref out.
core-pin REF="":
    #!/usr/bin/env bash
    set -euo pipefail
    ref="{{REF}}"
    [ -n "$ref" ] || ref="$(git -C "${CORE_REPO:-../systemprompt-core}" rev-parse HEAD)"
    printf '%s\n' "$ref" > bridge/CORE_REF
    echo "bridge/CORE_REF = $ref"

# Step B: land next on main. main is published-core only, and every push to
# it IS a release — .github/workflows/release.yml creates the vX.Y.Z tag,
# the GitHub Release with the bridge binaries, and the GHCR image. This
# recipe proves the commit and fast-forwards; nothing is tagged by hand.
release version:
    @test -z "$(git status --porcelain)" || (echo "ERROR: working tree not clean" && exit 1)
    @test "$(git branch --show-current)" = "next" || (echo "ERROR: run from next — main only moves by fast-forward from it" && exit 1)
    @! grep -q '^\[patch\.crates-io\]' Cargo.toml || (echo "ERROR: [patch.crates-io] is active — main builds against published crates only" && exit 1)
    @! grep -q '^\[patch\.crates-io\]' tests/Cargo.toml || (echo "ERROR: [patch.crates-io] is active in tests/Cargo.toml" && exit 1)
    scripts/sync-release-version.sh {{version}} --check
    bash scripts/check-core-ref.sh
    git fetch origin main next
    @test "$(git rev-parse HEAD)" = "$(git rev-parse origin/next)" || (echo "ERROR: HEAD != origin/next — push next first (CI gates it)" && exit 1)
    @git merge-base --is-ancestor origin/main HEAD || (echo "ERROR: main is not an ancestor of next — rebase or merge main into next first" && exit 1)
    just preflight
    git push origin HEAD:main
    @echo "main -> $(git rev-parse --short HEAD). release.yml is now building v{{version}}: gh run watch"

# Focused validation for the governed requirements extension.
requirements-check:
    @scripts/build-coordinator.sh run requirements-check "" -- cargo check -p systemprompt-requirements-workflow -p systemprompt-web-admin

requirements-test:
    @scripts/build-coordinator.sh run requirements-test "" -- cargo test --manifest-path tests/Cargo.toml -p mcp-unit-tests requirements_workflow

requirements-integration:
    @scripts/build-coordinator.sh run requirements-integration "" -- cargo test --manifest-path tests/Cargo.toml -p mcp-integration-tests requirements_workflow

requirements-build:
    @scripts/build-coordinator.sh run requirements-build "" -- cargo build -p systemprompt-requirements-workflow -p systemprompt-mcp-knowledge-bank

requirements-online-check:
    @scripts/build-coordinator.sh run requirements-online-check "" -- cargo check -p systemprompt-mcp-knowledge-bank

# Drive ingestion checks share the repository build coordinator.
check-knowledge-drive:
    @scripts/build-coordinator.sh run check-knowledge-drive "" -- python3 scripts/check-knowledge-drive.py

# Opt-in release HTTP benchmark; requires a seeded disposable *_test database.
# See docs/DASHBOARD_PERFORMANCE.md. Does not start or restart the live server.
dashboard-benchmark:
    @scripts/build-coordinator.sh run dashboard-benchmark "release" -- cargo nextest run --manifest-path tests/Cargo.toml --release -p admin-contract-tests --run-ignored only -E 'test(benchmark_dashboard_http)' --no-capture
