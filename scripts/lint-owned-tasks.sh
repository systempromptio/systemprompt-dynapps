#!/usr/bin/env bash
# Core's lint-owned-tasks gate over this install's sources; the rule lives in
# ../systemprompt-core/scripts/lint-owned-tasks.sh at bridge/CORE_REF (see core-gate.sh).
# scripts/owned-tasks-baseline.txt lists the files whose detached spawns are
# baselined; core matches allowlist entries against the absolute paths it
# prints, so the repo-relative entries are resolved here, and an entry whose
# file is gone fails the gate.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
baseline="$root/scripts/owned-tasks-baseline.txt"
mkdir -p "$root/target"
resolved="$(mktemp -p "$root/target" owned-tasks.XXXXXX)"
trap 'rm -f "$resolved"' EXIT
while IFS= read -r line; do
    entry="${line%%#*}"
    entry="$(printf '%s' "$entry" | sed 's/[[:space:]]*$//')"
    [ -n "$entry" ] || continue
    [ -f "$root/$entry" ] || { echo "lint-owned-tasks: stale baseline entry (file missing): $entry" >&2; exit 1; }
    printf '%s\n' "$root/$entry" >> "$resolved"
done < "$baseline"
bash "$root/scripts/core-gate.sh" lint-owned-tasks.sh --allowlist "${resolved#"$root"/}" extensions src bridge/src
