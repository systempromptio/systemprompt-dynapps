# DynCloud MCP — workflows (index)

A single call rarely finishes a real task. These are the chains, with the actual calls and
the reasoning behind the order. Tool detail is in `../navigation-tools.md`,
`../read-tools.md`, `../write-tools.md`, `../admin-tools.md`, `../file-tools.md` and
`../access-jit.md`; shared conventions in `../README.md`.

Golden thread: **locate → check → act → verify.** Locate the environment, check what you
are about to touch (count it, read its state), act, then read back what happened — because
several of these tools report success for the part they did and stay silent about the rest.

**This page exists so you can choose without opening anything.** Read the line, open the
one file you need.

## `discovery.md` — find the environment, and prove it answers

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | From nothing to the five location args | You were given a customer name and little else |
| 2 | Prove the environment is usable before working on it | Before any batch of calls, or when something failed once |
| 3 | "This project/environment does not appear" | The user swears it exists and the list disagrees |
| 4 | Work on several environments in one session | Comparing, or repeating a change across environments |

## `reading.md` — read data and configuration

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Answer a question about the customer's data | "How many…", "which…", "does X exist" |
| 2 | Read a record and its related records | You need one thing and what hangs off it |
| 3 | Review a company's configuration | Auditing settings, journals, taxes, warehouses |
| 4 | "The client says X does not appear" | Something is invisible to them and you must say why |
| 5 | Aggregate without pulling the data | Totals, counts by group, big tables |

## `writing.md` — change data

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Create a record safely | Adding a contact, a product, a journal |
| 2 | Change many records at once | A mass update — **count before you write** |
| 3 | Archive instead of deleting | They asked to "remove" something |
| 4 | Turn an Odoo option on or off | `res.config.settings` — never write the parameter directly |
| 5 | Run a business action | Confirm an order, validate a picking, call a custom method |

## `imports.md` — the standard Odoo import (Excel / CSV)

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Import a spreadsheet end to end | The user has an XLSX/CSV of contacts, products, orders |
| 2 | Why `parse_preview` is not the way here | You were about to call it, as the UI does |
| 3 | Map columns to fields explicitly | Building the `fields` / `columns` arrays |
| 4 | Dry-run, then commit | Always — it is one extra call and it catches everything |
| 5 | When the import fails | Messages, partial imports, "Unable to read file" |

## `files.md` — moving files in and out

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Attach a file to a record's chatter | Sending a contract, an image, a PDF to Odoo |
| 2 | Get a file out of the customer's Odoo | They want a document, or you must read it |
| 3 | Reuse one uploaded file for several writes | The same file goes to more than one record |
| 4 | Large files, and what "large" means here | Something bigger than a few MB |

## `modules.md` — install, upgrade, uninstall

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Install one or more modules | Enabling functionality on a customer |
| 2 | Upgrade modules | Applying a new version of custom or standard code |
| 3 | Uninstall — and say what it destroys | **Read this before, not after** |
| 4 | A module that "does not exist" | `e_module_not_found` on something you can see in the repo |
| 5 | It timed out — now what | The call died and you do not know what happened |

## `access.md` — permissions

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | A call was denied — what to do about it | Any `E_…GRANT` / `E_…FORBIDDEN` |
| 2 | Ask for access and use it | You know you need a category up front |
| 3 | The grant is pending | You are not an approver and it is waiting |
| 4 | See real values behind the masking | `hash-…` or `+34 ***…` where you need the actual data |

## `troubleshooting.md` — when something does not add up

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | **404 on every ORM call** | The most common failure of all |
| 2 | It timed out | And you must know whether anything happened |
| 3 | A tool or argument "does not exist" | The schema disagrees with the documentation |
| 4 | It worked yesterday and not today | Same call, same environment, different outcome |
| 5 | Hand the problem to the technical team | You are consulting, or it is beyond the MCP |

## Composition cheatsheet

| Goal | Chain |
| --- | --- |
| Locate an environment | `list_my_companies` → `list_projects` → `list_environments` → `list_databases` → `list_target_users` |
| Prove it is alive | `list_databases` (cheapest, no grant) → `system_odoo_version` |
| Read data | `orm_search_read` (one round-trip; not `orm_search` + `orm_read`) |
| Size a change before making it | `orm_search_count` → then write |
| Mass update | `orm_search` (ids) → `orm_search_count` (size) → say it → `orm_write` |
| Remove something | prefer `orm_write {"active": false}` over `orm_unlink` |
| Turn on an Odoo setting | `orm_create` on `res.config.settings` → `orm_call_method` `execute` |
| Import a spreadsheet | `file_upload_request` → POST → `orm_create` `base_import.import` with `"raw": true` → `execute_import` dry-run → `execute_import` |
| Attach a file | `file_upload_request` → POST → `orm_create` `ir.attachment` with a value-ref + `consume` |
| Get a file out | `orm_search_read` the binary field → descriptor → `curl` the `download_url` |
| Install modules | `system_list_modules` → `admin_install_module {module_names: [...]}` → `system_list_modules` |
| Uninstall | `system_list_modules` → **say what dies** → `admin_uninstall_module` → `system_list_modules` (dependants) |
| Denied call | read `error_code` → `request_hint` → `request_mcp_access` → retry once |
| Check a grant | `get_my_mcp_access` → read **`effective_state`**, not `state` |
| After a manager deploy | restart the MCP client, then re-read the tool schemas |
