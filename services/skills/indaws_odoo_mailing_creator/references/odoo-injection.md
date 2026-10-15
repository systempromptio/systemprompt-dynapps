# Odoo 19 injection & test-send procedure

How Odoo 19 Mass Mailing treats the HTML, and the exact MCP procedure to upload a mailing and send a real test. Verified against `runtime/v19/odoo-src`.

## Table of contents
- [1. What Odoo does to your HTML](#1-what-odoo-does-to-your-html)
- [2. Why a body-only fragment](#2-why-a-body-only-fragment)
- [3. Upload procedure (idempotent)](#3-upload-procedure-idempotent)
- [4. Verification after upload](#4-verification-after-upload)
- [5. Real test-send procedure (mail.mail)](#5-real-test-send-procedure-mailmail)
- [6. Hard rules](#6-hard-rules)

---

## 1. What Odoo does to your HTML

- `mailing.mailing` has two body fields (`mass_mailing/models/mailing.py:118`, `Mailing.body_arch`, y `mass_mailing/models/mailing.py:119-121`, `Mailing.body_html`):
  - `body_arch` — editor source (`sanitize='email_outgoing'`).
  - `body_html` — **what is actually sent** (`mass_mailing/models/mailing.py:1118`, `Mailing._action_send_mail`), `render_engine='qweb'`.
  - **No server-side sync** between them (the OWL editor syncs in the client). → Write BOTH with the same HTML.
- `sanitize='email_outgoing'` is **deliberately lax** (`odoo/orm/fields_textual.py:584-591`, `Html._get_attrs`, v19; en v18 el mismo atajo vive en `odoo/fields.py:2165-2180`, `Html._get_attrs`): `sanitize_tags=False`, `sanitize_attributes=False`, `sanitize_conditional_comments=False`. It **keeps** `<style>`, `@media`, `@font-face`, `[if mso]` comments, all attributes and classes. It only strips `<form>` and bare `<o:...>` Office tags, and normalizes full documents to fragments.
- **Odoo does NOT inline CSS** (no premailer). Your CSS must already be inline (juice does this).
- At send time Odoo wraps your `body_html` inside `mass_mailing.mass_mailing_mail_layout` (`mass_mailing/data/mailing_data_templates.xml:8-36`, `mass_mailing_mail_layout`): it adds its own `<head>` (metas + `OfficeDocumentSettings`) and injects the `mass_mailing_mail.scss` `<style>`. That SCSS has `.o_layout :not(.fa){font-family:Arial}` **without `!important`** (`mass_mailing/static/src/scss/mass_mailing_mail.scss:5-7`, `.o_layout`) → inline font-family wins, so no override is needed.
- Send-time link tracking rewrites `<a href>` (UTM/click tracking), whitelist `/unsubscribe_from_list`, `/view`, `/cards/`. Content/structure untouched.
- `{{ }}` placeholders are consumed by the qweb/inline-template render at send time → they vanish. Plain UPPERCASE words (`CUSTOMER_NAME`) survive verbatim.

## 2. Why a body-only fragment

Because Odoo supplies the `<head>` itself, inject **only the inner `<body>`** wrapped in `<div class="o_layout o_basic_theme">`. `prepare_odoo.cjs` does exactly this: folds the head `<style>` into the body (Odoo keeps it), drops the duplicate `OfficeDocumentSettings`, keeps the rest of the `[if mso]` comments. The `o_layout` wrapper activates Odoo's margin reset (`html:has(.o_layout){margin:0;padding:0}`) which kills the top white strip.

## 3. Upload procedure (idempotent)

> **Method choice (important).** The MCP `orm_create` **fails with `-32000`** for large/binary payloads (verified: even an empty `ir.attachment` create fails; image hosting is impossible via MCP). And the harness can corrupt large `Read` outputs, so transcribing base64/HTML by hand is unreliable. **For attachments (PNG icons) and large bodies, use the `scripts/odoo_push.cjs` script** (JSON-RPC, reads files from disk — nothing large passes through the model). It needs credentials in env (`ODOO_URL`, `ODOO_DB`, `ODOO_USERNAME`, `ODOO_API_KEY`); keep them in a gitignored `.env.local`. The MCP path below is fine only for small reads/writes.

### Robust path — `odoo_push.cjs` (recommended for real uploads)
`odoo_push.cjs` auto-loads `.env.local` from the skill directory and resolves paths via `__dirname`, so it
runs from any cwd (no `cd`/`source`). Point the script path and the file args wherever they live.
```bash
# 1) host each PNG icon (prints {id, checksum, url}); verify checksum == local sha1
node scripts/odoo_push.cjs attach <png> "<name>.png"
# 2) substitute the /web/image/<id> URLs into the MJML, then compile/prepare/validate
# 3) create a NEW mailing from a structural template, body read from disk
node scripts/odoo_push.cjs create-from-template <tmplId> "<subject>" forge/dist/<slug>.odoo.html
# 4) (re)write the body later if needed
node scripts/odoo_push.cjs write-body <mailingId> forge/dist/<slug>.odoo.html
# 5) send a real test (after confirming the recipient)
node scripts/odoo_push.cjs send-test <mailingId> "<email>" [--server=N]
```
Verify public image access before sending: `curl -s -o /dev/null -w "%{http_code} %{content_type}" <ODOO_URL>/web/image/<id>` must be `200 image/png`. CRITICAL: substitute the REAL attachment ids returned by step 1 into the HTML — never assume/guess ids. Re-read the mailing afterwards to confirm the URLs match.

### MCP path (small payloads only)
Use the dedicated MCP write tool. Let `FRAG` = contents of `forge/dist/<slug>.odoo.html`.

1. **Find or pick the record.** If the user gave a record id, use it. Otherwise search by subject:
   ```
   mcp__odoo-indaws__orm_search_read
     model: "mailing.mailing"
     domain: [["subject", "=", "<SUBJECT>"]]
     fields: ["id", "subject"]
   ```
   Update if found; otherwise `orm_create` a `mailing.mailing` with at least `subject` (and `mailing_model_id`/`contact_list_ids` as the user requires).

2. **Write both body fields** (single call):
   ```
   mcp__odoo-indaws__orm_write
     model: "mailing.mailing"
     ids: [<ID>]
     values: { "body_html": <FRAG>, "body_arch": <FRAG> }
   ```
   Returns `{"ok": true}`. **Never** use `orm_execute_kw method=write` with large payloads — it returns `-32000`.

## 4. Verification after upload

Re-read and confirm the HTML survived the write-time sanitize:
```
mcp__odoo-indaws__orm_search_read
  model: "mailing.mailing"
  domain: [["id", "=", <ID>]]
  fields: ["id", "subject", "body_html"]
```
Check in the returned `body_html`: contains `o_layout`, `@media`, and `[if mso]`, and the byte length is close to `FRAG`. Report to the user: "Mailing #<ID> '<subject>' updated as draft, <N> KB." **Do not reopen the visual editor** afterwards (it regenerates body_html from body_arch via OWL and can alter tables/MSO).

## 5. Real test-send procedure (mail.mail)

This is the reliable recipe (state → sent). Confirm with the user first (GATE).

1. **Create the mail** (small fields; body is large but create works):
   ```
   mcp__odoo-indaws__orm_create
     model: "mail.mail"
     values: {
       "subject": "[TEST] <SUBJECT>",
       "body_html": <body_html of the mailing>,
       "email_to": "<TEST_ADDRESS>",
       "mail_server_id": <SERVER_ID>     // e.g. the configured Brevo server
     }
   ```
   The result is a string like `mail.mail(NNNN,)`. **Read NNNN** — never guess; do not assume it is the previous id + 1.

2. **Send it** (process the queue for that id):
   ```
   mcp__odoo-indaws__orm_execute_kw
     model: "mail.mail"
     method: "process_email_queue"
     args: [[NNNN]]
   ```

3. **Confirm** state:
   ```
   mcp__odoo-indaws__orm_search_read
     model: "mail.mail"
     domain: [["id", "=", NNNN]]
     fields: ["id", "state", "email_to"]
   ```
   Expect `state: "sent"`. Tell the user it was sent and to check that inbox.

## 6. Hard rules
- Write **both** `body_html` and `body_arch`, same HTML, via `orm_write`.
- **Read the real `mail.mail` id**; never guess.
- Confirmation gate before upload AND before test send.
- Mass send is OUT OF SCOPE — tests only.
- Do not reopen the Odoo visual editor after injecting (it regenerates the HTML).
