#!/usr/bin/env bash
# Fail unless every config plane has been applied to the database.
#
# Core 0.73 boots no plane: access control, governance and gateway policies
# reach the database only through `systemprompt admin config apply`, and a
# plane that was never applied passes requests ungoverned (profile
# `governance.unapplied_planes: warn`) or refuses them (`deny`). Setup and
# deploy run the apply and then this check, so a skipped apply stops the
# pipeline instead of leaving a node quietly ungoverned.
#
#   scripts/check-config-applied.sh <systemprompt-binary> [profile-name]
#
# macOS + Linux compatible; needs jq.
set -euo pipefail

BIN="${1:?usage: check-config-applied.sh <systemprompt-binary> [profile-name]}"
PROFILE="${2:-}"

args=(admin config planes list --json)
if [ -n "$PROFILE" ]; then
    args+=(--profile "$PROFILE")
fi

if ! listing="$("$BIN" "${args[@]}" 2>/dev/null)"; then
    echo "check-config-applied: 'admin config planes list' failed${PROFILE:+ for profile $PROFILE}" >&2
    exit 1
fi

unapplied="$(printf '%s' "$listing" | jq -r '.items[] | select(.applied != "yes") | .plane')"
if [ -n "$unapplied" ]; then
    echo "check-config-applied: never applied: $(printf '%s' "$unapplied" | tr '\n' ' ')" >&2
    echo "  run: systemprompt admin config apply --reason \"<why>\"${PROFILE:+ --profile $PROFILE}" >&2
    exit 1
fi
echo "check-config-applied: every config plane is applied${PROFILE:+ ($PROFILE)}"
