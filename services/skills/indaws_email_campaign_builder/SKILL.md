---
name: indaws-email-campaign-builder
version: 1.1.1
effort: medium
model: sonnet
description: >-
  Crea campañas de email marketing en Odoo 19 (Email Marketing + Marketing Automation, JSON-RPC)
  con branding Indaws. Todo se crea en estado DRAFT — nada se envía sin confirmación humana.
  Cubre: campaña UTM, segmentación de audiencia, secuencias de email, plantillas HTML, flujos de
  automation y ghost follow-up.
  Triggers: "crea una campaña de email", "secuencia de nurturing", "email marketing Odoo",
  "marketing automation", "flujo de emails", "campaña de reactivación", "webinar email", "/email-sequence".
---

# Email Campaign Builder

Crea campañas de email marketing en Odoo 19 con branding Indaws. **Todo se crea en estado `draft`**;
nada se envía sin confirmación explícita del usuario. Contenido en español (es-ES).

## Agent Routing

| Phase | Model | Runs as |
|-------|-------|---------|
| Briefing + diseño de flujo | sonnet | este skill |
| Generación HTML / plantillas | sonnet | este skill (delega render complejo a `odoo-mailing-creator`) |
| Escritura en Odoo (campaña/mailing/lista/automation) | sonnet | `common-skills:odoo-pilot` |

## Workflow

1. **Briefing** — objetivo, audiencia, producto, CTA. Elegir tipo de flujo (tabla en `references/workflow.md`).
2. **Flujo automático** — diseñar secuencia y triggers de Marketing Automation. Plantillas: `references/flow-templates.md`.
3. **Contenido HTML** — generar body con estructura del editor Odoo (`references/workflow.md`) + plantillas en `assets/templates/`. Subject/preheader: `references/email-best-practices.md`. Placeholders: `schemas/placeholders.json`.
4. **Ghost follow-up** (opcional) — actividades comerciales paralelas. Detalle: `references/ghost-followup.md`.
5. **Crear en Odoo (DRAFT)** — vía scripts (ver REFERENCES) o `odoo-pilot`. Modelos: `references/odoo-email-models.md`.
6. **Verificar + entregar** — comprobar `body_arch` + `body_html`, entregar URLs directas y checklist comercial (`references/workflow.md`).

Ejemplos reales (Migración V16, Webinar BI) y checklist pre-activación: `references/workflow.md`.

## GOTCHAS

- **`mailing.mailing` vs `marketing.campaign`** — usar Marketing Automation (`marketing.campaign` + `marketing.activity`) SOLO cuando hay flujos concatenados con triggers; un envío único es un `mailing.mailing` simple.
- **`body_arch` Y `body_html`** — escribir SIEMPRE ambos con el mismo valor. Si falta `body_arch` el editor sale vacío; si falta `body_html` el email sale vacío.
- **Estado DRAFT vs sent** — el mailing nace en `draft`. NUNCA cambiar a `sending`/`done` ni llamar a `action_send_mail` sin confirmación explícita del usuario.
- **Listas, opt-out y GDPR** — verificar que la `mailing.list` no incluye contactos con `opt_out=True`; respetar bajas. No importar contactos sin base legal.
- **A/B test** — Odoo reparte por porcentaje y solo envía la variante ganadora al resto; configurar `ab_testing_*` antes de poblar la lista, no después.
- **Plantillas QWeb de email** — `{{ object.field }}` se renderiza por destinatario; un placeholder mal escrito rompe el render silenciosamente. Validar con `scripts/validate_template.sh`.
- **Rate / throttle de envío** — Odoo envía por lotes vía cron `ir.cron` mail queue; un mailing grande no sale instantáneo. No asumir entrega inmediata al confirmar.
- **Videos YouTube** — los emails no reproducen vídeo; usar thumbnail + link (`references/workflow.md`).

## NEVER

- NUNCA enviar un mailing ni activar un flujo de automation sin confirmación explícita del usuario.
- NUNCA crear nada fuera del estado `draft`.
- NUNCA incluir contactos con `opt_out=True` ni saltarse opt-out / GDPR.
- NUNCA escribir solo `body_html` o solo `body_arch`.
- NUNCA usar emojis en contenido oficial ni vocabulario prohibido (`references/workflow.md`).

## ALWAYS

- SIEMPRE crear campañas, mailings y flujos en estado `draft`.
- SIEMPRE presentar un resumen con URLs directas para revisión humana antes de cualquier envío.
- SIEMPRE escribir `body_arch` y `body_html` juntos.
- SIEMPRE aplicar branding Indaws (colores, tono, cero emojis) y revisar vocabulario.
- SIEMPRE validar la lista de contactos contra opt-out / GDPR antes de asociarla.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| `odoo-pilot` | `common-skills` | Autenticar y escribir en Odoo (campaña/mailing/lista/automation) |
| `odoo-mailing-creator` | `ai-claude-marketing` | Render HTML email complejo, cross-client / Outlook-safe |
| `brand-voice` | `ai-claude-marketing` | Adaptación de tono y vocabulario del contenido |
| `dynapps-brand` | `common-skills` | Colores, tipografía, identidad visual |

## Consumed by

| Skill | Suite | When |
|-------|-------|------|
| `campaign-planning` | `ai-claude-marketing` | Email como canal dentro de un plan de campaña |
| `event-planner` | `ai-claude-marketing` | Secuencias de invitación y follow-up de eventos |

## Success Criteria

Mailing(s) creados en `draft`, con `body_arch` + `body_html` poblados, lista validada (opt-out/GDPR),
subject < 50 chars, CTA con URL válida, branding Indaws aplicado, y resumen con URLs entregado para
revisión humana. Cero envíos sin confirmación.

## REFERENCES

| File | Contents |
|------|----------|
| `references/workflow.md` | Flujo visual, estructura HTML editor Odoo, ejemplos reales, vocabulario, checklist |
| `references/flow-templates.md` | Plantillas de flujos por tipo de campaña |
| `references/ghost-followup.md` | Actividades comerciales ghost (LinkedIn, llamada, WhatsApp) |
| `references/odoo-email-models.md` | Modelos Odoo: utm.campaign, mailing.mailing, marketing.* |
| `references/email-best-practices.md` | Subject lines, preheader, best practices de email |
| `references/youtube-casos-exito.md` | Playlist de casos de éxito para insertar en emails |
| `schemas/placeholders.json` | Placeholders / variables QWeb disponibles |
| `assets/templates/*.html` | newsletter, event-invite, followup, base-email, campaign-report |
| `scripts/create_campaign.sh` | Crear campaña UTM (`--name`) |
| `scripts/create_mailing.sh` | Crear mailing (`--subject --body-html-file`) |
| `scripts/create_list.sh` | Crear lista de contactos (`--create`) |
| `scripts/create_server_action.sh` | Automatizaciones (`--name --model --code`) |
| `scripts/create_followup_activities.sh` | Ghost follow-up (`--partner-ids --activity-type`) |
| `scripts/validate_template.sh` | Validar plantilla HTML / QWeb |

`common:references/shared/` — conexión Odoo y batch ops vía odoo-pilot.
