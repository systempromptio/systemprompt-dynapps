#!/usr/bin/env bash
# Boot a runtime image over one recorded release schema and require it to
# finish booting. Invoked by gates.yml's boot job as
#   boot-image-over-rung.sh <image> <rung-fixture.sql>
# with Postgres at localhost:5432 (user/password `test`).
#
# Why: the PR gates compiled and tested the code but never started the server
# with the real services/ tree, so a declaration the binary refuses only
# surfaced in release.yml's upgrade-boot. 0.76.0 failed there twice (a missing
# MCP binary, then `streams[0]: missing field publishers`) after every PR gate
# was green. This is release.yml's per-rung boot, run on the PR's own image.
set -euo pipefail

image="${1:?usage: boot-image-over-rung.sh <image> <rung-fixture.sql>}"
fixture="${2:?missing rung fixture}"
gate_email="${RELEASE_GATE_EMAIL:?RELEASE_GATE_EMAIL is required}"
export PGPASSWORD=test
die() { echo "::error::$*"; exit 1; }
[ -f "$fixture" ] || die "no rung fixture at $fixture"

rung="$(basename "$fixture" .sql)"; rung="${rung#release-baseline-}"
db="boot_${rung//./_}"
psql -h localhost -U test -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE $db"
grep -v '^\\' "$fixture" | grep -v "set_config('search_path'" \
    | psql -q -h localhost -U test -d "$db" -v ON_ERROR_STOP=1 > /dev/null
psql -q -h localhost -U test -d "$db" -v ON_ERROR_STOP=1 \
    -f tests/integration/schema-upgrade/src/seed_hot_tables.sql

vol="storage_data_${db}"
docker volume create "$vol" > /dev/null
docker run -d --name "boot_$db" --network host \
    -e ANTHROPIC_API_KEY=sk-ant-release-gate \
    -e ENTRA_OIDC_CLIENT_SECRET=release-gate-entra-not-a-real-secret \
    -e SYSTEMPROMPT_ADMIN_EMAIL="$gate_email" \
    -e DATABASE_URL="postgres://test:test@localhost:5432/$db" \
    -v "$vol:/app/storage/data" "$image" > /dev/null

healthy=""
for _ in $(seq 1 60); do
    # The early-bind listener answers /health 200 {"status":"starting"} during
    # bootstrap, so require the finished-boot body or /readyz.
    if curl -fsS http://localhost:8080/health 2>/dev/null | jq -e '.status == "healthy"' > /dev/null 2>&1 \
        || curl -fsS http://localhost:8080/readyz > /dev/null 2>&1; then healthy=1; break; fi
    [ "$(docker inspect -f '{{.State.Running}}' "boot_$db")" = true ] || break
    sleep 5
done
docker logs "boot_$db" 2>&1 | tail -40
docker rm -f "boot_$db" > /dev/null
[ -n "$healthy" ] || die "the image did not finish booting over release $rung (see the container log above)"
echo "booted over release $rung"
