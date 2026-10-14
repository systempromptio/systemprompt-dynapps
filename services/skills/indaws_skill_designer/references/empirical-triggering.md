# Empirical triggering — medir el disparo real de una skill

`scripts/eval_triggering.py` mide **empíricamente** si una skill dispara para las queries que le
tocan, **con el catálogo federado completo cargado** (las ~141 skills que `claude -p` ve como plugins).
Es el complemento dinámico de `trigger_collisions.py` (que solo mide solape léxico Jaccard).

## Qué lo hace distinto del `run_eval.py` de Anthropic
Anthropic inyecta una skill **sintética** y la mide **aislada** ("¿esta description atrae esta query?").
Aquí lanzamos `claude -p` en el entorno real, donde **todas las skills compiten**, y capturamos **QUÉ
skill gana** (`Skill` tool → `input.skill`). Eso permite medir lo que Anthropic no puede: **colisiones
reales** entre skills del ecosistema (quién le roba queries a quién), no una estimación léxica.

## Cuándo usarlo (y cuándo NO)
- **SÍ:** dirimir una colisión que `trigger_collisions.py` marcó como sospechosa; validar que una
  description reescrita mejora recall sin robar a vecinas; auditar una skill "que nunca se dispara".
- **NO:** en cada edición, ni en CI. Cada query×run es **una sesión Claude real** (tokens + tiempo).
  Empieza pequeño (8–12 queries, `--runs 1`) y sube `--runs` solo para lo dudoso.

## Uso
```bash
# 1. (opcional) dibujar un eval-set y REVISARLO a mano
scripts/eval_triggering.py --skill dev-task-writer --generate 16 -o dtw.evals.json

# 2. medir
scripts/eval_triggering.py --skill dev-task-writer --eval-set dtw.evals.json --runs 3 --workers 6
```
Eval-set = `[{"query": "...", "should_trigger": true|false}, ...]`. Los negativos valiosos son
**near-misses**: comparten vocabulario pero necesitan OTRA skill (no triviales).

## Cómo leer las métricas
- **recall** — % de queries propias (should_trigger) donde la skill gana ≥50% de sus runs. Bajo = la
  description no atrae lo suyo, o una vecina se lo lleva (mira el steal map).
- **specificity** — % de near-misses ajenos donde la skill NO dispara. Bajo = skill "greedy", roba.
- **precision** — de las queries donde dispara, cuántas eran realmente suyas.
- **miss rate** — queries propias donde no dispara **nada** (ni ella ni otra): description muda o query
  que Claude resuelve directo sin skill.
- **steal_map** — ⚔️ cuando pierde una query suya, qué skill se la lleva. **Es la colisión empírica.**
  Si `X` aparece robando queries de `Y`, sus descriptions se solapan de verdad → desambiguar triggers.

## Cómo funciona / garantías
- `--permission-mode plan` (sin escrituras) + cwd temporal + **el proceso se mata al primer tool_use**
  → la skill detectada **no llega a ejecutarse** (medimos disparo, no efecto).
- Heurística "first-tool decides": si el primer tool que usa Claude no es `Skill`, cuenta como no-disparo
  (barato). Una skill bien descrita dispara up-front; si Claude explora antes, se contabiliza como miss.
- Runtime namespacea las skills de plugin (`developer-skills:x`); el script normaliza al nombre bare.
- Modelo: el configurado en la sesión (para que el disparo coincida con la experiencia real).

## Punto ciego MEDIDO: las skills que actúan sobre el repo (2026-08-08)

**`first-tool decides` no discrimina en el territorio «haz algo con este código».** Midiendo con 17
queries una skill de planificación **en desarrollo local, nunca publicada** (no la busques en
ninguna suite): **recall 0%, miss rate 100%**. El control, con `dev-plan` —skill establecida,
instalada y con esos triggers en su description— sobre el **mismo** eval-set: **también 0% y
100%**. 51 sesiones `claude -p` gastadas, y la conclusión es sobre el instrumento.

La causa: ante «monta un plan por fases para este módulo», Claude explora primero (`Bash`, `Read`)
y la heurística lo cuenta como miss. En cambio `llm-wiki-builder` sí disparó up-front con «monta
un wiki para acumular papers» — una petición que no tiene repo que explorar.

Consecuencias prácticas:

- **Corre siempre un control** con una skill establecida del mismo territorio antes de creerte un
  recall bajo. Sin control, un 0% se lee como «la description es mala» cuando puede ser el harness.
- Para skills de acción-sobre-repo, dirime con **`trigger_collisions.py`** (proxy léxico, 0 tokens,
  determinista) y ojo con su API: `frontmatter_desc()` toma **contenido**, no ruta — pasarle la
  ruta devuelve `""` en silencio y produce un «sin colisiones» que es un falso verde.
- Ese proxy sí encontró lo que importaba en el caso real: las dos no colisionaban **porque sus
  triggers estaban en idiomas distintos**, así que la vieja seguía ganando el territorio inglés sin
  disputa. Eso una medición de recall no lo habría dicho.

## Segundo punto ciego MEDIDO: el SUELO del arnés (2026-09-04)

**Un 0% de recall no demuestra que la description sea mala: puede ser el suelo del instrumento.**
Medido en 63 sesiones sobre tres skills, mismo entorno, mismo día:

| skill | queries propias | dispara |
| --- | --- | --- |
| `odoo-verify-green` | 6 | **5 de 18 runs** (2 queries) |
| `odoo-i18n-manager` | 6 | **0 de 18** |
| `odoo-traceback-doctor` (control) | 2 | **0 de 4** |

El control es lo que importa: *"me sale este traceback de Odoo y no sé leerlo"* es una consulta de
manual para `odoo-traceback-doctor`, y no dispara ninguna vez. **La tasa base de activación en este
arnés está pegada a cero**, así que las cifras absolutas no separan "description sorda" de
"aquí casi nada dispara". No se puede concluir "reescribe la description" a partir de un 0%.

**Y el arnés tiene VARIANZA entre corridas: 3 runs no bastan para comparar variantes.** De una
observación aislada salió la hipótesis "las frases situacionales llegan y las escuetas no"
(*"la autora reporta 0 fallos en su entorno"* 3/3 frente a *"dice que los tests pasan"* 0/3).
**Replicada con un diseño pareado de 6 pares —misma intención, versión escueta y versión
situacional, 42 sesiones— NO se confirma:** 28% las escuetas contra 44% las situacionales, con
**un par en dirección contraria** (`"está en verde"` 3/3 escueta contra 2/3 situacional) y dos
empates a cero. Y la misma query cambia de resultado entre días: *"…0 fallos en su entorno…"* dio
3/3 un día y 1/3 al siguiente. Una diferencia de 3 disparos sobre 18 es ruido, no señal.

**Lo que SÍ se repite entre corridas** es que las frases de la lista `Triggers:` no garantizan
nada: de las 4 frases de `odoo-verify-green` probadas *literales*, tres midieron **0/3**
(`"los tests pasan"`, `"0 fallos"`, `"ya está hecho"`) y una 3/3 (`"está en verde"`). Y una query
que contiene *verbatim* `"traceback de Odoo"` no carga `odoo-traceback-doctor`. Escribir la frase
en la lista no la hace disparar.

**Corolario para `optimize_description.py`:** si el suelo es cero, el optimizador no tiene señal
que subir. Medido el mismo día sobre `odoo-i18n-manager`: 4 iteraciones, train clavado en 2/6 y
test en 2/4, y devuelve **la description original sin cambios**. Ese resultado negativo es
información — dice "el problema no está en el texto"—, no un fallo del script.

**Antes de sacar conclusiones de un recall bajo, corre un CONTROL**: una skill distinta con una
query de manual. Si el control también da 0, lo que estás midiendo es el arnés.

## Optimización de description (`optimize_description.py`)
Bucle data-driven que **propone** (no escribe) una description mejor:
```bash
scripts/optimize_description.py --skill dev-task-writer --eval-set dtw.evals.json --max-iters 4 --runs 2
scripts/optimize_description.py --self-test    # valida el plumbing (stubbea la medición)
```
- **Split train/test estratificado** (`--holdout 0.4`, `--seed 42`): mejora SOLO con train, elige el
  ganador por score de **test** → anti-overfit. Para si train pasa entero o a las `--max-iters`.
- **Anti-repetición:** al modelo se le pasan los intentos previos con "no los repitas".
- **Límite duro 1024 chars** con reescritura de seguridad si se pasa.
- **Método:** inyecta la description candidata como command sintético en `.claude/commands/` y mide su
  disparo contra el catálogo real (el cache de plugins es copia → editar el repo no lo cambiaría).
- **Confundido:** la skill real (description actual) también compite; si acapara una query propia,
  cuenta como no-disparo del candidato. Lee el "best" como pull relativo. Ideal para skills nuevas o
  cuya description vas a cambiar. NO reescribe el SKILL.md: revisa y aplica a mano.

## Valor incremental — ¿se gana la skill su sitio? (`eval_value.py`)
```bash
scripts/eval_value.py --skill odoo-traceback-doctor --tasks tasks.json --runs 3   # --self-test
```
Corre cada tarea CON el catálogo y SIN skills (`--append-system-prompt` que prohíbe la Skill tool),
gradúa ambas con un grader LLM y compara **pass-rate + coste** ($/tokens/tiempo), con mean ± stddev.
- **Δ pass-rate ≈ 0 / negativo** → sin valor medible: candidata a fusión/deprecación (más aún si añade coste).
- **Aserciones no-discriminantes** (pasan CON y SIN skill) → no miden el valor de la skill; las marca
  (esto cubre "auditar la calidad de los tests"). `tasks.json = [{"prompt","assertions":[...]}]`.
- Caveat: `plan-mode` (sin escrituras) infravalora skills cuyo valor ES escribir → `--danger-full`.

## Gate de revamp — juez A/B ciego (`blind_compare.py`)
```bash
scripts/blind_compare.py --old skills/x/SKILL.md --new /ruta/revamp/SKILL.md   # --kind output | --self-test
```
Un juez puntúa dos artefactos SIN saber cuál es cuál (evita sesgo de autoría), elige ganador y luego
des-ciega. **Regla:** el revamp no se mergea si no bate al original. Barato (1 llamada). No ejecuta la
skill (el cache de plugins lo impediría): compara los SKILL.md o dos outputs que le des.

## Dashboard de gobernanza (`dashboard.py`)
```bash
scripts/dashboard.py $SKILLS_ROOT            # → reports/ecosystem-health.html
```
HTML autónomo (tema claro/oscuro, marca): salud %, PASS por suite, fallos duros y warnings. Publícalo
con la tool **Artifact** (`file_path` = ese HTML). Determinista/barato (reusa verify_suite + validate_ecosystem).

## Packaging (`package_skill.py`)
```bash
scripts/package_skill.py skills/x --validate-only     # valida frontmatter (dureza Anthropic)
scripts/package_skill.py skills/x -o x.skill          # valida y zipea
```
Valida name kebab-case ≤64, description ≤1024 sin `<>`, claves en allowlist; luego ZIP excluyendo
`__pycache__`/`node_modules`/`.git`/`reports/*.html`. Distribución puntual (el modelo normal es git-plugin).

## Visor de resultados (`results_viewer.py`)
Convierte el JSON de cualquiera de las 3 herramientas (`eval_triggering`, `optimize_description`,
`eval_value`) en un artifact HTML navegable (autodetecta el tipo):
```bash
scripts/results_viewer.py resultado.json -o resultado.html   # → publicar con la tool Artifact
```
Muestra: tiles de métricas, steal-map como barras, trayectoria train/test de la optimización, y las
aserciones no-discriminantes. Tema claro/oscuro, autónomo.

## Ejemplo real (2026-07) — `odoo-traceback-doctor`
Corrida real (`optimize_description`, 8 queries, runs 1, 2 iters): la description **no pasa de 2/4**
train ni test ni optimizando. Dos causas empíricas: (1) la skill real instalada (desc vieja) compite
con el candidato sintético; (2) Opus **resuelve muchos tracebacks directo, sin cargar skill**. Lección:
no toda skill que "encaja" dispara — a veces el problema no es el wording sino que el modelo se basta.
Render del resultado en `reports/example-optimize-odoo-traceback-doctor.html` (visor).

## Gotcha observado
Skills muy capaces de responderse "en directo" (p.ej. diagnóstico de un traceback simple) pueden **no
dispararse** aunque la description encaje: Claude resuelve sin skill. Es señal real, no bug — o la
description no es lo bastante "pushy", o la tarea es demasiado trivial para justificar cargar la skill.
