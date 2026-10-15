# Validation Harness (multi-principal, over HTTP)

An exposure is a security surface, so the only convincing test is **adversarial,
over the wire, with more than one identity**. Unit tests (odoo-test-generator
pattern 22) prove the model-layer logic; this harness proves the *deployed
server* holds — including the auth, session and dispatch layers a unit test never
exercises. Run it before every handoff.

> **The one rule:** never validate an exposure with only `admin`. Admin reads
> everything, so "a restricted user sees X it shouldn't" is *structurally
> invisible*. Always drive it with **≥2 principals** (an admin and a restricted
> user) and assert the **difference**. The single most common real bug is a server
> that looks perfect under one admin login and leaks under a restricted user.

## What to assert (the check matrix)

Four families of checks. The bundled `scripts/mcp_probe.py` covers the
load-bearing ones; the exhaustive 36-check suite is
`staging-local-18/scripts/pentest_runner.py` (sections 11–14), which this matrix
mirrors.

| Family | Asserts | Example |
|--------|---------|---------|
| **Gate** | An unexposed model/field/method is refused | `describe_model("res.users")` on a sales server → error |
| **Adversarial** | Caller-supplied input can't widen access | `aggregate` with a domain on a non-exposed field, or dotted traversal `partner_id.credit_limit` → rejected |
| **Base toolset** | The self-sufficiency tools work for a legit caller | `whoami`, `list_models`, `get_selection`, `aggregate(group_by=state)` succeed |
| **Files** (if exposed) | Binaries never leak inline; the pre-signed path works | a `search_read` of a binary field returns a value-ref, **not** base64; `attachment_link` yields a working short-TTL URL for an exposed record and is **refused** for one the caller's ACL denies |
| **Privilege isolation** | A restricted principal gets strictly less than admin — *including metadata* | a model the restricted user's ACL denies is refused **and** absent from `list_models` / `describe_model` / `capabilities` |

The privilege-isolation family is the point of the whole exercise: prove
`restricted ⊊ admin`, and that metadata tools never *name* a model the restricted
user can't read (metadata is data).

## Provisioning recipe

You need, on the target DB: the server under test, a **restricted user** in the
server's company, and an OAuth token per principal. Crucially, give the isolation
check its **own dedicated server** that exposes a sensitive model
(`ir.config_parameter`) — don't pollute the clean business server with it, or the
gate/adversarial checks get false failures. Per global rules, do live Odoo work
through the `odoo-pilot` scripts; for token minting and bulk record setup an
`odoo shell` block is the pragmatic path:

```python
# via:  docker compose run --rm --no-deps -T odoo bash -c "echo '<this>' | odoo shell -d <DB> --no-http -c /etc/odoo/odoo.conf"
u   = env.ref("base.user_admin"); co = u.company_id
# restricted principal — internal user, in the server's company
ltd = env["res.users"].search([("login","=","mcp-ltd")], limit=1) or env["res.users"].create(
    {"name":"MCP Ltd","login":"mcp-ltd",
     "group_ids":[(6,0,[env.ref("base.group_user").id])]})   # v19: group_ids; v16-18: groups_id
ltd.write({"company_ids":[(6,0,[co.id])],"company_id":co.id})
# dedicated ISOLATION server exposing a sensitive model the restricted user can't read
iso = env["mcp.server.config"].search([("name","=","ISO")],limit=1) or env["mcp.server.config"].create(
    {"name":"ISO","company_id":co.id,"enable_tools":True,"core_tools":True,
     "user_ids":[(6,0,[u.id, ltd.id])]})
icp = env["mcp.model"].search([("model_name","=","ir.config_parameter")],limit=1) \
      or env["mcp.model"]._ensure_catalog("ir.config_parameter")
if not env["mcp.server.model"].search([("server_id","=",iso.id),("model_id","=",icp.id)],limit=1):
    env["mcp.server.model"].create({"server_id":iso.id,"model_id":icp.id,"allow_read":True})
# one token per principal (scope=odoo)
a,_ = env["oauth.provider.token"].sudo().create_token_pair(u,  co,"probe-a",scope="odoo",ttl_hours=2,refresh_ttl_days=7)
b,_ = env["oauth.provider.token"].sudo().create_token_pair(ltd,co,"probe-l",scope="odoo",ttl_hours=2,refresh_ttl_days=7)
env.cr.commit(); print("ADMIN="+a); print("LTD="+b); print("ISO="+iso.slug)
```

Gotchas learned the hard way:
- The restricted user must be in the **server's company**, or it gets "MCP not
  configured for this company" instead of a clean permission result.
- v19 creates users with `group_ids`; v16–v18 with `groups_id` (see
  `common-skills:odoo-version-api-reference` v19 diff).
- Keep the business server clean — sensitive-model exposure goes on the dedicated
  ISO server, or §gate/§adversarial checks fail for the wrong reason.
- A probe field assumed "not exposed" must really be unexposed on *that* server;
  pre-seeded demo exposures sometimes already include it (pick a field you control).

## Running it

Bring the server up on a port (for a docker stack: a backgrounded `odoo` with the
module mounted), then:

```bash
MCP_BASE=http://localhost:<port> MCP_DB=<db> SALES_SLUG=<business-slug> \
ADMIN_TOKEN=<a> RESTRICTED_TOKEN=<b> PROBE_DENIED_MODEL=ir.config_parameter \
python3 scripts/mcp_probe.py
```

`mcp_probe.py` initializes a session per principal and runs the load-bearing
isolation assertions (restricted gets the toolset; the denied model is refused
*and* invisible in metadata; admin is not spuriously blocked), printing PASS/FAIL
and exiting non-zero on any failure. For full coverage (all four families, ~36
checks, adversarial domain/measure/traversal cases) point
`staging-local-18/scripts/pentest_runner.py` at the same env vars
(`SALES_SLUG`/`SECOND_SLUG`/`ISOLATION_SLUG`/`MCP_ACCESS_TOKEN`/`RESTRICTED_TOKEN`/
`PROBE_*`).

## Pass bar for handoff

- Restricted principal: base toolset works; **every** model outside its ACL is
  refused and unnamed in all metadata surfaces.
- Adversarial: no domain/measure/groupby on a non-exposed field, no dotted
  traversal, gets through.
- Admin: not spuriously blocked on what it should reach.

Green here + the unit suite (pattern 22) + an audit pass
(`odoo-code-auditor` → `mcp-exposure-security.md`) = shippable.
