# 2026-09-01 — Launch scope: no federated identity yet, a minimal identity spine, and CI-published deploys

**Status:** decided (Ed, 2026-09-01). Three decisions taken the same day as
`2026-09-01-knowledge-bank-remote-shared-index.md`; recorded together because
each one bounds the others.

## 1. Federated identity is not a launch requirement; personal tokens run the RAG flow

**Decision.** *(Superseded on 2026-09-08 by
`2026-09-08-knowledge-bank-reads-through-connected-mcp.md`: the per-user
Atlassian connector now exists, and `sync_knowledge` reads through it.)* The
knowledge bank (project-context RAG) launched without federated
identity to Atlassian. Each user passed their **own** Atlassian e-mail and API token
as headers on the remote MCP connection (`x-atlassian-email`,
`x-atlassian-api-token`, kept in their local `.mcp.json`), and `sync_knowledge`
read Confluence and Jira as that user. Every document a user syncs is granted to
`user:<id>` in the shared index; search is filtered by those grants.

**Why.** An OAuth 3LO app, its approval, and an `external_auth` grant flow are weeks
of coordination with Astound IT for a POC whose value is the index, not the login.
Personal tokens give exactly the per-user read semantics federated identity would —
Atlassian enforces the caller's permissions on every read — with nothing to
provision on the Atlassian side.

**What changes later.** Only the *source* of the credential: an OAuth grant minted
through `external_auth` replaces the header token. The shared index, the per-user
grants, the tool surface and the client shape stay as they are.

## 2. A basic federated-identity structure exists from day one: two projects, two groups

**Decision.** The instance is built on a deliberately small identity spine supplied by
SSO (ADFS SAML, `login.astoundcommerce.com`): **two projects** — `commerce` and
`core` — derived from **two AD groups** (`Systemprompt-Commerce`,
`Systemprompt-Core`) carried in the SSO group claim. Every governance rule this
instance ships — what a user may see in the admin, which marketplace plugins and
MCP servers they get, which knowledge they can read, what the gateway lets through
— is expressed against that spine and nothing else. No organisation/plan/seat/
department model, no per-project databases.

**Why.** Governance that keys on a rich org model nobody has yet is governance that
has to be rewritten when the real model arrives. Two projects and two groups are the
smallest structure that is real today (they exist in AD), lets the admin surface be
project-scoped (`ProjectScope`), and gives every later feature a stable subject to
attach to. Growing it is adding groups, not redesigning.

**Consequences.** `projects-from-ad-groups` is the model (`038_projects.sql`); the
org/plan/seat/department spine is deleted; the knowledge bank's `adfs_group` grant
subject stays in the schema for when a document should be shared with a project
rather than a person, but nothing writes it at launch (decision 1).

## 3. GitHub CI/CD publishes every deploy asset; deploy is "an image plus a database"

**Decision.** Ed owns the release path in GitHub Actions: every push to `main`
publishes the GHCR image, the bridge binaries for Linux/macOS/Windows and the GitHub
Release (`.github/workflows/release.yml`); `next` runs the gates only. The deploy
service is streamlined so that deploying an instance means **pointing a published
image at a database** — no source checkout, no build on the target, no per-host
tooling beyond a container runtime and Postgres with pgvector.

**Why.** Co-workers deploy to targets we do not operate (Oracle VMs, remote
Postgres); the only reproducible contract across those targets is a pinned image and
a `DATABASE_URL`. Building on the target reintroduces every toolchain and
vendoring problem the `next`/`main` split exists to keep out of production.

**Consequences.**
- `just setup-local` installs the released server and MCP binaries instead of
  compiling; `docker-compose.yml` runs the published image against the bundled
  pgvector Postgres or a remote one.
- The knowledge bank's dedicated Postgres (`deploy/fly/knowledge-db/`) follows the
  same rule: it is provisioned once with `flyctl`, and the instance reaches it
  through one secret, `knowledge_database_url`.
- `bridge/CORE_REF` pins the core ref CI builds against; `scripts/check-core-ref.sh`
  enforces a release tag on `main`.
