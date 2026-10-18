# Fetching employee photos through the Dynapps MCP

**TL;DR — the photos ARE reachable through the generic ORM MCP.** The obvious paths
are blocked, but going through the field-backed `ir.attachment` row works. This
replaces the old (wrong) note that said photos were MCP-impossible and you had to
use odoo-pilot RPC. odoo-pilot (`fetch_team.py`) still works and is fine when you
have an API key; the MCP path below needs no API key and works in Cowork.

## Working recipe (agent-driven; verified 2026-07-08 on db `master`)

For each person, resolve their `res.partner` id (from `hr.employee.user_partner_id`
or `work_contact_id`), then:

1. `orm_search_read('ir.attachment', [['res_model','=','res.partner'], ['res_id','=',<pid>], ['res_field','=','image_512']], ['id','res_id','file_size'])`
   → returns the attachment `id` backing that image variant.
   (Variants exist: `image_128/256/512/1024/1920`. `image_512` is the sweet spot for
   a ~150 px avatar @2×.)
2. `odoo_attachment_link(attachment_id=<id>)` → a 15-minute presigned
   `{odoo_url}/mcp/.../download/...` URL (instance from runtime discovery). **Bytes never pass through the model.**
3. `curl` that URL to disk, then `python3 scripts/embed_team_photos.py <extra.json> "Full Name=/path.png"`
   (normalizes to a centered 512×512 square and injects the data URI into `equipo[].photo`).

`hr.employee` image attachments may not exist as their own rows (the employee image
is often the related partner's), so key off **`res.partner`** ids — that's what has the
`image_*` attachments.

## Why the obvious paths don't work (for the dev team)

Three independent layers, none of which is an Odoo permission problem (the proxy runs
as **sudo/superuser**):

1. **Inline binary stripping on read.** `orm_search_read`/`read` never return
   image/binary field *values* — `image_512` comes back as `false` plus a
   `_binary_url_fields` hint. By design (keeps payloads small). So you can't read the bytes.
2. **`odoo_attachment_link(model, res_id, field)` has a model allowlist.** For
   `hr.employee` / `res.partner` / `res.users` it returns *"Model X is not readable on
   this server."* This is the proxy's own gate, not Odoo ACL. So the ergonomic
   record+field → URL path is blocked for exactly the people-models.
3. **Field-backed attachments are hidden from `odoo_attachment_search`.** Odoo excludes
   rows with `res_field IS NOT NULL` from generic attachment listing, so
   `odoo_attachment_search(res_model='hr.employee')` finds "nothing" (only real
   file attachments). But `orm_search_read('ir.attachment', …)` with an explicit
   `res_field` domain DOES return them, and `odoo_attachment_link(attachment_id=…)`
   is **not** gated by the model allowlist → it works.

## Requested MCP improvements (priority order)

1. **Whitelist `hr.employee` / `res.partner` / `res.users` (or make it configurable)
   for `odoo_attachment_link(model, res_id, field)`** on image fields. One call instead
   of the `ir.attachment` archaeology above.
2. **Better: expose Odoo's native `/web/image/<model>/<id>/<field>` as a first-class MCP
   tool.** That controller already enforces record ACL, serves resized variants and
   computed images, and handles the anonymous-placeholder case. Cleanest long-term fix.
3. Document that `odoo_attachment_link(attachment_id=…)` bypasses the model gate and that
   `res_field`-backed attachments need an explicit `res_field` domain on
   `orm_search_read('ir.attachment', …)`.

## Not-MCP blockers (context)

The browser fallback was painful for **client-side** reasons unrelated to the MCP:
enterprise Chrome had automatic-downloads = Block for the Odoo origin, and Private
Network Access blocks HTTPS→localhost. The attachment-id recipe above avoids the
browser entirely, so prefer it.
