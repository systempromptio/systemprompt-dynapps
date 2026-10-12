#!/usr/bin/env bash
# Gate: a schema file deleted from git leaves no table behind. For every
# `extensions/**/schema/*.sql` (not a migration) deleted since the ladder
# floor was recorded, each `CREATE TABLE` it declared must be named by a
# `DROP TABLE` in a tracked migration under extensions/.
#
# Why: in the tree this fork started from, two crate deletions shipped with
# no drop migration, and a later production dump still carried their 19
# tables, a scheduled_jobs row and a dead migration ledger — schema that no
# code declares, that every fresh install lacks, and that nothing at boot
# reports. The boot-time undeclared-relation check (core SchemaDoctor)
# catches this on a running instance; this gate catches it in the PR.
#
# Why the scan starts at the floor: history before the first Dynapps rung is
# Astound's — crates and tables no Dynapps database ever held, retired with no
# migration this fork carries. The floor rung is the oldest database the
# ladder upgrades, so a deletion older than the rung cannot leave anything
# behind here. The range needs full history (CI checks out with
# `fetch-depth: 0`); a shallow clone fails loudly rather than scanning nothing.
#
# A migration may drop a table under any name form: `DROP TABLE t`,
# `DROP TABLE IF EXISTS t`, with or without `public.`. A table re-created
# under the same name by a surviving schema file (here or in core) is not
# dead and is skipped.
set -euo pipefail

cd "$(dirname "$0")/.."

# A table that moved into core is still declared. Core is the sibling
# checkout when the patch is active, else the published crates in the
# cargo registry (what `main` builds against).
core_dirs=()
if [ -d ../systemprompt-core/crates ]; then
    core_dirs+=(../systemprompt-core/crates)
else
    for d in "${CARGO_HOME:-$HOME/.cargo}"/registry/src/*/systemprompt-*/; do
        [ -d "$d" ] && core_dirs+=("$d")
    done
fi

floor="$(sed -n 's/^FLOOR="\(.*\)"$/\1/p' scripts/check-schema-baseline.sh)"
[ -n "$floor" ] || { echo "check-dropped-schema: cannot read FLOOR from scripts/check-schema-baseline.sh" >&2; exit 1; }
rung="tests/fixtures/schema/release-baseline-$floor.sql"
since="$(git log --diff-filter=A --format='%H' -n 1 -- "$rung" || true)"
if [ -z "$since" ] && [ -f "$rung" ]; then
    # The rung is being recorded in this very change set and is not committed
    # yet: nothing can have been deleted since it, so the range is empty.
    since=HEAD
fi
[ -n "$since" ] || { echo "check-dropped-schema: no commit adds $rung (shallow clone, or the rung was never committed)" >&2; exit 1; }

declared_now="$({ grep -rhoiE 'CREATE TABLE (IF NOT EXISTS )?[a-z_.]+' extensions "${core_dirs[@]}" --include='*.sql' || true; } \
    | awk '{print tolower($NF)}' | sed 's/^public\.//' | sort -u)"
dropped="$({ grep -rhoiE 'DROP TABLE (IF EXISTS )?[a-z_.]+' extensions --include='*.sql' || true; } \
    | awk '{print tolower($NF)}' | sed 's/^public\.//' | sort -u)"

status=0
while IFS=$'\t' read -r commit path; do
    [ -n "$path" ] || continue
    case "$path" in */migrations/*|*/migrations-pending/*|*/seeds/*) continue ;; esac
    tables="$( { git show "${commit}^:${path}" 2>/dev/null || true; } \
        | { grep -oiE 'CREATE TABLE (IF NOT EXISTS )?[a-z_.]+' || true; } \
        | awk '{print tolower($NF)}' | sed 's/^public\.//' | sort -u)"
    for t in $tables; do
        if grep -qx "$t" <<<"$declared_now"; then continue; fi
        if grep -qx "$t" <<<"$dropped"; then continue; fi
        echo "check-dropped-schema: $path (deleted in ${commit:0:8}) declared table '$t' and no migration drops it" >&2
        status=1
    done
done < <(git log --diff-filter=D --name-only --format='%H' "$since..HEAD" -- 'extensions/**/schema/*.sql' 'extensions/**/schema/**/*.sql' \
    | awk 'NF==1 && /^[0-9a-f]{40}$/ {c=$1; next} NF {print c "\t" $0}')

if [ "$status" -ne 0 ]; then
    echo "check-dropped-schema: add DROP TABLE IF EXISTS … CASCADE to a migration under extensions/web/schema/migrations/" >&2
fi
exit "$status"
