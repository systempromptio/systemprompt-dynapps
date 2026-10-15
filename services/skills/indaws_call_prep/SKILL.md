---
name: indaws-call-prep
description: >
  Genera fichas de preparacion de llamadas y reuniones comerciales.
  Investiga la cuenta en Odoo CRM (historial de oportunidades, notas, actividades),
  enriquece con Clay (contactos, empresa, tech stack) y genera agenda, preguntas
  de descubrimiento, objeciones probables y contexto competitivo.
  Usar cuando el usuario pida: preparar llamada, prep call, preparar reunion,
  call prep, ficha de cuenta, preparar demo, agenda de reunion.
version: 1.0.2
model: sonnet
effort: low
---

# Call Prep

Genera fichas de preparacion completas para llamadas y reuniones comerciales, combinando historial de Odoo CRM con enriquecimiento de Clay.

## Workflow

### Paso 0: Recopilacion de Informacion
PREGUNTAR al usuario:
1. **Con quien es la reunion** (nombre de empresa, contacto o ID de oportunidad en Odoo)
2. **Tipo de reunion**: discovery, demo, propuesta, seguimiento, negociacion
3. **Asistentes conocidos** (nombres, cargos si los tiene)
4. **Objetivo de la reunion** (que quiere conseguir)

> **DETENTE** y espera respuesta del usuario.

### Paso 1: Investigar en Odoo
See references/shared/odoo-connection.md for connection pattern.
Usando odoo-pilot, extraer:
- **1.1 Oportunidad actual** (crm.lead): name, stage_id, expected_revenue, probability, description, partner_id, contact_name, email_from, phone, tag_ids, source_id, create_date, date_deadline
- **1.2 Historial con la cuenta** (crm.lead todos): dominio `[["partner_id","=",{partner_id}]]` — ganadas, perdidas, activas
- **1.3 Notas internas** (mail.message): ultimos 10 mensajes del lead
- **1.4 Actividades previas** (mail.activity completadas)
- **1.5 Pedidos anteriores** (sale.order si es cliente existente)

### Paso 1.5: Enriquecimiento Clay (si disponible)
Si Clay MCP esta conectado: buscar empresa y asistentes. Extraer: cargo, LinkedIn, tamano, sector, tech stack, funding.

### Paso 2: Generar Ficha de Preparacion
Usar template de `references/ficha_template.md` con agendas de `references/agendas_por_tipo.md`.

## RULES

- **NEVER** inventes historial. Solo usar datos reales de Odoo
- Las preguntas de descubrimiento deben adaptarse al contexto (no genericas)
- Si no hay historial en Odoo, indicar "Primera interaccion - sin historial previo"
- Las objeciones deben ser realistas para el mercado Odoo espanol
- Si Clay no esta conectado, omitir secciones de tech stack y senales
- El formato debe ser limpio y scaneable (el comercial lo leera 5 min antes de la reunion)

## GOTCHAS

1. Clay MCP no siempre esta disponible — disenar el output para funcionar sin el
2. Un partner puede tener multiples oportunidades — listar TODAS, no solo la activa
3. Las notas de mail.message pueden contener HTML — limpiar antes de presentar
4. Los sale.order pueden ser de servicios anteriores (no solo implantaciones Odoo)
5. Si el usuario da solo un nombre de empresa, buscar primero en res.partner y luego en crm.lead

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Fetch CRM data (crm.lead, mail.message, sale.order) |

## Consumed by
No other skill consumes call-prep directly.

## References

| File | Content |
|------|---------|
| [references/ficha_template.md](references/ficha_template.md) | Template completo de la ficha de preparacion |
| [references/agendas_por_tipo.md](references/agendas_por_tipo.md) | Agendas detalladas por tipo de reunion (5 tipos) |
