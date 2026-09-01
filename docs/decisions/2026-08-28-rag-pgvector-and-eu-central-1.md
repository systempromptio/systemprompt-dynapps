# 2026-08-28 — RAG on Bedrock: pgvector, eu-central-1, grants at query time

Status: decided and implemented. Supersedes the four open decisions in
`docs/tickets/req-047-knowledge-bank-mcp.md`.

## What prompted this

An AWS Bedrock API key arrived, which was the last external dependency on the
knowledge bank. Ten requirements (REQ-044 through REQ-052) had been sitting at
"Blocked externally" behind it. Before building, the key's actual reach was
probed rather than assumed.

## The three decisions

### Region: eu-central-1, enforced at boot

All three models REQ-047 names were invoked successfully in Frankfurt.
eu-west-1 carries **no rerank model at all** and currently 403s this account
pending verification; us-east-1 works but is outside the EU. The Node source
system independently defaults to eu-central-1 for the same reason, which is
corroboration rather than coincidence.

The subtlety is that REQ-037 data-residency routing **does not cover this
path**. That mechanism is `governance:` metadata on provider/model YAML plus
route `requires:` on the AI gateway, and the knowledge bank calls Bedrock
directly from the MCP process. Nothing in the gateway sees these calls. So the
region is enforced in code: `bedrock::assert_eu_region` runs at client
construction and refuses to start on a non-EU region, naming the requirement in
the error. Config alone would have been an invitation to a quiet edit.

Two further findings from the probe: Bedrock's managed Knowledge Bases are
denied to this key (403), confirming that in-process retrieval was the right
shape; and model-invocation logging is not configured, which REQ-026 reviewers
will ask about even though our own audit spine records every call.

### Store: pgvector, not a LanceDB port

The Node system uses LanceDB, a local file database, and porting it would have
been the smaller translation. pgvector won on REQ-052 rather than on storage:
it lets the `knowledge_document_grants` join run **inside** the candidate
queries. With a file store, ACL filtering would have had to sit in application
code above the store — where it still reads unauthorized rows, and where a
fixed candidate limit leaks how many were removed through the result count.

Cost: the local Docker Postgres moves to `pgvector/pgvector:pg18`, and any
deployment target must be able to `CREATE EXTENSION vector`. The migration
fails loudly where it cannot, which is deliberate — degrading silently to
text-only search would make the knowledge bank quietly worse at its job.

### Access control: at query time, from the identity already in hand

REQ-052 is the only P0 in the group and is **not** part of the migration. The
Node system is a single-user local tool whose own config says all storage stays
local; it has no notion of a caller.

The `KnowledgeRetriever` trait now takes the calling `Principal` on every
method that can surface content. This needed no new plumbing: each MCP tool
handler already received a `RequestContext` and discarded it. A principal
reduces to `(subject_kind, subject_value)` pairs reusing the vocabulary of
`access_control_rules` — `user`, `adfs_group`, `role` — so a document grant and a marketplace entitlement describe a
principal identically, rather than inventing a second scheme.

Two consequences are load-bearing and were chosen, not inherited:

- **A document with no grants is reachable by nobody.** Ingestion writes grants
  from the source system's own permissions (REQ-048); a document whose
  permissions could not be read gets none. A missing document is a support
  ticket; a leaked one is an incident.
- **`list_sources` reports the caller's counts, not the index's.** True totals
  would disclose how much exists in a category the caller cannot read.

Grants are rewritten wholesale on each sync rather than merged, so a permission
removed upstream disappears at the next sync. The propagation lag is exactly
the sync interval — stated plainly because REQ-052 asks that changes propagate,
not that they propagate instantly.

## Also decided, more briefly

- **Ingestion runs as scheduler jobs**, not a new `extensions/cli/*` binary.
  The instance already schedules jobs and an operator can trigger one manually;
  a second deployment surface bought nothing.
- **Archive categories are enforced in the retriever**, not merely described in
  the tool text. `source_types` stays optional, but omitting it now searches
  every *live* source and archive categories must be named. Access control is
  what makes the broad default safe.

## What this does not unblock

Nothing on the AWS side remains. The blockers are source access: Jira 401s on
every call for want of an application licence, and Confluence exposes one page
in one space. So the index is empty, the server ships `enabled: false`, and the
REQ-046 precision@5 fixture is checked in as an explicit placeholder that
refuses to report a score. See `rag-access-request.local.md`.

GitHub and Salesforce, also named in REQ-048, have no access request raised.
