# 2026-09-02 — Self-hosted multi-node deployments share one rendered profile

**Status:** accepted.

## Context

The customer deploys the published image on Oracle Cloud as three nodes
(India, US, EU) against one PostgreSQL 16 primary with DR replicas. Until
now the only documented install was `docs/install/docker.md`: a `.env` file
and `docker/entrypoint.sh` authoring a profile inside the container on first
boot. That path generates a fresh RSA signing key, OAuth pepper and manifest
seed per container, and keeps the signing key on no volume.

## Decision

1. **Multi-node means one rendered profile directory, identical on every
   node**, bind-mounted through `SYSTEMPROMPT_PROFILE_DIR`. It carries the
   three shared identity secrets (`signing_key_pem`, `oauth_at_rest_pepper`,
   `manifest_signing_secret_seed`) generated once by
   `deploy/self-host/render-profile.sh`. The `.env` path stays for single-node
   installs and is documented as single-node only.
2. **The signing key ships as the `signing_key_pem` secret**, never as a
   file the entrypoint generates. The entrypoint now refuses a supplied
   profile without it, and stops generating `/app/signing_key.pem` in that mode.
3. **Every node reads and writes the primary at launch.** Read replicas are
   DR only. Per-region replica reads (`database_url` = replica,
   `database_write_url` = primary) are plumbed but deliberately not enabled:
   cross-region replica lag broke read-after-write for sessions on Fly.
4. **PostgreSQL 16 is the tested floor for this customer**, because OCI's
   managed service tops out there; the e2e runs `pgvector/pgvector:pg16`.
5. **Knowledge bank stays a hard boot requirement** (pgvector on the
   instance database, `bedrock_api_key`). The customer has both; no off switch.
6. **Client binaries are served by the customer's own gateway** at
   `/files/downloads/*` and listed on `/documentation/downloads`; GitHub
   Release links are never shown to end users because the repository is private.

## Found while proving it (open, core)

- **The MCP service registry is per server name, not per instance.** Every node
  spawns its own MCP children and registers them in the shared `services` table
  rows by name; boot-time reconciliation treats another node's live PID as
  stale. Two nodes booting concurrently race, and one fails with "MCP services
  running but not properly registered in database". Mitigation at launch: nodes
  boot strictly one at a time (runbook §5). Fix: scope the registry rows by
  `server.instance_id` in core; until then the e2e boots serially on purpose.
- **`publish_pipeline` output is node-local but its bootstrap run is cluster-locked**,
  so only the first node rendered its site. Fixed in this repo: the entrypoint
  runs the pipeline through the lock-free manual runner on every node.

## Consequences

- `scripts/e2e-published-release.sh` is the release-acceptance proof for this
  topology and runs against every published tag before hand-over.
- Rotating `signing_key_pem` is a whole-cluster restart that signs everyone out;
  `SECRETS.md` says so.
- A future entrypoint change that reintroduces per-container secret generation
  under `SYSTEMPROMPT_PROFILE_DIR` would silently break the cluster; the
  e2e's cross-node PAT assertion exists to catch that.
