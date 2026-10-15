---
name: indaws-odoo-flow-coverage-planner
version: 1.0.5
effort: high
model: opus
description: >
  Genera escenarios funcionales de test para los modulos instalados en una instancia Odoo (14-19),
  automatizando solo validaciones API-aserables y derivando las visuales a checklist humano.
  NOT ejecucion de flujos → odoo-flow-tester.
  Triggers: "probar todo lo instalado", "bateria de pruebas funcionales", "coverage post-migracion",
  "planificar tests odoo", "matriz de cobertura modulos", "scenarios funcionales".
---

# Odoo Flow Coverage Planner v1.0

Construye cobertura funcional desde el inventario de modulos **instalados** y la entrega al
`odoo-flow-tester` para que la ejecute. Esta skill **planifica y genera**; NO ejecuta flujos.

**Principio de eficiencia (clave).** Solo automatiza validaciones **API-aserables** (estado,
valores calculados, cascadas, integridad) que son token-baratas para la IA. Las validaciones
**visuales** (maquetacion de PDF, capturas, UI/UX, fidelidad de diseno) son token-pesadas para la
IA y mas rapidas para un humano -> se derivan a `human_visual_checklist.md` y NO se auto-validan.
Detalle y tabla de decision: `references/validation-channels.md`.

Conexion: ver `references/shared/odoo-connection.md`. Operaciones API via `odoo-pilot`.

## Que produce (entregables)

1. `inventory.json` — modulos instalados clasificados (estandar / OCA / custom-Studio).
2. `coverage_matrix.md` — modulo -> funcionalidad -> **canal (IA/humano)** -> flow_id -> estado.
3. `flows/*.yaml` — un flujo por escenario **IA** (contrato real, ver `references/flow-generation.md`).
4. `scope.md` — documento de alcance que consume el `odoo-flow-tester` en su Paso 0/1.
5. `human_visual_checklist.md` — validaciones VISUALES para humano (lo token-pesado, fuera de IA).

## Agent Routing

| Paso | Modelo | Por que |
|------|--------|---------|
| 0 — Conexion + alcance | sonnet | Recopilar requisitos |
| 1 — Inventario instalado | sonnet (subagente) | Lectura mecanica de `ir.module.module` |
| 2 — Mapeo funcionalidades + canal | **opus** | Cruce catalogo + descubrimiento custom + canal IA/humano |
| 3 — Generacion de flujos + checklist | **opus** | Flujos IA (contrato real) + checklist visual humano |
| 4 — Handoff flow-tester | sonnet | Invocacion + paso de artefactos |
| 5 — Consolidacion | sonnet | Informe final de cobertura |

## Workflow (6 Pasos)

### Paso 0 — Conexion + alcance [STOP]
Preguntar: conexion Odoo, version, alcance (todos/apps/custom), modo handoff (A/B/C). Ver `config.json`.

### Paso 1 — Inventario de modulos instalados [SONNET SUBAGENTE]
`scripts/list_installed_modules.sh` sobre `ir.module.module state=installed`. Clasificar: estandar Odoo / OCA / custom-Studio. Guardar `inventory.json`. Ver `references/module-inventory.md`.

### Paso 2 — Mapeo a funcionalidades + canal [OPUS]
Cruzar el inventario con `scripts/catalog_map.json` (modulo -> app -> flujos/cadenas del
catalogo del flow-tester). Para un modulo **custom concreto**, `scripts/map_module_to_flows.py <ruta>`
(determinista) enruta por sus `depends` + modelos `_inherit` a las cadenas E2E que toca — base del
test E2E scoped al modulo (Gate 4 de Desarrollador 2.0). Para inventario fino de custom/Studio, lanzar
`Agent` con `odoo-module-scanner`. Para CADA funcionalidad, asignar **canal**:
`ai` (API-aserable), `human` (visual) o `split` (datos IA + aspecto humano), segun
`references/validation-channels.md`. Resultado: lista de funcionalidades con flujo asignado o
marcado como **gap**, y canal. Detalle: `references/coverage-taxonomy.md`.

### Paso 3 — Generacion de escenarios [OPUS]
- Solo funcionalidades de canal `ai` (y la parte de datos de `split`) generan flujo:
  - Catalogo: referenciar su `flow_id`, que da el catalogo de `odoo-flow-tester` con su `flows.py` (`--list` para flujos, `--chains` para cadenas).
  - Gaps custom: **escribir** `flows/<flow_id>.yaml` en el **contrato real** del executor
    (`references/flow-generation.md`; validar con `odoo-flow-tester/scripts/validate_flow.py`).
- Funcionalidades `human` (y la parte de aspecto de `split`) -> `human_visual_checklist.md`
  (QUE mirar, DONDE, criterio). NO generan flujo ni consumen tokens de IA.
- Generar `coverage_matrix.md`, `scope.md` y `human_visual_checklist.md` desde `templates/`.

### Paso 3.5 — Validacion del plan [STOP]
Presentar `coverage_matrix.md`: % cobertura **automatizable (IA)**, flujos reutilizados/generados,
items derivados al humano, y modulos sin cobertura posible (y por que). Esperar aprobacion.

### Paso 4 — Handoff al odoo-flow-tester [SONNET]
Invocar el skill `odoo-flow-tester` pasando: `scope.md` como documento de analisis, la carpeta
`flows/` como flujos custom, y el **modo** (A/B/C) elegido en Paso 0. Contrato exacto en
`references/handoff-protocol.md`. El flow-tester ejecuta su propio Paso 1.5 (STOP aprobacion) y,
en modo C, Safe Mode + Snapshot + revert.
**OBLIGATORIO en este paso:** notificar al usuario, de forma visible, la lista REAL de
validaciones manuales (canal `human`) pendientes de `human_visual_checklist.md` (formato y reglas
en `references/handoff-protocol.md`). No diferir ni resumir con texto generico.

### Paso 5 — Consolidacion [SONNET]
Recoger los resultados del flow-tester y consolidar en la matriz: por cada funcionalidad `ai`,
estado PASS/FAIL/SKIP + evidencia. Los items `human` quedan como pendientes en
`human_visual_checklist.md` para que la persona los marque. Entregar informe de cobertura final
(automatizado IA + pendiente humano).

## Delegates to

| Skill | Suite | Para que | Via |
|-------|-------|----------|-----|
| odoo-pilot | common | Conexion API + lectura de `ir.module.module` | Bash inline (scripts) |
| odoo-module-scanner | common | Inventario de modulos custom/Studio | Agent |
| odoo-flow-tester | common | Verificar/ejecutar/revertir los flujos generados | Agent (Paso 4) |

## Consumed by

| Skill | Suite | Para que |
|-------|-------|----------|
| odoo-database-upgrader | developer | Cobertura funcional completa post-upgrade (FASE 4 validacion) |
| migration-assistant | common | Bateria de pruebas tras migracion |
| project-validation-drafting | consultant | Identificar flujos a cubrir por app/modulo en modo PLANIFICAR |

## NEVER
1. Ejecutar flujos directamente — esta skill SOLO planifica/genera; ejecuta el flow-tester.
2. Inventar `flow_id` fuera del formato `CHAIN_<AREA>_<DESC>` (el flow-tester los rechaza).
3. Reimplementar el esquema de flujos — el contrato real está en `references/flow-generation.md`; valida con `validate_flow.py`.
4. Hardcodear credenciales/URLs — usar `config.json` o variables de entorno.
5. Generar flujos que creen datos maestros (solo transaccionales). Maestros → `odoo-test-data-generator`.
6. Declarar cobertura "100%" silenciando modulos sin flujo: listar SIEMPRE los gaps.
7. **Generar validaciones VISUALES como flujo IA** — token-pesado → SIEMPRE a `human_visual_checklist.md`.

## ALWAYS
1. Partir del inventario VIVO (`ir.module.module` state=installed), no de suposiciones.
2. **Clasificar cada validacion por canal (ai/human/split)** antes de generar (token-eficiencia).
3. Reutilizar el catalogo del flow-tester antes de generar un flujo nuevo (evita duplicar).
4. Para cada flujo custom incluir `assertions` y `revert`; validar con `validate_flow.py`.
5. Para informes, **partir**: datos -> assert API (IA); aspecto -> checklist humano.
6. Marcar `odoo_version` igual a la version real de la instancia (auto-detectada).
7. Registrar gaps explicitos (modulo instalado sin escenario y motivo).
8. **En CADA handoff notificar la lista real de validaciones manuales pendientes** con Ruta, Cómo probar y criterio — nunca genérico. Ver `references/handoff-protocol.md`.

## GOTCHAS
0. **`flow_schema.md` (prosa) DESINCRONIZADO con el executor real.** El contrato real usa `odoo_version`, `preconditions` como lista plana, `values:` (no `data:`), sin atajos `confirm/cancel`. SIEMPRE generar en contrato real y validar. Ver `references/flow-generation.md` + `examples/`.
1. **Modulos instalados != apps.** `ir.module.module` incluye dependencias tecnicas (base, web,
   mail...) que no tienen flujo funcional. `catalog_map.json` marca cuales son "no testeables".
2. **Custom/Studio no esta en el catalogo.** Esos SIEMPRE generan gaps -> flujos nuevos via
   odoo-module-scanner + autoria manual. No asumir que un `x_studio_*` se prueba solo.
3. **Version-sensibilidad.** Métodos cambian entre versiones (`is_storable`, `action_sheet_move_post`, `plan_id`...). Generar acorde a `target_version`.
4. **El flow-tester re-pide aprobacion (Paso 1.5).** Tu Paso 3.5 aprueba COBERTURA; el suyo aprueba EJECUCION.
5. **Sin datos maestros = precondiciones fallan.** Coordinar con `odoo-test-data-generator` antes del modo B/C.

## References

| File | Content |
|------|---------|
| `references/module-inventory.md` | Enumerar/clasificar modulos instalados |
| `references/validation-channels.md` | **Canal IA vs humano** (token-eficiencia): tabla de decision |
| `references/coverage-taxonomy.md` | Mapeo modulo->funcionalidad->flujo; descubrimiento custom |
| `references/flow-generation.md` | Autoria de flujos (contrato real del executor) |
| `references/handoff-protocol.md` | Contrato exacto de entrega al odoo-flow-tester |
| `scripts/catalog_map.json` | Mapa modulo -> app -> flujos/cadenas del catalogo |
| `templates/` | coverage_matrix, human_visual_checklist, scope_document, flow_scenario.yaml |
| `examples/CHAIN_BDKSALES_PICKING_SECTIONS.yaml` | Ejemplo concreto que PASA `validate_flow.py` |
