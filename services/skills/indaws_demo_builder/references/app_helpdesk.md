# Helpdesk - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| helpdesk | Helpdesk base | Siempre (si app seleccionada) |
| helpdesk_timesheet | Horas en tickets | Si registra horas de soporte |
| helpdesk_sale | Tickets desde ventas | Si vende soporte |
| helpdesk_stock | Devoluciones | Si gestiona RMAs |

## Configuracion

### Ajustes Principales
- **Equipos**: Crear 1-2 equipos (Soporte Tecnico, Atencion al Cliente)
- **SLAs**: Definir 2-3 reglas de SLA (respuesta, resolucion)
- **Etapas**: Nuevo, En Progreso, En Espera, Resuelto, Cerrado
- **Tipos de ticket**: Incidencia, Consulta, Solicitud de mejora
- **Asignacion automatica**: Activar round-robin si varios agentes

### Features Opcionales
- Portal del cliente para seguimiento de tickets
- Base de conocimiento (knowledge) para autoservicio
- Ratings/CSAT para satisfaccion del cliente
- Acuerdos SLA con alertas automaticas

## Datos Maestros

### Equipos (1-2)
- Equipo principal con 2-3 miembros
- Horario de atencion configurado
- SLAs asignados al equipo

### Tickets (10-15)
- Distribuidos en todas las etapas
- Prioridades variadas (baja, media, alta, urgente)
- Al menos 2 con SLA cumplido y 2 con SLA vencido
- Clientes diferentes (no todos del mismo)
- Notas internas y respuestas en chatter

## Flujo de Negocio - Distribucion por Estado

### Nuevo - 20%
**Crear**: helpdesk.ticket en stage "Nuevo"
**Para la demo**: Tickets recien recibidos sin asignar

### En Progreso - 35%
**Crear**: Mover a "En Progreso", asignar agente
**Para la demo**: Trabajo activo de soporte

### En Espera - 15%
**Crear**: Mover a "En Espera" (esperando respuesta cliente)
**Para la demo**: Mostrar que hay seguimiento

### Resuelto - 25%
**Crear**: Mover a "Resuelto"
**Para la demo**: Tickets cerrados, para metricas

### Cerrado - 5%
**Crear**: Mover a "Cerrado" (fold=True)
**Para la demo**: Historico

## Tips para Demo Impactante
- Tener 1 ticket urgente con SLA a punto de vencer (genera urgencia visual)
- Mostrar dashboard con metricas del equipo
- 1 ticket con respuesta del cliente en portal (si activo)
- Registrar horas de soporte en al menos 2 tickets
- Mostrar automatizacion: ticket asignado automaticamente al crear

## Comandos API (odoo-pilot)

```bash
# Crear ticket
create_record.sh helpdesk.ticket '{"name": "Error en facturacion", "partner_id": PARTNER_ID, "team_id": TEAM_ID, "priority": "2"}'

# Mover de etapa
update_record.sh helpdesk.ticket TICKET_ID '{"stage_id": STAGE_ID}'
```
