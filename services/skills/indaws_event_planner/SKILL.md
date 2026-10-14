---
name: indaws-event-planner
description: >
  Planificacion y ejecucion end-to-end de eventos de negocio Indaws (conferencias,
  desayunos, talleres). Use this skill when the user planifica, organiza, promociona,
  ejecuta o sigue un evento: setup en Odoo 19 (event.event, event.registration, tickets,
  website_event + project), promocion LinkedIn y email, presupuesto, checklist dia-del-evento
  y follow-up (leads CRM, recap, KPIs). Triggers: "organizar evento",
  "desayuno de negocio", "crear evento en Odoo", "registro de asistentes", "promocionar evento",
  "seguimiento post-evento". 20-24 eventos/ano, 20-50 asistentes, presencial.
version: 1.1.1
effort: medium
model: sonnet
---

# Event Planner

Planifica, ejecuta y haz seguimiento de eventos de negocio Indaws en Odoo 19.

## Workflow

```
IDEA (T-8)  →  PLAN (T-6)   →  PROMOTE (T-4)  →  PREPARE (T-1)  →  EXECUTE (T-0)  →  FOLLOW-UP (T+48h)
 Tema/Fecha    Odoo Event+     Email + 4 posts    Catering+AV+      Checklist        Email gracias →
 Venue/Speaker Project+tasks   LinkedIn           materiales       dia evento       leads CRM → recap → KPIs
```

Detalle semana a semana: [references/event-timeline.md](references/event-timeline.md).

### Pasos (steps)
1. **Briefing** — rellenar ficha del evento (objetivo, KPIs, capacidad, presupuesto). Ver [references/promotion-and-followup.md](references/promotion-and-followup.md).
2. **Setup Odoo** — crear `event.event` (fecha, venue, capacidad, tickets, registro web) + `project.project` con stages y tareas. Script: [scripts/create_event_project.py](scripts/create_event_project.py). API: [references/odoo-event-setup.md](references/odoo-event-setup.md).
3. **Promocion** — 4 posts LinkedIn + 3 emails (DRAFT). Tablas: [references/promotion-and-followup.md](references/promotion-and-followup.md). Tareas: [references/task-templates.md](references/task-templates.md).
4. **Presupuesto** — linea por categoria, estimado vs real. Ver [references/budget-template.md](references/budget-template.md).
5. **Ejecucion** — checklist dia del evento: [templates/event-checklist.html](templates/event-checklist.html), minuto a minuto en [references/day-of-checklist.md](references/day-of-checklist.md).
6. **Follow-up (48h)** — leads CRM, recap LinkedIn, KPIs, reporte [templates/event-report.html](templates/event-report.html). Flujo en [references/promotion-and-followup.md](references/promotion-and-followup.md).

## Modelos Odoo

| Modulo | Modelo | Uso |
|--------|--------|-----|
| Events | event.event | Definicion del evento (draft → confirm → done) |
| Events | event.registration | Registros de asistentes (aforo) |
| Events | event.ticket | Tipos de entrada (gratis / pago / VIP) |
| Projects | project.project / project.task | Gestion de tareas |
| CRM | crm.lead | Leads post-evento |

## Agent Routing

| Tarea | Agente / Skill |
|-------|----------------|
| Operaciones Odoo (crear/leer event.event, registrations, project) | **odoo-pilot** (SONNET) |
| Posts de promocion LinkedIn | linkedin-content-manager |
| Emails de evento (DRAFT) | email-campaign-builder |
| Brand compliance de materiales | dynapps-brand |
| Diseño de collateral | Canva / Figma via ~~design |

## GOTCHAS

- **event.event estados**: draft → confirm → done. Crear queda en `draft`; NO confirmar ni publicar sin que el usuario lo pida. `event_open()`/`button_confirm` cambia estado.
- **event.registration y aforo**: `seats_max` limita registros; al llenarse Odoo bloquea altas. Verifica `seats_available` antes de prometer plazas. Estados de registro: draft/open/done/cancel.
- **Tickets de pago vs gratis**: ticket con precio > 0 requiere flujo de pago (eCommerce/website_sale_event); ticket gratis confirma directo. No mezclar sin avisar.
- **website_event publicacion**: el evento NO sale en la web hasta `is_published=True`. Publicar es accion explicita del usuario, nunca automatica.
- **Recordatorios por email**: los mail schedulers del evento (`event.mail`) pueden enviar solos al confirmar. Crear las comunicaciones en DRAFT y NO activar el envio automatico sin confirmacion.
- **Fechas / timezone**: `date_begin`/`date_end` se guardan en UTC. Define siempre la tz del evento y valida la hora local mostrada; un desfase manda invitaciones con hora errada.

## NEVER / ALWAYS

- **NEVER** confirmar (`button_confirm`), publicar en web (`is_published`) o disparar envios de email sin confirmacion explicita del usuario.
- **NEVER** enviar correos desde Odoo: todo email se crea en estado DRAFT.
- **NEVER** sobrepasar `seats_max` ni gestionar tareas/registros fuera de Odoo.
- **ALWAYS** crear evento y proyecto en Odoo (event.event + project.project).
- **ALWAYS** cada evento con objetivo medible, linea de presupuesto y materiales brand-compliant (dynapps-brand).
- **ALWAYS** follow-up de leads en 48h.

## Delegates to

| Skill | Para |
|-------|------|
| **odoo-pilot** | Toda operacion contra Odoo 19 (event.event, event.registration, event.ticket, project, crm.lead) via JSON-RPC/JSON2 |
| linkedin-content-manager | Promocion (4 posts/evento) |
| email-campaign-builder | Secuencias de email (DRAFT) |
| dynapps-brand | Compliance de materiales |

## Consumed by

| Skill | Suite | Why |
|-------|-------|-----|
| campaign-planning | marketing | Logistica, setup Odoo, promocion y follow-up de eventos de la campana |
| content-collection | common | Almacenar fotos/videos/recursos post-evento |

## Success Criteria

- Evento y proyecto creados en Odoo con tareas por fase.
- Objetivo medible, KPIs y presupuesto definidos.
- Promocion (LinkedIn + email DRAFT) preparada y brand-compliant.
- Checklist dia-del-evento ejecutado; nada confirmado/publicado/enviado sin OK del usuario.
- Follow-up en 48h: leads en CRM con tag, recap publicado, KPIs medidos.

## REFERENCES

- [references/event-timeline.md](references/event-timeline.md) — timeline semana a semana
- [references/odoo-event-setup.md](references/odoo-event-setup.md) — comandos API Odoo (via odoo-pilot)
- [references/task-templates.md](references/task-templates.md) — tareas por fase
- [references/promotion-and-followup.md](references/promotion-and-followup.md) — ficha, LinkedIn, email, roles, leads, KPIs
- [references/budget-template.md](references/budget-template.md) — plantilla de presupuesto
- [references/email-templates.md](references/email-templates.md) — plantillas de email
- [references/day-of-checklist.md](references/day-of-checklist.md) — checklist minuto a minuto
- [scripts/create_event_project.py](scripts/create_event_project.py) — auto-crear proyecto + tareas
- [scripts/generate_report.py](scripts/generate_report.py) — generar reporte post-evento
- [schemas/placeholders.json](schemas/placeholders.json) — placeholders de templates
- [templates/event-report.html](templates/event-report.html) · [templates/event-checklist.html](templates/event-checklist.html) · [templates/base.html](templates/base.html)
- `common:references/shared/` — conexion Odoo y batch ops via odoo-pilot
