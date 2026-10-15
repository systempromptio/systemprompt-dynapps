# Documentation access — the MCP contract (shared)

**Every skill in the plugin gets its platform detail from the documentation MCP server, and
this file is the contract.** It states three things and nothing more: **how the MCP is
identified**, **how it is checked before the first lookup**, and **what happens when it is
not there**.

It deliberately does **not** teach how to query well — how to phrase a retrieval question,
which tool fits which need, how to chain tools, how large content comes back. That is
`common-skills:dynbrain`, and it is the single source of truth for it. **Don't reimplement
any of it here or in a skill: read that skill and use it.**

Hard rule, unchanged from before: **if it can't be looked up and isn't in the skill's core →
say so, say what is missing, and DON'T invent** (see
[`danger-gates.md`](danger-gates.md), rule 8).

## 1. Which MCP am I using? (never hardcode a name)

The documentation tools come from **an MCP server the user configured, and its name is
theirs to choose.** The same corpus can be called anything on another machine, or renamed
tomorrow. The callable tool ids therefore have the shape `mcp__*__docs_search`,
`mcp__*__docs_answer`, … where the middle segment is **whatever the user named their
server** — a segment you read at runtime, never one you write down.

**Identify the server by its tool family, never by its name.** The documentation MCP is the
one exposing the `docs_*` family — matched with the wildcard `mcp__*__docs_*`:

> This identification rule belongs to `common-skills:dynbrain`; it is restated here only
> because the preflight in § 2 depends on it. **If the two ever diverge, dynbrain wins** —
> it is the source of truth, this is its application.

- **Read** (the only ones a skill in this plugin uses): `docs_search`, `docs_answer`,
  `docs_get`, `docs_list_spaces`, `docs_list`, `docs_outline`, `docs_export`,
  `docs_metadata`.
- **Write** (`docs_create`, `docs_update`, `docs_move`, `docs_trash`, …): out of scope here,
  see § 5.

Decide like this:

| Situation | What you do |
|---|---|
| **Exactly one** connected server exposes `mcp__*__docs_*` | Use it. |
| **The user names one** ("use the Dyncloud docs one", "the internal MCP") | Use that one, and keep it for the rest of the session. |
| **More than one**, **none loaded**, or otherwise ambiguous | **Ask the user which server to use — before calling any tool.** Don't guess: the wrong server searches the wrong corpus and answers confidently from it. |

Once chosen, stay on that server for the whole task unless the user switches.

**On this Dynapps instance** the `docs_*` family is served by the portal's hosted `dynbrain`
server (portal.dynapps.es), and Odoo data by the portal's `dynapps-odoo` server. Both are
connected per user at `/admin/connectors` through the generic OAuth connector and run as that
user's own portal account. The name rule above still holds: a user may have named either
server something else in their own client.

## 2. Preflight (before the first lookup, not after it fails)

1. **Are the `docs_*` tools present?** Look for the `mcp__*__docs_*` family exposed by
   whatever agent you are running in. Namespacing differs per agent; the family doesn't.
2. **Present → use the MCP.** It is the only route to platform detail in this plugin. Resolve
   the documentation you need (§ 3), then query it as `common-skills:dynbrain` describes.
3. **Absent → `[STOP]`** — that is: don't answer as if you knew, and don't degrade silently;
   halt and hand the decision to the user. Tell the user the documentation MCP
   is how this plugin reads the platform, that it is not connected, and that **they must
   connect it and complete the OAuth handshake themselves — you cannot do it for them.**
   State the goal, not a menu path: how a server is connected depends on the agent (in Claude
   Code it is the `/mcp` command; other agents have their own config). Then degrade as in § 6.

The handshake is interactive by design, which is exactly why the absence is handled by
**prompting the user**, never by quietly answering from memory.

## 3. Which documentation (space) — discover it by name, never by id

The server exposes several documentations (*spaces*), and the platform one is **`Plataforma
Dyncloud`**. Its numeric id is **an implementation detail that may change**: hardcoding it is
a bug, and passing a wrong one silently searches somebody else's corpus.

- Call **`docs_list_spaces`** and pick the space **by matching its name** (the platform one
  is the one named for the Dyncloud platform).
- Only then pass that discovered id to narrow a query. **Never a literal id written into a
  skill**, and never one carried over from another session's notes.
- Navigating a space's structure (`docs_outline`) and everything else about querying is
  `common-skills:dynbrain`.

## 4. What the skills ask for

Skills in this plugin carry **questions, not paths**. The core that must survive an outage
stays in the skill (safety rules, routing, gotchas, the query protocol); every concrete fact
— a version, a repo path, a host, a chart value, a runbook step — is asked to the MCP at the
moment it is needed. A skill that repeats a fact instead of asking for it is a copy that will
go stale.

## 5. Read-only: a gap gets reported, not written

These skills use **read tools only**. If the documentation is missing something, is wrong, or
contradicts what you observe: **say so to the user and point at where it belongs.** Do not
create or edit documentation from a devops skill — authoring is a deliberate act with its own
skill (`common-skills:dynbrain`) and its own review, and retrieval quality depends on it.

Retrieval also **runs as the connected user** and honours their permissions, so an empty
result can mean "no access", not "not documented". Say which of the two you can't tell apart
rather than concluding the documentation is missing.

## 6. Degradation — what to do when there is no MCP

When the tools are absent, unreachable, or the user declines to connect them, the marker for
this state is **`MCP unavailable`**. Then, in this order:

1. **Say it plainly.** "The documentation MCP is not connected, so I can't confirm the
   platform detail." No silent best-effort.
2. **Ask the user to connect it** (§ 2.3) — the handshake is theirs, and you offer to resume
   the moment it is up.
3. **Still answer with the skill's core.** Safety rules, routing, the golden rules and the
   gotchas are embedded precisely because they must survive this; they are useful on their
   own.
4. **Mark explicitly what remains unconfirmed** — which parts of the answer would have come
   from the documentation and are therefore pending verification.
5. **Never invent** the missing part. Not a version, not a path, not a name.

**Known limitation, accepted on purpose.** Repo hosts and paths are documentation content
now, so with `MCP unavailable` a skill **cannot compose the full URL of a repo**. The correct
behaviour is to say the MCP is needed for that URL — not to reconstruct it from memory, which
is how a wrong host gets stated as fact.

## 7. From your skill

Reference this file; don't restate it:

```markdown
## Prerequisites
Platform detail is read live from the documentation MCP — see
`references/shared/docs-access.md` for the contract and `common-skills:dynbrain` for how to
query it. With **MCP unavailable**: answer from this skill's core and mark what stays
unconfirmed.
```

Related: [`danger-gates.md`](danger-gates.md) (the safety floor, rule 8 "NEVER invent") ·
[`access-model.md`](access-model.md) (what the user's level allows).
