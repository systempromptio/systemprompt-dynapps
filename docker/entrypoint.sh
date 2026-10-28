#!/bin/sh
# Container entrypoint for systemprompt-dynapps.
# Authors a profile via `systemprompt admin setup` on first boot,
# waits for Postgres, runs migrations, starts the server.
set -eu

# One-click platforms (Railway et al.) export unfilled template variables as
# empty strings; admin setup would record "" as a configured provider key.
# Treat blank as unset.
[ -n "${ANTHROPIC_API_KEY:-}" ] || unset ANTHROPIC_API_KEY
[ -n "${OPENAI_API_KEY:-}" ] || unset OPENAI_API_KEY
[ -n "${GEMINI_API_KEY:-}" ] || unset GEMINI_API_KEY
[ -n "${GITHUB_TOKEN:-}" ] || unset GITHUB_TOKEN
[ -n "${EXTERNAL_URL:-}" ] || unset EXTERNAL_URL

# Platform-neutral external URL. Render injects RENDER_EXTERNAL_URL; every
# other catalog template sets EXTERNAL_URL explicitly.
EXTERNAL_URL="${EXTERNAL_URL:-${RENDER_EXTERNAL_URL:-}}"

PROFILE_DIR="${SYSTEMPROMPT_PROFILE_DIR:-/app/.systemprompt/profiles/docker}"
PROFILE_FILE="$PROFILE_DIR/profile.yaml"
SECRETS_FILE="$PROFILE_DIR/secrets.json"

if [ -n "${SYSTEMPROMPT_PROFILE_DIR:-}" ]; then
    # A profile directory was supplied (e.g. bind-mounted air-gap profile).
    # Do not generate anything — just validate the expected files exist.
    if [ ! -f "$PROFILE_FILE" ]; then
        echo "ERROR: SYSTEMPROMPT_PROFILE_DIR is set but $PROFILE_FILE is missing." >&2
        exit 1
    fi
    if [ ! -f "$SECRETS_FILE" ]; then
        echo "ERROR: SYSTEMPROMPT_PROFILE_DIR is set but $SECRETS_FILE is missing." >&2
        exit 1
    fi
    # A supplied profile names its own database; the readiness probe below
    # must not fall back to the compose-only `postgres` hostname.
    if [ -z "${DATABASE_URL:-}" ]; then
        DATABASE_URL="$(jq -r '.database_url // empty' "$SECRETS_FILE")"
        if [ -z "$DATABASE_URL" ]; then
            echo "ERROR: $SECRETS_FILE has no database_url." >&2
            exit 1
        fi
    fi
    # Multi-node deployments share one signing key through the
    # `signing_key_pem` secret; a per-container file would mint tokens the
    # other nodes reject and would be lost on every restart.
    if [ "$(jq -r '.signing_key_pem // empty' "$SECRETS_FILE" | wc -c)" -le 1 ]; then
        echo "ERROR: $SECRETS_FILE has no signing_key_pem (deploy/self-host/README.md)." >&2
        exit 1
    fi
    SIGNING_KEY_FROM_SECRETS=1
else
    if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ] && [ -z "${GEMINI_API_KEY:-}" ]; then
        echo "ERROR: set at least one of ANTHROPIC_API_KEY, OPENAI_API_KEY, GEMINI_API_KEY in .env" >&2
        exit 1
    fi
    if [ -z "${DATABASE_URL:-}" ]; then
        echo "ERROR: DATABASE_URL is required." >&2
        exit 1
    fi
    if [ ! -f "$PROFILE_FILE" ] && [ -z "${SYSTEMPROMPT_ADMIN_EMAIL:-}" ]; then
        echo "ERROR: SYSTEMPROMPT_ADMIN_EMAIL is required on first boot (the administrator admin setup creates)." >&2
        exit 1
    fi

    if [ ! -f "$PROFILE_FILE" ]; then
        echo "Generating profile via admin setup..."
        # Default provider = first configured key (setup picks up the
        # ANTHROPIC/OPENAI/GEMINI_API_KEY env vars itself).
        if [ -n "${ANTHROPIC_API_KEY:-}" ]; then DEFAULT_PROVIDER=anthropic
        elif [ -n "${OPENAI_API_KEY:-}" ]; then DEFAULT_PROVIDER=openai
        else DEFAULT_PROVIDER=gemini
        fi
        /app/bin/systemprompt admin setup -e docker \
            --default-provider "$DEFAULT_PROVIDER" --yes --no-migrate

        # Setup authors a localhost dev profile; patch the parts the
        # container environment dictates.
        # 1. Bind publicly (Render/compose port detection needs 0.0.0.0).
        #    Overridable via HOST for platforms whose internal networking is
        #    IPv6-only (Railway healthchecks need HOST=::).
        # Quoted: bare "::" (IPv6 any) is invalid YAML.
        sed -i "s/^  host: 127\.0\.0\.1$/  host: \"${HOST:-0.0.0.0}\"/" "$PROFILE_FILE"
        # 1b. Binaries ship in /app/bin, not a cargo target dir.
        sed -i 's|^  bin: .*|  bin: /app/bin|' "$PROFILE_FILE"
        # 2. Point at the real database, not setup's generated localhost one.
        jq --arg db "$DATABASE_URL" '.database_url = $db' "$SECRETS_FILE" \
            > "$SECRETS_FILE.tmp" && mv "$SECRETS_FILE.tmp" "$SECRETS_FILE"
        # 2b. Optional read/write split: reads stay on DATABASE_URL, writes go
        #     to the primary named here (core refuses a standby as write target).
        if [ -n "${DATABASE_WRITE_URL:-}" ]; then
            jq --arg db "$DATABASE_WRITE_URL" '.database_write_url = $db' "$SECRETS_FILE" \
                > "$SECRETS_FILE.tmp" && mv "$SECRETS_FILE.tmp" "$SECRETS_FILE"
        fi
        # 2c. The gateway accounting journal and at-rest sealing refuse to
        #     start without encryption_master_key, and setup does not mint it.
        if [ -z "$(jq -r '.encryption_master_key // empty' "$SECRETS_FILE")" ]; then
            key="$(od -An -tx1 -N32 /dev/urandom | tr -d ' \n')"
            jq --arg key "$key" '.encryption_master_key = $key' "$SECRETS_FILE" \
                > "$SECRETS_FILE.tmp" && mv "$SECRETS_FILE.tmp" "$SECRETS_FILE"
        fi
        chmod 600 "$SECRETS_FILE"
        # 3. Advertise the public URL when the platform provides one
        #    (EXTERNAL_URL, or RENDER_EXTERNAL_URL via the fallback above).
        if [ -n "${EXTERNAL_URL:-}" ]; then
            sed -i "s|^  api_external_url: .*|  api_external_url: ${EXTERNAL_URL}|" "$PROFILE_FILE"
            sed -i "/^  cors_allowed_origins:/a\\  - ${EXTERNAL_URL}" "$PROFILE_FILE"
        fi
    fi
fi

export SYSTEMPROMPT_PROFILE="$PROFILE_FILE"

# Refresh image-shipped files on a /app/storage/files volume when the image
# changed. Shipped files are overwritten, uploads beside them are kept; a
# writable image layer (no volume) already carries the current copy.
SHIPPED_FILES=/app/share/storage-files
if [ -f "$SHIPPED_FILES.stamp" ] && [ -w /app/storage/files ]; then
    want="$(cat "$SHIPPED_FILES.stamp")"
    have="$(cat /app/storage/files/.image-stamp 2>/dev/null || true)"
    if [ "$want" != "$have" ]; then
        echo "Refreshing shipped storage files on the volume..."
        cp -a "$SHIPPED_FILES/." /app/storage/files/
        printf '%s\n' "$want" > /app/storage/files/.image-stamp
    fi
fi

# Probe DATABASE_URL directly when provided (managed Postgres, e.g. Render);
# fall back to the compose-style host/user/db vars otherwise.
if [ -n "${DATABASE_URL:-}" ]; then
    pg_probe() { pg_isready -d "$DATABASE_URL"; }
    echo "Waiting for Postgres at DATABASE_URL host..."
else
    PG_HOST="${PG_HOST:-postgres}"
    PG_USER="${PG_USER:-systemprompt}"
    PG_DB="${PG_DB:-systemprompt}"
    pg_probe() { pg_isready -h "$PG_HOST" -U "$PG_USER" -d "$PG_DB"; }
    echo "Waiting for Postgres at ${PG_HOST}..."
fi
i=0
until pg_probe >/dev/null 2>&1; do
    i=$((i + 1))
    if [ "$i" -ge 300 ]; then
        echo "ERROR: Postgres did not become ready within 300s." >&2
        exit 1
    fi
    sleep 1
done
echo "Postgres is ready."

if [ -z "${SIGNING_KEY_FROM_SECRETS:-}" ] && [ ! -f /app/signing_key.pem ]; then
    echo "Generating signing key..."
    /app/bin/systemprompt admin keys generate --output /app/signing_key.pem
fi

# A managed volume/database outlives the image, so a database seeded by an older
# tag can carry checksums for migrations that were since edited in the source
# tree. --repair-drift repairs that case, and only that case, then retries
# once; every other failure aborts boot with core's classified error and hint
# (a blind repair-and-retry used to repeat the same failure twice).
run_migrations() {
    echo "Running database migrations..."
    /app/bin/systemprompt infra db migrate --repair-drift
}

# Core boots no config plane: access control, governance and gateway policies
# reach the database only through `admin config apply`, run here wherever the
# migrations run (the one-shot migrate step, or a node that migrates on boot).
# The apply takes core's advisory lock, so replicas cannot race it. A conflict
# (an entity changed in both the image and the console) exits non-zero and is
# left for /admin/sync; the node still starts, but it does not start with a
# plane that was never applied.
apply_config() {
    echo "Applying the image's services/ declarations (admin config apply)..."
    /app/bin/systemprompt admin config apply \
        --reason "${CONFIG_APPLY_REASON:-image start: apply shipped services/}" \
        || echo "WARN: config apply reported conflicts; resolve them on /admin/sync." >&2
    /app/bin/check-config-applied.sh /app/bin/systemprompt
}

# `migrate` is the one-shot step for profiles with database.migrate_on_boot:
# false (deploy/self-host): migrate, ensure the admin, exit.
if [ "${1:-}" = "migrate" ]; then
    run_migrations
    /app/bin/systemprompt admin bootstrap
    apply_config
    exit 0
fi

migrate_on_boot() {
    awk '/^database:/ { in_db = 1; next }
         /^[^ #]/ { in_db = 0 }
         in_db && /^  migrate_on_boot:[[:space:]]*false/ { off = 1 }
         END { exit off ? 1 : 0 }' "$PROFILE_FILE"
}

if migrate_on_boot; then
    run_migrations
    apply_config
else
    echo "database.migrate_on_boot is false: skipping migrations (run the one-shot migrate step first)."
fi

# server.role from the profile: gateway nodes serve neither the console nor the
# public site, so they skip the admin bootstrap and go straight to serving.
# web/dist is rendered by the scheduler's bootstrap site_render on every node
# that runs the scheduler, before the server reports ready, so a gateway
# scale-out neither re-ingests content nor renders a site it never serves.
server_role() {
    awk '/^server:/ { in_server = 1; next }
         /^[^ #]/ { in_server = 0 }
         in_server && /^  role:/ { sub(/^  role:[[:space:]]*/, ""); sub(/[[:space:]]*(#.*)?$/, ""); gsub(/["\047]/, ""); print; exit }' "$PROFILE_FILE"
}
ROLE="$(server_role)"
ROLE="${ROLE:-all}"

if [ "$ROLE" = "gateway" ]; then
    echo "server.role is gateway: skipping admin bootstrap."
else
    echo "Ensuring bootstrap admin user..."
    /app/bin/systemprompt admin bootstrap
    if migrate_on_boot; then
        apply_config
    fi
fi

echo "Starting services..."
exec /app/bin/systemprompt infra services start --foreground
