---
name: indaws-demo-helpdesk-ticket
description: >
  Prepara la respuesta a un ticket de helpdesk de Odoo. Lee el ticket + chatter,
  cruza con el histórico del cliente (tickets previos, último técnico que le
  atendió, proyectos y equipos instalados), identifica el SLA aplicable según
  priority y team, y produce dos artefactos: (1) una nota interna con
  diagnóstico preliminar y asignación sugerida, que se publica como
  mail.message en el chatter del ticket, y (2) un borrador de email al cliente
  (empático, plan de actuación con hitos, cita del SLA). Al finalizar, asigna
  el técnico sugerido (el que más veces ha atendido a ese cliente). Usar
  cuando el usuario pida: responder un ticket, redactar respuesta, preparar
  plan de actuación, atención al cliente.
version: 1.0.2
model: sonnet
effort: medium
---

# Demo · Helpdesk ticket

Asistente de primera respuesta a tickets. Lee contexto, redacta y publica nota interna + sugiere asignación.

## Prerequisitos

- **odoo-pilot** (common) para `helpdesk.ticket`, `helpdesk.sla`, `mail.message`, `res.partner`, `res.users`, `project.project`, `sale.order`
- **Credenciales Odoo** en env
- Python 3.8+ stdlib

## Workflow

| Paso | Accion | Output |
|------|--------|--------|
| 1 | Autenticar. Leer `config/config.json`. | -- |
| 2 | Ejecutar `python scripts/main.py --ticket-id <ID>`. El script lee el ticket + chatter, identifica el SLA aplicable, busca tickets previos del partner y proyectos asociados, detecta el técnico sugerido (el que más ha atendido antes a este cliente) y produce `output/ticket<ID>_analysis.json` + HTML con placeholders. | analysis.json + draft.html |
| 3 | Leer `output/ticket<ID>_analysis.json`. Redactar en `output/ticket<ID>_narrative.json`: (a) **nota interna** para el equipo con diagnóstico preliminar + hipótesis + asignación sugerida + cita del SLA, (b) **borrador de email al cliente** con tono empático, plan de actuación con hitos (diagnóstico remoto + visita si aplica) y compromiso concreto del SLA, (c) **`assigned_user_id`** (opcional override del sugerido automático). | narrative.json |
| 4 | Ejecutar `python scripts/main.py --ticket-id <ID> --finalize`. Aplica el narrative al HTML, **publica la nota interna** como `mail.message` (subtype "Note"), **asigna el técnico** al ticket vía `user_id`, y guarda el HTML final. NO envía el email al cliente (queda como borrador en el HTML). NO cambia el stage. | HTML final + ticket actualizado |
| 5 | Mostrar al usuario: ruta HTML + resumen (ticket asignado a X, SLA crítico de Yh vence a las ZZ:ZZ). | -- |

## Formato esperado del narrative

### `internal_note` (bloque HTML para mail.message interno)
Técnico y preciso. Secciones:
1. **Contexto** (1-2 frases resumen del ticket y cliente)
2. **Diagnóstico preliminar** (3-4 hipótesis razonadas, ordenadas por probabilidad)
3. **Plan sugerido** (diagnóstico remoto + si procede visita presencial, con horas concretas)
4. **SLA aplicable** (cita textual del nombre del SLA y el compromiso temporal)
5. **Asignación propuesta** (nombre del técnico + motivo: "ha atendido N tickets previos de este cliente")
6. **Contexto adicional** (equipos instalados detectados, proyectos cerrados, si aplica)

### `customer_draft` (email al cliente)
Tono empático, sin tecnicismos innecesarios. 3-4 párrafos:
1. Apertura reconociendo el problema y empatía con el impacto
2. Qué vamos a hacer (plan con hitos concretos, horas)
3. Compromiso del SLA ("nuestro contrato establece X, nos comprometemos a Y antes de las HH:MM")
4. Cierre con firma del técnico asignado + contacto directo

### `assigned_user_id` (opcional)
Si se omite, se usa el sugerido automático (técnico más frecuente del cliente). Si el cliente no tiene historial, el sugerido es el del ticket (si lo tiene) o el team_leader.

## RULES

1. **SIEMPRE** leer el chatter del ticket — puede contener información crítica añadida posterior a la creación
2. **NUNCA** envíes email al cliente automáticamente — solo borrador en HTML (la skill solo publica la nota interna)
3. **NUNCA** cambies stage_id en v1 — se hace manualmente tras revisar
4. **SIEMPRE** cita el SLA con su nombre literal en Odoo (ej. "SLA Crítico - Respuesta urgente 4h")
5. Si no hay histórico del cliente (primer ticket), el técnico sugerido es el que esté asignado al team, o el team_leader
6. La nota interna debe ser **HTML seguro** — no metas `<script>`, `<iframe>` ni CSS inline pesado (Odoo puede sanitizar)

## GOTCHAS

1. `helpdesk.sla` en Odoo se matchea por `team_id + priority + stage_id`. Para calcular el SLA aplicable correctamente al ticket: `priority == ticket.priority` (comparación como string "0" / "1" / "2" / "3")
2. `helpdesk.ticket.priority` es `selection`: "0" Normal · "1" Low · "2" High · "3" Urgent — en esta instancia Lumina: "0" Normal, "1" Prioritario, "2" Premium, "3" Crítico
3. `mail.message` con subtype "Note" (subtype_id = 2 en Odoo estándar, **verificar en la instancia**) → interno, no envía email
4. `res.users.partner_id` es el partner del autor (admin → partner_id=3 o similar) — necesario para `author_id`
5. Tickets cancelados (stage 5) y resueltos (stage 4) **cuentan para el histórico** — son referencia de intervenciones pasadas, no solo los abiertos

## Delegates to
- **common:odoo-pilot** — todas las lecturas/escrituras

## References
| File | Content |
|---|---|
| `config/config.json` | Firma, ventanas temporales, política de asignación |
