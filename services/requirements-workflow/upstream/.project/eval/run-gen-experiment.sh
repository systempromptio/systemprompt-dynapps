#!/usr/bin/env bash
# Diagnostic generator experiment (NON-CANONICAL): re-run the full store-locator
# baseline with alternate generator model(s) via the runtime --gen-model override,
# keeping the fixed Mistral Large 3 judge. Does NOT edit config.yaml or the harness.
# Purpose: distinguish model-shaped failures from prompt/rubric gaps (see the
# threshold decision in the opsx-eval work). Results are diagnostic only and do
# NOT replace the canonical gpt-oss-120b baseline.
#
# Usage: run-gen-experiment.sh <gen-model-id> [<gen-model-id> ...]
set -uo pipefail
cd "$(dirname "$0")"

if [ "$#" -lt 1 ]; then
  echo "usage: run-gen-experiment.sh <gen-model-id> [<gen-model-id> ...]" >&2
  exit 1
fi

ARTIFACTS="discovery design proposal grooming specs fsd isd stories tasks technical-design"

for GEN in "$@"; do
  TS=$(date +%Y%m%d-%H%M%S)
  SAFE=$(echo "$GEN" | tr '/.:' '___')
  OUT=".results/genexp-$SAFE-$TS"
  mkdir -p "$OUT"
  echo ""
  echo "############### GEN-EXPERIMENT gen=$GEN judge=mistral (diagnostic) -> $OUT ###############"
  for a in $ARTIFACTS; do
    echo ""
    echo "======================== ARTIFACT: $a (gen=$GEN) ========================"
    node run.mjs eval --artifact "$a" --dir store-locator \
      --mode release --runs 20 --judge-samples 3 --concurrency 20 --gen-model "$GEN" --save \
      2>&1 | tee "$OUT/$a.log"
    echo "---- done: $a ----"
  done
  echo "$OUT" > ".results/genexp-latest-$SAFE.txt"
  echo "GENEXP_COMPLETE gen=$GEN $OUT"
done

echo ""
echo "GENEXP_ALL_COMPLETE"
