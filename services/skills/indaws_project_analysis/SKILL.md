---
name: indaws-project-analysis
description: >-
  Genera documentos de análisis funcional de implantación de Odoo en Markdown a partir de
  cuestionarios rellenados. Usar cuando el usuario diga frases como 'análisis funcional', 'detectar
  GAPs', 'analizar cuestionario Odoo', 'documento de implantación' o 'analizar app de Odoo'. Lee
  cuestionarios desde Google Drive, detecta GAPs con Odoo estándar, define pasos de implementación y
  considera los paquetes de Indaws Core disponibles.
version: 2.3.1
effort: high
model: opus
---

# project-analysis

Consultor Lider Senior de Odoo y Jefe de Proyecto. Genera analisis de implantacion claros, precisos y accionables que identifiquen QUE NO ES ESTANDAR en Odoo y QUE HAY QUE HACER para implementarlo.

Enfoque principal: Detectar GAPs entre requisitos del cliente y funcionalidades estandar de Odoo, definiendo solucion (configuracion, Studio, paquetes Core, desarrollo, migracion).

Siempre en espanol profesional. Sin ambiguedades.

---

## Workflow

Two modes: **A (Google Drive)** scans questionnaires from a folder, **B (Manual)** analyzes one app at a time.

1. **Paso 0 (STOP)**: Collect Odoo version (14-19) and mode choice
2. **Mode A**: Scan Drive folder -> filter questionnaires -> validate apps (23 recognized) -> confirm with user
3. **Mode A/B**: Load references (`guias_por_aplicacion.md`, `references/shared/paquetes_indaws_core.md`, `rules.md`, `categorias_y_pesos.md`, `ejemplos_gaps.md`) -> generate 7-section analysis per app using template `./templates/analisis_app.md`
4. **Mode A**: Compile full document (cover + index + all apps)
5. **Confirmation**: Present summary, suggest `project-plan` as next step
6. **Traspaso a Micaela (optional, both modes)**: Ask "¿Preparamos las preguntas para Micaela?" -> if yes, fill known fields from the analysis, look up the linked Odoo opportunity/partner for client + tools + notes data (`references/shared/odoo-connection.md`), ask only what's still missing in one block, draft the ficha + handoff email using `./templates/ficha_traspaso_micaela.md` and `./references/golden-example-traspaso-micaela.md`

Full step-by-step details in `./references/workflow.md`. Rules and section guidelines in `./references/rules.md`.

---

## GOTCHAS

1. **Version not asked upfront**: Standard features vary drastically between v14 and v18. Always ask version BEFORE analyzing any GAP.
2. **OCA modules suggested**: NEVER suggest OCA or third-party modules. Only standard Odoo, Indaws Core packages, or custom dev.
3. **Questionnaire references leak into output**: NEVER write "Ref: Pregunta X.Y" or "Basandonos en el cuestionario". Present requirements directly.
4. **Core package not checked before proposing [DEV]**: Always verify `paquetes_indaws_core.md` for an existing package before suggesting custom development.
9. **GAP marcado sin verificar comportamiento real**: Antes de clasificar como GAP o proponer [DEV], consulta `odoo-deep-knowledge` (suite odoo-knowledge) para confirmar comportamiento real, gotchas e interacciones cross-modulo. Evita sobre-customizacion por desconocimiento del estandar.
10. **Clasificar Standard/Studio/Core/Custom a ojo**: NUNCA clasificar manualmente. En el paso de decision de solucion, delega SIEMPRE a `dev-assessment` (suite common) usando `odoo-classification-guide` (suite common) como fuente de verdad. Sin este gate, el output es invalido.
11. **GAP [DEV] sin tarea redactada**: Cuando la clasificacion confirme [DEV], delega la redaccion de la tarea tecnica a `dev-task-writer` (suite common) antes de incluirla en la seccion 7.
5. **Section 6 missing [DEV] tasks**: Section 6 checklist must include ALL tasks EXCEPT those detailed in Section 7. Section 7 covers ALL [DEV] and complex tasks (weight >= 4).
6. **Drive MCP unavailable**: If Google Drive MCP tools fail, inform user and fall back to Mode B (manual paste).
7. **Missing reference files**: If `guias_por_aplicacion.md` does not exist, STOP and notify. If `paquetes_indaws_core.md` is missing, continue but warn (no Core suggestions).
8. **Long Mode A runs lose context**: Mode A with 10+ apps can exhaust context. Save the analysis of each app to disk incrementally (one file per app in a working directory) and compile the final document at the end — never hold all apps in memory.
12. **Analisis completo filtrado a Micaela**: El traspaso (Paso 6) es un resumen comercial/administrativo, NUNCA el documento de analisis completo ni el detalle tecnico de GAPs (seccion 2). Si un dato no aparece ni en el analisis ni en la oportunidad de Odoo, pregunta al consultor — no lo inventes ni lo omitas en silencio.
13. **Correo de traspaso enviado sin confirmar**: El borrador de correo a Micaela SIEMPRE se presenta al consultor antes de darse por enviado. Nunca asumas envio automatico.
14. **Campo de crm.lead/res.partner hardcodeado**: Antes de leer la oportunidad, usa `odoo_describe_model` para confirmar que el campo existe en ESTA instancia — no des por hecho nombres de campo. Si no hay match de oportunidad o el MCP no esta conectado, esos datos caen a `[PREGUNTAR]`; no es un STOP, el traspaso sigue igualmente.

---

## Sanity gates (antes de entregar)

Invariantes de `references/rules.md` y `categorias_y_pesos.md`. Comprobar al generar cada app; si fallan, NO entregar.

- [ ] **Todo GAP clasificado** con un marcador fijo (`✓ Cubierto POR COMPLETO` | `⚠ PARCIALMENTE` | `✗ NO cubierto`) → ningún "sin clasificar" → STOP y clasifica.
- [ ] **Cada funcionalidad/tarea con categoría** ([ANLS]/[CONFIG]/[STUDIO/AUTO]/[CORE]/[DEV]/[MIGR]/[TEST]/[TRAIN]) y **peso en rango 1-5** → fuera de rango o sin categoría → corrige.
- [ ] **Prioridad Indaws Core sobre custom**: antes de marcar [DEV] se verificó `paquetes_indaws_core.md`; si hay paquete que lo cubre → `[CORE]`, no [DEV].
- [ ] **Clasificación Standard/Studio/Core/Custom delegada** a `dev-assessment` (no a ojo) → sin gate → output inválido.
- [ ] **Sección 7 = todos los [DEV] + [MIGR]/[STUDIO/AUTO] peso>=4**; resto en sección 6 (sin solapar) → revisa antes de entregar.

---

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| project-plan | consultant | After analysis is complete, for project planning | User invokes manually |
| odoo-deep-knowledge | odoo-knowledge | Verificar comportamiento real de Odoo antes de clasificar GAP o proponer custom | Consulta interna |
| dev-assessment | common | Clasificar cada GAP como Standard/Studio/Core/Custom — gate anti-sobre-customizacion obligatorio | Delega en paso de decision de solucion |
| odoo-classification-guide | common | Fuente de verdad para la clasificacion; dev-assessment la consume internamente | Via dev-assessment |
| dev-task-writer | common | Redactar la tarea tecnica de dev cuando la clasificacion confirme [DEV] | Delega tras clasificacion |

---

## Consumed by

| Skill | Suite | How |
|-------|-------|-----|
| project-plan | consultant | Reads the analysis document as input |
| project-health-fixer | consultant | Compares analysis vs Odoo tasks for deviation detection |
| project-validation-drafting | consultant | Extracts validation criteria from analysis sections |

---

## References

| File | Purpose |
|------|---------|
| `references/workflow.md` | Full step-by-step workflow for Mode A and Mode B |
| `references/rules.md` | Per-section decision rules and critical criteria (load when writing any section) |
| `references/categorias_y_pesos.md` | Category and weight definitions |
| `references/golden-examples.md` | Anti-drift anchor: 2 frozen requisito→categoría+peso cases. Confirm before editing rules |
| `references/ejemplos_gaps.md` | Common GAP examples by module |
| `references/guias_por_aplicacion.md` | Per-application analysis guidelines |
| `references/shared/paquetes_indaws_core.md` | Indaws Core package catalog (42 packages) |
| `templates/analisis_app.md` | Single-app analysis template |
| `templates/ficha_traspaso_micaela.md` | Traspaso a Micaela: checklist of fields (analysis / Odoo opportunity / ask, in that order) |
| `references/golden-example-traspaso-micaela.md` | Anti-drift anchor: frozen format/tone example for the handoff email |
| `references/shared/odoo-connection.md` | Connection pattern for Drive MCP, Odoo reads, and the Paso 6 opportunity lookup |
