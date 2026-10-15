# Client Request Manager - Workflow Detail

## Mode: GENERAR

### Paso 1: Extraccion de Contexto

Ejecutar script de extraccion para obtener datos de la tarea:

```bash
export ODOO_URL="<url>"
export ODOO_DB="<db>"
export ODOO_KEY="<api_key>"
export ODOO_USER="<usuario>"

./scripts/fetch_task_context.sh <task_id> --pilot-dir /path/to/odoo-pilot/scripts
```

El script obtiene 4 conjuntos de datos:
- Tarea (`project.task`): nombre, descripcion HTML, horas asignadas, etapa, proyecto
- Checklists (`xx.checklist.item`): items vinculados a la tarea con `name`, `weight`, `state`
- Proyecto (`project.project`): nombre, partner, fechas
- Requests existentes (`x_client_request`): peticiones ya creadas para esa tarea

Ademas, lee el documento de analisis funcional proporcionado por el consultor.

**Validaciones post-extraccion**:
- La tarea existe y tiene project_id
- El proyecto tiene partner_id (necesario para enviar emails)
- Si la tarea no tiene descripcion ni checklists: avisar al consultor y pedir contexto manual

**Si la tarea se busca por nombre** en vez de ID:
1. Buscar con `search_records.sh project.task '[["name","ilike","<nombre>"],["project_id","=",<project_id>]]'`
2. Si hay multiples resultados, mostrar lista y pedir al usuario que confirme el ID

### Paso 2: Analisis Inteligente con Cotejo

Este es el paso mas importante. El LLM cruza 5 fuentes para determinar que peticiones son necesarias:

#### Fuente A: Descripcion de la tarea
Analiza la descripcion HTML de la tarea para identificar dependencias del cliente:
- Menciones a datos que necesitamos del cliente
- Referencias a ficheros, exports o migraciones
- Decisiones de negocio pendientes
- Accesos o credenciales necesarias

#### Fuente B: Checklists de la tarea
Cada item del checklist que requiere input del cliente genera una peticion candidata:
- Items que mencionan "cliente", "validar", "confirmar", "enviar", "datos"
- Items de tipo configuracion que dependen de informacion del negocio

#### Fuente C: Patrones por tipo de tarea
Consulta `./references/analisis_peticiones.md` para identificar peticiones tipicas segun el tipo de tarea (Ventas, Inventario, Contabilidad, etc.).

#### Fuente D: Cotejo con el analisis funcional (CRITICO)

OBLIGATORIO: Para cada peticion candidata, verifica en el documento de analisis:
- **Si la informacion ya esta en el analisis** -> DESCARTAR la peticion
  - Ejemplo: Analisis dice "3 tarifas: mayorista, minorista, VIP" -> No pedir listas de precios
- **Si el analisis menciona el tema pero falta el detalle/fichero** -> MANTENER la peticion
  - Ejemplo: Analisis dice "migrar 2.000 productos desde Sage" -> Si pedir el fichero de productos
- **Si el analisis no menciona el tema** -> MANTENER la peticion

#### Fuente E: Requests existentes
Verificar que no existan peticiones ya creadas para esta tarea con el mismo proposito. Si existe una peticion similar -> DESCARTAR.

#### Presentacion al consultor

Presenta tabla de peticiones propuestas con justificacion:

```
| # | Titulo                  | Tipo        | Prioridad | Deadline | Justificacion                              |
|---|-------------------------|-------------|-----------|----------|--------------------------------------------|
| 1 | Export productos CSV    | data_file   | blocker   | +14 dias | Analisis confirma 2000 prod pero no fichero|
| 2 | Reglas de descuento     | information | required  | +7 dias  | No mencionado en analisis                  |
| X | Listas de precios       | information | -         | -        | DESCARTADA: analisis S3.2 indica 3 tarifas |
```

Las peticiones descartadas se muestran con X para que el consultor entienda el razonamiento.

**DETENTE** -- VALIDACION OBLIGATORIA. Espera que el consultor apruebe, elimine, anada o modifique.

### Paso 3: Creacion de Registros en Odoo

Por cada peticion aprobada, crear registro en `x_client_request`:

```bash
$PILOT_DIR/create_record.sh "x_client_request" '{
  "x_name": "<titulo>",
  "x_task_id": <task_id>,
  "x_partner_id": <partner_id>,
  "x_description": "<html_descripcion>",
  "x_type": "<tipo>",
  "x_priority": "<0_o_1>",
  "x_deadline": "<YYYY-MM-DD>",
  "x_state": "draft"
}'
```

**Reglas para la descripcion (x_description)**:
- Redactar en HTML limpio
- Incluir: que se necesita, por que, y en que formato preferiblemente
- Tono cercano en segunda persona (tu/vosotros)

**Calculo de deadlines**:
- El deadline se calcula a partir de `planned_date_begin` de `project.task`, NO de la fecha actual
- El deadline es el dia anterior al inicio de la tarea (o el ultimo dia laborable anterior si cae en fin de semana)
- Si la tarea NO tiene `planned_date_begin`, usar +14 dias y avisar al consultor
- Si el consultor ha indicado deadlines especificos, respetarlos

---

## Mode: FOLLOW-UP

### Paso 1: Dashboard del Proyecto

Ejecutar script de extraccion de requests:

```bash
./scripts/fetch_requests.sh <project_id> --pilot-dir /path/to/odoo-pilot/scripts
```

Presentar dashboard agrupado:

```
Proyecto: COPC -- Client Requests
---------------------------------------
Borrador (pendientes de enviar):    3
Enviadas (esperando respuesta):     5
  - Con respuesta detectada:        2
Vencidas (sin respuesta):           2
Recibidas:                          8
Canceladas:                         1
```

DETENTE -- el consultor ve el estado global y decide que gestionar.

### Paso 2: Gestion de BORRADORES (envio inicial)

Si hay requests en estado `draft`:
- Presentar lista ordenada por prioridad y deadline
- Sugerir cuales enviar ahora (bloqueantes primero, luego por deadline cercano)

Por cada peticion que el consultor apruebe enviar:

1. Generar email profesional usando template `./templates/email_peticion.md`
2. Presentar borrador al consultor
3. DETENTE -- VALIDACION del email
4. Enviar email al cliente via proceso de 3 pasos (bug JSON2 workaround):

```bash
# Paso A: Crear mail.message directamente (HTML limpio en chatter)
POST /json/2/mail.message/create
{
  "vals_list": [{
    "model": "x_client_request",
    "res_id": <request_id>,
    "body": "<html_email>",
    "message_type": "comment",
    "subtype_id": 1,
    "author_id": <author_partner_id>,
    "partner_ids": [[4, <partner_id>]],
    "subject": "[<proyecto>] <titulo_peticion>"
  }]
}

# Paso B: Crear mail.mail y enviar
POST /json/2/mail.mail/create
{
  "vals_list": [{
    "mail_message_id": <message_id>,
    "body_html": "<html_email>",
    "email_from": "<nombre_autor> <<email_autor>>",
    "email_to": "<email_cliente>",
    "subject": "[<proyecto>] <titulo_peticion>",
    "state": "outgoing",
    "auto_delete": true
  }]
}

POST /json/2/mail.mail/send
{ "ids": [<mail_id>] }

# Paso C: Crear notificacion (icono de enviado en chatter)
POST /json/2/mail.notification/create
{
  "vals_list": [{
    "mail_message_id": <message_id>,
    "res_partner_id": <partner_id>,
    "notification_type": "email",
    "notification_status": "sent"
  }]
}
```

5. Cambiar estado a sent:
```bash
$PILOT_DIR/update_record.sh "x_client_request" <request_id> '{"x_state": "sent"}'
```

### Paso 3: Gestion de VENCIDAS (recordatorios)

Si hay requests con `x_is_overdue = True` (o `x_state=sent` y `x_deadline < hoy`):

| Accion | Que hace |
|--------|----------|
| **Recordatorio** | Genera email follow-up con template `./templates/email_followup.md`, valida, envia |
| **Extender deadline** | Actualiza `x_deadline` con nueva fecha |
| **Cancelar** | Cambia a `cancelled` |
| **Marcar recibida** | Si respondieron por otro canal, marca como `received` |

### Paso 4: Gestion de RESPONDIDAS

Si hay requests con `x_has_response = True` y `x_state = sent`:
- Mostrar titulo + resumen del ultimo mensaje del partner en el chatter
- Preguntar al consultor: Marcar como recibida? Notas?
- Actualizar:
```bash
$PILOT_DIR/update_record.sh "x_client_request" <request_id> '{
  "x_state": "received"
}'
```

---

## Mode: CERRAR

### Paso 1: Requests con Respuesta No Procesada

- Ejecutar `fetch_requests.sh` filtrando `x_has_response=True` y `x_state=sent`
- Para cada una: mostrar titulo + ultimo mensaje del partner, preguntar marcar como recibida
- Permitir cerrar en batch

### Paso 2: Resumen

- Presentar resumen de lo cerrado esta sesion
- Si quedan vencidas sin respuesta, sugerir modo FOLLOW-UP
