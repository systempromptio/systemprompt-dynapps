# Modelos Odoo - Ticket Report

Referencia completa de modelos y campos utilizados por el skill ticket-report.

**NUNCA inventar nombres de modelos o campos. Si no está aquí, no existe.**

---

## helpdesk.ticket

Modelo principal de tickets de Helpdesk.

### Campos estándar

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | Integer | ID único del ticket |
| `name` | Char | Nombre/Asunto del ticket |
| `create_date` | Datetime | Fecha de creación del ticket |
| `close_date` | Datetime | Fecha de cierre del ticket (cuando pasa a Solved) |
| `write_date` | Datetime | Última modificación del ticket |
| `stage_id` | Many2one → helpdesk.stage | Etapa actual del ticket |
| `priority` | Selection | Prioridad: "0" (Normal), "1" (Urgente), "2" (Muy urgente), "3" (Crítico) |
| `user_id` | Many2one → res.users | Usuario asignado (campo estándar "Asignado a") |
| `team_id` | Many2one → helpdesk.team | Equipo de helpdesk |
| `partner_id` | Many2one → res.partner | Cliente/Contacto asociado |
| `description` | Html | Descripción del ticket |
| `sla_deadline` | Datetime | Fecha límite de SLA |

### Campos personalizados (x_studio_*)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `x_studio_assigned_developer_id` | Many2one → res.users | **Desarrollador asignado** (campo principal para buscar tickets de un dev) |
| `x_studio_ticket_type` | Selection | Tipo de ticket |
| `x_studio_checked_by_developer` | Boolean | Revisado por el desarrollador |
| `x_studio_production_support` | Char | Soporte en producción |
| `x_studio_cliente_proyecto` | Char | Cliente/Proyecto asociado |
| `x_studio_project_stage` | Selection | Etapa del proyecto |
| `x_studio_customer_priority` | Selection | Prioridad asignada por el cliente |
| `x_studio_horas_disponibles` | Float | Horas disponibles |

### Notas importantes

- **Campo principal de búsqueda por desarrollador**: `x_studio_assigned_developer_id`
- **Campo estándar de asignación**: `user_id` (usado por la asignación estándar de Odoo Helpdesk)
- **Diferencia**: Un ticket puede tener `user_id` = consultor y `x_studio_assigned_developer_id` = desarrollador

---

## helpdesk.stage

Etapas del flujo de tickets.

| ID | Nombre | Tipo | Descripción |
|----|--------|------|-------------|
| 1 | New | Abierto | Ticket recién creado, sin asignar |
| 2 | In Progress | Abierto | En proceso de gestión |
| 6 | En desarrollo | Abierto | Desarrollo activo en curso |
| 15 | Testeo Desarrollo | Abierto | En fase de testing/QA |
| 5 | Espera cliente | Abierto | Pendiente de respuesta del cliente |
| 14 | Espera subida producción | Abierto | Desarrollo listo, esperando deploy |
| 3 | Solved | Cerrado | Ticket resuelto |
| 4 | Cancelled | Cerrado | Ticket cancelado |

### Filtros útiles

```
# Tickets abiertos (no resueltos ni cancelados)
[["stage_id", "in", [1, 2, 5, 6, 14, 15]]]

# Tickets cerrados (resueltos)
[["stage_id", "=", 3]]

# Tickets en desarrollo activo
[["stage_id", "in", [2, 6]]]

# Tickets bloqueados/en espera
[["stage_id", "in", [5, 14, 15]]]
```

---

## mail.message

Mensajes del chatter de cualquier modelo Odoo.

### Campos relevantes

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | Integer | ID del mensaje |
| `model` | Char | Modelo relacionado (ej: "helpdesk.ticket") |
| `res_id` | Integer | ID del registro relacionado (ej: ID del ticket) |
| `body` | Html | Contenido del mensaje (HTML) |
| `date` | Datetime | Fecha del mensaje |
| `author_id` | Many2one → res.partner | Autor del mensaje |
| `subtype_id` | Many2one → mail.message.subtype | Subtipo (nota, comentario, seguimiento, etc.) |

### Filtros útiles

```
# Mensajes de un ticket específico (con contenido)
[["model", "=", "helpdesk.ticket"], ["res_id", "=", 12007], ["body", "!=", false], ["body", "!=", ""]]

# Mensajes de varios tickets
[["model", "=", "helpdesk.ticket"], ["res_id", "in", [12007, 11258]], ["body", "!=", false]]
```

### Notas sobre el chatter

- Los mensajes HTML deben limpiarse (quitar tags) para análisis de texto
- Los mensajes de tracking (cambios de campo) suelen tener `subtype_id` de tipo tracking
- Para análisis de tardanza, buscar palabras clave: "espera", "pendiente", "bloqueado", etc.

---

## res.users

Usuarios de Odoo.

### Campos relevantes

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | Integer | ID del usuario |
| `name` | Char | Nombre completo |
| `login` | Char | Email/Login del usuario |

### Búsqueda de usuario por email

```
[["login", "=", "luis.herrera@indaws.es"]]
```

---

## Cálculos de tiempo

### Tiempo de resolución

```
tiempo_resolución = close_date - create_date
```

- Solo válido para tickets con `close_date` (etapa Solved)
- Se mide en segundos y se convierte a horas/días para visualización

### Antigüedad de un ticket

```
antigüedad = fecha_actual - create_date
```

### Días sin actividad

```
días_sin_actividad = fecha_actual - write_date
```

---

## Umbrales de detección de atascos

| Etapa | Umbral (días) | Indicador |
|-------|--------------|-----------|
| New | > 7 | Sin asignar demasiado tiempo |
| Espera cliente | > 14 sin actividad | Cliente no responde |
| En desarrollo | > 21 | Desarrollo demasiado largo |
| Testeo Desarrollo | > 14 | Testing prolongado |
| Espera subida producción | > 7 | Deploy pendiente demasiado tiempo |

## Umbrales de carga de trabajo

| Indicador | Umbral Normal | Umbral Moderada | Umbral Alta |
|-----------|--------------|-----------------|-------------|
| Tickets abiertos totales | <= 10 | 11-15 | > 15 |
| Tickets antiguos (>30 días) | <= 3 | 4-5 | > 5 |
| Tickets atascados | <= 1 | 2-3 | > 3 |
