---
name: indaws-odoo-migration-pitch
description: >
  Genera presentacion PDF de 10 paginas para vender migracion Odoo a clientes existentes,
  combinando auditoria real del sistema (odoo-pilot), analisis de codigo custom (GitHub) y
  clasificacion de modulos Indaws Core. NOT incluye presupuesto ni horas → quotation-builder.
  Triggers: propuesta de migracion, pitch de migracion, presentacion de upgrade,
  vender migracion, upsell migracion, migration pitch,
  diagnostico comercial de migracion, analisis previo comercial upgrade Odoo.
version: 1.1.1
model: opus
effort: high
---

# Odoo Migration Pitch

Presentacion PDF de 10+ paginas con auditoria real del Odoo del cliente y analisis de codigo clonado. Persuasion comercial dirigida al decision-maker (CTO/CFO/CEO).

## Workflow

READ `references/workflow_detail.md` before executing any step — it contains all queries, HTML snippets, placeholder lists, and validation templates.

See references/shared/odoo-connection.md for connection pattern.

- **Paso 0 — Recopilacion**: Preguntar cliente, version actual, repo GitHub, credenciales Odoo read-only, coste licencias (opcional), version destino (default v19), datos consultor. STOP y esperar confirmacion.
- **Paso 1 — Clonar repos**: Ejecutar `scripts/clone_indaws_repos.sh` para repos Indaws Core + clonar repo cliente. Paralelizar con Paso 2.
- **Paso 2 — Auditoria Odoo read-only**: Confirma con el usuario que las credenciales son de solo lectura antes de conectar. Lanza la extraccion como subagente sonnet (ver Agent Routing): via odoo-pilot, consultar modulos instalados, Studio (campos+vistas), automatizaciones, informes, usuarios. Calcular complexity score. Presentar resumen → VALIDACION 1. STOP y esperar aprobacion.
- **Paso 3 — Clasificacion de modulos**: Escanear `__manifest__.py` del cliente. Clasificar cada modulo Custom/OCA con prioridad: Indaws Core > Studio > Standard (release notes) > Obsoleto > Migrar. Generar tabla agrupada (Eliminar/Reemplazar/Evaluar/Migrar) → VALIDACION 2. STOP y esperar aprobacion.
- **Paso 4 — Generar PDF**: Cargar referencias (sales_arguments, version_improvements, indaws_ecosystem). Calcular timeline dinamico via reglas_timeline. Rellenar 60+ placeholders del template HTML. Invocar `/dynapps-brand`. Convertir a PDF via Playwright (A4, printBackground, margins 0) → VALIDACION 3.

## Prerequisites

- **odoo-pilot**: Conexion read-only al Odoo del cliente
- **Credenciales Odoo**: ODOO_URL, ODOO_DB, ODOO_KEY, ODOO_USER del cliente
- **SSH GitHub Indaws**: Para clonar repos Core
- **dynapps-brand**: Directrices de marca para HTML
- **Playwright**: Conversion HTML a PDF

## RULES

1. **NUNCA** escribas en el Odoo del cliente — SOLO LECTURA (`search_read` unicamente)
2. **NUNCA** inventes datos de modulos — solo los que reporta `ir.module.module` con `state=installed`
3. **NUNCA** generes el PDF sin las 3 validaciones previas aprobadas por el usuario
4. **SIEMPRE** clona los repos Indaws Core — no asumas funcionalidad sin verificar codigo real
5. **SIEMPRE** usa `/dynapps-brand` antes de generar el HTML
6. **SIEMPRE** usa Playwright para conversion HTML a PDF
7. **NUNCA** incluyas precios, horas ni estimaciones economicas — eso es `/quotation-builder`
8. **SIEMPRE** incluye la seccion del 25% de incremento de licencias como argumento de urgencia
9. **SIEMPRE** genera tarjetas de mejoras solo para apps instaladas del cliente
10. **NUNCA** muestres LOC, APIs deprecadas ni detalles tecnicos en la salida comercial
11. **SIEMPRE** aplica Reglas de Oro: Import/Export → Core, Dashboards → Spreadsheets
12. Un modulo es Indaws Core solo si lo confirma el prefijo `indaws_` o el `author`; la URL del cliente sale siempre de `ODOO_URL`, nunca escrita a mano
13. El PDF esta dirigido al **decision-maker** — tono profesional y persuasivo

## GOTCHAS

1. **Studio volume thresholds**: <30 campos = Bajo, 30-80 = Medio, >80 = Alto. No confundir campos con vistas.
2. **Complexity score formula** usa pesos diferentes: custom_modules*15, oca*20, studio_fields*0.3. Ver workflow_detail.md.
3. **Timeline fallback**: Si no hay datos suficientes del repo, usar 8+6+2=16 semanas. Las horas son INTERNAS, el PDF solo muestra semanas.
4. **page-flow vs page-fixed**: Mapa de Modulos usa `page-flow` (altura auto). Portada y Proximos Pasos usan `page-fixed` (297mm exacto).
5. **Clasificacion orden estricto**: Core > Studio > Standard > Obsoleto > Migrar. Saltarse el orden genera clasificaciones incorrectas.
6. **OCA modules sin version destino**: Se cuentan como oca_sin_version en complexity score (peso 20, el mas alto).
7. **Consolidar modulos relacionados**: Agrupar en una fila cuando tengan la misma accion para no inflar la tabla.
8. **OCA por `website`, no solo por `author`**: algunos partners OCA ponen su nombre en `author`; `website` con `github.com/OCA` es la señal fiable.
9. **Campos Studio**: `[["state","=","manual"]]` excluye los de Odoo base pero puede traer campos de modulos OCA; cuenta como Studio solo los `x_`/`x_studio_`.
10. **`ir.actions.report` trae los informes estandar**: filtra por `report_name` que no empiece por modulos estandar (account, sale, stock, purchase, mrp...).
11. **Playwright ausente**: el HTML se genera pero el PDF falla sin error. Comprueba `playwright --version` antes del Paso 4.
12. **Coste de licencias**: es opcional, pero sin el el argumento del 25% cae a una formula generica. Pideselo al comercial.
13. **Repo del cliente privado**: si el clonado falla por autenticacion, pide acceso o token al usuario.

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Paso 0-1 | opus | Recopilacion critica + setup |
| Paso 2 | **sonnet (subagente)** | Extraccion mecanica read-only: auth + search_read. El resumen y la VALIDACION 1 los hace el principal |
| Paso 3 | opus | Analisis de codigo, decision de clasificacion |
| Paso 4 | opus | Generacion HTML complejo, 60+ placeholders |

### Agent() real — Pattern C "Post-action"

En Paso 4, tras generar el HTML completo, invocar pdf-renderer como subagente:

```
Agent({
  description: "Generate migration pitch PDF",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Convert {customer_name}_migration_pitch.html to PDF using the pdf-renderer skill (common-skills:pdf-renderer). Format: A4, printBackground: true, margins: 0. Save as {customer_name}_migration_pitch.pdf"
})
```

En Paso 2, la extraccion va en un subagente (Pattern A) para no llenar el contexto principal con resultados crudos del ORM:

```
Agent({
  description: "Read-only Odoo audit for migration pitch",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Using common-skills:odoo-pilot with ODOO_URL/ODOO_DB/ODOO_USER/ODOO_KEY, run ONLY search_read (never create/write/unlink) and follow the Paso 2 queries in {skill_dir}/references/workflow_detail.md. Return JSON: installed modules (name, shortdesc, author, website), Studio fields and views counts, automations (base.automation + ir.cron), custom reports, active users, version."
})
```

**NO es Agent()**: dynapps-brand (Read brand tokens).

## Delegates to

| Skill | Suite | How | Agent()? |
|-------|-------|-----|----------|
| odoo-pilot | common | Paso 2 — dentro del subagente de auditoria | **Yes — Pattern A** |
| dynapps-brand | common | Paso 4 — Read brand-tokens.json | No |
| pdf-renderer | common | Paso 4 — HTML→PDF via Playwright | **Yes — Pattern C** |

## Consumed by

| Skill | Suite | When |
|-------|-------|------|
| quotation-builder | sales | Opcional — usa datos de modulos clasificados para presupuesto |

## References

| File | Content | Read at |
|------|---------|---------|
| `references/workflow_detail.md` | Full step detail: queries, HTML, placeholders, validations, success criteria | Before any step |
| `references/sales_arguments.md` | Argumentos de venta: 25% sobrecoste, deuda tecnica, seguridad | Paso 4 |
| `references/version_improvements.md` | Mejoras por version y por app (tarjetas HTML) | Paso 4 |
| `references/indaws_ecosystem.md` | Indaws Cloud, Core, metodologia | Paso 4 |
| `references/module_classification.md` | Reglas de clasificacion de modulos | Paso 3 |
| `references/reglas_timeline.md` | Calculo dinamico de semanas por fase | Paso 4 |
| `references/odoo-versions/ODOO_{V}_RELEASE_NOTES.md` | Release notes de la version destino | Paso 3 |
| `templates/migration_pitch.html` | Template HTML base (bento + glassmorphism) | Paso 4 |
| `scripts/clone_indaws_repos.sh` | Clonado automatico de repos Indaws Core | Paso 1 |
