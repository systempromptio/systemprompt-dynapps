# DynCloud MCP — navigation tools

Eight tools whose only job is to answer *which* environment you are about to touch. They
are cheap, they are read-only, and skipping them is how somebody writes to the wrong
customer. Shared conventions are in `README.md`.

The chain is fixed, and each step produces the id the next one needs:

```
list_my_companies → list_projects → list_environments → list_databases → list_target_users
   company_id         project_id      environment_id        db              target_user_login
```

Those five values are exactly the five location args every ORM/admin call demands.

---

## list_my_companies

The companies the calling user may act on. The entry point when you have nothing.

```json
{}
```
```json
{ "items": [ { "id": 3, "name": "Dynapps España" } ] }
```

Notes: filtered by the caller's own company access in the manager. One company is the
normal case; more than one means you must pick before anything else.

---

## list_projects

Projects of a company. Feeds `project_id` into `list_environments`.

```json
{ "company_id": 3 }
```
```json
{ "items": [ { "id": 901, "name": "acme-projects", "company_id": 3 },
             { "id": 900, "name": "acme-erp", "company_id": 3 } ] }
```

**Filtered by `project.mcp_enabled` as well as by ACL.** A project that exists but is not
exposed to MCP simply is not in this list. When somebody insists a project exists and you
cannot see it, that is the first explanation — not a bug and not a permissions error you
can request your way out of.

`company_id` is optional if you set an active context.

---

## list_environments

The individual customer Odoo instances of a project. This is where `environment_id` comes
from, and it carries the fields that decide how the rest of the session behaves.

```json
{ "project_id": 900 }
```
```json
{ "items": [ { "id": 4021, "name": "main",
               "uuid": "<environment uuid>",
               "primary_url": "https://<environment fqdn>",
               "aud": "<environment fqdn>",
               "stage_type": "production", "state": "running",
               "has_connect_as_perm": true } ] }
```

Read three of them before doing anything else:

- **`stage_type`** decides your permissions. `development` usually has most categories
  open; `production` gates writes and module admin behind JIT grants (`access-jit.md`).
- **`state`** tells you whether it is running. A stopped or hibernated environment answers
  nothing, and that is not a failure of your call.
- **`uuid`** is the environment's identity across the platform — worth quoting in a report
  to the technical team.

**Cascade filter:** an environment is visible only if **both** its project and itself have
`mcp_enabled`. Either flag off and it is invisible here *and* to every proxy tool.

---

## list_databases

The databases on that environment. **This is a real HTTP round-trip to the customer's
Odoo**, served by `dyncloud_auth` — so a successful answer proves three things at once: the
environment is up, its `dyncloud_auth` is installed, and it verified the manager's signed
token.

```json
{ "environment_id": 4022 }
```
```json
{ "databases": ["acmedb"], "auto_picked": "acmedb" }
```

`auto_picked` appears when there is exactly one, which is the usual case — take it.

**Use this as your health check.** It is the cheapest call that exercises the whole
signing chain, and it needs no grant on any stage. If it works and an `orm_*` call 404s,
you have learned something precise: `dyncloud_auth` is there and `dyncloud_api` is not
(`workflows/troubleshooting.md` §1).

---

## list_target_users

The logins you may impersonate on that database. Also a live round-trip.

```json
{ "environment_id": 4022, "db": "acmedb" }
```
```json
{ "users": [ { "id": 2, "login": "admin", "name": "Administrator",
               "share": false, "active": true,
               "is_admin": true, "is_default_admin": true } ] }
```

**Whichever login you pick, that user's permissions and record rules apply on the target.**
Pick deliberately: an admin login for `admin_*` tools and for configuration; a regular user
when you want to see the data *as that user sees it* — which is the honest way to answer
"why can't my client see this record?".

---

## set_active_context / get_active_context / clear_active_context

Store `company_id` / `project_id` / `environment_id` / `db` / `target_user_login` for the
session so the `list_*` tools stop needing them.

```json
{ "company_id": 3, "project_id": 900, "environment_id": 4021, "db": "odoo",
  "target_user_login": "admin" }
```

**Only the `list_*` discovery tools read it.** ORM and admin tools ignore it entirely and
still require all five arguments explicitly. That asymmetry is deliberate, not an
oversight: exploring with a remembered context is convenient, while writing to an
environment you did not name in the call itself is the failure mode the whole design is
built to prevent.

It validates as it stores — company in your allowed set, project inside the company,
environment inside the project — so a context that saves is a context that is coherent.

`get_active_context` reads it back (worth doing before a long session, to be sure you are
where you think). `clear_active_context` drops it.
