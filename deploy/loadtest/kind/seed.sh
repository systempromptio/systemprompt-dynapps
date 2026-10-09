#!/usr/bin/env bash
# Seed load-test identities into the kind cluster and store their credentials.
#   seed.sh <users> [set] [projects]
#     users     how many users (lt-<set>-NNNN@loadtest.astounddigital.com)
#     set       credential set name (default u<users>); one Secret per set
#     projects  comma list the users join and the keys round-robin over
#               (default commerce,core; `commerce` alone = one hot project)
# Each user gets: a users row (CLI), one project_members row and a manual
# primary project (SQL; the CLI has no membership command), an unscoped PAT
# (the CLI cannot see astound's project provider, so `--scope project=` is
# refused; core attributes the request to the primary project instead), and a
# session minted with POST /api/public/gateway/sessions (0.64.0 has no /v1/sessions). Lines `token,session_id` land in .tokens/<set>.csv and in
# Secret loadtest-tokens-<set>, which the generator Jobs mount.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
N="${1:?usage: seed.sh <users> [set] [projects]}"
SET="${2:-u$N}"
PROJECTS="${3:-commerce,core}"
K="kubectl --context kind-astound-loadtest -n loadtest"
OUT="$HERE/.tokens/$SET.csv"
mkdir -p "$HERE/.tokens"

# Users and memberships first, in one SQL round trip after the CLI creates rows.
inner=$(cat <<'INNER'
set -eu
export SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/k8s/profile.yaml
N="$1"; SET="$2"; PROJECTS="$3"
i=1
while [ "$i" -le "$N" ]; do
    name=$(printf 'lt-%s-%04d' "$SET" "$i")
    systemprompt admin users create --name "$name" --email "$name@loadtest.astounddigital.com" --if-not-exists >/dev/null 2>&1 || true
    i=$((i + 1))
done
echo "users ok" >&2
INNER
)
$K exec deploy/astound-admin -c server -- sh -c "$inner" sh "$N" "$SET" "$PROJECTS"

proj_array="'$(echo "$PROJECTS" | sed "s/,/','/g")'"
$K exec deploy/postgres -c postgres -- psql -U postgres -d systemprompt -q -c "
WITH u AS (
  SELECT id, (ARRAY[$proj_array])[1 + (row_number() OVER (ORDER BY name) - 1) % cardinality(ARRAY[$proj_array])] AS p
  FROM users WHERE name LIKE 'lt-$SET-%'
), m AS (
  INSERT INTO project_members (project_id, user_id, source)
  SELECT p, id, 'manual' FROM u ON CONFLICT DO NOTHING
)
INSERT INTO user_scope_defaults (user_id, primary_project_id, source)
SELECT id, p, 'manual' FROM u
ON CONFLICT (user_id) DO UPDATE SET primary_project_id = EXCLUDED.primary_project_id, source = 'manual';" >&2

keys=$(cat <<'INNER'
set -eu
export SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/k8s/profile.yaml
SET="$1"
while read -r uid; do
    [ -n "$uid" ] || continue
    tok=$(systemprompt admin users api-key issue --user "$uid" --name "loadtest-$SET" 2>&1 \
        | grep -oE 'sp-live-[^[:space:]]+' | head -1)
    [ -n "$tok" ] || { echo "no token for $uid" >&2; continue; }
    sid=$(curl -s -X POST -H "Authorization: Bearer $tok" http://astound-gateway:8080/api/public/gateway/sessions | jq -r '.session_id // empty')
    [ -n "$sid" ] || { echo "no session for $uid" >&2; continue; }
    echo "$tok,$sid"
done
INNER
)
$K exec deploy/postgres -c postgres -- psql -U postgres -d systemprompt -Atc \
    "SELECT id FROM users WHERE name LIKE 'lt-$SET-%' ORDER BY name" \
    | $K exec -i deploy/astound-admin -c server -- sh -c "$keys" sh "$SET" > "$OUT"
chmod 600 "$OUT"
$K create secret generic "loadtest-tokens-$SET" --from-file=tokens.csv="$OUT" \
    --dry-run=client -o yaml | $K apply -f - >/dev/null
echo "seeded $(wc -l < "$OUT" | tr -d ' ') credentials into $OUT and Secret loadtest-tokens-$SET"
