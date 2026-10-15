# Vinculacion de Skills

## Mapa de Skills Disponibles

Cada accion de la agenda debe incluir el skill a ejecutar y los parametros necesarios para que el consultor pueda invocar directamente.

### Skills Existentes

| Skill | Invocacion | Parametros | Prerequisitos |
|-------|-----------|------------|---------------|
| `client-request-manager` | Modo GENERAR / FOLLOW-UP / CERRAR | Proyecto ID, Tarea ID (GENERAR) | Modelo x_client_request existente |
| `odoo-configurator` | Ejecutar checklists de tarea | Tarea ID, Entorno (PROD/TEST) | Credenciales Odoo |
| `training-planner` | Modo PLANIFICAR / CONTENIDO | Proyecto ID (PLANIFICAR), Tarea ID (CONTENIDO) | Analisis funcional en Knowledge |
| `project-meeting-minutes` | Generar acta | Proyecto ID, notas de reunion | x_studio_knowledge configurado |
| `dev-assessment` | Evaluar desarrollo | Tarea [DEV] ID | Descripcion de la tarea |
| `importation-data-assistant` | Importar datos | Client request ID con adjunto | Datos en x_client_request |
| `project-validation-drafting` | Modo PLANIFICAR / DOCUMENTAR | Proyecto ID (PLANIFICAR), Request ID (DOCUMENTAR) | Tarea completada |
| `odoo-flow-tester` | Modo A / B / C | Proyecto ID, Flujo | Datos de prueba existentes |
| `service-desk` | Diagnosticar ticket | Ticket ID | Acceso al Odoo del cliente |
| `project-weekly-update` | Generar update semanal | Proyecto ID | project-health-monitor (se lanza automaticamente) |
| `project-health-fixer` | Corregir problemas | Proyecto ID | project-health-monitor ejecutado (JSON de metricas) |

### Skills Futuros (no creados aun)

| Skill | Proposito | Cuando se usara |
|-------|-----------|-----------------|
| `technical_dev_validation` | Testear desarrollos entregados | Tarea [DEV] completada por desarrollador |
| `go-live-validation` | Checklist pre-produccion | Go-Live <= 30 dias |

Cuando estos skills no existan, la agenda debe indicar:
- `→ Skill no disponible aun. Accion manual requerida.`

---

## Formato de Accion en la Agenda

Cada accion debe incluir:

```
**N. [Proyecto] Titulo descriptivo** (~Xmin)
   - Contexto: que se ha detectado y por que es urgente/importante
   - IDs relevantes: request #X, tarea #Y, ticket #Z
   → Ejecutar: `nombre-del-skill` modo X, proyecto/tarea/ticket ID
   → Prerequisito: (si aplica) ejecutar primero `otro-skill`
```

## Dependencias entre Skills

Algunas acciones requieren ejecutar primero otro skill:

| Skill | Prerequisito | Razon |
|-------|-------------|-------|
| `project-health-fixer` | `project-weekly-update` (que lanza `project-health-monitor`) | Necesita JSON de metricas actualizado |
| `training-planner` CONTENIDO | Verificar que la sesion esta en el plan | El plan debe existir |
| `importation-data-assistant` | `client-request-manager` CERRAR | Marcar request como recibida primero |
| `project-validation-drafting` DOCUMENTAR | Checklists de la tarea completados | No validar antes de terminar |
| `project-meeting-minutes` | Reunion realizada | No generar acta de reunion futura |

La agenda debe detectar estas dependencias y avisar si falta un prerequisito.
