# Email Campaign Builder — Workflow detallado

Flujo completo, plantilla HTML del editor Odoo, ejemplos reales y checklist.
Todo se crea SIEMPRE en estado `draft`. Nada se envía sin confirmación humana.

---

## Flujo visual

```
1. BRIEFING        → Objetivo · Audiencia · Producto · CTA
                     (Nurturing/Evento/Newsletter · Leads/Clientes/Segmento ·
                      ERP/BI/Desarrollo · Agendar/Demo/Descargar)
        │
        ▼
2. FLUJO AUTOMÁTICO → Email 1 (Intro) → Email 2 (Valor) → Email 3 (Caso) → Email 4 (CTA)
                     Triggers Marketing Automation:
                     begin · mail_open/mail_not_open · mail_click/mail_not_click ·
                     mail_reply/mail_bounce
        │
        ▼
3. GHOST FOLLOW-UP  → LinkedIn Connect · Llamada comercial · WhatsApp · Nota interna
   (complementario)   Detalle: references/ghost-followup.md
        │
        ▼
4. VERIFICACIÓN     → DRAFT → review en Odoo → verificar body_arch + body_html → OK
   + ENTREGA          Entregar: URLs directas a cada mailing, checklist comercial, report
```

---

## Flujos por tipo de campaña

| Objetivo | Flujo | Emails | Spacing | Marketing Automation |
|----------|-------|--------|---------|---------------------|
| Nurturing | nurturing-sequence | 5-7 | 3-5 días | Sí |
| Evento | event-sequence | 3+1 | Variable | Sí |
| Reactivación | reactivation-sequence | 3-4 | 4-7 días | Sí |
| Producto | product-launch | 4-5 | 3-4 días | Sí |
| Newsletter | newsletter-single | 1 | Periódico | No |
| Lead gen | lead-gen-sequence | 3-4 | 3-5 días | Sí |

Plantillas de flujo completas: `references/flow-templates.md`.

---

## Estructura HTML obligatoria para el editor Odoo

El body debe seguir esta estructura para que el editor visual de Odoo lo renderice.
Escribir SIEMPRE en ambos campos `body_arch` y `body_html` con el mismo valor.

```html
<div class="o_layout o_newsletter_theme" style="padding:0 3%;background-color:#f5f5f5;">
  <table class="o_mail_wrapper" style="width:100%;border-collapse:separate;">
    <tbody><tr>
      <td class="o_mail_no_resize o_not_editable"> </td>
      <td class="o_mail_no_options o_mail_wrapper_td oe_structure" style="background:#fff;width:600px;">

        <div class="o_mail_block_header_logo">
          <div class="o_mail_snippet_general"><!-- Logo --></div>
        </div>

        <div class="o_mail_block_title_text">
          <div class="o_mail_snippet_general"><!-- Contenido --></div>
        </div>

        <div class="o_mail_block_footer_social">
          <div class="o_mail_snippet_general"><!-- Footer --></div>
        </div>

      </td>
      <td class="o_mail_no_resize o_not_editable"> </td>
    </tr></tbody>
  </table>
</div>
```

```bash
VALUES="{\"body_arch\": $BODY_ESCAPED, \"body_html\": $BODY_ESCAPED}"
```

### Videos YouTube en emails

Los emails no reproducen vídeo. Usar thumbnail + link (playlist: `references/youtube-casos-exito.md`):

```html
<a href="https://www.youtube.com/watch?v=VIDEO_ID" target="_blank">
  <img src="https://img.youtube.com/vi/VIDEO_ID/hqdefault.jpg"
       alt="Ver caso de éxito"
       style="max-width:100%;border-radius:8px;border:2px solid #6B68FA;">
</a>
```

---

## Colores Indaws (referencia rápida)

Source of truth: `dynapps-brand/references/brand-system.md`. Gradients siempre dark → light.

```css
--indaws-blue-lilac: #6B68FA;   /* CTAs, headings */
--indaws-blue-space: #1C265D;   /* Header, texto */
--indaws-warm-yellow: #E5B92B;  /* Highlights, accents */
--indaws-light-sky: #8AC2DB;    /* Links secundarios */
```

## Vocabulario Indaws

| Usar | No usar |
|------|---------|
| tu (tutear) | usted |
| inversión | precio, coste, gasto |
| especialistas | técnicos |
| propuesta | presupuesto |
| reto | problema |
| acompañamiento | soporte |
| evolución continua | mantenimiento |

---

## Ejemplos de campañas reales

### Campaña: Migración V16 a V20

Contexto: 16 empresas con Odoo V16 necesitan migrar. Objetivo: nurturing educativo para generar demanda.

| Elemento | Valor |
|----------|-------|
| Flujo | 4 emails + ghost follow-up |
| Spacing | 2 meses entre emails |
| CTA | Agendar llamada con comercial |

```
Email 1 (Feb) — Caso Todotrofeo · Pain: "tu versión ya no recibe actualizaciones" · CTA: ver caso
Email 2 (Abr) — Caso Prima Protección · Pain: "las nuevas features no llegan a V16" · CTA: ver caso
Email 3 (Jun) — Caso Espacio Orgánico · Pain: "el coste de mantener V16 aumenta" · CTA: agendar llamada
Email Recordatorio — Oferta limitada · Urgencia: slots limitados · CTA: reservar slot
```

IDs en Odoo: Marketing Automation 97 · Lista 1615 · Comercial Anxo Nieto (1159)

### Campaña: Webinar BI Manufacturing

Contexto: webinar sobre BI para fabricación. Objetivo: generar registros al evento.

| Elemento | Valor |
|----------|-------|
| Flujo | 3 emails + reminder día anterior |
| Audiencia | Leads manufacturing |
| CTA | Reservar plaza |

```
Email 1 (D-14) — Save the date · tema, fecha/hora · CTA: reservar plaza
Email 2 (D-7)  — Speakers y agenda · ponentes con bio · CTA: reservar plaza
Email 3 (D-1)  — Recordatorio · "mañana a las X", acceso directo · CTA: añadir a calendario
Post-evento    — Grabación + next steps · link grabación, slides · CTA: agendar demo
```

---

## Checklist pre-activación (entregar al comercial)

```
[ ] body_arch tiene contenido (verificar en editor Odoo)
[ ] body_html tiene contenido (preview del email)
[ ] Subject line < 50 caracteres
[ ] Preheader definido
[ ] CTA con URL válida
[ ] Lista de contactos asignada (revisar opt-out / GDPR)
[ ] Comercial asignado
[ ] Ghost follow-up configurado (si aplica)
[ ] Vocabulario revisado contra brand-voice
[ ] Sin emojis en contenido oficial
[ ] Estado = draft (NO enviar sin confirmación explícita del usuario)
```
