# Start here: one node, one database

Ten minutes on one Linux VM with Docker. Everything below carries over to
the three-node layout in `README.md` unchanged: the profile directory you
render here is the one every later node mounts.

## 1. Pull the image

```bash
echo "$GHCR_TOKEN" | docker login ghcr.io -u <github-user> --password-stdin
```

## 2. Render the profile (once)

```bash
cd deploy/self-host
./render-profile.sh --out ./profile \
  --external-url https://ai.example.com \
  --admin-email you@example.com \
  --trusted-proxies 127.0.0.0/8 \
  --database-url 'postgres://systemprompt:<db-password>@postgres:5432/systemprompt' \
  --anthropic-key sk-ant-...
```

- `--external-url` is the HTTPS address users will open. Put nginx or your
  balancer in front with a real certificate; the settings it needs are in
  `README.md` §4. For a first smoke test on a laptop, `https://localhost:8443`
  with a self-signed proxy works.
- `--trusted-proxies` is where that proxy connects from. Same host: `127.0.0.0/8`.
- Own PostgreSQL instead of the bundled one: run `provision.sql` against it as
  the admin user, and put its URL (with `?sslmode=require`) in `--database-url`.
- Keep `./profile/secrets.json` safe. It contains the signing key; losing it
  logs everyone out, leaking it is a credential leak.

## 2b. Prove the inputs before starting anything

```bash
./preflight.sh --image ghcr.io/astound-digital/systemprompt-astound:0.60.0 \
  --profile-dir ./profile --external-url https://ai.example.com
```

Every line must pass; a failure names the stage and the fix.

## 3. Start

```bash
export SYSTEMPROMPT_TAG=0.60.0 POSTGRES_PASSWORD='<db-password>'
docker compose -f docker-compose.single.yml pull
docker compose -f docker-compose.single.yml up -d          # own database: add `app` at the end
docker compose -f docker-compose.single.yml ps              # wait for app: healthy
```

First boot runs the migrations, creates the admin user and renders the site.
If `verify-node.sh` reports the public site missing (entrypoints before the
per-node render skipped it), run this once after `healthy`:

```bash
docker compose -f docker-compose.single.yml exec \
  -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml \
  app systemprompt infra jobs run publish_pipeline
```

## 4. Check

```bash
./verify-node.sh --url http://localhost:8080 --container "$(docker compose -f docker-compose.single.yml ps -q app)" --version 0.60.0
```

Then sign in at `https://ai.example.com/admin/login` as the admin e-mail you
gave, register a passkey, and hand users the
`https://ai.example.com/documentation/downloads` page.

## 5. Upgrade

```bash
export SYSTEMPROMPT_TAG=<new-version>
docker compose -f docker-compose.single.yml pull && docker compose -f docker-compose.single.yml up -d
```

Versioned tags never move; migrations are forward-only, so back up the
database before upgrading.

## Growing to three nodes

Copy `./profile` to each new node, run `provision.sql` on the shared
primary if you are moving off the bundled database, and follow `README.md`
from §4. Boot nodes one at a time.
