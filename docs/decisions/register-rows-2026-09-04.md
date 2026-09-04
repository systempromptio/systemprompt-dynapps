# Draft register rows — 2026-09-04

Two new rows for the *Decisions* sheet of
`[Enterprise AI] systemprompt.io requirements register`. The sheet currently runs
DEC-001 to DEC-008 and carries **no row about knowledge or RAG**, so the questions
blocking REQ-046 to REQ-052 have never been formally put to Astound.

Columns match the sheet: `ID`, `Topic`, `Related REQs`, `Question for Astound`,
`Why it matters / what it blocks`, `Options & our recommendation`, `Owner`,
`Status`. Paste as-is. Do not edit the workbook from this repository.

---

## DEC-009

**Topic** — Knowledge source scope and permission propagation

**Related REQs** — REQ-046, REQ-047, REQ-048, REQ-049, REQ-052

**Question for Astound** — Four questions, all about scope rather than
engineering. (1) Which sources are actually in scope: Confluence and Jira are
implemented; is Salesforce to be *indexed* or does live query suffice; is GitHub
genuinely wanted? (2) Do you want a per-organisation document folder — a Google
Drive (or SharePoint) folder synced into the index and readable by that
organisation — or is operator-placed content sufficient? (3) Which Confluence
spaces and Jira projects should be searchable, and are the pilot users granted
read on all of them? (4) Who owns the evaluation set — roughly thirty questions
with their correct answering documents — that REQ-046's precision@5 target is
measured against?

**Why it matters / what it blocks** — Blocks REQ-046 (unmeasurable without an
evaluation set), REQ-047 and REQ-048 (source scope undefined), REQ-049 (no entity
model), and constrains REQ-052. The retrieval engine is built and deliberately
disabled: it has never been validated against real content, and validation needs a
real corpus. The readable scope today is one Confluence space of two pages and one
Jira project with no issues. Separately, REQ-052 assumes retrieval can filter on
"source-system entitlements" — ADFS sends group names only, from which we derive
three flat roles and one project label, so entitlement filtering is achievable per
source (read as the caller, or query live) and not by propagating claims.

**Options & our recommendation** — Recommend: (a) keep Confluence and Jira read as
the caller — the source system has already decided what each person may read, and
copying its ACL is a correctness liability; (b) leave Salesforce as a live
accessor and index only published knowledge articles if anything; (c) drop GitHub
from scope unless a concrete use case exists; (d) start with the existing
operator-placed file source and add a per-organisation Drive folder only if
self-service is genuinely wanted — a folder is a flat blast radius and a
mis-filed document has no second gate; (e) name the spaces and projects and grant
the pilot users read on them, which is the single change that unblocks the most.
Roughly one week to index and validate once a corpus and an evaluation set exist.

**Owner** — Astound

**Status** — Awaiting Astound

---

## DEC-010

**Topic** — Outcome definition for productivity analytics

**Related REQs** — REQ-005, REQ-007, REQ-008, REQ-045

**Question for Astound** — What counts as an outcome? Consumption is measured
fully — tokens, cost, latency, model, per user per day, with dashboards per user.
What we cannot say is whether that spend produced anything. Choose the outcome
signal: Jira issue transitions (cycle time, reopen rate, throughput), source
control (pull requests merged, review turnaround, change failure rate), Salesforce
records advanced or resolved, or a human-ratified version of the session goal we
already capture. And confirm we may have API credentials for whichever system is
chosen.

**Why it matters / what it blocks** — Blocks any claim that AI usage improves
delivery. Today the only outcome-shaped data is the model's own end-of-session
assessment of whether it met its goal — a useful coaching signal, not a business
metric, and we decline to present it as one. Nothing joins usage to a ticket
closing or a deal advancing. Each candidate needs two things: an API key to sync
that system, and an identity mapping rule from the external account to a platform
user. The second is the harder half and is already the substance of DEC-002, which
remains open.

**Options & our recommendation** — Recommend: pick **one** signal, not four.
Jira transitions are the most concrete for engineering work and the cheapest to
connect given Atlassian access is needed for the knowledge bank anyway; roughly
one week to ingest once credentials and the identity mapping rule are settled. If
no external credentials are forthcoming, fall back to human-ratified session goals
— no connector, and it converts a model opinion into a real label. Whichever is
chosen, reuse DEC-002's identity mapping rule rather than inventing a second one,
with a manual override table for the mismatches that always exist.

**Owner** — SP + Astound

**Status** — Awaiting Astound
