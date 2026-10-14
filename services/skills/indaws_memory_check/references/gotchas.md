# Gotchas

Known pitfalls when reconciling `.ai-memory/`. Read before any write.

---

## 1. Manifest parsing

- **Never `exec` or `eval` a `__manifest__.py`.**
  Manifests are plain dict literals 99% of the time, but occasionally include comments, trailing commas, or odd whitespace. Use `ast.literal_eval`. If that fails, walk the AST for the first `Dict` node and `literal_eval` that. A manifest that cannot be parsed is a soft warning — record `{technical_name, path}` only and move on. Never crash the sync over one bad manifest.

- **`depends` may be missing.**
  Some skeletal modules omit `depends` entirely (implicit `['base']`). Treat missing as an empty list in the diff, NOT as a difference.

- **`version` may be missing.**
  Modules scaffolded by very old templates sometimes omit the `version` key. Record `null` (YAML `~`). Do not invent `"1.0.0"`.

- **`__manifest__.py` vs `__openerp__.py`.**
  Very legacy Odoo (v8 and earlier) used `__openerp__.py`. The skeleton targets v17+ which is `__manifest__.py` only. Do NOT fall back to the old name — if a repo has `__openerp__.py`, it predates the v2 ai-memory schema and should be migrated by a human first.

---

## 2. YAML round-trip

- **Comments in `modules.yml` are lost on write.**
  The skeleton's template (`modules.yml.jinja`) ships with extensive example comments. Once `sync` writes for the first time, those comments disappear. This is expected — the examples are onboarding aids, not invariants.

- **PyYAML is required.**
  Use `yaml.safe_load` / `yaml.safe_dump`, never `yaml.load` with the default Loader. Preflight-check the import on startup. If PyYAML is missing, exit `2` with a clear error message and install hint: `pip install pyyaml`. If the skill runs in an isolated environment (e.g., pipx), install via `pipx inject <env> pyyaml`. Never silently fall back to a hand-rolled parser — the error surface is explicit and actionable.

- **Preserve empty lists as `[]`, not `null`.**
  The templates use `custom: []`. After sync, if the bucket is still empty, write `[]` explicitly (serialize as inline flow). An unset value vs an empty list is semantically different to downstream tools.

---

## 3. Path quirks

- **Symlinks in `parts/custom/`.**
  Some repos symlink OCA modules into `parts/custom/` for dev convenience. Resolve symlinks when deciding the final path to record, but classify by the resolved target (OCA symlinks go to the `oca` bucket).

- **Case sensitivity.**
  Module technical names are lowercase by Odoo convention. Enforce lowercase in the diff — if a directory is `AcmeSale`, record `acmesale` and warn the user that the directory name is non-standard.

- **Monorepo vs single-repo.**
  When the custom path is `src/` (monorepo), there may be non-module subdirectories at the same level (e.g., `src/tests/`, `src/docs/`). Only directories with a `__manifest__.py` count. Do NOT treat `src/` itself as a module.

---

## 4. Hash and snapshot

- **Line endings matter.**
  Repos cloned on Windows may end up with `\r\n` line endings locally. The hash algorithm normalizes to `\n` before hashing. If you skip normalization, CI (Linux) and local (Windows) produce different hashes — the snapshot appears stale forever.

- **BOMs break the parser.**
  Strip the UTF-8 BOM (`\ufeff`) at the start of any source file before hashing. Some editors on Windows write it by default.

- **Hash order matters.**
  The hash is computed over sources **in the order declared by `.schema-version:snapshot_sources`**. Do NOT sort the list, do NOT deduplicate. Order is part of the contract.

---

## 5. ADR detection

- **False positives in module→ADR matching.**
  Grepping for `acme_sale` matches `acme_sale_approval` and `acme_sale_reports` too. Use word-boundary matching (`\bacme_sale\b`) to avoid matching substrings.

- **ADRs may span modules.**
  One ADR may document "the acme_sale family" and cover three modules. The loose word-boundary grep catches this — if the ADR mentions `acme_sale`, `acme_sale_approval`, and `acme_sale_reports`, all three are considered covered.

- **Do not count the `decisions/README.md`.**
  That file is the template meta-document. It mentions module names in examples, which would produce false positives.

- **ADRs are NEVER auto-created.**
  Not under `--yes`, not in non-interactive mode, not "as a placeholder". Creating a blank "proposed" ADR on behalf of the dev means the rationale box stays empty forever — the ADR gets merged as noise, loses all value, and clutters `decisions/` with undocumented decisions. Every auto-created ADR would have to be rewritten by hand anyway, so skip the middle step. The only way an ADR enters the repo is: (a) interactive mode, dev says `s` to the prompt → stub created, dev fills it in later; (b) dev writes one from scratch. Full stop.

- **Pre-adoption state is filesystem-derived, not cached.**
  The skill decides "pre-adoption" vs "ADR discipline in effect" on every run by counting real `.md` files in `decisions/` (excluding README and `*template*.md`). There is no state file to update, no flag to flip. This means: copy a repo from a pre-adoption client to one with mature ADRs, and the skill instantly switches behavior — no manual migration needed.

---

## 6. Git staging

- **`git add .ai-memory/` may stage files the user did not expect.**
  If the user has local-only edits in `.ai-memory/knowledge/*` that they intentionally left unstaged, `sync` would add them too. Mitigation: use `git add` only for the machine-zone subpaths (`inventory/`, `decisions/`) and never for `knowledge/` or `CONTEXT.md`. The `--no-stage` flag opts out entirely.

- **Repo may not be a git repo.**
  Someone running the skill in a plain folder (e.g., testing the reconciler on an unzipped archive). Check `git rev-parse --git-dir` before staging; skip the `git add` step silently if outside a git repo, and mention it in the summary.

---

## 7. Concurrency

- **Two `sync` runs in parallel corrupt `modules.yml`.**
  No file locking in MVP. Document as a known limitation: the dev should not run `memory-check sync` and a skill that writes to `modules.yml` at the same time. In practice this is unlikely — the reconciler takes <1s on realistic repos — but if Layer 3 ever runs against the same checkout as a local dev, add a `.ai-memory/.lock` file.

---

## 8. Exit code discipline

- **Never exit 0 after detecting drift in `check`.**
  `quality-gate` reads the exit code. Exit 0 means "all good, no findings". If `check` prints a drift report AND exits 0, Layer 2 silently accepts the drift. Always map `any_drift → 1`.

- **Never exit 1 on errors.**
  `1` means "drift found, which is normal and handled". Malformed YAML, unsupported schema version, or IOError must exit `2` so Layer 2 surfaces them as CRITICAL.

---

## 9. Scan edge cases

- **Empty path = silent empty scan.**
  If `modules.path` resolves to an empty directory, `scan_modules.py` returns `[]`. The skill then detects "all modules were removed" in `modules.yml` and would happily blank the custom list. Mitigate: warn when the scan returns empty AND `modules.yml` has non-empty `custom` entries. Require explicit `--force` in that edge case, otherwise abort with a message pointing to the mismatch.

- **`modules.exclude` matches directory NAMES, not absolute paths.**
  A plain entry like `docs` in `.gitworkflow.yml` skips every directory named `docs`, at any depth. It does NOT anchor to the repo root. If you need to exclude only a specific subtree, use a glob pattern (e.g., `docs/generated/*`) — globs are matched with `fnmatch` against the repo-relative POSIX path. The hardcoded defaults (`.git`, `__pycache__`, `.venv`, `venv`, `node_modules`, hidden dirs) are always applied on top; `exclude` never removes them.

- **`__manifest__.py` without `__init__.py`.**
  A directory with a manifest file but no `__init__.py` is malformed Odoo (won't load at runtime), but `scan_modules.py` still picks it up. The YAML will list a module that does not actually work. Warn when this happens (`[WARNING] <name> has __manifest__.py but no __init__.py — module won't load`) and either skip the entry or mark it with a `broken: true` flag in the YAML, per team convention.

- **CRLF manifests (Windows clones).**
  After `git clone` on Windows with `autocrlf` enabled, `__manifest__.py` may have `\r\n` line endings. `ast.literal_eval` handles this for the parse itself, but the module `path` and string values inside could contain unexpected `\r` characters that leak into the YAML output. Mitigate: normalize before parsing with `read_text(encoding="utf-8").replace("\r\n", "\n")`. Verify this normalization is applied in `scripts/scan_modules.py`.
