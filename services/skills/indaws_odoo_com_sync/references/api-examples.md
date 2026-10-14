# API Examples — Indaws/Dynapps CRM vía conector MCP de Odoo

El CRM de Indaws/Dynapps se lee/escribe a través del **conector MCP de Odoo** (tools `whoami`,
`orm_search_read`, `orm_write`, `orm_create`). **NO** usar curl, `odoo-pilot`, `search_records.sh` ni
JSON-RPC/JSON2: el backend ya no está accesible en `https://www.indaws.es` (302 → dynapps.es) ni en
`https://www.portal.dynapps.es` (web pública Next.js). Verificar la sesión con `whoami` antes de empezar.

Datos informativos de la instancia: DB `master`, usuario `victor.peris@dynapps.es`. Cada bloque de abajo es el
payload del tool `orm_search_read`.

---

## Buscar oportunidades de la última semana

> **Scope por ID de etapa, no por nombre ni probabilidad.** El domain usa `stage_id in [26,3,27,29]` (pipeline cualificado). Ver `workflow-detail.md` → "Configuración de etapas y scope". El antiguo `probability < 100` queda obsoleto.

`<lunes-semana-actual>` = lunes de la semana en curso; `<hoy>` = fecha actual (`YYYY-MM-DD`).

```jsonc
// orm_search_read
{
  "model": "crm.lead",
  "domain": [
    ["create_date", ">=", "<lunes-semana-actual> 00:00:00"],
    ["create_date", "<=", "<hoy> 23:59:59"],
    ["type", "=", "opportunity"],
    ["active", "=", true],
    ["stage_id", "in", [26, 3, 27, 29]],
    ["email_from", "!=", false]
  ],
  "fields": [
    "id", "name", "partner_name", "email_from", "phone",
    "stage_id", "date_deadline", "probability", "create_date",
    "user_id", "description", "expected_revenue", "contact_name", "activity_ids",
    "x_studio_ai_toma_de_datos", "x_studio_fecha_demo", "x_studio_fecha_toma_datos",
    "x_studio_toma_datos_transcript", "x_studio_ai_guion_demo", "x_studio_demo_users_ids",
    "x_studio_commercial_email", "x_studio_commercial_name", "x_studio_commercial_phone"
  ],
  "limit": 100,
  "context": {"lang": "es_ES"}
}
```

---

## Buscar oportunidades de los últimos 30 días (Fase 2)

`<hoy-30d>` = fecha de hace 30 días (`YYYY-MM-DD`). En Fase 2 solo se refrescan `expected_revenue` y
`probability` en el portal (ver `workflow-detail.md`), así que bastan los campos mínimos.

```jsonc
// orm_search_read
{
  "model": "crm.lead",
  "domain": [
    ["create_date", ">=", "<hoy-30d> 00:00:00"],
    ["type", "=", "opportunity"],
    ["active", "=", true],
    ["stage_id", "in", [26, 3, 27, 29]],
    ["email_from", "!=", false]
  ],
  "fields": [
    "id", "name", "partner_name", "email_from",
    "stage_id", "expected_revenue", "probability"
  ],
  "limit": 200,
  "context": {"lang": "es_ES"}
}
```

---

## Actividades pendientes de una oportunidad (mail.activity)

La "próxima acción" del Paso 1.1b. Usa SOLO los campos listados — **nunca** traigas `note` (blobs HTML enormes, ver gotcha del SKILL).

```jsonc
// orm_search_read
{
  "model": "mail.activity",
  "domain": [["res_model", "=", "crm.lead"], ["res_id", "=", <LEAD_ID>]],
  "fields": ["activity_type_id", "summary", "date_deadline", "user_id", "state"],
  "order": "date_deadline asc",
  "limit": 10,
  "context": {"lang": "es_ES"}
}
```

`activity_type_id` devuelve `[id, name]`; resuelve el tipo por **id** contra el mapa de tipos de actividad (ver `workflow-detail.md` → Paso 1.1b). `state` puede ser `overdue`, `today`, `planned`.

Tipos de actividad por ID (Indaws): `59` Toma de datos · `38` Demo · `61` Presentación de análisis ·
`2` Llamada · `3` Reunión · `4` Por hacer · `1` Correo electrónico.

---

## Extraer mensajes del chatter de una oportunidad

Incluye actividades cerradas (mensajes con subtipo de actividad) para el histórico que va en la descripción.

```jsonc
// orm_search_read
{
  "model": "mail.message",
  "domain": [
    ["res_id", "=", <LEAD_ID>],
    ["model", "=", "crm.lead"],
    ["message_type", "in", ["comment", "email"]]
  ],
  "fields": ["date", "body", "author_id", "message_type", "subtype_id"],
  "order": "date desc",
  "limit": 10,
  "context": {"lang": "es_ES"}
}
```

---

## Obtener etapas disponibles del CRM (refrescar el mapa canónico)

Ejecuta esto si una query devuelve un `stage_id` cuyo id no está en el mapa canónico (ver `workflow-detail.md` → "Mapa canónico de etapas"). El `id` es inmutable; el `name`/`sequence` cambian.

```jsonc
// orm_search_read
{
  "model": "crm.stage",
  "domain": [],
  "fields": ["id", "name", "sequence", "is_won"],
  "order": "sequence asc",
  "limit": 50,
  "context": {"lang": "es_ES"}
}
```

Tras refrescar, actualiza la tabla del mapa en `workflow-detail.md` y reajusta el scope `stage_id in [...]` si las etapas cualificadas cambiaron de id.

**Mapa canónico vigente (filtra/razona SIEMPRE por `id`, no por nombre — ver GOTCHA 1):**

| id | Etapa | scope sync |
|----|-------|-----------|
| 1  | Lead | ✗ (sin cualificar) |
| 26 | Cualificado | ✓ |
| 3  | Propuesta | ✓ |
| 27 | Negociación | ✓ |
| 29 | En análisis | ✓ |
| 28 | En Espera | ✗ (parada) |
| 4  | Ganado | ✗ (ganada) |

Scope de sincronización: `stage_id in [26,3,27,29]`. Si esta query devuelve ids fuera de la tabla,
el CRM cambió → actualizar el mapa en `SKILL.md`/`workflow-detail.md` y aquí. El mapa antiguo de Indaws
`[22,19,12,24,13,8,2,15,9,3,7,23]` está OBSOLETO desde la migración a Dynapps; no usarlo.

> Nota: en Odoo 19 `crm.stage` ya no expone `probability`; no lo pidas en `fields`.

---

## Campos personalizados disponibles en crm.lead (Indaws)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `expected_revenue` | float | Ingresos esperados (v19, sustituye a `planned_revenue`) |
| `date_deadline` | date | **Cierre estimado** (no tiene campo nativo en el portal — va en la descripción) |
| `activity_ids` | one2many | IDs de actividades pendientes (`mail.activity`) — usar para la próxima acción |
| `x_studio_ai_toma_de_datos` | html | Toma de datos rellenada con AI (resumir; no volcar literal) |
| `x_studio_fecha_toma_datos` | date | Fecha de la toma de datos |
| `x_studio_fecha_demo` | date | Fecha de la demo |
| `x_studio_toma_datos_transcript` | html | Transcript de la toma de datos (resumir) |
| `x_studio_ai_guion_demo` | html | Guión de la demo generado con AI (resumir) |
| `x_studio_demo_users_ids` | m2m | Usuarios que realizaron la demo |
| `x_studio_commercial_email` | char | Email del comercial responsable |
| `x_studio_commercial_name` | char | Nombre del comercial responsable |
| `x_studio_commercial_phone` | char | Teléfono del comercial responsable |

**IMPORTANTE:** `planned_revenue` no existe en Odoo 19. Causa `InvalidFieldError` si se incluye en fields.

---

## Respuesta esperada de orm_search_read

```json
[
  {
    "id": 123,
    "name": "Implantación Odoo - Empresa XYZ",
    "partner_name": "Empresa XYZ S.L.",
    "email_from": "contacto@empresa.com",
    "phone": "+34 612 345 678",
    "stage_id": [3, "Propuesta"],
    "date_deadline": "2026-06-30",
    "probability": 40.0,
    "create_date": "2026-05-19 09:15:00",
    "user_id": [12, "Comercial Nombre"],
    "description": "...",
    "expected_revenue": 15000.0,
    "contact_name": "Juan García",
    "activity_ids": [12381],
    "x_studio_ai_toma_de_datos": "...",
    "x_studio_fecha_toma_datos": "2026-05-20",
    "x_studio_fecha_demo": "2026-05-25",
    "x_studio_commercial_email": "comercial@indaws.es",
    "x_studio_commercial_name": "Comercial Nombre",
    "x_studio_commercial_phone": "+34 600 000 000"
  }
]
```

Los campos Many2one (`stage_id`, `user_id`, `activity_type_id`) devuelven `[id, name]` o `false` si no tienen valor. **Resuelve etapas y tipos de actividad por su `id` (el primer elemento), nunca por el nombre.**
