# Workflows — the standard Odoo import (Excel / CSV)

The consulting workhorse: a customer sends a spreadsheet of contacts, products or opening
balances and it has to end up in Odoo. This runs **Odoo's own importer**, the same one
behind *Import records* in the UI — so validations, `many2one` resolution by name and
error messages are exactly the ones the customer would get.

The spreadsheet **never enters this conversation**. It goes to the manager and is injected
into the target by reference (`../file-tools.md`).

---

## 1. End to end

```text
# 1. Check the target can take a raw binary write
system_odoo_version { ...five args... }
  → capabilities: ["raw_binary_write"]        # required; if absent, stop and see §5

# 2. Get a one-time upload URL (on the MANAGER, not on the customer's Odoo)
file_upload_request { filename: "contacts.xlsx" }
  → { upload_url: "...", expires_minutes: 60 }

# 3. Move the bytes yourself
#    curl -sS -X POST -F "file=@contacts.xlsx" "<upload_url>"
#      -> {"ok": true, "attachment_id": 1270, "bytes": 5226}

# 4. Create the import wizard on the target, referencing the file
orm_create {
  ...five args...,
  model: "base_import.import",
  vals: {
    res_model: "res.partner",
    file_name: "contacts.xlsx",
    file_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    file: { "__ref__": "attachment", "id": 1270, "raw": true, "consume": true }
  }
}
  → ids: [4]

# 5. Dry run
orm_call_method {
  ...five args...,
  model: "base_import.import", method_name: "execute_import",
  args: [[4],
         ["name","email","phone","city","function"],
         ["Name","Email","Phone","City","Job Position"],
         { has_headers: true, quoting: "\"", separator: ",", encoding: "utf-8",
           date_format: "", datetime_format: "",
           float_thousand_separator: ",", float_decimal_separator: ".",
           advanced: false, keep_matches: false,
           name_create_enabled_fields: {}, import_set_empty_fields: [],
           import_skip_records: [], fallback_values: {},
           skip: 0, limit: 2000 },
         true]                                  # ← dryrun
}
  → { ids: [20,21,22,23,24], messages: [], name: ["Aurora Belmonte", "Íñigo Etxeberria", …] }

# 6. Same call with dryrun = false
```

`"raw": true` is **mandatory** here and wrong almost everywhere else — the reasoning is in
`../file-tools.md`. Without it the field silently receives base64 *text* and the import
fails later with *"Unable to read file"*.

`consume: true` deletes the staged file from the manager the moment it is injected. Use it
unless you plan to import the same file into more than one environment.

---

## 2. Why `parse_preview` is not the way here

The UI calls `parse_preview` to show you the columns and guess the mapping. Through the MCP
it is **unusable**: it answers with the target model's **entire field tree** — 236 KB for
`res.partner` — and the proxy truncates large responses well below that, so you get a
fragment with the useful part cut off.

Go straight to `execute_import` with an explicit mapping. You are not losing much: the
mapping is the part you should be deciding deliberately anyway.

If you genuinely need to see the columns first, read them from the file **before**
uploading it — you have it locally, and a couple of lines of shell tell you the header row
without a round-trip.

---

## 3. Mapping columns to fields

`execute_import` takes two parallel arrays:

- **`fields`** — the Odoo technical field names, in file-column order.
- **`columns`** — the header labels as they appear in the file.

They must be the same length and in the same order. Use `""` in `fields` to skip a column.

For relational fields, Odoo's suffixes work as in the UI:

| Suffix | Meaning |
| --- | --- |
| `partner_id` | Resolve by display name |
| `partner_id/id` | Resolve by external ID (XML id) |
| `partner_id/.id` | Resolve by database id |

`name_create_enabled_fields: {"partner_id": true}` lets the importer **create** missing
related records instead of failing. Powerful and dangerous: on a typo-ridden file it
invents partners. Leave it off unless the customer asked for exactly that.

---

## 4. Always dry-run first

The last positional argument is `dryrun`. `true` runs the whole import inside a transaction
and rolls it back, returning **the same messages the real run would produce**.

Read two things in the answer:

- **`messages`** — empty means clean. Anything in it is a row-level problem: unresolvable
  relation, bad date format, missing required field.
- **`ids`** — how many records it would touch. Compare it with the rows in the file; a
  mismatch means rows were skipped and `messages` says why.

The ids from a dry run are **not** the ids of the real run. Do not report them as created.

---

## 5. When it fails

| Symptom | What it is |
| --- | --- |
| `"Unable to read file"` | The file arrived as base64 text — `"raw": true` was missing, or the target lacks `raw_binary_write` |
| `E_TARGET_LACKS_RAW_BINARY` | The environment predates the raw-binary support. It must be rebuilt on a newer release — a technical-team task |
| `413` / body too large | The file exceeds the target's request cap. Split the file, or ask the technical team to raise it |
| `messages` full of relation errors | The mapping is wrong, or the related records do not exist yet — import those first |
| Encoding mess in names (`Ã±`) | `encoding` in the options does not match the file. UTF-8 is the safe default for XLSX |
| Timed out | The import may still be running (`../README.md` → timeouts). **Read the target before retrying**, or you import twice |

That last one deserves care: a re-run of a partially applied import duplicates records.
Count the model before and after (`orm_search_count`) rather than assuming.
