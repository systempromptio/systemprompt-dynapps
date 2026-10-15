# Odoo Event Setup — API Commands

Use odoo-pilot scripts to create and manage events and projects in Odoo 19.

## Create Event

```bash
# Authenticate first
eval $(./scripts/auth.sh)

# Create event
./scripts/create_record.sh event.event '{
  "name": "IA Aplicada a tu Sistema de Gestion — Desayuno Indaws",
  "date_begin": "2026-02-20 08:00:00",
  "date_end": "2026-02-20 11:00:00",
  "date_tz": "Europe/Madrid",
  "location_id": VENUE_PARTNER_ID,
  "seats_max": 50,
  "seats_limited": true,
  "auto_confirm": true,
  "website_published": true
}'
```

### Event Fields

| Field | Type | Description |
|-------|------|-------------|
| name | Char | Event title |
| date_begin | Datetime | Start (UTC) |
| date_end | Datetime | End (UTC) |
| date_tz | Char | Timezone |
| location_id | Many2one (res.partner) | Venue partner record |
| seats_max | Integer | Maximum capacity |
| seats_limited | Boolean | Enable capacity limit |
| auto_confirm | Boolean | Auto-confirm registrations |
| website_published | Boolean | Show on website |
| description | Html | Event description for website |
| note | Html | Internal notes |
| tag_ids | Many2many | Event tags |
| organizer_id | Many2one (res.partner) | Organizer |

## Create Event Tickets

```bash
./scripts/create_record.sh event.event.ticket '{
  "name": "Entrada General",
  "event_id": EVENT_ID,
  "seats_max": 50,
  "price": 0
}'
```

## Create Registration (manual)

```bash
./scripts/create_record.sh event.registration '{
  "event_id": EVENT_ID,
  "partner_id": PARTNER_ID,
  "name": "Nombre Asistente",
  "email": "email@example.com",
  "phone": "+34600000000",
  "state": "open"
}'
```

### Registration States
- `draft` — Unconfirmed
- `open` — Confirmed
- `done` — Attended
- `cancel` — Cancelled

### Mark Attendance
```bash
./scripts/update_record.sh event.registration '[REG_ID]' '{"state": "done"}'
```

## List Registrations
```bash
./scripts/search_records.sh event.registration '[["event_id","=",EVENT_ID]]' '["name","email","phone","state"]' 100
```

---

## Create Project

```bash
./scripts/create_record.sh project.project '{
  "name": "Evento: IA Aplicada — 20 Feb 2026",
  "description": "Proyecto de planificacion para el desayuno Indaws del 20 de febrero",
  "allow_task_dependencies": true
}'
```

### Project Fields

| Field | Type | Description |
|-------|------|-------------|
| name | Char | Project name |
| description | Html | Project description |
| user_id | Many2one (res.users) | Project manager |
| partner_id | Many2one (res.partner) | Customer (if applicable) |
| date_start | Date | Start date |
| date | Date | End date |
| allow_task_dependencies | Boolean | Enable task dependencies |

## Create Tasks

```bash
./scripts/create_record.sh project.task '{
  "name": "Confirmar fecha y venue",
  "project_id": PROJECT_ID,
  "user_ids": [[6, 0, [USER_ID]]],
  "date_deadline": "2026-01-15",
  "description": "Visitar venue si es nuevo. Confirmar capacidad, AV, catering, parking.",
  "stage_id": STAGE_ID,
  "priority": "1"
}'
```

### Task Fields

| Field | Type | Description |
|-------|------|-------------|
| name | Char | Task title |
| project_id | Many2one | Parent project |
| user_ids | Many2many (res.users) | Assignees |
| date_deadline | Date | Due date |
| description | Html | Task details |
| stage_id | Many2one (project.task.type) | Stage/column |
| priority | Selection | "0" normal, "1" high |
| tag_ids | Many2many | Tags |
| parent_id | Many2one | Parent task (subtask) |

### Get Project Stages
```bash
./scripts/search_records.sh project.task.type '[["project_ids","in",[PROJECT_ID]]]' '["name","sequence"]'
```

### Create Custom Stages (if needed)
```bash
for stage in "Planificacion" "Promocion" "Preparacion" "Ejecucion" "Seguimiento"; do
  ./scripts/create_record.sh project.task.type "{\"name\":\"$stage\",\"project_ids\":[[4,PROJECT_ID]]}"
done
```

---

## Create CRM Lead (Post-Event)

```bash
./scripts/create_record.sh crm.lead '{
  "name": "Lead evento: [Nombre Asistente] — Desayuno IA Feb 2026",
  "partner_id": PARTNER_ID,
  "email_from": "email@example.com",
  "phone": "+34600000000",
  "source_id": SOURCE_ID,
  "tag_ids": [[6, 0, [TAG_ID]]],
  "description": "Asistio al desayuno IA aplicada del 20 feb. Interesado en [tema]."
}'
```

### Bulk Lead Creation from Event
```bash
# Get all attendees (state=done)
ATTENDEES=$(./scripts/search_records.sh event.registration '[["event_id","=",EVENT_ID],["state","=","done"]]' '["name","email","phone","partner_id"]' 100)

# For each attendee, check if already client
# If not client → create CRM lead
```

---

## Useful Queries

### Event Dashboard
```bash
# Registration count by state
./scripts/search_records.sh event.registration '[["event_id","=",EVENT_ID]]' '["state"]' 200

# Count confirmed
./scripts/execute_method.sh event.registration search_count '[[["event_id","=",EVENT_ID],["state","=","open"]]]'
```

### Project Progress
```bash
# Tasks by stage
./scripts/search_records.sh project.task '[["project_id","=",PROJECT_ID]]' '["name","stage_id","date_deadline","user_ids"]' 100
```
