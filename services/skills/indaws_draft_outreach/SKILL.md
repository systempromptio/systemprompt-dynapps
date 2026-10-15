---
name: indaws-draft-outreach
description: >
  Genera emails y mensajes LinkedIn de prospectcion con investigacion previa.
  Siempre investiga al prospecto antes de redactar (Clay + Odoo). Usa framework
  AIDA (Atencion, Interes, Deseo, Accion). Soporta: cold, warm, re-engagement,
  post-evento, post-demo. Genera email + variante LinkedIn + secuencia follow-up.
  Usar cuando el usuario pida: escribir email, draft outreach, contactar lead,
  mensaje LinkedIn, secuencia de emails, email de prospectcion, cold email.
version: 1.0.2
model: sonnet
effort: low
---

# Draft Outreach

Redaccion de outreach comercial con investigacion previa obligatoria. Framework AIDA adaptado al mercado espanol de implantacion Odoo.

## Prerequisitos

- **odoo-pilot** (common): Historial CRM (crm.lead, res.partner)
- **Credenciales Odoo**: `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, `ODOO_USER`
- See references/shared/odoo-connection.md for connection pattern.
- **Clay MCP** (recomendado): Investigacion del prospecto

## Workflow

> Ver `references/workflow_detail.md` para pasos detallados, plantillas por tipo y formato.

1. **Recopilar info** - Preguntar: destinatario, tipo (cold/warm/re-engagement/post-evento/post-demo), contexto, objetivo. **DETENTE y espera respuesta.**
2. **Investigar** (OBLIGATORIO) - Buscar en Odoo (crm.lead + res.partner via odoo-pilot) y Clay (si disponible). Sintetizar en ficha de prospecto.
3. **Redactar con AIDA** - Atencion (asunto max 6 palabras) + Interes (dato especifico) + Deseo (beneficio concreto) + Accion (peticion facil, 15 min).
4. **Generar outputs** - Email principal + variante LinkedIn (3 frases) + secuencia follow-up (Touch 2 dia+3, Touch 3 dia+7, Break-up dia+14).

## Rules

1. **NUNCA redactar sin investigar.** La investigacion es el paso 1, no opcional.
2. **NUNCA plantillas genericas.** Minimo 1 dato personalizado por email.
3. **NUNCA mentir o exagerar** datos del prospecto.
4. Si no hay datos suficientes, decirlo y sugerir investigacion adicional.
5. Cada touch usa angulo diferente. Nunca "Solo queria hacer seguimiento".
6. Texto plano, sin emojis. Cold: se lee entero en el móvil sin hacer scroll; warm, lo justo para el contexto compartido.
7. Asuntos: sin mayusculas excepto primera palabra y nombres propios.
8. Un solo CTA por email. Sin presion.

## Gotchas

1. **Clay no siempre disponible** - Si no hay Clay MCP, investigar solo con Odoo + preguntar al usuario por datos adicionales.
2. **Historial Odoo vacio** - No asumir que es cold; preguntar al usuario si ha habido contacto previo fuera de CRM.
3. **Re-engagement vs cold** - Si la ultima interaccion fue hace >6 meses, tratar como semi-cold (no asumir que recuerdan).
4. **Firma del email** - No inventar cargo ni datos de contacto; usar los del usuario o preguntar.
5. **LinkedIn tiene limite 300 chars** en solicitud de conexion (sin InMail). Adaptar longitud.
6. **Mercado espanol** - Tuteo profesional ("tu") salvo que el usuario pida "usted". Evitar anglicismos innecesarios.
7. **Follow-up timing** - Los dias son laborables, no naturales. Ajustar si hay festivos.

## Delegates to

| Skill | Suite | When | Agent()? |
|-------|-------|------|----------|
| odoo-pilot | common | CRM lead/partner data | No |

## Consumed by

| Skill | Suite | Purpose |
|-------|-------|---------|
| crm-lead-triage | sales | Follow-up email generation |
| daily-briefing | sales | Outreach task scheduling |

## References

| File | Contents |
|------|----------|
| `references/outreach_templates.md` | Framework AIDA tabla, estructuras por tipo (cold/warm/re-engagement), secuencia follow-up, frases prohibidas y recomendadas |
| `references/workflow_detail.md` | Pasos 0-3 detallados, ficha prospecto, output format, plantillas por tipo, formato email, success criteria |
