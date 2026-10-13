#!/bin/bash
# Agent Farm end-to-end: keys → publish → pin → import → catalog → user →
# farm sync → run → DB trail → dashboard. Stops at the first failing step
# (later steps depend on earlier state) and prints a summary like sweep.sh.
# Logs: $OUT_DIR (default: a fresh mktemp -d). FROM=30 resumes at a step;
# SKIP="10 20" skips steps (e.g. when the kit is already published).
set -uo pipefail
FARM_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="${OUT_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/agent-farm-e2e.XXXXXX")}"
mkdir -p "$OUT_DIR"
FROM="${FROM:-00}"; SKIP=" ${SKIP:-} "
STEPS=(00-keys 10-publish-kit 20-pin 30-start-and-import 40-verify-catalog 50-mint-user 60-farm-sync 70-run 80-assert-db 90-dashboard)
PASS=0; FAIL=0; SKIPPED=0
for s in "${STEPS[@]}"; do
  n="${s%%-*}"
  if [[ "$n" < "$FROM" || "$SKIP" == *" $n "* ]]; then
    SKIPPED=$((SKIPPED+1)); printf '  \033[2m-\033[0m %-24s skipped\n' "$s"; continue
  fi
  log="$OUT_DIR/$s.log"
  if bash "$FARM_DIR/$s.sh" > "$log" 2>&1; then
    PASS=$((PASS+1)); printf '  \033[0;32m✓\033[0m %-24s → %s\n' "$s" "$log"
  else
    FAIL=$((FAIL+1)); printf '  \033[0;31m✗\033[0m %-24s → %s\n' "$s" "$log"
    tail -15 "$log" | sed 's/^/      /'
    break
  fi
done
echo ""
echo "Pass: $PASS    Fail: $FAIL    Skipped: $SKIPPED    Logs: $OUT_DIR/"
exit $((FAIL > 0 ? 1 : 0))
