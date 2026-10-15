#!/bin/bash
# Night-audit end-to-end: preflight → run → wait → assert → follow-up, then
# the metrics report, which is written even when an earlier step failed.
# Stops at the first failing step (later steps need its state).
# Logs: $LOG_DIR (default: a fresh mktemp -d). FROM=30 resumes at a step;
# SKIP="40" skips steps (e.g. no bridge-configured `claude` on this machine).
set -uo pipefail
NIGHT_DIR="$(cd "$(dirname "$0")" && pwd)"
LOG_DIR="${LOG_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/night-audit-e2e.XXXXXX")}"
mkdir -p "$LOG_DIR"
FROM="${FROM:-00}"; SKIP=" ${SKIP:-} "
STEPS=(00-preflight 10-run 20-wait 30-assert 40-followup)
PASS=0; FAIL=0; SKIPPED=0
for s in "${STEPS[@]}"; do
  n="${s%%-*}"
  if [[ "$n" < "$FROM" || "$SKIP" == *" $n "* ]]; then
    SKIPPED=$((SKIPPED+1)); printf '  \033[2m-\033[0m %-16s skipped\n' "$s"; continue
  fi
  log="$LOG_DIR/$s.log"
  if bash "$NIGHT_DIR/$s.sh" > "$log" 2>&1; then
    PASS=$((PASS+1)); printf '  \033[0;32m✓\033[0m %-16s → %s\n' "$s" "$log"
  else
    FAIL=$((FAIL+1)); printf '  \033[0;31m✗\033[0m %-16s → %s\n' "$s" "$log"
    tail -15 "$log" | sed 's/^/      /'
    break
  fi
done
if [[ -s "$NIGHT_DIR/.state/run_id" ]]; then
  bash "$NIGHT_DIR/90-report.sh" > "$LOG_DIR/90-report.log" 2>&1 \
    && printf '  \033[0;32m✓\033[0m %-16s → %s\n' 90-report "$LOG_DIR/90-report.log" \
    || printf '  \033[0;31m✗\033[0m %-16s → %s\n' 90-report "$LOG_DIR/90-report.log"
  grep -E 'written:' "$LOG_DIR/90-report.log" | sed 's/^ */  /'
fi
echo ""
echo "Pass: $PASS    Fail: $FAIL    Skipped: $SKIPPED    Logs: $LOG_DIR/"
exit $((FAIL > 0 ? 1 : 0))
