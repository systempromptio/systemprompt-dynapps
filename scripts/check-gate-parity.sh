#!/usr/bin/env bash
# Every source gate core runs (../systemprompt-core/scripts/check-gates.txt at
# bridge/CORE_REF) has a decided disposition here: a wrapper scripts/<gate>.sh
# that execs core's script, or a line in scripts/core-gate-map.txt naming the
# install script or recipe that holds the rule, or an `n/a:` reason. A new core
# gate turns this red until someone decides; a map line for a gate core no
# longer runs, or naming a script this repo does not have, is stale and fails.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CORE_DIR="${SYSTEMPROMPT_CORE_DIR:-$REPO_ROOT/../systemprompt-core}"
MAP="$REPO_ROOT/scripts/core-gate-map.txt"
LIST="$CORE_DIR/scripts/check-gates.txt"

[ -d "$CORE_DIR" ] || {
    echo "check-gate-parity: $CORE_DIR not found; CI provides it via .github/actions/core-checkout" >&2
    exit 2
}
[ -f "$LIST" ] || { echo "check-gate-parity: $LIST missing" >&2; exit 1; }
[ -f "$MAP" ] || { echo "check-gate-parity: $MAP missing" >&2; exit 1; }

declare -A disposition
fail=0
while IFS= read -r raw; do
    line="${raw%%#*}"
    [ -n "$(echo "$line" | tr -d '[:space:]')" ] || continue
    gate="$(echo "$line" | awk '{print $1}')"
    rest="$(echo "$line" | sed -E 's/^[^[:space:]]+[[:space:]]+//; s/[[:space:]]+$//')"
    [ -z "${disposition[$gate]:-}" ] || { echo "check-gate-parity: $gate listed twice in core-gate-map.txt" >&2; fail=1; }
    case "$rest" in
        wrapper)
            [ -f "$REPO_ROOT/scripts/$gate.sh" ] || { echo "check-gate-parity: $gate is 'wrapper' but scripts/$gate.sh does not exist" >&2; fail=1; }
            grep -q 'core-gate\.sh' "$REPO_ROOT/scripts/$gate.sh" 2>/dev/null \
                || { echo "check-gate-parity: scripts/$gate.sh does not exec core-gate.sh" >&2; fail=1; }
            ;;
        install:justfile:*)
            recipe="${rest#install:justfile:}"
            grep -qE "^${recipe}:" "$REPO_ROOT/justfile" || { echo "check-gate-parity: $gate names justfile recipe $recipe, which does not exist" >&2; fail=1; }
            ;;
        install:*)
            script="${rest#install:}"
            [ -f "$REPO_ROOT/scripts/$script" ] || { echo "check-gate-parity: $gate names scripts/$script, which does not exist" >&2; fail=1; }
            ;;
        "n/a: "?*) ;;
        *)
            echo "check-gate-parity: $gate has an unreadable disposition '$rest' (wrapper | install:<script> | install:justfile:<recipe> | n/a: <reason>)" >&2
            fail=1 ;;
    esac
    disposition["$gate"]="$rest"
done < "$MAP"

declare -A core_gates
while IFS= read -r gate; do
    gate="$(echo "$gate" | tr -d '[:space:]')"
    [ -n "$gate" ] || continue
    core_gates["$gate"]=1
    if [ -z "${disposition[$gate]:-}" ]; then
        if [ -f "$REPO_ROOT/scripts/$gate.sh" ]; then
            disposition["$gate"]=wrapper
        else
            echo "check-gate-parity: core gate '$gate' has neither scripts/$gate.sh nor a line in scripts/core-gate-map.txt — decide it" >&2
            fail=1
        fi
    fi
done < "$LIST"

for gate in "${!disposition[@]}"; do
    [ -n "${core_gates[$gate]:-}" ] || { echo "check-gate-parity: core no longer runs '$gate'; drop its line from core-gate-map.txt" >&2; fail=1; }
done

[ "$fail" -eq 0 ] || exit 1
echo "check-gate-parity: ${#core_gates[@]} core gates decided against core $(tr -d '[:space:]' < "$REPO_ROOT/bridge/CORE_REF")"
