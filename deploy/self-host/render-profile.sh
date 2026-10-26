#!/usr/bin/env bash
# Render the profile directory every node mounts: profile.yaml from the
# template, secrets.json from your inputs plus the cluster's generated
# cryptographic identity (pepper, manifest seed, RSA signing key).
#
# Run it ONCE, on one machine, and copy the resulting directory to every
# node unchanged. Re-running against an existing directory keeps the
# generated values (so nodes stay in agreement) and only rewrites what you
# pass in.
#
# Usage:
#   render-profile.sh --out /etc/dynapps/profile \
#     --external-url https://ai.example.com --admin-email platform-admin@example.com \
#     --trusted-proxies 10.0.0.0/8,10.1.0.0/16 \
#     --database-url 'postgres://systemprompt:pw@db:5432/systemprompt?sslmode=require' \
#     --anthropic-key sk-ant-... \
#     [--secret name=value ...] [--image ghcr.io/systempromptio/systemprompt-dynapps:0.44.0]
#
# Portable: bash, openssl, jq. --image checks the template's version pin
# against the image label and validates the rendered profile inside the
# published image (needs docker). The provider catalog and gateway routes
# are not in the profile: they ship inside the image.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="" EXTERNAL_URL="" TRUSTED="" DATABASE_URL="" ANTHROPIC="" IMAGE="" ADMIN_EMAIL=""
EXTRA=()
while [ $# -gt 0 ]; do
    case "$1" in
        --out) OUT="$2"; shift 2 ;;
        --external-url) EXTERNAL_URL="$2"; shift 2 ;;
        --admin-email) ADMIN_EMAIL="$2"; shift 2 ;;
        --trusted-proxies) TRUSTED="$2"; shift 2 ;;
        --database-url) DATABASE_URL="$2"; shift 2 ;;
        --anthropic-key) ANTHROPIC="$2"; shift 2 ;;
        --secret) EXTRA+=("$2"); shift 2 ;;
        --image) IMAGE="$2"; shift 2 ;;
        -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done
[ -n "$OUT" ] || { echo "--out is required" >&2; exit 2; }
[ -n "$EXTERNAL_URL" ] || { echo "--external-url is required" >&2; exit 2; }
[ -n "$TRUSTED" ] || { echo "--trusted-proxies is required (the load balancer CIDRs)" >&2; exit 2; }
[ -n "$ADMIN_EMAIL" ] || { echo "--admin-email is required (the bootstrap admin account; boot fails without it)" >&2; exit 2; }
case "$EXTERNAL_URL" in
    https://*) ;;
    *) echo "--external-url must be https:// — the sign-in cookie is Secure and WebAuthn needs a secure context" >&2; exit 2 ;;
esac
EXTERNAL_URL="${EXTERNAL_URL%/}"

mkdir -p "$OUT"
PROFILE="$OUT/profile.yaml"
SECRETS="$OUT/secrets.json"

proxies=""
IFS=',' read -ra cidrs <<< "$TRUSTED"
for c in "${cidrs[@]}"; do
    c="$(echo "$c" | tr -d '[:space:]')"
    [ -n "$c" ] && proxies="${proxies}  - ${c}\n"
done
proxies="${proxies}  - 127.0.0.0/8\n  - ::1/128"

awk -v url="$EXTERNAL_URL" -v email="$ADMIN_EMAIL" -v proxies="$proxies" '
    { gsub(/@EXTERNAL_URL@/, url); gsub(/@ADMIN_EMAIL@/, email) }
    /^@TRUSTED_PROXIES@$/ { gsub(/\\n/, "\n", proxies); print proxies; next }
    { print }
' "$HERE/profile.yaml.template" > "$PROFILE"

if [ -f "$SECRETS" ]; then
    current="$(cat "$SECRETS")"
else
    current='{}'
fi
gen() { openssl rand -base64 48 | tr -d '\n=/+' | cut -c1-48; }
pepper="$(echo "$current" | jq -r '.oauth_at_rest_pepper // empty')"
seed="$(echo "$current" | jq -r '.manifest_signing_secret_seed // empty')"
pem="$(echo "$current" | jq -r '.signing_key_pem // empty')"
key="$(echo "$current" | jq -r '.encryption_master_key // empty')"
[ -n "$pepper" ] || pepper="$(gen)"
# Ed25519 seed: the loader decodes standard base64 and requires exactly 32 bytes.
[ -n "$seed" ] || seed="$(openssl rand -base64 32)"
# 32 bytes as 64 hex characters: seals at-rest secrets and the gateway journal.
[ -n "$key" ] || key="$(openssl rand -hex 32)"
# The loader base64-decodes this secret into the PEM (same envelope the Fly
# deploy uses), so the value is base64 of the PKCS#8 file, on one line.
if [ -z "$pem" ]; then
    pem="$(openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 2>/dev/null | openssl base64 -A)"
fi

next="$(echo "$current" | jq \
    --arg pepper "$pepper" --arg seed "$seed" --arg pem "$pem" --arg key "$key" \
    '.oauth_at_rest_pepper = $pepper | .manifest_signing_secret_seed = $seed | .signing_key_pem = $pem | .encryption_master_key = $key')"
[ -n "$DATABASE_URL" ] && next="$(echo "$next" | jq --arg v "$DATABASE_URL" '.database_url = $v')"
[ -n "$ANTHROPIC" ]    && next="$(echo "$next" | jq --arg v "$ANTHROPIC" '.anthropic = $v')"
for kv in "${EXTRA[@]+"${EXTRA[@]}"}"; do
    k="${kv%%=*}"; v="${kv#*=}"
    next="$(echo "$next" | jq --arg k "$k" --arg v "$v" '.[$k] = $v')"
done

umask 077
printf '%s\n' "$next" > "$SECRETS"
chmod 0600 "$SECRETS"
chmod 0644 "$PROFILE"

missing=""
for k in database_url anthropic; do
    [ "$(echo "$next" | jq -r --arg k "$k" '.[$k] // empty')" ] || missing="$missing $k"
done
if [ -n "$missing" ]; then
    echo "WARN: secrets.json still lacks:$missing (see SECRETS.md; the node will not boot without them)" >&2
fi

echo "rendered $PROFILE and $SECRETS"
echo "  external url : $EXTERNAL_URL"
echo "  admin email  : $ADMIN_EMAIL"
echo "  trusted CIDRs: $TRUSTED"
echo "  secrets keys : $(echo "$next" | jq -r 'keys | join(", ")')"

if [ -n "$IMAGE" ]; then
    # The template is pinned to the image release on its first line; the
    # schema it renders is that release's. A mismatch is refused here rather
    # than surfacing as an "unknown field" on some node.
    tv="$(sed -n '1s/^# template-version: \([0-9.][0-9.]*\)$/\1/p' "$HERE/profile.yaml.template")"
    iv="$(docker image inspect "$IMAGE" -f '{{index .Config.Labels "org.opencontainers.image.version"}}' 2>/dev/null || true)"
    if [ -z "$iv" ]; then
        docker pull -q "$IMAGE" >/dev/null 2>&1 || true
        iv="$(docker image inspect "$IMAGE" -f '{{index .Config.Labels "org.opencontainers.image.version"}}' 2>/dev/null || true)"
    fi
    if [ -z "$tv" ]; then
        echo "ERROR: profile.yaml.template has no '# template-version:' line; use the bundle that matches $IMAGE" >&2
        exit 1
    fi
    if [ -n "$iv" ] && [ "$iv" != "$tv" ]; then
        echo "ERROR: profile.yaml.template is version $tv but $IMAGE is $iv — download the bundle for $iv" >&2
        exit 1
    fi
    [ -n "$iv" ] || echo "WARN: $IMAGE carries no version label; template version $tv not cross-checked" >&2
    echo "validating profile schema inside $IMAGE ..."
    # secrets.json is 0600 and owned by whoever ran this script; the image's
    # own user is uid 1000, so validate as the caller rather than as `app`.
    docker run --rm --entrypoint /app/bin/systemprompt \
        --user "$(id -u):$(id -g)" \
        -v "$OUT:/p:ro" -e SYSTEMPROMPT_PROFILE=/p/profile.yaml "$IMAGE" \
        admin config validate --strict /p/profile.yaml
fi
echo "Copy $OUT to every node unchanged (uid 1000, secrets.json 0600)."
