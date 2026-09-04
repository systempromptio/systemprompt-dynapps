# 2026-09-04 — RAG source scope and the permission model per source

**Status:** partly decided. The per-source grant shape and the residency constraint
are settled and implemented. Source scope (Drive, Salesforce, GitHub) is open and
belongs to the Customer. Complements
`2026-09-01-knowledge-bank-remote-shared-index.md` (transport, shared index,
per-user credentials) and `2026-09-02-knowledge-bank-file-sources-restored.md`
(file sources).

## Context

Jay Dave's requirements REQ-046 through REQ-052 describe an enterprise knowledge
platform over Jira, Confluence, GitHub and Salesforce, with retrieval filtered by
"the user's identity, project, role, permissions, and source-system entitlements".
All are `Open` in the register, and our standing answer on most of them —
"blocked on the knowledge-platform backend" — is true but says nothing about who
decides what.

The *Decisions* sheet of the register carries DEC-001 to DEC-008 and **no row about
knowledge or RAG at all**. The questions that actually block REQ-046 to REQ-052
have therefore never been put to the Customer. This document names them, and
records what has already been settled so those parts are not reopened.

The engine itself is built and disabled. It is disabled **because it has not been
validated against real content** — it passes its own suites and has been run over
fixtures and one two-page Confluence space, and nobody has measured whether what
it returns is correct. Unvalidated retrieval is worse than no retrieval: a
confident wrong citation is harder to catch than a missing one.

## Decided

1. **Confluence and Jira are read as the caller.** Each user syncs with their own
   Atlassian credentials and a document is granted to `user:<caller>` only. We do
   not re-derive Atlassian's ACL — space permissions, page restrictions, project
   roles and issue security schemes would have to be modelled and then kept
   correct as they change, and every copy of an ACL eventually disagrees with its
   original. Accepted cost: the index is the union of what individuals could see,
   not an organisational mirror.

2. **Grants are deny-by-default and joined inside the retrieval query.** A
   document the caller cannot read is never scanned, never ranked, and never
   reaches the reranker. A document with no grants is reachable by nobody and is
   removed by the next complete sync or prune.

3. **Embedding and rerank stay in `eu-central-1`.** Enforced at configuration
   load; a non-European region is refused rather than accepted. Residency cannot
   drift through a config edit. (Recorded in
   `2026-08-28-rag-pgvector-and-eu-central-1.md`.)

4. **Salesforce is a live accessor, not an index — for now.** Object- and
   field-level security plus sharing rules and role hierarchy are evaluated per
   record and are not expressible as a document grant; a snapshot of them is stale
   as soon as a sharing rule changes.

5. **Skills route between stores; there is no single search box.** A Jira issue is
   a state machine, a Confluence page is prose, a Salesforce record is a row, a
   vector chunk is a fragment with a citation pointer. The retrieval engine is one
   tool among several, and skills encode which store answers which question.

## Open — these belong to the Customer

1. **The per-organisation folder.** Proposal: one Google Drive folder per
   organisation, synced as a file source and granted to that organisation's
   members.

   *For:* it is the only source where we own the permission model rather than
   inheriting one, it is explainable in a sentence, it needs no per-user consent
   to a third-party system, and it gives an organisation a self-service path that
   does not involve an engineer.

   *Against:* a folder is a flat blast radius — one mis-filed document is a
   disclosure with no second gate. It adds a connector, an OAuth application and a
   sync to operate, and it creates a second copy of material that usually already
   lives somewhere governed.

   *Alternatives:* keep the existing operator-placed file source with configured
   grants per category (no connector, no OAuth, no self-service); or SharePoint /
   OneDrive instead of Drive, which is the better fit if the corporate directory
   is Microsoft — the ingestion side is identical, only the connector changes.

   Nothing is built. There is no Drive connector, folder convention, or sync in
   the repository.

2. **Is Salesforce indexed at all?** If yes, only genuinely static material
   (published knowledge articles) is a safe candidate; records should stay live.

3. **Is GitHub in scope?** REQ-047 names it. No connector exists. If it is not
   actually wanted, the requirement should be narrowed rather than left open.

4. **Who owns the evaluation set?** REQ-046's precision@5 target is a number that
   cannot be produced without an agreed question set and its answering documents.

## The identity gap, stated once

REQ-052 assumes retrieval can be filtered by the user's "source-system
entitlements". It cannot be, from claims.

ADFS conveys an email address, a display name, and a list of group names. From
those we derive a flat three-tier role model (administrator, project manager,
user) and one project label (`commerce` or `core`). That is the entire entitlement
signal the directory sends. There are no per-user tiers, data-classification
levels, clearances, or source-system entitlements.

REQ-052 is therefore satisfiable **per source** — by reading as the caller, or by
querying live — and not by propagating claims. A richer model requires either new
claims from the directory or a connector that reads each source system's own
permission model directly, and both are decisions rather than configuration.

## Consequences

- The published page `services/content/documentation/enterprise-knowledge-rag.md`
  states this architecture, its limits, and the open decisions, so the Customer
  reads the same account we hold internally.
- DEC-009, drafted in `register-rows-2026-09-04.md`, puts the open items above to
  the Customer in the register's own format.
- No runtime behaviour changes. The knowledge bank stays disabled until there is a
  corpus to index and an evaluation set to judge it by.
