---
title: "Enterprise Knowledge & RAG"
description: "The knowledge bank architecture: which sources are indexed, how each one is permissioned, why tiered permissions do not arrive from the identity provider, and what has to be supplied before Jira, Confluence and Salesforce can be searched."
author: "Astound Digital"
slug: "enterprise-knowledge-rag"
keywords: "rag, knowledge bank, retrieval, pgvector, confluence, jira, salesforce, google drive, permissions, access control"
kind: "guide"
public: true
tags: ["enterprise", "knowledge", "rag"]
published_at: "2026-09-04"
updated_at: "2026-09-04"
after_reading_this:
  - "Know which knowledge sources are built, which are proposed, and which are not started"
  - "Understand how a document's readership is decided, source by source"
  - "See why identity-provider groups cannot carry source-system entitlements"
  - "Know exactly what has to be supplied before the pipeline can index anything real"
related_docs:
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
  - title: "Usage, Adoption & Productivity Analytics"
    url: "/documentation/enterprise-analytics"
  - title: "Authentication & Login"
    url: "/documentation/authentication"
---

# Enterprise Knowledge & RAG

**TL;DR:** the retrieval engine is built. It is switched off because it has never been validated against real content, and it cannot be validated until we have access to that content. This page states the architecture, the permission model source by source, the decisions still open, and what has to arrive before any of it indexes anything real.

## What exists today

The knowledge bank is an MCP server. Clients reach it the same way they reach any other governed tool, and it exposes six tools: `search_knowledge`, `list_knowledge_sources`, `sync_knowledge`, `add_knowledge_document`, and — for administrators — `knowledge_index_stats` and `prune_knowledge`.

Retrieval is hybrid. Every chunk carries both a 1024-dimension embedding and a Postgres full-text vector; the two candidate sets are fused by reciprocal rank fusion and then reranked. Embeddings are Amazon Titan v2, reranking is Cohere Rerank v3.5, and both run on Bedrock in **eu-central-1** — the only European region offering rerank. The region is enforced when configuration loads: a non-European region is refused rather than silently accepted, so residency cannot drift through a config edit.

Access control is not a filter applied to results. Grants are joined **inside** the candidate query, so a document the caller may not read is never scanned, never ranked, and never reaches a reranker. A document with no grants is reachable by nobody.

**It is not switched on in the current deployment, because it has not been tested against real content.** The engine passes its own test suites and has been exercised against fixtures and one small Confluence space; it has never been run over a corpus anyone would use it on, and nobody has measured whether what it returns is right. Shipping retrieval that has not been validated is worse than shipping none, because a confident wrong answer is harder to catch than a missing one. The server stays disabled until there is a corpus to index and an evaluation set to judge it by. The code, its schema, and its test suites are in the product; enabling it is a line of configuration, and the work in between is validation, not development.

## The four source classes

Knowledge does not arrive from one place, and the four candidate sources differ in the only way that really matters: **who decides who may read a document.**

### Confluence and Jira — read as the caller

Implemented. Each user syncs with their own Atlassian credentials, and a document is granted to `user:<that caller>` and nobody else.

This is deliberate. Atlassian has already decided what that person may read; re-deriving its ACL into our grant table would mean modelling space permissions, page restrictions, project roles and issue security schemes, and then keeping the copy correct as they change. Reading as the caller makes that impossible to get wrong — a page restricted to a group is findable by exactly those members of the group who ran a sync. The cost is that the index is a union of what individuals could see, not an organisational mirror, and a page nobody has synced is a page nobody can find.

### Salesforce — not indexed

There is no Salesforce ingestion. Salesforce is reached as a **live accessor** instead: a per-call bearer token minted through RFC 7523 JWT-bearer against an administratively mapped Salesforce username. Queries hit the org and return current data; nothing is copied into the index.

Salesforce is the hardest of the four to index, and the reason is structural. Its read model is object-level and field-level security, plus sharing rules, plus role hierarchy, evaluated per record. None of that is expressible as a document grant, and a snapshot of it goes stale the moment a sharing rule changes. Live query is the honest answer for records; indexing would only be right for genuinely static material such as published knowledge articles.

### GitHub — not started

Named in the requirements as a fourth source. No connector exists.

### A per-organisation file folder — proposed, decision pending

The three sources above all inherit somebody else's permission model. There is a case for one source where we own it outright: a folder per organisation, synced into the index as a file source.

**The proposal.** Each organisation on the platform gets a Google Drive folder. Its contents sync into the knowledge bank on a schedule and are granted to that organisation's members. Anything the organisation wants its agents to know — standards, playbooks, contracts, meeting notes — is dropped in a folder, and appears in search.

**The argument for it.** It is the only source with a permission model simple enough to explain in a sentence, and the only one that needs no per-user consent to a third-party system. It also gives an organisation a self-service path that does not involve an engineer.

**The argument against it.** A folder is a flat blast radius. Everything in it is readable by everyone the folder is granted to, so a single mis-filed document is a disclosure with no second gate to catch it. It also adds a connector, an OAuth application, and a sync to operate — and it introduces a second copy of documents that usually already live somewhere governed.

**Alternatives, if the answer is no.** Keep the existing file source, where categories are configured with explicit grants and content is placed by an operator: no new connector, no OAuth, but no self-service either. Or use SharePoint/OneDrive rather than Drive, which is the better fit if the corporate directory is Microsoft — the ingestion side is identical, only the connector changes.

**This is not built and not decided.** No Drive connector, folder convention, or sync exists today.

## Querying: skills, not a single search box

A single `search()` over everything is the wrong interface, because these stores answer different kinds of question. A Jira issue is a state machine — its value is status, assignee, transitions, and age. A Confluence page is prose, and the useful unit is a passage with a citation. A Salesforce record is a row, best reached by a precise query rather than a similarity match. A vector chunk is a fragment that has to be handed back with a pointer to its source.

The intended shape is therefore **skills that know which store answers which question**, and route accordingly: a question about what was decided goes to the vector index; a question about what is in flight goes to Jira; a question about an account goes to Salesforce, live. The retrieval engine is one tool among several rather than the front door.

## Permissions, and the gap

Grants are expressed in the platform's own authorization vocabulary — `user:`, `role:`, and Active Directory group — so a knowledge grant reads like every other access rule in the system. Retrieval is deny-by-default and filtered inside the query, and denials are audited like any other tool call.

**Tiered permissions do not arrive from the identity provider.** This is the single most important limitation on this page, and it is worth stating precisely.

Sign-in through ADFS conveys an email address, a display name, and a list of group names. From those groups we derive a **flat three-tier role model** — administrator, project manager, user — and **one project label** (Commerce or Core). That is the entire entitlement signal available from the directory. There are no per-user tiers, no data-classification levels, no source-system entitlements, and no clearance attributes, because the directory does not send any.

The consequence: a requirement for retrieval filtered by "the user's source-system entitlements" cannot be met by propagating identity-provider claims, because those claims do not carry entitlements. It can only be met **per source** — by reading as the caller (as Confluence and Jira do), or by querying live (as Salesforce does). Any richer model requires either new claims from the directory, or a connector that reads each source system's own permission model directly. Both are decisions, not configuration.

## Where each requirement stands

| Ref | Requirement | State |
|---|---|---|
| REQ-044 | User awareness — access constrained by session project, role and permissions | Supported for MCP servers and models; knowledge sources follow once the bank is enabled |
| REQ-045 | Conversation history, searchable and identity-aware | History is captured today; see [Conversation History & Search](/documentation/enterprise-conversation-history) |
| REQ-046 | Knowledge accuracy SLO, precision@5 above 70% | Blocked — no benchmark corpus or agreed question set exists to measure against |
| REQ-047 | Enterprise RAG pipeline over Jira, Confluence, GitHub and Salesforce | Engine built; Jira and Confluence implemented; GitHub not started; Salesforce live-query rather than indexed |
| REQ-048 | Governed integrations with Salesforce, Confluence and Jira | Confluence and Jira read as the caller; Salesforce is a live accessor. Blocked on API access and OAuth applications |
| REQ-049 | Knowledge graph for semantic queries | Not started. Requires an agreed entity and relationship model first |
| REQ-050 | Use-case examples library | Not started. Requires a taxonomy and a curation owner |
| REQ-051 | User-facing searchable conversation history | Captured; the user-facing search surface ships with the knowledge platform |
| REQ-052 | Context-aware retrieval access control | Retrieval-time filtering is built and tested. Source-system entitlement propagation is constrained by the identity gap described above |

## What has to be supplied

Every item below sits outside engineering. Each blocks something specific.

**Atlassian OAuth application.** A 3LO application — client id, client secret, classic scopes plus `offline_access`. The full per-user OAuth path is already implemented and switched off; without the application, every user pastes a personal API token by hand. This is the difference between a pilot and something an organisation can roll out.

**Salesforce API access.** A connected app permitting per-user authorisation. Today there is one administratively mapped username and no per-user consent, which means Salesforce access is not individually attributable in the way the other sources are.

**GitHub API access**, if GitHub is genuinely in scope as a fourth source. If it is not, the requirement should be narrowed rather than left open.

**A corpus worth indexing.** The configured scope today is one Confluence space of two pages and one Jira project with no issues. Retrieval quality cannot be demonstrated, let alone measured, against that.

**An evaluation set.** The 70% precision@5 target is unmeasurable without an agreed set of questions and their correct answering documents. Roughly thirty questions is enough to make the number meaningful.

**A pgvector database and a Bedrock key.** Operational prerequisites for running the server at all. These are provisioning tasks rather than decisions, but they gate the validation work above.

## Decisions still open

1. **Does the per-organisation Drive folder get built** — and if so, Google Drive or SharePoint? Or is the existing operator-placed file source sufficient?
2. **Is Salesforce indexed, or does it stay live-query only?** Indexing means accepting that a document grant cannot express Salesforce sharing rules.
3. **Is GitHub in scope**, and if so as code, as pull-request discussion, or both?
4. **Who owns the evaluation set** and the retrieval-quality process once it exists?
5. **Is a knowledge graph actually wanted**, or is it a proxy for questions that relationship-aware retrieval over the existing sources would answer?
