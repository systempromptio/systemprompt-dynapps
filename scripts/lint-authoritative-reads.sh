#!/usr/bin/env bash
# Security-critical lookups and revocations run on the primary, never a
# replica. `database_url` may name a standby that lags the primary by seconds;
# a session revoked or a login code consumed on the primary and read on the
# replica a moment later is still live there. This repo's identity repositories
# take the pool as an argument, so the decision sits at the call sites:
#
#   * every identity handler that calls a sensitive repository function is
#     mounted in a route group that extensions/web/src/router/admin/api.rs
#     merges over `write_pool`, never `read_pool`;
#   * every other call site lives in a file that takes its pool from
#     `write_pool` (or `db.write`), forwards a caller's pool (PASSTHROUGH, whose
#     own fn is itself sensitive), or is listed in ALLOWLIST with its reason.
set -uo pipefail
cd "$(dirname "$0")/.."

ROUTES=extensions/web/admin/identity/src/routes/api.rs
ROUTER=extensions/web/src/router/admin/api.rs
REPO_DIR=extensions/web/admin/identity/src/repositories

SENSITIVE_FNS=(
    revoke_user_access
    revoke_access_by_email
    revoke_signin_session
    revoke_all_signin_sessions
    revoke_cookie_session
    sign_out
    consume_dev_login_code
    find_active_user_id_by_login
    insert_dev_login_code
)

# Files whose sensitive fn only forwards the pool its caller handed it; the
# caller is the checked site.
PASSTHROUGH=(
    extensions/web/admin/identity/src/sign_out.rs
    extensions/web/src/router/sign_out.rs
)

# file:function pairs whose pool is not provably the write pool. Every entry
# must exist and carries its reason here:
#   dev-login CLI: refuses every profile but a development, non-cloud one, where
#   the read and write pools are the same database.
ALLOWLIST=(
    "extensions/cli/dev-login/src/main.rs:main"
)
WRITE_POOL_RE='write_pool\b|\bdb\.write\b'

fail=0
for f in "$ROUTES" "$ROUTER"; do
    [ -f "$f" ] || { echo "lint-authoritative-reads: missing $f"; fail=1; }
done
[ "$fail" -eq 0 ] || exit 1

group_of_handler() {
    awk -v h="$1" '
        /^pub fn [a-z_]+\(\) -> Router/ { match($0, /fn [a-z_]+/); group = substr($0, RSTART + 3, RLENGTH - 3) }
        index($0, h) { print group }
    ' "$ROUTES" | sort -u
}

router_fn_pool() {
    awk -v g="$1" '
        /^(pub\(super\) |pub )?fn [a-z_]+\(/ { name = $0; sub(/^.*fn /, "", name); sub(/\(.*/, "", name); pool = ""; hit = 0 }
        index($0, "identity_routes::" g "()") { hit = 1 }
        /with_state\(Arc::clone\((read|write)_pool\)\)/ { match($0, /(read|write)_pool/); pool = substr($0, RSTART, RLENGTH); if (hit) { print name ":" pool; hit = 0 } }
    ' "$ROUTER"
}

pattern="$(printf '%s|' "${SENSITIVE_FNS[@]}")"; pattern="(${pattern%|})\\("
call_sites() {
    git ls-files 'extensions/**/*.rs' | xargs grep -nE "$pattern" 2>/dev/null \
        | grep -vE '^[^:]+:[0-9]+:\s*(//|use |pub(\([a-z]+\))? (async )?fn)'
}
fn_at() {
    awk -v n="$2" 'NR <= n && /fn [A-Za-z_][A-Za-z0-9_]*/ { match($0, /fn [A-Za-z_][A-Za-z0-9_]*/); f = substr($0, RSTART + 3, RLENGTH - 3) } NR == n { print f }' "$1"
}

handlers=()
others=()
while IFS=: read -r file lineno _; do
    case "$file" in "$REPO_DIR"/*) continue ;; esac
    fn="$(fn_at "$file" "$lineno")"
    case "$fn" in
        *_handler) handlers+=("$fn") ;;
        *) others+=("$file:$lineno:$fn") ;;
    esac
done < <(call_sites)
mapfile -t handlers < <(printf '%s\n' "${handlers[@]}" | sort -u)
[ "${#handlers[@]}" -gt 0 ] || { echo "lint-authoritative-reads: no identity handler calls a sensitive repository fn — scope broken?"; exit 1; }

for handler in "${handlers[@]}"; do
    groups="$(group_of_handler "$handler")"
    [ -n "$groups" ] || { echo "lint-authoritative-reads: $handler is not mounted in $ROUTES"; fail=1; continue; }
    for group in $groups; do
        mounted="$(router_fn_pool "$group")"
        [ -n "$mounted" ] || { echo "lint-authoritative-reads: route group $group ($handler) is not mounted by $ROUTER with a read_pool/write_pool state"; fail=1; continue; }
        case "$mounted" in
            *:write_pool) ;;
            *) echo "lint-authoritative-reads: $handler is in $group, mounted over the read pool (${mounted%%:*}); revocations must use write_pool"; fail=1 ;;
        esac
    done
done

allowed() {
    local key="$1"
    for entry in "${ALLOWLIST[@]}"; do [ "$entry" = "$key" ] && return 0; done
    return 1
}
for entry in "${ALLOWLIST[@]}"; do
    file="${entry%%:*}"; fn="${entry##*:}"
    grep -qE "fn ${fn}\b" "$file" 2>/dev/null || { echo "lint-authoritative-reads: stale allowlist entry $entry"; fail=1; }
done

passthrough() {
    for entry in "${PASSTHROUGH[@]}"; do [ "$entry" = "$1" ] && return 0; done
    return 1
}
for entry in "${PASSTHROUGH[@]}"; do
    grep -qE "pub(\([a-z]+\))? (async )?fn ($(printf '%s|' "${SENSITIVE_FNS[@]}" | sed 's/|$//'))\(" "$entry" 2>/dev/null \
        || { echo "lint-authoritative-reads: PASSTHROUGH $entry defines no sensitive fn of its own; check it as a call site instead"; fail=1; }
done

for site in "${others[@]}"; do
    file="${site%%:*}"; rest="${site#*:}"; lineno="${rest%%:*}"; fn="${rest#*:}"
    passthrough "$file" && continue
    grep -qE "$WRITE_POOL_RE" "$file" && continue
    allowed "$file:$fn" && continue
    echo "$file:$lineno: calls a revocation/login repository fn (in $fn) from a file that never takes write_pool; pass the write pool or allowlist it with a reason"
    fail=1
done

[ "$fail" -eq 0 ] || exit 1
echo "lint-authoritative-reads: ok (${#handlers[@]} handlers on write_pool, ${#others[@]} other call sites, ${#SENSITIVE_FNS[@]} repository fns)"
