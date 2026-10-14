# Workflows — when something does not add up

The MCP fails in a handful of recognisable ways. Recognising them is most of the work: the
difference between "no funciona" and "this environment is missing `dyncloud_api`" is the
difference between an afternoon and a five-minute fix.

**Whatever you conclude, adapt the delivery to who is asking** (§5, and `../../../SKILL.md` →
"Who is asking?").

---

## 1. 404 on every ORM call — the most common failure

```json
{ "ok": false, "error": "http_404", "error_code": "E_TARGET_HTTP_4XX",
  "target_body": { "raw": "<!doctype html>…404 Not Found…" } }
```

That raw HTML is the target Odoo saying **the endpoint does not exist**. It almost always
means one thing: **`dyncloud_api` is not installed on that environment.**

Why it happens even on a correctly deployed environment: the module ships with
`auto_install`, which only fires on a **fresh** database. On a database that already
existed, Odoo requires a dependency to be in `'to install'` during that same operation — so
an environment that merely *upgraded* `dyncloud_auth` never marks `dyncloud_api`, and the
code sits in the addons path, uninstalled, forever.

**Confirm it in two calls:**

```text
list_databases      { environment_id: ... }     # served by dyncloud_auth
system_odoo_version { ...five args... }         # served by dyncloud_api
```

- Both work → not this. Look at §3 or the specific error.
- **First works, second 404s** → confirmed: `dyncloud_auth` is there, `dyncloud_api` is
  not.
- Both fail → the environment may be down or unreachable; check `state` in
  `list_environments`.

**The fix is installing the module on that environment**, which is not something this MCP
can do (the tool that would install it is itself proxied through the missing module). It is
a technical-team task. Report: environment, project, and that `dyncloud_api` is missing
while `dyncloud_auth` responds.

---

## 2. It timed out

**First: nothing was cancelled.** The environment kept working. Establish what actually
happened before touching anything (`modules.md` §5 for modules; `imports.md` §5 for
imports).

Then identify **which** timeout fired, because they have different owners:

| What you saw | Which layer | Who owns it |
| --- | --- | --- |
| Bare `The operation timed out.`, no `error_code` | **1 — your MCP client** | The user's own machine/config |
| `E_TARGET_TIMEOUT`, naming a limit and a field | **4 — the manager** | The MCP server record in DynCloud |
| Anything else slow but answering | 2/3/5/6 — platform | Technical team (devops) |

The full table is in `../README.md`. The short version for a report: **quote the literal
error**. Its shape is the diagnosis, and paraphrasing it destroys the only evidence.

---

## 3. A tool or an argument "does not exist"

The schema you are calling with does not match the server's. Almost always: **your client
cached the tool catalogue when the session started**, and the manager has been deployed
since.

**Restart your MCP client** (in Claude Code, restart the session; `claude --continue` keeps
the conversation) and read the tool schema again. In the session that produced this skill,
a new tool and a changed argument (`module_name` → `module_names`) were invisible until
restart.

Only after restarting is it worth suspecting anything else.

---

## 4. It worked yesterday and not today

Check in this order — cheapest first:

1. **The grant expired.** `get_my_mcp_access` and read **`effective_state`**, not `state`.
   A grant showing `state: "active"` can be `effective_state: "expired"`, and the
   enforcement uses the effective one.
2. **The environment changed state.** `list_environments` → `state`. Stopped or hibernated
   environments answer nothing.
3. **The environment was rebuilt** on a different release. `system_odoo_version` →
   `capabilities`. A feature that was there can be absent on a rebuild from an older pin.
4. **The client was restarted with a different configuration**, or the manager was
   redeployed (§3).

---

## 5. Hand the problem over

When it is beyond the MCP — a missing module, a platform timeout, a permission nobody can
grant — stop debugging and produce a report somebody can act on.

**Always include:** environment and project names, the **stage**, the **time**, the tool
you called, and the **literal error text and code**. If a grant is involved, its reference
and `effective_state`. The `jti` in `meta` is a short request identifier worth quoting —
it lets the technical team find the exact call in the logs.

**For a technical profile**, that plus your diagnosis is enough; they will take it from
there.

**For a consulting profile**, write it *for them to forward*, in their language, with no
instruction to investigate anything:

> "No he podido instalar el módulo en el entorno *main* de *acme-erp*. El entorno
> responde, pero le falta un componente que DynCloud necesita para operar sobre él, y eso
> solo lo puede instalar el equipo técnico.
>
> Para que lo resuelvan, mándales esto:
> **proyecto:** acme-erp · **entorno:** main (producción) · **hora:** 13:42 ·
> **qué falla:** las llamadas a `/dyncloud/api/v1/...` devuelven 404 · **detalle:**
> `dyncloud_auth` responde pero `dyncloud_api` no está instalado."

Never leave a consultant with a stack trace, and never leave them with "ha fallado" either.
The middle ground — what happened, why it is not their fault, and exactly what to send —
is the whole job.
