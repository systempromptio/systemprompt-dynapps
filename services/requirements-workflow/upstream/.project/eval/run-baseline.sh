#!/usr/bin/env bash
# One-shot baseline runner: judge every feature artifact against the
# store-locator golden case at release grade (gpt-oss generator + Mistral
# Large 3 judge), 20 runs/flow x 3 judge samples, concurrency 20.
set -uo pipefail
cd "$(dirname "$0")"

TS=$(date +%Y%m%d-%H%M%S)
OUT=".results/baseline-$TS"
mkdir -p "$OUT"

ARTIFACTS="discovery design proposal grooming specs fsd isd stories tasks technical-design"

echo "BASELINE run $TS -> $OUT"
for a in $ARTIFACTS; do
  echo ""
  echo "======================== ARTIFACT: $a ========================"
  node run.mjs eval --artifact "$a" --dir store-locator \
    --mode release --runs 20 --judge-samples 3 --concurrency 20 --save \
    2>&1 | tee "$OUT/$a.log"
  echo "---- done: $a ----"
done

# Record where the newest results live so the assembler can find them.
echo "$OUT" > ".results/baseline-latest.txt"
echo ""
echo "BASELINE_COMPLETE $OUT"
