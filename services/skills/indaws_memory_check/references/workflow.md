# Workflow

Detailed operational steps for `memory-check`. SKILL.md keeps only the summary — this file is the full procedure, extracted to keep the skill entry point under 150 lines.

Read this file when you are about to run `check` or `sync`, or when debugging why the skill exited with a specific code.

---

## Startup sequence

### Step 1 — Verify `.ai-memory/` exists

```bash
test -d .ai-memory/
```

If missing → **exit with code 3** (non-fatal). Print Spanish message:

> "Este repo no tiene `.ai-memory/`. Genera la estructura con el skeleton (`copier update`) y vuelve a ejecutar `memory-check`."

Code 3 tells `git-manager review` to skip the drift warning silently.

### Step 2 — Read and validate `.schema-version`

```bash
cat .ai-memory/.schema-version
```

Expected format (YAML):

```yaml
version: 1
layout: v2
snapshot_auto_generate: true
snapshot_path: inventory/SNAPSHOT.md
snapshot_sources:
  - inventory/modules.yml
  - inventory/studio.yml
  - inventory/integrations.yml
```

Validate:

- `version` is present and matches a supported value (currently `1`).
- `layout` is `v2`.
- `snapshot_sources` is a non-empty list.

On validation failure → **exit with code 2**. Print a Spanish error identifying the broken field.

### Step 3 — Detect custom modules path

Default: `parts/custom/`. If it does not exist, look for these fallbacks in order:

1. `custom/` (legacy layout)
2. `addons/custom/`
3. `src/` (monorepo layout)

If none exists → treat the custom module list as empty (repo may only have core/OCA). Do NOT error.

### Step 4 — Determine mode

- `--check` flag (or action `check`) → dry-run, read-only, exit with the drift code.
- `--sync` flag (or action `sync`) → apply changes, write files, optionally ask for ADRs.
- Default when called as slash command without args → prompt the user: "¿Qué quieres hacer, `check` o `sync`?"

Additional flags:

- `--no-stage` → do NOT run `git add .ai-memory/` after sync (default is to stage).
- `--no-adr-prompt` → skip ADR prompts entirely (useful for CI/batch).
- `--yes` → skip the "¿aplicar cambios?" prompt and auto-apply YAML changes. **Does NOT create ADRs.** Missing ADRs in non-interactive mode (or under `--yes`) are logged as warnings (`[WARNING] módulo X añadido sin ADR — requiere intervención manual`) and listed in the final summary, so the dev can come back and author them by hand. ADRs are always human-authored; see [adr-template.md](adr-template.md).
- `--json` → (only valid with `check`) emit findings as a JSON array on stdout instead of the human text report. The human text is redirected to stderr so stdout stays clean for piping to `jq`, `quality-gate`, or other consumers. Exit codes are unchanged (0/1/2/3). Each JSON element has the shape `{"type", "severity", "technical_name", "path", "message"}`. Passing `--json` with `sync` is an error (exit 2) — sync mutates state and runs interactive prompts, so a stable JSON stream is not well defined there.

---

## Action workflows

### check — Dry-run drift report [SONNET]

Read [detection-rules.md](detection-rules.md). Do NOT write anything.

1. Run `scripts/scan_modules.py` → list of `{technical_name, path, version, depends}` from filesystem.
2. Load `inventory/modules.yml` (YAML parse). If malformed → exit 2.
3. Compute the diff (added / removed / version_changed / depends_changed).
4. For each added module, check `decisions/*.md` for any ADR referencing the technical_name (word-boundary grep for the slug). Missing → flag as `missing_adr`.
5. Recompute the SHA256 of the current `snapshot_sources` (see [snapshot-format.md](snapshot-format.md)) and compare with the hash stored in `inventory/SNAPSHOT.md`. Mismatch → flag as `snapshot_stale`.
6. Print a human-readable report (Spanish) grouped by finding type.
7. **Exit code**: `0` if no drift, `1` if any drift detected, `2` on error.

### sync — Apply drift corrections [OPUS]

Same detection pipeline as `check`, then:

1. Show the full drift report first. Ask: "¿Aplicar estos cambios? [s/n]". Abort on `n`.
2. Apply changes to `inventory/modules.yml`:
   - Append new modules (under the right section: `custom`, `core`, `oca` — see [detection-rules.md](detection-rules.md) for classification).
   - Remove entries for modules no longer on disk.
   - Update `version` and `depends` on modified modules.
   - Set `generated_at` to today's ISO date and add `memory-check@{version}` to `generated_by`.
3. For each module flagged `missing_adr`:
   - **Interactive mode (TTY + no `--yes`)**: prompt one by one — "El módulo `<name>` no tiene ADR. ¿Quieres crear uno ahora? [s/n]". If yes → create `decisions/YYYY-MM-DD-<slug>.md` from [templates/new_adr.md](../templates/new_adr.md), pre-filled with context (module name, today's date, tags). Print the path so the dev can open it later. Do NOT block on the content.
   - **Non-interactive mode (no TTY) or `--yes`**: NEVER create ADRs. Log `[WARNING] módulo <name> añadido sin ADR — requiere intervención manual` to stderr and add the name to the summary's "ADRs pendientes" list. ADRs require human context — a stub would pollute `decisions/` without capturing rationale.
   - Pre-adoption state (no real ADRs in the repo yet, see [detection-rules.md](detection-rules.md)): missing ADRs are surfaced at `info` severity, not `warning`. The prompt and warning still fire, but Layer 2 won't treat them as blocking.
4. Regenerate `inventory/SNAPSHOT.md` via `scripts/snapshot_gen.py`. See [snapshot-format.md](snapshot-format.md) for the exact format.
5. Unless `--no-stage`, run `git add .ai-memory/` (only `inventory/` and `decisions/` subpaths — never `knowledge/` or `CONTEXT.md`).
6. Print summary: modules added/removed/updated, ADRs created, snapshot regenerated.
7. **Exit code**: `0` if nothing changed (repo was already in sync), `1` if changes applied, `2` on error.

---

## Implementation notes

Both actions are implemented by `scripts/reconcile.py {check|sync}`. It orchestrates `scan_modules.py` (filesystem → list of modules) and `snapshot_gen.py` (YAML sources → SNAPSHOT.md + hash). The skill does not invoke these scripts directly — only through `reconcile.py`.

## User-facing messages

All prompts, errors, and report lines are in **Spanish** (matches team language). Code comments, variable names, and SKILL.md stay in English (matches other skills).

Sample messages:

- "Detectados 3 módulos nuevos sin reflejo en `modules.yml`: `acme_sale`, `acme_stock`, `acme_hr`."
- "El módulo `acme_sale` no tiene ADR asociado. ¿Quieres crear uno ahora? [s/n]"
- "SNAPSHOT.md desactualizado — el hash no coincide con los YAML actuales."
- "Error: `.schema-version` declara layout `v1`, esta skill solo soporta `v2`. Migra con `copier update`."

## Dry-run mode

`check` is inherently dry-run. For additional verbosity during `sync`, accept `--verbose` to print each file write and hash computation. Without `--verbose`, summarize with a count-based table.

## Out of MVP

These are explicit non-goals for v0.1.0 and must NOT be attempted:

- Studio detection (requires live Odoo access — belongs to Layer 3).
- Integration detection (requires heuristic analysis of manifest dependencies + code patterns — deferred).
- Dependency graph analysis across modules.
- Creating or modifying `knowledge/` files.
- Auto-generating ADR bodies (the template is pre-filled with metadata; the user writes the content).
