#!/usr/bin/env bash
# Run the scenario matrix in scenarios.json into one report directory.
#   matrix.sh [name-filter-regex]       e.g. matrix.sh 's1|s5'
# LOADTEST_SUITE picks the list: `scenarios` (default, the full matrix) or
# `smoke` (the ~15-minute pre-release subset).
# Ramps stop at the first step whose report has stop=true and record the last
# passing step as the ceiling in <out>/ceilings.json. Each run is 2 minutes.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
FILTER="${1:-.}"
OUT="${LOADTEST_OUT:-$ROOT/reports/loadtest/kind/$(date -u +%Y%m%dT%H%M%SZ)}"
# Never overwrite an earlier matrix: a reused directory gets a timestamp suffix.
[ ! -e "$OUT" ] || OUT="$OUT-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$OUT"
echo "matrix -> $OUT"

SUITE="${LOADTEST_SUITE:-scenarios}"
entries="$(python3 -c '
import json, sys
for s in json.load(open(sys.argv[1]))[sys.argv[2]]:
    print(json.dumps(s))' "$HERE/scenarios.json" "$SUITE")"

while IFS= read -r e <&3; do
    name="$(echo "$e" | jq -r .name)"
    echo "$name" | grep -qE "$FILTER" || continue
    args=(); while IFS= read -r a; do args+=("$a"); done < <(echo "$e" | jq -r '.args[]?')
    deploy=(); while IFS= read -r a; do deploy+=(--set "$a"); done < <(echo "$e" | jq -r '.deploy[]?')
    [ ${#deploy[@]} -eq 0 ] || "$HERE/cluster.sh" deploy "${deploy[@]}" > "$OUT/$name.deploy.log" 2>&1
    if [ "$(echo "$e" | jq '.ramp | length // 0')" -gt 0 ]; then
        last_ok=null
        for c in $(echo "$e" | jq -r '.ramp[]'); do
            "$HERE/run.sh" "$name-c$c" --out "$OUT" --concurrency "$c" "${args[@]+"${args[@]}"}" </dev/null
            stop="$(jq -r .stop "$OUT/$name-c$c.json")"
            valid="$(jq -r .valid "$OUT/$name-c$c.json")"
            if [ "$valid" != true ]; then last_ok='"invalid"'; break; fi
            [ "$stop" = true ] && break
            last_ok="$c"
        done
        python3 - "$OUT" "$name" "$last_ok" <<'PY'
import json, os, sys
out, name, last = sys.argv[1], sys.argv[2], sys.argv[3]
path = os.path.join(out, "ceilings.json")
d = json.load(open(path)) if os.path.exists(path) else {}
d[name] = None if last == "null" else ("invalid" if last == '"invalid"' else int(last))
json.dump(d, open(path, "w"), indent=2)
PY
    else
        "$HERE/run.sh" "$name" --out "$OUT" "${args[@]+"${args[@]}"}" </dev/null
    fi
    [ ${#deploy[@]} -eq 0 ] || "$HERE/cluster.sh" deploy > "$OUT/$name.revert.log" 2>&1
done 3<<< "$entries"
echo "done: $OUT"
