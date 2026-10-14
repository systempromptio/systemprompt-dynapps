# Modelos Odoo para Email Marketing — Referencia Técnica

## Versión: Odoo 19 (JSON2 Protocol)

---

## Modelos Principales

### utm.campaign
**Propósito:** Campaña UTM que agrupa mailings y actividades de marketing.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | Char | Nombre de la campaña (obligatorio) |
| `title` | Char | Título/etiqueta |
| `tag_ids` | Many2many → `utm.tag` | Etiquetas |
| `stage_id` | Many2one → `utm.stage` | Etapa de la campaña |
| `user_id` | Many2one → `res.users` | Responsable |

**Crear:**
```bash
bash create_record.sh "utm.campaign" '{"name": "Nurturing Q1 2025"}'
```

### mailing.mailing
**Propósito:** Un email individual (mailing) dentro de una campaña.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `subject` | Char | Línea de asunto (obligatorio) |
| `body_html` | Html | Contenido HTML del email |
| `preview_text` | Char | Preheader text |
| `email_from` | Char | Remitente ("Nombre <email>") |
| `reply_to` | Char | Email de respuesta |
| `mailing_model_id` | Char | Modelo target (ej: "mailing.list") |
| `contact_list_ids` | Many2many → `mailing.list` | Listas de contactos |
| `campaign_id` | Many2one → `utm.campaign` | Campaña asociada |
| `schedule_date` | Datetime | Fecha programada de envío |
| `state` | Selection | Estado: draft/in_queue/sending/done |
| `mailing_type` | Selection | Tipo: mail/sms |
| `body_arch` | Html | Arquitectura del body (para editor drag&drop) |

**Estados:**
- `draft` — Borrador (estado inicial, seguro)
- `in_queue` — En cola para envío
- `sending` — Enviando
- `done` — Enviado

**⚠️ SIEMPRE crear en estado `draft`. Nunca cambiar estado programáticamente.**

**Crear:**
```bash
bash create_record.sh "mailing.mailing" '{
  "subject": "Bienvenido a Indaws",
  "body_html": "<html>...</html>",
  "mailing_model_id": "mailing.list",
  "contact_list_ids": [[4, 1]],
  "campaign_id": 5,
  "state": "draft",
  "email_from": "Indaws <info@indaws.es>",
  "reply_to": "info@indaws.es"
}'
```

### mailing.list
**Propósito:** Lista de contactos para mailings.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | Char | Nombre de la lista (obligatorio) |
| `is_public` | Boolean | ¿Visible públicamente? |
| `contact_count` | Integer | Número de contactos (computed) |
| `contact_ids` | One2many → `mailing.contact` | Contactos de la lista |

**Crear:**
```bash
bash create_record.sh "mailing.list" '{"name": "Leads Q1 2025", "is_public": false}'
```

**Buscar:**
```bash
bash search_records.sh "mailing.list" '[["name","ilike","Leads"]]' '["id","name","contact_count"]'
```

### mailing.contact
**Propósito:** Contacto dentro de una lista de mailing.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | Char | Nombre del contacto |
| `email` | Char | Email (obligatorio) |
| `company_name` | Char | Nombre de la empresa |
| `title_id` | Many2one → `res.partner.title` | Título |
| `list_ids` | Many2many → `mailing.list` | Listas a las que pertenece |
| `tag_ids` | Many2many → `res.partner.category` | Etiquetas |
| `opt_out` | Boolean | ¿Se ha dado de baja? |

### mail.activity
**Propósito:** Actividad programada (para ghost follow-up).

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `activity_type_id` | Many2one → `mail.activity.type` | Tipo de actividad |
| `summary` | Char | Resumen breve |
| `note` | Html | Nota/descripción detallada |
| `date_deadline` | Date | Fecha límite |
| `user_id` | Many2one → `res.users` | Usuario asignado |
| `res_model_id` | Many2one → `ir.model` | Modelo del registro vinculado |
| `res_id` | Integer | ID del registro vinculado |

**Tipos de actividad comunes:**
```
mail.mail_activity_data_todo     — To-Do genérico
mail.mail_activity_data_call     — Llamada telefónica
mail.mail_activity_data_meeting  — Reunión
mail.mail_activity_data_email    — Email
mail.mail_activity_data_upload   — Subir documento
```

**Crear actividad vinculada a un contacto:**
```bash
# Primero obtener el model ID de res.partner
MODEL_ID=$(bash search_records.sh "ir.model" '[["model","=","res.partner"]]' '["id"]' 1 | grep -o '"id":[0-9]*' | grep -o '[0-9]*')

bash create_record.sh "mail.activity" "{
  \"activity_type_id\": 4,
  \"summary\": \"Conectar en LinkedIn\",
  \"note\": \"Ghost follow-up de campaña nurturing\",
  \"date_deadline\": \"2025-02-20\",
  \"user_id\": 2,
  \"res_model_id\": ${MODEL_ID},
  \"res_id\": 45
}"
```

### ir.actions.server
**Propósito:** Server actions para automatizaciones.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | Char | Nombre de la acción |
| `model_id` | Many2one → `ir.model` | Modelo sobre el que actúa |
| `state` | Selection | Tipo: code/object_create/object_write/multi |
| `code` | Text | Código Python a ejecutar (si state=code) |

---

## Relaciones entre Modelos

```
utm.campaign
  └── mailing.mailing (campaign_id)
        └── mailing.list (contact_list_ids - M2M)
              └── mailing.contact (list_ids - M2M)

res.partner
  └── mail.activity (res_id + res_model_id)
```

## URLs de Odoo para Revisión

Las URLs siguen el patrón de Odoo 19:

```
Campaña:  ${ODOO_URL}/odoo/utm-campaign/${ID}
Mailing:  ${ODOO_URL}/odoo/mass-mailing/${ID}
Lista:    ${ODOO_URL}/odoo/mailing-list/${ID}
Contacto: ${ODOO_URL}/odoo/contacts/${ID}
```

**Nota:** Las URLs exactas pueden variar según la configuración del menú en la instancia. Si las URLs anteriores no funcionan, usar el formato genérico:
```
${ODOO_URL}/odoo/${MODEL}/${ID}
```

## Campos M2M — Sintaxis de Escritura

Para campos Many2many en Odoo, usar la sintaxis de comandos:
- `[[4, ID]]` — Añadir relación existente (link)
- `[[6, 0, [ID1, ID2]]]` — Reemplazar todas las relaciones
- `[[3, ID]]` — Eliminar relación (unlink)

Ejemplo para `contact_list_ids`:
```json
{"contact_list_ids": [[4, 5]]}       // Añadir lista ID 5
{"contact_list_ids": [[6, 0, [5, 8]]]} // Establecer listas 5 y 8
```
