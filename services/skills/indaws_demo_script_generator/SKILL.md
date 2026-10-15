---
name: indaws-demo-script-generator
description: >-
  Genera guiones de demo personalizados para oportunidades comerciales en Odoo y agendas
  profesionales para el cliente. Usar cuando pidan un guion de demo o una agenda para una
  oportunidad.
version: 1.1.1
model: sonnet
effort: medium
---

# Demo Script Generator

Genera un **guion interno de demo** (Markdown -> HTML) para el consultor y una **agenda profesional HTML** para el cliente, a partir de una oportunidad CRM en Odoo. Ambos entregables se guardan sobre el lead: el guion como nota interna (`mail.message`) **y** como adjunto HTML (`ir.attachment`); la agenda como nota interna. No se escribe en ningun campo del lead.

Complementa a `demo-builder`: este skill genera el GUION; `demo-builder` genera los DATOS de demo.

## Workflow

> Detalle completo de cada paso: `references/workflow_detail.md`

See references/shared/odoo-connection.md for connection pattern.

- **Paso 0**: Recopilar datos CRM (lead ID/nombre, duracion, fecha, instancia demo opcional, contexto). **STOP** -- esperar respuesta usuario.
- **Paso 1**: Extraer datos de Odoo via `./scripts/fetch_opportunity_data.sh` (lead, partner, tags, chatter, toma_de_datos, adjuntos). Descargar y leer PDFs/documentos adjuntos.
- **Paso 2**: Analizar pain points (prioridad: toma_de_datos > adjuntos > description > tags). Leer references (pain_to_demo_mapping, flujos_demo_por_app, demo_best_practices). Seleccionar max 5 flujos + momentos WOW.
- **Paso 2.5**: **VALIDACION** -- Presentar plan (cliente, sector, flujos, estructura temporal, estado datos demo). **STOP** -- esperar aprobacion.
- **Paso 3**: Generar guion interno Markdown usando template `./templates/guion_demo.md` y framework F.A.V.O.R.I.T.E.
- **Paso 4**: Generar agenda cliente HTML usando template `./templates/agenda_cliente_html.md`. Items = resultados de negocio, NO funcionalidades.
- **Paso 4.5**: **VALIDACION** -- Presentar guion + agenda + plan almacenamiento. **STOP** -- esperar aprobacion.
- **Paso 5**: Guardar sobre el lead: guion HTML como nota interna (`mail.message`, subtype_id=2) **y** como adjunto (`ir.attachment`, `text/html`) + agenda HTML como nota interna (`mail.message`, subtype_id=2). Ninguna escritura a campos del lead.

## RULES

### Interaccion
- **SIEMPRE** pedir datos obligatorios antes de empezar (Paso 0) y esperar respuesta
- **OBLIGATORIO**: 2 validaciones con STOP (Paso 2.5 y 4.5) -- nunca saltarlas
- **NUNCA** escribir en Odoo sin aprobacion explicita del usuario
- **NUNCA** inventar datos del cliente; si falta informacion, preguntar

### Fuentes de datos (orden de prioridad)
1. `xx_data_collection` (toma de datos)
2. Adjuntos relevantes (PDFs, Word, Excel, PPT) -- **NUNCA ignorar adjuntos**
3. `opportunity.description`
4. `chatter_notes`
5. Tags y sector del partner

### Almacenamiento
- El guion se guarda **dos veces sobre el lead**: nota interna (`mail.message`, subtype_id=2) y adjunto HTML (`ir.attachment`, `res_model='crm.lead'`, `mimetype='text/html'`)
- **NUNCA** escribir en campos del lead ni crear campos nuevos en `crm.lead` -- modelo compartido por los cinco paises del grupo
- **El historico se acumula, no se pisa**: cada ejecucion crea una nota y un adjunto nuevos con su fecha. Nunca preguntes por sobrescribir ni borres guiones anteriores
- La nota del guion empieza con el titulo `Guion de demo — <cliente> — <fecha demo>`, para distinguirla de la nota de la agenda y poder buscarla

### Guion interno (para el consultor)
- Talking points en 2a persona ("ustedes", "su equipo")
- F.A.V.O.R.I.T.E. completo para momentos WOW, simplificado para el resto
- Framing before/after por bloque, precauciones y plan B incluidos

### Agenda cliente
- NO mencionar: "Odoo", "ERP", "modulo", jerga tecnica
- SI usar: "la solucion", "la plataforma", "el sistema"
- Cada item = resultado de negocio. Ultimo item siempre = "Preguntas y proximos pasos"
- 3-5 items maximo

### HTML (ambos entregables)
- Estilos inline obligatorios (Odoo no procesa CSS externo)
- Color principal: `#875A7B` (violeta Odoo)
- Tablas con bordes y padding

### Estructura temporal
- **30 min**: Apertura 5 + Demo 20 (max 2 flujos) + Cierre 5
- **60 min**: Apertura 10 + Demo 40 (3-4 flujos) + Q&A 5 + Cierre 5
- **90 min**: Apertura 10 + Demo1 35 + Pausa 5 + Demo2 25 + Q&A 10 + Cierre 5
- **NUNCA** mas de 5 flujos (feature overload -> sugerir 2 sesiones si 6+ apps)

## GOTCHAS

1. El guion NO se guarda en ningun campo del lead -- el campo de Studio se retiro y no tiene sustituto. Va como nota interna + adjunto. **NUNCA** crees un campo nuevo en `crm.lead`: es un modelo compartido por los cinco paises del grupo
2. `mail.message` requiere `subtype_id: 2` para nota interna (NO se envia al cliente)
3. Adjuntos del lead son la fuente mas rica y la mas ignorada -- descargar SIEMPRE los PDFs/docs
4. Cada ejecucion **acumula**: anade una nota y un adjunto nuevos, fechados. No se sobrescribe nada, no se pregunta por sobrescritura. `has_previous_scripts` solo informa de que ya hubo guiones antes
5. Busqueda por nombre puede devolver multiples leads -- presentar lista para seleccion
6. Si `description_empty` y no hay toma_de_datos ni adjuntos, avisar que el guion sera generico
7. HTML debe ser auto-contenido con estilos inline -- Odoo ignora CSS externo y clases

## Agent Routing

- Todo el workflow corre en el modelo de la sesión; conversacional, con 2 STOP obligatorios

### Agent() condicional — solo si se necesita configurar instancia demo

Si el guion requiere datos de demo que aun no existen en la instancia, delegar a demo-builder (que a su vez puede invocar odoo-configurator). Esto es raro — normalmente el guion se crea ANTES de la demo.

**NO es Agent()**: odoo-pilot (Bash inline — fetch_opportunity_data.sh, create/update records).

## Delegates to

| Skill | Suite | How | Agent()? |
|-------|-------|-----|----------|
| odoo-pilot | common | Bash inline (scripts) | No |
| odoo-configurator | consultant | Config demo (opcional, raro) | **Condicional** |

## Consumed by

No other skill consumes demo-script-generator directly.

## References

| File | Content |
|------|---------|
| `references/workflow_detail.md` | Pasos 0-5 completos, reglas detalladas, validaciones, success criteria |
| `references/pain_to_demo_mapping.md` | Tabla pain point -> modulo Odoo -> flujo demo recomendado |
| `references/flujos_demo_por_app.md` | Catalogo de flujos demo por aplicacion Odoo con pasos y momentos WOW |
| `references/demo_best_practices.md` | Framework F.A.V.O.R.I.T.E., framing, mejores practicas de demo |
| `templates/guion_demo.md` | Template Markdown para el guion interno del consultor |
| `templates/agenda_cliente_html.md` | Template HTML para la agenda del cliente |
| `scripts/fetch_opportunity_data.sh` | Script de extraccion de datos de oportunidad via odoo-pilot |
