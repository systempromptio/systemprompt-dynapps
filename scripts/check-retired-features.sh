#!/usr/bin/env bash
# Gate: a retired feature stays retired. No tracked file outside the history
# allowlist below may name it.
#
# Why: 348b8efb removed the knowledge-bank MCP server, but its deploy config,
# self-host preflight, secrets, compose volumes, CI images and published docs
# survived for weeks. The preflight checked for a binary the image no longer
# shipped and the docs told operators pgvector was mandatory. Deleting a crate
# does not find the prose and scripts around it; this gate does.
#
# Allowlist: records of what already happened, which must not be rewritten —
# applied migrations and their tombstones, recorded schema baselines, the
# changelog, dated ops analyses, demo transcripts, and the gates that cite
# the removal (this one included).
set -euo pipefail

cd "$(dirname "$0")/.."

pattern='knowledge[-_ ]bank|knowledge-db|sync_knowledge|search_knowledge|storage/knowledge|systemprompt-mcp-knowledge|knowledge_database_url|app_knowledge|bedrock_api_key|--bedrock-key'

allow=(
    ':!extensions/*/schema/migrations/*'
    ':!tests/fixtures/schema/*'
    ':!CHANGELOG.md'
    ':!docs/ops/*'
    ':!demo/*'
    ':!scripts/check-retired-features.sh'
    ':!scripts/check-dropped-schema.sh'
    ':!scripts/check-dockerfile-paths.sh'
    ':!scripts/check-migration-numbers.sh'
)

if hits="$(git grep -nIiE "$pattern" -- . "${allow[@]}")"; then
    echo "$hits" >&2
    echo "check-retired-features: the knowledge bank was removed in 348b8efb; delete these references (history belongs in CHANGELOG.md)" >&2
    exit 1
fi
