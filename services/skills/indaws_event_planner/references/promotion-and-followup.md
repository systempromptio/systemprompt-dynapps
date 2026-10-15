# Promocion y Follow-up — Detalle

Tablas operativas de promocion (LinkedIn + email) y conversion de leads post-evento.
Resumen de flujo en SKILL.md; aqui el detalle.

---

## Ficha del Evento (briefing inicial)

```
┌────────────────────────────────────────────────────────────┐
│                    FICHA DEL EVENTO                        │
├───────────────┬────────────────────────────────────────────┤
│ Nombre        │ [Titulo atractivo, brand-compliant]        │
│ Fecha/Hora    │ [Dia, hora inicio, hora fin]               │
│ Venue         │ [Nombre + direccion completa]              │
│ Formato       │ [Desayuno / Conferencia / Taller / Webinar]│
│ Capacidad     │ [20-50 tipico]                             │
│ Speaker(s)    │ [Nombre, cargo, bio breve]                 │
│ Target        │ [Clientes / Prospects / Mixto / Industria] │
│ Objetivo      │ [Leads / Brand awareness / Retencion]      │
│ KPIs          │ [Metricas concretas de exito]              │
│ Presupuesto   │ [Total asignado]                           │
│ Co-org        │ [Partners si aplica]                       │
└───────────────┴────────────────────────────────────────────┘
```

---

## Plan LinkedIn (4 posts + recap)

| Timing | Tipo | Funnel | Contenido |
|--------|------|--------|-----------|
| T-4 sem | Anuncio | TOFU | "Organizamos [evento]. Tema: [tema]" |
| T-3 sem | Speaker | MOFU | "Que vas a aprender: [puntos clave]" |
| T-2 sem | Urgencia | MOFU | "Quedan [X] plazas. Registrate" |
| T-1 sem | Final | BOFU | "Ultima semana para apuntarte" |
| T+2 dias | Recap | MOFU | Resumen + fotos + insight clave |

Delegar redaccion y publicacion a **linkedin-content-manager**.

---

## Campana Email (3 emails)

| Email | Timing | Destinatarios | Contenido |
|-------|--------|---------------|-----------|
| Invitacion | T-4 sem | Lista completa | Presentar evento, CTA registro |
| Recordatorio | T-1 sem | Registrados: logistica / No-reg: ultima oportunidad |
| Agradecimiento | T+2 dias | Asistentes: gracias + recursos |

Plantillas en [email-templates.md](email-templates.md). Creacion en Odoo via
**email-campaign-builder** — siempre en estado DRAFT.

---

## Roles Clave (dia del evento)

| Rol | Responsabilidad | Quien |
|-----|-----------------|-------|
| Speaker | Presentacion principal | Victor / invitado |
| Host | Bienvenida, moderacion, Q&A | Marketing |
| Logistica | Montaje, catering, AV | Marketing + venue |
| Registro | Check-in, badges | Marketing |
| Foto/Video | Documentar el evento | Marketing / externo |

---

## Flujo de Conversion de Leads

```
                    ┌────────────────────┐
                    │ Asistente registrado│
                    └─────────┬──────────┘
                              │
                    ┌─────────▼──────────┐
                    │    Asistio?        │
                    └─────────┬──────────┘
              ┌───────────────┴───────────────┐
              │ SI                             │ NO
              ▼                                ▼
    ┌─────────────────────┐        ┌─────────────────────┐
    │ Lead CRM            │        │ Email "te echamos   │
    │ Tag: "Evento [X]"   │        │ de menos" + recurso │
    │ Asignar comercial   │        │ Lead tag: "No-show" │
    └─────────┬───────────┘        └─────────────────────┘
              │
              ▼
    ┌─────────────────────┐
    │ Email follow-up 48h │
    │ Llamada 1 semana    │
    └─────────────────────┘
```

---

## KPIs a Medir

| KPI | Como medir |
|-----|-----------|
| Tasa asistencia | Asistentes / Registrados (Odoo Events) |
| Leads generados | Leads nuevos con tag evento (CRM) |
| Coste por lead | Presupuesto / Leads |
| Satisfaccion | Encuesta post-evento (avg score) |
| ROI | Revenue from leads / Presupuesto (90 dias) |

Reporte post-evento: usar [../templates/event-report.html](../templates/event-report.html)
(metricas, presupuesto estimado vs real, satisfaccion, lecciones, proximos pasos).
