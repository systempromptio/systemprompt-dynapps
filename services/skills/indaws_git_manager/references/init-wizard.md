# `init` action — Onboarding wizard reference

Detailed flow for the `init` action of the `git-manager` skill. Designed to be the first thing a user runs in a new repo.

## Detection phase (automatic, no questions)

1. Scan existing branches (`git branch -r`) to infer branch model
2. Read remote URL to detect provider (GitLab/GitHub)
3. Check for CI files (`.gitlab-ci.yml`, `.github/workflows/`)
4. Check for `.copier-answers.yml` (skeleton-generated repo)
5. Check provider CLI installed — if not, offer to install (using environment detection for the right command)
6. Check provider CLI authenticated — if not, guide through auth setup
7. Auto-detect modules layout (for the `modules.path` question below):
   - If `parts/custom/` exists and contains `__manifest__.py` files → default `parts/custom`
   - Else if the repo root contains `__manifest__.py` files → default `.` (legacy layout)
   - Else iterate the fallback chain (`custom/`, `addons/custom/`, `src/`) — first match wins
   - If nothing matches → no default; ask the user explicitly

## Interactive phase (ask the user, one question at a time)

1. "¿Qué sistema de tareas usáis?" → detect task pattern (`P{project}-{task}`, `JIRA-{number}`, etc.)
2. "¿Cómo queréis integrar los cambios?" → merge method (squash recommended, explain trade-offs)
3. "¿Cómo queréis lanzar a producción?" → release mode (develop-based vs release-based)
4. "¿Dónde están los módulos?" (ruta relativa al repo root) → modules path
   - Default: `parts/custom/` (repos generados con skeleton)
   - Legacy: `.` (módulos en la raíz, repos antiguos GitHub)
   - Otros: `custom/`, `addons/custom/`, `src/` o ruta custom
   - Pre-select the value from the detection phase; let the user override.
5. Review: show summary of detected + configured values, confirm before generating

## Implementation

Delegate rendering to `scripts/render_gitworkflow.py`. The script clones (or uses a cached local copy of) the odoo-skeleton, reads its `copier.yml` to know which questions to ask, asks them (with the auto-detected defaults from the Detection phase above), renders the skeleton's `.gitworkflow.yml.jinja` template, and writes the result to the repo root.

The skill does **NOT** duplicate the skeleton's question list or template structure — the skeleton is the single source of truth.

Usage:

```bash
python3 scripts/render_gitworkflow.py [--repo-root .] [--skeleton-path PATH] [--dry-run] [--force]
```

## Output

`.gitworkflow.yml` generated at the repo root with inline comments (autodocumented by the template). The file includes a `modules:` section with the structure below. Default values come from the skeleton, not from this skill.

```yaml
modules:
  path: "parts/custom"     # or "." for legacy repos with modules at the root
  exclude:                  # optional: list of directory names or globs
    - "docs"                # (illustrative only — actual defaults come from the skeleton)
```

## Contract

What this skill guarantees about the `modules` field:

- `modules.path` is a POSIX relative path from the repo root.
- `modules.exclude` is a list of strings. Each string is either a directory name (matched against the basename of each directory during the walk) or an fnmatch glob (matched against the repo-relative POSIX path).
- Entries in `modules.exclude` **add to** the hardcoded prune list, they do not replace it.

## Hardcoded prune list

Owned by this skill, always applied (in addition to `modules.exclude`):

- Hidden dirs (any name starting with `.`)
- `.git`
- `__pycache__`
- `.venv`, `venv`
- `node_modules`

## Where the actual default values come from

The team's project template (Copier or equivalent) decides what a newly generated `.gitworkflow.yml` contains. This skill does **not** own those defaults and does not duplicate them in its documentation or code — it only enforces the contract above.

## Drift validation (post-generation)

After writing the file, validate against platform settings if CLI is available. Suggest fixing drift if detected. See the main SKILL.md startup sequence step 5 for details on the drift auto-fix logic.
