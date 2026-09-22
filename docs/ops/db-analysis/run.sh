#!/usr/bin/env bash
# Runs every query in queries/ against the restored prod dump and writes results/<name>.txt.
# Read-only: the analysis database is never modified (refresh_conversation_facts runs in a
# rolled-back transaction inside its own query file).
set -euo pipefail
cd "$(dirname "$0")"
CONTAINER="${CONTAINER:-sp-systemprompt-astound-dcd323ed-local-postgres-1}"
DB="${DB:-sp_prod_analysis}"
for f in queries/${1:-*}.sql; do
  name=$(basename "$f" .sql)
  docker exec -i "$CONTAINER" psql -U systemprompt -d "$DB" -v ON_ERROR_STOP=0 -P pager=off -q < "$f" > "results/$name.txt" 2>&1 || true
  echo "$name: $(wc -l < "results/$name.txt") lines"
done
