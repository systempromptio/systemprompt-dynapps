# Self-hosted deployment: three nodes, one PostgreSQL primary

This directory is everything a customer needs to run the published Astound
Digital gateway image on their own infrastructure. It is written for the
launch topology — three stateless nodes (India, US, EU) behind the customer's
load balancers, one PostgreSQL 16 primary that every node reads and writes,
read replicas for disaster recovery only — and it is what
`scripts/e2e-published-release.sh` reproduces in Docker before each release
is handed over. Nothing here assumes Oracle Cloud specifically; the OCI notes
are in `docs/install/oracle-cloud.md`.

**Starting with one node?** Read `SINGLE-NODE.md` first; it is the same
bundle with the bundled database and one compose file.

| file | purpose |
|---|---|
| `SINGLE-NODE.md`, `docker-compose.single.yml`, `provision-bundled.sql` | the one-node starting point |
| `profile.yaml.template` | the per-deployment profile; rendered once, identical on every node. Its first line pins it to the image release; the provider catalog and gateway routes are not in it — they ship inside the image |
| `secrets.json.example`, `SECRETS.md` | every secret, who owns it, how to inject and rotate it |
| `render-profile.sh` | renders the profile directory and generates the cluster's shared cryptographic identity |
| `provision.sql` | one-off database provisioning, run by the DBA as the admin user |
| `docker-compose.node.yml` | one node |
| `preflight.sh` | pre-boot validation: image, secrets, profile-to-image mapping, database, DNS/TLS |
| `verify-node.sh` | the post-boot checklist as a script |

## 1. Prerequisites

| what | requirement |
|---|---|
| Nodes | 3 Linux VMs (amd64 or arm64), Docker Engine 24+ with Compose v2, 2 vCPU / 4 GB RAM minimum each, outbound HTTPS to the model provider and to AWS Bedrock |
| Image access | a GitHub personal access token with `read:packages` issued by Astound (the package is private); `docker login ghcr.io` on each node |
| Database | PostgreSQL **16** (tested floor for this deployment; 18 is the general floor) reachable from all three regions on 5432 with TLS, with the `vector`, `uuid-ossp` and `pgcrypto` extensions available |
| Model provider | a Google service-account JSON for Vertex AI (the gateway routes `claude-*` to Vertex, failing over to Anthropic), plus an Anthropic API key |
| Public entry | one hostname (for example `ai.example.com`) with TLS terminated at the load balancer in each region, all three forwarding to their local node. HTTPS is mandatory: the sign-in cookie is `Secure` and the SAML assertion is posted back to an `https` ACS URL |

## 2. Database

Have the DBA run `provision.sql` once against the primary as the
administrative user. It creates the `systemprompt` role and database and
installs the three extensions, which need elevated rights the application
role never gets:

```bash
psql "postgres://ADMIN:pw@db-primary.example.internal:5432/postgres?sslmode=require" \
  -v pw="$APP_DB_PASSWORD" -f provision.sql
```

The application connection string is then:

```
postgres://systemprompt:<pw>@db-primary.example.internal:5432/systemprompt?sslmode=require
```

- **Always the primary.** Boot runs `SELECT pg_is_in_recovery()` and refuses
  a standby. Read replicas are for DR at launch; per-region replica reads are
  a later, separately tested option (`database_write_url` in `SECRETS.md`).
- **Connections.** Each node opens up to 50 connections (the pool default).
  Size the primary for at least `3 × 50 + replication + admin`, or add
  `database: { pool: { max_connections: N } }` to the profile.
- **Private CA.** If the primary's certificate is not publicly trusted, mount
  the CA bundle and append `?sslrootcert=/etc/astound/pg-ca.pem` to
  `database_url` in `secrets.json` (commented mount in the compose file).
- **Migrations run at boot** and are forward-only. Roll back only to an image
  whose migrations match the database, and take a base backup before every
  upgrade.

## 3. Secrets

Read `SECRETS.md`. The short version: six required keys, three of which are
generated once and must be identical on all nodes because they are the
cluster's signing identity. Render on one machine, copy everywhere:

```bash
./render-profile.sh --out ./profile \
  --external-url https://ai.example.com \
  --admin-email platform-admin@example.com \
  --trusted-proxies 10.10.0.0/16,10.20.0.0/16,10.30.0.0/16 \
  --database-url 'postgres://systemprompt:<pw>@db-primary.example.internal:5432/systemprompt?sslmode=require' \
  --anthropic-key sk-ant-... \
  --image ghcr.io/astound-digital/systemprompt-astound:0.63.1     # validates the result in the image
```

`--trusted-proxies` is the list of CIDRs your load balancers connect from.
Without it the platform ignores `X-Forwarded-For` and attributes every
request to the balancer, and rate limiting can refuse unidentified callers.

Store the resulting `profile/secrets.json` in your vault as the source of
truth and render it onto each node at deploy time (OCI Vault, HashiCorp
Vault, SOPS — any tool that can write a file):

```bash
sudo install -d -m 0750 -o 1000 -g 1000 /etc/astound/profile
sudo install -m 0644 -o 1000 -g 1000 profile/profile.yaml /etc/astound/profile/profile.yaml
sudo install -m 0600 -o 1000 -g 1000 profile/secrets.json /etc/astound/profile/secrets.json
```

The container runs as uid 1000 and mounts the directory read-only. Nothing
is ever written there: runtime state the node keeps for itself (the gateway
accounting journal, `/app/storage/data/gateway-journal`) lives on the
`app_storage_data` volume, which must stay per node — never a shared mount.
It must also be writable by uid 1000. A named volume from these compose files
is, because the image creates `/app/storage/data` owned by that uid. A bind
mount or pre-existing volume must be `chown -R 1000:1000`; on Kubernetes set
`securityContext.fsGroup: 1000`. The server checks this before it starts and
names each directory it cannot write, its owner and the uid it runs as.

## 4. Network

| rule | value |
|---|---|
| Load balancer → node | HTTP to port 8080 only |
| Health check | `GET /api/v1/health`, expect HTTP 200 **and** the body to contain `"healthy"`. During boot the same path answers `200 {"status":"starting"}` and must not be admitted |
| Forwarded headers | `X-Forwarded-For`, `X-Forwarded-Proto: https`, `Host` |
| Streaming | disable response buffering (nginx `proxy_buffering off`), idle timeout ≥ 300 s on `/v1/*` and `/api/*` (SSE) |
| Block externally | `/metrics` (unauthenticated Prometheus) and `/api/v1/health/detail` |
| Drain | the server has no graceful shutdown: remove a node from the balancer, wait 30 s, then stop it |
| Security groups | 8080 from the balancer only; 5000–5999 (MCP servers) and 9000–9999 (A2A agents) are node-internal and must not be reachable from outside the node |
| Egress | `aiplatform.googleapis.com:443`, `oauth2.googleapis.com:443`, `api.anthropic.com:443`, `ghcr.io:443` (pulls), the primary on 5432 |

A worked nginx configuration with every one of these rules is generated by
`scripts/e2e-published-release.sh` (see the `lb/nginx.conf` heredoc).

## 5. Boot

On every node, with `/etc/astound/profile` in place:

```bash
echo "$GHCR_TOKEN" | docker login ghcr.io -u <github-user> --password-stdin
export SYSTEMPROMPT_TAG=0.63.1 ASTOUND_PROFILE_DIR=/etc/astound/profile
export DATABASE_URL='postgres://systemprompt:<pw>@db-primary.example.internal:5432/systemprompt?sslmode=require'
docker compose -f docker-compose.node.yml pull
docker compose -f docker-compose.node.yml run --rm app migrate   # node 1 only, once
docker compose -f docker-compose.node.yml up -d
```

**Migrate once, then boot.** The profile sets `database.migrate_on_boot:
false`, so no node changes the schema as it starts. The `migrate` one-shot
(the image's entrypoint running `systemprompt infra db migrate`, then
`admin bootstrap`) moves the shared primary forward and creates the bootstrap
admin, from node 1, before any node serves. A node started against a schema
it does not match refuses to boot and names the pending migration and the
`systemprompt infra db migrate --profile self-host` command.

**Nodes may boot together.** Each node claims its own instance id in the
shared database at boot, so the MCP registry and the per-instance state of
concurrently starting nodes no longer collide; once `migrate` has run, start
them in any order. Rolling upgrades still go one node at a time, behind the
balancer, so capacity never drops to zero. Run the checklist on each node
once it is `healthy`, passing the node's `server.role`:

```bash
./verify-node.sh --url http://localhost:8080 --container <compose container name> --version 0.63.1 --role all
```

**If `verify-node.sh` reports the public site missing on a node**, render its
copy by hand (entrypoints before the per-node render skipped it on every node
but the first, because of a database-wide job lock):

```bash
docker compose -f docker-compose.node.yml exec \
  -e SYSTEMPROMPT_PROFILE=/app/.systemprompt/profiles/self-host/profile.yaml \
  app systemprompt infra jobs run publish_pipeline
```

`verify-node.sh` checks health, the admin login page, the public site, every
bridge download against its checksum, that `/v1/messages` refuses anonymous
calls, the binary version, the MCP child server, the migration ledger and the bootstrap admin. Run it on every node
and after every upgrade.

First sign-in: `https://ai.example.com/admin/login`. The bootstrap admin is
the `admin` user with the e-mail given to `--admin-email`, and sign-in is AD FS SSO only, so configure the relying party
first (`/documentation/adfs-sso-setup` on the node's own documentation site). With SSO down, use the CLI break-glass
session described in `/documentation/authentication`.

## 6. Upgrade and roll back

Versioned image tags are immutable. One node at a time, behind the balancer:

```bash
cosign verify --certificate-identity-regexp='https://github.com/Astound-Digital/systemprompt-astound/' \
  --certificate-oidc-issuer=https://token.actions.githubusercontent.com \
  ghcr.io/astound-digital/systemprompt-astound:<new-version>
# drain the node at the balancer, wait 30 s, then:
SYSTEMPROMPT_TAG=<new-version> docker compose -f docker-compose.node.yml pull
SYSTEMPROMPT_TAG=<new-version> docker compose -f docker-compose.node.yml run --rm app migrate   # first node only
SYSTEMPROMPT_TAG=<new-version> docker compose -f docker-compose.node.yml up -d
./verify-node.sh --version <new-version> ...
```

The one-shot `migrate` on the first upgraded node applies the new
migrations; every node, that one included, then boots against the migrated
schema. Rolling back the image is the same command with the
previous tag and is safe only while the database has not been migrated
forward — restore the pre-upgrade base backup otherwise.

### Read-only root filesystem

The server writes only to the directories below (core's writable-path
contract) and at boot write-probes each one for the node's `server.role`,
refusing to start with one error that names every root it cannot write.
With this profile's `paths`:

| root | path | volume |
|---|---|---|
| `system.logs` | `/app/logs` | per node, may be ephemeral |
| `storage.files` | `/app/storage/files` | per node, or shared; see below |
| `storage.exports` | `/app/storage/exports` | per node |
| `storage.data` | `/app/storage/data` (scratch under `data/scratch`) | per node, already `app_storage_data` |
| `web.dist` | `/app/web/dist` | per node, already under `app_web` (admin and all roles) |
| `services.cache_dir` | `/app/services-cache` (set in the profile) | per node, may be ephemeral (admin and all roles) |
| CLI session store | `/app/.systemprompt/sessions` | per node, ephemeral (`emptyDir`) |

The CLI session store is not a server root: the entrypoint's `admin
bootstrap` and `infra jobs run` resolve a CLI session and write it under
`/app/.systemprompt/sessions`, so it needs a writable mount too. The profile
directory itself stays read-only. MCP child servers may write `/tmp`; give it
a `tmpfs`. The image creates every path above owned by uid 1000, so a named
volume or `emptyDir` mounted there is writable. For Docker Compose, add
`read_only: true`, `tmpfs: [/tmp]` and a volume for every row to the `app`
service.

`/app/storage/files` also carries the image's own admin templates and
assets. The image keeps a pristine copy in `/app/share/storage-files` with a
content stamp, and the entrypoint copies it onto the volume whenever the
stamp differs from the volume's `.image-stamp`: shipped files are refreshed
on every upgrade, files uploaded beside them are kept. A file a release
deletes stays on the volume until removed by hand.

### Kubernetes (GKE)

The Helm chart `deploy/helm/systemprompt-astound/` implements everything below; runbook: `docs/install/gke.md`.

- **Storage:** use `storage: {backend: gcs, bucket: …, credentials:
  workload_identity}` (the block commented in `profile.yaml.template`) so
  uploaded and generated files live in one bucket every pod sees.
  `paths.storage` still needs a per-pod volume for the journal, exports and
  scratch space.
- **Role split:** run exactly one `admin` Deployment (`server.role: admin`,
  `replicas: 1`): it runs the scheduler, MCP and agent reconciliation and the
  console. Run the gateway pods as a separate Deployment with `server.role:
  gateway`, scaled freely; they mount only the gateway, bridge, OAuth and
  health routes and run no workers, and they need neither `web.dist` nor
  `services.cache_dir`. Two `admin` replicas would run the scheduler twice.
  Check a node with `verify-node.sh --role gateway` or `--role admin`.
- **Migrations:** a Job running the image with argument `migrate`, before the
  rollout; no pod migrates on boot.

## 7. Clients

The gateway serves its own client binaries, so users never need GitHub
access: `https://ai.example.com/documentation/downloads` lists Windows,
macOS, Linux x86_64 and aarch64 builds with checksums, and the Linux
installer one-liner. Linking a client is `https://ai.example.com/admin/connect`
after sign-in, or `systemprompt admin bridge issue-code --user-id <email>`
inside a node container for headless onboarding.

## 8. Launch checklist

| # | item | owner | where it goes |
|---|---|---|---|
| 1 | GHCR read token | Astound | `docker login ghcr.io` on each node |
| 2 | Public hostname + TLS certificate per region | customer network | load balancer |
| 3 | Load balancer CIDRs | customer network | `render-profile.sh --trusted-proxies` |
| 3a | Bootstrap admin e-mail (the `admin` account; first sign-in) | customer admin | `--admin-email` |
| 4 | PostgreSQL 16 primary, TLS, extensions via `provision.sql` | customer DBA | `--database-url` |
| 5 | Application DB password | customer DBA | `provision.sql -v pw=`, `--database-url` |
| 6 | Anthropic API key | customer | `--anthropic-key` |
| 8 | Rendered `profile/` in the vault, copied to `/etc/astound/profile` on all nodes | customer ops | bind mount |
| 9 | Security groups per §4 | customer network | — |
| 10 | Node 1 booted and `verify-node.sh` green, then nodes 2 and 3 | customer ops | — |
| 11 | AD FS relying party configured, first admin signed in through SSO | customer admin | `/admin` |
| 12 | Base backup schedule and restore test | customer DBA | — |
