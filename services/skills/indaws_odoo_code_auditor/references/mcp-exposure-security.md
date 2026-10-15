# MCP / API Exposure Security — Detection Guide

Audit checklist for any module that **exposes Odoo to an external agent** over a
generic surface: MCP servers (`mcp.server.config` + auto-CRUD / methods / custom
tools), JSON-RPC bridges, OpenAPI endpoints, or any tool that lets a caller pick
the model / fields / domain at runtime. These surfaces have a failure mode plain
modules don't: the caller chooses what to touch, so the access model must hold
under *adversarial* input, not just intended use.

> **Building one, not auditing?** Use the `odoo-mcp-builder` skill — it bakes
> these rules in by construction (domain playbooks + least-privilege defaults).
> This guide is the audit counterpart; the multi-principal test counterpart is
> `odoo-test-generator` pattern 22.

## The invariant (the one rule)

> An exposure never grants more than **`admin positive-list ∩ caller ir.model.access ∩ ir.rule`**.
> **Metadata is data** (model names, field names, schemas, selection values count).
> Reads run **as the caller** (never `sudo()` / `su=True`).

If a finding can be framed as "this breaks the intersection" or "this leaks
metadata about something the caller can't read", it's a real issue.

Two root causes produce ~all bugs here:
1. **Conflating "exposed" with "readable".** The admin positive-list (what the
   server exposes) is NOT the user's ACL. Code that checks only the exposure,
   or runs `su`, grants more than the user has.
2. **Testing with a single all-powerful identity (admin).** Privilege leaks are
   structurally invisible to one omnipotent principal — see `multi-principal` below.

## Detection greps

```bash
# Reads/tools that elevate — core reads must run as the user
grep -rn 'env(su=True)\|sudo()' --include="*.py" .            # justify each in exposure paths

# User-supplied query parts reaching the ORM unvalidated
grep -rn '_read_group\|\.read_group(\|\.search(\|name_search' --include="*.py" .
grep -rn "args\.get(\"domain\")\|params\.get(\"domain\")\|\"group_by\"\|\"measures\"\|\"fields\"" --include="*.py" .

# Metadata surfaces — must be ACL-filtered, not just exposure-filtered
grep -rn 'fields_get\|list_models\|describe\|capabilities\|model_ids' --include="*.py" .

# check_access_rights / check_access — presence is a good sign in exposure code
grep -rn 'check_access_rights\|check_access\|has_access' --include="*.py" .
```

## Checklist (per exposure path)

| # | Check | FAIL signal |
|---|-------|-------------|
| 1 | **Exposure ≠ ACL.** Every model-touching tool re-checks the *caller's* ACL (runs non-`su`, or explicit `check_access_rights("read")`). | A tool gated only by the positive-list (`allow_read` flag) and/or running `su`. |
| 2 | **Metadata is ACL-filtered.** Tools that return model names / schemas / field lists / counts (`list_models`, `describe_model`, `get_selection`, `capabilities`) filter by the caller's ACL too. | A model the caller can't read still appears (its name/schema leaks as "harmless" metadata). |
| 3 | **Query parts validated against the positive-list.** `group_by`, `measures`, `fields` AND the **`domain`** are checked against the exposed-field set. | `domain` passed raw to `read_group`/`search` → caller filters/infers on non-exposed fields. |
| 4 | **No relation traversal past the list.** A dotted path (`partner_id.credit_limit`) is validated on the *full* path, not just the base field — or rejected outright. | `field.split(".")[0]` only → traversal into a non-exposed comodel field leaks one hop deeper. |
| 5 | **Reads run as the user.** Core/metadata reads use a per-user env, never `su`, so `ir.rule` + ACL apply. | `self.env(su=True)` on a read path; comment claims "honors ir.rule" while elevated. |
| 6 | **`sudo()` minimal + justified.** Any `sudo()` in an exposure path is scoped to one cheap, non-sensitive read and documented. | Broad `sudo()`; `sudo()` returning record data to the caller. |
| 7 | **Error sanitization.** Tool errors don't echo `str(exc)` / internal paths; argument-validation errors map to a clean message. | Raw exception text or stack detail returned to the MCP client. |
| 8 | **Multi-principal tested.** The test suite exercises a **restricted** principal, not just admin (see below). | Every test runs as `base.user_admin` → privilege leaks can't be caught. |
| 9 | **Read-back never `read([])`.** After create/write, the value returned to the caller reads an explicit field list (or just `{"id"}`), never an empty list. | `record.read([])` → returns ALL fields, leaking non-exposed ones when every field was set via value-ref (empty exposed-write list). |

## Anti-patterns seen in the wild (PASS / FAIL)

**A. `aggregate` domain not validated (inference leak)**
```python
# FAIL — domain reaches read_group unchecked; caller filters on a hidden field
rows = env[model]._read_group(args.get("domain", []), groupby=gb, aggregates=ms)
# PASS — every referenced field (group_by + measures + domain, incl. dotted) ∈ exposed set
refs = set(gb) | {m.split(":")[0] for m in ms if m != "__count"} | _domain_field_refs(domain)
bad = [f for f in refs if f not in (exposed | {"id"})]
if bad: raise UserError(_("Fields not exposed: %s", ", ".join(bad)))
```

**B. Dotted-path traversal half-validated**
```python
fields.add(leaf[0].split(".")[0])   # FAIL — only base field checked; partner_id.credit_limit slips
fields.add(leaf[0])                  # PASS — full path; never matches the flat exposed set → rejected
```

**C. Metadata surface not ACL-filtered**
```python
# FAIL — names every active exposure regardless of the caller's ACL
for exp in config.model_ids.filtered("active"): out.append(exp.model_name)
# PASS — single source of truth, filtered by the caller's ir.model.access (non-su env)
def _readable_exposures(self, user):
    env = self.env(user=user.id)
    return self.model_ids.filtered("active").filtered(
        lambda e: (m := env.get(e.model_name)) is not None
        and m.check_access_rights("read", raise_exception=False))
```
→ When the same filter feeds two surfaces (a tool *and* a resource), extract ONE
helper; duplicated copies drift.

**D. Empty read-back leaks all fields**
```python
# FAIL — exposed-write list ended up empty (all set via value-refs) → read([]) = ALL fields
return rec.read(written_fields)          # written_fields == [] → full record leaks
# PASS — never pass an empty list; return only the id (also re-checked against the whitelist)
return {"id": rec.id}
```

**E. Body buffered before the size check (auth=none → OOM)**
```python
raw = request.httprequest.get_data()     # FAIL — whole multi-GB POST in RAM before any cap
# PASS — reject by Content-Length first, then read at most max_bytes+1, before consuming the token
if (request.httprequest.content_length or 0) > MAX: raise UserError(...)
raw = stream.read(MAX + 1)

## Multi-principal test requirement

A single admin identity reads everything, so it can never reveal "a restricted
user sees X they shouldn't". **Mandate at least two principals**: full (admin)
+ restricted (e.g. `base.group_user`-only, or a user without ACL on a sensitive
exposed model). Assert the restricted principal:
- does **not** see ACL-denied models in `list_models` / `capabilities`,
- is **denied** `describe_model` / `resolve` / `aggregate` on a model its ACL blocks,
- gets a **subset** of what admin sees.

(See `odoo-test-generator` → `patterns/22-mcp-multi-principal.md`.)

## ISO mapping
A.8.3 (information access restriction), A.5.15 (access control), A.8.28
(input validation), A.5.34 / A.8.11 (sensitive data / masking).
