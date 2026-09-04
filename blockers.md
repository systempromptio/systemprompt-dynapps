# Blockers — project-context knowledge bank (Phase 0)

Status as of 2026-09-01, after the knowledge-bank MCP server
(`extensions/mcp/knowledge-bank/`) was reshaped into its deployment form: a
remote server on `astound.systemprompt.io`, one shared index on a dedicated
pgvector Postgres, and every user reading Confluence and Jira with their own
credentials. Everything the engineering side owns is built and tested; what
remains is infrastructure provisioning, access, content, and decisions that sit
with the Customer, Astound IT, or another team. Each item says what is blocked,
why, who can unblock it, and what happens the moment it is.

Reference: requirements `docs/migration/project-context/PHASE0-REQUIREMENTS.md`;
ticket `docs/tickets/req-047-knowledge-bank-mcp.md`; decisions
`docs/decisions/2026-09-01-knowledge-bank-remote-shared-index.md` (and the
partly superseded `…-ingestion-in-process.md`).

---

## What is NOT blocked (proven live on 2026-09-01)

- **Bedrock, eu-central-1.** Titan v2 embeddings, Cohere rerank, and the
  `openai.gpt-oss-120b-1:0` chat model all answer with the instance key
  (`bedrock_api_key`, profile secret store). The env-gated test
  `tests/integration/mcp/src/knowledge_live.rs` ingests a real Confluence space and
  gets the right documents back from a real search.
- **Confluence and Jira as the caller.** A user's own e-mail + API token, passed as
  `x-atlassian-email` / `x-atlassian-api-token` headers on the MCP connection, read
  the `EAICOMM` space and project (HTML → Markdown, footer comments, enrichment,
  version date, canonical URL) and every document lands with a `user:<id>` grant.
- **The write path.** Per-source chunk strategies, LLM dating with provenance,
  per-user incremental sync (an unchanged corpus makes zero model calls), per-user
  grant removal on a complete fetch, orphan deletion, rebuild against the embedding
  cache, and a page synced by two users stored once and granted twice.
- **Access control.** Per-document grants are joined inside the retrieval queries;
  a caller without a matching subject sees nothing (DB-backed tests).
- **Manageability.** `prune_knowledge` (dry run + apply), the extended
  `knowledge_index_stats`, and the `just knowledge-db-*` recipes against psql.

---

## B1 — No corpus for retrieval parity (M1)  · owner: Customer / Astound delivery team

**What.** Milestone M1 ("retrieval parity over a fixed evaluation set") needs the
existing project-context corpus and an agreed question set. Neither is available.

**Why it blocks.** The engine is proven on fixtures and one small Confluence space;
parity is a claim about *the* corpus. Without it REQ-046 (>70 % precision@5)
cannot report a number — `tests/integration/mcp/fixtures/knowledge-benchmark.json`
is a checked-in placeholder that deliberately refuses to score.

**Note.** File sources are gone (decision, 2026-09-01): the hand-authored
`meeting_notes` / `adrs` / `specs` trees from the Cursor repo are indexed only
once they are in Confluence (or Jira). The corpus path is Confluence and Jira
only.

**To unblock.**
1. Land the hand-authored material in a Confluence space users can read, and list
   that space under `sources.confluence.spaces`.
2. Agree ~30 evaluation questions with their answering document ids and replace the
   placeholder fixture (drop `"placeholder": true`).

**Then.** A user runs `sync_knowledge`, the benchmark runs with
`KNOWLEDGE_BANK_BEDROCK_KEY` set, and the top hits are compared against the Node
engine's.

## B2 — What the users can read is one space and an empty project  · owner: Astound IT / space & project admins

**What.** The configured scope is one Confluence space (`EAICOMM`, two pages) and
one Jira project (`EAICOMM`, zero issues). The real project material lives in
spaces and projects that have to be both readable by the pilot users *and* listed
in the configuration.

**Why it blocks.** FR-ING-2/3 (Confluence and Jira as corpus sources) are
implemented but have almost nothing to index; the "index mirrors the source"
property (FR-ING-7) cannot be demonstrated on real data. There is no service
account any more: a space is indexed exactly as far as the syncing users can read
it, so a user who cannot open a space contributes nothing from it.

**To unblock.** Confirm the pilot users have **read** on every Confluence space and
**Browse Projects + View Comments** on every Jira project that should be
searchable, then list them in `services/knowledge-bank/config.yaml`
(`sources.confluence.spaces`, `sources.jira.projects`, optional `jql`).

**Note.** The "RAG access probe (delete me)" page in `EAICOMM` is still there;
someone with delete rights should remove it. It disappears from the index at the
next complete sync of anyone who held it.

## B3 — Atlassian group restrictions · CLOSED (Ed, 2026-09-01)

No mapping is needed. Restrictions are enforced by Atlassian for the syncing
user, and a document is granted only to the users who synced it, so a page
restricted to a group is findable exactly by the members of that group who ran
`sync_knowledge`. `group_map` / `unmapped_groups` and the grant translator are
deleted.

## B4 — Live Jira/Confluence MCP servers · DEFERRED for the POC (Ed, 2026-09-01)

**Decision.** No Atlassian OAuth app for now. `services/mcp/{jira,confluence}.yaml`
stay disabled; "open the cited source" relies on **individuals' own tokens** — the
existing `atlassian` skill (`services/skills/atlassian`) drives Jira and Confluence
through its scripts with the user's personal API token, and the knowledge-bank hit
carries the canonical page/issue URL to open. The same personal token is what the
knowledge bank itself now syncs with.

**To revisit later.** Register a 3LO app (classic scopes + `offline_access`, callback
`https://astound.systemprompt.io/admin/auth/atlassian/callback`), set
`atlassian_client_secret`, enable the two servers, and settle the `mcp.atlassian.com`
audience question flagged in `services/mcp/jira.yaml` with one curl. Federated
identity would then replace the header token as the knowledge bank's credential
source; nothing else in the design changes.

## B5 — Provision the Fly knowledge database  · owner: whoever holds Fly access for `sp-a2f658d8bc5f`

**What.** The index lives on a dedicated Fly Postgres app with pgvector,
`sp-a2f658d8bc5f-knowledge` (`fra`). It does not exist yet.

**Why it blocks.** pgvector is a hard requirement: the knowledge-bank child exits
without it and core fails the whole platform boot. The shared
`systemprompt-db-prod` cluster cannot host it (no `vector` extension; tenant role
cannot `CREATE DATABASE`). Until the app exists and the production profile carries
`knowledge_database_url`, `just deploy-next` would point the child at the tenant
database and the deploy would not come up.

**To unblock.**
```bash
just knowledge-db-create
KNOWLEDGE_DB_PASSWORD='…' just knowledge-db-provision
systemprompt admin config secret set knowledge_database_url \
  'postgres://knowledge:<pw>@sp-a2f658d8bc5f-knowledge.flycast:5432/knowledge' --profile production
just deploy-next
```
Full procedure: `deploy/fly/knowledge-db/README.md`, `docs/install/knowledge-db.md`.

## B6 — Bedrock model-invocation logging is off  · owner: AWS account admin (Astound)

**What.** Account `266359797998` has no model-invocation logging destination.

**Why it blocks.** Not a functional blocker; a REQ-026 (immutable AI audit trail)
review will ask for AWS-side evidence of what was sent to the models. Our own
audit spine records every tool call, but not the payloads Bedrock received.

**To unblock.** Enable Bedrock model-invocation logging to an S3 bucket or
CloudWatch log group in eu-central-1 and confirm the API key's IAM principal is
permitted to write there.

## B7 — Merging: this clone's gates are red for unrelated reasons  · owner: whoever lands the concurrent `extensions/web/admin` work

**What.** The knowledge-bank change set is staged but not committed. `just verify`
cannot go green in this working tree right now because a concurrent change in
`extensions/web/admin` (department-dimension removal, deleted JS assets) fails
`lint-inline-comments`, `check-dead-repository-code`, and root-workspace clippy,
and the stale debug binary aborts `just prepare` on the deleted assets.

**Why it blocks.** The knowledge-bank crate and both test crates pass on their
own; none of the failures name a knowledge-bank file.

**To unblock.** Land or stash the admin work, run `just prepare` (it applies the
knowledge-bank schema itself) and `just verify`, then commit. Review the index
before committing: it also holds that other work's staged deletions.

## B8 — pgvector on the Fly `postgres-flex` image is unverified  · owner: whoever runs B5

**What.** `flyctl postgres create` deploys `flyio/postgres-flex:17`. Newer tags of
that image are reported to ship `vector`; the tag Fly picks today has not been
checked.

**Why it blocks.** B5's `CREATE EXTENSION vector` either succeeds or does not, and
the answer decides whether the stock app is enough.

**To unblock.** Run `just knowledge-db-provision`. If the `CREATE EXTENSION` fails,
redeploy the Postgres app from `deploy/fly/knowledge-db/Dockerfile`
(`postgres-flex:17` + `postgresql-17-pgvector`) and re-run; then record which
path worked here and close the item.

## B9 — API access for the sources named in REQ-047 · owner: Customer / Astound IT

**What.** REQ-047 names Jira, Confluence, GitHub and Salesforce. We have
individuals' personal Atlassian tokens and nothing else.

- **Atlassian** — no 3LO OAuth application. The full per-user OAuth path is built
  and switched off (`handlers/atlassian_auth/`, `services/atlassian_oauth.rs`,
  sealed per-user credential table); without a client id and secret every user
  pastes a personal API token by hand. This is B4, now promoted from DEFERRED: it
  is the difference between a pilot and something an organisation can roll out.
- **Salesforce** — no connected app permitting per-user authorisation. Today there
  is one administratively mapped username and no per-user consent, so Salesforce
  access is not individually attributable the way the other sources are. Nothing
  is indexed; it is a live accessor only.
- **GitHub** — no connector, no access, no decision that it is in scope.

**Why it blocks.** REQ-048 (governed integrations across all three) cannot be
demonstrated, and REQ-052's per-user entitlement story is only as good as the
per-user credential behind it.

**To unblock.** Register the Atlassian 3LO app (classic scopes + `offline_access`,
callback `https://astound.systemprompt.io/admin/auth/atlassian/callback`), set
`atlassian_client_secret`, and enable the servers. Decide whether Salesforce is
indexed at all before commissioning a connected app for it, and confirm whether
GitHub is genuinely in scope — see DEC-009 in
`docs/decisions/register-rows-2026-09-04.md`.

## B10 — Validation, not deployment, is what keeps RAG switched off · owner: shared

**What.** `services/mcp/knowledge-bank.yaml` carries `enabled: false`. The reason
is **not** missing infrastructure. It is that the engine has never been run over
content anyone would use it on, and nobody has measured whether what it returns is
correct.

**Why it blocks.** Unvalidated retrieval is worse than no retrieval — a confident
wrong citation is harder to catch than a missing answer. B1 (no evaluation set)
and B2 (nothing readable to index) are the two inputs validation needs; B5 and B8
are prerequisites for running it at all, but clearing them alone would not justify
switching it on.

**To unblock.** Clear B1 and B2, index the corpus, run the benchmark, and record
the precision@5 number against REQ-046. Enable only once there is a number.

## B11 — No agreed outcome metric for productivity analytics · owner: Astound

**What.** Consumption is measured fully; there is no definition of what counts as
a result. The only outcome-shaped data is the model's own end-of-session
assessment of whether it met its goal.

**Why it blocks.** REQ-007 and REQ-008 are already `Blocked` on DEC-002 (which SCM
is authoritative, and how an external identity maps to a platform user). The same
question applies to Jira and Salesforce: each candidate outcome needs an API key
to sync that system and an identity mapping rule.

**To unblock.** Answer DEC-010 in `docs/decisions/register-rows-2026-09-04.md` —
pick one outcome signal and confirm credentials for its source system. Background:
`docs/decisions/2026-09-04-usage-to-outcome-mapping.md`.

---

## Operator checklist once B2 and B5 clear

```bash
# instance secrets (per profile)
systemprompt admin config secret set bedrock_api_key <key>
systemprompt admin config secret set knowledge_database_url 'postgres://knowledge:…@sp-a2f658d8bc5f-knowledge.flycast:5432/knowledge' --profile production

# scope: services/knowledge-bank/config.yaml
#   atlassian.base_url, sources.confluence.spaces, sources.jira.projects

# deploy, then verify from a client with the three headers in .mcp.json
just deploy-next
systemprompt plugins mcp logs knowledge-bank
# sync_knowledge → list_knowledge_sources → search_knowledge
# admins: knowledge_index_stats, prune_knowledge {dry_run: true}; just knowledge-db-report
```
