<!-- verified: v18.0 @ 2026-06 (stock_landed_costs ↔ purchase/account/mrp_landed_costs/subcontracting; anclas verificadas vs fuente) -->
# Landed Costs — Integrations (Odoo 18.0 CE)

Cómo Landed Costs conecta con compras, contabilidad y fabricación. Núcleo en [`core.md`](core.md);
gotchas del motor en [`gotchas.md`](gotchas.md) (LCG1-14).

## Compras y factura

1. **Producto LC**: `product.template.landed_cost_ok` + `split_method_landed_cost` (`product.py`);
   debe ser **tipo servicio** (`account_move.py::_onchange_is_landed_costs_line`).
2. **Crear el LC desde la factura del proveedor/transportista**: `account.move.button_create_landed_costs`
   (`stock_landed_costs/models/account_move.py:21`, `AccountMove.button_create_landed_costs`) crea un `stock.landed.cost` con `vendor_bill_id` y una cost line por cada
   `account.move.line` con `is_landed_costs_line=True` (price_subtotal → moneda compañía).
3. **PO line propaga el flag**: `purchase.py::PurchaseOrderLine._prepare_account_move_line` pone
   `is_landed_costs_line = product_id.landed_cost_ok` → el botón "Create Landed Costs" aparece en la factura.

## Objetivo: recepciones vs fabricación

4. **Recepciones** (CE): `target_model='picking'` + `picking_ids`; `_get_targeted_move_ids` =
   `picking_ids.move_ids`.
5. **Fabricación** (`mrp_landed_costs`, auto_install con mrp): `target_model` gana `'manufacturing'` +
   `mrp_production_ids`; reparte sobre `move_finished_ids` (los productos acabados).
6. **Subcontratación** (`mrp_subcontracting_landed_costs`, auto_install): solo facilita identificar
   pickings de subcontratación; el reparto usa el flujo `picking` estándar (sin target_model propio).

## Contabilidad

7. El asiento (`account.move` tipo `entry`) usa `account_journal_id`; débito = `stock_valuation` del
   producto (o `expense` si dropship), crédito = `account_id` de la cost line o
   `property_stock_account_input_categ_id`. Solo para productos `valuation='real_time'`.
8. `reconcile_landed_cost` reconcilia `stock_input` entre la factura y el asiento del LC, solo en
   anglosajón con la factura posteada.

## Gotchas cross-module

**LCG15. El botón "Create Landed Costs" solo aparece si la línea de factura es LC.** Depende de
`is_landed_costs_line` (propagado desde la PO line si el producto es `landed_cost_ok`). Una factura de
flete con un producto normal no ofrece crear el LC.

**LCG16. En fabricación el reparto va a los productos ACABADOS, no a los componentes.**
`mrp_landed_costs::_get_targeted_move_ids` usa `mrp_production_ids.move_finished_ids`. El coste se
suma a la valoración del producto fabricado.

**LCG17. La reconciliación factura↔LC solo en anglosajón + factura posteada.** Sin
`anglo_saxon_accounting` o con la factura en borrador, no hay reconciliación automática de `stock_input`.

**LCG18. Un LC creado desde factura NO impacta dos veces.** El SVL del LC no toca el price_unit del
receipt cuando hay `vendor_bill_id` (`_should_impact_price_unit_receipt_value`), para no duplicar con
el coste de la propia factura.

**LCG19. La subcontratación NO añade un modelo/objetivo propio.** `mrp_subcontracting_landed_costs`
reparte vía el picking de la recepción subcontratada (flujo `picking` normal), no por la MO.

**LCG20. Cambiar/desactivar un producto LC ya usado está bloqueado.** `stock_landed_costs/models/product.py:18`, `ProductTemplate.write` lanza `UserError`
si intentas quitar `landed_cost_ok` o cambiar el tipo de un producto ya presente en un apunte LC.
