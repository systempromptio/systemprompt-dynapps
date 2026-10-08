#!/usr/bin/env bash
# Pre-boot validation, one stage at a time, so a failed deployment names the
# stage that is wrong before any container starts. Exit 0 only when every
# stage passes. Pair with verify-node.sh (post-boot) for the whole picture.
#
# Usage:
#   preflight.sh --image ghcr.io/astound-digital/systemprompt-astound:X.Y.Z \
#                --profile-dir /etc/astound/profile \
#                [--database-url 'postgres://...']   # defaults to secrets.json database_url
#                [--external-url https://ai.example.com]   # adds the DNS + TLS stage
#
# Stages: tools → image (pull, version, signature) → database (reach, TLS,
# primary, extensions, role rights) → profile (files, ownership, schema inside
# the image, secrets present and well-formed, URL/issuer agreement) → DNS/TLS.
# Portable across macOS and Linux (docker, jq, openssl, curl; psql optional —
# database checks run inside the image when psql is absent).
set -uo pipefail

IMAGE="" PROFILE_DIR="" DB_URL="" EXTERNAL_URL=""
while [ $# -gt 0 ]; do
    case "$1" in
        --image) IMAGE="$2"; shift 2 ;;
        --profile-dir) PROFILE_DIR="$2"; shift 2 ;;
        --database-url) DB_URL="$2"; shift 2 ;;
        --external-url) EXTERNAL_URL="${2%/}"; shift 2 ;;
        -h|--help) sed -n '2,16p' "$0"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done
[ -n "$IMAGE" ] || { echo "--image is required" >&2; exit 2; }
[ -n "$PROFILE_DIR" ] || { echo "--profile-dir is required" >&2; exit 2; }

PASS=0 FAIL=0
ok()   { printf '  PASS  %s\n' "$1"; PASS=$((PASS+1)); }
bad()  { printf '  FAIL  %s\n' "$1"; FAIL=$((FAIL+1)); }
stage(){ printf '\n── %s ──\n' "$1"; }
# Run the CLI or psql from inside the image so the host needs nothing installed.
inimg() { docker run --rm --entrypoint "$1" "${@:2}"; }
sql()   { docker run --rm --entrypoint psql "$IMAGE" "$DB_URL" -tAc "$1" 2>&1; }

stage "tools"
for t in docker jq openssl curl; do command -v "$t" >/dev/null && ok "$t present" || bad "$t missing"; done
docker info >/dev/null 2>&1 && ok "docker daemon reachable" || bad "docker daemon not reachable (is the user in the docker group?)"

stage "image: $IMAGE"
VERSION="${IMAGE##*:}"
if docker pull -q "$IMAGE" >/dev/null 2>&1; then
    ok "pulled (registry credentials and package access are right)"
else
    bad "pull failed: run 'docker login ghcr.io' with a token that has read:packages and access to the package"
fi
label="$(docker image inspect "$IMAGE" -f '{{index .Config.Labels "org.opencontainers.image.version"}}' 2>/dev/null)"
[ -n "$label" ] && [ "$label" = "$VERSION" ] && ok "image label version $label matches the tag" || bad "image version label '$label' ≠ tag '$VERSION'"
bv="$(inimg /app/bin/systemprompt "$IMAGE" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
[ "$bv" = "$VERSION" ] && ok "binary inside the image reports $bv" || bad "binary reports '$bv', expected $VERSION"
for b in systemprompt-mcp-agent; do
    docker run --rm --entrypoint test "$IMAGE" -x "/app/bin/$b" 2>/dev/null && ok "MCP binary $b present" || bad "MCP binary $b missing (boot would fail)"
done
n="$(docker run --rm --entrypoint sh "$IMAGE" -c 'ls /app/storage/files/downloads | wc -l' 2>/dev/null | tr -d ' ')"
[ "${n:-0}" -ge 9 ] && ok "client downloads baked in ($n files)" || bad "client downloads incomplete in the image ($n files)"
if command -v cosign >/dev/null; then
    cosign verify --certificate-identity-regexp='https://github.com/Astound-Digital/systemprompt-astound/' \
        --certificate-oidc-issuer=https://token.actions.githubusercontent.com "$IMAGE" >/dev/null 2>&1 \
        && ok "cosign signature verifies" || bad "cosign signature does not verify"
else
    printf '  SKIP  cosign not installed; signature not verified\n'
fi

stage "profile: $PROFILE_DIR"
P="$PROFILE_DIR/profile.yaml"; S="$PROFILE_DIR/secrets.json"
[ -f "$P" ] && ok "profile.yaml present" || bad "profile.yaml missing"
[ -f "$S" ] && ok "secrets.json present" || bad "secrets.json missing"
if [ -f "$P" ]; then
    # The rendered profile keeps the template's version pin on line 1; it must
    # be the image's release, because the schema the loader applies is that
    # release's. The catalog and routes are not checked here: they are inside
    # the image, not in this directory.
    tv="$(sed -n '1s/^# template-version: \([0-9.][0-9.]*\)$/\1/p' "$P")"
    if [ -z "$tv" ]; then
        bad "profile.yaml has no '# template-version:' line — it was not rendered from the $VERSION bundle"
    elif [ "$tv" = "$VERSION" ]; then
        ok "profile.yaml was rendered from the $VERSION template"
    else
        bad "profile.yaml was rendered from the $tv template but the image is $VERSION — re-render from the $VERSION bundle"
    fi
    if grep -Eq '^(providers|gateway):' "$P"; then
        bad "profile.yaml still carries a top-level providers:/gateway: section — the catalog and routes ship inside the image since 0.44.0; delete them from the profile"
    else
        ok "profile.yaml carries no provider catalog or gateway routes (they are in the image)"
    fi
fi
if [ -f "$S" ]; then
    jq -e . "$S" >/dev/null 2>&1 && ok "secrets.json is valid JSON" || bad "secrets.json is not valid JSON"
    perm="$(stat -c '%a %u' "$S" 2>/dev/null || stat -f '%Lp %u' "$S")"
    case "$perm" in "600 1000") ok "secrets.json is 0600 owned by uid 1000" ;; *) bad "secrets.json permissions/owner '$perm' (want 600, uid 1000: the container runs as app=1000)" ;; esac
    for k in database_url oauth_at_rest_pepper manifest_signing_secret_seed signing_key_pem; do
        [ -n "$(jq -r --arg k "$k" '.[$k] // empty' "$S")" ] && ok "secret $k present" || bad "secret $k missing (SECRETS.md)"
    done
    prov=0; for k in vertex anthropic gemini; do [ -n "$(jq -r --arg k "$k" '.[$k] // empty' "$S")" ] && prov=1; done
    [ "$prov" = 1 ] && ok "at least one model provider key present" || bad "no provider key (vertex / anthropic / gemini)"
    plen="$(jq -r '.oauth_at_rest_pepper // ""' "$S" | wc -c)"; [ "$plen" -gt 32 ] && ok "pepper ≥ 32 chars" || bad "oauth_at_rest_pepper shorter than 32 chars"
    slen="$(jq -r '.manifest_signing_secret_seed // ""' "$S" | base64 -d 2>/dev/null | wc -c | tr -d ' ')"
    [ "$slen" = "32" ] && ok "manifest seed decodes to 32 bytes" || bad "manifest_signing_secret_seed must be base64 of exactly 32 bytes (got $slen)"
    if jq -r '.signing_key_pem // ""' "$S" | base64 -d 2>/dev/null | openssl pkey -noout 2>/dev/null; then
        ok "signing_key_pem decodes to a valid private key"
    else
        bad "signing_key_pem is not base64 of a PKCS#8 PEM"
    fi
    [ -n "$DB_URL" ] || DB_URL="$(jq -r '.database_url // empty' "$S")"
fi
if [ -f "$P" ]; then
    ext="$(sed -n 's/^  api_external_url: //p' "$P" | head -1)"; iss="$(sed -n 's/^  jwt_issuer: //p' "$P" | head -1)"
    [ -n "$ext" ] && [ "$ext" = "$iss" ] && ok "api_external_url and jwt_issuer agree ($ext)" || bad "api_external_url '$ext' ≠ jwt_issuer '$iss'"
    case "$ext" in https://*) ok "external URL is https" ;; *) bad "external URL must be https (Secure cookie, SSO callback)" ;; esac
    grep -q "^  email: " "$P" && ok "system_admin.email set" || bad "system_admin.email missing (bootstrap fails)"
    grep -A1 "^  trusted_proxies:" "$P" | grep -q "^  - " && ok "trusted_proxies has at least one CIDR" || bad "trusted_proxies empty"
    if [ -n "$EXTERNAL_URL" ] && [ "$EXTERNAL_URL" != "$ext" ]; then bad "--external-url $EXTERNAL_URL ≠ profile api_external_url $ext"; fi
    # The mapping that matters: the profile the image will load is the one on
    # this disk. Validate it with the image's own loader, from the same mount.
    if docker run --rm --entrypoint /app/bin/systemprompt -v "$PROFILE_DIR:/p:ro" -e SYSTEMPROMPT_PROFILE=/p/profile.yaml "$IMAGE" \
         admin config validate --strict /p/profile.yaml >/dev/null 2>&1; then
        ok "image's loader accepts profile.yaml (strict schema)"
    else
        bad "image's loader rejects profile.yaml: docker run --rm --entrypoint /app/bin/systemprompt -v $PROFILE_DIR:/p:ro -e SYSTEMPROMPT_PROFILE=/p/profile.yaml $IMAGE admin config validate --strict /p/profile.yaml"
    fi
fi

stage "database"
if [ -z "$DB_URL" ]; then
    bad "no database URL (pass --database-url or fix secrets.json)"
else
    host="$(printf '%s' "$DB_URL" | sed -E 's|^[a-z]+://[^@]*@([^:/?]+).*|\1|')"
    case "$DB_URL" in *sslmode=require*|*sslmode=verify*) ok "connection string requires TLS" ;; *) bad "connection string lacks sslmode=require" ;; esac
    if out="$(sql 'select 1')" && [ "$(printf '%s' "$out" | tr -d '[:space:]')" = "1" ]; then
        ok "reachable and authenticated as the application role ($host)"
        v="$(sql 'show server_version' | tr -d ' ')"; maj="${v%%.*}"
        [ "${maj:-0}" -ge 16 ] && ok "PostgreSQL $v (≥ 16)" || bad "PostgreSQL '$v' is below 16"
        r="$(sql 'select pg_is_in_recovery()' | tr -d ' ')"; [ "$r" = "f" ] && ok "target is the primary (not a standby)" || bad "target is a read replica; boot refuses it"
        for e in vector uuid-ossp pgcrypto; do
            c="$(sql "select count(*) from pg_extension where extname='$e'" | tr -d ' ')"
            [ "$c" = "1" ] && ok "extension $e installed" || bad "extension $e missing (provision.sql; on OCI enable it on the DB system first)"
        done
        if sql 'create table if not exists _preflight_probe(id int); drop table _preflight_probe' >/dev/null 2>&1; then
            ok "role can create and drop tables (migrations will work)"
        else
            bad "role cannot create tables in the database (GRANT ALL ON SCHEMA public)"
        fi
        db="$(sql 'select current_database()' | tr -d ' ')"; [ "$db" = "systemprompt" ] && ok "database name is systemprompt" || printf '  NOTE  database is %s\n' "$db"
        mig="$(sql "select count(*) from information_schema.tables where table_name='extension_migrations'" | tr -d ' ')"
        [ "$mig" = "1" ] && printf '  NOTE  existing installation: migration ledger present, boot will migrate forward\n' || printf '  NOTE  empty database: first boot will create the schema\n'
    else
        bad "cannot connect: $(printf '%s' "$out" | head -2 | tr '\n' ' ')"
    fi
fi

if [ -n "$EXTERNAL_URL" ]; then
    stage "dns and tls: $EXTERNAL_URL"
    h="${EXTERNAL_URL#https://}"; h="${h%%/*}"; hostonly="${h%%:*}"
    getent hosts "$hostonly" >/dev/null 2>&1 || nslookup "$hostonly" >/dev/null 2>&1 && ok "$hostonly resolves" || bad "$hostonly does not resolve"
    if echo | openssl s_client -connect "${h}${h##*:*}" -servername "$hostonly" 2>/dev/null | grep -q "Verify return code: 0"; then
        ok "TLS certificate chain verifies"
    else
        printf '  NOTE  TLS chain not verified from here (balancer not up yet, or private CA)\n'
    fi
fi

printf '\n── %s pass, %s fail ──\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
