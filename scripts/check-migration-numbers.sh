#!/usr/bin/env bash
# Gate: a migration file may never reuse a number an established database has
# already applied.
#
# The web migration chain was retired in ab902af2 (files 002-027 deleted,
# declarative schema + seeds became the bootstrap), but every database that ran
# those migrations still carries a ledger row for each number. Refilling a
# retired slot makes the recorded checksum disagree with the file, and the
# migrator refuses to boot — the whole deployment crash-loops on a mismatch that
# names a migration nobody recognises.
#
# A slot is spent whether or not its file is still in the tree, so deleting a
# migration is only half the job: the number has to be recorded here too.
# a6a57f3f deleted 034_knowledge_bank.sql without doing that, which left 034
# looking free to `ls` while every database from v0.42.0 on had it applied.
#
# Each extension records its spent slots in schema/migrations/.retired-through:
#
#   # comments and blank lines are ignored
#   027            <- high-water mark, exactly one bare number: all slots <= it
#   burned 034     <- an isolated spent slot above the mark, repeatable
#
# New migrations take the next number above the mark that is not burned. The
# mark is deliberately parsed line by line: reading the whole file through
# `tr -cd '0-9'` turns "027 034" into 27034 and silently retires the entire
# tree, so more than one bare number is a hard error rather than a guess.
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
while IFS= read -r marker; do
    dir=$(dirname "$marker")
    ext=$(printf '%s' "$dir" | sed -E 's|^extensions/([^/]+)/.*|\1|')

    retired=""
    burned=" "
    marker_bad=0
    while IFS= read -r line || [ -n "$line" ]; do
        line="${line%%#*}"
        line=$(printf '%s' "$line" | tr -d '[:space:]')
        [ -n "$line" ] || continue
        case "$line" in
            burned[0-9]*)
                n="${line#burned}"
                case "$n" in
                    *[!0-9]*) echo "$marker: malformed burned slot '$line'"; marker_bad=1 ;;
                    *) burned="$burned$((10#$n)) " ;;
                esac
                ;;
            *[!0-9]*)
                echo "$marker: unrecognised line '$line'"
                marker_bad=1
                ;;
            *)
                if [ -n "$retired" ]; then
                    echo "$marker: more than one high-water mark ('$retired' then '$line')"
                    echo "  One bare number only; isolated slots go on 'burned <n>' lines."
                    marker_bad=1
                else
                    retired=$((10#$line))
                fi
                ;;
        esac
    done < "$marker"

    if [ "$marker_bad" -ne 0 ]; then fail=1; continue; fi
    [ -n "$retired" ] || { echo "$marker: no number recorded"; fail=1; continue; }

    for f in "$dir"/[0-9]*.sql; do
        [ -e "$f" ] || continue
        n=$(basename "$f" | sed -E 's/^0*([0-9]+)_.*/\1/')
        n=$((10#$n))
        if [ "$n" -le "$retired" ]; then
            echo "$ext: $(basename "$f") reuses migration number $n, retired through $retired"
            echo "  Renumber it above $retired — established databases have already spent that slot."
            fail=1
        elif [ "${burned#* $n }" != "$burned" ]; then
            echo "$ext: $(basename "$f") reuses burned migration number $n"
            echo "  That slot shipped and was later deleted; established databases have it applied."
            fail=1
        fi
    done
done < <(find extensions -path '*/schema/migrations/.retired-through' | sort)

if [ "$fail" -eq 0 ]; then
    echo "check-migration-numbers: ok"
fi
exit "$fail"
