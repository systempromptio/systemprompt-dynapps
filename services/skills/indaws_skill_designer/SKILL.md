---
name: indaws-skill-designer
version: 3.12.3
model: opus
effort: medium
description: "Design, audit, optimize, and govern Claude Code skills across a federated 9-suite ecosystem. Use when creating new skills, reviewing existing ones, optimizing token efficiency, detecting duplicates, validating the ecosystem, generating catalogs, moving skills between suites, or managing cross-suite dependencies. Covers 9 skill categories, progressive disclosure, gotcha engineering, agent routing, and federated architecture (vertical common + horizontal peers)."
---

# Skill Designer v3.0

Skill architect and ecosystem governor for a federated multi-repo skill ecosystem.

## Agent Routing

| Phase | Model | Runs as |
|-------|-------|---------|
| Step 0 — Classify + discover | sonnet | Subagent |
| Step 1 — Read categories | sonnet | Subagent |
| Step 2 — Apply patterns | **opus** | Principal |
| Step 3 — Generate/Review | **opus** | Principal |
| Step 3E — Ecosystem scripts | sonnet | Subagent |
| Step 4 — Deliver | **opus** | Principal |

## Federated Architecture

See `ARCHITECTURE.md` (common root). **Vertical:** common → child suites. **Horizontal:** child suites delegate to each other via CATALOG.md. Rule: 3+ suites → common. 2 → horizontal. 1 → local.

## Workflow

### Step 0 — Classify + Discover [SONNET SUBAGENT]

Launch Agent(model: sonnet):
- **Input:** user's request
- **Tasks:**
  1. Determine mode: Design / Audit / Optimize / Ecosystem
  2. If Design: read CATALOG.md from ALL suites (`$SKILLS_ROOT/ai-claude-*/CATALOG.md`) to check if a similar skill already exists
  3. If Audit/Optimize: locate the skill directory
  4. If Ecosystem: determine action (validate, duplicates, catalog, add, move, dependencies)
- **Output:** `{mode, existing_similar_skills[], skill_path?, target_suite?, ecosystem_action?}`
- **Must NOT:** make design decisions

### Step 1 — Classify [SONNET SUBAGENT]

Launch Agent(model: sonnet) to read `references/categories.md` and classify the skill into one of 9 categories. Also determine:
- **Target suite** using `references/ecosystem-governance.md` routing tree
- **Target layer** (0-3) if the skill goes to common, using ARCHITECTURE.md
- **Granularity test** (`references/skill-granularity.md`): ¿es una capacidad con trigger propio o reuso (→ skill), solo volumen (→ reference de una skill existente), o un flujo lineal (→ NO orquestar)?

### Step 2 — Apply patterns [OPUS — principal]

Read references based on mode:
- `references/anthropic-official-guidelines.md` — **READ FIRST**
- `references/best-practices.md` — Indaws-specific patterns
- `references/anti-patterns.md` — 20 anti-patterns
- `references/workflow-patterns.md` — 5 in-skill workflow structures (read in Design mode)
- `references/agent-routing-patterns.md` — sonnet/opus routing per phase
- `references/ecosystem-governance.md` — federation rules, dependency protocol

### Step 3 — Generate or Review [OPUS — principal]

**Design mode:**
1. Verify no duplicate exists (Step 0 output)
2. Read `references/agent-execution-tree.md` — design the agent tree: list delegations, classify each (Pattern A/B/C or inline), place Agent() calls in workflow, estimate cost
3. Generate SKILL.md + folder using `templates/`. Include: frontmatter, workflow with **real Agent() calls** per `agent-execution-tree.md`, routing tags, GOTCHAS, NEVER/ALWAYS, Delegates/Consumed, REFERENCES
4. Reference `common:references/shared/` — don't reimplement connection, batch, fields
5. Declare deps + update CATALOG.md + DEPENDENCIES.md

**Audit mode:** Run `references/audit-checklist.md` (22 checks), produce report.

**Optimize mode:** Analyze token consumption, suggest compaction.

**Ecosystem mode:** See Step 3E.

### Step 3E — Ecosystem Governance [SONNET SUBAGENT]

Launch Agent(model: sonnet) for the requested action:

**Available actions:**

1. **Validate ecosystem:** `scripts/validate_ecosystem.py $SKILLS_ROOT`
2. **Detect duplicates:** `scripts/detect_duplicates.py $SKILLS_ROOT`
3. **Generate catalog:** `scripts/generate_catalog.py <suite-dir>` (one or all)
4. **Verify suite:** `scripts/verify_suite.py <suite-dir>` — 12-point checklist (incl. description ≤1024), X/N PASS
5. **Route a skill:** ecosystem-governance.md decision tree → suite + layer
6. **Add skill:** Create → CATALOG → DEPENDENCIES → verify no duplicate
7. **Move skill:** Copy → delete → update CATALOGs → update DEPENDENCIES → grep refs
8. **Impact analysis:** grep DEPENDENCIES.md across suites for consumers
9. **Revamp suite:** `references/revamp-playbook.md` (8 fases). **Mantenimiento:** `scripts/doctor.py` + `scripts/staleness.py --latest N`; ritual en `references/maintenance-playbook.md`. **Dos niveles en `doctor`:** los *issues* bloquean y se baselinean (incluye `check_disk_refs.py`: rutas de addon y `depends` vs el `__manifest__.py` real); los **AVISOS** no bloquean ni se baselinean (`check_version_claims.py`: evidencia débil, precisión medida en su docstring). **Acota por modelo y por módulo** cuando la afirmación lo nombra —pregunta al inventario del wiki de código (ruta por `ODOO_WIKI_INVENTORY`, ver el docstring del checker), y para un módulo mira si existe el DIRECTORIO en vez de grepear la cadena—; sin inventario en disco se comporta como antes. Medido: 25 → 19 sospechosos, y los descartados eran afirmaciones correctas. El pre-push corre `--no-advisories`; `--advisories` los lista todos.
10. **Empirical tooling (opt-in, costly — real `claude -p` sessions; ver `references/empirical-triggering.md`):** `eval_triggering.py` (triggering real + steal-map), `optimize_description.py` (propone, no aplica), `eval_value.py` (valor incremental con/sin skill + coste), `blind_compare.py` (A/B ciego para revamps). Para zanjar colisiones, cambios de description y gates de revamp — nunca por edición.
11. **Governance dashboard / packaging:** `dashboard.py <root>` → self-contained health HTML (publish with Artifact tool); `package_skill.py <dir>` validates (Anthropic-strict frontmatter) then zips a `.skill`.

The principal (opus) reviews the report and proposes actions.

### Step 4 — Deliver [OPUS — principal]

Output: skill folder, audit report, optimized version, ecosystem health report, or migration plan.

## RULES

### NEVER
1. Create a skill that straddles multiple categories — split it. But NEVER split by *volume* (use references) nor add an orchestrator for a linear single-intent flow — see `references/skill-granularity.md`
2. Put full API documentation inline in SKILL.md — use references/
3. Include information Claude already knows about coding
4. Hardcode credentials, IDs, or URLs — use config.json
5. Use `activation`, `tags`, `globs` in frontmatter — ghost fields, zero effect
6. Duplicate content that exists in common Layer 1/2 — delegate instead
7. Create a skill without checking ALL CATALOG.md files first
8. Hardcode absolute paths in Agent() prompts — use skill names for portability. Ni marcar un subagente como **obligatorio**: hay configuraciones que prohíben el fan-out, así que ese paso no se ejecuta y nadie se entera → delegación **condicional + camino inline**, siempre (`ecosystem-governance.md` §«La delegación se escribe CONDICIONAL», con los datos que lo forzaron).

### ALWAYS
1. Include GOTCHAS section with real failure patterns
2. Use progressive disclosure (SKILL.md < 150 lines, details in references/)
3. Make description trigger-oriented, third person, with action verbs
4. Declare `effort`; name a model only where a step is actually delegated to a subagent (conditional, with an inline path).
5. Reference `common:references/shared/` for connection, batch, field patterns
6. Update CATALOG.md and DEPENDENCIES.md after any add/move/delete
7. Check for horizontal delegation opportunities before duplicating functionality

## GOTCHAS

1. **`model: sonnet` in frontmatter does NOT route to sonnet** in slash commands. Must use explicit `Agent(model: sonnet)` in the workflow. Y los pasos 0, 1 y 3E **de esta misma skill no son obligatorios**: pre-flight rojo, fan-out prohibido o el usuario pidiéndolo → inline, y se dice (predicar la regla 9 sin aplicársela es la vía rápida a que no se crea ninguna).
2. **Plugin cache doesn't hot-reload.** After modifying a skill via symlink, old content persists until session restart.
3. **Ghost frontmatter eats tokens silently.** `activation`, `tags`, `globs` are parsed but ignored.
4. **References deeper than 1 level get partially read.** Keep all references direct from SKILL.md.
5. **Descriptions over 1024 chars are truncated.** Anthropic hard limit.
6. **Duplicate skills across suites cause silent model confusion.** Same name → Claude may load wrong one.
7. **CATALOG.md is auto-generated** — never edit manually. Run `generate_catalog.py` after changes.
8. **Horizontal deps are delegation, not code import.** A skill invokes another by name — it doesn't copy references or scripts from another suite.
9. **Moving a skill breaks references.** After moving, grep all SKILL.md files across ALL suites for the old location.
9b. **Puntero a la reference de OTRA skill:** nómbralo en prosa —«la referencia `view-layer.md` de `odoo-owl-component-builder`»—, que ningún checker lo lee como ruta. Con barra necesita prefijo de suite **y** `skills/` intermedio; a secas, `validate_ecosystem` lo busca en la skill que lo escribe (desde 2026-08-16 avisa, no bloquea). Y un ejemplo de ruta escrito literal en un `.md` lo leen los checkers como puntero real: este gotcha se pilló a sí mismo.
10. **Hardcoded paths in Agent() prompts break portability.** Use skill name only — Claude already knows installed skills. Never write absolute paths like `$SKILLS_ROOT/.../SKILL.md`.
11. **`effort` is NOT a real Agent() parameter** — encode it in the prompt text to steer a subagent. In a SKILL.md it *is* real: the loader validates it (`low|medium|high|xhigh|max` or an integer) and warns on a bad value. The full verified field list is `scripts/frontmatter_keys.py`, the one list every gate imports.
12. **Un 0% de recall puede ser el SUELO del arnés, y el arnés tiene VARIANZA.** Medido 2026-09-04: el control `odoo-traceback-doctor` con query de manual dispara **0/4** —y la query contiene *verbatim* una de sus frases de `Triggers:`—. 3 runs NO bastan para comparar variantes: un diseño pareado de 6 pares (42 sesiones) **no confirmó** la hipótesis de que las frases situacionales lleguen mejor (28% vs 44%, un par al revés), y la misma query da 3/3 un día y 1/3 al siguiente. Lo único que se repite: **estar en la lista `Triggers:` no hace que dispare**. Con suelo cero `optimize_description.py` devuelve el original — es información. `references/empirical-triggering.md`.
13. **Empirical eval: namespaced names + real cost.** In `claude -p` the Skill tool fires with `<plugin>:<name>` (e.g. `developer-skills:x`) for plugin skills, bare for installed ones — `eval_triggering.py` normalizes to bare; any name comparison MUST strip `prefix:`. Each query×run is one real `claude -p` session (plan-mode, killed at first tool → skill never executes): accurate but costly, opt-in only, never CI-per-edit.

## Delegates to
None — standalone meta-skill.

## Consumed by
All skills in all suites — for ecosystem governance, skill design, audit, and optimization.

## REFERENCES

- `references/anthropic-official-guidelines.md` — **READ FIRST** — Anthropic official rules · `references/categories.md` — 9 skill categories · `references/best-practices.md` — Design patterns and tips
- `references/anti-patterns.md` — 20 anti-patterns · `references/audit-checklist.md` — audit checklist · `references/workflow-patterns.md` — 5 in-skill workflow structures
- `references/tool-design-patterns.md` — 6 Anthropic patterns (Thariq/2026-03) · `references/ecosystem-governance.md` — Federation rules, routing, deps, add/move/revamp
- `references/skill-granularity.md` — one skill = one capability; volume→references; when to split / when to orchestrate
- `references/agent-routing-patterns.md` + `agent-execution-tree.md` — model per phase, Agent() patterns A/B/C, and how to build the agent tree (DESIGN MODE)
- `references/revamp-playbook.md` — 8-phase revamp recipe · `references/empirical-triggering.md` — empirical triggering harness (recall/precision/steal-map, cost)
- `ARCHITECTURE.md` (4 layers, deps) · `templates/` · `scripts/` — doctor, staleness, audit_skill, generate_catalog, detect_duplicates, validate_ecosystem, verify_suite · empirical: **eval_triggering, optimize_description, eval_value, blind_compare** · **dashboard, results_viewer, validate_skill, package_skill**
