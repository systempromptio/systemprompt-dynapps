---
title: "Deploy: One Node and PostgreSQL"
description: "Run the published Astound Digital gateway image on one Linux machine against PostgreSQL, from pulling the image to the first admin sign-in."
author: "Astound Digital"
slug: "deploy"
keywords: "deploy, self-host, docker, docker compose, postgresql, pgvector, ghcr, image, profile, secrets, single node, install"
kind: "guide"
public: true
tags: ["documentation", "deployment", "configuration"]
published_at: "2026-09-02"
updated_at: "2026-09-02"
after_reading_this:
  - "Pull and verify the published image"
  - "Provision PostgreSQL with the three required extensions"
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
---

# Deploy: One Node and PostgreSQL

This page takes one Linux machine with Docker from nothing to a running,
verified gateway. It is the same procedure the Astound-published bundle in
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
| Machine | Linux (amd64 or arm64), Docker Engine 24+ with Compose v2, 2 vCPU and 4 GB RAM minimum, outbound HTTPS |
| Image access | a GitHub token with `read:packages`, issued by Astound. The image is private |
| PostgreSQL | version 16 or newer, reachable from the machine on 5432 with TLS. The `vector`, `uuid-ossp` and `pgcrypto` extensions must be available. The bundled option runs `pgvector/pgvector:pg18` beside the app |
| Model provider | an Anthropic API key (OpenAI or Gemini also work; `claude-*` models route to Anthropic) |
| Knowledge bank | an AWS Bedrock long-term API key with model access in `eu-central-1`. The knowledge-bank server refuses to start without it, and that stops the node |
| Public hostname | an HTTPS address with TLS terminated by nginx on the machine or by a load balancer in front of it. HTTPS is mandatory: the sign-in cookie is `Secure` and passkeys require a secure context |

## 1. Get the bundle and the image

```bash
git clone https://github.com/Astound-Digital/systemprompt-astound.git
cd systemprompt-astound/deploy/self-host
echo "$GHCR_TOKEN" | docker login ghcr.io -u <github-user> --password-stdin
```

The image is `ghcr.io/astound-digital/systemprompt-astound:<version>`. Always
pin a version; `latest` follows the main branch. Every image is signed:

```bash
cosign verify \
  --certificate-identity-regexp='https://github.com/Astound-Digital/systemprompt-astound/' \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@
```

## 2. Provision the database

Skip this step if you use the bundled PostgreSQL: its first start installs
the extensions itself.

For a database you manage, run `provision.sql` once as the administrative
user. It creates the `systemprompt` role and database and installs the three
extensions, which need rights the application role never gets:

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
the second file holds the signing key, so keep it in your vault.

```bash
./render-profile.sh --out ./profile \
  --external-url https://ai.example.com \
  --admin-email you@example.com \
  --trusted-proxies 127.0.0.0/8 \
  --database-url 'postgres://systemprompt:<pw>@<db-host>:5432/systemprompt?sslmode=require' \
  --anthropic-key sk-ant-... \
  --bedrock-key ... \
  --image ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@
```

| Flag | Value |
|------|-------|
| `--external-url` | the HTTPS address users open. It becomes the JWT issuer and the CORS origin |
| `--admin-email` | the first admin account, created at boot as user `admin` |
| `--trusted-proxies` | the address your TLS proxy or load balancer connects from. nginx on the same machine: `127.0.0.0/8`. Without it forwarded headers are ignored and rate limiting can refuse callers |
| `--database-url` | from step 2, or `postgres://systemprompt:<pw>@postgres:5432/systemprompt` for the bundled database |
| `--image` | optional; validates the rendered profile inside the image before you start anything |

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
client download against its checksum, anonymous rejection on `/v1/messages`,
the binary version, both MCP servers, the `admin_console` agent, the
migration ledger and the admin user. All checks must pass before you go on.

Sign in at `https://ai.example.com/admin/login` with the admin e-mail,
register a passkey, and hand users the
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

## Growing to more nodes

Copy the profile directory to each new machine unchanged, point every node at
the same PostgreSQL primary, and boot the nodes one at a time. The bundle's
`README.md` covers the three-node layout, load balancers per region, and the
rules that apply once more than one node shares a database.
