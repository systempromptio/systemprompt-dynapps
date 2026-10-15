# Compras (Purchases) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| purchase | Gestion de compras | Siempre (si app seleccionada) |
| purchase_stock | Integracion compras-inventario | Si hay productos fisicos |
| purchase_requisition | Acuerdos de compra | Si hay contratos marco con proveedores |
| purchase_mrp | Compras-fabricacion | Si hay MRP activo |

## Configuracion

### Ajustes Principales
- **Acuerdos de compra**: Activar si el cliente negocia precios por volumen/periodo
- **Aprobacion de compras**: Activar si hay control de gastos (doble aprobacion)
- **Reglas de reabastecimiento**: Configurar para productos clave
- **Unidades de medida**: Consistentes con ventas

### Features Opcionales
- Acuerdos marco: Para proveedores recurrentes con precios pactados
- Politica de recepcion: "Recibir cada producto al llegar" o "Todo de una vez"
- Analisis de compras: Dashboard activado por defecto

## Datos Maestros

### Proveedores (3-6 contactos)
- Al menos 3 proveedores con datos completos (nombre, direccion, email, telefono)
- 1 proveedor con terminos de pago a 60 dias
- 1 proveedor con acuerdo marco (si feature activa)
- Informacion de proveedor en los productos (seller_ids con precios y plazos)

### Productos con Proveedores
- Asociar al menos 5 productos con proveedor (product.supplierinfo)
- Incluir plazo de entrega del proveedor (delay)
- Incluir precio de compra (price)
- Al menos 1 producto con 2 proveedores alternativos

### Reglas de Reabastecimiento (si configuradas)
- 2-3 productos con regla de stock minimo
- Punto de pedido realista segun sector

## Flujo de Negocio - Distribucion por Estado

### Solicitud de Presupuesto (draft) - 20% de registros
**Crear**: purchase.order con state=draft
**Campos clave**: partner_id, order_line, date_order, date_planned
**Para la demo**: SdP pendientes de enviar al proveedor

### PO Confirmada - 30% de registros
**Crear**: purchase.order + button_confirm
**Para la demo**: Pedidos enviados al proveedor, pendientes de recepcion

### Parcialmente Recibida - 15% de registros
**Crear**: PO confirmada + validar picking parcial
**Para la demo**: Recepciones en curso, mostrar trazabilidad

### Totalmente Recibida - 20% de registros
**Crear**: PO confirmada + validar picking completo
**Para la demo**: Pendientes de factura del proveedor

### Facturada y Pagada - 15% de registros
**Crear**: PO recibida + crear factura + action_post + pago
**Para la demo**: Flujo completo cerrado

## Tips para Demo Impactante
- Crear 1 PO con varias lineas para mostrar recepcion parcial
- Tener 1 producto con regla de reabastecimiento que dispare PO automatica
- Si hay acuerdos marco, mostrar como selecciona automaticamente el mejor precio
- Tener algun producto con plazo de entrega largo para mostrar planificacion
- Registrar nota del proveedor en alguna PO (simula comunicacion)

## Comandos API (odoo-pilot)

```bash
# Crear solicitud de presupuesto
create_record.sh purchase.order '{"partner_id": SUPPLIER_ID, "order_line": [[0, 0, {"product_id": PROD_ID, "product_qty": 10, "price_unit": 15.0}]]}'

# Confirmar pedido de compra
execute_method.sh purchase.order PO_ID button_confirm

# Validar recepcion
execute_method.sh stock.picking PICKING_ID button_validate
```
