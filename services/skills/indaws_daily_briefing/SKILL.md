---
name: indaws-daily-briefing
description: >
  Briefing matutino priorizado para el equipo comercial de Indaws.
  Extrae datos en tiempo real del CRM de Odoo (pipeline, reuniones, actividades
  pendientes, alertas) y genera un resumen accionable del dia.
  Usar cuando el usuario pida: briefing, resumen del dia, que tengo hoy,
  morning briefing, mi agenda comercial, prioridades del dia.
version: 1.0.2
model: sonnet
effort: low
---

# Daily Briefing

Briefing matutino priorizado que extrae datos en tiempo real del CRM de Odoo y genera un resumen accionable para el comercial.

## Workflow

### Paso 0: Identificar Comercial
PREGUNTAR al usuario: para quien es el briefing (nombre o ID). Si no especifica, generar briefing general del equipo.

### Paso 1: Extraer Datos de Odoo
See references/shared/odoo-connection.md for connection pattern.
Usando odoo-pilot, obtener:
- **1.1 Oportunidades** (crm.lead): dominio `[["user_id","=",{user_id}],["active","=",true],["type","=","opportunity"]]`
- **1.2 Actividades pendientes** (mail.activity): dominio `[["user_id","=",{user_id}],["date_deadline","<=","{hoy}"]]`
- **1.3 Reuniones de hoy** (calendar.event): dominio con start entre 00:00 y 23:59 de hoy
- **1.4 Oportunidades cerradas este mes** (crm.lead): stage_id.is_won=true, date_closed >= primer dia mes

### Paso 2: Analizar y Priorizar
Calcular: pipeline activo, oportunidades urgentes (deadline <= 7 dias o prob >= 70%), estancadas (sin `last_activity_date` en >10 dias; cascada `date_last_stage_update` → `write_date`), cerrando pronto, actividades vencidas, revenue cerrado mes.

### Paso 3: Generar Briefing
Usar template de `references/briefing_template.md` con las reglas de priorizacion incluidas.

## RULES

- **NEVER** inventes datos. Todos los numeros vienen de Odoo
- Las acciones deben ser concretas y especificas (no "hacer seguimiento" sino "llamar a X para confirmar presupuesto")
- Prioridad #1 debe ser UNA sola accion, la mas impactante del dia
- Si no hay reuniones, indicar "Sin reuniones programadas"
- Si no hay alertas, indicar "Pipeline saludable - sin alertas"
- Formato limpio, sin emojis, sin markdown excesivo

## GOTCHAS

1. calendar.event usa partner_ids (Many2many) — buscar con `[["partner_ids","in",[partner_id]]]`
2. Las actividades vencidas (date_deadline < hoy) son MAS urgentes que las de hoy
3. Oportunidades "estancadas" = sin `last_activity_date` (chatter) reciente, no `write_date`: lo mueve cualquier escritura automatica y da falsos estancados
4. El modo "Equipo" necesita iterar por todos los user_ids del equipo comercial
5. expected_revenue puede ser 0 en leads tempranos — no incluir en calculo de pipeline

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Fetch CRM data (crm.lead, mail.activity, calendar.event) |

## Consumed by
No other skill consumes daily-briefing directly.

## References

| File | Content |
|------|---------|
| [references/briefing_template.md](references/briefing_template.md) | Template del briefing + reglas de priorizacion + modos de ejecucion |
