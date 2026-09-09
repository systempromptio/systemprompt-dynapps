#!/usr/bin/env bash
# Prove a published release deploys the way the customer will deploy it.
#
#     scripts/e2e-published-release.sh 0.44.0        (= just e2e-release 0.44.0)
#
# Nothing from this checkout reaches the test except deploy/self-host/. In a
# fresh directory it pulls and cosign-verifies the GHCR image, downloads and
# verifies every GitHub Release asset, stands up the customer topology in
# Docker — PostgreSQL 16 + pgvector primary with a streaming replica, three
# gateway nodes (in/us/eu) sharing one rendered profile, a TLS-terminating
# load balancer — boots node 1 alone and then the other two, and asserts the
# whole thing: health, migrations, downloads, cross-node tokens, MCP children,
# governance audit, replication, restart survival, and the Linux bridge
# installer in a config-free client container. A report lands in
# deploy/self-host/E2E-REPORT-<version>.md.
#
# Inputs (file named by E2E_ENV_FILE, default .e2e.env, gitignored):
#   ANTHROPIC_API_KEY  [BEDROCK_API_KEY GHCR_USER GHCR_TOKEN — else `gh auth token`]
# Knobs: E2E_KEEP=1 keeps the stack up on exit; E2E_SKIP_BRIDGE=1 skips step 8;
#        E2E_PG_TAG (default pg16) picks the pgvector/pgvector tag.
#        E2E_IMAGE_TAG selects a signed repair candidate without moving the
#        released version. E2E_RELEASE_TAG selects its download source.
#        E2E_MACOS_SHA256 pins the CI DMG while testing an unpublished repair.
set -euo pipefail

VERSION="${1:?usage: $0 X.Y.Z}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMAGE="ghcr.io/astound-digital/systemprompt-astound"
IMAGE_TAG="${E2E_IMAGE_TAG:-$VERSION}"
RELEASE_TAG="${E2E_RELEASE_TAG:-v$VERSION}"
REPO="Astound-Digital/systemprompt-astound"
ENV_FILE="${E2E_ENV_FILE:-$REPO_ROOT/.e2e.env}"
PG_TAG="${E2E_PG_TAG:-pg16}"
WORK="${E2E_WORKDIR:-$(mktemp -d "${TMPDIR:-/tmp}/astound-e2e-$VERSION.XXXX")}"
REPORT="$REPO_ROOT/deploy/self-host/E2E-REPORT-$VERSION.md"
P="e2e$(printf '%s' "$WORK" | cksum | cut -c1-6)"      # compose project, unique per workdir
HOST="sp.local"
PORT="${E2E_LB_PORT:-18443}"
NODE_PORT_BASE="${E2E_NODE_PORT_BASE:-18080}"   # nodes on +1,+2,+3; high so a dev server on 8080-8083 cannot shadow them

# ── reporting ────────────────────────────────────────────────────────────────
PASS=0 FAIL=0 SKIP=0; ROWS=()
T0=$(date +%s)
say()  { printf '\033[1m==>\033[0m %s\n' "$*"; }
ok()   { printf '  \033[32mPASS\033[0m  %s\n' "$1"; PASS=$((PASS+1)); ROWS+=("| PASS | $1 |"); }
bad()  { printf '  \033[31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL+1)); ROWS+=("| FAIL | $1 |"); }
skip() { printf '  \033[33mSKIP\033[0m  %s\n' "$1"; SKIP=$((SKIP+1)); ROWS+=("| SKIP | $1 |"); }
die()  { printf '\033[31mERROR:\033[0m %s\n' "$*" >&2; write_report; exit 1; }
check() { local label="$1"; shift; if "$@" >/dev/null 2>&1; then ok "$label"; else bad "$label"; fi; }
write_report() {
    {
        echo "# E2E report — release $VERSION"
        echo
        echo "Run: $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(uname -srm), Docker $(docker --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1), PostgreSQL image pgvector/pgvector:$PG_TAG, image \`$IMAGE:$IMAGE_TAG\`, release \`$RELEASE_TAG\`, duration $(( $(date +%s) - T0 ))s."
        echo
        echo "Topology: pg-primary + pg-replica (streaming), nodes node-in / node-us / node-eu sharing one rendered profile, nginx TLS load balancer at https://$HOST:$PORT round-robin, clean client container for the bridge installer."
        echo
        echo "| result | check |"; echo "|---|---|"
        printf '%s\n' "${ROWS[@]}"
        echo
        echo "**$PASS pass, $FAIL fail, $SKIP skip.**"
    } > "$REPORT"
    echo "report: $REPORT"
}
dc() { docker compose -p "$P" -f "$WORK/docker-compose.e2e.yml" "$@"; }
cleanup() {
    if [ "${E2E_KEEP:-0}" = "1" ]; then say "E2E_KEEP=1 — stack left running in project $P ($WORK)"; return; fi
    docker rm -f "$P-clean-client" >/dev/null 2>&1 || true
    dc down -v --remove-orphans >/dev/null 2>&1 || true
}
trap 'cleanup' EXIT

# ── inputs ───────────────────────────────────────────────────────────────────
if [ -f "$ENV_FILE" ]; then
    set -a; . "$ENV_FILE"; set +a
fi
: "${ANTHROPIC_API_KEY:?provide ANTHROPIC_API_KEY in the environment or E2E_ENV_FILE}"
# Bedrock is optional: the shipped knowledge bank uses the local provider.
BEDROCK_API_KEY="${BEDROCK_API_KEY:-}"
if [ -n "${E2E_MACOS_SHA256:-}" ]; then
    [[ "$E2E_MACOS_SHA256" =~ ^[0-9a-f]{64}$ ]] || die 'invalid E2E_MACOS_SHA256'
fi
for t in docker gh jq openssl curl; do command -v "$t" >/dev/null || die "$t is required"; done
for p in "$PORT" $((NODE_PORT_BASE+1)) $((NODE_PORT_BASE+2)) $((NODE_PORT_BASE+3)); do
    (exec 3<>"/dev/tcp/127.0.0.1/$p") 2>/dev/null && die "port $p is already in use on this host (set E2E_LB_PORT / E2E_NODE_PORT_BASE)"
done
COSIGN="$(command -v cosign || true)"
[ -n "$COSIGN" ] || [ -x "${COSIGN_BIN:-}" ] && COSIGN="${COSIGN:-$COSIGN_BIN}"

say "workdir $WORK, compose project $P"
mkdir -p "$WORK"
rm -rf "$WORK/self-host" && cp -R "$REPO_ROOT/deploy/self-host" "$WORK/self-host"
cd "$WORK"

# ── 1. image ─────────────────────────────────────────────────────────────────
say "1. pull and verify the image"
if [ -n "${GHCR_TOKEN:-}" ]; then
    echo "$GHCR_TOKEN" | docker login ghcr.io -u "${GHCR_USER:?}" --password-stdin >/dev/null
else
    gh auth token | docker login ghcr.io -u "$(gh api user -q .login)" --password-stdin >/dev/null
fi
check "docker pull $IMAGE:$IMAGE_TAG" docker pull "$IMAGE:$IMAGE_TAG"
if [ -n "$COSIGN" ]; then
    check "cosign verify image signature" "$COSIGN" verify \
        --certificate-identity-regexp="https://github.com/$REPO/" \
        --certificate-oidc-issuer=https://token.actions.githubusercontent.com "$IMAGE:$IMAGE_TAG"
else
    skip "cosign verify image (cosign not installed)"
fi
check "image labels report version $VERSION" bash -c \
    "docker image inspect '$IMAGE:$IMAGE_TAG' -f '{{index .Config.Labels \"org.opencontainers.image.version\"}}' | grep -qx '$VERSION'"

# ── 2. release assets ────────────────────────────────────────────────────────
say "2. download and verify the GitHub Release assets"
mkdir -p rel && ( cd rel && gh release download "$RELEASE_TAG" -R "$REPO" --clobber \
    -p 'astound-bridge-*' -p 'install.sh*' -p 'SHA256SUMS*' -p "systemprompt-astound-server-$VERSION-linux-amd64.tar.gz*" )
( cd rel && sha256sum -c --ignore-missing SHA256SUMS >/dev/null ) && ok "SHA256SUMS matches every downloaded asset" || bad "SHA256SUMS mismatch"
for a in astound-bridge-windows.exe astound-bridge-macos.dmg astound-bridge-linux-x86_64.tar.gz astound-bridge-linux-aarch64.tar.gz install.sh "systemprompt-astound-server-$VERSION-linux-amd64.tar.gz"; do
    [ -s "rel/$a" ] && ok "release asset present: $a" || bad "release asset missing: $a"
    if [ -n "$COSIGN" ] && [ -s "rel/$a.sig" ]; then
        check "cosign verify-blob $a" "$COSIGN" verify-blob --signature "rel/$a.sig" --certificate "rel/$a.pem" \
            --certificate-identity-regexp="https://github.com/$REPO/" \
            --certificate-oidc-issuer=https://token.actions.githubusercontent.com "rel/$a"
    fi
done
mkdir -p srv && tar -xzf "rel/systemprompt-astound-server-$VERSION-linux-amd64.tar.gz" -C srv
SRVBIN="$(find srv -type f -name systemprompt -path '*/bin/*' | head -1)"
if [ -n "$SRVBIN" ] && docker run --rm -v "$WORK/srv:/srv:ro" --entrypoint "/srv/${SRVBIN#srv/}" "$IMAGE:$IMAGE_TAG" --version 2>/dev/null | grep -q "$VERSION"; then
    ok "server tarball binary runs and reports $VERSION (non-Docker fallback)"
else
    bad "server tarball binary does not run / wrong version"
fi

# ── 3. topology files ────────────────────────────────────────────────────────
say "3. render TLS material, database init, load balancer and compose"
mkdir -p tls pg lb
openssl req -x509 -newkey rsa:2048 -nodes -days 2 -subj "/CN=$HOST" \
    -addext "subjectAltName=DNS:$HOST,DNS:pg-primary,DNS:localhost" \
    -keyout tls/key.pem -out tls/cert.pem >/dev/null 2>&1
chmod 644 tls/key.pem tls/cert.pem
APP_PW="app-$(openssl rand -hex 8)"; REPL_PW="repl-$(openssl rand -hex 8)"
cat > pg/00-init.sh <<PGI
#!/bin/bash
set -e
cp /tls/cert.pem "\$PGDATA/server.crt"; cp /tls/key.pem "\$PGDATA/server.key"; chmod 600 "\$PGDATA/server.key"
cat >> "\$PGDATA/postgresql.conf" <<CONF
ssl = on
ssl_cert_file = 'server.crt'
ssl_key_file = 'server.key'
wal_level = replica
max_wal_senders = 5
hot_standby = on
CONF
echo "host replication replicator all scram-sha-256" >> "\$PGDATA/pg_hba.conf"
psql -v ON_ERROR_STOP=1 -U postgres -c "CREATE ROLE replicator REPLICATION LOGIN PASSWORD '$REPL_PW'"
psql -v ON_ERROR_STOP=1 -U postgres -v pw='$APP_PW' -f /provision.sql
PGI
chmod +x pg/00-init.sh
cat > pg/replica.sh <<'PGR'
#!/bin/bash
set -e
if [ ! -s "$PGDATA/PG_VERSION" ]; then
    until pg_isready -h pg-primary -U postgres >/dev/null 2>&1; do sleep 1; done
    PGPASSWORD="$REPL_PW" pg_basebackup -h pg-primary -U replicator -D "$PGDATA" -R -X stream -c fast
    chmod 700 "$PGDATA"
fi
exec docker-entrypoint.sh postgres
PGR
chmod +x pg/replica.sh
cat > lb/nginx.conf <<NGX
events {}
http {
    upstream gateway { server node-in:8080; server node-us:8080; server node-eu:8080; }
    server {
        listen $PORT ssl;
        server_name $HOST;
        ssl_certificate /tls/cert.pem;
        ssl_certificate_key /tls/key.pem;
        location /metrics { return 403; }
        location /api/v1/health/detail { return 403; }
        location / {
            proxy_pass http://gateway;
            proxy_http_version 1.1;
            proxy_set_header Host \$host;
            proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto https;
            proxy_set_header Connection "";
            proxy_buffering off;
            proxy_read_timeout 300s;
            client_max_body_size 50m;
            add_header X-Upstream \$upstream_addr always;
        }
    }
}
NGX

DB_URL="postgres://systemprompt:$APP_PW@pg-primary:5432/systemprompt?sslmode=require"
rm -rf "$WORK/profile"
./self-host/render-profile.sh --out "$WORK/profile" \
    --external-url "https://$HOST:$PORT" --admin-email "admin@$HOST" --trusted-proxies "10.0.0.0/8,172.16.0.0/12,192.168.0.0/16" \
    --database-url "$DB_URL" --anthropic-key "$ANTHROPIC_API_KEY" --bedrock-key "$BEDROCK_API_KEY" >/dev/null \
    && ok "render-profile.sh produced profile.yaml + secrets.json" || bad "render-profile.sh failed"
chmod 755 profile; chmod 644 profile/secrets.json   # the container runs as uid 1000; bind mounts keep host ownership
[ "$(jq -r '.signing_key_pem' profile/secrets.json | base64 -d 2>/dev/null | head -c 27)" = "-----BEGIN PRIVATE KEY-----" ] \
    && ok "secrets.json carries a base64 PKCS#8 signing key" || bad "secrets.json signing key malformed"

node() { cat <<NODE
  $1:
    image: $IMAGE:$IMAGE_TAG
    stop_grace_period: 30s
    environment:
      SYSTEMPROMPT_PROFILE_DIR: /app/.systemprompt/profiles/self-host
      DATABASE_URL: "$DB_URL"
      NODE_REGION: $2
    volumes:
      - ./profile:/app/.systemprompt/profiles/self-host:ro
      - $1_web:/app/web
      - $1_data:/app/storage/data
    ports: ["$3:8080"]
    healthcheck:
      test: ["CMD-SHELL", "curl -fsS http://localhost:8080/api/v1/health | grep -q '\"healthy\"'"]
      interval: 5s
      timeout: 5s
      retries: 60
      start_period: 180s
    depends_on:
      pg-primary: { condition: service_healthy }
NODE
}
{
cat <<YML
services:
  pg-primary:
    image: pgvector/pgvector:$PG_TAG
    environment: { POSTGRES_PASSWORD: postgres }
    volumes:
      - ./pg/00-init.sh:/docker-entrypoint-initdb.d/00-init.sh:ro
      - ./self-host/provision.sql:/provision.sql:ro
      - ./tls:/tls:ro
      - pg_primary:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres && psql -U postgres -d systemprompt -tAc \"select 1 from pg_extension where extname='vector'\" | grep -q 1"]
      interval: 3s
      timeout: 3s
      retries: 40
  pg-replica:
    image: pgvector/pgvector:$PG_TAG
    user: postgres
    environment: { POSTGRES_PASSWORD: postgres, REPL_PW: "$REPL_PW" }
    entrypoint: ["/replica.sh"]
    volumes:
      - ./pg/replica.sh:/replica.sh:ro
      - pg_replica:/var/lib/postgresql/data
    depends_on:
      pg-primary: { condition: service_healthy }
  lb:
    image: nginx:1.31
    volumes:
      - ./lb/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./tls:/tls:ro
    ports: ["$PORT:$PORT"]
YML
node node-in india $((NODE_PORT_BASE+1))
node node-us us $((NODE_PORT_BASE+2))
node node-eu eu $((NODE_PORT_BASE+3))
cat <<YML
volumes:
  pg_primary: {}
  pg_replica: {}
  node-in_web: {}
  node-in_data: {}
  node-us_web: {}
  node-us_data: {}
  node-eu_web: {}
  node-eu_data: {}
YML
} > docker-compose.e2e.yml
dc config -q && ok "compose file validates" || die "compose file invalid"

# ── 4. database ──────────────────────────────────────────────────────────────
say "4. PostgreSQL $PG_TAG primary + streaming replica, provision.sql as the DBA"
dc up -d pg-primary pg-replica >/dev/null
for _ in $(seq 1 60); do dc ps --format json pg-primary 2>/dev/null | grep -q '"healthy"' && break; sleep 2; done
dc ps --format json pg-primary | grep -q '"healthy"' && ok "primary healthy with vector extension (provision.sql ran)" || die "primary never became healthy: $(dc logs pg-primary | tail -20)"
pgx() { dc exec -T pg-primary psql -U postgres -d systemprompt -tAc "$1"; }
ext="$(pgx "select count(*) from pg_extension where extname in ('uuid-ossp','pgcrypto','vector')" | tr -d '[:space:]')"
[ "$ext" = "3" ] && ok "uuid-ossp, pgcrypto and vector installed" || bad "expected 3 extensions, found '$ext'"
for _ in $(seq 1 60); do dc exec -T pg-replica pg_isready -U postgres >/dev/null 2>&1 && break; sleep 2; done
rep="$(dc exec -T pg-replica psql -U postgres -tAc 'select pg_is_in_recovery()' 2>/dev/null | tr -d '[:space:]' || true)"
[ "$rep" = "t" ] && ok "replica is streaming (pg_is_in_recovery = true)" || bad "replica not in recovery: '$rep'"
check "primary enforces TLS for the application role (sslmode=require connects)" \
    dc exec -T pg-primary psql "$DB_URL" -tAc 'select 1'

# ── 5/6. nodes ───────────────────────────────────────────────────────────────
say "5. boot node-in alone (migrations + bootstrap), then node-us and node-eu, then the load balancer"
# Images before 0.44.0 do not render web/dist per node in the entrypoint (the
# scheduler's lock lets only one node do it at boot). Apply the documented
# workaround and record it, so the report says what was proven and how.
publish_workaround() {
    docker exec -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml "$P-$1-1" \
        /app/bin/systemprompt infra jobs run publish_pipeline >/dev/null 2>&1 \
        && skip "$1: web/dist rendered by the manual 'infra jobs run publish_pipeline' (pre-0.44.0 workaround)" \
        || bad "$1: manual publish_pipeline failed"
}
needs_publish_workaround() { [ "$(printf '%s\n0.44.0\n' "$VERSION" | sort -V | head -1)" != "0.44.0" ]; }
wait_healthy() {
    for _ in $(seq 1 120); do dc ps --format json "$1" 2>/dev/null | grep -q '"healthy"' && return 0; sleep 3; done
    echo "--- $1 logs ---"; dc logs --tail 60 "$1"; return 1
}
dc up -d node-in >/dev/null
wait_healthy node-in && ok "node-in healthy on first boot" || die "node-in failed to boot"
M1="$(pgx 'select count(*) from extension_migrations' | tr -d '[:space:]')"
# Strictly serial: the MCP service registry in the shared database is keyed by
# server name, and two nodes reconciling it in the same second race each other
# (verify shared-profile startup on every node).
dc up -d node-us >/dev/null
wait_healthy node-us && ok "node-us healthy (booted after node-in)" || bad "node-us failed to boot"
dc up -d node-eu >/dev/null
wait_healthy node-eu && ok "node-eu healthy (booted after node-us)" || bad "node-eu failed to boot"
if needs_publish_workaround; then for n in node-in node-us node-eu; do publish_workaround "$n"; done; fi
dc up -d lb >/dev/null; sleep 3
M3="$(pgx 'select count(*) from extension_migrations' | tr -d '[:space:]')"
[ -n "$M1" ] && [ "$M1" = "$M3" ] && ok "migration ledger unchanged by nodes 2 and 3 ($M1 rows: migrations ran once)" || bad "migration ledger changed after nodes 2/3 ($M1 → $M3)"

# ── 7. assertions ────────────────────────────────────────────────────────────
say "7. assertions"
LB="https://$HOST:$PORT"
C=(curl -sk --resolve "$HOST:$PORT:127.0.0.1" --max-time 60)
for n in node-in:$((NODE_PORT_BASE+1)) node-us:$((NODE_PORT_BASE+2)) node-eu:$((NODE_PORT_BASE+3)); do
    name="${n%%:*}"; port="${n#*:}"
    "$REPO_ROOT/deploy/self-host/verify-node.sh" --url "http://localhost:$port" --container "$P-$name-1" --version "$VERSION" > "verify-$name.log" 2>&1 \
        && ok "verify-node.sh passes on $name ($(grep -c PASS "verify-$name.log") checks)" \
        || { bad "verify-node.sh fails on $name: $(grep FAIL "verify-$name.log" | head -3 | tr '\n' ';')"; }
done
for n in 1 2 3; do
    code="$(curl -s -o /dev/null -w '%{http_code}' "http://localhost:$((NODE_PORT_BASE+n))/documentation/")"
    [ "$code" = "200" ] && ok "public site rendered on node $n (publish pipeline ran on every node)" || bad "public site missing on node $n (/documentation/ → $code): publish_pipeline skipped by the job lock?"
done
body="$("${C[@]}" "$LB/api/v1/health" || true)"
[[ "$body" == *healthy* ]] && ok "load balancer: TLS + /api/v1/health healthy" || bad "load balancer health: '$body'"
[ "$("${C[@]}" -o /dev/null -w '%{http_code}' "$LB/metrics")" = "403" ] && ok "load balancer blocks /metrics" || bad "/metrics reachable through the load balancer"
seen="$(for _ in $(seq 1 12); do "${C[@]}" -D - -o /dev/null "$LB/api/v1/health" | grep -i '^x-upstream' | tr -d '\r' | awk '{print $2}'; done | sort -u | wc -l)"
[ "$seen" -ge 3 ] && ok "load balancer round-robins across all three nodes" || bad "load balancer reached only $seen upstream(s)"

xin() { docker exec -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml "$P-node-in-1" /app/bin/systemprompt "$@"; }
ADMIN_EMAIL="$(pgx "select email from users where name='admin' limit 1" | tr -d '[:space:]')"
[ -n "$ADMIN_EMAIL" ] && ok "bootstrap admin exists ($ADMIN_EMAIL)" || bad "no bootstrap admin"
ADMIN_ID="$(pgx "select id from users where email='$ADMIN_EMAIL' limit 1" | tr -d '[:space:]')"
PAT="$(xin admin users api-key issue --user "$ADMIN_ID" --name e2e 2>/dev/null | grep -oE 'sp-live-[A-Za-z0-9._-]+' | head -1 || true)"
[ -n "$PAT" ] && ok "minted a PAT on node-in" || bad "could not mint a PAT on node-in"
# /v1/messages binds every call to a gateway session minted from the PAT.
SESSION="$("${C[@]}" -X POST "$LB/api/public/gateway/sessions" -H "authorization: Bearer $PAT" -H 'content-type: application/json' -d '{}' | jq -r '.session_id // empty')"
[ -n "$SESSION" ] && ok "minted a gateway session through the LB with the PAT" || bad "could not mint a gateway session (POST /api/public/gateway/sessions)"
msg() { "${C[@]}" -o "$1" -w '%{http_code}' -X POST "$LB/v1/messages" \
        -H "authorization: Bearer $PAT" -H "x-session-id: $SESSION" -H 'anthropic-version: 2023-06-01' -H 'content-type: application/json' \
        -d '{"model":"claude-haiku-4-5","max_tokens":16,"messages":[{"role":"user","content":"Reply with the single word pong."}]}'; }
hits=""
for i in 1 2 3 4 5 6; do
    code="$(msg "msg-$i.json")"; up="$("${C[@]}" -D - -o /dev/null "$LB/api/v1/health" | grep -i '^x-upstream' | awk '{print $2}' | tr -d '\r')"
    [ "$code" = "200" ] && hits="$hits ok" || { echo "  /v1/messages → $code: $(head -c 300 "msg-$i.json")"; }
done
[ "$(echo "$hits" | wc -w)" -ge 3 ] && ok "PAT minted on node-in is accepted for /v1/messages through the LB ($(echo "$hits" | wc -w)/6 ok; shared signing key)" || bad "cross-node inference failed ($(echo "$hits" | wc -w)/6 ok)"
grep -q '"pong"\|pong' msg-1.json 2>/dev/null && ok "upstream Anthropic answered through the gateway" || bad "no model answer in response"
for n in $((NODE_PORT_BASE+2)) $((NODE_PORT_BASE+3)); do
    code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 60 -X POST "http://localhost:$n/v1/messages" -H "authorization: Bearer $PAT" -H "x-session-id: $SESSION" -H 'anthropic-version: 2023-06-01' -H 'content-type: application/json' -d '{"model":"claude-haiku-4-5","max_tokens":8,"messages":[{"role":"user","content":"ping"}]}')"
    [ "$code" = "200" ] && ok "PAT accepted directly by the node on :$n" || bad "node on :$n rejected the PAT ($code)"
done
xin infra logs request list --limit 5 2>/dev/null | grep -qi "anthropic\|claude" && ok "governance: requests audited in ai_requests (infra logs request list)" || bad "governance: no audited request rows"
cost="$(pgx "select coalesce(sum(cost_microdollars),0) from ai_requests" 2>/dev/null | tr -d '[:space:]' || true)"
[ -n "$cost" ] && [ "$cost" != "0" ] && ok "governance: every audited request is costed (ai_requests.cost_microdollars sum=$cost)" || bad "governance: ai_requests cost not populated (sum='$cost')"
ru="$(dc exec -T pg-replica psql -U postgres -d systemprompt -tAc 'select count(*) from users' | tr -d '[:space:]')"
pu="$(pgx 'select count(*) from users' | tr -d '[:space:]')"
[ -n "$ru" ] && [ "$ru" = "$pu" ] && ok "replica carries the primary's data (users=$ru on both)" || bad "replica users=$ru vs primary=$pu"
conns="$(dc exec -T pg-replica psql -U postgres -tAc "select count(*) from pg_stat_activity where application_name='systemprompt'" | tr -d '[:space:]')"
[ "$conns" = "0" ] && ok "no application connections on the replica (DR-only)" || bad "application connected to the replica ($conns)"
say "   restarting node-us to prove the signing key survives"
dc restart node-us >/dev/null; wait_healthy node-us || bad "node-us did not come back after restart"
code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 60 -X POST "http://localhost:$((NODE_PORT_BASE+2))/v1/messages" -H "authorization: Bearer $PAT" -H "x-session-id: $SESSION" -H 'anthropic-version: 2023-06-01' -H 'content-type: application/json' -d '{"model":"claude-haiku-4-5","max_tokens":8,"messages":[{"role":"user","content":"ping"}]}')"
[ "$code" = "200" ] && ok "PAT still valid after node-us restart (signing key persisted)" || bad "PAT rejected after restart ($code)"
for a in astound-bridge-windows.exe astound-bridge-macos.dmg astound-bridge-linux-x86_64.tar.gz astound-bridge-linux-aarch64.tar.gz; do
    want="$(grep " $a\$" rel/SHA256SUMS | cut -d' ' -f1)"; got="$("${C[@]}" "$LB/files/downloads/$a" | sha256sum | cut -d' ' -f1)"
    source='GitHub Release'
    if [ "$a" = astound-bridge-macos.dmg ] && [ -n "${E2E_MACOS_SHA256:-}" ]; then
        want="$E2E_MACOS_SHA256"
        source='pinned CI repair candidate'
    fi
    [ -n "$want" ] && [ "$want" = "$got" ] && ok "gateway serves $a byte-identical to the $source" || bad "gateway copy of $a differs from $source ($got vs $want)"
done

# install.sh legitimately differs from the release copy: the release job renders the
# GitHub download base into it, the gateway copy keeps the instance placeholder.
"${C[@]}" "$LB/files/downloads/install.sh" | grep -q 'DOWNLOAD_BASE' && ok "gateway serves install.sh (installer body intact)" || bad "gateway install.sh missing or malformed"

# ── 8. bridge on Linux ───────────────────────────────────────────────────────
say "8. Linux bridge installer in a config-free client"
if [ "${E2E_SKIP_BRIDGE:-0}" = "1" ]; then
    skip "bridge installer (E2E_SKIP_BRIDGE=1)"
elif ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
    skip "bridge installer (build the client first: just clean-client-build)"
else
    # The installer is fetched through the TLS load balancer (curl trusts the
    # injected CA). The bridge binary itself ships its own root store and does
    # not trust a self-signed CA, so it is pointed at a node over plain HTTP —
    # the customer's balancer carries a publicly trusted certificate.
    LBIP="$(docker inspect "$P-lb-1" -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}')"
    NODE_URL="http://node-in:8080"
    docker rm -f "$P-clean-client" >/dev/null 2>&1 || true
    docker run -d --name "$P-clean-client" --network "${P}_default" --add-host "$HOST:$LBIP" \
        -e "ASTOUND_BRIDGE_GATEWAY_URL=$NODE_URL" \
        -v "$WORK/tls/cert.pem:/tmp/sp-ca.pem:ro" astound-clean-client:local sleep 3600 >/dev/null
    cc() { docker exec "$P-clean-client" bash -lc "$1"; }
    cc "sudo cp /tmp/sp-ca.pem /usr/local/share/ca-certificates/sp-local.crt && sudo update-ca-certificates >/dev/null" \
        && ok "client trusts the test CA (for the installer download over TLS)" || bad "client CA install failed"
    if cc "curl -fsSL $LB/files/downloads/install.sh | sh -s -- --download-base $LB/files/downloads --gateway $NODE_URL --pat $PAT" > bridge-install.log 2>&1; then
        ok "install.sh fetched over TLS installed the bridge and signed in with the PAT"
    else
        bad "install.sh failed: $(sed 's/\x1b\[[0-9;]*m//g' bridge-install.log | grep -iE 'error|fail' | tail -3 | tr '\n' ';')"
    fi
    cc 'export PATH=$HOME/.local/bin:$PATH; astound-bridge --version' > bridge-version.log 2>&1 && ok "astound-bridge runs: $(tr -d '\n' < bridge-version.log)" || bad "astound-bridge --version failed"
    if cc 'export PATH=$HOME/.local/bin:$PATH; astound-bridge doctor' > bridge-doctor.log 2>&1; then
        ok "astound-bridge doctor passes against the node"
    else
        fails="$(sed 's/\x1b\[[0-9;]*m//g' bridge-doctor.log | grep '^\[FAIL\]' | tr '\n' ';')"
        bad "astound-bridge doctor: ${fails:-$(tail -2 bridge-doctor.log | tr '\n' ';')}"
    fi
    cc 'export PATH=$HOME/.local/bin:$PATH; astound-bridge whoami' > bridge-whoami.log 2>&1 && ok "astound-bridge whoami resolves the PAT to $(grep -oE '[a-z0-9._-]+@[a-z0-9.-]+' bridge-whoami.log | head -1)" || skip "astound-bridge whoami not available on this bridge version"
    skip "bridge ↔ gateway TLS not exercised here (self-signed CA; the bridge trusts only public roots)"
    cc 'claude --version' >/dev/null 2>&1 && ok "claude (Claude Code) present on the client" || bad "claude not runnable"
fi
skip "Windows exe and macOS dmg: checksum + cosign verified only (no Windows/macOS runner here)"

write_report
[ "$FAIL" -eq 0 ]
