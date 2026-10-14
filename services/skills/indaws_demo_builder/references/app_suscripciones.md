# Suscripciones (Subscriptions) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| sale_subscription | Suscripciones base | Siempre (si app seleccionada) |
| sale_subscription_stock | Suscripciones con envio | Si envia productos periodicos |

## Configuracion

### Ajustes Principales
- **Planes de suscripcion**: Crear 2-3 planes (Mensual, Trimestral, Anual)
- **Productos recurrentes**: Marcar productos como recurrentes
- **Plantillas de presupuesto**: Con productos de suscripcion pre-cargados
- **Facturacion automatica**: Configurar generacion automatica de facturas

### Features Opcionales
- Portal del cliente para gestion de suscripcion
- Upselling automatico
- Alertas de churn (riesgo de baja)

## Datos Maestros

### Planes de Suscripcion (2-3)
- Plan Mensual (recurrence_id con repeticion mensual)
- Plan Anual (con descuento vs mensual)
- Plan Premium (opcional, con servicios adicionales)

### Productos Recurrentes (3-5)
- Licencia/Servicio basico (ej: "Plan Estandar", "Servicio Mensual")
- Servicio premium
- Complemento/addon opcional
- Todos con precio recurrente

## Flujo de Negocio - Distribucion por Estado

### Suscripciones (sale.order con recurrence)
| Estado | % | Descripcion |
|--------|---|-------------|
| Borrador | 10% | Propuestas de suscripcion |
| En Progreso | 50% | Suscripciones activas |
| Pausada | 10% | Temporalmente detenidas |
| Cerrada | 20% | Finalizadas normalmente |
| Cancelada | 10% | Canceladas por el cliente |

### Distribucion Temporal
- Suscripciones activas: inicio hace 1-6 meses (variado)
- Suscripciones cerradas: inicio hace 6-12 meses, cierre hace 1-3 meses
- Al menos 1 con proxima factura este mes

## Tips para Demo Impactante
- Mostrar dashboard de MRR (Monthly Recurring Revenue) con datos
- Tener suscripciones con diferentes planes para comparar
- 1 suscripcion proxima a renovar (para mostrar el proceso)
- Mostrar upsell: anadir servicio adicional a suscripcion existente
- Tener historico de facturas recurrentes generadas
- Si portal activo, mostrar autoservicio del cliente

## Comandos API (odoo-pilot)

```bash
# Crear suscripcion
create_record.sh sale.order '{"partner_id": PARTNER_ID, "recurrence_id": PLAN_ID, "order_line": [[0, 0, {"product_id": RECURRING_PROD_ID, "product_uom_qty": 1}]]}'

# Confirmar suscripcion
execute_method.sh sale.order SUB_ID action_confirm

# Pausar suscripcion
execute_method.sh sale.order SUB_ID set_paused

# Cerrar suscripcion
execute_method.sh sale.order SUB_ID set_close
```
