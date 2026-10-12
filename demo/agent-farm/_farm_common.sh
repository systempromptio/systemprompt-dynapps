#!/bin/bash
# Shared helpers for the Agent Farm e2e (demo/agent-farm/*.sh).
# Sourced, never run: every step script does
#   source "$(cd "$(dirname "$0")" && pwd)/_farm_common.sh"
# Portable across macOS (BSD) and Linux (GNU): no grep -P, no sed -i,
# no head -n -1; temp dirs come from mktemp -d.

FARM_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../_common.sh
source "$FARM_DIR/../_common.sh"

KIT_DIR="${KIT_DIR:-$PROJECT_DIR/../systemprompt-agent-farm}"
FARM_ID="${FARM_ID:-dynapps-agent-farm}"
FARM_PLUGIN="${FARM_PLUGIN:-dynapps-odoo-delivery}"
FARM_AGENT="${FARM_AGENT:-odoo_module_reviewer}"
FARM_AGENTS="odoo_module_reviewer odoo_migration_planner odoo_release_notes_writer odoo_test_author"
FARM_PROJECT="${FARM_PROJECT:-odoo-delivery}"
KIT_IMAGE="${KIT_IMAGE:-ghcr.io/systempromptio/systemprompt-agent-farm}"
KIT_CHANNEL="${KIT_CHANNEL:-stable}"
FARM_USER_EMAIL="${FARM_USER_EMAIL:-farm-demo@dynapps.local}"
FARM_USER_NAME="${FARM_USER_NAME:-farm-demo}"
RUN_TIMEOUT="${RUN_TIMEOUT:-900}"

STATE_ADMIN_JWT="$FARM_DIR/.admin_jwt"
STATE_COOKIE="$FARM_DIR/.cookie.admin"
STATE_USER_TOKEN="$FARM_DIR/.token.user"
STATE_USER_ID="$FARM_DIR/.user_id"
STATE_DIGEST="$FARM_DIR/.digest"
STATE_VERSION="$FARM_DIR/.version"
STATE_RUN_ID="$FARM_DIR/.run_id"
STATE_RUN_STARTED="$FARM_DIR/.run_started_at"

PROFILE_YAML="$PROJECT_DIR/.systemprompt/profiles/$PROFILE/profile.yaml"

need() {
  local c
  for c in "$@"; do
    command -v "$c" >/dev/null 2>&1 || { fail "missing required tool: $c"; exit 1; }
  done
}
need curl jq python3

sha256_of() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'
  else sha256sum "$1" | awk '{print $1}'; fi
}

# Admin JWT from a CLI session (cached in .admin_jwt; re-minted when rejected).
farm_admin_jwt() {
  if [[ -s "$STATE_ADMIN_JWT" ]]; then
    local code
    code=$(curl -s -o /dev/null -w '%{http_code}' -H "Authorization: Bearer $(cat "$STATE_ADMIN_JWT")" \
      "$BASE_URL/api/v1/admin/services/status" || echo 000)
    if [[ "$code" == "200" ]]; then cat "$STATE_ADMIN_JWT"; return; fi
  fi
  local jwt
  jwt=$("$CLI" admin session login --token-only --profile "$PROFILE" 2>/dev/null | tail -1 | tr -d '[:space:]')
  [[ -n "$jwt" ]] || { fail "admin session login produced no token" >&2; return 1; }
  ( umask 077; printf '%s' "$jwt" > "$STATE_ADMIN_JWT" )
  printf '%s' "$jwt"
}

# Header used for admin API calls; override FARM_ADMIN_AUTH to use an admin PAT.
farm_admin_auth_header() {
  if [[ -n "${FARM_ADMIN_AUTH:-}" ]]; then printf 'Authorization: Bearer %s' "$FARM_ADMIN_AUTH"
  else printf 'Authorization: Bearer %s' "$(farm_admin_jwt)"; fi
}

# The admin account the dashboard is fetched as: ADMIN_EMAIL, else the first
# user holding the admin role.
farm_admin_email() {
  if [[ -n "${ADMIN_EMAIL:-}" ]]; then echo "$ADMIN_EMAIL"; return; fi
  cli_json infra db query "SELECT email FROM users WHERE 'admin' = ANY(roles) ORDER BY created_at LIMIT 1" \
    | jq -r '.items[0].email // empty' 2>/dev/null
}

# Writes an admin session cookie jar via the dev-login sidecar and prints its
# path. Reuses the jar while /admin still answers 200 with it.
farm_admin_cookie() {
  if [[ -s "$STATE_COOKIE" ]]; then
    local code
    code=$(curl -s -o /dev/null -w '%{http_code}' -b "$STATE_COOKIE" "$BASE_URL/admin" || echo 000)
    if [[ "$code" == "200" ]]; then echo "$STATE_COOKIE"; return; fi
  fi
  local bin email out link path
  bin="$(dirname "$CLI")/systemprompt-dev-login"
  [[ -x "$bin" ]] || { fail "systemprompt-dev-login not found beside $CLI" >&2; return 1; }
  email=$(farm_admin_email)
  [[ -n "$email" ]] || { fail "no admin user found; set ADMIN_EMAIL" >&2; return 1; }
  out=$(SYSTEMPROMPT_PROFILE="$PROFILE_YAML" "$bin" "$email" 2>&1) || { fail "dev-login failed: $out" >&2; return 1; }
  link=$(printf '%s\n' "$out" | grep -oE 'https?://[^[:space:]"]+' | head -1)
  [[ -n "$link" ]] || { fail "dev-login printed no link: $out" >&2; return 1; }
  # Redeem against BASE_URL so the cookie lands on the host curl talks to.
  path=$(printf '%s' "$link" | sed -E 's#^https?://[^/]+##')
  rm -f "$STATE_COOKIE"
  ( umask 077; curl -s -o /dev/null -c "$STATE_COOKIE" "$BASE_URL$path" )
  grep -q 'access_token' "$STATE_COOKIE" 2>/dev/null || { fail "dev-login redeem set no access_token cookie" >&2; return 1; }
  echo "$STATE_COOKIE"
}

# HTTP helpers: print body, set HTTP_CODE global.
#   http_json METHOD URL AUTH_HEADER [BODY]
http_json() {
  local method="$1" url="$2" auth="$3" body="${4:-}" tmp
  tmp=$(mktemp)
  if [[ -n "$body" ]]; then
    HTTP_CODE=$(curl -s -o "$tmp" -w '%{http_code}' -X "$method" -H "$auth" \
      -H 'Content-Type: application/json' --data "$body" "$url" || echo 000)
  else
    HTTP_CODE=$(curl -s -o "$tmp" -w '%{http_code}' -X "$method" -H "$auth" "$url" || echo 000)
  fi
  cat "$tmp"; rm -f "$tmp"
}

assert_http() {
  local actual="$1" expected="$2" label="$3"
  if [[ "$actual" == "$expected" || ( "$expected" == "2xx" && "$actual" =~ ^2 ) ]]; then
    echo -e "  ${GREEN}✓ PASS${R} — $label: HTTP $actual"
  else
    echo -e "  ${RED}✗ FAIL${R} — $label: expected HTTP $expected, got $actual" >&2
    exit 1
  fi
}

load_user_pat() {
  [[ -s "$STATE_USER_TOKEN" && -s "$STATE_USER_ID" ]] || { fail "no farm user; run 50-mint-user.sh first"; exit 1; }
  USER_PAT=$(cat "$STATE_USER_TOKEN"); FARM_USER_ID=$(cat "$STATE_USER_ID")
}

# First case id of an eval suite (CASE env wins).
farm_first_case() {
  local suite="${1:-$FARM_AGENT}" f
  if [[ -n "${CASE:-}" ]]; then echo "$CASE"; return; fi
  f="$KIT_DIR/farm/evals/$suite.yaml"
  [[ -f "$f" ]] || { echo ""; return; }
  python3 -I - "$f" <<'PY'
import sys
try:
    import yaml
    doc = yaml.safe_load(open(sys.argv[1])) or {}
    cases = doc.get("cases") if isinstance(doc, dict) else doc
    print((cases or [{}])[0].get("id", ""))
except ImportError:
    import re
    for line in open(sys.argv[1]):
        m = re.match(r"\s*-\s*id:\s*[\"']?([^\"'\s#]+)", line)
        if m:
            print(m.group(1)); break
PY
}

# SQL literal quoting for ids we interpolate.
sql_q() { printf "'%s'" "$(printf '%s' "$1" | sed "s/'/''/g")"; }
