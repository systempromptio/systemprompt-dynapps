---
title: "Deploy: Validate Every Stage"
description: "The deployment as a sequence of stages, each with the command that proves it, the output to expect, and the failure signatures with their fixes."
author: "Astound Digital"
slug: "deploy-troubleshooting"
keywords: "troubleshooting, deploy, preflight, validate, image, ghcr, postgresql, pgvector, secrets, vault, profile, entrypoint, health, load balancer, boot"
kind: "guide"
public: true
tags: ["documentation", "deployment", "troubleshooting"]
published_at: "2026-09-02"
updated_at: "2026-09-02"
after_reading_this:
  - "Prove each deployment stage on its own before moving to the next"
  - "Read the boot log stage by stage and know what each line means"
  - "Recognise every known failure signature and apply its fix"
  - "State the exact condition of a deployment from two scripts"
related_playbooks:
  - title: "Deploy: One Node and PostgreSQL"
    url: "/documentation/deploy"
  - title: "Deploy on Oracle Cloud"
    url: "/documentation/deploy-oracle-cloud"
  - title: "Proposed: Target Deployment Architecture"
    url: "/documentation/deploy-target-architecture"
---

# Deploy: Validate Every Stage

A deployment is eight stages. Each one has a command that proves it, and a
deployment is in a known state when you can say which stage is the first
one that fails. Two scripts in `deploy/self-host/` cover the stages
mechanically: `preflight.sh` proves stages 1 to 4 before any container
starts, `verify-node.sh` proves stages 5 to 8 after it does. Run both after
every change and keep their output with the change record.

```bash
./preflight.sh --image ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@ \
  --profile-dir /etc/astound/profile --external-url https://ai.example.com
./verify-node.sh --url http://localhost:8080 --container <app container> --version @RELEASE_VERSION@
```

| # | Stage | Proved by |
|---|-------|-----------|
| 1 | Image: reachable, right version, signed, complete | `preflight.sh` image stage |
| 2 | Secrets: rendered, well-formed, on the node | `preflight.sh` profile stage |
| 3 | Profile: mapped into the image and accepted by it | `preflight.sh` profile stage |
| 4 | Database: reachable, TLS, primary, extensions, rights | `preflight.sh` database stage |
| 5 | Boot: every entrypoint step in the log | `docker compose logs app` |
| 6 | Node: health, site, downloads, MCP, agent, admin | `verify-node.sh` |

On **0.44.0**, stage 1 reports `FAIL  cosign signature does not verify` if <!-- pinned-release -->
cosign is installed. That is correct and expected: GitHub Actions is
unavailable on this account, so the release pipeline did not run and the image
was built and pushed by hand, unsigned. The check is not broken and has not
been relaxed — an unsigned image genuinely cannot be verified. Confirm the
image by digest instead
(`sha256:cd4e2a50edb5f63a4f6b1426aa4fe7d41788d300d5158b5670b475b548a69437`)
and read the rest of the stage: version, MCP binaries and baked-in downloads
are all still checked and must still pass. Signing resumes when the pipeline
does.
| 7 | Edge: DNS, TLS, forwarded headers, health check body | commands below |
| 8 | Traffic: sign-in, a governed model call, audit row | commands below |

## 1. Image

```bash
echo "$GHCR_TOKEN" | docker login ghcr.io -u <github-user> --password-stdin
docker pull ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@
docker image inspect ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@ \
  -f '{{index .Config.Labels "org.opencontainers.image.version"}}'      # @RELEASE_VERSION@
docker run --rm --entrypoint /app/bin/systemprompt \
  ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@ --version   # systemprompt @RELEASE_VERSION@
```

| Signature | Meaning | Fix |
|-----------|---------|-----|
| `denied` / `unauthorized` on pull | token lacks `read:packages`, or the token's account has no access to the package | ask Astound for a token on the package; `docker logout ghcr.io` then log in again |
| `manifest unknown` | the tag does not exist | versioned tags are `X.Y.Z` without a `v`; check the release list |
| label version differs from the tag | wrong image | never run it; the tag was pulled from the wrong registry or mirror |
| `cosign verify` fails | the image was not built by the release pipeline | do not run it |

## 2. Secrets

Secrets are one JSON file, `secrets.json`, rendered once by
`render-profile.sh` and identical on every node. It never comes from
environment variables and it is never generated inside the container. The
flow is: render on an operator machine, store the file in your vault as the
source of truth, write it onto each node at deploy time, mount the
directory read-only.

```bash
jq -r 'keys[]' /etc/astound/profile/secrets.json
# database_url  oauth_at_rest_pepper  manifest_signing_secret_seed  signing_key_pem  anthropic  bedrock_api_key
stat -c '%a %U' /etc/astound/profile/secrets.json                    # 600 <uid 1000>
jq -r .manifest_signing_secret_seed secrets.json | base64 -d | wc -c   # 32
jq -r .signing_key_pem secrets.json | base64 -d | openssl pkey -noout && echo key-ok
sha256sum /etc/astound/profile/secrets.json                          # identical on every node
```

| Signature | Meaning | Fix |
|-----------|---------|-----|
| `manifest_signing_secret_seed is invalid: expected 32-byte seed` | the seed is not base64 of 32 bytes | re-render with `render-profile.sh`; never hand-type it |
| `signing_key_pem secret is invalid: Invalid symbol 45` | a raw PEM was stored; the value must be base64 of the PEM | re-render |
| `secrets.json has no signing_key_pem` | the file was hand-built | re-render |
| `permission denied` reading the profile | not readable by uid 1000 | `chown 1000:1000`, `chmod 600` |
| tokens minted on one node rejected by another | `secrets.json` differs between nodes | copy the vault copy to every node; compare `sha256sum` |

## 3. Profile mapped into the image

The container reads exactly one directory:
`SYSTEMPROMPT_PROFILE_DIR=/app/.systemprompt/profiles/self-host`, which the
compose file binds from `ASTOUND_PROFILE_DIR` on the host. The profile carries
only what differs per deployment — URLs, database, admin, secrets pointer,
governance hook. The provider catalog (models, pricing, limits) and the gateway
routes are not in it: they ship inside the image as
`services/ai/providers.yaml` and `services/ai/gateway.yaml`, so every node and
every environment on the same image serves the same models. Line 1 of the
rendered profile records the template release (`# template-version: X.Y.Z`);
`preflight.sh` refuses a profile whose pin is not the image's version.

Prove the mapping with the image's own loader, from the same path the
container will mount, and inspect the catalog the image carries:

```bash
docker run --rm --entrypoint /app/bin/systemprompt \
  -v /etc/astound/profile:/p:ro -e SYSTEMPROMPT_PROFILE=/p/profile.yaml \
  ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@ admin config validate --strict /p/profile.yaml
grep -E 'api_external_url|jwt_issuer|^  email' /etc/astound/profile/profile.yaml
head -1 /etc/astound/profile/profile.yaml
docker run --rm --entrypoint /app/bin/systemprompt \
  -v /etc/astound/profile:/p:ro -e SYSTEMPROMPT_PROFILE=/p/profile.yaml \
  ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@ admin config catalog provider list
docker run --rm --entrypoint /app/bin/systemprompt \
  -v /etc/astound/profile:/p:ro -e SYSTEMPROMPT_PROFILE=/p/profile.yaml \
  ghcr.io/astound-digital/systemprompt-astound:@RELEASE_VERSION@ admin config gateway route list
```

After boot, confirm the running container sees the same file:

```bash
docker compose exec app sh -c 'sha256sum /app/.systemprompt/profiles/self-host/profile.yaml'
sha256sum /etc/astound/profile/profile.yaml
```

| Signature | Meaning | Fix |
|-----------|---------|-----|
| `unknown field` on validate | a key the image does not know; the template is from a different release | re-render from the bundle that matches the image version (`head -1 profile.yaml` names the template release) |
| `still carries a top-level \`providers:\` section ... move the block into services/ai/providers.yaml` | a profile rendered from an older bundle, when the catalog and routes still lived in the profile | re-render from the @RELEASE_VERSION@ bundle; the catalog is in the image, nothing to copy |
| `admin config catalog provider list` is empty, or `/v1/messages` answers `No gateway route matches model` | the image's `services/config/config.yaml` does not include `ai/providers.yaml` and `ai/gateway.yaml` | a build defect, not a deployment one — report the image tag |
| `SYSTEMPROMPT_PROFILE_DIR is set but .../profile.yaml is missing` | wrong `ASTOUND_PROFILE_DIR`, or the directory is not readable | fix the path in the environment; check ownership |
| `No admin email configured for 'admin'` | `system_admin.email` missing | re-render with `--admin-email` |
| sign-in page loads but login fails with a cookie error | `api_external_url` is not the URL in the browser, or not https | re-render with the real public URL |

## 4. Database

```bash
psql "$DATABASE_URL" -c 'select version()'                          # PostgreSQL 16.x or newer
psql "$DATABASE_URL" -tAc 'select pg_is_in_recovery()'              # f  (must be the primary)
psql "$DATABASE_URL" -tAc "select extname from pg_extension where extname in ('vector','uuid-ossp','pgcrypto')"  # three rows
psql "$DATABASE_URL" -c 'create table _probe(i int); drop table _probe'   # role can migrate
psql "$DATABASE_URL" -c '\conninfo'                                  # ... SSL connection
```

On Oracle Cloud the extensions must first be enabled on the DB system
(Configuration → Extensions) or `CREATE EXTENSION` fails inside the
database. Security lists must allow 5432 from the node subnet, and the
private endpoint's FQDN must resolve from the node.

| Signature | Meaning | Fix |
|-----------|---------|-----|
| `Postgres did not become ready within 300s` | host unreachable, security list, DNS, wrong port | `pg_isready -d "$DATABASE_URL"` from the node; check the OCI security list and the private endpoint FQDN |
| `password authentication failed` | wrong password, or the role was not created | run `provision.sql` as the admin user |
| `database_write_url points at a read-only standby` / `pg_is_in_recovery` is `t` | the URL is a replica | point at the primary; replicas are for DR only |
| `knowledge-bank: database ... has no pgvector; it is required` | `vector` not installed on the database the knowledge bank uses | enable at system level, then `CREATE EXTENSION vector` |
| `permission denied for schema public` during migration | role lacks rights | `GRANT ALL ON SCHEMA public TO systemprompt` (in `provision.sql`) |
| `SSL connection is required` / handshake errors | `sslmode` missing or a private CA | add `?sslmode=require`; for a private CA mount the bundle and set `PGCA_CERT_PATH` |

## 5. Boot

The entrypoint logs each step in order. Read `docker compose logs -f app`
and find the first step that does not appear:

| Line | What happened | If it is the last line you see |
|------|---------------|-------------------------------|
| `Waiting for Postgres at DATABASE_URL host...` | readiness probe started | stage 4 |
| `Postgres is ready.` | database reachable | — |
| `Running database migrations...` then `Database migration completed successfully` | schema applied | migration error text names the table; a checksum mismatch means the database was seeded by a different tag: repair per the log's hint |
| `Ensuring bootstrap admin user...` | admin user upserted | `No admin email configured` → stage 3 |
| `Publishing web assets for this node...` | site rendered into this node's `web/dist` | a warning here never stops boot; the public site 404s until fixed |
| `Starting services...` and `MCP Servers (1/1)` | children up | the count grows with every server you enable; knowledge-bank (disabled by default) exits on a missing `bedrock_api_key` or pgvector (stages 2 and 4) |
| `Secrets initialization failed` | a secret is malformed | stage 2 signatures |
| `MCP services running but not properly registered in database` | two nodes booted at the same moment | start this node again once the other is `healthy`; boot nodes one at a time |
| `signing key init: ... unavailable` | `signing_key_pem` missing or malformed | stage 2 |
| `Startup failed after ...` | the reason is in the `Caused by:` lines above it | match it against this page |

Health reports the boot phase honestly: `{"status":"starting"}` until
migrations, bootstrap and children are done, then `{"status":"healthy","version":"@RELEASE_VERSION@"}`.

## 6. Node

`verify-node.sh` checks health, the admin login page, the rendered site,
every client download against its checksum, that `/v1/messages` refuses
anonymous calls, the binary version, both MCP servers, the `admin_console`
agent, the migration ledger, and the admin user. Every line must pass.

| Signature | Meaning | Fix |
|-----------|---------|-----|
| `GET /documentation/ → 404` | the site was not rendered on this node | `docker compose exec -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml app systemprompt infra jobs run publish_pipeline` |
| download `.sha256` mismatch | the image is not a release build | pull the versioned tag again |
| `MCP server ... not answering` | child died after boot | `docker compose logs app` for the child's exit reason; usually stage 2 or 4 |
| `no users row` | bootstrap did not run | boot log, stage 5 |

## 7. Edge

```bash
dig +short ai.example.com                                              # the load balancer
echo | openssl s_client -connect ai.example.com:443 -servername ai.example.com 2>/dev/null | grep 'Verify return code'   # 0 (ok)
curl -fsS https://ai.example.com/api/v1/health                         # {"status":"healthy",...}
curl -s -o /dev/null -w '%{http_code}\n' https://ai.example.com/metrics # 403
curl -s https://ai.example.com/bridge-auth/setup -o /dev/null -w '%{redirect_url}\n'  # https://..., never http://
```

| Signature | Meaning | Fix |
|-----------|---------|-----|
| balancer marks the backend unhealthy while `docker compose ps` says healthy | the check watches status only and read `starting`, or checks a path other than `/api/v1/health` | health check path `/api/v1/health`, match body `healthy` |
| install commands on the setup page start with `http://` | `X-Forwarded-Proto` not forwarded | forward it, and confirm the balancer CIDR is in `trusted_proxies` |
| every request appears to come from the balancer's address; rate limits trip | balancer CIDR missing from `trusted_proxies` | re-render with `--trusted-proxies`, restart |
| streamed answers stall or cut off | response buffering on, idle timeout too short | buffering off, idle timeout ≥ 300 s |
| `/metrics` returns 200 publicly | rule missing | return 403 for `/metrics` and `/api/v1/health/detail` |

## 8. Traffic

Prove one governed model call end to end and see it audited:

```bash
ADMIN_ID=$(psql "$DATABASE_URL" -tAc "select id from users where email='you@example.com'")
docker compose exec -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml app \
  systemprompt admin users api-key issue --user "$ADMIN_ID" --name smoke       # prints sp-live-...
SESSION=$(curl -fsS -X POST https://ai.example.com/api/public/gateway/sessions \
  -H "authorization: Bearer $PAT" -H 'content-type: application/json' -d '{}' | jq -r .session_id)
curl -fsS -X POST https://ai.example.com/v1/messages \
  -H "authorization: Bearer $PAT" -H "x-session-id: $SESSION" \
  -H 'anthropic-version: 2023-06-01' -H 'content-type: application/json' \
  -d '{"model":"claude-haiku-4-5","max_tokens":16,"messages":[{"role":"user","content":"Reply pong."}]}'
docker compose exec -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml app \
  systemprompt infra logs request list --limit 3                                 # the call, its cost, its user
```

| Signature | Meaning | Fix |
|-----------|---------|-----|
| `Invalid or revoked API key` | the token was cut at the `.`; both halves are needed | copy the whole `sp-live-<prefix>.<secret>` |
| `unknown or revoked session; mint one at POST /api/public/gateway/sessions` | `x-session-id` is not a minted session | mint one as above; the bridge does this for users automatically |
| `missing required x-session-id header` | header absent | add it |
| upstream `401`/`403` from the provider | provider key wrong | fix `anthropic` in `secrets.json`, restart |
| the call succeeds but `infra logs request list` shows no row | audit pool not writable | stage 4 rights on the role |

## Stating the deployment's condition

The condition of a deployment is the first failing stage, or "all eight
proved" with the two scripts' output attached. Keep that output with every
change: it is what turns "it does not work" into a stage number and a fix.
