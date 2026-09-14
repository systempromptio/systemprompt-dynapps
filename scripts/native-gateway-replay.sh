#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[[ "$#" == 3 && "$1" = /* && -x "$1" && "$2" = /* && -d "$2" && "$3" = /* ]] || { echo 'Expected absolute exporter, artifact directory and report path' >&2; exit 64; }
cd "$ROOT"
if [[ -z "${TEST_DATABASE_URL:-${DATABASE_URL:-}}" && -f "$ROOT/.systemprompt/profiles/local/secrets.json" ]]; then
    DATABASE_URL=$(python3 - "$ROOT/.systemprompt/profiles/local/secrets.json" <<'PYDB'
import json, sys
value = json.load(open(sys.argv[1])).get("database_url")
if not isinstance(value, str) or not value:
    sys.exit("Native gateway replay requires a database_url")
print(value)
PYDB
)
    export DATABASE_URL
fi
[[ -n "${TEST_DATABASE_URL:-${DATABASE_URL:-}}" ]] || { echo 'Supply TEST_DATABASE_URL or local fixture database profile' >&2; exit 64; }
export ANTHROPIC_API_KEY=sk-native-replay-fixture
fingerprint=$(python3 - <<'PYHASH'
import hashlib, os
print(hashlib.sha256(os.environ.get("TEST_DATABASE_URL", os.environ.get("DATABASE_URL", "")).encode()).hexdigest())
PYHASH
)
BUILD_FORCE=1 exec "$ROOT/scripts/build-coordinator.sh" run native-gateway-replay "$1:$2:$3:$fingerprint" -- "$1" "$2" "$3"
