# Demonstrate RAG

You walk the Astound knowledge bank end to end, live, using exactly two MCP
servers: `atlassian` (the hosted Atlassian MCP, reached through this
instance as the signed-in user's connected account) and `knowledge-bank`
(the project-context RAG engine). At each stage you say what is happening,
run the call, check the result against a stated expectation, and mark it
PASS or FAIL. You never invent a result: a stage that cannot be checked is
reported as SKIPPED with the reason.

## Use when

- "Show me how the knowledge bank works."
- "Demonstrate the RAG flow." / "Prove the sync and search work for me."
- "Why can't I see a page in search?" (run the flow; the stage that fails
  is the answer.)

## Inputs

- **Search phrase** (optional). Default: the `[KB DEMO]` workspace phrases
  below, which exist in EAICOMM on every instance where the demo was seeded.
- **Source categories** (optional). Default `confluence` and `jira`.

## The flow

Run the stages in order. Print a one-line header per stage, the call you
make, the evidence, and the verdict. Keep evidence short: ids, counts, the
first cited `uri`.

### Stage 1 — Identity and the connected account

Explain: nothing Atlassian is configured on this client. The only credential
is the systemprompt bearer; the Atlassian token is banked server-side when
the user connects on `/admin/profile#connected-accounts`, and the gateway
swaps it in on every call to the `atlassian` server.

Call `atlassian` → `getAccessibleAtlassianResources` `{}`.

PASS when `resources` has at least one entry with a `cloudId`. Note the
`products[].access` value: `read-write` means the connector's write scopes
were granted; `read-only` means only reading. Record the cloud id for later.

FAIL when the call errors with "no token banked" or a 503: the account is
not connected. Say so and stop — every later stage depends on it.

### Stage 2 — What the index already holds for this user

Explain: the index is shared, but every count is *this user's*: a document
is visible only through a grant to the caller's user id, roles or AD group.

Call `knowledge-bank` → `list_knowledge_sources` `{}`.

Record, per category, the readable document count, the last sync time, and
any "last sync failed" reason. PASS when the listing renders. Quote the
`confluence` and `jira` lines verbatim.

### Stage 3 — Sync as the caller

Explain: `sync_knowledge` reads Confluence and Jira *through the connected
account*, so it indexes exactly what this user may read, and grants each
document to `user:<id>`. It runs in the background.

Call `knowledge-bank` → `sync_knowledge` `{"source_types": ["confluence", "jira"]}`.

PASS when the reply says the sync started "through your connected Atlassian
account". If it says Atlassian was skipped, or refuses with a link to
`/admin/profile#connected-accounts`, that is the graceful-degradation
branch: report it as FAIL for this stage, explain that the files categories
still sync, and continue to Stage 4 with whatever is indexed.

If the reply says a sync is "already in progress", the run started on an
earlier call in this session: treat it as PASS-pending and go straight to
polling. Call `sync_knowledge` at most once per run; a duplicate call never
starts a second sync.

Then poll `list_knowledge_sources` every 5 seconds, up to 3 minutes, until
the `confluence` and `jira` last-sync times are later than the ones you
recorded in Stage 2. Report the elapsed time and the new counts.

A single "Connection error. Retrying…" on the first `knowledge-bank` call
after the server was restarted is the client re-opening its MCP session
(the old session id is gone with the restarted child). The retry succeeds;
it is not a knowledge-bank fault and does not change the verdict.

### Stage 4 — Search with citations

Explain: hybrid retrieval — a vector arm and a full-text arm, each filtered
by the caller's grants inside the SQL, fused by reciprocal rank, reranked.
Every hit carries `uri`, `lines`, `date` and `date_source`.

Run two searches with `knowledge-bank` → `search_knowledge`:

1. `{"query": "why do cart totals round half-to-even for Larkspur", "source_types": ["confluence"], "top_k": 5}`
   PASS when a hit titled like `ADR-001` appears and its `uri` starts with
   the site and contains `/wiki/spaces/EAICOMM/pages/`.
2. `{"query": "Nightjar purge misses UTM-tagged listing URLs", "source_types": ["jira"], "top_k": 5}`
   PASS when a hit's `uri` contains `/browse/EAICOMM-`.

If the user gave their own phrase, run it as a third search and report the
top hit with its citation. Open nothing; the citation is the evidence.

### Stage 5 — Who may read what

Explain: an Atlassian document is granted to every user who synced it and
to nobody else; a files document is granted to its category's configured
readers. A user with no connected account, or who never synced, gets no
hits for these documents even as an administrator.

This stage is explanatory unless a second identity is available. If the
user can run the same search as another account, compare; otherwise mark
SKIPPED with "needs a second user" and state the rule.

### Stage 6 — Degradation

Explain the contract without breaking anything:

- account not connected or Atlassian unreachable → unscoped sync = files
  categories only, with the reason recorded per category and shown by
  `list_knowledge_sources`; scoped sync = refused with the same reason;
- one unreadable page or issue → skipped, counted, never fatal;
- full-text arm down → vector-only results; reranker down → fusion order.

Mark PASS if Stage 3 showed the connected branch, or if Stage 3 showed the
skipped branch with a reason (that is the contract working).

## Report

End with one table: stage, verdict, evidence (one line each). Then three
sentences: what was proven, what was not, and the single next action if
anything failed (usually "connect Atlassian on /admin/profile" or "run
`just knowledge-demo` to seed the workspace").

## Rules

- Use only the `atlassian` and `knowledge-bank` servers. No other MCP, no
  shell, no file writes.
- Never paste a token, a bearer or a cookie into the output.
- Never call an Atlassian write tool; this skill reads.
- Do not summarise page bodies; cite them.
