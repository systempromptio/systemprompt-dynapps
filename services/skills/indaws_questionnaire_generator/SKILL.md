---
name: indaws-questionnaire-generator
description: >-
  Genera cuestionarios pre-rellenados en Odoo Knowledge para cada aplicación de un proyecto de
  implantación. Lee datos del pedido (toma de datos, painpoints, resumen), personaliza los
  cuestionarios eliminando preguntas irrelevantes y resaltando las pendientes. Soporta la vertical
  de construcción (inBuilt). Usar cuando haya que generar cuestionarios pre-rellenados por app.
version: 2.2.2
effort: medium
model: sonnet
allowed-tools: [Read, Bash, Glob, Grep, common-skills:odoo-pilot]
---

# Generador de Cuestionarios de Implantacion Odoo

Genera cuestionarios de analisis pre-rellenados para cada aplicacion de Odoo que se vaya a implantar. Los cuestionarios se publican como articulos de Knowledge bajo el articulo del proyecto.

Los cuestionarios se generan con informacion extraida del pedido de venta (toma de datos, painpoints, resumen). Las preguntas ya contestadas se marcan, las irrelevantes se eliminan, y las pendientes se resaltan.

## Workflow

See `references/workflow-detail.md` for step-by-step commands and field details.

1. **Recopilar datos** (Paso 0): Pedir ID proyecto + ID pedido de venta. STOP.
2. **Leer pedido** (Paso 1): Via odoo-pilot. Campos criticos: `x_studio_toma_de_datos`, `x_studio_services_ids`, `x_studio_painpoint_ids`, `x_studio_conceptos_proyecto`. Si toma de datos vacia, STOP y ofrecer leer la oportunidad.
3. **Detectar construccion** (Paso 1c): Buscar indicios inBuilt en datos del pedido. Ver `references/construction-vertical.md`.
4. **Leer proyecto y Knowledge** (Paso 2): Verificar/crear articulo Knowledge padre y "Cuestionarios de analisis".
5. **Generar cuestionarios** (Paso 3): Para cada app en `x_studio_services_ids`, leer plantilla de `./templates/`, pre-rellenar, crear articulo Knowledge. Para cubrir comportamientos/edge-cases reales del dominio (limites estandar, gotchas conocidos), apoyarse en `odoo-deep-knowledge` (suite odoo-knowledge) al generar preguntas sobre flujos complejos.
6. **Articulo pendientes** (Paso 4): Crear resumen con todas las preguntas pendientes agrupadas por app.
7. **Resumen** (Paso 5): Presentar estadisticas y proximos pasos.

See references/shared/odoo-connection.md for connection pattern.

### Pre-fill Logic

| Status | Marker | HTML Background |
|--------|--------|-----------------|
| Contestada | `[Pre-rellenado desde toma de datos]` | `#d4edda` (verde) |
| Parcial | `[Parcial - completar en sesion]` | `#ffe0b2` (naranja) |
| Pendiente | `[Pendiente]` | `#fff3cd` (amarillo) |
| Painpoint | `Punto de dolor identificado: {desc}` | `#e3f2fd` (azul) |
| Irrelevante | (eliminada) | -- |

## GOTCHAS

1. **x_studio_toma_de_datos vacio**: Si esta vacio, los cuestionarios salen sin pre-rellenar. Siempre ofrecer leer la oportunidad vinculada antes de continuar en modo basico.
2. **x_studio_services_ids vacio**: Sin este campo no se sabe que cuestionarios generar. Informar y preguntar al usuario.
3. **Cuestionario duplicado**: Si ya existe un articulo Knowledge hijo con nombre similar, crear otro genera duplicados. Siempre verificar antes de crear.
4. **Plantilla inexistente**: Si una app de services_ids no tiene plantilla en `./templates/`, informar y saltar. No inventar preguntas.
5. **Texto de preguntas modificado**: NUNCA cambiar el texto original de una pregunta que se mantiene. Solo eliminar irrelevantes o pre-rellenar.
6. **Construccion no detectada**: Si se omite el paso 1c, los cuestionarios de Ventas/Compras/Facturacion pueden faltar secciones de obra, o se genera Planificacion en vez de Proyectos y Obras.
7. **Respuesta ambigua marcada como contestada**: Si la informacion es parcial, SIEMPRE marcar como parcial, nunca como contestada. Un pre-rellenado incorrecto es peor que una pregunta pendiente.

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | CRUD de knowledge.article, lectura sale.order/project.project | Bash inline |
| odoo-deep-knowledge | odoo-knowledge | Edge-cases y comportamientos reales de dominio al generar preguntas de flujos complejos | Consulta interna en Paso 3 |

## Consumed by

Used manually by consultores at the start of the analysis phase. Output feeds into `project-analysis`.

## References

| File | Content | Load when |
|------|---------|-----------|
| `references/workflow-detail.md` | Pasos detallados, campos, logica pre-rellenado | Always |
| `references/construction-vertical.md` | Reglas deteccion y efecto inBuilt | Paso 1c |

## Templates

14 plantillas en `./templates/`: `configuraciones-generales.md`, `crm.md`, `ventas.md`, `facturacion.md`, `contabilidad.md`, `helpdesk.md`, `integraciones.md`, `compras.md`, `inventario.md`, `tpv.md`, `recursos-humanos.md`, `servicio-externo.md`, `planificacion.md`, `proyectos-obra.md`.
