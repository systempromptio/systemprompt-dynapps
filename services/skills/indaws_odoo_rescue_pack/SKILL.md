---
name: indaws-odoo-rescue-pack
version: 1.7.0
effort: high
model: opus
description: >-
  Informe del ESTADO ACTUAL (técnico) de un Odoo heredado, en TRES escenarios de acceso: (1) BD+código
  → salud del código + personalizaciones en BD (Studio/acciones) en `odoo_studio`; (2) solo BD sin
  fuente y (3) BD productiva (solo lectura + consentimiento) → ingeniería inversa delegada a
  `odoo-reverse-engineering`. EXTRA opcional: coste de migrar. NOT presupuesto (→ quotation-builder) ·
  NOT auditoría de un módulo suelto (→ odoo-code-auditor).
  Triggers: "rescue pack técnico", "estado actual del código y la BD", "qué se hizo a mano", "no tenemos el código fuente del cliente".
---

# Odoo Rescue Pack (técnico)

Skill **orquestadora**. Informe del **ESTADO ACTUAL** en dos pilares: **salud del código** (calidad,
deuda, seguridad, rendimiento) y **personalizaciones en la BD** (Studio + acciones). Delega auditoría
y presupuesto. **Alcance = solo lo instalado**, por autor. **Tres escenarios detectados en Fase 0**
(la certeza degrada 1→3 y se dice en el informe): **(1)** BD+código → flujo clásico; **(2)** solo BD
sin fuente y **(3)** BD productiva con consentimiento → **delegan en `odoo-reverse-engineering`**
(mapa y contrato en `references/access-scenarios.md`). Migrar (sec. 07) = EXTRA opcional.

## Agent Routing

> **El coste no son los agentes, son sus llamadas** (215K cada una, medido) → GOTCHA de coste.

| Fase | Modelo | Corre como |
|------|--------|-----------|
| 0 Acceso + alcance | opus | Principal |
| 0.5 Empaquetar el corpus | — | **Inline** (`scripts/pack_corpus.sh`) |
| 1 Triaje (BD: instalados) | — | **Inline** (`scripts/module_triage.sh`; es un script, no un agente) |
| 2 Salud + flujos + bugs (una sola pasada) | opus | Subagent → `odoo-code-auditor`, **1 por área funcional** |
| 3 Extracción Studio/acciones | — | **Inline** (scripts SQL; no es un agente) |
| 3b Módulo `odoo_studio` | opus | Principal (+ scaffolding delegado) |
| 4 Priorización + valoración (sec. 4-6) | opus | Principal (+ `odoo-effort-estimator` · `odoo-database-analyzer`) |
| 5 Síntesis + PDF con marca | opus | Principal (+ `dynapps-brand` · `pdf-renderer`) |
| 6 Handoff comercial | — | → `quotation-builder` |
| **2/3** Reconstrucción sin fuente | opus | Subagent → `odoo-reverse-engineering` (reemplaza/degrada Fases 1-2) |

## References

| File | Load when |
|------|-----------|
| [access-scenarios.md](references/access-scenarios.md) · [access-prerequisites.md](references/access-prerequisites.md) | **Fase 0** — 3 escenarios, detección, delegación RE, degradación · métodos de acceso a código/BD |
| [module-triage.md](references/module-triage.md) · [studio-extraction.md](references/studio-extraction.md) · [agent-budget.md](references/agent-budget.md) | Fase 1-3 — alcance=instalado por autor · extraer Studio/acciones · **coste del subagente, corpus y presupuesto de llamadas (Fase 2)** |
| [synthesis-and-crosscheck.md](references/synthesis-and-crosscheck.md) · [deliverable-structure.md](references/deliverable-structure.md) | Fase 4-5 — cruce/preguntas/propuesta · **dos documentos** (A + anexo B), encuadre, maquetación |
| [gotchas.md](references/gotchas.md) · [report_template.md](assets/report_template.md) · [anexo_tecnico_template.md](assets/anexo_tecnico_template.md) · [config.json](config.json) | Gotchas (siempre) · plantilla ejecutivo (A) · plantilla anexo técnico (B) · datos (producto, `odoo_studio`, targets) |
| `references/shared/odoo-lifecycle-licensing.md` · `references/shared/odoo-connection.md` | Fase 4 retirada Odoo.sh/+25% · fallback API live `odoo-pilot` |

## Workflow

### Fase 0 — Acceso + escenario [OPUS principal]
1. **Detectar el escenario 1/2/3** (árbol en `access-scenarios.md`; puede ser **mixto** por módulo).
   Puerta: al menos lectura de BD, y **caso 3 exige consentimiento escrito**.
2. **[STOP] ¿se ofrece migración?** (`{ofrecer_migracion}`). Sí → pedir `{version_objetivo}` y
   `{pct_migracion_recurrente}`. No → fuera sección 07, Op.2 y las skills de gap de versión.
3. **[STOP] Confirmar**: (a) escenario y límites · (b) caso 3 → **consentimiento escrito**, sin él
   PARAR · (c) ¿módulo `odoo_studio` o sólo diagnóstico? · (d) `{detalle}` (`completo` por defecto) ·
   (e) `{en_produccion}`. **Qué activa cada respuesta y los datos a recoger →
   `access-scenarios.md`.** No improvises: se hacen todas o el alcance sale mal.

> **Casos 2/3 (sin fuente):** `odoo-reverse-engineering` = **motor de DATOS** (hasta `consolidate.py`;
> **nunca su `build_report.py`** → renderizador único = Fase 5). Salud degrada (caso 2 parcial vía
> `.pyc`, caso 3 "no evaluable"). **Dump-only sin addons:** reconstrucción por SQL (Fase 3). Contrato
> y degradación en `access-scenarios.md`.

### Fase 0.5 — Empaquetar el corpus custom [INLINE — es un script]
`scripts/pack_corpus.sh <addons> <out>` → **un fichero por módulo**, con **números de línea reales**
(las citas a fichero y línea siguen siendo exactas) + `CORPUS_INDEX.md`. El auditor lo lee de una vez: ~90
llamadas → ~15. **Sin esto la Fase 2 cuesta 5-8× más sin averiguar nada extra.** Sáltalo en casos 2/3.

### Fase 1 — Triaje (alcance dirigido por la BD) [INLINE — es un script]
El alcance lo manda la BD: **solo lo instalado**. `scripts/module_triage.sh <out> --src <paths>` (solo lectura) → clasifica
los instalados por `author` en **STANDARD** (ignorar), **OCA** (solo continuidad) y **CUSTOM** (exhaustivo), y marca
instalados **sin fuente** (riesgo). Confirmar autores ambiguos como principal. Detalle en `module-triage.md`. → **sec. 03**.

### Fase 2 — Salud del código + flujos + bugs de runtime, EN UNA SOLA PASADA [OPUS SUBAGENT]
**Un agente por ÁREA FUNCIONAL** (no uno por módulo y otro por flujo: leerían lo mismo dos veces).
Recibe los `corpus_*.txt` de su área y hace todo de una pasada:
- **Salud:** Calidad/Deuda, Seguridad, Rendimiento, Convenciones; **0-10** por módulo.
- **Flujos + bugs de runtime** (si `{detalle}`=completo) → anexo **B1-B9**; si aún no opera
  (`{en_produccion}`=no), "se manifestará al operar". **EXTRA (si migración=sí):** Migrabilidad 0-10.

**Obligatorio en su prompt** ([agent-budget.md](references/agent-budget.md)): sus `corpus_*.txt` y
leerlos de una vez · **presupuesto de llamadas** ("máx. ~20, agrupa greps") · los **volúmenes de BD**
de la Fase 3 · devolver **≤45 líneas** y el informe **a fichero**.

**OCA** → versión instalada + continuidad vs `{version_objetivo}`; **STANDARD** → ignorar. → **sec. 05
y anexo B**. *(Casos 2/3: la sustituye/degrada `odoo-reverse-engineering`.)*

### Fase 3 — Extracción Studio/acciones + reconstrucción custom por SQL [INLINE — son scripts]  ← solo Postgres, sin Odoo
Restaurar dump + **solo Postgres** (`docker-deployment` `up.sh <proj> db` + `restore.sh`, `PG*` del `.env`). Por SQL solo-lectura: `db_state.sh` (conteo → **sec. 2**), `extract_studio.sh` (Studio/acciones → **sec. 3**), `reconstruct_custom.sh` (capa declarativa de cada custom → `FOOTPRINT.md`), `db_intelligence.sh` (dimensionado: deps, complejidad, **fill-rate**, actividad). Los dos últimos son el **motor del escenario dump-only sin addons** (RE no aplica): recuperan ~toda la capa declarativa, solo el Python imperativo queda opaco. Qué extrae cada script + porqués en `studio-extraction.md` + `access-prerequisites.md`.

### Fase 3b — Consolidar en módulo `odoo_studio` [OPUS principal] (condicional al [STOP] de Fase 0)
Si se pidió como entregable: aglutinar lo extraído en **un módulo `odoo_studio`** (nombre en
`config.json`); por elemento decidir mantener/reescribir nativo/sustituir por estándar o Core
(`odoo-classification-guide`). Scaffolding → `odoo-module-creator`+`odoo-data-files`; records según
`odoo-studio-migration`. El custom pasa a git, auditable (realimenta Fase 2). Si no, recomendación.

### Fase 4 — Priorización + valoración + oportunidades Core [OPUS principal]
Repartir hallazgos de Fase 2 (valorar con `odoo-effort-estimator`; presu formal → `quotation-builder`):
**Sec. 4** mejoras VALORADAS en 2 niveles; **Sec. 5** no críticos → preguntas a consultoría; **Sec. 6**
propuesta value-based. **Op.2 (migrar) y sección 07 solo si `{ofrecer_migracion}`=sí**; si no, solo Op.1.
- **Migraciones recurrentes (en Op.2):** **calcula** el coste de implantar ese mismo alcance **desde cero** (`odoo-effort-estimator` + el mapeo a Indaws Core de la sec. 6) × `{pct_migracion_recurrente}` = importe **por salto, cada `cada_n_anios`** (`config.json`), **siempre con su condición**: vale mientras no se amplíe la base instalada. Sin datos para calcularlo (casos 2/3) se pregunta y va como **cifra estimada**. Método en `synthesis-and-crosscheck.md`.
- **Mapeo a Indaws Core (sec. 6):** cruza cada custom problemático con el **catálogo de paquetes Indaws Core**
  (`references/shared/paquetes_indaws_core.md`) y propón sustituto **con precio**; los importadores/wizards
  → **Framework de Migración e Integración de Datos** (el plan de datos NO va aquí: observación para análisis,
  sec. 5). Opciones y salvedad coste/plazo en `synthesis-and-crosscheck.md`.

### Fase 5 — PDF(s) cliente con marca [OPUS principal] — RENDERIZADOR ÚNICO (los 3 casos)
**Único** generador de PDF (en casos 2/3 el JSON de `odoo-reverse-engineering` es solo insumo; **no** su
`build_report.py`). Genera **DOS documentos** a `docs/` (o solo A si `{detalle}`=ejecutivo):
**A — ejecutivo** (`report_template.md`, portada + 6 secciones); **B — anexo técnico**
(`anexo_tecnico_template.md`, B1-B9 con `file:line`); A **referencia** a B en la tabla de salud 0-10.
Render con **`dynapps-brand` OBLIGATORIO** + `pdf-renderer`. **Nunca tablas 4+ col con `<code>` largos**
(desbordan A4) → bloques verticales. **Verificación en dos pasos, no a ojo:** (1) script PyMuPDF que compare
el `bbox` derecho de **todas** las páginas contra el margen; (2) PNG **sólo de las marcadas** + portada. Estructura y bandas de marca en `deliverable-structure.md`.

### Fase 6 — Handoff comercial
Pasar el diagnóstico (esfuerzo/módulo, riesgos, datos a migrar) a `quotation-builder` → genera Fase 0 (Rescue Pack), propuesta y precio. Producto/precio en `config.json` (`rescue_pack_*`), nunca a fuego. **La migración recurrente NO va al presupuesto** (decidido 2026-08-19): se pinta **solo en el PDF** (Op.2). `quotation-builder` no tiene dónde meter un importe bienal —su paso 7 es la migración puntual y el 9 una cuota mensual— y montarlo es otro cambio. Si alguien lo retoma, la decisión está aquí, no es un olvido.

## NEVER
- Empezar sin **lectura de BD**; Caso 3 (producción) sin **consentimiento escrito** (bloqueante). **Escribir/levantar Odoo sobre producción** (casos 1/3): solo Postgres por SQL; reventar la BD/ejecutar Odoo **solo** sobre la copia desechable del Caso 2.
- Presentar la reconstrucción de casos 2/3 como **código confirmado**: respeta los niveles de certeza de `odoo-reverse-engineering`. Generar el PDF con su `build_report.py` (**renderizador único = Fase 5**).
- Reimplementar ingeniería inversa / auditoría / presupuesto / montar Postgres a mano (todo delegado).
- En el PDF: sin `dynapps-brand` (marca OBLIGATORIA) · otro partner · horas/tarifa €/h (solo importe €). PROHIBIDO. Dar la **implantación desde cero** como precio firme (es una reconstrucción) o callar que una cifra la aportó el consultor.

## ALWAYS
- **Al subagente: material ya leído + presupuesto de llamadas** ("máx. ~20", "≤45 líneas de vuelta, informe a fichero").
- Tratar la BD como solo lectura y extraer lo que vive **fuera de módulos** (Studio/acciones) → `odoo_studio`; distinguir hechos (BD/código) de hipótesis (consultoría); consultar `gotchas.md` antes de concluir.

## GOTCHAS
Completa en [gotchas.md](references/gotchas.md). Críticas:
- **El coste son las LLAMADAS del agente, no cuántos lanzas.** Medido: un rescate = 167,9 M, **118,2 M
  en los subagentes**; 549 tool calls → **215K cada una**; 5 agentes sobre el mismo corpus de 100K =
  **735×**. De ahí Fase 0.5, "un agente por área" y el presupuesto; `/clear` **no** lo baja → [agent-budget.md](references/agent-budget.md).
- **Auditar solo código deja fuera media foto**: Studio, modelos custom y acciones viven en la BD → sin extraerlos se infraestima. Extraer por SQL contra `ir_*` con **solo Postgres** (`docker-deployment` `db`).
- **"Instalado sin fuente" ≠ caja negra**: `ir_model_data` atribuye cada elemento a su módulo → `reconstruct_custom.sh` recupera la capa declarativa por SQL; solo el Python imperativo queda opaco.
- **Secretos y fill-rate**: `ir_config_parameter` = secretos → solo `key`, **nunca values en claro**. Fill-rate (`db_intelligence.sh`): campo 0% o modelo 0 filas = candidato a NO migrar.
- **El módulo de Studio es `studio_customization`, SINGULAR** (`web_studio/models/ir_module_module.py:16-19`, `IrModuleModule.get_studio_module`). En plural, el `NOT EXISTS` descarta justo lo que Studio creó — medido: salían **cero** de tres. Si un rescate devuelve pocas personalizaciones, mira ese nombre antes de concluir que el cliente no tocó nada. Cobertura y método: `tools/bench-rescue` (**12/12**, y 0 filas en una base sin tocar).

## Delegates to
`odoo-reverse-engineering` (casos 2/3, sin fuente) · `odoo-module-scanner` · `odoo-code-auditor` ·
`odoo-classification-guide` · `odoo-studio-migration` · `odoo-module-creator`+`odoo-data-files`
(scaffolding `odoo_studio`) · `odoo-database-analyzer`+`odoo-version-api-reference` (gap versión) ·
`odoo-effort-estimator` · `docker-deployment` (Postgres) · `odoo-pilot` (fallback live) · `dynapps-brand`+`pdf-renderer` (PDF) · `quotation-builder`.

## Consumed by
- `quotation-builder` (diagnóstico → Fase 0 / Rescue Pack) · flujo comercial de consultoría.
