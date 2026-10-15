# Anti-Patterns in Skill Design

## 1. The Encyclopedia Skill
**Problem:** SKILL.md is 500+ lines with everything inline. Claude reads it all on every invocation.
**Fix:** Progressive disclosure. SKILL.md is the index (50-100 lines), details in references/.

## 2. The Obvious Skill
**Problem:** Skill tells Claude things it already knows ("to create a file, use the Write tool").
**Fix:** Focus on what pushes Claude OUT of its defaults. Gotchas, edge cases, internal conventions.

## 3. The Railroad Skill
**Problem:** Rigid step-by-step instructions that break when the situation varies slightly.
**Fix:** Give Claude information and constraints, not scripts to follow. Let it adapt.

## 4. The Kitchen Sink Skill
**Problem:** One skill tries to cover multiple categories (scaffolding + verification + deployment).
**Fix:** Split into focused skills that compose. A deployment skill can invoke a verification skill.

## 5. The Trust-the-Doc Skill
**Problem:** Skill references external documentation without verifying. Docs can be wrong or outdated.
**Fix:** Always validate against the real system. Include introspection capabilities.
**Real case:** Doc said field was `x_studio_lot_state`, real field was `state`. 7 errors caught by introspection.

## 6. The No-Gotchas Skill
**Problem:** Skill has instructions but no failure patterns. Claude hits the same errors repeatedly.
**Fix:** Every time Claude fails, add a gotcha. This is the highest-ROI activity for skill maintenance.

## 7. The Hardcoded Skill
**Problem:** Skill has hardcoded IDs, URLs, or credentials that break in different environments.
**Fix:** Use config.json pattern. First run asks user, subsequent runs read config.

## 8. The Prose-Heavy Skill
**Problem:** Instructions are long paragraphs. Claude spends tokens parsing what it needs.
**Fix:** Tables, bullet lists, code blocks. Information density > readability.

## 9. The Silent Skill
**Problem:** Skill does work but doesn't report results in a structured way.
**Fix:** Every skill should produce a clear output: report, table, or structured JSON. The output IS the product.

## 10. The Untriggerable Skill
**Problem:** Description says "helps with X" but doesn't match what users actually ask.
**Fix:** Description should contain the exact phrases users type: "create module", "audit code", "run tests".

## 11. The Immortal Script Skill
**Problem:** Skill stores state in its own directory. State is lost on skill upgrade.
**Fix:** Use `${CLAUDE_PLUGIN_DATA}` for persistent state.

## 12. The Method Actor Skill
**Problem:** Skill has elaborate persona/role-play instructions that consume tokens without adding value.
**Fix:** One-line role. "You are a migration tester." Focus on WHAT, not WHO.

## 13. The Island Skill
**Problem:** Skill duplicates content that already exists in a hub/reference skill. Example: putting engine-patterns inside `odoo-module-creator` when `odoo-version-api-reference` is the centralized hub that ALL skills consume. This creates two sources of truth that drift apart.
**Fix:** Identify the hub skill for the knowledge domain. Put content there, reference it from consumer skills. Hub skills are "Library Reference" category — they exist to be consumed by others.
**Real case:** engine-patterns.md put in module-creator/references/. Needed by auditor, test-generator, and upgrader too. Moved to version-api-reference and referenced by all four.

## 14. The Phantom Reference
**Problem:** Skill references another skill that doesn't exist in the repo. Claude will try to invoke it and fail silently or with a confusing error.
**Fix:** Before adding a skill to delegation tables or "suggest next" sections, verify it exists: `ls skills/<skill-name>/SKILL.md`. If it doesn't exist, don't reference it.
**Real case:** `/odoo-deep-knowledge` referenced in module-creator's delegation table. Skill doesn't exist in the developer-skills repo.

## 15. The Ghost Frontmatter
**Problem:** Skill uses frontmatter fields that Claude Code doesn't recognize (`activation`, `tags`, `globs`, `requires`). They consume tokens on every load but have zero effect.
**Fix:** Only use recognized top-level fields: `name`, `description`, `version`, `allowed-tools`, `compatibility`, `model`. The `description` field IS the trigger — put activation phrases there. Anything custom (author, mcp-server, category, tags, documentation, support) goes NESTED under a `metadata:` block — there it's allowed and ignored safely, not flagged as ghost. `allowed-tools` restricts tool access (e.g. `"Bash(python:*) WebFetch"`); `compatibility` (1-500 chars) declares environment requirements.
**Real case:** `requires: {credentials: [...]}` at top level across 11 skills — moved the info to DEPENDENCIES.md and dropped the field.

## 16. The Misplaced Skill
**Problem:** Skill lives in the wrong suite. A skill used by consultants AND developers is in the developer repo, so consultants don't have it.
**Fix:** Follow the routing decision tree in `ecosystem-governance.md`. If 2+ roles need it → common. Run `detect_duplicates.py` to catch cases where someone created a copy instead of moving the original.

## 17. The Orphan Dependency
**Problem:** Skill depends on another skill from a different suite but doesn't declare it. When the dependency changes, the consumer breaks silently.
**Fix:** Declare all cross-suite dependencies in the suite's `DEPENDENCIES.md`. Run `validate_ecosystem.py` to catch undeclared dependencies.

## 18. The Catalog Drift
**Problem:** Skills are added or renamed but CATALOG.md is never regenerated. Other skills and users discover stale or missing entries.
**Fix:** Run `generate_catalog.py` after any skill add/remove/rename. The catalog is the source of truth for skill discovery.

## 19. The Ignored Skill
**Problem:** The skill triggers and loads, but Claude doesn't follow its instructions. This is a *distinct* failure from under-triggering — the content is in context but gets diluted, buried, or read as optional. Four common causes:
1. **Too verbose** — critical rules drown in prose. Claude can't tell what matters.
2. **Buried** — the key constraint is on line 80, after setup boilerplate.
3. **Ambiguous** — "make sure to validate properly" gives nothing concrete to act on.
4. **Model "laziness"** — Claude takes the cheap path and skips validation/thorough steps.
**Fix:**
- Put critical instructions at the TOP, under a `## Important` / `## Critical` header. Repeat the single most important rule if needed.
- Replace vague directives with a concrete checklist: `CRITICAL: before X, verify: (a) … (b) … (c) …`.
- For any *deterministic* critical check, bundle a **script** instead of describing the check in prose — code executes the same every time; language interpretation drifts (see [[The Trust-the-Doc Skill]] for the validation-over-trust principle).
- For laziness, add explicit encouragement ("take your time, quality over speed, do not skip validation"). **Nuance:** this works better in the *user prompt* than in SKILL.md — note it in the skill's usage examples rather than padding the body.

## 20. The In-Folder README
**Problem:** A `README.md` lives *inside* the skill folder. It's never loaded by the progressive-disclosure system (only SKILL.md + referenced files are), so it's dead weight that duplicates SKILL.md and drifts out of sync.
**Fix:** All skill documentation goes in SKILL.md or `references/`. A repo-level README (one per suite, for human visitors on GitHub) is fine and expected — just not one per skill folder.
**Real case:** Sanity check found `README.md` inside multiple marketing skills (email-campaign-builder, event-planner, seo-manager, linkedin-content-manager), each a stale partial copy of its SKILL.md.

## 21. SSOT Zealotry (consolidating harmless duplication)
**Problem:** Treating *any* repetition as a defect and launching a consolidation project for it. SSOT is **per-purpose**: distinct purposes legitimately keep separate sources of truth (e.g. `coding-standards.md`, `core-philosophy.md`, `skill-designer` each own a different purpose — not duplication).
**Fix:** Triage by IMPACT, not by repetition. Consolidate to an SSOT only what is **load-bearing / causes bugs when it drifts / is costly to sync** — connection config (`ODOO_URL`), runtime contracts, generated indexes, cross-referenced IDs. **Leave alone** harmless duplication of human-facing conventions that agree and don't affect runtime, even if repeated in many files. Repetition ≠ problem.
**Real case:** the dev conventions repeated across 4 CLAUDE.md + 7 CONTRIBUTING looked like duplication, but the only actual drift was a cosmetic git co-author trailer (3 variants) — zero runtime impact. Not worth consolidating. Contrast: `ODOO_URL` hardcoded in 2 configs WAS worth fixing (wrong value breaks the connection).

## 22. Team backlog inside the skill folder
**Problem:** Putting team ideas / "discuss later" notes (a `TODO.md` or `BACKLOG.md`) *inside* a skill folder. It ships and gets cached with the skill, mixes team communication with the skill artifact, and isn't discoverable as a channel. If it's ever linked from SKILL.md REFERENCES it also burns tokens on **every** trigger (the disclosure system auto-reads referenced files).
**Fix:** Team ideas are a **communication channel, not skill documentation**. They go in one `IDEAS.md` at the **suite repo root** (`<root>/ai-claude-<suite>/IDEAS.md`), created on demand, tagged per skill (`- [ ] [skill-name] idea — why (added YYYY-MM-DD)`), and never linked from any SKILL.md. Aggregate across suites with `scripts/todos.py`. Keep the three containers distinct: `IDEAS.md` (team channel) vs the skill's `docs/` (its own multi-phase technical plans) vs `references/` (content the skill needs to run). See maintenance-playbook.md §IDEAS.
**Real case:** the dynapps-brand "canonical CMS asset URLs" idea was first dropped as `dynapps-brand/TODO.md` (wrong container — travels with the skill) and moved to `ai-claude-common/IDEAS.md`.

## 23. The Silent Skip (un gate que se calla lo que no midió)

**Problem:** el checker encuentra un caso que no sabe resolver y hace `continue`. Sale **0 issues** y
todo el mundo lee "verde", cuando en realidad no comprobó nada de ese caso. Es la peor clase de
mentira porque tiene forma de certificado.

**Fix:** lo que se salta se **cuenta y se dice** (`saltadas: N`), y ese contador entra en el informe.
Si el número no es cero, el verde lleva asterisco.

**Real case (2026-08-19, campaña de citas):** `check_citations.py` daba 0 issues con **924 de 7.216
citas saltadas en silencio** — el fichero no resolvía, y sin fichero no hay ancla que exigir. Al
arreglar el resolvedor aparecieron 519 problemas reales de golpe. Nadie había tocado el contenido:
solo dejó de callarse.

**Aplicado a los cuatro checkers (2026-08-20).** La regla se predicaba y solo la cumplía quien la
descubrió. Ahora los cuatro imprimen su contador —también en `--issues`, que es lo que ve el CI— y
lo que salió al encenderlos es la razón de que la regla exista:

| checker | comprueba | NO comprueba |
|---|---:|---|
| `check_citations` | 7.217 citas | **2.601**: 1.021 reconocidas que no son la primera de su frase (`analizar` solo mira una por frase), 771 fuera de backticks, 618 con algo entre el backtick y la ruta (un prefijo de version dentro del propio backtick), 184 exentas, 7 con ruta abreviada |
| `check_version_claims` | 75 afirmaciones | **18.564** líneas que nombran una versión: 9.544 sin palabra de frontera, 4.558 por debajo del major mínimo con árbol, 3.984 que ya contrastan dos versiones |
| `check_disk_refs` | 161 rutas | **52**: 44 plantillas (`odoo{N}`, `odooNN`), 4 segmentos de ejemplo, 4 donde solo se comprueba el primer segmento |
| `validate_ecosystem` | 207 skills | **43**: 27 candidatos descartados por ruido, 11 refs a skills externas que nadie verifica, 4 skills ignoradas por gitignore, 1 nombre derivado |

Ninguno de esos números es un defecto por sí mismo — casi todos los descartes son deliberados y son
lo que hace útil al checker (en `check_version_claims`, los descartes son lo que baja 315 sospechosos
a 24). Lo que no es defendible es que no se vean: **1.021 citas que el propio lector reconoce y que
nadie comprueba** llevaban ahí toda la campaña, y salieron el día que el gate empezó a contar.

**Y el contador siguió pagando (2026-08-20, mismo día).** Al desglosar POR FORMA lo que decía no
comprobar, dos de las tres categorías estaban mal contadas, y detrás de cada error había trabajo real:

- Las «618 con algo entre el backtick y la ruta» eran 240 de prosa mal clasificada —el heurístico
  tomaba el backtick de CIERRE de un símbolo anterior por una apertura, así que `` `_compute_x`
  (models.py:567) `` contaba como código— y 68 citas de verdad con el prefijo de versión dentro del
  backtick, que es el idioma de media `odoo-deep-knowledge`. Contar bien y leer ese prefijo sacó **67
  citas a revisión**, con dos errores de contenido que llevaban meses ahí.
- Y **126 citas de forma perfectamente válida no las entregaba el troceador**: `frases()` exigía un
  `. ` cada 400 caracteres y, cuando no lo había —filas largas de tabla, `_index/cheatsheet.md` entero—
  dejaba un hueco que nadie recorría. Ni ancla, ni línea, ni aviso, y **el contador tampoco las veía**,
  porque las contaba como «reconocidas».

La moraleja no es «contar está bien»: es que **el contador también hay que desglosarlo y comprobarlo**.
Un total agregado tranquiliza; el desglose por forma es lo que delata que la categoría miente.

## 24. The Optional Dependency (fallback silencioso que cambia el veredicto)

**Problem:** el script usa una librería que no está declarada y, si falta, degrada **sin avisar**. El
gate se porta distinto en cada máquina y el que lo escribió nunca ve el modo degradado.

**Fix:** biblioteca estándar, o dependencia declarada y comprobada al arrancar. Si de verdad hace
falta un fallback, **que grite**. Y un test que ejecute el módulo *sin* la librería.

**Real case:** el rango real de un elemento XML se calculaba con **lxml**; en la máquina de quien lo
escribió estaba, y en CI no. Sin lxml el rango colapsaba a una sola línea y el checker aceptaba citas
que en local marcaba. Reescrito con **expat** (stdlib) y con un test que lo corre en un subproceso
con `lxml` bloqueado.

## 25. The Indentation Parser (medir por sangría o regex lo que tiene sintaxis)

**Problem:** deducir estructura contando espacios o casando una regex por línea. Funciona en los
ejemplos y falla en cuanto el código respira: strings multilínea, llaves dentro de comillas,
atributos que se parecen a otros.

**Fix:** usa el parser del lenguaje — `ast` en Python, emparejamiento de **llaves** (saltando
cadenas y comentarios) en JS, un parser XML de verdad. Si no hay parser, di que ese formato **no se
valida**; no finjas.

**Real case:** tres a la vez. `class X` en JS "medía" **una línea**, así que toda cita al cuerpo salía
como error (277 → 86 al cambiar a llaves). En XML, `\bid=` casaba dentro de `t-att-id="group.location.id"`
y dentro de `expr="//div[@id='so_width']"`, y las dos cosas llegaron a escribirse como ancla en el KB.
Y en Python, `ast.alias` tiene `.name`, así que un `from x import foo` daba envolvente `foo` — un
nombre **importado**, que no se declara en ese fichero.
