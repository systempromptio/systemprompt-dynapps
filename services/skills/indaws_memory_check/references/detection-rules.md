# Detection rules

How `memory-check` decides what counts as drift, and how it classifies each finding. The rules are deterministic — given a repo state, two runs must produce identical output.

## What to scan

Walk the custom modules path (see SKILL.md "Step 3: Detect custom modules path" for fallbacks). A module is any directory containing a `__manifest__.py` file.

Do NOT descend into:
- Directories whose name starts with `.` (e.g., `.git`, `.venv`).
- `__pycache__/` and `*.egg-info/`.
- Nested module directories inside a module's own `tests/` or `data/` folders (those are test fixtures, not real modules).

### Configurable excludes

The hardcoded prune list is `{.git, __pycache__, .venv, venv, node_modules}` plus any directory whose name starts with `.`. These are **always** applied — they cannot be disabled.

Projects can ADD more excludes via two mechanisms, combined additively:

1. **`.gitworkflow.yml` → `modules.exclude`** (preferred, checked-in):

   ```yaml
   modules:
     path: "parts/custom"
     exclude:
       - "docs"                  # plain entry: match by directory name
       - "docs/generated/*"      # glob entry: fnmatch against repo-relative path
   ```

   The values in `modules.exclude` are a property of the repository, not of this skill. New repos inherit whatever list the project's template decides to ship. This skill only enforces the contract (entries are strings, plain or glob, additive to the hardcoded prune list) — it does NOT document, duplicate, or own the default list.

2. **`scan_modules.py --exclude <entry>`** (CLI, repeatable or comma-separated):

   ```bash
   python3 scan_modules.py --exclude docs --exclude "vendor/*"
   python3 scan_modules.py --exclude docs,tests,vendor
   ```

**Matching rules**:

- Plain entries (no `*?[` metacharacters) match by **directory name** only — compared against the basename of each directory encountered during the walk. `docs` skips every directory called `docs`, regardless of depth.
- Glob entries (containing `*`, `?`, or `[`) are applied with `fnmatch.fnmatchcase` against (a) the repo-relative POSIX path of the directory, and (b) the bare directory name — either match prunes.

**Precedence** (highest wins, all ADD to hardcoded defaults):

1. `--exclude` CLI flag on `scan_modules.py`.
2. `modules.exclude` from `.gitworkflow.yml` (read by `reconcile.py` and forwarded).
3. Hardcoded defaults (`.git`, `__pycache__`, `.venv`, `venv`, `node_modules`, hidden dirs) — ALWAYS applied.

The hardcoded defaults are never replaced — the configured `exclude` list only adds entries.

## Finding types

Each finding has a **type**, a **severity** (info / warning / critical), and a **remediation** (what `sync` will do).

| Type | Severity | Detection rule | Remediation |
|------|----------|---------------|-------------|
| `module_added` | warning | `__manifest__.py` exists on disk, no matching `technical_name` entry in `modules.yml` | Append to `modules.custom` (or `modules.oca` if path matches OCA pattern — see "Classification" below) |
| `module_removed` | warning | Entry in `modules.yml` with a `path:` that no longer exists on disk | Remove entry from the relevant list |
| `version_changed` | info | `__manifest__.py` `version` differs from `modules.yml` entry `version` | Update `version` in the entry |
| `depends_changed` | info | `__manifest__.py` `depends` list differs from `modules.yml` entry `depends` (set comparison, order-insensitive) | Update `depends` in the entry |
| `missing_adr` | warning / info | A `module_added` has no ADR file in `decisions/` that mentions the technical name. Downgraded to `info` when the repo is in **pre-adoption state** (no real ADRs yet, see below) | Interactive prompt to create an ADR (see adr-template.md); warnings only — ADRs are never auto-created |
| `snapshot_stale` | critical | SHA256 of current `snapshot_sources` differs from the hash recorded in `SNAPSHOT.md` | Regenerate `SNAPSHOT.md` from scratch |
| `schema_missing` | critical | `.schema-version` file absent or unreadable | Exit 2 (cannot proceed) |
| `schema_unsupported` | critical | `.schema-version` has `version` not in `{1}` or `layout` not in `{v2}` | Exit 2 (cannot proceed) |

Every drift of any severity raises the exit code to `1` (except the two `schema_*` which raise it to `2`).

## Classification: custom vs core vs oca

When adding a new module entry, decide which sub-list to append to:

1. **core** — path matches `parts/indaws-core/` OR `technical_name` starts with `indaws_`. These are Indaws Core modules; the entry should carry `source: indaws_databases`.
2. **oca** — path matches `parts/oca/*` OR the module has an `OCA/` upstream reference in its manifest `website` field. Record `repo:` if inferrable from the path (e.g., `parts/oca/partner-contact/partner_firstname` → `repo: oca/partner-contact`).
3. **custom** — everything else (default bucket). This is the expected bucket for 90% of Layer-1 usage.

The MVP does NOT classify into `studio` — Studio modules are only visible from a live Odoo and are handled by Layer 3.

## Reading `__manifest__.py` safely

The manifest is a Python file but is conventionally a single dict literal. Parsing rules:

1. Read file as UTF-8.
2. `ast.literal_eval` on the file content. If it fails, fall back to `ast.parse` + walking the tree to find the first top-level `Expr(Dict(...))` and `ast.literal_eval` on that node.
3. Never `exec` or `eval` the file. A malformed manifest is a warning, not a crash — record the module as `{technical_name, path}` only and flag it for the user.

Fields to extract:
- `name` (string, human-readable — not used as the key but recorded as `note` if short enough).
- `version` (string, e.g., `"18.0.1.0.0"` — recorded as-is).
- `depends` (list of strings — recorded as-is, sorted for deterministic YAML diffs).
- `category` (optional, recorded as `note` if no better signal).

The `technical_name` is the **directory name**, not the `name` field in the manifest. This is the Odoo convention.

## Path variants

The repo layout varies across clients. `scan_modules.py` accepts a `--path` override, and the SKILL reads these fallbacks in order: `parts/custom/`, `custom/`, `addons/custom/`, `src/`. Record the path in `modules.yml` as a **repo-relative POSIX path** with forward slashes — regardless of OS. This keeps the YAML portable.

## Depends diff semantics

Two `depends` lists are equal if they contain the same set of strings (order and duplicates are ignored). A single-element difference triggers `depends_changed` — partial matches are NOT considered "close enough".

When updating, write the new list sorted alphabetically. This keeps YAML diffs readable across syncs.

## ADR association

A module is considered to have an ADR if any file in `decisions/*.md` contains the technical_name (case-insensitive, whole-token match using word boundaries). The detection is intentionally loose — ADRs may describe multiple modules in one document.

Exception: the `README.md` in `decisions/` is the template meta-document, NOT an ADR. Skip it when scanning.

## Pre-adoption state (first-time repos)

When a repo adopts `memory-check` for the first time, it typically has many pre-existing custom modules but **zero ADRs** in `decisions/` (only the README/template meta-files shipped by the skeleton). Raising a `warning` for each pre-existing module would be noisy and unfair — those decisions were made before the team adopted ADRs.

**Detection rule**: the repo is in *pre-adoption state* if `.ai-memory/decisions/` contains no `.md` files other than:

- `README.md` (the template meta-document)
- Any file matching `template.md`, `_template.md`, or `adr-template.md` (case-insensitive)

**Behavior while in pre-adoption state**:

- `missing_adr` findings are emitted at severity `info` (not `warning`).
- The message reads: *"módulo pre-existente sin ADR — flag, no error"*.
- The repo still returns exit code `1` for `check` (drift is still drift), but Layer 2 (quality-gate) treats `info` as non-blocking.

**When the state flips**: as soon as **one real ADR** lands in `decisions/` (any `.md` file that is not a meta-file), the repo is no longer in pre-adoption. Subsequent `module_added` without an ADR are emitted at `warning` severity, matching the normal contract. There is no "grace period" beyond the first ADR — the moment the team has at least one ADR, the skill assumes ADR discipline is in effect.

This rule deliberately has no state file: it is re-derived from the filesystem on every run, so no cache can go stale.

## Example report

```
Drift report for .ai-memory/ (check mode)

[warning] 2 modules added
  - acme_sale_approval (parts/custom/acme_sale_approval, v18.0.1.0.0)
    missing ADR
  - acme_stock_barcode (parts/custom/acme_stock_barcode, v18.0.1.0.0)
    has ADR: decisions/2026-03-14-acme-stock-barcode.md

[info] 1 version change
  - acme_hr_custom: 18.0.1.0.0 -> 18.0.1.1.0

[critical] SNAPSHOT.md stale
  - recorded hash: 7a8f...
  - current hash:  b3e1...

Exit code: 1
```
