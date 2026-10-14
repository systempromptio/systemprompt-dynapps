# Ejemplo Junior — Descuento automatico por volumen

**Titulo**: Descuento automatico por volumen en lineas de pedido venta | **Odoo**: 18.0
**Asignado**: Carlos Martinez (Junior) | **Estimacion**: 16h | **Limite**: 2026-04-02

## Contexto de Negocio

El comercial de Pulycort aplica descuentos por volumen manualmente (consulta Excel, escribe descuento a mano). Genera errores y retrasos. Con esto, el descuento se calcula automaticamente al introducir cantidad.

## Objetivo

Al introducir cantidad en linea de pedido, el sistema aplica descuento segun reglas de volumen del producto/categoria. El comercial puede sobreescribir (es sugerencia, no bloqueo).

## Alcance

**Incluido**:
- Modelo de reglas de descuento
- Calculo auto en `sale.order.line`
- Vista de configuracion de reglas
- Override manual

**Excluido**:
- NO aplicar en facturacion
- NO acumulado entre pedidos
- NO integracion con pricelist
- NO migracion de Excel

## Guia de Implementacion (adaptada Junior — pasos numerados con refs)

**Paso 1**: Investiga si ya existe. Revisa `sale_loyalty` (v18) — si cubre descuento por volumen, NO desarrollar, avisame.
- Ruta: `$ODOO_SRC/odoo18/addons/sale_loyalty/`

**Paso 2**: Revisa `sale.order.line` campo `discount` y como se usa.
- Ruta: `$ODOO_SRC/odoo18/addons/sale/models/sale_order_line.py`
- Busca: `discount` en ese fichero para ver donde se aplica y como.

**Paso 3**: Crea modelo nuevo para reglas (producto/categoria + qty minima + descuento).
- Ref patron: mira como `product.pricelist.item` define reglas escalonadas.
- Ruta: `$ODOO_SRC/odoo18/addons/product/models/product_pricelist.py`
- Clave: las reglas se ordenan por qty descendente (mayor qty = mayor prioridad).

**Paso 4**: Extiende `sale.order.line` con onchange/compute al cambiar `product_id` o `product_uom_qty`.
- Cuidado: no pisar descuento manual. Considera un booleano `discount_manual` para distinguir.
- Ref patron: como `sale_margin` reacciona a cambios en `sale_margin/models/sale_order_line.py`.

**Paso 5**: Crea menu `Ventas > Configuracion > Descuentos por Volumen`. Indicador visual auto vs manual en la linea.

**Paso 6**: Tests — minimo estos escenarios:
- qty sin regla → descuento 0%
- qty > 100 (regla 5%) → descuento 5%
- qty > 500 (regla 10%) → descuento 10%
- cambio de producto → recalcula
- override manual → no se pisa al cambiar qty
- regla producto > regla categoria (prioridad)
- Ref patron: `$ODOO_SRC/odoo18/addons/sale_margin/tests/`

## Criterios de Aceptacion

- [ ] Producto con regla >100uds=5%: qty=150 → descuento 5% automatico
- [ ] Cambio qty 150→50 → descuento 0%
- [ ] Cambio qty 150→600 (regla >500=10%) → descuento 10%
- [ ] Override manual a 7% → no se sobreescribe al cambiar qty
- [ ] Regla producto tiene prioridad sobre categoria
- [ ] Tests cubren los 6 escenarios

## Referencias

| Recurso | Ruta | Para que |
|---------|------|----------|
| `product.pricelist.item` | `$ODOO_SRC/odoo18/addons/product/models/product_pricelist.py` | Patron reglas escalonadas |
| `sale_margin` | `$ODOO_SRC/odoo18/addons/sale_margin/` | Computed en sale.order.line |
| `sale_loyalty` | `$ODOO_SRC/odoo18/addons/sale_loyalty/` | Verificar que no existe ya |

## Restricciones

- No pisar pricelist (mecanismos independientes) — confirmar conmigo si no esta claro
- No `sudo()`: respetar multi-company
- Naming: modulo `sale_volume_discount`, campos sin prefijo `x_` (es desarrollo Core)

## Preguntas Abiertas

- Confirmar que la categoria a usar es `product.category` (no `pos_categ_id`)

## Ayuda

- Timer: pide ayuda si llevas **1.5h** atascado sin avanzar
- Contacto: Toni por Slack #dev-tokyo

## Entregables

1. [ ] Rama `feature/sale_volume_discount`
2. [ ] Tests con los 6 escenarios
3. [ ] PR con descripcion
4. [ ] Staging validado por comercial Pulycort
