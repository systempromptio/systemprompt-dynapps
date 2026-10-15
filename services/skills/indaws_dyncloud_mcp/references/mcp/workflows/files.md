# Workflows — moving files in and out

Two directions, one principle: **the bytes never cross this conversation.** Tool details
and the reasoning behind the value-ref are in `../file-tools.md`; these are the sequences.

---

## 1. Attach a file to a record's chatter

```text
# 1. Upload URL (single use, expires)
file_upload_request { filename: "contract.pdf" }
  → { upload_url: "...", expires_minutes: 60 }

# 2. Move the bytes — your shell, not a tool
#    curl -sS -X POST -F "file=@contract.pdf" "<upload_url>"
#      -> {"ok": true, "attachment_id": 1265, "bytes": 21600}

# 3. Create the attachment on the customer's Odoo, by reference
orm_create {
  ...five args...,
  model: "ir.attachment",
  vals: { name: "contract.pdf",
          res_model: "res.partner", res_id: 9,
          mimetype: "application/pdf",
          datas: { "__ref__": "attachment", "id": 1265, "consume": true } }
}
  → ids: [44]
```

`res_model` + `res_id` are what put it in that record's chatter. Without them the
attachment exists but hangs off nothing, which looks to the user like it never arrived.

**No `"raw": true` here.** `ir.attachment.datas` is a normal base64 Binary field; the raw
envelope is only for `base_import.import.file` (`imports.md`).

**Verify by checksum, not by faith:**

```text
orm_search_read { ...five args..., model: "ir.attachment",
                  domain: [["id","=",44]],
                  fields: ["name","file_size","checksum","res_model","res_id"] }
```

`checksum` is the SHA-1 of the stored content. Compare it with `shasum` of your local file
and you know the transfer was exact — a stronger statement than "the call returned ok".

**Without a shell**, hand the `upload_url` to the user: opening it in a browser serves a
small upload form and the file goes straight to the manager. Then ask them for the
`attachment_id` it answers with, and continue from step 3.

---

## 2. Get a file out of the customer's Odoo

```text
orm_search_read { ...five args..., model: "ir.attachment",
                  domain: [["res_model","=","res.partner"], ["res_id","=",9]],
                  fields: ["name","datas"] }
  → datas: { download_url: "https://…/download/<token>",
             field: "datas", size_bytes: 21600, expires_minutes: 60 }
```

Then decide what the file is for:

- **The user wants it** → give them `download_url` and say it expires. It is served with a
  short-lived token; opening it saves the file.
- **You need to read it** → download to disk and open it with the file-reading tool:
  ```bash
  curl -sSL -o /tmp/contract.pdf "<download_url>"
  ```
  Never pipe the bytes into the conversation.

**A small file comes back as inline base64 instead of a descriptor.** That is the
threshold working as intended — below a few KB a round-trip costs more than the bytes. If
you expected a descriptor and got a string, the file was simply small.

---

## 3. Reuse one uploaded file for several writes

Omit `consume`. The staged file survives on the manager until its TTL expires, so the same
`attachment_id` can be referenced in several `orm_create` / `orm_write` calls — the same
document attached to five records, or the same spreadsheet imported into two environments.

```text
datas: { "__ref__": "attachment", "id": 1265 }              # first write, and the rest
datas: { "__ref__": "attachment", "id": 1265, "consume": true }   # the last one
```

Once consumed, referencing it again fails with *"Attachment reference is not accessible on
this server."* — the same message you get for an id that never existed or that belongs to
somebody else. That ambiguity is deliberate, so the error cannot be used to probe what
exists.

---

## 4. Large files, and what "large" means here

Two different limits, and they fail differently:

- **The upload to the manager** is bounded by the MCP server's own cap. Exceeding it fails
  at the `curl`, before Odoo is involved.
- **The injection into the target** is bounded by the environment's request-body cap. K8s
  environments run 50 MB; a Docker environment that has not been configured runs the
  module's 4 MiB default and answers **413**.

If a legitimate file hits 413, that is a per-environment setting the technical team can
raise. Say the size and the environment when you report it — those two facts are the whole
ticket.

For anything genuinely large, prefer splitting the work (import in batches) over asking for
a higher cap: a 200 MB import that times out mid-flight is a worse problem than four 50 MB
ones that each finish.
