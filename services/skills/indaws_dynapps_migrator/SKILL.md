---
name: indaws-dynapps-migrator
description: >
  Orquesta el proceso completo de migración Odoo entre versiones para clientes
  Dynapps. Punto de entrada único: acepta cliente + BD + repo + versión destino
  y guía todo el flujo (prerequisitos → upgrade BD → migración código →
  validación → go-live), delegando a las skills especializadas y preguntando al
  usuario solo cuando se necesita decisión humana. Distingue desde el entry-point
  el escenario (on-premise/GitLab vs odoo.sh/GitHub) y puede correr de forma
  autónoma hasta entregar un entorno local levantado con código y BD migrados.

  TRIGGER: "migra el cliente X", "empieza la migración de Y", "continúa la
  migración de Z", "migration flow", "migrate customer", "iniciar migración".
version: 1.7.0
model: opus
effort: high
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, Agent]
---

# Dynapps Migrator

Skill orquestadora del flujo de migración Dynapps. Gestiona 5 bloques:
prerequisitos → upgrade BD (Upgrade Service + iterate) → migración de código
→ validación → entrega y go-live. El dev solo interviene en los STOP points definidos.

> **El port no entrega.** Los agentes de migración de código no publican ramas ni
> abren MRs al terminar: dejan el código en local y el entorno levantado sobre la
> base migrada. Subir se valora **después** de la validación, como paso propio.

## Escenario (resuelto en el entry-point)

La skill resuelve **un único `scenario`** al arrancar y ramifica el track de código
y el go-live en consecuencia (ver `references/scenarios.md`):

- **`onpremise_gitlab`** — Bélgica dyncloud + España: GitLab, `glab`, pipeline verde,
  `parts/oca_pull`, Upgrade Service on-premise.
- **`odoosh_github`** — Bélgica odoo.sh: GitHub, `gh`, **sin pipeline** (gate = pre-commit
  + tests locales), addons vendorizados en `parts/oca`, BD/go-live delegados a
  `odoo-database-upgrader` con `target=odoo.sh`.

## Modo autónomo (hito "Entorno local listo")

Tras resolver el escenario, imprimir requerimientos (`references/requirements.md`) y una
confirmación, la skill puede correr **autónoma** BLOQUE 1 → 2 → 3 hasta entregar un
**entorno local levantado y sin errores con código y BD migrados** — parando solo en los
gates irreducibles. MRs, validación funcional y go-live quedan **fuera** de la corrida
autónoma. Detalle y criterio de aceptación en `references/autonomous-mode.md`.

> **Nota go-live**: la subida del dump resultante al servidor de producción
> se realiza manualmente (fuera del flujo automatizado). Ver BLOQUE 5.

## Regla absoluta — acciones GitLab

**Ninguna acción que escriba en GitLab se ejecuta sin confirmación explícita del usuario**,
independientemente del modo de permisos o de cualquier "bypass" activo. Esto aplica a:

- `git push` (cualquier rama)
- `glab mr create` / `glab mr update`
- Cualquier otro comando `glab` que modifique el estado remoto

El patrón obligatorio es: **preparar → mostrar resumen → esperar confirmación → ejecutar**.
Un "sí" previo para una acción no autoriza la siguiente. Cada push y cada MR es una confirmación independiente.

## Regla absoluta — neutralización de datos de producción

El dump de origen es de **producción**. La BD restaurada en local **debe estar neutralizada
y verificada** antes de arrancar `odoo-bin` por primera vez. Nunca se arranca Odoo contra una
BD de producción sin: correo saliente desactivado, crons desactivados, conectores externos
(EDI/banca/transportistas) y credenciales blanqueadas, y `database.is_neutralized = True`.
El gate es **duro**: check en rojo → corregir, nunca avanzar. Ver
`dynapps-db-upgrade-runner/references/neutralization.md`.

## Regla por defecto — paridad de comportamiento 1:1

Ante cualquier **diferencia de comportamiento introducida por la nueva versión** (una feature
que Odoo cambió, movió, eliminó o rehízo), el **default es preservar el comportamiento que el
cliente tenía en la versión origen, 1:1**. No se adopta el comportamiento nuevo de Odoo por el
hecho de que sea el estándar de la v-destino. Desviarse del 1:1 (aceptar el cambio de versión,
rediseñar, o dejar caer la funcionalidad) requiere **decisión de negocio explícita** — nunca se
asume. Esto guía las decisiones por módulo (STOP ④), los TODOs de port (STOP ⑥) y la validación
de paridad (`odoo-migration-parity`). Ver `references/autonomous-mode.md`.

## Regla — el entorno migrado debe ser reproducible por terceros

El entregable ("Entorno local listo") debe poder **levantarlo otra persona de forma independiente**
con solo dos artefactos: (1) el **código migrado en GitLab/GitHub** y (2) la **BD recién migrada por
el Upgrade Service**. Nada de parches locales, pasos manuales no versionados ni datos que vivan solo
en la máquina de quien migró. Todo lo necesario (scripts de migración, deps del buildout, config)
va **commiteado al repo**; la BD es la salida del Upgrade Service (con su neutralización documentada
y reproducible). Ver el gate de reproducibilidad en `references/autonomous-mode.md`.

## Regla — aprender entre migraciones (Dynbrain)

El flujo mantiene un **bucle de dos vías** con el catálogo de errores comunes de Dynbrain
(`Desarrollo › Migraciones › Errores comunes`):

- **Lectura**: en el pre-flight y en la convergencia se consulta el catálogo y se lleva como
  checklist **preventivo** — el objetivo es no cometer el error, no corregirlo después.
- **Escritura**: al detectar un fallo **del mecanismo** (no del negocio de un cliente), se
  registra genericado, deduplicando antes contra lo ya escrito.

El filtro de admisión es estricto y **ante la duda no se escribe**: una ficha con datos de un
cliente contamina el catálogo de todos. Detalle en `references/dynbrain-feedback-loop.md`.

## Startup — detección de estado

Al iniciar, ejecutar en orden (ver `references/input-detection.md`):

1. Parsear el prompt + **resolver `scenario`** (ver `references/scenarios.md`)
2. Comprobar prerequisites de la máquina
3. Comprobar buildout del cliente en `$ODOO_SRC/buildouts/<cliente><version>/`
4. Comprobar workdir en curso: `indaws-migrate playbook-session 2>/dev/null`
5. Si workdir en curso → **resume mode** (ver `references/resume-mode.md`)
6. **Consultar el catálogo de errores comunes en Dynbrain** y llevarlo como checklist
   preventivo (ver `references/dynbrain-feedback-loop.md`)
7. Imprimir **requerimientos del escenario** + 1 confirmación (ver `references/requirements.md`)
   antes de arrancar el modo autónomo (ver `references/autonomous-mode.md`)

## Paso 0 — Plataforma (antes de cualquier otra pregunta)

**Preguntar siempre con AskUserQuestion, nunca asumir:**

> ¿Dónde vive esta migración? · **dyncloud BE** (buildout, GitLab con CI y pre-commit,
> estructura `xx_all`/`modules_to_remove`/`migrations`) · **Odoo.sh** (sin CI propio ni
> estructura buildout; el upgrade lo lanza la plataforma y puede no haber tickets)

Con la respuesta, escribir `profile.yaml` en el workdir:

```python
from gates.profile import MigrationProfile
MigrationProfile.for_platform(respuesta, client=cliente,
                              source_version=origen, target_version=destino).save(workdir)
```

Ese perfil manda en el resto del flujo: cada gate declara las capacidades que necesita
y el runner lo **salta con motivo** cuando la plataforma no las tiene. **Saltado no es
verde**: se reporta `not_applicable` con su razón y su compensación, para que una
migración en Odoo.sh no acabe menos validada sin que se vea en el informe.

Lo que NO se asume nunca: que exista pipeline, que exista pre-commit, que exista
estructura buildout, ni **que exista un ticket**. Donde falta el ticket, el motivo y
quién acepta se registran en el workdir.

## Paso 1 — Generar el plan, no guiarlo

Con el perfil escrito, generar el dev-plan y ejecutarlo por fases:

```python
from planner.builder import PlanInputs, build_plan
build_plan(profile, PlanInputs(cliente, origen, destino, workdir, repo), plan_dir)
```

Siete fases, las mismas en las dos plataformas — lo que cambia es qué gates aplican.
Es un plan en formato dev-plan 1.0 de nivel `alta`, sin motor ni `state.yaml`: el estado es
la tabla `## Phases` de `plan.md` (todas las filas `pending` al generarlo) y el plan lleva
`goal.md`, `plan.md`, `decisions.md`, `GOTCHAS.md`, `BRIEF.md`, `phases/NN-slug/phase.md`,
`docs/` y `logs/.gitignore`. El frontmatter de cada fase tiene solo `executor`, `risk`,
`rounds` y `red_first` (las de datos van a `opus`, con `risk: ["migration"]`, dos rondas y
rojo previo); los accesos van en `## Requires` (un comando que prueba la capacidad en el
perfil y cómo conseguirla) y los documentos en `## Docs`.
Cada fase con gates lleva su `verify` en `verify/<slug>.py`, que invoca el runner (es un
comando del proyecto, no del motor), y el `## Verify` de la fase es un bloque cercado con
ese comando: **el verify no es prosa, es un comando que devuelve 0 o 1**. El `plan.md`
incluye una sección con lo que esa plataforma no puede validar y con qué se compensa, para
que un plan de Odoo.sh no parezca igual de sólido que uno de dyncloud. Las skills, de fase
y transversales, no las fija el generador: se eligen al activar, con las de la sesión.

Dos cosas que el plan generado deja dichas y conviene respetar:

- **Las fases 2 y 3 se lanzan juntas.** El upgrade tarda decenas de minutos y la base
  del repositorio no depende de él.
- **Antes de reparar nada, fotografiar el estado recién migrado.** Si no, los gates de
  valores miden el resultado de nuestros propios arreglos en vez de la migración.

## Gates (capa propia, fuera del kit)

`gates/` de esta skill. Importa el `indaws_migration_kit` pero **no lo modifica**: el
kit lo comparten otras migraciones que no siguen estos lineamientos.

| Gate | Necesita | Qué detecta |
|---|---|---|
| `value-parity` | BD origen + BD migrada | Campos propios que llegan **vacíos** aunque existan y la tabla esté llena |
| `module-versions` | repo | Carpetas `migrations/` por encima del manifiesto (**no se ejecutan nunca**) y manifiestos que no superan lo instalado |
| `checklog` | log de una ejecución | ERROR/WARNING no ignorados; en Odoo.sh es el único control de calidad que hay |
| `delivery-hygiene` | perfil + datos de entrega | Rama, co-autor, nota, y etiquetas/revisor/ticket **solo donde la plataforma los tiene** |

El `value-parity` se apoya en un **baseline** de pérdidas aceptadas (`gates/value_loss_baseline.yaml`
del workdir): cada pérdida deliberada se clasifica una vez con su motivo — ticket
opcional — y el gate solo falla ante pérdidas nuevas. Sin baseline, un diff de valores
es ruido: en una migración real dio 179 hallazgos, casi todos deliberados.

## Input mínimo

| Campo | Ejemplo | Cuándo es obligatorio |
|---|---|---|
| `cliente` | tecniman | Siempre |
| `version_origen` | 10 | Siempre |
| `version_destino` | 18 | Siempre |
| `db_source` | `~/dumps/tecniman.dump` (archivo) **o** `postgres://localhost:5432/tecniman_v10` (DSN) | Siempre |
| `repo_path` | $ODOO_SRC/buildouts/tecniman18/parts/custom | Si el buildout ya existe |
| `repo_url` | git@gitlab.dynapps.be:customers/v10/tecniman.git | Solo si buildout NO existe aún |

> **`db_source` — caso normal es un archivo local**: el origen suele llegar como un
> **dump comprimido en local** (`.dump`/`.sql`/`.gz`/`.zip`/`.tar[.gz]`), no como una BD
> viva. Si `db_source` es una **ruta de fichero**, el flujo lo **descomprime y restaura**
> en PostgreSQL local ANTES del BLOQUE 2 (ver `references/input-detection.md`). Si es un
> **DSN** `postgres://…`, se usa la BD tal cual. Alias aceptados en el prompt: `db`,
> `dump`, `db_connection`.
>
> **Escenarios soportados**: on-premise/GitLab (Bélgica dyncloud + España) y
> odoo.sh/GitHub. El `scenario` se resuelve en el entry-point (ver `references/scenarios.md`).
>
> **Resolución de `repo`**: preferir siempre `repo_path` local — la skill trabaja directamente
> con ficheros en disco (Read/Edit/Bash). La URL remota solo es necesaria para el clone inicial
> en BLOQUE 1; después `glab` deduce el remote desde `git remote -v` del repo local.

Pedir solo los campos que falten (AskUserQuestion).
Si el buildout existe, intentar auto-detectar `repo_path`:
```bash
ls $ODOO_SRC/buildouts/${CLIENTE}${VERSION_DESTINO}/parts/custom 2>/dev/null
```

## Workflow (5 bloques)

Detalle completo (paralelización, cada bloque, los 9 STOP points) en
**`references/workflow.md`**. Resumen:

| Bloque | Qué | Skills | STOP |
|---|---|---|---|
| 0 | Detección de estado y routing | (interno) | |
| 1 | Preparación local (máquina + buildout) | `dynapps-dev-bootstrap`, `dynapps-local-env-setup` | ①② |
| 2 | Upgrade BD + **gate de neutralización** + iterate | `dynapps-db-upgrade-runner`, `odoo-database-upgrader` | ③④ |
| 3 | Migración de código (ramificado por `scenario`) — **el port hasta arrancar limpio en local = hito autónomo "Entorno local listo"**, sin subir nada; los MR (⑦) son posteriores | `dynapps-code-migration`, `odoo-module-upgrader` | ⑤⑥⑦ |
| 4 | Validación: paridad origen↔migrado + flujos base end-to-end + flujos custom + smoke RPC + **validación en navegador** (Capa 2b, opcional) | `odoo-migration-parity`, `odoo-pilot` | ⑧ |
| 5 | Entrega (ramas/MRs, ya validado) + go-live manual | `git-manager`, `odoo-pilot` | ⑨ |

> **Frontera autónoma**: BLOQUES 1–3 corren autónomos hasta el hito "Entorno local listo"
> (BD migrada+neutralizada, código portado, `odoo-bin` arrancando sin errores), es decir
> hasta ANTES de crear MRs. STOP ⑦ (MR/merge), STOP ⑧ (validación) y STOP ⑨ (go-live) son
> posteriores al hito y quedan fuera de la corrida autónoma.

> **Paralelización**: el BLOQUE 2 tiene una espera larga (upgrade en servidores de Odoo). Lanzar
> el upgrade en background con vigilante y avanzar en paralelo todo lo que NO depende del dump
> migrado (buildout, análisis de módulos, rebase de histórico). Detalle en `references/workflow.md`.

## GOTCHAS

1. **El dump es de producción y llega SIN neutralizar.** Gate de neutralización DURO en BLOQUE 2:
   nunca arrancar `odoo-bin` contra la BD restaurada antes de verificar `is_neutralized=True`,
   mail servers y crons desactivados. Check en rojo → corregir, nunca avanzar.
2. **Toda escritura en GitLab necesita confirmación explícita** (push, `glab mr create|update`),
   independientemente del modo de permisos. Un "sí" previo no autoriza la siguiente acción.
3. **Nunca arrancar MRs por módulo sin el histórico git integrado** (BLOQUE 3, STEP 0.5): si el
   rebase fundacional + `[MIG] Initial Commit` no están, la base se reescribe luego y hay que rehacer.
4. **CI puede estar bloqueado por accesos** (usuario External → 0 jobs; sin acceso a `dyn-edi` →
   build casca al clonar). Entonces el gate pasa a ser **verde en local** (pre-commit + tests).
   Estos fallos son petición a infra, no fix de código.
5. **La subida del dump a producción (go-live) es MANUAL** — fuera del flujo automatizado. El skill
   verifica gates y espera confirmación (STOP ⑨), pero no sube nada al servidor por sí mismo.
6. **Preferir siempre `repo_path` local sobre `repo_url`** — la skill trabaja con ficheros en disco;
   la URL remota solo se usa para el clone inicial del BLOQUE 1.
7. **Resolver `scenario` ANTES de ramificar** (`references/scenarios.md`). El host del repo es la
   señal más fiable (`gitlab.dynapps.be` → `onpremise_gitlab`; `github.com` → `odoosh_github`).
   BLOQUE 3 (código) y 5 (go-live) ramifican por `scenario`; NO asumir GitLab por defecto.
8. **Consultar Dynbrain ANTES, no después** (`references/dynbrain-feedback-loop.md`): el
   catálogo existe para evitar el error, no para explicarlo una vez cometido. Y al escribir,
   el filtro es de las TRES condiciones: mecanismo + reproducible + genericable.
9. **La corrida autónoma termina en "Entorno local listo"** (`references/autonomous-mode.md`), ANTES
   de cualquier escritura en GitLab/GitHub. No crear MRs ni pushear dentro del modo autónomo.

10. **La validación en navegador (BLOQUE 4, Capa 2b) NUNCA es un gate ni entra en CI.** Una
   corrida de navegador conducida por un modelo no es reproducible: produce hallazgos, no un
   pass/fail. Los gates duros siguen siendo `-u all` limpio, `checklog-odoo` y
   `--fail-on-regression`. Y **no entra en la corrida autónoma**: es posterior al hito local.

## References

| Fichero | Contenido |
|---|---|
| `references/scenarios.md` | Switch de plataforma (on-premise/GitLab vs odoo.sh/GitHub): señales, precedencia, routing |
| `references/requirements.md` | Checklist de requerimientos por escenario (bloqueante vs informativo) |
| `references/dynbrain-feedback-loop.md` | Bucle de aprendizaje con Dynbrain: vía de lectura (preventiva) y de escritura (catálogo de errores comunes) |
| `references/autonomous-mode.md` | Modo autónomo: hito "Entorno local listo", STOPs que se mantienen vs auto-avanzan |
| `references/workflow.md` | Workflow detallado BLOQUE 0–5 + paralelización + STOP points |
| `references/input-detection.md` | Detección de estado inicial y qué preguntar |
| `references/stop-points.md` | Los 9 STOP points detallados |
| `references/block-routing.md` | Routing de cada bloque a cada skill con parámetros |
| `references/resume-mode.md` | Cómo detectar y retomar una migración en curso |
| `odoo-migration-parity:references/browser-layer.md` | **Capa 2b — validación en navegador** (Playwright MCP): selección de flujos por hallazgos, presupuesto de tokens, clasificador de ruido, destinos de un hallazgo |
| `references/convergence-runbook.md` | **Del dump migrado al `-u all` limpio**: guardas de overrides, `-i` de dependencias nuevas, re-neutralización post-upgrade, traspaso de datos entre campos renombrados |
| `common:references/shared/` | Convenciones compartidas cross-suite |

## Delegates to

| Skill | Suite | Bloque | Rol |
|---|---|---|---|
| `dynapps-dev-bootstrap` | developer | 1 | Prerequisites de máquina |
| `dynapps-local-env-setup` | developer | 1 | Buildout del cliente |
| `dynapps-db-upgrade-runner` | developer | 2 | Upgrade BD (Upgrade Service) |
| `odoo-database-upgrader` | developer | 2 | Iterate loop + planificación módulos |
| `dynapps-code-migration` | developer | 3 | Migración repo de código |
| `odoo-module-upgrader` | developer | 3 | Port de módulos custom |
| `git-manager` | developer | 3 | Commits, MRs, pipeline validation |
| `odoo-migration-parity` | developer | 4 | Paridad de capacidades origen↔migrado (gate previo a go-live) |
| `odoo-pilot` | common | 4 + 5 | Smoke RPC local y producción |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| migration-portfolio | migration-lab | Ejecutar la migración técnica (módulos + BD) de cada proyecto de la cartera |
