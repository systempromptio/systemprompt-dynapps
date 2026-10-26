#!/usr/bin/env bash
# Service state is owned and injected, never a process-wide static.
#
# A `static NAME: OnceLock<..>` / `LazyLock<..>` of pure data — a compiled
# regex, a constant table — is fine. One that holds a loaded config, a cache,
# a pool, a client, a scanner, or a `Result` of loading something, or whose
# initialiser reads a file, the environment or a bootstrap singleton, is a
# hidden dependency: it pins the first caller's view for the process, cannot be
# invalidated or replaced, and makes every test share one state. Such state is
# loaded once at a composition root (the extension value, `ManagedState`, a
# router layer) and handed to its consumers.
#
# Takes source roots as positional arguments; defaults to this repo's. Each
# allowlist entry names the one constraint that keeps the static.
set -euo pipefail

cd "$(dirname "$0")/.."

roots=("$@")
if [ ${#roots[@]} -eq 0 ]; then
    roots=(extensions src bridge/src)
fi

allowlist=(
    # Core's authz hook, gateway guard and subject-provider factories receive
    # only a pool, so the per-database registry is the one slot they can all
    # reach; it is keyed by database and built once per key.
    extensions/web/shared/admin-core/src/authz/mod.rs
    # The services-sync caches are owned by the sync plane work and are
    # outside this sweep.
    extensions/web/shared/admin-core/src/repositories/sync/
)

# A static whose declared type names service state.
type_pattern='static[[:space:]]+[A-Z_0-9]+[[:space:]]*:[[:space:]]*(std::sync::)?(OnceLock|LazyLock)<[^=]*(Result|Pool|Client|Config|Cache|Scanner|Policy|Registry|Store|Tx|Sender|Receiver)[^=]*=' 
# An initialiser that performs I/O or reaches a bootstrap singleton.
init_pattern='(OnceLock|LazyLock)::new\(\|\|[^;]*(::get\(\)|read_to_string|read_dir|env::var|::load\()'

existing=()
for root in "${roots[@]}"; do
    [ -d "$root" ] && existing+=("$root")
done

hits=$(
    {
        rg -nU --glob '*.rs' --glob '!**/target/**' "$type_pattern" "${existing[@]}" 2>/dev/null || true
        rg -nU --multiline-dotall --glob '*.rs' --glob '!**/target/**' "$init_pattern" "${existing[@]}" 2>/dev/null || true
    } | sort -u
)

for allowed in "${allowlist[@]+"${allowlist[@]}"}"; do
    hits=$(echo "$hits" | grep -v "^${allowed}" || true)
done

if [ -n "$hits" ]; then
    echo "✗ process-wide service state behind a OnceLock/LazyLock:" >&2
    echo "$hits" | sed 's/^/    /' >&2
    echo >&2
    echo "Load it once at the composition root and inject it (extension value," >&2
    echo "ManagedState, router Extension). Pure data (regex, const tables) is fine." >&2
    exit 1
fi
echo "✓ no OnceLock/LazyLock holds service state"
