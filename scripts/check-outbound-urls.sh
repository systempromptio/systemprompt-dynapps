#!/usr/bin/env bash
# Every outbound HTTP request passes its destination through core's SSRF
# guard. A file under extensions/ or src/ that sends a request on a reqwest
# client (`http.get(..)`, `self.client.post(..)`, `Client::builder()..post(..)`)
# must itself call `validate_outbound_url` or `validate_outbound_url_with_trust`
# (directly or through admin-core's `validate_mesh_url`), so the admission
# decision sits beside the send and a new request site cannot skip it.
#
# The allowlist is empty by design: a destination that must be reached
# without the guard is a design decision for review, not a line here.
#
# Takes source roots as positional arguments; defaults to this repo's.
set -euo pipefail

cd "$(dirname "$0")/.."

roots=("$@")
if [ ${#roots[@]} -eq 0 ]; then
    roots=(extensions src)
fi

allowlist=()

send_pattern='(^|[^a-zA-Z0-9_])(http|client|self\.http|self\.client|builder|\.build\(\)\?)\s*\n?\s*\.\s*(get|post|put|patch|delete|head|request)\('
guard_pattern='validate_outbound_url(_with_trust)?\(|validate_mesh_url\('

existing=()
for root in "${roots[@]}"; do
    [ -d "$root" ] && existing+=("$root")
done

status=0
while IFS= read -r file; do
    for allowed in "${allowlist[@]+"${allowlist[@]}"}"; do
        [[ "$file" == "$allowed"* ]] && continue 2
    done
    grep -q 'reqwest' "$file" || continue
    if ! rg -q "$guard_pattern" "$file"; then
        status=1
        echo "✗ $file sends an outbound request without validate_outbound_url:" >&2
        rg -nU "$send_pattern" "$file" | sed 's/^/    /' >&2
    fi
done < <(rg -lU "$send_pattern" "${existing[@]}" --glob '*.rs' --glob '!**/target/**' 2>/dev/null | sort -u)

if [ "$status" -ne 0 ]; then
    echo >&2
    echo "Pass the destination through systemprompt::models::net::validate_outbound_url" >&2
    echo "(or validate_outbound_url_with_trust / admin-core's validate_mesh_url) in the" >&2
    echo "file that sends the request." >&2
    exit 1
fi
echo "✓ every outbound request site validates its destination"
