#!/usr/bin/env bash
# bridge/CORE_REF names the systemprompt-core ref CI checks out beside this
# repo: bridge/ depends on systemprompt-bridge by path, and the active
# [patch.crates-io] on `next` resolves through it. Two rules keep it honest:
#
#   * the file exists and holds exactly one ref (tag or 40-char SHA);
#   * when [patch.crates-io] is COMMENTED (the `main` steady state), the ref
#     must be the tag of the published core the pins name, `v<pin>` — so the
#     bridge is proven against the same core the server crates come from.
#
# While the patch is active the ref is a SHA on core `next`. Two lockfile rules
# then prove the patch is live rather than silently dropped by cargo:
#
#   * no tracked Cargo.lock carries a `[[patch.unused]]` block (cargo writes
#     one when the pins and the sibling's version disagree);
#   * the root pin equals `version =` in ../systemprompt-core/Cargo.toml when
#     that sibling is present (SYSTEMPROMPT_CORE_DIR overrides the location).
#
# In published-core mode the mirror rule holds: no `systemprompt*` package in a
# tracked Cargo.lock resolves from a `path+` or `git+` source, which is what an
# escaped local patch looks like once committed.
#
# A bridge-only hotfix ships without a core release: `v<pin>-bridge.N` is a
# core tag that differs from `v<pin>` only under bin/bridge, so the server
# crates are still exactly the published ones. Where a core checkout with both
# tags is at hand (CI checks it out beside this repo) that is verified.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FILE="$REPO_ROOT/bridge/CORE_REF"
CORE_DIR="${SYSTEMPROMPT_CORE_DIR:-$REPO_ROOT/../systemprompt-core}"
REGISTRY='registry+https://github.com/rust-lang/crates.io-index'

tracked_lockfiles() {
    git -C "$REPO_ROOT" ls-files '*Cargo.lock' 'Cargo.lock'
}

patch_active_rules() {
    local pin="$1" lock unused core_version
    while IFS= read -r lock; do
        [ -n "$lock" ] || continue
        unused="$(grep -n '^\[\[patch\.unused\]\]' "$REPO_ROOT/$lock" || true)"
        [ -z "$unused" ] || {
            echo "check-core-ref: $lock carries [[patch.unused]] — cargo dropped the [patch.crates-io]" >&2
            echo "  and resolved systemprompt from crates.io. Align the pins with ../systemprompt-core" >&2
            echo "  (scripts/sync-core-version.sh <core-version>) and re-run cargo update -w." >&2
            exit 1
        }
    done < <(tracked_lockfiles)
    [ -f "$CORE_DIR/Cargo.toml" ] || return 0
    core_version="$(sed -n 's/^version = "\([0-9.]*\)".*/\1/p' "$CORE_DIR/Cargo.toml" | head -1)"
    [ -n "$core_version" ] || { echo "check-core-ref: could not read version from $CORE_DIR/Cargo.toml" >&2; exit 1; }
    [ "$pin" = "$core_version" ] || {
        echo "check-core-ref: [patch.crates-io] is active but Cargo.toml pins systemprompt $pin" >&2
        echo "  while $CORE_DIR/Cargo.toml is at $core_version; a 0.x pin only matches its own minor," >&2
        echo "  so the patch is inert. Run scripts/sync-core-version.sh $core_version." >&2
        exit 1
    }
}

published_core_rules() {
    local lock findings fail=0
    while IFS= read -r lock; do
        [ -n "$lock" ] || continue
        findings=$(awk -v registry="$REGISTRY" '
function flush() {
    if (inpkg && name ~ /^systemprompt/ && source != "" && source != registry) {
        printf "  %s:%d\n    %s resolves from %s\n", FILENAME, start, name, source
    }
    inpkg = 0
}
/^\[\[package\]\]/ { flush(); name = ""; source = ""; start = FNR; inpkg = 1; next }
inpkg && /^name = / { name = $0; sub(/^name = "/, "", name); sub(/"$/, "", name) }
inpkg && /^source = / { source = $0; sub(/^source = "/, "", source); sub(/"$/, "", source) }
/^$/ { flush() }
END { flush() }
' "$REPO_ROOT/$lock")
        [ -z "$findings" ] || { printf '%s\n' "$findings" >&2; fail=1; }
    done < <(tracked_lockfiles)
    [ "$fail" -eq 0 ] || {
        echo "check-core-ref: [patch.crates-io] is commented but a tracked lockfile resolves a" >&2
        echo "  systemprompt crate from a path+ or git+ source; drop the local patch and re-resolve." >&2
        exit 1
    }
}

read_pin() {
    sed -n 's/^systemprompt = { version = "\([0-9.]*\)".*/\1/p' "$REPO_ROOT/Cargo.toml" | head -1
}

[ -f "$FILE" ] || { echo "check-core-ref: $FILE is missing" >&2; exit 1; }
ref="$(tr -d '[:space:]' < "$FILE")"
[ -n "$ref" ] || { echo "check-core-ref: $FILE is empty" >&2; exit 1; }
[ "$(wc -l < "$FILE")" -le 1 ] || { echo "check-core-ref: $FILE must hold a single ref" >&2; exit 1; }

case "$ref" in
    v[0-9]*.[0-9]*.[0-9]*-bridge.[0-9]*) ;;
    v[0-9]*.[0-9]*.[0-9]*) ;;
    *)
        [[ "$ref" =~ ^[0-9a-f]{40}$ ]] || {
            echo "check-core-ref: '$ref' is neither a vX.Y.Z tag nor a 40-char SHA" >&2; exit 1; }
        ;;
esac

pin="$(read_pin)"
[ -n "$pin" ] || { echo "check-core-ref: could not read the systemprompt pin from Cargo.toml" >&2; exit 1; }

if grep -qE '^\[patch\.crates-io\]' "$REPO_ROOT/Cargo.toml"; then
    patch_active_rules "$pin"
else
    published_core_rules
    if [ "${ref%-bridge.*}" = "v$pin" ] && [ "$ref" != "v$pin" ]; then
        core="$CORE_DIR"
        if git -C "$core" rev-parse -q --verify "$ref^{commit}" >/dev/null 2>&1 \
            && git -C "$core" rev-parse -q --verify "v$pin^{commit}" >/dev/null 2>&1; then
            outside="$(git -C "$core" diff --name-only "v$pin" "$ref" -- . ':(exclude)bin/bridge')"
            [ -z "$outside" ] || {
                echo "check-core-ref: $ref changes core outside bin/bridge:" >&2
                echo "$outside" >&2
                exit 1
            }
        fi
        echo "check-core-ref: $ref ok (bridge hotfix of v$pin)"
        exit 0
    fi
    [ "$ref" = "v$pin" ] || {
        echo "check-core-ref: [patch.crates-io] is commented (published-core mode) but" >&2
        echo "  bridge/CORE_REF = $ref while Cargo.toml pins systemprompt $pin." >&2
        echo "  Set bridge/CORE_REF to v$pin so the bridge builds against the released core." >&2
        exit 1
    }
fi
echo "check-core-ref: $ref ok"
