# The knowledge database on Fly

The knowledge bank's index (`knowledge_*` tables, pgvector) lives on its own Fly
Postgres app, `sp-a2f658d8bc5f-knowledge`, in `fra`. It is not on the shared
`systemprompt-db-prod` cluster for two reasons established on 2026-09-01: that
cluster (`flyio/postgres-flex 17.7`, 13 databases) has no `vector` extension, and
the tenant role cannot `CREATE DATABASE`, so neither adding the extension nor
carving out a database there is something this repo can do. The dedicated app is
owned by us, provisioned with the scripts beside this file, and reached over the
private network only.

Fly Postgres is not on the Fly-Launch managed path, so every step is an explicit
`flyctl` command. The `just knowledge-db-*` recipes wrap them.

## 1. Create the app

```bash
just knowledge-db-create
# = flyctl postgres create --name sp-a2f658d8bc5f-knowledge --region fra \
#     --initial-cluster-size 1 --vm-size shared-cpu-1x --volume-size 10 --org "$FLY_ORG"
```

`FLY_ORG` defaults to the organisation of the application app; `flyctl apps list`
shows it. Keep the `postgres` superuser password `flyctl` prints — provisioning
needs it once, as `KNOWLEDGE_DB_SUPERUSER_PASSWORD`.

## 2. Provision the role, the database and pgvector

```bash
KNOWLEDGE_DB_SUPERUSER_PASSWORD='<printed by create>' \
KNOWLEDGE_DB_PASSWORD='<choose one>' just knowledge-db-provision
```

Every `knowledge-db-*` recipe goes through `psql.sh` in this directory: it opens
`flyctl proxy 15432:5432` to the app, runs your local `psql` against it, and
closes the proxy on exit (`flyctl postgres connect` runs psql *on* the machine
and cannot read local files). Provisioning runs `provision.sql` as the superuser:
`CREATE ROLE knowledge`, `CREATE DATABASE knowledge OWNER knowledge`, then
`CREATE EXTENSION IF NOT EXISTS vector` inside it. The knowledge-bank binary
applies its own tables at boot; nothing else is created by hand.

**`CREATE EXTENSION vector` will fail on the stock image, so step 2 always needs
the redeploy below.** Settled on 2026-09-01: `flyctl postgres create` provisions
`flyio/postgres-flex:18.1`, which answers
`ERROR: extension "vector" is not available`. The role and the database are
created by that run; only the extension is missing. Redeploy the app from the
`Dockerfile` beside this file, which adds `postgresql-18-pgvector` to the same
base image, then re-run step 2 — it is idempotent:

```bash
flyctl deploy -a sp-a2f658d8bc5f-knowledge --dockerfile deploy/fly/knowledge-db/Dockerfile .
```

That yields `vector 0.8.6`.

**The Dockerfile's tag must track the major version the cluster runs.** It read
`FROM flyio/postgres-flex:17` while Fly provisioned 18.1, and deploying that
would have started a downgraded major against an existing data directory rather
than adding an extension. Read the running major from
`flyctl image show -a sp-a2f658d8bc5f-knowledge` before changing it.

This closes blocker B8.

## 3. Point the production profile at it

```bash
systemprompt admin config secret set knowledge_database_url \
  'postgres://knowledge:<pw>@sp-a2f658d8bc5f-knowledge.flycast:5432/knowledge' \
  --profile production
```

Use the **flycast** (6PN) hostname, never an external one: the application app and
the database share the private network, and a public route would add the
cross-region latency documented for the main database. Then `just deploy-next`.
The knowledge-bank child opens this URL at boot, probes `pg_available_extensions`,
and exits with `knowledge-bank: database <host> has no pgvector; it is required
(see docs/install/knowledge-db.md)` — failing the platform boot — if the
extension is missing. That is deliberate: the database must exist before deploy.

## 4. Inspect, prune

```bash
export KNOWLEDGE_DB_PASSWORD='<the role password>'   # every recipe below needs it
just knowledge-db-psql            # interactive psql on the knowledge database
just knowledge-db-report          # size, documents, chunks, grants, syncing users, cache rows, last syncs
just knowledge-db-prune           # dry run of the prune SQL (STALE_DAYS=90)
just knowledge-db-prune DRY_RUN=0 # apply it
```

`prune.sql` is the same SQL the admin-only `prune_knowledge` tool runs; the tool
is the normal path and the recipe is for when the server is down or an operator
prefers psql.
