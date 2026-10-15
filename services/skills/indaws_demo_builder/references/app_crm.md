# CRM - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| crm | CRM base | Siempre (si app seleccionada) |
| crm_iap_mine | Generacion de leads | Si quiere prospeccion automatica |
| sale_crm | Integracion CRM-Ventas | Si tiene Ventas activo |
| website_crm | Formularios web | Si tiene Website activo |

## Configuracion

### Ajustes Principales
- **Equipos de ventas**: Crear 1-2 equipos con miembros asignados
- **Etapas del pipeline**: Personalizar etapas segun proceso del cliente
- **Actividades**: Configurar tipos de actividad (llamada, reunion, email, demo)
- **Scoring**: Activar si quiere puntuacion automatica de leads
- **Asignacion**: Configurar reglas de asignacion si hay varios comerciales

### Etapas Recomendadas para Demo
1. Nuevo (Lead)
2. Calificado
3. Demo programada / Propuesta
4. Negociacion
5. Ganado
6. Perdido (fold=True)

## Datos Maestros

### Pipeline (12-18 oportunidades)
- Distribuir en todas las etapas (no solo "Nuevo")
- Revenue esperado realista segun sector
- Probabilidades variadas (10% a 90%)
- Al menos 3 con actividades programadas
- Al menos 2 con notas en el chatter
- Fechas de cierre distribuidas (pasadas, este mes, futuro)

### Equipos de Ventas
- 1-2 equipos con 2-3 miembros cada uno
- Objetivo mensual del equipo configurado

### Tags
- 3-5 etiquetas relevantes al sector (ej: "VIP", "Urgente", "Renovacion", "Nuevo cliente")

## Flujo de Negocio - Distribucion por Estado

### Nuevo - 15% de leads
**Crear**: crm.lead con stage_id del primer stage
**Para la demo**: Leads recien llegados, mostrar calificacion

### Calificado - 20%
**Crear**: Mover a stage calificacion
**Para la demo**: Contacto realizado, informacion basica completa

### Propuesta/Demo - 25%
**Crear**: Mover a stage propuesta, asociar presupuesto si hay ventas
**Para la demo**: Presupuesto enviado, pendiente de respuesta

### Negociacion - 20%
**Crear**: Mover a stage negociacion, revenue alto
**Para la demo**: En negociacion activa, mostrar actividades

### Ganado - 15%
**Crear**: action_set_won + date_closed en el pasado
**Para la demo**: Cerrados con exito, para estadisticas

### Perdido - 5%
**Crear**: action_set_lost con lost_reason
**Para la demo**: Para analisis de motivos de perdida

## Tips para Demo Impactante
- Pipeline visual con oportunidades en cada etapa (vista kanban atractiva)
- Al menos 2 oportunidades con valor alto para que destaquen
- Actividades vencidas en 1-2 oportunidades (muestra urgencia)
- 1 oportunidad recien ganada (confeti/animacion en Odoo 17+)
- Asociar presupuesto a al menos 1 oportunidad (si Ventas activo)
- Configurar prevision mensual con datos suficientes para graficos

## Comandos API (odoo-pilot)

```bash
# Crear oportunidad
create_record.sh crm.lead '{"name": "Nombre oportunidad", "partner_id": PARTNER_ID, "expected_revenue": 5000, "probability": 30, "stage_id": STAGE_ID, "user_id": USER_ID, "type": "opportunity"}'

# Mover de etapa
update_record.sh crm.lead LEAD_ID '{"stage_id": NEW_STAGE_ID}'

# Marcar como ganada
execute_method.sh crm.lead LEAD_ID action_set_won

# Marcar como perdida
execute_method.sh crm.lead LEAD_ID action_set_lost '{"lost_reason_id": REASON_ID}'
```
