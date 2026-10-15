# Mapa de Delegación — apoyarse en toda la suite

Este skill **no reimplementa** nada. Diseña y orquesta; cada parte del análisis y del esqueleto la
produce el builder especializado. Formato portable: **nombre de skill, no rutas** (Claude conoce las
skills instaladas). Si una skill no está disponible en la sesión → hacer esa parte inline y avisar.

---

## Por fase del workflow

### Step 1 — ¿Se debe construir? (gate anti-código)

| Skill | Aporta |
|-------|--------|
| `dev-assessment` | Pirámide Standard→Studio→Core→Custom + estimación técnica |
| `odoo-classification-guide` | Fuente de verdad de clasificación (golden rules) |

### Step 2 — Diseño de la solución

| Skill | Aporta |
|-------|--------|
| `odoo-deep-knowledge` | Gotchas, comportamientos ocultos, interacciones cross-módulo del mecanismo |
| `odoo-flow-tester` | Probe con rollback para verificar el mecanismo central en vivo |
| `odoo-version-api-reference` | Firmas/API correctas de la versión objetivo (sin memoria) |
| `odoo-impact-analyzer` | Blast radius del cambio: qué se ve afectado |
| `odoo-module-scanner` | Inventario de un módulo existente a extender (brownfield) |
| `odoo-context-briefing` | Inventario de la instancia/custom del cliente antes de diseñar |
| `indaws-core-developer` | Arquitectura indaws_core (5 capas, deps) si el diseño toca la suite |
| `odoo-code-auditor` | Auditar el módulo de referencia antes de apuntar el dev a él |
| `tl-assistant` | Decision-frameworks (build vs configure, one/two-way door) y ADR upstream |

### Step 3 — Generar el esqueleto (cada parte a su builder)

| Parte del esqueleto | Skill |
|---------------------|-------|
| Modelos, campos, relaciones, ORM | `odoo-orm-modeler` |
| ACL `ir.model.access.csv` + record rules | `odoo-security-designer` |
| Vistas (form/list/search/kanban) | `odoo-view-designer` |
| Wizards (`TransientModel`) | `odoo-wizard-builder` |
| Componentes JS/OWL | `odoo-owl-component-builder` |
| Reports QWeb | `odoo-qweb-report-builder` |
| Datos XML / seed | `odoo-data-files` |
| Integración mail/chatter/actividades | `odoo-mail-integration` |
| Scripts de migración pre/post/end | `odoo-module-migration-scripts` |
| Scaffolding real del módulo desde el diseño | `odoo-module-creator` |

> Los **tests (unit + e2e) los escribe el dev**, no nosotros. Nosotros damos los **criterios de
> aceptación** (QUÉ verificar) y **exigimos** la cobertura en el DoD. El dev puede apoyarse en
> `odoo-test-generator` para escribirlos — es su herramienta, no parte de nuestro esqueleto.

### Step 4/5 — Empaquetar y estimar

| Skill | Aporta |
|-------|--------|
| `dev-task-writer` | Envuelve diseño+esqueleto en tarea formal (7 secciones, impacto, criterios) |
| `odoo-effort-estimator` | Horas por componente + banda de riesgo + factor IA |
| `dynapps-brand` + `pdf-renderer` | PDF corporativo del entregable (solo tras OK) |

---

## Cómo delegar (patrón)

- **Pattern A (análisis/scan)**: lanzar la skill de análisis, recoger el resultado estructurado,
  usarlo como input del diseño. Ej: `odoo-module-scanner` antes de diseñar una extensión.
- **Pattern B (builder por parte)**: por cada parte del esqueleto, invocar su builder con el
  contrato ya decidido (el builder produce el stub, no re-decide arquitectura).
- **Pattern C (post-diseño)**: `dev-task-writer` + `odoo-effort-estimator` al final, con el diseño
  cerrado.

Regla: **el arquitecto decide; el builder ejecuta la decisión**. Nunca al revés (un builder no fija
la arquitectura). Si un builder propone algo que cambia el diseño → vuelve a Step 2, no se parchea.
