#!/usr/bin/env bash
# Reject dead config and diary entries in services/** YAML comments.
#
# Two rules, both over tracked and untracked `services/**/*.y{a,}ml`:
#
#   * no commented-out YAML mapping longer than 5 consecutive lines. A comment
#     line that parses as YAML (`# key: value`, `# - item`) with a value of at
#     most 3 words is disabled config (`# note: a sentence of prose` is not);
#     a run of more than 5 is a whole block kept "in case", which the loader
#     never sees and nobody re-validates. Delete it — git holds the old block.
#     Short runs stay legal: a one-line example beside a key is documentation.
#   * no date (YYYY-MM-DD) in a comment. A dated comment narrates when a
#     setting changed, and goes stale the day after; state what the setting
#     does now. Dates in values are untouched.
#
# Exemption: put `# doc-ok: <reason>` on the line above a dated comment.
# Generated files (a `GENERATED` marker in the first 3 lines) are skipped, and
# so are `# flag: …` lines: both are written by import scripts, not by hand.
set -uo pipefail

cd "$(dirname "$0")/.."

MAX_RUN=5

FOUND=$(git ls-files -co --exclude-standard 'services/*.yaml' 'services/**/*.yaml' \
        'services/*.yml' 'services/**/*.yml' | sort -u | while IFS= read -r file; do
    [ -f "$file" ] || continue
    head -3 "$file" | grep -q 'GENERATED' && continue
    awk -v max_run="$MAX_RUN" '
        function flush() {
            if (run > max_run) {
                print FILENAME ":" run_start ": " run " consecutive lines of commented-out YAML (cap " max_run ") — delete the block"
            }
            run = 0
        }
        {
            is_comment = ($0 ~ /^[[:space:]]*#/)
            if ($0 ~ /^[[:space:]]*#[[:space:]]*flag:/) { flush(); prev = $0; next }
            body = $0
            sub(/^[[:space:]]*#[[:space:]]*/, "", body)
            looks_yaml = 0
            if (body ~ /^(-[[:space:]]+)?["'\'']?[A-Za-z0-9_.\/-]+["'\'']?:([[:space:]]|$)/ || body ~ /^-[[:space:]]+[^[:space:]]/) {
                value = body
                sub(/^(-[[:space:]]+)?["'\'']?[A-Za-z0-9_.\/-]+["'\'']?:[[:space:]]*/, "", value)
                looks_yaml = (split(value, tok, /[[:space:]]+/) <= 3)
            }
            if (is_comment && looks_yaml) {
                if (run == 0) run_start = FNR
                run++
            } else if (!(is_comment && $0 ~ /^[[:space:]]*#[[:space:]]*$/ && run > 0)) {
                flush()
            }

            comment = ""
            if (is_comment) {
                comment = $0
            } else if (match($0, /[[:space:]]#[[:space:]]/)) {
                comment = substr($0, RSTART)
            }
            if (comment ~ /20[0-9][0-9]-[0-9][0-9]-[0-9][0-9]/ && prev !~ /doc-ok:/) {
                print FILENAME ":" FNR ": date in a comment — state the current behaviour; history lives in git"
            }
            prev = $0
        }
        END { flush() }
    ' "$file"
done)

if [ -z "$FOUND" ]; then
    echo "check-yaml-comments: OK (no commented-out blocks or dated comments in services/)"
    exit 0
fi

echo "check-yaml-comments: services/ YAML comments must describe current config only."
echo ""
echo "$FOUND"
exit 1
