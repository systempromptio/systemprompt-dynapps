# Punto de Venta (Point of Sale) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| point_of_sale | TPV base | Siempre (si app seleccionada) |
| pos_sale | Integracion POS-Ventas | Si combina ventas y TPV |
| pos_restaurant | Restaurante | Si es hosteleria con mesas |
| pos_loyalty | Fidelidad en TPV | Si usa programas de puntos |

## Configuracion

### Ajustes Principales
- **Punto de venta**: Crear 1-2 TPVs con nombre descriptivo (ej: "Tienda Principal", "Barra")
- **Metodos de pago**: Efectivo + Tarjeta (minimo)
- **Categorias POS**: Organizar productos por categoria visual
- **Impuestos**: IVA incluido en precios para B2C
- **Recibos**: Personalizar con datos de empresa

### Features Opcionales (restaurante)
- Gestion de mesas y pisos
- Notas de cocina
- Dividir cuenta
- Propinas

## Datos Maestros

### Productos Disponibles en POS
- 10-15 productos con "Disponible en POS" activo
- Todos con precio de venta
- Categorias POS visuales (con iconos si posible)
- Al menos 1 combo o pack

### Sesiones y Pedidos Historicos
- 2-3 sesiones cerradas (historico de ultimos dias)
- 5-8 pedidos POS por sesion (con mezcla de metodos de pago)
- Al menos 1 devolucion

## Flujo de Negocio - Distribucion por Estado

### Pedidos POS (pos.order)
| Estado | % | Descripcion |
|--------|---|-------------|
| Pagado | 90% | Ventas completadas normalmente |
| Facturado | 10% | Con factura asociada (cliente B2B) |

### Sesiones (pos.session)
- 2-3 sesiones cerradas (con cierre de caja correcto)
- 1 sesion abierta (para demo en vivo)

## Tips para Demo Impactante
- Abrir sesion de TPV y hacer 1-2 ventas en vivo
- Mostrar diferentes metodos de pago (efectivo + tarjeta)
- Si restaurante, mostrar gestion de mesas y plano
- Tener sesiones previas cerradas para mostrar historico y cierre de caja
- Mostrar devolucion de un pedido anterior
- Dashboards de ventas POS con graficos

## Comandos API (odoo-pilot)

```bash
# Los pedidos POS normalmente se crean via la interfaz POS
# Para historico, crear sesion + pedidos via API:

# Abrir sesion
execute_method.sh pos.session 0 create '{"config_id": POS_CONFIG_ID}'

# Los pedidos POS tienen estructura compleja, se recomienda
# crearlos via la interfaz o importacion
```
