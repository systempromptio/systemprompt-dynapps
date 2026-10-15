# Git Manager — Actions (full descriptions)

The parent action entry below is the **single source of truth** for each flow.
The shortcut SKILL.md files are thin wrappers. When you need to change how an
action works, edit THIS file — do NOT duplicate logic into the shortcut.

### 1. status — Where am I? What's next? [SONNET]
Analyze git state: current branch, uncommitted changes, commits ahead/behind, MR/PR status, platform drift. Detect which flow the user is in by matching branch name against config patterns (or branch naming conventions in inference mode). End with a concrete next-step suggestion.

### 2. guide — Interactive workflow wizard [OPUS]
Walk the user through a complete flow step by step. Read [git-concepts.md](git-concepts.md) for operational knowledge. Read [gotchas.md](gotchas.md) before risky steps.

**Technical user**: Ask what they need (feature/bugfix/hotfix/release) → task ID → execute each step with confirmation.

**Non-technical user (consultant mode)**: When the request sounds functional ("quiero contribuir con algo", "tengo un cambio que hacer", "quiero añadir una funcionalidad"), enter assisted mode:
1. Ask in plain language: "¿Qué quieres hacer exactamente?" (new feature / report a bug / improve something)
2. Ask: "¿Tienes una tarea asignada en Odoo?" — if yes, extract task ID; if no, explain they need one
3. Translate their intent to the right git flow (feature branch, commit, MR/PR)
4. Execute each step with explanations in non-technical language
5. Guide all the way through to MR/PR creation with a clear description

Detect the user's profile from how they phrase their request. Do not ask "are you technical or non-technical?" — infer it.

### 3. branch — Create branch [SONNET]
Determine from config (or inference): which flow type, which branch to create from, naming pattern. Verify base branch is up to date. Ask for task ID if the pattern needs one. In inference mode, suggest the name and confirm before creating.

### 4. commit — Stage and commit [SONNET]
Check for sensitive files (.env, credentials) — warn and refuse to stage. Read [commit-standards.md](commit-standards.md) for semantic judgment on choosing the type. Use the commit format from config (or conventional commits default in inference mode).

**ai-memory check** (non-blocking): If `.ai-memory/` exists in the repo, ask the user: "Este repo tiene `.ai-memory/`. ¿Has ejecutado `memory-check`?" If they say no, suggest running it first but do NOT block the commit — there are situations where it doesn't apply. If `memory-check` is available as a skill, offer to run it with `--check` (dry-run) to show drift status.

### 5. push — Push and create MR/PR [SONNET]

**Pre-commit gate**: If `.pre-commit-config.yaml` exists in the repo, run `pre-commit run --all-files` BEFORE pushing. This is mandatory, not optional.
- First: check `which pre-commit`. If not installed → ask: "El repo tiene pre-commit configurado pero no lo tienes instalado. ¿Quieres que lo instale yo o prefieres hacerlo tú?" If yes → install using the right command for the OS (see [environment-detection.md](environment-detection.md)). If no → show the command and stop.
- All pass → proceed with push
- Failures → show which hooks failed. If auto-fixable (formatting), apply fixes, stage them, amend the commit, then proceed. If not auto-fixable (lint errors), stop and show the user what to fix.

Push branch. Ask about MR/PR creation. Read [mr-template.md](mr-template.md) for the description. Auto-detect change categories from the diff. Use provider commands from [providers.yml](providers.yml). Read [gotchas.md](gotchas.md) — never `--remove-source-branch` on permanent branches.

### 6. mr-status — Check MR/PR status [SONNET]
Query provider CLI for current branch's MR/PR. Report: pipeline, approvals, discussions, merge readiness.

### 7. pipeline — Diagnose CI failure [OPUS]
Read [pipeline-diagnosis.md](pipeline-diagnosis.md). Identify failed jobs, read logs, classify error (pre-commit/test/build/install/deploy), distinguish code issues from infrastructure issues, suggest fix with local verification command.

### 8. cherry-pick — Selective production deploy [OPUS]
Read [git-concepts.md](git-concepts.md) for merge method implications on cherry-pick. Read [gotchas.md](gotchas.md) for cherry-pick gotchas. Preview with `--no-commit`. **Requires confirmation** — production operation.

### 9. hotfix — Emergency production fix [OPUS]
Read [gotchas.md](gotchas.md) — **production FIRST, then integration**. Create hotfix branch from production → fix → MR/PR to production → tag → sync to integration. If a release branch is open, sync there too.

### 10. release — Release branch [OPUS]
Read config for release mode (or ask in inference mode). Read [git-concepts.md](git-concepts.md) for release strategy concepts. Develop-based: all features go together. Release-based: selective inclusion.

### 11. tag — Manage version tags [OPUS]
Read [gotchas.md](gotchas.md) — always annotated, may trigger deployments. Verify on production branch. Calculate next version from existing tags. **Requires confirmation.**

### 12. recovery — Fix mistakes [OPUS]
Read [recovery.md](recovery.md) and guide through the appropriate playbook. Decision tree: pushed or not? → specific scenario.

### 13. init — Generate .gitworkflow.yml [OPUS]
Interactive onboarding wizard, designed to be the first thing a user runs in a new repo. Two phases: automatic detection (branches, provider, CI, modules layout) + interactive questions (task pattern, merge method, release mode, modules path). Delegates rendering to `scripts/render_gitworkflow.py` using the odoo-skeleton as single source of truth.

Read [init-wizard.md](init-wizard.md) for the full flow, the `modules` contract, the hardcoded prune list, and the rationale for not duplicating skeleton defaults.

### 14. sync — Sync feature branch with base [SONNET]
Verify no uncommitted changes. Determine base branch and sync method from config (or from [git-concepts.md](git-concepts.md): rebase for squash workflows, merge for ff). On conflict → recovery playbook #6.

### 15. cleanup — Remove stale branches [SONNET]
List merged/closed MR/PR branches. List remote branches inactive > configured threshold. **Never delete protected branches.** Show list, confirm before each deletion group.

### 16. review — AI code review of a MR/PR [OPUS]
Fetch the MR/PR diff. Check blockers first (pipeline failed? unresolved discussions?). Then delegate deep analysis to the `quality-gate` skill, which invokes `odoo-code-auditor` for Odoo-specific checks (security, antipatterns, performance, conventions, missing tests, reimplemented standard features).

**Memory drift check** (if `.ai-memory/` present): invoke the `memory-check` skill (delegated) with action `check --json`. Claude resolves it by name (same plugin). Do NOT call the script directly via shell. Parse the returned findings and fold them into the review output alongside code findings. Drift types:
- `module_added` — new module in the PR but not in `.ai-memory/inventory/modules.yml`
- `module_removed` — module deleted but still listed in inventory
- `missing_adr` — new/changed module without an ADR in `.ai-memory/decisions/`
- `snapshot_stale` — `SNAPSHOT.md` hash doesn't match current YAML state
- If `memory-check` exits with code 3 (`.ai-memory/` absent), skip this check silently — non-adopted repos are not in scope.

**Auditoría de afirmaciones (contrastar la descripción del PR con el código)**: el diff dice qué cambia, no si lo que el autor promete se cumple. Antes de reportar, extrae del título y del cuerpo del PR cada **afirmación comprobable** y verifícala contra el código:

- qué **bloquea / valida / avisa** (¿el error llega al usuario, o lo traga un `except` del motor y solo queda en un registro técnico?),
- qué declara **no afectado** ("las plantillas X no cambian", "retrocompatible", "apagado por defecto") → localiza la ruta y confirma que no pasa por el código nuevo,
- qué dice **cubrir con tests** → abre los tests y comprueba que afirman ESO (ver `common-skills:odoo-verify-green`),
- qué dice haber **actualizado** (versión de manifest, i18n, migraciones, docs).

Marca cada una **verificada / falsa / no comprobable** y saca las que no se sostienen en una sección propia del informe: **"Afirmaciones del PR que no se sostienen"**. Una afirmación falsa es un hallazgo de pleno derecho, con su severidad — y suele ser el más caro, porque el reviewer humano lee la descripción y no vuelve a comprobarla. *(Medido 2026-08-13 en `indawsodoo/indaws-edi#19`: código correcto, tests coherentes con el código, y aun así la funcionalidad anunciada —"avisa y no deja exportar"— no existía para el usuario porque el `UserError` lo capturaba el motor. Ningún detector del diff lo habría visto.)*

**Lo que el diff NO ensena, y donde estaban los bloqueantes** *(medido 2026-09-02 en `indawsodoo/indaws-edi#30`: los dos hallazgos bloqueantes de la revision no estaban en ninguna linea anadida)*:

- **Los defaults y los datos que le caen encima al codigo nuevo.** Un PR que anade una accion, un boton o un punto de entrada se lee **contra los `data/*.xml`, los defaults de campo y las ACL** que aplican a lo que crea. *Caso: la accion nueva creaba un documento cuyo proveedor no declaraba `resend_policy` en su `data/`, y el default del modelo es `block` + `required` → la accion era inusable a partir del primer uso, y el diff estaba impecable.*
- **Ejecuta mentalmente la accion una SEGUNDA vez.** Casi todo lo que rompe en un flujo de negocio real (regenerar el fichero de un periodo, reenviar, corregir un registro y repetir) sale de la segunda pasada, no de la primera. Es una pregunta fija de la revision, no una intuicion.
- **Un cambio en un modulo NUCLEO se revisa desde sus CONSUMIDORES.** La pregunta no es "¿esta bien el campo nuevo?" sino "¿que ve quien ya usaba esto?". *Caso: el PR anadia un campo y un metodo a `indaws_edi_base`, del que cuelgan SII, Verifactu, facturas EDI y stock; el unico cambio real de comportamiento para ellos era el TEXTO de un error — y ese texto reescribia un `msgid` ya traducido.*
- **Buscar quien es "el primero".** `records[0].campo`, `[:1]`, un `mapped()` que se colapsa, un `ensure_one()` que falta: sacar de N registros el criterio del primero y aplicarlo a todos. *Caso: el formato y el layout salian de `moves[0]` mientras el codigo de empresa se resolvia por registro → un fichero de ancho fijo internamente incoherente, en silencio.*

**Un test que YA EXISTE no se juzga por si pasa, sino por si DISCRIMINA.** Por cada assert nuevo del diff: *¿que pasa si borro el codigo que vigila?* El que pasa igual no protege nada y encima invita a confiar. El criterio y los casos medidos estan en **`common-skills:odoo-verify-green`**: delegale tambien el juicio de los tests del diff, no solo el verde del pipeline. *Caso del mismo PR: el assert que daba nombre al test (`assertNotIn(b"\x97", raw)`) pasaba con y sin el fix; lo que discriminaba era la asercion de debajo. Reportarlo como "el test no vale" habria sido tan falso como darlo por bueno.*

**i18n es codigo visible, y hay un fallo que ningun test caza:** **reescribir un `msgid` ya traducido deja su traduccion huerfana** y la UI vuelve al idioma fuente sin que falle nada. Regla mecanica en la revision: **PR con cadenas nuevas o cambiadas y cero ficheros bajo `i18n/` = hallazgo**. Criterio y mecanica de catalogos en `odoo-i18n-manager`.

**Dos veredictos, separados y los dos escritos:** *"¿esta bien el PR?"* y *"¿queda arreglado el problema del cliente?"*. Pueden no coincidir, y darlos juntos hace que se cierre la incidencia al mergear. *Caso del mismo PR: el codigo era correcto y el ticket del cliente seguia abierto, porque el cliente ejecuta el modulo legacy que el PR no toca.*

Solo aplica a `review` (en `precheck` todavía no hay descripción); ahí el equivalente es contrastar el mensaje de los commits.

Report findings by severity (critical/warning/info) with line references and suggested fixes. Offer to auto-fix where possible. This action does NOT contain Odoo-specific rules — all knowledge lives in the existing analysis skills. Shortcut: `/git-review-pr`.

### 17. precheck — Local pre-MR review [OPUS]
Pre-MR equivalent of `review`. The **developer** runs this before opening a PR to catch issues early. Same detectors as `review`, different diff target.

**Target**: `git diff <base>..HEAD` where `<base>` comes from `.gitworkflow.yml` → `branches.integration` (usually `develop` or `18.0-develop`). If no config, infer from repo signals and confirm.

**Flow**:
1. Verify there are local commits ahead of the base branch. If not, tell the user there's nothing to review and exit.
2. Compute the diff and the list of changed files.
3. Delegate code analysis to `quality-gate` → `odoo-code-auditor` on the changed files (same detectors as action `review`).
4. **Memory drift check** (if `.ai-memory/` present): invoke the `memory-check` skill (delegated) with action `check --json`. Same drift types as `review` above. If exit code 3, skip silently.
5. Consolidate findings by severity: `critical` / `warning` / `info` / `suggestion`.
6. If any `critical` → tell the user: *"Hay X hallazgos críticos. Te recomiendo arreglarlos antes de abrir el PR. Quieres que los corrija yo?"*. No bloquea técnicamente — la decisión es del dev.
7. If user accepts auto-fix, apply the fixes, stage them, and suggest a commit message. Do NOT commit automatically (the dev owns the commit).

**Difference vs `/git-review-pr`**: same logic, different diff target (local branch vs remote PR) and different consumer (dev vs reviewer). Use `precheck` before `/git-pr`; use `review` before approving a PR. Shortcut: `/git-precheck`.

**Why not a separate skill with its own logic**: `precheck` shares 100% of the detectors with `review` (delegated to `quality-gate` + `odoo-code-auditor` + `memory-check`). The `git-precheck/` folder is a slash-command shortcut only (same pattern as `git-start`, `git-pr`, etc.) — it delegates back to this action. No duplicated logic, just a discoverable entry point. The post-MR GitLab agent (future, service separate) imports these references as source of truth.

### 18. migrate — Bootstrap a legacy repo into the new workflow [OPUS]
One-shot action to bring a legacy repo (typically GitHub, pre-workflow) up to the current Indaws standard: `.gitworkflow.yml`, `.ai-memory/`, linting configs, `.gitignore` merge, optional pre-commit reformatting — all in **two separate commits** (infra + optional reformat). Does NOT push; the dev reviews first. Invokes the adjacent `scripts/migrate-legacy-repo.sh` (skeleton source is URL-agnostic: reads `SKELETON_PATH` / `SKELETON_REPO_URL` from env, or prompts the dev).

Read [migrate-action.md](migrate-action.md) for the full flow (preflight, context detection, the 4 interactive questions, post-processing of `.gitworkflow.yml`, commit separation rules, incremental mode, scope boundaries). Shortcut: `/git-migrate`.

### 19. new-repo — Bootstrap a brand-new customer repo [OPUS]
Sister of `migrate` for the opposite starting state: empty directory, no `.git`, no commits. Generates the full skeleton with `parts/custom/` layout (excluding GitLab-only files when targeting GitHub), runs `git init`, makes the first commit, creates the dual production/integration branches. Optionally calls `gh repo create + push` if `gh` is authenticated. Invokes the adjacent `scripts/new-repo.sh` (same `SKELETON_PATH` / `SKELETON_REPO_URL` resolution as migrate). Default platform is `github` while GitLab is not operational; will flip to `gitlab` once the migration is complete — the user-facing command does NOT change, only the internal logic.

Read [new-repo-action.md](new-repo-action.md) for the full flow (preflight, version-hint detection, questions, copier excludes per platform, branch setup, optional remote creation, scope boundaries). Shortcut: `/git-new-repo`.

## Startup sequence (full)

### Step 0: Detect environment
Read [environment-detection.md](environment-detection.md). Detect OS, shell, and terminal. This determines:
- Which install commands to suggest for provider CLI (brew/apt/winget)
- Which shell syntax to use in examples (bash vs powershell)
- Known terminal limitations (e.g., Windows CMD path length, PowerShell quoting)

Store the result for the rest of the session. Do NOT report it to the user unless relevant (e.g., when installing a CLI).

### Step 1: Verify git repo
`git rev-parse --git-dir`. If not a repo, stop.

### Step 2: Read workflow config

Look for `.gitworkflow.yml` in repo root.

**Config mode** (file found): Read it entirely. Interpret every field semantically by its key name and inline comments. The file is self-documenting. Use it to guide all subsequent actions.

**Inference mode** (file not found): Detect conventions from the repo:
- Branches: `git branch -r` → infer roles by name (`master`/`main` = production, `develop`/`dev`/`staging` = integration)
- Provider: remote URL patterns
- Commit style: assume conventional commits (industry standard)
- Task pattern: ask once per session if needed (detect from current branch name if possible)
- Merge method: query platform API if available, otherwise ask once

In inference mode, confirm before ambiguous actions. At session end, suggest: "Want me to generate a `.gitworkflow.yml` so I don't need to ask next time?"

### Step 3: Detect provider and verify CLI
Order: config `provider` field → CLI availability → remote URL.

**CLI check**: Verify the provider CLI is installed (`which gh` / `which glab`). If not installed:
1. Ask: "No tienes {cli} instalado. ¿Quieres que lo instale yo o prefieres hacerlo tú?"
2. If user says yes → run the install command from [providers.yml](providers.yml) for the detected OS (Step 0)
3. If user says no → show the install command and stop

**Auth check**: If CLI installed, verify authentication (`gh auth status` / `glab auth status`). If not authenticated:
1. Ask: "Tienes {cli} instalado pero no autenticado. ¿Quieres que te guíe?"
2. If yes → run auth flow from [providers.yml](providers.yml)

This same pattern applies to **every external command** the skill needs to run. Before executing any tool (`pre-commit`, `gh`, `glab`), always check it exists first. If missing, ask the user: install it for them or let them do it.

### Step 4: Repo context snapshot
Check which exist: `.gitworkflow.yml`, CI config, `.pre-commit-config.yaml`, `CODEOWNERS`, `dyn-build.yaml`, `.copier-answers.yml`. Report findings.

### Step 5: Validate platform settings and auto-fix drift (config mode only)
If provider CLI available, compare config against platform API using [providers.yml](providers.yml).

**Detection**: Compare each setting in `merge_requests` against the platform API. Build a drift report table (Setting / Config / Platform / Status).

**Auto-fix**: If drifts are found that can be fixed via API, ask the user: "He detectado {n} diferencias entre tu config y la plataforma. ¿Quiero corregirlas?" If the user says yes, apply the fixes using the `platform_fix` commands from [providers.yml](providers.yml). Show confirmation after each fix.

**What can be auto-fixed**:
- Merge method (disable non-configured methods)
- Delete branch on merge (enable/disable)
- For GitLab: squash option, merge checks

**What CANNOT be auto-fixed** (report only):
- Branch protection rules (requires admin / paid tier)
- Required approvals (requires admin / paid tier)
- Status checks (requires CI configuration)

**Platform truth > config** for settings that exist in both places. Config truth for skill-only settings (naming, flows, commits).

## Slash command shortcuts (detail)

8 thin-wrapper skills provide quick access to the most common actions. Each one invokes git-manager with a specific action and streamlined input.

**Typical feature cycle**: `/git-start` → `/git-sync` → `/git-precheck` → `/git-pr`. **One-shot bootstrap**: `/git-new-repo` (new customer onboarding) or `/git-migrate` (legacy repo onboarding) — each runs once per repo. **Utilities**: `/git-status` (any time), `/git-review-pr` (reviewer-side, on open PRs).

| Slash | Maps to | What it does |
|-------|---------|-------------|
| `/git-new-repo` | `new-repo` | Bootstrap a brand-new customer repo from the skeleton: copier-render + `git init` + first commit + dual branches + optional `gh repo create`. One-time per repo. |
| `/git-migrate` | `migrate` | Bootstrap a legacy repo: generates `.gitworkflow.yml`, copies `.ai-memory/`, linting configs, optional pre-commit reformatting. One-time per repo. |
| `/git-start <task-id>` | `branch` | Creates feature branch. Reads task format from `.gitworkflow.yml`; if no config, asks interactively. |
| `/git-sync` | `sync` | Syncs feature branch with base (rebase or merge per config). |
| `/git-precheck` | `precheck` | Pre-MR review: runs the same detectors as `/git-review-pr` but against the local diff (`<base>..HEAD`). Use it *before* `/git-pr` to catch issues early. |
| `/git-pr` | `push` | Pushes current branch and creates MR/PR with auto-generated description. |
| `/git-review-pr <id>` | `review` | AI code review on an open PR: Odoo antipatterns, security, tests, conventions, memory drift. Delegates to quality-gate + code-auditor + memory-check. |
| `/git-status` | `status` | Current branch, changes, MRs, pipeline — full repo snapshot. |

These shortcuts exist as separate skills (`git-start/`, `git-pr/`, `git-precheck/`, etc.) so they appear in the slash command list. They all delegate to this skill's logic and references.

**Editing rule** (canonical flow): the parent action entry above is the **single source of truth** for each flow. The shortcut SKILL.md files are thin wrappers with the action body kept minimal (banner + short usage). When you need to change how an action works, edit this file — do NOT duplicate logic into the shortcut. This prevents drift between parent and shortcuts.

## Branch cleanup policy (full)

After an MR/PR is merged, feature and hotfix branches are **deleted by default**. This keeps the repo clean. The traceability required by ISO27001 is preserved in:
1. The squashed commit in the target branch (full history)
2. The archived MR/PR in the platform (diff, comments, approvals, pipeline)
3. Platform audit events (who did what, when)

Protected/permanent branches (develop, master, version branches) are **never deleted**.

## Dry-run mode (full)

When user asks "what would you do?" / "dry-run" / "show commands": show exact commands in order without executing, with explanations. Ask "Execute? [y/n]" at the end.
