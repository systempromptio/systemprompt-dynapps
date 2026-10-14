# DynCloud MCP — files

Moving a file to or from a customer's Odoo **without its bytes ever entering this
conversation**. That is the whole design: a spreadsheet of 20.000 orders would otherwise
cost more context than the task it serves.

Two directions, two mechanisms:

- **In:** upload the file to the **manager**, then reference it by id inside `vals`. The
  proxy resolves the reference into content on its way out to the target.
- **Out:** a binary field over the inline threshold comes back as a **descriptor with a
  `download_url`**, not as base64.

---

## file_upload_request

Issues a one-time, short-lived upload URL. Category `data_write`.

```json
{ "filename": "contacts.xlsx" }
```
```json
{ "upload_url": "https://<manager>/mcp/odoo/<slug>/upload/<token>",
  "expires_minutes": 60,
  "instructions": "POST the file to this one-time URL (multipart field 'file')…" }
```

**The file lands on the MANAGER, not on the customer's Odoo.** It is a staging area.
Nothing reaches the target until you reference it in a write.

Then move the bytes yourself — no tool of yours does it:

```bash
curl -sS -X POST -F "file=@contacts.xlsx" "<upload_url>"
#  -> {"ok": true, "attachment_id": 1270, "bytes": 5226}
```

In **Windows PowerShell**, `curl` is an alias for `Invoke-WebRequest`; call `curl.exe`
explicitly or use the native form. **Without a shell**, hand the URL to the user: opening
it serves a small upload form, and the file goes straight to the manager.

The URL is **single use** and expires (`expires_minutes`, configurable per MCP server).
Retrying a spent one fails forever — ask for another.

---

## The value-ref

Read `attachment_id` from the POST answer and use it as the value of a binary field:

```json
{ "model": "ir.attachment",
  "vals": { "name": "contract.pdf", "res_model": "res.partner", "res_id": 9,
            "mimetype": "application/pdf",
            "datas": { "__ref__": "attachment", "id": 1270, "consume": true } } }
```

The proxy replaces that object with the real content **right before the request leaves the
manager** for the target. The bytes cross the manager→target hop, which has no context
budget, instead of crossing this conversation, which does.

Three keys:

| Key | Meaning |
| --- | --- |
| `id` | The `attachment_id` from the upload |
| `consume` | `true` deletes it from the manager the instant it is injected, instead of waiting out its TTL. Use it whenever the file is for one write only. |
| `raw` | See below. Almost always **omit it**. |

If you reference an id that no longer exists — already consumed, or swept by TTL — the
write fails with *"Attachment reference is not accessible on this server."* That is also
what a reference to somebody else's file looks like, deliberately: the message does not
distinguish them, so an id cannot be used to probe what exists.

---

## `"raw": true` — one field needs it, the rest break with it

Almost every Odoo `Binary` field follows the legacy convention: you hand the ORM base64 and
it stores the decoded bytes. `ir.attachment.datas` works exactly that way, and the plain
value-ref above is right for it.

**`base_import.import.file` breaks that convention.** Its own help says *"raw binary (not
base64)"*: Odoo fills it through a multipart controller, not through the ORM, and the
importer hands the column straight to the XLSX reader. Feed it base64 and **nothing
raises** — the base64 *text* is stored, and the import dies much later with an opaque
*"Unable to read file"*.

JSON cannot carry arbitrary bytes, so the manager cannot solve this alone. `"raw": true`
wraps the content in an envelope that **`dyncloud_api` decodes into real bytes on the
target**, immediately before the write:

```json
{ "model": "base_import.import",
  "vals": { "res_model": "res.partner",
            "file_name": "contacts.xlsx",
            "file_type": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "file": { "__ref__": "attachment", "id": 1270, "raw": true, "consume": true } } }
```

**Set it only for a field documented as raw.** On an ordinary binary field it stores the
content wrong — the mirror image of the bug it fixes.

It needs the target to advertise **`raw_binary_write`** in `system_odoo_version`
→ `capabilities`. Against an older environment the write fails with
`E_TARGET_LACKS_RAW_BINARY`, which says the environment must be rebuilt on a newer release.
It **fails loudly rather than corrupting data** — that is the intended behaviour, not a
regression.

---

## Coming back: the download descriptor

Read a binary field and, above the inline threshold, you do not get base64:

```json
{ "id": 44, "name": "contract.pdf",
  "datas": { "download_url": "https://<manager>/mcp/odoo/<slug>/download/<token>",
             "field": "datas", "size_bytes": 21600, "expires_minutes": 60 } }
```

- **Give `download_url` to the user directly** when they want the file. It is served with
  a short-lived token.
- **To read what is inside** — summarise a PDF, look at a screenshot — download it to disk
  and open it with the file-reading tool. Never paste the bytes into the conversation.
- `size_bytes` is the **real** size of the file.
- **Never try to reconstruct the file from the response.** There is nothing to
  reconstruct; the descriptor is the answer.

```bash
curl -sSL -o /tmp/contract.pdf "<download_url>"   # then read /tmp/contract.pdf
```

Small binaries still come back inline as base64 — the threshold exists so a 400-byte icon
does not need a round-trip. If you expected a descriptor and got a string, the file was
simply below it.

---

## TTL

One TTL governs both directions and is a field on the MCP server record in the manager
(*File TTL (minutes)*, 60 by default). A background job sweeps expired files. A consultant
lives both halves of a transfer in one conversation, so one deadline for both is the honest
model — and `consume` is how you avoid waiting for it at all.
