# Navigation — finding the thing you are about to act on

With hundreds of live environments, guessing an id is how you operate on somebody else's
business. These tools are cheap, read-only, and exist so you never have to.

Shared conventions in `README.md`.

---

## The chain

```text
list_my_companies {}          → company_id
list_projects { company_id }  → project_id
list_environments { project_id } → environment_id   ← what every other tool wants
```

Each step filters by what the caller may see **and** by the `mcp_enabled` kill-switch, so
an empty answer means "nothing visible to you", never "nothing exists".

---

## list_my_companies

The customer/country companies the caller has access to. No arguments — derived from the
user's `company_ids`.

Use it to start when you do not know where you are. Skip it when the user named the
project and you can find it another way.

---

## list_projects

Input: `{ "company_id": 7 }` (optional — falls back to the active context).

Filtered by ACL **and** `project.mcp_enabled`. A project not exposed to MCP simply does not
appear.

---

## list_environments

Input: `{ "project_id": 12 }` (optional — falls back to the active context).

The step that matters, because its answer tells you **what kind of environment you are
dealing with** before you plan anything:

```json
{ "id": 3, "name": "main", "uuid": "fe136141-…", "primary_url": "https://…",
  "aud": "…", "stage_type": "production", "state": "running",
  "has_connect_as_perm": true, "project_type": "k8s", "project_id": 12 }
```

- `project_type` — **docker or k8s**. Read it before choosing a tool; half of them behave
  differently and some refuse outright.
- `stage_type` — which access line applies, so it predicts whether you will need a grant.
- `state` — lifecycle (`running`, `destroying`, `error`), not health.

Cascade rule: an environment is invisible if **either** its own `mcp_enabled` or its
project's is off.

---

## list_databases · list_target_users

Both reach the customer Odoo over HTTP rather than reading Cloud's own records, so they
cost a round trip and can fail for reasons that have nothing to do with the platform.

`list_databases { environment_id }` — most environments have exactly one, returned as
`auto_picked`. `list_target_users { environment_id, db }` — the logins available to
impersonate.

**On an ops server you rarely need either.** They feed the customer-Odoo proxy tools
(`orm_*`, `admin_*`) documented in the `dyncloud-mcp` skill. Reach for them when you are
about to cross into that other server, not to operate the environment.

---

## list_actions

History of a project's or environment's actions with the registry breakdown —
`effective_state`, exception class, timeline, whether a trace still exists — but
**without** the registry payload.

Input: `{ "project_id": 12 }` or `{ "environment_id": 3 }`, optional `limit`.

The split is deliberate: navigation answers *what exists and in what state*;
`platform_list_actions` (in `tools-read.md`) answers *why it failed and with what data*,
and only it can return the payload, which carries deployment configuration.

Use this one to scan; use the other when something needs explaining.

---

## list_release_tags

**k8s only** — `E_NOT_SUPPORTED_FOR_RUNTIME` elsewhere.

The release tags an environment can actually be upgraded to. This is the only producer of
the `release_tag_id` that `platform_deploy(op="upgrade_release")` requires; without it that
operation cannot be performed from MCP at all.

Input: `{ "environment_id": 3 }` (optional — active context).

```json
{ "id": 127, "tag": "odoo-19.0-r24-rc8", "release_number": 24, "is_rc": true,
  "is_current": false, "commit_sha": "d9d1b2f5", "release_mode": "templated",
  "has_content": true, "is_obsolete": false,
  "source": "releases-dev", "profile": "odoo-19.0 @ releases-dev",
  "last_refreshed_at": "…" }
```

- The set is **exactly** what the deploy wizard would offer for this environment — same
  app, same release source when one is pinned — not every tag that exists. Ids from here
  are ids `platform_deploy` accepts.
- **`is_current`** marks the environment's pinned release. Without checking it you cannot
  tell an upgrade from a no-op.
- `has_content: false` means the tag was discovered but its spec was never fetched;
  `is_obsolete: true` means it has been superseded.
- **`E_NO_TARGET_APP`** when the environment has no app assigned — an explicit error, not
  an empty list, because the wizard blanks the set there on purpose to prevent a cross-app
  pin. "I cannot tell which apply" is not "there are none".

Newest release first. Be aware it returns **everything**, with no `limit` and no way to
filter obsolete tags — a large answer for the sake of one id.

---

## The active context

```text
set_active_context { company_id, project_id, environment_id, db, target_user_login }
get_active_context {}
clear_active_context {}
```

All fields optional, partial sets allowed, and each is validated against the last (a
project must belong to the company, an environment to the project).

**Only the `list_*` discovery tools read it.** Tools that act on an environment want the id
in the call, deliberately: with this many environments, a silent fallback on a write is
precisely the accident worth engineering against.

Set it while exploring one environment; clear it when you move on, so a later call cannot
inherit a stale target.
