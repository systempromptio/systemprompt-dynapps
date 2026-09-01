#!/usr/bin/env bash
# Run psql against the dedicated Fly knowledge Postgres through a `flyctl proxy`.
#
# `flyctl postgres connect` runs psql ON the Fly machine, so it cannot read
# local SQL files or stdin reliably. This script opens a local proxy port,
# waits for it, runs the local psql with every argument it was given, and
# closes the proxy on exit. Used by the `just knowledge-db-*` recipes.
#
#   KNOWLEDGE_DB_APP        app name (default sp-a2f658d8bc5f-knowledge)
#   KNOWLEDGE_DB_PROXY_PORT local port (default 15432)
#   KNOWLEDGE_DB_PASSWORD   password of role `knowledge`          (normal use)
#   KNOWLEDGE_DB_SUPERUSER_PASSWORD  password of `postgres`, printed by
#                           `flyctl postgres create`              (--admin)
#
#   psql.sh [--admin] <psql args...>
set -euo pipefail

app="${KNOWLEDGE_DB_APP:-sp-a2f658d8bc5f-knowledge}"
port="${KNOWLEDGE_DB_PROXY_PORT:-15432}"

if [ "${1:-}" = "--admin" ]; then
    shift
    : "${KNOWLEDGE_DB_SUPERUSER_PASSWORD:?set KNOWLEDGE_DB_SUPERUSER_PASSWORD (from flyctl postgres create)}"
    url="postgres://postgres:${KNOWLEDGE_DB_SUPERUSER_PASSWORD}@localhost:${port}/postgres"
else
    : "${KNOWLEDGE_DB_PASSWORD:?set KNOWLEDGE_DB_PASSWORD (the password of role knowledge)}"
    url="postgres://knowledge:${KNOWLEDGE_DB_PASSWORD}@localhost:${port}/knowledge"
fi

command -v psql >/dev/null 2>&1 || { echo "psql is required (libpq)" >&2; exit 2; }
command -v flyctl >/dev/null 2>&1 || { echo "flyctl is required" >&2; exit 2; }

flyctl proxy "${port}:5432" -a "$app" >/dev/null 2>&1 &
proxy=$!
# shellcheck disable=SC2064
trap "kill $proxy 2>/dev/null || true" EXIT

for _ in $(seq 1 30); do
    if pg_isready -h localhost -p "$port" >/dev/null 2>&1; then
        exec_ok=1
        break
    fi
    sleep 1
done
[ "${exec_ok:-0}" = 1 ] || { echo "flyctl proxy to $app did not come up on :$port" >&2; exit 1; }

psql -v ON_ERROR_STOP=1 "$url" "$@"
