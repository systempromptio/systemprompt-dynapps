# Client Requests (Peticiones al Cliente)


> **Trazabilidad perdida (sept-2026)**: el campo `x_checklist_id`, que vinculaba una petición con un item concreto del checklist, se eliminó. `xx.checklist.item` tampoco tiene campo de vuelta, así que ese vínculo ya no existe en el modelo de datos y no es recuperable.

## Qué es

El modelo `x_client_request` para gestionar peticiones formales al cliente (datos, documentos, decisiones, accesos, validaciones). Usa mail.thread para comunicación via chatter.

## Cuándo aplica

- Fase 3 (Pre-proyecto): Generar peticiones iniciales desde el análisis
- Fase 4 (Ejecución): Follow-up, recordatorios y cierre de peticiones
- Cualquier momento: Cuando el consultor necesita algo del cliente

## Qué debe saber el consultor

### Modelo x_client_request

| Campo | Tipo | Descripción |
|-------|------|-------------|
| x_name | Char | Título descriptivo |
| x_task_id | Many2one → project.task | Tarea asociada |
| x_project_id | Related | Proyecto (via tarea) |
| x_partner_id | Many2one → res.partner | Contacto del cliente |
| x_description | Html | Descripción detallada |
| x_type | Selection | information / data_file / validation / decision / access |
| x_priority | Selection | 0 (normal) / 1 (urgente) |
| x_state | Selection | draft / sent / received / cancelled |
| x_deadline | Date | Fecha límite |
| x_date_sent | Datetime (computed) | Fecha de primer envío |
| x_date_received | Datetime (computed) | Fecha de recepción |
| x_is_overdue | Boolean (computed) | sent + deadline < hoy |
| x_has_response | Boolean (computed) | Hay mensajes del partner en chatter |

### 3 Modos del skill

| Modo | Qué hace | Cuándo |
|------|----------|--------|
| **GENERAR** | Analiza TODAS las tareas del proyecto + análisis funcional, crea requests en draft | Al inicio del proyecto |
| **FOLLOW-UP** | Dashboard, envío de emails, recordatorios, detección de respuestas | Ejecución |
| **CERRAR** | Marca como recibidas las requests respondidas (bulk) | Ejecución |

### 5 Tipos de petición

| Tipo | Descripción | Deadline típico |
|------|-------------|-----------------|
| information | Información necesaria para configurar | 7 días |
| data_file | Fichero de datos para importar (Excel/CSV) | 14 días |
| validation | Validación de lo implementado | 7 días |
| decision | Decisión del cliente sobre alternativas | 5 días |
| access | Accesos a sistemas externos | 7 días |

### Cotejo inteligente (modo GENERAR)

Al generar requests, el skill coteja 5 fuentes:
1. Descripción de la tarea
2. Checklists de la tarea
3. Patrones por tipo de tarea
4. Análisis funcional (si se proporciona)
5. Requests existentes (evitar duplicados)

Si la información ya está en el análisis funcional, la petición se **descarta** (se muestra al usuario con referencia a la sección del análisis donde está la información).

## Qué debe hacer el consultor

1. Ejecutar modo GENERAR al inicio del proyecto (analiza todas las tareas a la vez)
2. Validar la lista de peticiones antes de que se creen
3. Ejecutar modo FOLLOW-UP regularmente para enviar y hacer seguimiento
4. Revisar y aprobar los borradores de email antes de enviar
5. Cerrar las requests respondidas con modo CERRAR

## Skills relacionados

- `client-request-manager`: Skill principal para gestionar requests
- `importation-data-assistant`: Localiza archivos adjuntos en requests respondidas
- `project-validation-drafting`: Crea requests de tipo "validation"
- `consultant-daily-planner`: Prioriza requests vencidas y pendientes

## Errores comunes

1. **Ejecutar GENERAR tarea por tarea**: Debe ejecutarse al inicio analizando todas las tareas del proyecto
2. **No comprobar requests existentes**: Puede crear duplicados
3. **No hacer follow-up**: Las requests sin seguimiento se pierden
4. **Mencionar información interna en emails**: NUNCA incluir datos de horas, scores o información interna
5. **No establecer deadlines**: Sin fecha límite, el cliente no prioriza la respuesta
