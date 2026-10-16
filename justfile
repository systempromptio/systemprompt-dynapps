# systemprompt-dynapps
set dotenv-load
# Without this, `just cli ... --full-name "Test User"` word-splits the quoted
# value into two arguments before the CLI ever parses it.
set positional-arguments
import 'ci.just'

# check-dead-repository-code can consult a sibling checkout so a symbol only
# it consumes is not reported dead. Optional; an already-set value wins.
export SIBLING_REPO := env("SIBLING_REPO", if path_exists("../systemprompt-template") == "true" { "../systemprompt-template" } else { "" })

CLI_RELEASE := "target/release/systemprompt"

# Cloud profile every deploy targets: tenant e43527c1cbb5, Fly app
# sp-e43527c1cbb5, served at https://dynapps.systemprompt.io
# (tenant hostname https://e43527c1cbb5.systemprompt.io).
# See .systemprompt/profiles/production/.
DEPLOY_PROFILE := "production"
# Where .github/workflows/release.yml publishes the gateway image.
IMAGE := "ghcr.io/systempromptio/systemprompt-dynapps"

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
    # Run the sidecar beside the selected CLI. `plugins run` initializes the
    # complete services catalog before spawning an extension, which can make
    # this recovery command fail on an unrelated config/schema mismatch.
    CLI_BIN="{{CLI}}"
    DEV_LOGIN_BIN="$(dirname "$CLI_BIN")/systemprompt-dev-login"
    if [ ! -x "$DEV_LOGIN_BIN" ]; then
        echo "ERROR: $DEV_LOGIN_BIN not found. Run: just build" >&2
        exit 1
    fi
    exec "$DEV_LOGIN_BIN" "{{USER}}"

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

# Build (Unix) - one build in flight at a time, always of the latest source;
# cargo's own incremental cache decides how much recompiles
[unix]
build *FLAGS:
    @scripts/build-coordinator.sh run build "{{FLAGS}}" -- {{just_executable()}} _build-uncoordinated {{FLAGS}}

# What is the build/lint/test state right now? Read this before running anything.
[unix]
build-status *RECIPE:
    @scripts/build-coordinator.sh status {{RECIPE}}

# Kept for muscle memory: `just build` always compiles the current tree, so
# this is the same recipe.
[unix]
build-force *FLAGS:
    @scripts/build-coordinator.sh run build "{{FLAGS}}" -- {{just_executable()}} _build-uncoordinated {{FLAGS}}

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
    # a crate can deliberately query tables that database will never have.
    # Those queries are covered by the committed .sqlx/ cache and can only ever
    # build offline, so first-run setup and anyone with a fresh database needs
    # this escape hatch.
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

# Clippy (Unix) - one in flight at a time, same coordinator as `just build`
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
    # Why: same escape hatch as `build`. A crate's queries can name tables the
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
    # tests/ is a standalone workspace and is not covered by --workspace
    SQLX_OFFLINE=true cargo clippy --manifest-path tests/Cargo.toml --workspace --all-targets {{FLAGS}} -- -D warnings
    # bridge/ is a standalone workspace and is not covered by --workspace
    cargo clippy --manifest-path bridge/Cargo.toml --all-targets {{FLAGS}} -- -D warnings
    # Why: the GUI (src/registry.rs's marketplace seam included) is cfg'd to
    # windows|macos, so a Linux clippy never compiles it and a core API break
    # there first surfaces in release.yml's mac/win jobs (0.53.0). Clippy does
    # not link, so the Windows cfg set checks on Linux with no mingw toolchain;
    # macOS-only code needs a mac runner (release.yml is the first to see it).
    if [ "$(uname -s)" = "Linux" ]; then
        rustup target add x86_64-pc-windows-gnu
        cargo clippy --manifest-path bridge/Cargo.toml --all-targets --target x86_64-pc-windows-gnu {{FLAGS}} -- -D warnings
    fi

# Focused Desktop/Bridge policy and protocol regression tests.
test-bridge:
    @scripts/build-coordinator.sh run test-bridge "" -- {{just_executable()}} _test-bridge-uncoordinated

_test-bridge-uncoordinated:
    cargo nextest run --no-fail-fast --manifest-path tests/Cargo.toml -p bridge-unit-tests
    node --experimental-vm-modules --test tests/bridge/*.test.mjs

# Unit tests: the unit crates of the tests/ workspace.
# If sqlx offline errors appear, run `just prepare` first to refresh .sqlx.
# Emergency connector hotfix: exact token boundary and connection regressions.
test-hotfix:
    @bash scripts/build-coordinator.sh run test-hotfix "" -- just _test-hotfix-uncoordinated

_test-hotfix-uncoordinated:
    SQLX_OFFLINE=true cargo nextest run --no-fail-fast --manifest-path tests/Cargo.toml -p web-unit-tests -E 'test(connector_oauth::) | test(connector_cards::)'

test-unit:
    @scripts/build-coordinator.sh run test-unit "" -- {{just_executable()}} _test-unit-uncoordinated

# Why: without --no-fail-fast nextest stops at the first failing test, so a
# Gates round reports one finding and the next one waits for another 45-min
# round (the 0.54.0 pin-flip took five rounds for twenty). Every tier runs to
# completion and the job lists every failure at once.
#
# Unit tests select their three crates explicitly. Database tiers use
# --workspace to unify their broader feature set and reuse it across tiers.
_test-unit-uncoordinated:
    cargo nextest run --no-fail-fast --manifest-path tests/Cargo.toml -p mcp-unit-tests -p web-unit-tests -p bridge-unit-tests
    node --experimental-vm-modules --test tests/bridge/*.test.mjs

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
    cargo nextest run --no-fail-fast --manifest-path tests/Cargo.toml --workspace -E 'package(mcp-integration-tests) | package(web-integration-tests) | package(admin-db-core-tests) | package(admin-db-config-tests) | package(gateway-integration-tests) | package(schema-upgrade-tests)'

# HTTP contract suite: drives every admin route under three principals and
# diffs the result against tests/contract/admin/baseline.txt. Same throwaway-
# database convention as test-integration. A status change fails the run; if
# it is deliberate, re-record with UPDATE_CONTRACT_BASELINE=1 and list it in
# the PR.
test-contract *ARGS:
    @scripts/build-coordinator.sh run test-contract "$*" -- {{just_executable()}} _test-contract-uncoordinated "$@"

_test-contract-uncoordinated *ARGS:
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
    cargo nextest run --no-fail-fast --manifest-path tests/Cargo.toml --workspace -E 'package(admin-contract-tests)' "$@"

# All tests
# Every tier runs even after one fails. As `just` dependencies they stopped
# at the first red tier, so the contract tier's failures stayed hidden behind
# unit and integration fixes for three rounds in 0.59.
test:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    for tier in test-unit test-integration test-contract; do
        echo "==> $tier"
        {{just_executable()}} "$tier" || failed+=("$tier")
    done
    if [ "${#failed[@]}" -gt 0 ]; then printf '::error::test failed: %s\n' "${failed[@]}"; exit 1; fi

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
        check-fail-soft.sh
        check-swallowed-errors.sh
        check-tracing-messages.sh
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
        lint-inline-tests.sh
        lint-test-seams.sh
        lint-raw-ids.sh
        lint-env-vars.sh
        check-glob-reexports.sh
        check-comments.sh
        check-yaml-comments.sh
        lint-inline-comments.sh
        check-duplicate-types.sh
        check-field-copy-from.sh
        check-repository-naming.sh
        check-plane-ownership.sh
        check-ledger-reads.sh
        check-admin-template-links.sh
        check-admin-template-assets.sh
        # admin-css-classes + frontend-standards now run as cargo tests in
        # tests/unit/web/src/ (admin_css_classes.rs, frontend_standards.rs).
        check-dead-repository-code.sh
        check-file-headers.sh
        check-file-size.sh
        check-asset-reachability.sh
        check-workspace-deps.sh
        check-dockerfile-paths.sh
        check-dropped-schema.sh
        check-retired-features.sh
        validate-services.sh
        check-gateway-routes.sh
        check-core-ref.sh
        check-schema-baseline.sh
        check-skill-cli-flags.sh
        coverage-badge.sh
        check-docs-version.sh
        check-template-fields.sh
        check-mcp-tool-schemas.sh
    )
    # Why: cargo replaces the rust-contracts scanner binary while another gate
    # is exec'ing it ("never executed"), so build it once here, before the fan-out.
    contracts="{{justfile_directory()}}/../systemprompt-core/scripts/rust-contracts/Cargo.toml"
    if [ -f "$contracts" ]; then
        cargo build --quiet --locked --manifest-path "$contracts" || { echo "rust-contracts scanner failed to build" >&2; exit 1; }
    fi
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
# runs on every push to next and on the PR onto main (the release does not
# re-run it). Run it before you push so CI is confirmation, not discovery.
# `preflight` adds the coverage floor/ratchet on top, which CI measures on
# main and nightly without blocking.
verify: preflight-static preflight-lint test
    @echo "verify: format, sqlx cache, lint gates, clippy, docs, msrv, and tests all pass"

# ══════════════════════════════════════════════════════════════════════════════
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
#
# Single-flight through the coordinator: it takes the same lock as build,
# clippy and the test tiers, so a prepare never races a compile that is
# reading .sqlx, and two agents running it at once share one run.
prepare:
    @scripts/build-coordinator.sh run prepare "" -- {{just_executable()}} _prepare-uncoordinated

_prepare-uncoordinated:
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
    echo "Preparing SQLx offline cache..."
    export DATABASE_URL="$DB_URL"
    ROOT="{{justfile_directory()}}"
    # Crates whose own .sqlx cache this recipe creates as scratch.
    EXTENSION_DIRS="extensions/web extensions/mcp/shared extensions/mcp/systemprompt extensions/mcp/agent-farm"
    # The tracked cache is never written in place. `cargo sqlx prepare`
    # truncates and rewrites .sqlx as it goes, and this recipe runs it several
    # times, so an in-place run that dies part-way (a failed pass, or a SIGKILL
    # that no trap can catch) used to leave the cache worse than it found it —
    # observed once as 55 tracked entries gone, and once as 1142 → 3 files.
    # Every pass now lands in $NEW; the passes are unioned there (query
    # filenames are content hashes, so a plain copy merge cannot collide); and
    # the tracked .sqlx is replaced by one rename at the very end. A run that
    # fails at any earlier point leaves .sqlx exactly as it was.
    #
    # `cargo sqlx prepare --workspace` has no output-directory option: it
    # always writes the workspace-root .sqlx. For each root pass the tracked
    # cache is therefore moved to PARKED — a sibling on the same filesystem,
    # gitignored, never under /tmp — and moved back the moment the pass ends.
    # tests/.sqlx is a tracked symlink to ../.sqlx (the four query!-using test
    # crates resolve their offline cache through it); sqlx-cli would follow
    # the link and truncate the real cache, so the link is parked in $SNAP for
    # the tests pass and put back by the trap.
    #
    # A SIGKILL between a park and its unpark leaves the parked copy on disk.
    # Nothing is lost: the next run heals both before doing anything, and the
    # check-sqlx-cache gate refuses to pass while either is out of place.
    PARKED="$ROOT/.sqlx.prepare-parked"
    TESTS_LINK="$ROOT/tests/.sqlx"
    if [ -e "$PARKED" ]; then
        echo "Recovering the cache a killed prepare left parked at .sqlx.prepare-parked"
        rm -rf "$ROOT/.sqlx"
        mv "$PARKED" "$ROOT/.sqlx"
    fi
    if [ ! -L "$TESTS_LINK" ]; then
        echo "Restoring the tests/.sqlx symlink a killed prepare replaced with a directory"
        rm -rf "$TESTS_LINK"
        ln -s ../.sqlx "$TESTS_LINK"
    fi
    rm -rf "$ROOT"/.sqlx.prepare-scratch.* "$ROOT/.sqlx.new" "$ROOT/.sqlx.old"
    # The scratch lives beside the cache so the final rename never crosses a
    # filesystem.
    SNAP="$(mktemp -d "$ROOT/.sqlx.prepare-scratch.XXXXXX")"
    NEW="$SNAP/new"
    mkdir -p "$NEW"
    LINK_PARKED=0
    ROOT_PARKED=0
    unpark_root_cache() {
        if [ "$ROOT_PARKED" -eq 1 ]; then
            rm -rf "$ROOT/.sqlx"
            mv "$PARKED" "$ROOT/.sqlx"
            ROOT_PARKED=0
        fi
    }
    cleanup_prepare() {
        rc=$1
        unpark_root_cache
        for dir in $EXTENSION_DIRS; do
            rm -rf "$ROOT/$dir/.sqlx"
        done
        if [ "$LINK_PARKED" -eq 1 ]; then
            rm -rf "$TESTS_LINK"
            mv "$SNAP/tests-link" "$TESTS_LINK"
        fi
        rm -rf "$SNAP"
        if [ "$rc" -ne 0 ]; then
            echo ""
            echo "prepare FAILED — the tracked .sqlx cache was never written; it is"
            echo "exactly what it was before this run."
        fi
        exit "$rc"
    }
    trap 'cleanup_prepare $?' EXIT
    # One root-workspace pass: park the tracked cache, let sqlx write a fresh
    # one, harvest its queries into $NEW, restore the original.
    root_pass() {
        if [ -e "$ROOT/.sqlx" ]; then
            mv "$ROOT/.sqlx" "$PARKED"
            ROOT_PARKED=1
        fi
        cargo sqlx prepare --workspace "$@"
        cp "$ROOT/.sqlx/"query-*.json "$NEW/" 2>/dev/null || true
        unpark_root_cache
    }
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
    root_pass
    # Second pass: the tests workspace builds systemprompt-web-admin-governance with
    # `governance-ssr`, whose queries a default-feature prepare never expands
    # (and therefore prunes). The union with the first pass happens in $NEW.
    root_pass -- --package systemprompt-web-admin-governance --features systemprompt-web-admin-governance/governance-ssr
    # Per-crate prepare for binary/extension crates that cargo sqlx skips.
    #
    # These caches are SCRATCH, not artefacts: each crate's queries are copied
    # into $NEW and the per-crate directory is removed by the trap. Every crate
    # here is a root workspace member, so the root cache is the one the macros
    # resolve against. Leaving them on disk committed 1,703 duplicate files,
    # which is why a routine prepare once showed up as a 1,374-file diff.
    for dir in $EXTENSION_DIRS; do
        if [ -f "$ROOT/$dir/Cargo.toml" ]; then
            # Skip crates with no sqlx dependency — prepare would only
            # resurrect an orphaned .sqlx cache.
            if ! grep -qE '^sqlx' "$ROOT/$dir/Cargo.toml"; then
                continue
            fi
            echo "  Preparing $dir..."
            # Why: `... | tail -1` made the pipeline's status tail's, and the
            # trailing `|| true` discarded even that. A per-crate pass could
            # fail and its queries silently never reach the root cache — the
            # same shape as the bug this recipe exists to prevent.
            if ! ext_log=$(cd "$ROOT/$dir" && cargo sqlx prepare 2>&1); then
                echo "$ext_log" | tail -20
                echo "Error: cargo sqlx prepare failed in $dir"
                exit 1
            fi
            echo "$ext_log" | tail -1
            if ls "$ROOT/$dir/.sqlx/"*.json >/dev/null 2>&1; then
                cp "$ROOT/$dir/.sqlx/"*.json "$NEW/"
            fi
            rm -rf "$ROOT/$dir/.sqlx"
        fi
    done
    echo "  Preparing tests workspace..."
    TEST_SQLX_PKGS=$(cargo metadata --manifest-path tests/Cargo.toml --format-version 1 2>/dev/null \
        | jq -r '.packages[] | select(.dependencies[]?.name == "sqlx") | .name' \
        | sort -u)
    if [ -z "$TEST_SQLX_PKGS" ]; then
        echo "Error: could not resolve the test workspace sqlx package list."
        exit 1
    fi
    for pkg in $TEST_SQLX_PKGS; do
        cargo clean --manifest-path tests/Cargo.toml -p "$pkg" 2>/dev/null || true
    done
    if [ -L "$TESTS_LINK" ]; then
        mv "$TESTS_LINK" "$SNAP/tests-link"
        LINK_PARKED=1
    fi
    if ! test_log=$(cd "$ROOT/tests" && cargo sqlx prepare --workspace -- --all-targets 2>&1); then
        echo "$test_log" | tail -20
        echo "Error: cargo sqlx prepare failed in tests"
        exit 1
    fi
    echo "$test_log" | tail -1
    if ls "$TESTS_LINK/"*.json >/dev/null 2>&1; then
        cp "$TESTS_LINK/"*.json "$NEW/"
    fi
    rm -rf "$TESTS_LINK"
    mv "$SNAP/tests-link" "$TESTS_LINK"
    LINK_PARKED=0
    # Every pass succeeded: one swap replaces the tracked cache. Both renames
    # are within the same filesystem, so the window with no .sqlx at all is
    # two syscalls wide; a run killed before this line changed nothing.
    NEW_QUERIES=$(find "$NEW" -maxdepth 1 -name 'query-*.json' | wc -l)
    if [ "$NEW_QUERIES" -eq 0 ]; then
        echo "Error: every pass succeeded but produced no queries; refusing to swap."
        exit 1
    fi
    mv "$NEW" "$ROOT/.sqlx.new"
    if [ -e "$ROOT/.sqlx" ]; then mv "$ROOT/.sqlx" "$ROOT/.sqlx.old"; fi
    mv "$ROOT/.sqlx.new" "$ROOT/.sqlx"
    rm -rf "$ROOT/.sqlx.old"
    if ! git -C "$ROOT" diff --quiet -- tests/.sqlx; then
        echo "Error: tests/.sqlx changed — it must stay a symlink to ../.sqlx."
        exit 1
    fi
    echo "SQLx cache prepared successfully ($NEW_QUERIES production and test queries cached)"

# ══════════════════════════════════════════════════════════════════════════════
# SERVICES & DATABASE
# ══════════════════════════════════════════════════════════════════════════════

# True when this clone's Postgres answers on the profile's database_url.
# Used to decide whether `start` needs to touch docker at all.
_db-reachable:
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    [ -f "$SECRETS_FILE" ] || exit 1
    DB_URL=$(jq -r '.database_url // empty' "$SECRETS_FILE" 2>/dev/null)
    [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ] || exit 1
    PG_ISREADY=""
    if command -v pg_isready >/dev/null 2>&1; then PG_ISREADY="pg_isready"
    elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then PG_ISREADY="/opt/homebrew/opt/libpq/bin/pg_isready"
    elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then PG_ISREADY="/usr/local/opt/libpq/bin/pg_isready"
    fi
    if [ -n "$PG_ISREADY" ]; then
        exec "$PG_ISREADY" -d "$DB_URL" -t 2 >/dev/null 2>&1
    fi
    # No libpq on PATH — fall back to a bare TCP probe of host:port.
    HOSTPORT=${DB_URL#*@}
    HOST=${HOSTPORT%%:*}
    PORT=${HOSTPORT#*:}
    PORT=${PORT%%/*}
    timeout 2 bash -c "exec 3<>/dev/tcp/$HOST/$PORT" 2>/dev/null

# Start server (always uses local profile)
start:
    #!/usr/bin/env bash
    set -euo pipefail
    # Only touch docker when Postgres is not already answering. `docker compose`
    # can fail for reasons that have nothing to do with this clone's database —
    # a broken Docker Desktop WSL vsock relay exits 1 while the container is up
    # and healthy — and aborting here would preempt the CLI, which is what
    # detects an already-running server and reports it rather than clobbering it.
    if [ -f .systemprompt/docker/local.yaml ] \
        && ! just --quiet _db-reachable 2>/dev/null; then
        DB_UP_OK=1
        just db-up local || DB_UP_OK=0
        if [ "$DB_UP_OK" -ne 1 ] && ! just --quiet _db-reachable 2>/dev/null; then
            echo "error: 'just db-up' failed and Postgres is not reachable." >&2
            echo "  Check the daemon with 'docker ps'; on WSL an 'accept4 failed'" >&2
            echo "  vsock error means Docker Desktop needs restarting." >&2
            exit 1
        fi
        # `docker compose up -d` returns when the container is running, not when
        # Postgres is accepting connections. Recovery after an unclean Docker
        # shutdown can take a few minutes, so do not mistake that normal startup
        # window for a failed daemon. Override the bounded wait when needed.
        DB_START_TIMEOUT_SECONDS=${DB_START_TIMEOUT_SECONDS:-900}
        if ! [[ "$DB_START_TIMEOUT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
            echo "error: DB_START_TIMEOUT_SECONDS must be a positive integer." >&2
            exit 1
        fi
        echo "Waiting up to ${DB_START_TIMEOUT_SECONDS}s for Postgres to accept connections..."
        DB_READY=0
        for ((elapsed = 0; elapsed < DB_START_TIMEOUT_SECONDS; elapsed++)); do
            if just --quiet _db-reachable 2>/dev/null; then
                DB_READY=1
                break
            fi
            if [ "$elapsed" -gt 0 ] && [ $((elapsed % 15)) -eq 0 ]; then
                echo "  Postgres is still starting (${elapsed}s elapsed)..."
            fi
            sleep 1
        done
        if [ "$DB_READY" -ne 1 ]; then
            echo "error: Postgres did not become reachable within ${DB_START_TIMEOUT_SECONDS}s." >&2
            echo "  Inspect recovery with 'docker logs $(just _project_name local)-postgres-1'." >&2
            echo "  Override the wait with DB_START_TIMEOUT_SECONDS=<seconds> just start." >&2
            exit 1
        fi
        echo "Postgres is ready."
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

# Seed the fictional vet-group demo into the local instance (run after `just start`).
demo-vet-seed:
    ./demo/vet/seed.sh

# Grep the vet demo rows for real-tenant markers; expect 0 hits.
demo-vet-leak-check REF_DB="":
    ./demo/vet/leak-check.sh {{REF_DB}}

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
        # Why: offline. The database this just created is empty and unmigrated,
        # and the online build would verify every sqlx query against it. The
        # committed .sqlx/ cache is the source of truth at first run, so a clean
        # clone does not need a populated database to bootstrap.
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
        # `admin setup` writes core's default webhook authz mode; this binary
        # registers its authz hook in-process, and core refuses to boot a
        # registered hook under any other mode.
        "$BIN" admin config governance set --mode extension
        if [ "$HTTP_PORT" != "8080" ]; then
            "$BIN" admin config server set --port "$HTTP_PORT" \
                --api-server-url "http://localhost:${HTTP_PORT}" \
                --api-internal-url "http://localhost:${HTTP_PORT}" \
                --api-external-url "http://localhost:${HTTP_PORT}"
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
    # Why: offline, for the same reason as the debug build above — a fresh
    # install's database is unmigrated, so the online sqlx check cannot pass.
    # `just migrate` below still applies every registered migration; this only
    # decides where the compile-time query check reads its schema from.
    SQLX_OFFLINE=true just build --release
    python3 scripts/setup-local-kit-source.py "$PROFILE_DIR/profile.yaml" "$PROFILE_DIR/secrets.json"
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
# the CLI session happens to be switched to. The real docker is pinned ahead of
# any wrapper shim on PATH: a shim broke `docker build` during 0.51.0 with an
# error that pointed nowhere near the shim.
deploy *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -x /usr/bin/docker ]; then export PATH=/usr/bin:$PATH; fi
    docker_bin="$(command -v docker || true)"
    case "$docker_bin" in
        /usr/bin/*) ;;
        *) echo "warning: docker resolves to ${docker_bin:-<none>}, not /usr/bin/docker — a wrapper shim can fail 'docker build' with an unrelated error" >&2 ;;
    esac
    for flag in "$@"; do
        if [ "$flag" = "--check" ]; then
            exec {{CLI}} cloud deploy --profile {{DEPLOY_PROFILE}} "$@"
        fi
    done
    {{just_executable()}} core-guard
    python3 scripts/check-deploy-sources.py .systemprompt/profiles/{{DEPLOY_PROFILE}}/profile.yaml
    {{just_executable()}} deploy-check
    {{just_executable()}} build-all
    exec {{CLI_RELEASE}} cloud deploy --profile {{DEPLOY_PROFILE}} "$@"

# Pin a kit's services bundle in a profile (default: the production profile)
# to a GHCR digest, or to the kit's channel tag (e.g. `stable`) so every
# release the kit's CI publishes is one Import on /admin/sync away. The kit's
# image, public key and pull mode come from deploy/kit/known-kits.json; the
# entry is rewritten in place or inserted (with the `sources:` block) when
# absent. The digest is printed by the kit's publish workflow.
services-pin KIT REF PROFILE=".systemprompt/profiles/production/profile.yaml":
    python3 scripts/services-pin.py "{{KIT}}" "{{REF}}" "{{PROFILE}}"

# Export a marketplace from services/ into a kit repository tree (Anthropic
# marketplace format + systemprompt sidecars, no access block) and prove it
# re-imports identically. Runbook: deploy/kit/README-INTEGRATION.md.
kit-export MARKETPLACE DIR:
    #!/usr/bin/env bash
    set -euo pipefail
    BIN="$(dirname "{{CLI}}")/systemprompt-kit-export"
    if [ ! -x "$BIN" ]; then
        echo "ERROR: $BIN not found. Run: just build" >&2
        exit 1
    fi
    exec "$BIN" "{{MARKETPLACE}}" "{{DIR}}" --services "{{justfile_directory()}}/services"

# What "build next and next together" means in practice: while
# [patch.crates-io] is active the server compiles against ../systemprompt-core
# in place (no vendored copy), so the only way a deploy can ship exactly the
# core CI will gate is if that checkout is clean and sits at bridge/CORE_REF.
# Refuses otherwise; with the patch commented there is nothing to check.
core-guard:
    #!/usr/bin/env bash
    set -euo pipefail
    grep -qE '^\[patch\.crates-io\]' Cargo.toml || { echo "core-guard: patch commented; building against the published core"; exit 0; }
    core="${CORE_REPO:-../systemprompt-core}"
    # -e, not -d: a worktree's .git is a file, and a detached worktree is the sanctioned clean checkout.
    [ -e "$core/.git" ] || { echo "core-guard: no core checkout at $core" >&2; exit 1; }
    expected="$(tr -d '[:space:]' < bridge/CORE_REF)"
    head="$(git -C "$core" rev-parse HEAD)"
    [ "$head" = "$expected" ] || { echo "core-guard: $core is at ${head:0:12}, bridge/CORE_REF pins ${expected:0:12}; commit and 'just core-pin' first" >&2; exit 1; }
    dirty="$(git -C "$core" status --porcelain --untracked-files=all)"
    [ -z "$dirty" ] || { echo "core-guard: $core has uncommitted changes; commit them on core next (or set them aside) before deploying:" >&2; echo "$dirty" >&2; exit 1; }
    echo "core-guard: $core clean at ${head:0:12} == bridge/CORE_REF"

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

# Build all MCP servers (reads from manifest.yaml files) — single-flight;
# cargo skips packages whose source did not change.
build-mcp:
    @scripts/build-coordinator.sh run build-mcp "" -- {{just_executable()}} _build-mcp-uncoordinated

_build-mcp-uncoordinated:
    DATABASE_URL="$(just _db-url)" {{CLI}} build mcp --release

# The workspace build includes both MCP binaries. publish_pipeline includes
# bundle_admin_css and copy_extension_assets; run each producer once. The
# pipeline reads the local tree, so it is pinned to `local` — never the
# profile the CLI session happens to hold (deploy-check switches to production).
build-all:
    just build --release
    just bridge-package-host
    just downloads-fetch
    {{CLI_RELEASE}} infra jobs run publish_pipeline --profile local
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

# Local Kubernetes load-test rig: kind + the real Helm chart + Postgres +
# core's mock inference + Prometheus. No paid calls. deploy/loadtest/kind/README.md.
loadtest-kind-up:
    deploy/loadtest/kind/cluster.sh up

# Render deploy/loadtest/kind/.profile/secrets.json (gitignored).
loadtest-kind-secrets:
    deploy/loadtest/kind/cluster.sh secrets

# helm upgrade --install with values-kind.yaml; extra args pass through,
# e.g. `just loadtest-kind-deploy --set gateway.autoscaling.enabled=true`.
loadtest-kind-deploy *SET:
    deploy/loadtest/kind/cluster.sh deploy {{SET}}

# Seed USERS load-test users + PAT + session into .tokens/<set>.csv and a Secret.
loadtest-kind-seed USERS SET="" PROJECTS="commerce,core":
    deploy/loadtest/kind/seed.sh {{USERS}} {{ if SET == "" { "u" + USERS } else { SET } }} {{PROJECTS}}

# One run; options as deploy/loadtest/kind/run.sh documents.
loadtest-kind-run NAME *ARGS:
    deploy/loadtest/kind/run.sh {{NAME}} {{ARGS}}

# Every scenario in scenarios.json (optional name regex).
loadtest-kind-matrix FILTER=".":
    deploy/loadtest/kind/matrix.sh '{{FILTER}}'

# The ~15-minute pre-release subset (scenarios.json `smoke`).
loadtest-kind-smoke FILTER=".":
    LOADTEST_SUITE=smoke deploy/loadtest/kind/matrix.sh '{{FILTER}}'

# Image from locally built release binaries (run `just build --release` first).
loadtest-kind-image-local:
    deploy/loadtest/kind/cluster.sh image-local

# helm lint + template with values-kind.yaml + profile schema check in the image.
loadtest-kind-lint:
    deploy/loadtest/kind/cluster.sh lint

loadtest-kind-down:
    deploy/loadtest/kind/cluster.sh down

# Run a local or published image against .env
docker-run TAG="local":
    docker run -p 8080:8080 --env-file .env {{IMAGE}}:{{TAG}}

# The colleague install: pull the published image and start (docs/install/docker.md).
compose-up:
    docker compose pull && docker compose up -d

# Build the branded bridge. Its own standalone workspace, NOT the server
# workspace — `just build` does not touch it, and a bare `cargo build` from the
# repo root silently builds the server instead. A local build follows the
# sibling core wherever it is (on `next` the sibling is routinely ahead of
# bridge/CORE_REF between pins); only the package recipes, which produce what
# ships, insist on the pin.
bridge-build *ARGS: (core-checkout "warn")
    cd {{justfile_directory()}}/bridge && cargo build --release {{ARGS}}

# The client depends on systemprompt-bridge by relative path, and that crate is
# not published — so unlike the server, building the client needs the core
# repository checked out beside this one. Clones it when absent, leaves local
# work alone, and compares a clean checkout with bridge/CORE_REF: a mismatch
# is fatal by default and a warning with MISMATCH=warn.
core-checkout MISMATCH="fail":
    #!/usr/bin/env bash
    set -euo pipefail
    CORE="{{justfile_directory()}}/../systemprompt-core"
    if [ -d "$CORE/.git" ]; then
        if [ -n "$(git -C "$CORE" status --porcelain)" ]; then
            echo "core checkout has local changes — leaving it as it is."
        else
            echo "Using existing core checkout at $(git -C "$CORE" rev-parse --short HEAD)"
            # Updating here can mix a server built from one core with a bridge from another.
            expected="$(tr -d '\r\n' < "{{justfile_directory()}}/bridge/CORE_REF")"
            head="$(git -C "$CORE" rev-parse HEAD)"
            pinned="$(git -C "$CORE" rev-parse "$expected^{commit}")"
            if [ "$head" != "$pinned" ]; then
                if [ "{{MISMATCH}}" = "warn" ]; then
                    echo "warn: core checkout ${head:0:9} differs from bridge/CORE_REF ${pinned:0:9};" >&2
                    echo "      building against the checkout. Pin before packaging: just core-pin" >&2
                else
                    echo "core checkout differs from bridge/CORE_REF; pin core (just core-pin) before building." >&2
                    exit 1
                fi
            fi
        fi
    else
        echo "Cloning systemprompt-core beside this repo (the client needs it)."
        git clone --quiet https://github.com/systempromptio/systemprompt-core "$CORE"
    fi

# Package the branded bridge as a Linux release tarball into dist/
# Coordinated so two agents never package at once; every call rebuilds (cargo
# makes an unchanged tree cheap).
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
    @bash scripts/verify-bridge-macos.sh storage/files/downloads/dynapps-bridge-macos.dmg

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
# building them: `systemprompt-dynapps-server-<version>-linux-<arch>.tar.gz`
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
    name="systemprompt-dynapps-server-$v-linux-$arch.tar.gz"
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
    echo "==> downloading $name from release v$v"
    gh release download "v$v" -R systempromptio/systemprompt-dynapps -p "$name" -p SHA256SUMS -D "$tmp" \
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
    VOL="dynapps-claude-${SCOPE}"
    NAME="dynapps-claude-${SCOPE}"

    if [ "$(docker inspect -f '{{{{.State.Running}}}}' "$NAME" 2>/dev/null)" = "true" ]; then
        echo "Already running for this repo — opening another session in it."
        exec docker exec -it "$NAME" bash -lc claude
    fi
    docker rm -f "$NAME" >/dev/null 2>&1 || true

    if ! docker image inspect dynapps-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    # The client is a separate workspace and depends on systemprompt-bridge by
    # relative path into a sibling core checkout, so building it needs that
    # checkout present. `bridge-build` fetches it. Do this before the code is
    # spent: a first-run compile can outlive the code's 10-minute TTL, so
    # `just bridge-build` belongs in setup rather than here.
    BRIDGE="{{justfile_directory()}}/bridge/target/release/dynapps-bridge"
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
            dynapps-clean-client:local \
            -sf --max-time 5 "$GATEWAY/health" 2>/dev/null | grep -Eq '"status" *: *"healthy"'
    }

    if ! gateway_reachable; then
        # Separate the two causes before touching anything. If the gateway does
        # not answer on the host either, it is down or on another port, and
        # rebinding it would be treating the wrong illness — Docker Desktop and
        # WSL2 proxy host.docker.internal to host loopback, so a 127.0.0.1 bind
        # is genuinely reachable there and the bind is often not the problem.
        # A bootstrapping server answers 200 {"status":"starting"}; only a
        # finished boot counts as up.
        if ! curl -sf --max-time 5 "$HOST_GATEWAY/health" 2>/dev/null | grep -Eq '"status" *: *"healthy"'; then
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
        CODE_ENV=(-e DYNAPPS_BRIDGE_CODE="{{CODE}}")
    elif ! docker run --rm --entrypoint test \
            -v "$VOL":/home/tester \
            dynapps-clean-client:local \
            -f /home/tester/.config/dynapps/dynapps-bridge.pat >/dev/null 2>&1; then
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
        -e DYNAPPS_BRIDGE_GATEWAY_URL="$GATEWAY" \
        "${CODE_ENV[@]}" \
        -e CLEAN_CLIENT_EXEC_CLAUDE=1 \
        -e CLEAN_CLIENT_ALLOW_STATE=1 \
        -v "$VOL":/home/tester \
        -v "$BRIDGE:/usr/local/bin/dynapps-bridge:ro" \
        -v "{{justfile_directory()}}/deploy/clean-client/bootstrap.sh:/usr/local/bin/bootstrap.sh:ro" \
        "${PORTS[@]}" \
        dynapps-clean-client:local /usr/local/bin/bootstrap.sh

# ──────────────────────────────────────────────────────────────────────────────
# CLEAN CLIENT — a config-free Linux box for testing the Claude Code + bridge
# integration. See deploy/clean-client/README.md.
# ──────────────────────────────────────────────────────────────────────────────

# Build the clean-client image (context is deploy/clean-client only — no repo state)
clean-client-build *ARGS:
    docker build {{ARGS}} -f deploy/clean-client/Dockerfile -t dynapps-clean-client:local deploy/clean-client

# Shell into a clean client. PERSIST=1 keeps the login across runs.
# GATEWAY overrides the gateway URL (default: this WSL host on :8080).
clean-client PERSIST="0" GATEWAY="http://host.docker.internal:8080":
    #!/usr/bin/env bash
    set -euo pipefail
    if ! docker image inspect dynapps-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    # Mount the bridge you just built, read-only. Absent is not fatal: you can
    # still test Claude Code against the gateway without the bridge installed.
    BRIDGE="{{justfile_directory()}}/bridge/target/release/dynapps-bridge"
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
        MOUNTS+=(-v "$BRIDGE:/usr/local/bin/dynapps-bridge:ro")
    else
        echo "warn: $BRIDGE not found — run 'cd bridge && cargo build --release' for the full flow." >&2
    fi

    # PERSIST keeps ~/.config/dynapps and ~/.claude in a named volume so a PAT
    # survives a restart. Off by default: a throwaway HOME is the point.
    if [ "{{PERSIST}}" = "1" ]; then
        MOUNTS+=(-v dynapps-clean-home:/home/tester -e CLEAN_CLIENT_ALLOW_STATE=1)
        echo "State persists in volume 'dynapps-clean-home' — 'just clean-client-reset' wipes it."
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
        --name dynapps-clean-client \
        --hostname clean-client \
        --add-host host.docker.internal:host-gateway \
        -e DYNAPPS_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        "${MOUNTS[@]}" "${PORTS[@]}" \
        dynapps-clean-client:local

alias cc := clean-client-ready

# Clean client, signed in and ready: paste one code, then type `claude`.
clean-client-ready GATEWAY="http://host.docker.internal:8080":
    #!/usr/bin/env bash
    set -euo pipefail

    # Already running (another terminal has it): open a second shell in it
    # rather than failing on the name conflict.
    if [ "$(docker inspect -f '{{{{.State.Running}}}}' dynapps-clean-client 2>/dev/null)" = "true" ]; then
        echo "Container 'dynapps-clean-client' is already running — opening a shell in it."
        exec docker exec -it dynapps-clean-client bash -l
    fi

    # Stopped leftover (a crashed run, or one started without --rm): restart it
    # and shell in, so anything written outside the /home/tester volume survives.
    # Only if it refuses to start do we reclaim the name.
    if docker inspect dynapps-clean-client >/dev/null 2>&1; then
        echo "Container 'dynapps-clean-client' is stopped — restarting it."
        if docker start dynapps-clean-client >/dev/null 2>&1; then
            exec docker exec -it dynapps-clean-client bash -l
        fi
        echo "warn: it would not restart — removing it and starting fresh." >&2
        docker rm -f dynapps-clean-client >/dev/null
    fi

    if ! docker image inspect dynapps-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    BRIDGE="{{justfile_directory()}}/bridge/target/release/dynapps-bridge"
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
        --name dynapps-clean-client \
        --hostname clean-client \
        --add-host host.docker.internal:host-gateway \
        -e DYNAPPS_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        -e CLEAN_CLIENT_ALLOW_STATE=1 \
        -v dynapps-clean-home:/home/tester \
        -v "$BRIDGE:/usr/local/bin/dynapps-bridge:ro" \
        -v "{{justfile_directory()}}/deploy/clean-client/bootstrap.sh:/usr/local/bin/bootstrap.sh:ro" \
        "${PORTS[@]}" \
        dynapps-clean-client:local /usr/local/bin/bootstrap.sh

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

# Drops this clone's sessions and the credentials they stored, so the next run
# redeems a fresh code. ALL=1 signs out every clone on this host.
# Sign out of `just claude` (this repo; ALL=1 for every repo)
claude-reset ALL="0":
    #!/usr/bin/env bash
    set -euo pipefail
    # Scoped to this clone by default: signing every other checkout out because
    # one of them wanted a clean slate is not what anyone means by "reset".
    if [ "{{ALL}}" = "1" ]; then
        FILTER='name=^dynapps-claude-'
        echo "Signing out every repo on this host."
    else
        REPO_SLUG="$(basename "{{justfile_directory()}}" | sed 's|[^A-Za-z0-9]|-|g')"
        REPO_HASH="$(printf '%s' "{{justfile_directory()}}" | sha256sum | cut -c1-8)"
        FILTER="name=^dynapps-claude-${REPO_SLUG}-${REPO_HASH}-"
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
    -docker rm -f dynapps-clean-client 2>/dev/null
    -docker volume rm dynapps-clean-home
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
    if ! docker image inspect dynapps-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi
    BRIDGE="{{justfile_directory()}}/bridge/target/release/dynapps-bridge"
    MOUNTS=(-v "$REPO_ABS:/workspace/project")
    if [ -d "$BRIDGE" ]; then
        echo "ERROR: $BRIDGE is a directory (docker created it from a stale -v mount)." >&2
        echo "       Remove it with: sudo rmdir '$BRIDGE'" >&2
        exit 1
    fi
    if [ -f "$BRIDGE" ] && [ -x "$BRIDGE" ]; then
        MOUNTS+=(-v "$BRIDGE:/usr/local/bin/dynapps-bridge:ro")
    else
        echo "warn: $BRIDGE not found — run 'cd bridge && cargo build --release' for the full flow." >&2
    fi
    if [ "{{PERSIST}}" = "1" ]; then
        MOUNTS+=(-v dynapps-clean-home:/home/tester -e CLEAN_CLIENT_ALLOW_STATE=1)
        echo "State persists in volume 'dynapps-clean-home' — 'just clean-client-reset' wipes it."
    fi
    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it. Plugin OAuth loopback will not work." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi
    exec docker run -it --rm \
        --name dynapps-dev-sandbox \
        --hostname dev-sandbox \
        --add-host host.docker.internal:host-gateway \
        -e DYNAPPS_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        "${MOUNTS[@]}" "${PORTS[@]}" \
        dynapps-clean-client:local

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

# Adopt a published core (on `next`, patch blocks re-commented): bump every
# core pin, point bridge/CORE_REF at its tag, and gate locally
# (migrate + build + clippy). Then `just preflight` and `just release`.
# See docs/RELEASING.md.
#
# LOCAL-ONLY. The migrate is pinned to `--profile local`: a bare `infra db
# migrate` follows the active CLI session, and after `just deploy-check` that
# used to be `production` — 0.51.0 came within one keystroke of migrating the
# live database. A failed migrate stops the bump (no `|| true`). All three
# lockfiles are refreshed, so the core crates cannot drift between them
# (scripts/check-core-crate-versions.sh).
core-bump version:
    @! grep -q '^\[patch\.crates-io\]' Cargo.toml || (echo "ERROR: [patch.crates-io] is active — publish core and re-comment it first" && exit 1)
    scripts/sync-core-version.sh {{version}}
    cargo update -w
    cargo update -w --manifest-path tests/Cargo.toml
    cargo update -w --manifest-path bridge/Cargo.toml
    just db-up
    cargo run --bin systemprompt -- infra db migrate --profile local
    just build
    just clippy
    @echo "core-bump {{version}} complete — set dynapps's version with scripts/sync-release-version.sh X.Y.Z, then: just preflight && just release X.Y.Z"

# Pin bridge/CORE_REF to a core `next` commit while [patch.crates-io] is
# active (the sibling checkout's HEAD by default). CI checks that ref out.
core-pin REF="":
    #!/usr/bin/env bash
    set -euo pipefail
    ref="{{REF}}"
    core="${CORE_REPO:-../systemprompt-core}"
    [ -n "$ref" ] || ref="$(git -C "$core" rev-parse HEAD)"
    # check-core-ref.sh accepts only a vX.Y.Z tag or a full 40-char SHA, so expand abbreviations.
    case "$ref" in v[0-9]*) ;; *) ref="$(git -C "$core" rev-parse --verify "${ref}^{commit}")" ;; esac
    printf '%s\n' "$ref" > bridge/CORE_REF
    echo "bridge/CORE_REF = $ref"

# Promote the exact green next-push candidate through a frozen PR onto main.
# Repeat after the PR proof finishes; release.yml creates the tag and artifacts.
release version:
    bash scripts/release.sh {{version}}

# Deploy a published release to production from a clean worktree of
# origin/main — never from this tree, which may hold a peer's work or an
# active [patch.crates-io]. Binaries and downloads come from the vVERSION
# release (strict: verified against its SHA256SUMS, no host fallback), web/dist
# is rendered inside the worktree, the Fly image digest must move, and /health
# is watched for five minutes — 0.53.0 went healthy at +1 min and 503 from
# +2 to +5 while the shared database stalled. Leaves the worktree in place on
# failure for inspection.
deploy-release VERSION:
    bash scripts/deploy-release.sh {{VERSION}}

# Opt-in release HTTP benchmark; requires a seeded disposable *_test database.
# Uses scripts/benchmark-dashboard.py. Does not start or restart the live server.
dashboard-benchmark:
    @scripts/build-coordinator.sh run dashboard-benchmark "release" -- cargo nextest run --manifest-path tests/Cargo.toml --release -p admin-contract-tests --run-ignored only -E 'test(benchmark_dashboard_http)' --no-capture

# Selected core test packages, preserving core serial groups and Dynapps coordination.
test-core-feedback PACKAGES FILTER="all()":
    @scripts/test-core-feedback.sh "$@"

# Regenerate query metadata for one changed crate against the existing schema.
# Unlike prepare, this does not run migrations or remove other cache entries;
# useful when the installed CLI cannot yet load newly added configuration.
prepare-query-cache package:
    @scripts/build-coordinator.sh run prepare-query-cache "{{package}}" -- {{just_executable()}} _prepare-query-cache-uncoordinated {{package}}

_prepare-query-cache-uncoordinated package:
    #!/usr/bin/env bash
    set -euo pipefail
    task_cache_dir=$(mktemp -d "{{justfile_directory()}}/.build/query-cache.XXXXXX")
    trap 'rm -rf "$task_cache_dir"' EXIT
    task_database_url=$(jq -er '.database_url' .systemprompt/profiles/local/secrets.json)
    DATABASE_URL="$task_database_url" SQLX_OFFLINE=false SQLX_OFFLINE_DIR="$task_cache_dir" cargo check -p {{package}}
    for task_query_file in "$task_cache_dir"/query-*.json; do
        [ -f "$task_query_file" ] || continue
        cp "$task_query_file" .sqlx/
    done
