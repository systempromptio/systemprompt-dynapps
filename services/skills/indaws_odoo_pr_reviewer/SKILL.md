---
name: indaws-odoo-pr-reviewer
version: 1.2.2
effort: high
model: opus
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep, AskUserQuestion, WebFetch]
description: >-
  Revisa el PR de una tarea Odoo y decide merge o correcciones: contrato del arquitecto, tests en verde, fix verificado contra la fuente. Usala cuando revises o cierres un PR. También bug-fixes sin contrato (modo source-verified con falsación del test); cierra el bucle de odoo-solution-architect.

---

# odoo-pr-reviewer

Somos el arquitecto que diseñó la tarea; ahora **revisamos la entrega del dev en su PR**. La pregunta no
es "¿el código está bien?" en abstracto, sino **¿respeta el contrato que dimos y cumple los criterios?**.
Salida binaria: **merge** o **correcciones**. Orquesta el review genérico y añade **lo nuestro** (contrato,
tests, Docker, revisión manual). Par de `odoo-solution-architect`: él diseña, este cierra.

## Entrada

El **PR de GitHub** (URL o rama) y su `project.task`. Y el **MD de la tarea** de
`odoo-solution-architect` **si existe** (planos, pseudocódigo, criterios, contrato-fijo-vs-libertad,
DoR/DoD): es lo que define el **modo** (Step 0).

## Workflow

> **Routing**: la decisión (contrato/tests/manual) = **opus**; montaje Docker y review genérico = **sonnet** subagent. Conexión a Odoo (Step 7) por `references/shared/odoo-connection.md`.

### Step 0 — Reunir contexto y fijar MODO [pregunta: datos] (definición única — Steps 3, 7 y GOTCHA 1 la referencian)

Localizar el PR (rama, commits, ficheros tocados), buscar el MD de la tarea y **fijar el modo**;
**nunca STOP en seco**:

- **Modo CONTRATO** (hay MD): review completo, con el Step 3 como gate duro.
- **Modo BUG-FIX** (no hay MD; el caso más común de PR suelto): **se salta el Step 3** y en su lugar va
  **review source-verified** — el fix calca fielmente el patrón de core que dice replicar
  (`base_automation`, `mail.thread`…) y no rompe los otros caminos. **Los demás gates, intactos.**

Si el PR parece exigir contrato (tarea grande) pero el MD no aparece → pedirlo antes de seguir. Un bug-fix pequeño no necesita contrato: se revisa en modo bug-fix.

Se cierran además, y **se dejan escritas en el veredicto**: **qué toca** (`gh pr diff <n> --name-only` →
módulos, y si es **Core** o **cliente**: cambia el montaje entero), **versión Odoo** (rama base del PR) y
**estrategia de datos**, que se pregunta y se registra con su porqué (`stops-and-questions.md` §3.1).

### Step 1 — Montar la instancia efímera de review (delegar) [gate duro]

Cargar `references/review-instance.md` (montaje y destrucción) y `references/docker-verification.md` (qué
comprobar). No nos fiamos de CI, y **no se toca nada de lo que tengas en marcha**: worktree del PR → pinneo
de los demás repos a su rama estable → instancia efímera junto al repo (slot y puertos propios, mounts
desviados) → datos según el Step 0, neutralizados si son del cliente.

**NUNCA `git checkout` en el árbol montado** (gotchas 6 y 7). Anotar la ref de cada repo montado en el
veredicto. Si **no instala/actualiza en limpio → correcciones inmediatas**.

### Step 2 — Review genérico (delegar)

`/review` sobre el diff (correctness, simplificación, eficiencia), `quality-gate` (5 gates + ISO si
aplica) y `git-review-pr` para el manejo del PR. Recoger findings; **NO decidir aún**.

### Step 3 — Verificación de CONTRATO (lo nuestro) [gate duro · solo modo CONTRATO]

> **Modo bug-fix (sin MD):** saltar este step y hacer review source-verified (ver Step 0);
> luego seguir en Step 4.

Cargar `references/contract-review-checklist.md`. Contra el MD: firmas/campos `# CONTRACT` intactos, fronteras y deps sin cambios no acordados, pseudocódigo→código (mismo QUÉ), integración (`super()`, eventos) respetada. Si el dev cambió el contrato con razón → vuelve al arquitecto, no se aprueba por la vía rápida.

### Step 4 — Verificación de TESTS en el Docker (innegociable) [gate duro]

Cargar `references/test-verification.md`. Ejecutados **en el Docker del Step 1**, no se cree el CI: unit por cada criterio (contrato) o test de regresión (bug-fix) + e2e del caso de negocio (`odoo-flow-tester`), verde REAL con captura. **Falsación** (innegociable en bug-fix): revertir SOLO el fix y confirmar que el test FALLA. Criterio sin test = correcciones.

### Step 5 — Verificación de IMPACTO Odoo (en el Docker) [aplica en los DOS modos]

Cargar `references/impact-checklist.md`. **Modo contrato**: el checklist que el arquitecto marcó como
APLICA, comprobado en el Docker, no leído. **Modo bug-fix** (sin MD, sin checklist): no se salta — hay un
mínimo propio cuyo primer ítem es el **radio de explosión** del módulo tocado (dependientes transitivos,
cuántos son `auto_install`, qué proyectos del host montan el repo). En Core eso puede ser >130 módulos y una docena de proyectos por major — **mídelo, no lo cites**: los números han derivado dos veces (hoy 162 manifests, 76 `auto_install`, 26 proyectos en tres majores; comandos en `references/impact-checklist.md`). Ese número, medido, va al feedback del PR.

### Step 6 — Disparadores de revisión manual [gate · PARADA con registro]

Cargar `references/manual-review-triggers.md`. Cruzar lo tocado con el **scope preciso** de lo que un test verde NO prueba (contabilidad, dinero, datos reales, seguridad/`sudo()`, stock, integraciones, escala, cascadas, cron, UX…). Cada disparador → **REVISIÓN MANUAL OBLIGATORIA** con quién firma; sin firma **bloquea el merge**. **Calibrar**: ledger útil/no-útil, estrechar scope de mala ratio, podar lo que nunca pesca; disparador nuevo → scope estrecho desde el día 1.

### Step 7 — Decisión: merge o correcciones [2 confirmaciones duras]

Cargar `references/decision-and-feedback.md`.
- **Merge** solo con la puerta entera en verde (10 casillas), y **con confirmación explícita**.
- **Correcciones**: review priorizado con el POR QUÉ y la cláusula que incumple
  (`templates/pr-review-feedback.md`); constancia en la `project.task` (`odoo-pilot`).
- Fallo del CONTRATO y no del dev → escalar a `odoo-solution-architect`.
- **Cierre**: `destroy.sh` + quitar el worktree (un `down` deja el slot ocupado y los datos dentro).

## PARADAS Y PREGUNTAS
Cargar `references/stops-and-questions.md`. Regla: **para** si la acción sale fuera o es irreversible ·
**pregunta** si la respuesta cambia el trabajo · en lo demás, decide y sigue.

- **CONFIRMACIÓN DURA** antes de **publicar el review** (enseñando el texto íntegro que verá el dev) y
  antes de **mergear** (puerta de merge + radio de explosión + refs revisadas + quién firmó). Nunca
  automático por tener los gates en verde: mergear Core entra en 13 instalaciones.
- **PARADA con registro** en el Step 6: qué disparadores han saltado y **quién firma** cada uno. El
  modelo NO firma. El resultado se anota en el ledger de `manual-review-triggers.md`.
- **Preguntas**: estrategia de datos (Step 0) · modo, solo si es ambiguo (Step 0) · destruir o mantener
  la instancia (Step 7).
- **No paran**: falta de MD (→ bug-fix), findings intermedios, la mecánica del docker, un gate en rojo.
  **Sin TTY**: no se publica, no se mergea, no se firma; veredicto a fichero e instancia viva.

## NEVER

1. En **modo contrato**, aprobar sin el MD delante — sería un review a ciegas. (En **modo bug-fix** no hay MD: se sustituye Step 3 por review source-verified; NO se hace STOP.)
2. Merge sin **instalar en Docker limpio** (Step 1) y con tests en verde real cubriendo los criterios (contrato) o el bug con **falsación superada** (bug-fix) — CI declarado no basta —, o si el dev cambió una firma/campo `# CONTRACT` o metió deps no acordadas sin pasar por el arquitecto.
3. Pedir correcciones sin decir el POR QUÉ y qué criterio/contrato incumple (feedback accionable).
4. Aprobar por "tests verdes" un disparador de **revisión manual** sin firma humana — y el modelo no firma. Ni mergear ni publicar en el PR sin confirmación explícita.

## ALWAYS

1. Revisar contra el contrato del MD (o, sin MD, contra la fuente Odoo), no contra el gusto personal, distinguiendo fallo-del-dev de fallo-del-contrato (este vuelve al arquitecto).
2. Tests en verde real antes de merge: unit + e2e cubriendo los criterios (contrato) o test de regresión con falsación superada (bug-fix).
3. Feedback priorizado (bloqueantes → mayores → menores), constructivo y con el porqué.
4. Dejar constancia en la task y notificar en el PR; reconocer lo que está bien (review educativo).
5. Marcar la **revisión manual obligatoria** y **calibrar por resultado** (`manual-review-triggers.md`): el recall crece apilando reglas, la precisión solo estrechando scope y podando.

## GOTCHAS

0. **Qué medir y qué no.** La suite estándar **solo se corre si el módulo puede alcanzarla**: solo-permisos no la alcanza (Odoo la ejecuta como admin y esos módulos implican `base.group_system` — 4 deltas vacíos medidos); sí la alcanza si escribe datos, pone un `required`, toca una vista o restringe a un usuario normal. Control con el **mismo juego de módulos**, baseline **a 0**, **dirigir por clase** (19 tests = lo mismo que 166) y **refrescar el head antes de medir**. Todo en `references/test-verification.md` §3b.

1. **Review contra contrato ≠ genérico** — sin MD no se abandona la skill: **modo bug-fix** (source-verified + falsación) conservando todos los gates Odoo. `/review` a secas pierde esa rigurosidad.
2. **"Pasa en mi máquina"** — se ejecuta en el Docker limpio del Step 1; verde declarado no es verde. Usar dump del cliente si el comportamiento depende de sus datos, y **tumbar el Docker al cerrar**.
3. **Criterio sin test / e2e ausente** — lo más fácil de colar: el código "funciona" pero falta el test de un criterio o el flujo end-to-end → correcciones.
4. **El dev "mejoró" el contrato** — cambiar una firma aunque sea a mejor rompe a quien depende; vuelve al arquitecto.
5. **Verde ≠ correcto** — un test verde no cubre los disparadores manuales (contabilidad, dinero, datos, seguridad). Y el CÓMO del dev no es un error si respeta el contrato.
6. **Nunca `git checkout` en el árbol montado, y pinnea lo que NO revisas** — `indawsodoo/<MAJOR>` lo montan **13 proyectos** (`/18`; **20 proyectos únicos** montan algún major — remedido 2026-08-17, comando en `references/impact-checklist.md`) por bind mount, así que le cambias la rama a todos en caliente. Y **normalmente hay algún major en rama de trabajo**: compruébalo con `git -C <repo> branch --show-current` en cada montaje antes de revisar; no lo asumas. Sin pinnear, revisarías el PR **con tu trabajo a medias dentro**. Todo por `git worktree`.
7. **En custom de cliente puede no haber repo donde montas** — 16 de 22 montajes del parque no están versionados en su sitio, y algunos montan módulos sueltos (remapeo por subruta, no swap). Sin repo → abortar y localizarlo.

## Delegates to
`docker-deployment` (instancia efímera: `new.sh --mount`, `restore.sh`, `destroy.sh`) · `/review` ·
`quality-gate` · `git-review-pr`/`git-manager` · `odoo-flow-tester` (e2e) · `odoo-pilot`/MCP (constancia) ·
`odoo-solution-architect` (si el fallo es del contrato) · `odoo-deep-knowledge` (¿gotcha conocido?).

## Consumed by
`odoo-solution-architect` (developer) — cierra el bucle del arquitecto: revisa el PR de la tarea que
diseñó y decide merge o correcciones.

## REFERENCES
- `review-instance.md` — worktree, pinneo, instancia efímera, destrucción
- `stops-and-questions.md` — qué para, qué se pregunta, qué no
- `docker-verification.md` — gate de instalación y comando de tests
- `test-verification.md` — matriz criterio→test, verde real, falsación
- `contract-review-checklist.md` — contrato vs libertad del dev · `impact-checklist.md` — impacto
  Odoo y radio de explosión
- `manual-review-triggers.md` — 15 disparadores con scope + ledger de calibración
- `decision-and-feedback.md` — puerta de merge y feedback accionable
- `templates/` — feedback y aprobación · `examples/` — review de ejemplo
