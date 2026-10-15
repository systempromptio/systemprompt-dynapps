# Workflows — finding the environment, and proving it answers

Everything else depends on this. The five location args are not bureaucracy: with 300+ live
environments, a call that guesses is a change on somebody else's business.

---

## 1. From nothing to the five location args

You have a customer name. You need `company_id`, `project_id`, `environment_id`, `db` and
`target_user_login`.

```text
list_my_companies                          → { id: 3, name: "Dynapps España" }
list_projects   { company_id: 3 }          → { id: 901, name: "acme-projects" }
list_environments { project_id: 901 }       → { id: 4022, name: "acme-staging",
                                               stage_type: "development", state: "running" }
list_databases  { environment_id: 4022 }    → { databases: ["acmedb"], auto_picked: "acmedb" }
list_target_users { environment_id: 4022, db: "acmedb" }
                                           → { login: "admin", is_admin: true }
```

**Read `stage_type` and `state` while you are there.** The stage tells you whether writes
will need a grant; the state tells you whether the environment is even running. Both save
you a confusing failure two calls later.

A project has many environments — `main`, `staging`, `dev`, per-consultant demos. **Never
assume `main`.** If the user did not name one, ask, and say which ones exist. Deploying a
change to `staging` when they meant production is recoverable; the reverse is not.

---

## 2. Prove the environment is usable before working on it

Two calls, both cheap, before any batch:

```text
list_databases    { environment_id: 4022 }        # proves it is up and dyncloud_auth works
system_odoo_version { ...five args... }          # proves dyncloud_api is installed, and what it can do
```

Why both: `list_databases` is served by **`dyncloud_auth`**, `system_odoo_version` by
**`dyncloud_api`**. They are different modules, and the second is the one that is
frequently missing. Getting a clean answer from the first and a **404** from the second is
not a contradiction — it is a precise diagnosis (`troubleshooting.md` §1).

From `system_odoo_version`, note `capabilities`: it decides whether file imports will work
(`raw_binary_write`).

---

## 3. "This project / environment does not appear"

Before treating it as a bug, remember the filters:

- `list_projects` shows only projects with **`mcp_enabled`** — and only those your user may
  see.
- `list_environments` applies a **cascade**: the project **and** the environment must both
  have `mcp_enabled`. Either flag off and it is invisible here and to every proxy tool.

So there are three different situations behind the same symptom, and they have different
owners:

| What is happening | Who fixes it |
| --- | --- |
| Not exposed to MCP (`mcp_enabled` off) | Technical team — it is a flag in the manager |
| Your user lacks access to that company/project | Whoever manages project permissions |
| It genuinely does not exist | Nobody — check the name with the user |

**Do not request a JIT grant for this.** Grants unlock tool categories, not visibility;
asking for one will not make the environment appear.

---

## 4. Working on several environments in one session

Two habits keep this safe:

**Do not lean on the active context.** `set_active_context` only serves the `list_*` tools;
ORM and admin calls always carry their own five arguments. That is the design working for
you — in a multi-environment session, the arguments in the call are the only truth about
where it lands.

**Name the environment in every message to the user.** "En acme-staging he creado…" costs
four words and is the difference between a reviewable session and an untraceable one. When
you repeat a change across environments, say which ones are done and which are pending
after each step, not only at the end.
