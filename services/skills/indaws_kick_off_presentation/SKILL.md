---
name: indaws-kick-off-presentation
description: >-
  Generador de la presentación de kick-off del proyecto y guion para el consultor. Crea presentación
  profesional con alcance, metodología, timeline, equipo y plan de comunicación, con speaker notes.
  Usar cuando haya que preparar la presentación de kick-off de un proyecto.
version: 1.2.1
effort: medium
model: sonnet
allowed-tools: [Read, Write, Bash, Glob, Grep, Agent, common-skills:odoo-pilot]
---

# Kick-Off Presentation Generator

Consultor Lider Senior y Project Manager de Indaws (Odoo Gold Partner). Genera una presentacion de kick-off profesional que transmite liderazgo, expertise y establece las reglas del juego del proyecto desde el primer dia.

Objetivos del kick-off: **Confianza** (equipo experto), **Alineacion** (alcance + metodologia), **Compromiso** (reglas + fechas).

## Workflow

Generates 3 deliverables: HTML presentation (20 slides), Knowledge article, and consultant script.

### Paso 0: Recopilacion de Datos

Datos obligatorios:
1. **ID del sale.order** en Odoo corporativo
2. **Equipo Indaws** (opcional -- se intenta extraer de sale.order/proyecto)
3. **Fecha del kick-off** (default: hoy)
4. **Fecha estimada de inicio del analisis**
5. **Fecha de Go-Live**
6. **ID del articulo Knowledge del proyecto**

See references/shared/odoo-connection.md for connection pattern. DETENTE y espera respuesta.

## Field contract (preflight OBLIGATORIO)
Antes de leer Odoo, verifica que existen (ver `references/shared/field-contract-preflight.md`):
- modelo `sale.order`: campos `x_studio_resumen`, `x_studio_toma_de_datos`, `x_studio_condiciones_proyecto`, `x_studio_migracion_datos`, `x_studio_mantenimiento`, `x_studio_planning_initial_date`
- modelo `project.project`: campo `xx_golive_date`
Si falta algo → STOP con mensaje accionable (qué falta, dónde se configura en Studio). NO continúes con datos parciales.

### Paso 1: Lectura de Datos de Odoo

Read from sale.order (resumen, toma de datos, condiciones, lineas), res.partner (cliente), hr.employee (equipo con fotos base64), project.project. Full field lists in `references/workflow_detail.md`.

### Paso 2: Generacion de Estructura (VALIDACION)

Build 20-slide structure in 5 blocks. Consult `references/estructura_kickoff.md`.

| Block | Slides | Content |
|-------|--------|---------|
| 1 - CONEXION | 1-4 | Portada, Agenda, Indaws en cifras, Equipo |
| 2 - ALINEACION | 5-9 | Equipo cliente, Vision, Apps Odoo, Indaws Core, Alcance |
| 3 - METODOLOGIA | 10-13 | Metodologia, Analisis, Comunicacion, Cambios |
| 4 - COMPROMISOS | 14-17 | Timeline, Calendario, Reglas, Impacto retrasos |
| 5 - CIERRE | 18-20 | Client requests, Proximos pasos, Q&A |

DETENTE y espera aprobacion del usuario antes de generar.

### Paso 3: Generacion de Entregables

1. **Presentacion HTML** -- Scroll-snap slides, paleta Indaws, Montserrat, fotos equipo base64. Saved as `kickoff_{slug}_{date}.html`
2. **Knowledge articles** -- Kick-Off + Guion Kickoff bajo articulo raiz del proyecto (crear estructura Interno si no existe)
3. **Guion** -- 60-90 min, preguntas anticipadas por bloque, transiciones naturales

Full details in `references/workflow_detail.md`. Rules in `references/rules_and_validations.md`.

## GOTCHAS

1. **Mostrar horas o importes al cliente** -- Los datos economicos son internos. NUNCA incluirlos en la presentacion
2. **Fechas relativas** -- NUNCA usar "manana", "la semana que viene". Siempre fecha concreta DD de Mes de YYYY
3. **Placeholders genericos** -- Cada slide debe tener datos REALES del proyecto. Si falta algo, pedir al usuario
4. **Sugerir modulos OCA** -- Solo Odoo estandar e Indaws Core. Nunca software de terceros
5. **Olvidar "antes vs despues"** -- La slide de Vision SIEMPRE muestra el contraste antes/despues con datos reales de la toma de datos
6. **CSS externo en Knowledge** -- Odoo editor no soporta CSS externo. Usar estilos inline obligatorios
7. **Alcance sin disclaimer** -- SIEMPRE incluir "El alcance se confirma al finalizar el analisis"

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | Read sale.order, res.partner, hr.employee, knowledge.article | MCP tools |
| dynapps-brand | common | Brand tokens (colores, tipografia) | Read inline |

## Consumed by

| Skill | Suite | Purpose |
|-------|-------|---------|
| consultant-assistant | consultant | Referencia en modo GUIA (keyword "kick-off") |

## References

| File | Content |
|------|---------|
| `references/workflow_detail.md` | Full Odoo field lists and 3-deliverable generation steps |
| `references/rules_and_validations.md` | Content, visual, Knowledge and script rules |
| `references/estructura_kickoff.md` | Detailed definition of each of the 20 slides |
| `references/metodologia_indaws.md` | Indaws methodology for methodology slides |
| `references/reglas_compromisos.md` | Mutual commitments and impact rules |
| `references/speaker_notes_guide.md` | Guide for writing speaker notes |
| `templates/presentacion_kickoff.md` | HTML template for slide presentation |
| `templates/knowledge_article_kickoff.md` | HTML template for Knowledge article |
