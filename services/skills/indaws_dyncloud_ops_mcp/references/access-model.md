# The JIT access model

Why a call you are allowed to make comes back refused, and what to do about it.

Two independent things must both say yes: the **Cloud permission** the tool declares
(`:backup_create:`, `:environment_remove:`, …), which is the same check a human gets in
the web UI, and the **category grant** for the stage the environment sits on. The first is
about who you are; the second is about whether that power is switched on right now.

---

## Categories

Every tool declares one. The category — not the tool — is what gets granted.

| Category | What lives there | Typical posture |
| --- | --- | --- |
| `access_meta` | The grant tools themselves, the active context | Always allowed |
| `navigation` | `list_*` discovery | Open |
| `platform_read` | `platform_*` reads | Open |
| `platform_ops` | Deploy, lifecycle, domains, submodules, backups, action control, project creation | Open on development, **by request on staging AND production** |
| `platform_destructive` | Restore, destroy | By request on **all three** stages, and normally signed by a human |
| `ssh_exec` | `docker_ssh_exec` | By request on **all three** stages, second pair of eyes required |
| `data_read` / `data_write` / `schema` / `module_admin` / `system_ops` | The customer-Odoo proxy families | Usually absent from an ops server |

**The exact seeded modes**, from `cloud_broker/data/cloud_broker_access_seed.xml` — worth
reading once, because three of these are commonly guessed wrong:

| Category | development | staging | production |
| --- | --- | --- | --- |
| `platform_read` | open | open | open |
| `platform_ops` | open | **by_request** | **by_request** |
| `platform_destructive` | **by_request** | by_request | by_request |
| `ssh_exec` | **by_request** | by_request | by_request |

> **Staging is NOT open for `platform_ops`.** This is the one that surprises people: a
> deploy, a backup or a scale on a *staging* environment needs a grant exactly like
> production does. Only development is open. Plan for the grant at the start of the task
> rather than discovering it at the first write.
>
> And `ssh_exec` and `platform_destructive` are `by_request` even on **development** —
> there is no stage anywhere on which a shell or a destroy is free.

Seeded values are the defaults, not a guarantee: an operator can change any line per
stage. When it matters, `get_my_mcp_access` tells you what you actually hold, and the
refusal itself names the stage it evaluated.

`access_meta` short-circuits the whole engine: you can always ask for access and always
read your own grants, otherwise a locked-down stage would be a locked door with the key
inside.

## Modes

Each **stage** (development, staging, production…) carries one access line per category,
and the line's `mode` decides:

- **`open`** — allowed, no grant needed. Reads and navigation on most stages.
- **`by_request`** — needs an active grant. This is where `E_CATEGORY_NEEDS_GRANT` comes
  from.
- **`forbidden`** — never grantable on this stage. `E_CATEGORY_FORBIDDEN`, and asking for
  a grant will fail too. **Do not retry, and do not request** — tell the user instead.

A category with **no line at all** on the stage is denied fail-closed (`E_NO_LINE`), as is
an unknown category. Silence is never permission.

## The refusal, and the fix

```json
{
  "ok": false,
  "error_code": "E_CATEGORY_NEEDS_GRANT",
  "error": "Category 'platform_destructive' on stage 'production' requires an active JIT grant...",
  "request_hint": { "scope": "category", "category": "platform_destructive",
                    "duration_hours": 1, "tool": "request_mcp_access", "environment_id": 42 }
}
```

**Nothing was prepared and nothing was enqueued.** The engine runs before the handler, so
a refusal leaves no half-done work behind.

The `request_hint` is the call to make, near enough verbatim:

```text
request_mcp_access {
  environment_id: 42,
  scope: "category",
  category: "platform_destructive",
  duration_hours: 1,
  reason: "<why — this is stored in the audit log and read by whoever approves>"
}
```

`reason` is mandatory and it is not a formality: on a grant that needs signing, it is all
the approver sees. Write what you are going to do and to what, not "need access".

## Who gets it immediately, and who waits

Two flags on the access line decide, and they exist for different reasons.

**`requires_second_approver`** — when set, the approver shortcut does not apply: the
request is born `pending` **even if you are an approver yourself**. Seeded on `ssh_exec`,
because arbitrary shell on a customer host should always cost a second pair of eyes.

**`allow_interactive_self_approval`** — when set, you may approve your own request, but
**only from a human web session, never over MCP/API**. This is the two-channel control:
request with a token, approve with a browser. A stolen MCP token can ask for anything and
approve nothing.

So the answer to `request_mcp_access` is one of two shapes:

```json
{ "state": "active",  "auto_approved": true,  "remaining_minutes": 120, ... }
{ "state": "pending", "auto_approved": false, "requires_approval": true,
  "approver_count": 4, "next_step": "Grant is pending approval..." }
```

On `pending`: **say so and stop.** Tell the user which grant (`name`, e.g.
`BKRG/2026/00010`) needs signing and what it is for. Do not retry the blocked call, and do
not try to approve it from here — the tools that could are deliberately unavailable to an
MCP caller.

Poll `get_my_mcp_access` to see when it flips.

## Grants are idempotent, and reusable — but scoped to ONE environment

Asking again for a scope you already hold returns the existing grant with
`"idempotent": true` rather than creating a second one. So the polite move before a chain
of destructive work is to ask **once**, for a duration that covers the whole run — not per
call.

**Once per environment, though, not once per project.** The grant carries an
`environment_id` and covers that environment alone: a fleet-wide task across a project's
production and staging needs one request each, and a nine-environment sweep needs nine.
They are auto-approved for a developer, so this is bookkeeping rather than a wait — but
build it into the plan instead of hitting `E_CATEGORY_NEEDS_GRANT` on environment two
after environment one went through.

Give the `reason` something a reader will understand months later, in the audit log,
without this conversation: what is being changed and why, not "operations".

Read `effective_state`, not `state`: a grant whose row still says `active` may already be
`expired`, and `effective_state` is the one computed against the clock. `remaining_minutes`
tells you whether it will outlast what you are about to do.

When the work is done, `revoke_mcp_access` gives it back. Least privilege is not a
ceremony: an environment left write-open for four hours after a five-minute task is four
hours of exposure nobody chose.

## What the engine checks, in order

Worth knowing when a refusal does not match the table above:

1. **Kill-switch.** `mcp_enabled: False` on the environment or its project beats
   everything, including break-glass.
2. **Break-glass**, on the admin server only, for members of the break-glass group. Audited
   and logged at WARNING.
3. **`capability: "meta"`** — the access tools, always allowed.
4. Resolve the category; unknown → deny.
5. Resolve the stage's line for that category; missing → deny.
6. Apply the mode.
7. **Masking** is decided separately: a stage may enforce PII masking regardless of the
   above, and a `scope: "unmask"` grant is what lifts it.
