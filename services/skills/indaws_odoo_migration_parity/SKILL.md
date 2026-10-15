---
name: indaws-odoo-migration-parity
description: >-
  Valida la paridad entre el Odoo origen y el migrado: que se perdio, que es regresion y que es cambio de version, incluida la validacion en navegador de lo que solo existe en el cliente web. Usala cuando compares el entorno antiguo con el migrado. Compara a tres vías (origen, migrado, oráculo vanilla) para detectar funcionalidades perdidas tras una migración.
version: 0.3.1
model: opus
effort: high
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, Agent]
---

# Odoo Migration Parity

Valida que una migración **conserva** lo que el usuario final percibe como
funcionalidad — vistas, campos, menús, acciones, informes, permisos, filtros,
crons, automatizaciones, plantillas — comparando el entorno **origen** con el
**migrado**, y decidiendo para cada diferencia si es una **regresión** o un
**cambio legítimo entre versiones**.

> No es específico de ningún cliente: lo dirigen los inventarios, el oráculo, la KB y el baseline.

## El problema que resuelve

Comparar solo **origen ↔ migrado** no distingue una regresión de un cambio que Odoo hizo entre
versiones: los dos aparecen como "falta X", y ese ruido genera tickets. La tercera vía es un
**oráculo**: una instancia *vanilla* del destino con el mismo set estándar+OCA y **sin** el custom.
Con las tres, la clasificación es mecánica. Sin oráculo, lo estándar perdido se contrasta con la KB
(`references/known-version-changes.json`) y lo que no casa queda **UNKNOWN** — nunca se asume "es
de Odoo" sin evidencia. Matriz de decisión y atribución de dueño: `references/classification.md`.

## Las tres capas (comparar a la altura correcta)

| Capa | Qué valida | Coste | Cómo |
|---|---|---|---|
| **1 · Inventario de capacidades** | qué EXISTE (metadata), **incluido lo que hay dentro de cada vista** | barato, determinista, **sin arrancar Odoo** | `scripts/extract_inventory.py` (SQL) + `scripts/diff_inventory.py` |
| **2a · Comportamiento (API)** | que los flujos den el mismo resultado | caro, selectivo | delega en `common-skills:odoo-flow-tester` (Migration Integrity) sobre lo custom-crítico y los UNKNOWN |
| **2b · Navegador** | que lo que vive en el **cliente web** funcione al usarlo (OWL/`t-inherit`, widgets, Studio, wizards) | medio, **acotado por presupuesto** | Playwright MCP, dirigido por los hallazgos de la Capa 1 — `references/browser-layer.md` |
| **3 · Visual / maquetación** | render, PDF, posición — **píxeles** | no se automatiza | checklist humano (patrón `common-skills:odoo-flow-coverage-planner`) |

La Capa 2b cubre el hueco entre la 2a y la 3: un `xpath` de `t-inherit` que no casa deja el pipeline
verde y la funcionalidad fuera de la pantalla — por API es invisible, y no es cuestión de píxeles.
**Nunca es un gate** (ver `references/browser-layer.md`).

La Capa 1 cubre la mayoría de "faltan funcionalidades" —casi siempre **metadata que desapareció**:
un campo fuera del form, un menú, una vista desactivada— sin dos runtimes vivos.

## Workflow

**Paso 0 — Entradas** (AskUserQuestion para lo que falte):
- BD **origen** restaurada (dump de producción pre-upgrade) y BD **migrada**
  (salida de `dynapps-db-upgrade-runner` / `odoo-database-upgrader`).
- **Oráculo** (recomendado): ¿existe una vanilla de la versión destino con el
  mismo set de módulos estándar+OCA? Si no, ver `references/oracle-setup.md`
  (crearla o continuar en modo KB).
- **Inventario custom** (opcional pero recomendado): lista de módulos custom del
  proyecto (`--custom-modules`), del `migration.yaml` / `indaws-migrate modules
  list`. Si no se da, se clasifica por autor + prefijos (fallback).

**Paso 1 — Capa 1: extraer inventarios** (read-only, una conexión por BD):
```bash
python3 scripts/extract_inventory.py --dsn "$OLD_DSN"      --label old      --out old.json
python3 scripts/extract_inventory.py --dsn "$MIGRATED_DSN" --label migrated --out migrated.json
python3 scripts/extract_inventory.py --dsn "$ORACLE_DSN"   --label oracle   --out oracle.json   # si hay
```

**Paso 2 — Capa 1: diff clasificado → informe de decisiones**:
```bash
python3 scripts/diff_inventory.py \
  --old old.json --migrated migrated.json \
  --oracle oracle.json \
  --kb references/known-version-changes.json \
  --baseline baseline.json \
  --custom-modules "$CUSTOM_MODULES" \
  --out parity_report.md --json parity_findings.json \
  --fail-on-regression
```

**Paso 3 — [STOP] Revisión del equipo**: informe agrupado por dueño (dev / negocio /
equipo). UNKNOWN y ambiguos → triage: un `Agent(model:sonnet)` por caso consulta
`odoo-version-api-reference` y el enterprise local. Ver `references/classification.md`.

**Paso 4a — Capa 2a (comportamiento por API, selectivo)**: para custom-crítico y UNKNOWN que la
metadata no cierra, delegar en `common-skills:odoo-flow-tester` (Migration Integrity): mismos
flujos contra origen vs migrado. Ver `references/behavior-and-visual.md`.

**Paso 4b — Capa 2b (navegador)** — omitir si Playwright MCP no está disponible: construir la
lista de flujos **desde los hallazgos**, no desde un catálogo: DEACTIVATED reactivados, vistas
Studio, assets portados (`t-inherit`/patches/widgets), UNKNOWN de interacción, TODOs del port,
wizards y botones de los custom-críticos. Objetivo 5–15 flujos; si se pasa, se recorta por
prioridad y el resto va al checklist humano. Presupuesto duro de ≤12 llamadas por flujo.
Produce **hallazgos, nunca un pass/fail**. Ver `references/browser-layer.md`.

**Paso 5 — Capa 3 (visual)**: checklist humano de lo que solo se valida a ojo (informes,
PDFs, columnas), patrón `common-skills:odoo-flow-coverage-planner`. No automatiza render.

**Paso 6 — Baseline**: cada diff "aceptado/estándar" → `baseline.json`; la siguiente
migración deja de mostrarlo (converge a regresiones reales). Ver `references/classification.md §Baseline`.

## Salida — informe de decisiones

Markdown + JSON agrupados por **quién decide**:
- 🔴 **dev, antes de entregar**: `DATA_LOSS_RISK` (un modelo con datos se vació) · `REGRESSION` ·
  `DEACTIVATED` · `REQUIRED_BLANKED` · `TYPE_CHANGED` (mismo nombre, otro tipo: ¿cabe el dato?).
- 🟠 **negocio**: `CUSTOM_DROPPED` (drop intencionado vs port incompleto) · `RENAMED` (¿traspasó?).
- 🟡 `UNKNOWN` (triage) · 🟢 `INTER_VERSION` / `NEW_IN_TARGET` (documentar).

Verificado inyectando daños: **14/14** bien enrutados y **0** hallazgos entre bases idénticas;
mirar dentro de las vistas cuesta ~28 % más de ruido entre majors. Detalle: `tools/bench-parity`.

`--fail-on-regression` → exit 2 si hay bloqueantes: úsalo como **gate previo al go-live**.

## GOTCHAS

1. **Lo desactivado no desaparece: sigue con su xmlid.** Un cron o un menú en `active=false`
   funciona igual de poco que si lo hubieran borrado, y un diff por xmlid no los distingue —
   desactivar los crons es lo que hace el propio upgrade. El inventario trae `active` de todas las
   categorías que lo tengan; sin eso, **cinco de catorce daños inyectados eran invisibles**
   (`tools/bench-parity`).
2. **Identificadores estables**: compara por `xmlid` (`module.name`), `model::campo` y `model`. NO por id numérico ni etiqueta traducida (cambian entre versiones → ruido falso).
3. **El oráculo debe tener el MISMO set estándar+OCA** que el migrado (solo cambia: sin custom); si difiere, un OCA legítimamente ausente se lee mal. Ver `references/oracle-setup.md`.
4. **Sin oráculo, los estándar perdidos que no matchean la KB son UNKNOWN**, no INTER_VERSION. Nunca se asume "cambio de Odoo" sin evidencia (oráculo o KB).
5. **Datos transaccionales finos** (saldos, asientos, valores) → sub-modo *Migration Integrity* de `odoo-flow-tester`. Esta skill detecta la **pérdida a nivel de modelo** (`models` → **DATA_LOSS_RISK**), complementario.
6. **Conteo aproximado**: `rows` = `pg_class.reltuples`. Haz `ANALYZE` de ambas BDs antes del gate y confirma con conteos exactos los DATA_LOSS_RISK.
7. **Renames/fusiones de Odoo**: decláralos en `known-version-changes.json → model_renames` para que el sucesor presente salga **RENAMED**, no falsa pérdida.
8. **Vendor = no estándar**: autor no-Odoo/no-OCA → su pérdida es regresión (el oráculo nunca lo contiene). `--custom-modules` afina; el fallback por autor sobre-reporta a propósito.
9. **La Capa 1 no necesita Odoo arrancado** — solo las BDs. Reserva runtime para la Capa 2.
10. **La Capa 2b nunca bloquea**: una corrida de navegador conducida por un modelo no es
    reproducible. Produce hallazgos; los gates duros siguen siendo `-u all` limpio,
    `checklog-odoo` y `--fail-on-regression`. Y **nunca se meten tours en CI**.

## References

| Fichero | Contenido |
|---|---|
| `references/inventory-schema.md` | Esquema del JSON de inventario y qué categorías se extraen |
| `references/classification.md` | Reglas de clasificación 3-vías, atribución de dueño, triage y baseline |
| `references/oracle-setup.md` | Cómo construir la vanilla de destino (oráculo) y levantar los entornos |
| `references/known-version-changes.json` | KB (genérica, extensible) de cambios de Odoo por versión — clasificador fallback |
| `references/behavior-and-visual.md` | Capa 2a (delegar a flow-tester) y Capa 3 (checklist visual) |
| `references/browser-layer.md` | Capa 2b: validación en navegador con Playwright MCP — selección de flujos, presupuesto de tokens, ruido, destinos de un hallazgo |
| `common:references/shared/` | Convenciones compartidas cross-suite |

## Delegates to

| Skill | Suite | Cuándo |
|---|---|---|
| `common-skills:docker-deployment` | common | Levantar BD origen/oráculo en local para introspección |
| `common-skills:odoo-pilot` | common | Alternativa RPC a la extracción por SQL / inspección en vivo |
| `odoo-version-api-reference` | developer | Triage de UNKNOWN: diffs de API por versión |
| `common-skills:odoo-flow-tester` | common | Capa 2a: comportamiento (Migration Integrity origen vs migrado) |
| Playwright MCP (`browser_*`) | — (MCP externo, opcional) | Capa 2b: conducir flujos en el cliente web del entorno migrado |
| `common-skills:odoo-flow-coverage-planner` | common | Capa 3: patrón de checklist visual humano |

## Consumed by

| Skill | Suite | Para qué |
|---|---|---|
| `dynapps-migrator` | developer | BLOQUE 4 (validación): gate de paridad previo al go-live |
| `odoo-database-upgrader` | developer | FASE 4: paridad de capacidades además del `compare` de datos |
