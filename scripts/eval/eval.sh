#!/usr/bin/env bash
# Plugin eval harness: run a plugin's fixed prompt suite through the Windows
# Astound bridge, export every conversation from the gateway with the bridge's
# PAT, and compute deterministic metrics. No model scores anything here: the
# same exports always produce the same numbers.
#
#   scripts/eval/eval.sh run     <label> [plugin]   run the suite (costs money)
#   scripts/eval/eval.sh export  <label> [plugin]   pull bundles for the run's sessions
#   scripts/eval/eval.sh metrics <label> [plugin] [--against <label>]
#
# Needs WSL with the Windows user profile at $WIN_HOME, the bridge signed in
# and its proxy running. Docs: /documentation/analysis-evaluate-plugins.
set -euo pipefail

cmd=${1:?run|export|metrics}; label=${2:?label}; plugin=${3:-astound-salesforce}
here=$(cd "$(dirname "$0")" && pwd); root=$(cd "$here/../.." && pwd)
WIN_HOME=${WIN_HOME:-/mnt/c/Users/ejb50}
GATEWAY=${GATEWAY:-https://sp-dev.astound.digital}
PARALLEL=${PARALLEL:-4}
out="$root/evals/$plugin/$label"; raw="$out/raw"
winrun="$WIN_HOME/sp-skill-runs"; cc_projects="$WIN_HOME/.claude/projects/C--Users-ejb50-sp-skill-runs"
prompts="$here/prompts/$plugin.yaml"

pat() { tr -d '\r\n' < "$WIN_HOME/AppData/Roaming/astound/astound-bridge.pat"; }

run_one() {
  local skill=$1 dashed=${1//_/-} attempt
  for attempt in 1 2; do
    python3 "$here/suite.py" prompt "$prompts" "$skill" > "$winrun/$label.$skill.prompt.txt"
    local model; model=$(python3 "$here/suite.py" model "$prompts")
    timeout 1500 powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(wslpath -w "$here/run.ps1")" \
      -PromptFile "$(wslpath -w "$winrun/$label.$skill.prompt.txt")" \
      -OutFile "$(wslpath -w "$raw/results/$skill.json")" -Model "$model" >/dev/null 2>&1 || true
    # Why: a session that starts before the bridge's MCP servers are ready sees
    # no connector tools at all; that is a harness failure, not a skill result.
    if python3 "$here/suite.py" had-mcp "$raw/results/$skill.json" "$cc_projects"; then break; fi
    echo "  $dashed: no MCP tools on attempt $attempt" >&2
  done
  echo "$skill $attempt" >> "$raw/attempts.txt"
  echo "  done $dashed"
}

case $cmd in
  run)
    mkdir -p "$raw/results" "$winrun"; : > "$raw/attempts.txt"
    cp "$here/run.ps1" "$winrun/eval-run.ps1"
    python3 "$here/suite.py" manifest-start "$prompts" "$label" > "$out/manifest.json"
    export -f run_one; export here raw label prompts winrun cc_projects
    python3 "$here/suite.py" skills "$prompts" | xargs -P "$PARALLEL" -I{} bash -c 'run_one {}'
    python3 "$here/suite.py" manifest-finish "$out/manifest.json" "$raw"
    echo "run $label complete: $out/manifest.json" ;;
  export)
    mkdir -p "$raw/bundles" "$raw/cc" "$raw/datasets"
    token=$(pat)
    python3 - "$out/manifest.json" <<'PY' | while read -r skill sid; do
import json,sys
for s,r in json.load(open(sys.argv[1]))["runs"].items(): print(s, r["session_id"])
PY
      curl -sf -m 300 -H "Authorization: Bearer $token" \
        "$GATEWAY/admin/export/transcripts?session_id=$sid" -o "$raw/bundles/$skill.jsonl" \
        || echo "  export failed: $skill" >&2
      cp "$cc_projects/$sid.jsonl" "$raw/cc/$skill.jsonl" 2>/dev/null || echo "  no local transcript: $skill" >&2
    done
    window=$(python3 -c "import json;m=json.load(open('$out/manifest.json'));print(f\"from={m['started_at']}&to={m['finished_at']}\")")
    for ds in "analysis-skills?$window" "analysis-versions?marketplace=$plugin&$window" "analysis-marketplaces?$window"; do
      curl -sf -m 300 -H "Authorization: Bearer $token" \
        "$GATEWAY/admin/export/$ds&format=json" -o "$raw/datasets/${ds%%\?*}.json" \
        || echo "  dataset failed: ${ds%%\?*}" >&2
    done
    echo "export $label complete: $raw" ;;
  metrics)
    shift 3 || true
    python3 "$here/metrics.py" "$root/evals/$plugin" "$label" "$@" ;;
  *) echo "unknown command $cmd" >&2; exit 2 ;;
esac
