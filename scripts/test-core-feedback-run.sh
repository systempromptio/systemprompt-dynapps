#!/usr/bin/env bash
# The coordinator holds one lock across database preparation and nextest.
set -euo pipefail

[ "$#" -ge 2 ] || { echo "test-core-feedback: missing test workspace or cargo arguments" >&2; exit 64; }
test_manifest="$1"
shift

# A fixture name is an additional guard, not permission to migrate an inferred
# URL. DATABASE_URL must have been explicitly supplied by the caller.
python3 - <<'PYDB'
import os, re, sys
from urllib.parse import parse_qs, unquote, urlsplit
try:
    url = urlsplit(os.environ.get("DATABASE_URL", ""))
    database = unquote(url.path.removeprefix("/"))
    if (url.scheme not in ("postgres", "postgresql") or not database
            or "/" in database
            or any(key.lower() in ("database", "dbname") for key in parse_qs(url.query))
            or not re.search(r"(?:^|[_-])test(?:$|[_-])", database, re.IGNORECASE)):
        raise ValueError("not a dedicated test database")
except ValueError:
    sys.exit("test-core-feedback: DATABASE_URL must explicitly name a dedicated database with a 'test' name component; application profiles and maintenance databases are not accepted")
PYDB

export SYSTEMPROMPT_REQUIRE_DB=1
# Compile the migrator from retained metadata: a fresh fixture database cannot
# answer SQLx's compile-time queries before its owner schemas are installed.
# The installer reapplies dependent owner capture definitions and seeds on
# existing databases as well as creating a fresh schema.
SQLX_OFFLINE=true cargo run --manifest-path "$test_manifest" \
    --package systemprompt-test-migrate --bin systemprompt-test-migrate

# Preparation failure exits before nextest; no test mutates schema concurrently
# with the migrator or accidentally relies on a previous run's reporting state.
exec cargo "$@"
