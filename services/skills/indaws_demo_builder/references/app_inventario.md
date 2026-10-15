# Inventario (Inventory) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| stock | Gestion de inventario | Siempre (si app seleccionada) |
| stock_account | Valoracion de inventario | Si necesita contabilidad de stock |
| stock_barcode | Escaner de codigos de barras | Si usa lectores en almacen |
| stock_landed_costs | Costes en destino | Si hay costes de importacion |
| delivery | Metodos de envio | Si envia a clientes |
| stock_dropshipping | Dropshipping | Si proveedor envia directo al cliente |

## Configuracion

### Ajustes Principales
- **Almacenes**: Configurar al menos 1 almacen principal con nombre del cliente
- **Ubicaciones**: Activar multi-ubicacion si el cliente tiene zonas diferenciadas
- **Trazabilidad**: Activar lotes/numeros de serie segun sector (alimentacion=lotes, electronica=serie)
- **Rutas multi-paso**: Activar si el cliente tiene recepciones/envios en 2+ pasos
- **Unidades de medida**: Activar si vende en diferentes UdM (kg, litros, m2, unidades)

### Features Opcionales
- Valoracion de inventario: FIFO o Coste medio segun sector
- Reglas de reabastecimiento automatico (orderpoint)
- Alertas de stock minimo
- Categorias de producto con rutas especificas

## Datos Maestros

### Almacenes y Ubicaciones
- 1 almacen principal (renombrar con nombre empresa del cliente)
- Si multi-ubicacion: 2-3 ubicaciones (Zona A, Zona B, Zona Frio si alimentacion)
- Ubicacion de consigna (si aplica)

### Productos con Stock
- Al menos 8-10 productos tipo "almacenable" (type=product)
- Stock inicial realista (usar inventory adjustment o quant directo)
- Algunos con stock bajo (para mostrar alertas)
- 1-2 sin stock (para mostrar solicitudes de reabastecimiento)

### Categorias de Producto
- 3-5 categorias internas (product.category) relevantes al sector
- Asignar categoria a cada producto

## Flujo de Negocio - Distribucion por Estado

### Transferencias Internas
**Borrador** - 15%: Planificadas pero no confirmadas
**En espera** - 20%: Confirmadas, esperando disponibilidad
**Preparado** - 30%: Material disponible, listo para mover
**Hecho** - 35%: Completadas

### Recepciones (desde compras)
- 2-3 recepciones completadas (historico)
- 1-2 recepciones pendientes/parciales (actual)

### Envios (desde ventas)
- 3-4 envios completados (historico)
- 1-2 envios preparados/pendientes (actual)

### Ajustes de Inventario
- 1 ajuste completado (simulando inventario fisico reciente)

## Tips para Demo Impactante
- Tener stock variado: algunos productos con mucho stock, otros con poco
- Crear al menos 1 transferencia interna en estado "preparado" para validar en demo
- Si trazabilidad activa, tener lotes/series en al menos 3 productos
- Mostrar la vista de stock por ubicacion (quant tree)
- Tener 1 regla de reabastecimiento configurada para que salte durante demo
- Crear 1 envio parcial para mostrar backorder automatico

## Comandos API (odoo-pilot)

```bash
# Ajustar stock inicial (via stock.quant)
execute_method.sh stock.quant 0 _update_available_quantity '{"product_id": PROD_ID, "location_id": LOC_ID, "quantity": 100}'

# Crear transferencia interna
create_record.sh stock.picking '{"picking_type_id": INTERNAL_TYPE_ID, "location_id": LOC_SRC, "location_dest_id": LOC_DEST, "move_ids_without_package": [[0, 0, {"product_id": PROD_ID, "product_uom_qty": 5, "name": "Transfer", "location_id": LOC_SRC, "location_dest_id": LOC_DEST}]]}'

# Confirmar + asignar + validar picking
execute_method.sh stock.picking PICKING_ID action_confirm
execute_method.sh stock.picking PICKING_ID action_assign
execute_method.sh stock.picking PICKING_ID button_validate
```
