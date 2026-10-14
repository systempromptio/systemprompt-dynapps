# Ventas (Sales) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| sale_management | Gestion de ventas completa | Siempre |
| sale_stock | Integracion ventas-inventario | Si hay productos fisicos |
| sale_margin | Margenes en pedidos | Si el cliente necesita control de margenes |
| sale_loyalty | Programas de fidelidad/cupones | Si hay promociones |
| sale_project | Creacion de proyectos desde ventas | Si vende servicios con proyecto |
| sale_purchase | Compra desde venta (dropshipping) | Si hay dropshipping |

## Configuracion

### Ajustes Principales
- **Presupuestos y Pedidos**: Activar "Confirmacion en linea" para portal
- **Precios**: Activar listas de precios si el cliente maneja varios precios
- **Variantes de producto**: Activar si el sector lo requiere (muebles, moda)
- **Unidades de medida**: Activar si vende a granel o con UdM especiales
- **Descuentos**: Activar descuentos en lineas de pedido

### Features Opcionales
- Plantillas de presupuesto: Crear 2-3 plantillas con productos pre-cargados
- Terminos de pago: Configurar al menos 3 (Contado, 30 dias, 60 dias)
- Posiciones fiscales: Configurar segun tipo de cliente (nacional, intracomunitario, extracomunitario)
- Portal del cliente: Activar para mostrar autoservicio

## Datos Maestros

### Productos (10-15 productos)
- Minimo 5 productos fisicos (tipo=product) con stock
- 2-3 servicios (tipo=service) si aplica
- Variantes en al menos 2 productos (talla/color/modelo)
- Imagenes en todos los productos
- Precios de venta realistas del sector
- Al menos 2 categorias de producto

### Clientes (8-12 contactos)
- 5-8 empresas (is_company=True) con direccion completa
- 2-3 particulares
- Al menos 2 clientes con terminos de pago diferentes
- Posiciones fiscales variadas

### Listas de Precios (si activadas)
- Lista publica (precio base)
- Lista VIP o mayorista (-10% a -20%)

## Flujo de Negocio - Distribucion por Estado

### Presupuesto (draft) - 30% de registros
**Crear**: sale.order con state=draft
**Campos clave**: partner_id, order_line, date_order, validity_date, payment_term_id
**Para la demo**: Presupuestos pendientes de enviar al cliente, mostrar edicion y envio

### Pedido Confirmado - 25% de registros
**Crear**: sale.order confirmado con action_confirm
**Para la demo**: Pedidos listos para entregar/facturar, mostrar flujo de confirmacion

### Entregado - 20% de registros
**Crear**: Confirmar + validar picking asociado (stock.picking con button_validate)
**Para la demo**: Pedidos entregados pendientes de facturar

### Facturado - 15% de registros
**Crear**: Pedido entregado + _create_invoices() + action_post en factura
**Para la demo**: Facturas emitidas pendientes de cobro

### Pagado - 10% de registros
**Crear**: Factura + registrar pago (account.payment)
**Para la demo**: Flujo completo cerrado, para ver historico

## Tips para Demo Impactante
- Crear al menos 1 presupuesto con plantilla para mostrar autocompletado
- Tener 1 pedido con multiples lineas y descuento para mostrar totales
- Crear 1 presupuesto "grande" (importe alto) para que destaque en el pipeline
- Programar actividades en 2-3 presupuestos (llamadas, reuniones de seguimiento)
- Registrar 1-2 notas en el chatter de algun pedido para simular uso real
- Si hay portal activo, tener 1 presupuesto pendiente de firma del cliente

## Comandos API (odoo-pilot)

```bash
# Crear presupuesto
create_record.sh sale.order '{"partner_id": PARTNER_ID, "order_line": [[0, 0, {"product_id": PROD_ID, "product_uom_qty": 2}]]}'

# Confirmar pedido
execute_method.sh sale.order ORDER_ID action_confirm

# Crear factura desde pedido
execute_method.sh sale.order ORDER_ID _create_invoices

# Validar factura
execute_method.sh account.move INVOICE_ID action_post
```
