---
title: "Deploy: One Node and PostgreSQL"
description: "Run the published Dynapps gateway image on one Linux machine against PostgreSQL, from pulling the image to the first admin sign-in."
author: "Dynapps"
slug: "deploy"
keywords: "deploy, self-host, docker, docker compose, postgresql, ghcr, image, profile, secrets, single node, install, upgrade"
kind: "guide"
public: true
tags: ["documentation", "deployment", "configuration"]
published_at: "2026-09-02"
updated_at: "2026-10-07"
after_reading_this:
  - "Pull and verify the published image"
  - "Provision PostgreSQL with the two required extensions"
  - "Render the profile and secrets once and start the node"
  - "Verify the node and sign in as the first admin"
related_playbooks:
  - title: "Deploy: Validate Every Stage"
    url: "/documentation/deploy-troubleshooting"
  - title: "Deploy on Oracle Cloud"
    url: "/documentation/deploy-oracle-cloud"
  - title: "Expose Your Instance Remotely"
    url: "/documentation/remote-access"
  - title: "Downloads"
    url: "/documentation/downloads"
  - title: "Proposed: Target Deployment Architecture"
    url: "/documentation/deploy-target-architecture"
---

# Deploy: One Node and PostgreSQL

This page takes one Linux machine with Docker from nothing to a running,
verified gateway. It is the same procedure the published bundle in
`deploy/self-host/` of the repository automates, and the three-node layout
grows from it without redoing any step. On Oracle Cloud, read
[Deploy on Oracle Cloud](/documentation/deploy-oracle-cloud) alongside this
page for the OCI resources each step maps to.

The version in every command below is the one this gateway runs
(`@RELEASE_VERSION@`); the page is rendered from the same build, so it cannot
name a release the gateway is not on.

## What you need

| Item | Requirement |
|------|-------------|
| Machine | Linux (amd64 or arm64), Docker Engine 24+ with Compose v2, 2 vCPU and 4 GB RAM minimum, outbound HTTPS to `ghcr.io`, `aiplatform.googleapis.com`, `oauth2.googleapis.com` and `api.anthropic.com` |
| Image access | a GitHub token with `read:packages`, issued by systemprompt.io. The image is private |
| PostgreSQL | version 16 or newer, reachable from the machine on 5432 with TLS. The `uuid-ossp` and `pgcrypto` extensions must be available. The bundled option runs PostgreSQL 18 beside the app |
| Model provider | a Google service-account JSON with `roles/aiplatform.user` on one GCP project (every `claude-*` request is served from Vertex AI), plus an Anthropic API key for the failover |
| Public hostname | an HTTPS address with TLS terminated by nginx on the machine or by a load balancer in front of it. HTTPS is mandatory: the sign-in cookie is `Secure` and SSO callbacks must match the public HTTPS URL |

## 1. Get the bundle and the image

```bash
git clone https://github.com/systempromptio/systemprompt-dynapps.git
cd systemprompt-dynapps/deploy/self-host
echo "$GHCR_TOKEN" | docker login ghcr.io -u <github-user> --password-stdin
```

The image is `ghcr.io/systempromptio/systemprompt-dynapps:<version>`. Always
pin a version; `latest` follows the main branch.

Images built by the release pipeline are cosign-signed, and this verifies one:

```bash
cosign verify \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-dynapps/' \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  ghcr.io/systempromptio/systemprompt-dynapps:@RELEASE_VERSION@
```

## 2. Provision the database

Skip this step if you use the bundled PostgreSQL: its first start installs
the extensions itself.

For a database you manage, run `provision.sql` once as the administrative
user. It creates the `systemprompt` role and database and installs the two
extensions (`uuid-ossp`, `pgcrypto`), which need rights the application role
never gets. pgvector is not required:

```bash
psql "postgres://<admin>:<pw>@<db-host>:5432/postgres?sslmode=require" \
  -v pw="<application-db-password>" -f provision.sql
```

The application connection string is then:

```
postgres://systemprompt:<application-db-password>@<db-host>:5432/systemprompt?sslmode=require
```

Each node opens up to 50 connections by default. Migrations run at boot and
are forward-only, so take a backup before every upgrade.

## 3. Render the profile and secrets

Run once. The result is a directory with `profile.yaml` and `secrets.json`;
the second file holds the signing key, so keep it in your vault. The profile
holds only what differs per deployment (about a hundred lines); the provider
catalog and gateway routes ship inside the image, and the template's first
line pins it to the image release, which `--image` and `preflight.sh` check.

```bash
./render-profile.sh --out ./profile \
  --external-url https://ai.example.com \
  --admin-email you@example.com \
  --trusted-proxies 127.0.0.0/8 \
  --database-url 'postgres://systemprompt:<pw>@<db-host>:5432/systemprompt?sslmode=require' \
  --anthropic-key sk-ant-... \
  --image ghcr.io/systempromptio/systemprompt-dynapps:@RELEASE_VERSION@
```

| Flag | Value |
|------|-------|
| `--external-url` | the HTTPS address users open. Every host-dependent setting is derived from it: the JWT issuer, the CORS origin, the URLs the instance advertises for itself, and the OAuth callback URLs |
| `--admin-email` | the first admin account, created at boot as user `admin` |
| `--trusted-proxies` | the address your TLS proxy or load balancer connects from. nginx on the same machine: `127.0.0.0/8`. Without it forwarded headers are ignored and rate limiting can refuse callers |
| `--database-url` | from step 2, or `postgres://systemprompt:<pw>@postgres:5432/systemprompt` for the bundled database |
| `--anthropic-key` | the Anthropic API key, stored as the `anthropic` secret. It is the only model credential this installation uses: every `claude-*` request goes to Anthropic's API |
| `--secret name=value` | any other secret, repeatable. Optional ones (`dynapps_release_token`, `ghcr_pull_token`) are listed in `SECRETS.md` |
| `--image` | optional; validates the rendered profile inside the image before you start anything |

**Settle the hostname before the first boot.** Changing `--external-url` later
is not a re-render and a restart: the JWT issuer moves with it, so every issued
session and bridge token stops validating, and every bridge has to be
connected again. The Entra ID Enterprise Application is registered against this
host too, so IT has to update its Identifier and Reply URL — see
[Entra ID SSO setup](/documentation/entra-id-sso-setup).

`render-profile.sh` also generates the three secrets that form the
instance's identity: the OAuth pepper, the manifest signing seed, and an RSA
signing key. Re-running it keeps them; it only rewrites what you pass.
`SECRETS.md` in the bundle lists every key, its owner, and what breaks
without it.

## 4. Start

```bash
export SYSTEMPROMPT_TAG=@RELEASE_VERSION@ POSTGRES_PASSWORD='<pw>'
docker compose -f docker-compose.single.yml pull
docker compose -f docker-compose.single.yml up -d        # own database: append `app`
docker compose -f docker-compose.single.yml ps           # wait for app: healthy
```

First boot waits for PostgreSQL, applies the schema, creates the admin user,
and renders the public site. The container runs as uid 1000 and mounts the
profile directory read-only.

Releases before the per-node render landed in the entrypoint skipped this
step on every node but the first. If `verify-node.sh` reports the public
site missing, render it by hand once the node is healthy:

```bash
docker compose -f docker-compose.single.yml exec \
  -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml \
  app systemprompt infra jobs run publish_pipeline
```

## 5. Put TLS in front

The node listens on port 8080 over HTTP. Terminate TLS in front of it and
forward these headers; the full nginx block is in
[Expose Your Instance Remotely](/documentation/remote-access):

| Rule | Value |
|------|-------|
| Upstream | `http://127.0.0.1:8080` |
| Headers | `Host`, `X-Forwarded-For`, `X-Forwarded-Proto: https` |
| Streaming | response buffering off, read timeout 300 s on `/v1/` and `/api/` |
| Health check | `GET /api/v1/health`, HTTP 200 **and** body containing `"healthy"`. During boot the same path answers `{"status":"starting"}` |
| Block externally | `/metrics` and `/api/v1/health/detail` |
| Firewall | 8080 from the proxy only; 5000–5999 and 9000–9999 are internal servers and must not be reachable from outside the machine |

## 6. Verify and sign in

```bash
./verify-node.sh --url http://localhost:8080 \
  --container "$(docker compose -f docker-compose.single.yml ps -q app)" --version @RELEASE_VERSION@
```

The checklist covers health, the admin login page, the public site, every
client download against its checksum, anonymous rejection (`401`) on
`/v1/messages`, the binary version, the `systemprompt` MCP server on port
5010, the migration ledger and the admin user. It exits 0 only when every
check passes; resolve each failure before continuing.

Configure [Dynapps SSO](/documentation/entra-id-sso-setup) for the public hostname, sign in at `https://ai.example.com/admin/login`, and hand users the
[Downloads](/documentation/downloads) page, which your own gateway serves.

## Upgrade and roll back

```bash
export SYSTEMPROMPT_TAG=<new-version>
docker compose -f docker-compose.single.yml pull && docker compose -f docker-compose.single.yml up -d
./verify-node.sh --version <new-version> ...
```

Versioned tags never move. Rolling back to the previous tag is the same
command and is safe only while the database has not been migrated forward;
otherwise restore the backup taken before the upgrade.

## Change configuration without rebuilding

The image is rebuilt once per release. Configuration should not wait for one.

`services/` is copied into the image at build time (`COPY services
/app/services`) and read once at boot, so a running node serves whatever it was
built with. To change it between releases, bind-mount your own copy over the
baked-in tree and restart. `SYSTEMPROMPT_SERVICES_PATH` is already
`/app/services` in the image, so the mount is all that is needed.

Mount the whole tree:

```yaml
services:
  app:
    volumes:
      - ./services:/app/services
```

Or a single file, leaving the rest of the shipped tree in place:

```yaml
      - ./services/ai/gateway.yaml:/app/services/ai/gateway.yaml
```

Then `docker compose up -d app`. **A restart is required** — the services tree
is parsed once during boot, so an edit to a mounted file does nothing until the
container restarts.

Copy the shipped tree out first so you are editing what the node actually runs,
rather than a tree that has drifted from the release:

```bash
docker compose cp app:/app/services ./services
```

### Two things to know before you mount

**Without a mount, edits made through the admin UI do not survive.** The admin
gateway editor writes `/app/services/ai/gateway.yaml` inside the container.
That is a real file and the change takes effect, but it lives in the
container's writable layer — replacing the container on the next `up -d` or
upgrade silently restores the baked-in file and the edit is gone, with nothing
reported. If you administer routes through the UI, mount the tree.

**Do not mount read-only if you use the admin UI.** A `:ro` mount is the safer
choice for a tree you manage in git and deploy from — nothing in the container
can change it, and what you committed is what runs. But the gateway editor then
fails on write. Choose one: git-managed and read-only, or UI-managed and
writable. Mounting read-only while expecting the UI to work is the one
combination that produces a confusing failure.

A malformed or missing file in a mounted tree fails the boot loudly, naming the
file it could not load — it does not start with the configuration silently
absent.

## Growing to more nodes

Copy the profile directory to each new machine unchanged and point every node
at the same PostgreSQL primary. Nodes no longer need to boot one at a time:

- **Migrate once.** The multi-node profile sets `database.migrate_on_boot:
  false`, so no node changes the schema as it starts. Exactly one process
  migrates it first: the image's one-shot `migrate` command
  (`docker compose -f docker-compose.node.yml run --rm app migrate`, which
  runs `infra db migrate` and exits).
  A node that skips migrations checks the schema at boot and refuses to start,
  naming each pending or drifted migration and the `infra db migrate
  --profile <name>` command, if the database is behind its binary.
- **Then boot in parallel.** Each replica claims its instance id under a
  database lock before registering its MCP servers, so concurrent boots no
  longer race. Each replica needs its own id: leave `server.instance_id`
  unset so `HOSTNAME` is used. A second process with a live id refuses to
  boot.
- **Roles.** `server.role` is `all` (the default), `gateway` (only `/v1`
  inference, the bridge, OAuth, discovery and health; no scheduler, MCP
  servers or agents, so add these freely) or `admin` (everything except the
  gateway, and all background work). A deployment needs at least one `all` or
  `admin` node.
- **Load shedding.** `server.max_in_flight` caps concurrent requests per
  process. Past it the node answers `503` with `Retry-After: 1`, and
  `/readyz` reports `{"status":"saturated"}`, so the balancer can shed load
  instead of queueing it. Unset means no ceiling.
- **Read-only root.** The server writes only to declared directories
  (`system.logs`, `storage.files`, `storage.exports`, `storage.data`,
  `storage.data/scratch`, plus `web.dist` and the services cache on `all` and
  `admin` nodes). Boot probes each one for the node's role and names any it
  cannot write. Git checkouts and marketplace import staging use
  `storage/data/scratch`, so a container can run with a read-only root and no
  writable `/tmp`.
- **Shared uploads.** With `storage: {backend: gcs, bucket, prefix?,
  public_read?, credentials?}`, uploads and generated images go to one Google
  Cloud Storage bucket instead of each node's disk. `credentials` is
  `workload_identity` (the default) or `{secret: NAME}`. The gateway journal
  and exports stay on the node.

The bundle's `README.md` covers the three-node layout, load balancers per
region, and the exact commands.
