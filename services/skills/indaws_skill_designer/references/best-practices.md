# Best Practices for Skill Design

## Structure

### Progressive Disclosure (saves tokens)
SKILL.md is the INDEX, not the encyclopedia. It tells Claude what exists and when to read it.
Details go in references/, examples in examples/, scripts in scripts/.
```
skill/
├── SKILL.md              # Index: role, workflow, key rules, pointers to references
├── references/           # Deep content Claude reads on demand
│   ├── api.md            # Function signatures, field types
│   └── gotchas.md        # Failure patterns (highest signal)
├── scripts/              # Code Claude can execute or compose
├── templates/            # Output templates Claude copies
├── examples/             # Input/output examples
└── assets/               # Static files (logos, fonts, data)
```

### SKILL.md Should Contain
1. **Frontmatter:** description (trigger-oriented — activation phrases go INSIDE description), plus only recognized fields (`name`, `version`, `allowed-tools`, `compatibility`). No `tags`/`activation`/`globs` — they have no effect.
2. **Role:** One sentence — who is Claude when this skill is active
3. **Workflow:** Numbered steps with [STOP] points where user input is needed
4. **Key rules:** NEVER/ALWAYS lists — only the critical ones
5. **References section:** File list with one-line description of each
6. **Gotchas section:** Top 5-10 failure patterns with corrections

### SKILL.md Should NOT Contain
- Full API documentation (put in references/)
- Long code examples (put in examples/)
- Implementation details of scripts (they're self-documenting)
- Information Claude already knows (general coding, well-known libraries)
- Rigid step-by-step instructions that don't allow adaptation

## Framing: Problem-First vs Tool-First

Before writing, decide which way the skill is framed — it shapes the whole structure (Home Depot analogy):
- **Problem-first:** user describes an *outcome* ("set up a project workspace") and the skill orchestrates the right calls in the right order. The user never names the tools. Most workflow/automation skills are this.
- **Tool-first:** user already has a tool/connector and the skill teaches Claude the workflows and best practices for it ("I have Notion connected → how do I use it well"). Most MCP-enhancement skills are this.

Most skills lean one way. Knowing which tells you whether the SKILL.md leads with a goal-driven workflow or with tool-usage guidance. See `workflow-patterns.md` for the concrete in-skill structures.

## Instruction Adherence (making the skill actually be followed)

Triggering is only half the battle — a skill can load and still be ignored (see anti-pattern [[#19 The Ignored Skill]]). To maximize adherence:
- **Critical first.** Put the non-negotiable rules at the top under `## Important`/`## Critical`, not buried after setup prose.
- **Concrete over vague.** `CRITICAL: before create_X verify (a) name non-empty (b) date not in past` beats "validate properly".
- **Script the deterministic checks.** If a critical validation can be expressed as code, bundle a script and call it — code is deterministic, prose interpretation drifts.
- **Encourage against laziness, in the right place.** "Take your time, quality over speed, don't skip validation" helps — but it lands harder in the *user prompt* than in SKILL.md. Surface it via the skill's usage examples rather than bloating the body.

## Description Field

The description is a TRIGGER, not a summary. Claude scans it to decide "is there a skill for this request?"

Bad: "This skill helps with Odoo module development"
Good: "Create new Odoo modules (v14-19) following Indaws coding standards. Activates for module scaffolding, models, views, security."

Include: action verbs, specific nouns, version numbers, trigger phrases.

## Gotchas Section

This is the HIGHEST VALUE content in any skill. Build it from real failures.

Format:
```markdown
## GOTCHAS
1. **Field X was renamed in v18** — Use `new_name` not `old_name`. Claude defaults to old name.
2. **Method Y is private** — Cannot call via RPC. Use wizard Z instead.
3. **Model A requires field B** — Omitting it causes silent failure, not an error.
```

Update gotchas every time Claude hits a new edge case. A skill with 20 good gotchas is worth more than one with 200 lines of instructions.

## Scripts

Give Claude CODE to compose, not PROSE to interpret.

```python
# Good: a helper library Claude can import/use
def search_records(model, domain, fields):
    ...

# Bad: instructions telling Claude how to make HTTP requests to Odoo
# "To search records, make a POST request to /jsonrpc with..."
```

Scripts should be:
- Self-contained (no external deps when possible)
- Well-named (the filename IS the documentation)
- Composable (Claude chains them together)

## Config & Setup

If a skill needs user-specific config (credentials, preferences), use a config.json pattern:
1. On first use, check if config exists
2. If not, ask user for required values
3. Store in config.json in the skill directory
4. Read on subsequent uses

For structured questions, use AskUserQuestion tool.

## Memory & State

Some skills benefit from remembering previous executions:
- **Append-only log:** `executions.log` with timestamps and outcomes
- **JSON state:** `state.json` with last run data
- **SQLite:** For complex queries over historical data

Store in `${CLAUDE_PLUGIN_DATA}` (survives skill upgrades) not in the skill directory.

## Hooks vs Instructions: Deterministic vs Advisory

**Key distinction:** CLAUDE.md / SKILL.md instructions are *advisory* — Claude follows them most of the time but can drift. Hooks are *deterministic* — they execute shell commands that block or transform actions unconditionally.

| Mechanism | Guarantee | Use when |
|-----------|-----------|----------|
| SKILL.md NEVER rule | ~95% adherence | Style, preferences, soft constraints |
| Hook (PreToolUse) | 100% — blocks the tool call | Safety-critical: no writes to prod, no force-push, no secrets in commits |
| Hook (PostToolUse) | 100% — runs after every tool call | Auto-lint after edits, auto-format, log every action |

**Rule of thumb:** If violating a rule causes data loss or security risk, make it a hook. If it causes ugly code, keep it as a SKILL.md instruction.

### Hooks in skills
Skills can register hooks that activate only when the skill is called:
- `/careful` — Block destructive operations when touching prod
- `/freeze` — Block edits outside a specific directory
- `/verbose` — Log every tool call for debugging

Use for opinionated behavior you don't want running ALL the time.

## Token Optimization

1. **Compact JSON over verbose JSON** — Single-line entries where structure is repetitive
2. **CSS class names can be short** — In HTML output, `.st-p` beats `.status-pass`
3. **Don't repeat defaults** — If 90% of tests are type=verify priority=A, make those defaults
4. **References on demand** — Claude only reads what it needs from references/
5. **Gotchas > paragraphs** — A bulleted gotcha conveys more per token than an explanatory paragraph

## Verification subagents (factual / capability claims)

When a skill uses subagents to assert facts about a system ("can Odoo do X?", "does method Y exist?"), their output is NOT trustworthy as-is — **review against source before committing**. Two failure modes, asymmetric:

- **False-YES (hallucination)** — cites a method/field/behaviour that doesn't exist. *Caught* by a deterministic gate (e.g. `cite_audit --methods`: does the symbol exist in source?).
- **False-NO (undervaluation)** — answers "no / would need development" without checking all candidates (EE modules, bridges, localizations). **NOT caught by any gate**, and the dominant + costly error (~10 wrong verdicts in one capabilities pass: intercompany / commissions / rental / physical-inventory said NO but are EE-native). Root cause: an agent can't *recall* ~90 candidate modules from memory.

Fixes (validated, by leverage):
1. **Feed the closed candidate inventory first.** Don't ask open "can it do X?" — force the agent to enumerate the full module set (`ls` core + EE + `*bridge*` + l10n) as step 0 and derive answers from THAT list; a "NO" is valid only if no listed module provides it. *Validated:* an inventory-fed agent fixed the 3 false-NOs an unguided agent had made on `sale`.
2. **Adversarially re-check only the NO/DESARROLLO verdicts** (the YES is gate-covered) — a skeptic agent tries to refute each NO.
3. Require negative evidence inside a NO ("grepped modules X,Y,Z, term absent") so it's auditable.

---

## Citas a fuente Odoo: el formato es **ancla + línea**, y lo exige el `doctor`

Una cita `fichero:línea` es una coordenada dentro de un fichero **que edita otro**. No tiene ninguna
garantía de estabilidad: upstream mueve código y la cita queda apuntando a otra cosa.

**Medido en este monorepo (2026-08-18):**

- **9.222 citas** `fichero:línea` en las tres suites principales, y **6.040 sin ancla**.
- De 6 citas del KB muestreadas a mano, **3 habían derivado**.
- Una cita de v18 en una skill que cubre v17-v19 apuntaba, en v19, a **código sin relación** — con lo
  que el lector concluye que la skill miente.

### El formato

```
(`account/models/account_move_line.py:3310-3342`, `def _prepare_analytic_distribution_line`, v18)
(`odoo/models.py:7522`, `def _transient_clean_old_rows`, CE18 @99f5a33c64)
(`odoo/addons/base/models/ir_ui_view.py:143`, `class View`)
```

**Tres partes obligatorias y una recomendada**, y cada una cierra un fallo distinto — los cuatro
observados en este monorepo, no hipotéticos:

| Parte | Qué fallo cierra | Lo que pasó sin ella |
|---|---|---|
| **Ruta completa** (con `/`) | el fichero resuelto es **otro** | `models.py` a secas resuelve a `web_map/models/models.py`; `ir_ui_view.py`, a otro distinto. **Los dos ejemplos de esta misma página murieron así** |
| **Línea o rango** | salto instantáneo | sin ella hay que grepear siempre |
| **Símbolo** (`def X`, `class X`, campo) | la línea deriva y nadie la reconstruye | `_compute_title` citado en `:217-219` estaba en `:232` |
| **Versión o `@sha`** *(recomendado)* | el hecho es correcto **en otra versión** | `is_frontend` en `:386` es v18; en v19 esa línea es `allow_redirect = (...)`, y el lector concluye que la skill miente |

> **Una cita con `@sha` no necesita sello de «verificado»**: el sha ancla el árbol **y** el momento.

### Y no abrevies la segunda cita

```
MAL   (`website/models/theme_models.py:217`, `Theme`; y `customize_show`, que esta en `:72-73`)
BIEN  (`website/models/theme_models.py:217`, `class Theme`) y
      (`website/models/theme_models.py:72-73`, `customize_show`)
```

La abreviatura parece ahorro y cuesta caro: **11 de los 13 primeros casos que el checker marcó eran
esto** — dos citas en una línea, la segunda abreviada, y el emparejamiento símbolo↔cita se vuelve
ambiguo para cualquiera que lo lea, humano o script.

- **El ancla (el símbolo) es lo que hace la cita reconstruible.** Si la línea deriva, un `grep` la
  recupera. Sin ancla, la cita solo se puede creer o descartar.
- **La línea se queda** porque hace el salto instantáneo. Es un atajo, no la verdad.
- **El ancla va PEGADA a la cita**, con un conector (`, dentro de`, `, `` `def X` ``, `, en`) y nada
  más. No vale mencionar el símbolo en cualquier parte de la frase: se probó y daba **9 falsos
  positivos de 9** — cogía un símbolo que sí está en el fichero pero **no es el del sitio citado**.
- **Si el símbolo vive en una clase, cualifícalo**: `` `configmanager.__init__` ``, no `` `def
  __init__` ``. Un `__init__` a secas no reconstruye nada — hay cientos — y ese es todo el trabajo
  del ancla. `--fix-rutas` las deriva ya cualificadas, y el lector acepta el punto.
- **Y di la versión** cuando el hecho no sea igual en todas: `odoo18/…`, `v18 :386 · v19 :375`, o
  `@sha`. El `@sha` es el atajo bueno: ancla la versión **y** el momento a la vez, así que **una cita
  con `@sha` no necesita sello de «verificado»**.
- **Y si la escribes en el prefijo, el prefijo MANDA.** `odoo16/odoo/fields.py:312`, `Field.precompute` solo puede
  resolver contra v16. Al principio el checker solo reconocía 14-19, así que un `odoo12/...` no se
  veía como versión y caía por la regla del sufijo **en la línea equivalente de v18** — apuntando a
  otra cosa y pasando el gate. Si no hay árbol de esa versión, la cita se queda **sin resolver**, que
  es la verdad.
- **`sangría` ya no**: el ancla de `.js` se mide por **llaves**, la de `.xml` con un parser
  (`t-name` incluido) y la de `.scss` por llaves y `;`. Ver anti-patrón 25.
- **En `.scss` el ancla es la variable, el `@mixin`/`@function` o el selector**, y se escribe como
  se declara: `` `$o-theme-font-configs` ``, con `$` y guiones. El patrón laxo de nombres solo se
  aplica si el fichero citado es `.scss` — ampliarlo a todos convertiría cualquier palabra con guion
  pegada a una cita `.py` en un fallo nuevo de una cita que hoy está bien. El rango de una variable
  llega hasta su `;`, así que citar una línea de dentro de un mapa de 34 líneas es correcto y no sale
  como deriva. Y una cita a la cabecera de comentarios de un `primary_variables.scss` **no tiene
  ancla posible**: queda exenta, igual que un `__manifest__` o un import.
  *Esto era el único agujero declarado del gate: mientras no hubo parser, `envolvente_de` devolvía
  None para todo `.scss`, así que las 31 citas SCSS del monorepo se contaban como exentas y una cita
  inventada pasaba. Al darle parser, 21 pedían ancla y **5 estaban mal**: dos ponían la línea 287 de
  `website/static/src/scss/secondary_variables.scss`, `$o-theme-font` para `$o-theme-headings-font`,
  que en v18 y v19 está una línea más abajo, y tres mandaban a las quince primeras líneas de un
  `web_editor/static/src/scss/secondary_variables.scss`, `$o-color-palette` cuyo `map-merge` está en
  la 23 — una de ellas además al módulo equivocado (`website/` por `web_editor/`). Ninguna de las
  cinco era detectable mientras la categoría fuese «exenta».*

### El corrector: `--fix-rutas`

Completa la ruta corta y le deriva el ancla **leyendo el fichero real** (`ast` en `.py`, sangría en
`.js`, `xml_id` en `.xml`). No adivina: si el nombre resuelve a más de un fichero, la deja.

```bash
python3 scripts/check_citations.py --fix-rutas --dry-run --scope <suite>
python3 scripts/check_citations.py --fix-rutas --pista-dominio --scope ai-claude-odoo-knowledge
```

- **`--pista-dominio`** usa la carpeta de la página como módulo. Solo sirve donde el layout lo
  declara (`references/<capa>/<dominio>/` en el KB) y solo si deja **un** candidato. Sube la
  resolución del **10% al 29%**; el resto son `res_users.py` y compañía, que existen en **79
  módulos**, y ahí elegir es adivinar.
- **La línea se corrige entera o no se toca.** Si tiene dos citas y una sigue ambigua, no se toca
  ninguna: al entrar la línea en el diff el gate la mira completa y reportaría la que quedó mal.
- **Revisa el diff.** Si la cita ya había derivado, el ancla derivada es el símbolo que ocupa **hoy**
  esa línea, que puede no ser el que el autor quería.
- **`--por-simbolo`** desambigua por **contenido**: gana el candidato que tiene, en esa línea, un
  símbolo que la frase nombra — y solo si es único. No es una probabilidad, es una comprobación. Si
  empatan dos, se deja: **un empate es la señal de que hace falta una persona.**
- **Los ficheros AUTO-GENERADOS quedan exentos**: se corrige su fuente, no el generado.

- **`--simbolo-en-fichero`** relaja el criterio: basta que el símbolo esté en el fichero, no en la
  línea citada. **Rinde casi nada, y eso es el hallazgo**: sin guarda da 222 correcciones en el KB y
  **218 escriben un ancla que contradice la frase** — cierta para esa línea, y por eso pasa el gate.
  Con la guarda (la envolvente tiene que ser uno de los símbolos que la frase nombra) quedan **4**.
  Los otros 218 son una **lista de revisión humana**, no una corrección.

### Lo que NO se puede resolver a máquina, medido

Sobre **150 citas ambiguas al azar** de las tres suites:

| | |
|---|---|
| **33%** | la frase nombra un símbolo y **un solo candidato** lo tiene en esa línea → `--por-simbolo` |
| **35%** | la frase **no nombra ningún símbolo**: no hay con qué discriminar |
| **29%** | **ninguno casa**: o la cita ya derivó, o el símbolo no es el de esa línea |
| **2,7%** | el fichero **no está en los árboles indexados** (falta OCA y v19) |

Los dos bloques del medio (**64%**) necesitan a quien escribió la frase. un `sale_order.py` a secas puede
ser cualquiera de **130** ficheros y un `account_move.py` de **304**; elegir por probabilidad es
exactamente lo que esta regla existe para impedir.

### Qué pasa si no lo haces

`doctor.py` llama a `check_citations.py --issues --only-new` y **el `pre-push` bloquea**. Solo mira
**las líneas que tu rama añade**: la deuda heredada no molesta, pero **no se puede meter una cita
nueva que el siguiente no pueda comprobar**.

### Corregir citas derivadas

```bash
python3 .../scripts/check_citations.py --fix --dry-run --scope ai-claude-developer
```

**Siempre `--dry-run` primero.** El corrector es deliberadamente conservador y aun así **no todo lo
que propone es correcto**: citar el `@api.onchange` de un método, o una línea del cuerpo, **son citas
buenas**, y hubo que enseñarle a no tocarlas. Si la cita y el ancla apuntan a métodos distintos, no
lo arregla nadie automáticamente — hay que leer la frase.

## Escribir un gate que no mienta (medido en la campaña de citas, 2026-08-19/20)

Un `check_*.py` que bloquea el push es una afirmación: *"esto está comprobado"*. Cinco reglas que
salieron de arreglar uno con 7.216 casos, y ninguna es teórica.

1. **Lo que se salta, se cuenta.** Ver anti-patrón 23. 924 de 7.216 citas se saltaban en silencio y
   el gate decía 0 issues. Un contador de saltadas habría destapado el agujero el primer día.
2. **Presupuesto de tiempo, explícito.** Indexar tres majors más de Odoo llevó el barrido de **50 s a
   3 min 08 s**. Un gate lento se salta con `--no-verify` —lo dice el comentario del propio hook— así
   que se descartó y se resolvió con **ruta directa + un glob acotado** para los casos raros: mismo
   resultado, 49 s. Antes de indexar algo, pregunta cuántos ficheros son.
3. **La evidencia va DELANTE de escribir, no en la auditoría de después.** Tres veces autocompleté
   metadatos "porque el fichero lo confirma" y tres veces metí basura: 218 de 222 anclas contradecían
   la frase; luego 21 de 51; luego 7 módulos elegidos por parecido de nombre. Lo que funcionó fue una
   guarda previa: **escribe solo si algún símbolo que la frase nombra cae DENTRO del rango del ancla**
   —o si la frase no nombra ninguno, y entonces no hay nada que contradecir—. Con tokens de 4+
   caracteres y por palabra completa, que un `orm` no case dentro de `format`.
4. **Dos decisiones distintas no se automatizan juntas.** Mover una cita a la declaración de al lado
   se puede comprobar contra la fuente; decidir que esa declaración es *de lo que habla la frase*, no.
   Cuando las junté, el ancla acabó en `_logger` en una frase sobre `studio.approval.rule`.
5. **Una afirmación sin metadato es honesta; una con el metadato equivocado pasa el gate diciendo una
   mentira.** Ante la duda, exenta. Es la regla que decidió las 219 citas que hoy no llevan ancla:
   la línea es una clave de `__manifest__`, un `import` o una etiqueta XML sin `id`, y ahí **no existe
   ancla posible**. Inventarla habría subido el contador de verdes y bajado la fiabilidad.

**Y dos que no son sobre el gate sino sobre trabajar con él:**

- **Nunca midas con la herramienta de otra rama.** Un `git checkout` de otra sesión dejó el árbol con
  el checker viejo: decía `sin ancla: 56` y 382 tests, cuando con el bueno eran **0 y 389**. Fija la
  rama (o el sha) del checker antes de medir, y si el árbol se mueve bajo los pies, **descarta la
  medida** en vez de interpretarla.
- **Sube la versión justo antes del push, no al abrir la rama.** Con varias ramas vivas, el número que
  elegiste hace media hora ya no es mayor que el de `master` y `skill-guard` te tumba. Pasó tres veces
  el mismo día; `skill_guard.py --fix` hace el bump solo.

**Y la prueba de que esto no es opinión:** el script llevaba **nueve parches en un día y cero tests**.
El banco que se le puso (34 casos herméticos, 389 en la suite) encontró el fallback silencioso de lxml
**el mismo día**. Cualquier script que bloquee un push necesita banco propio, hermético —ficheros
fabricados en `tmp_path`, sin depender del parque de Odoo— porque en CI ese parque no está.
