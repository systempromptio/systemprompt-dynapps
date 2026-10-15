<!-- verified: v18.0 @ 2026-06 (mrp CE + mrp_subcontracting/mrp_account + mrp_workorder/mrp_mps/quality/mrp_plm/mrp_maintenance EE; anclas vs fuente) -->
# MRP (`mrp`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md). **EE** = Enterprise. CE cubre ~11/18; EE completa.

**MRPCAP1. ¿BOM multinivel (sub-conjuntos)?** — **NATIVO** · baja · `mrp.bom.line → child_bom_id` (expande sub-BOMs en la MO).
**MRPCAP2. ¿Kits / phantom BOM?** — **NATIVO** · baja · `mrp.bom → type='phantom'` (se explota en venta/entrega sin MO).
**MRPCAP3. ¿BOM por variante?** — **NATIVO** · media · `mrp.bom → product_id` (variante concreta) vs `product_tmpl_id` (todas).
**MRPCAP4. ¿Órdenes de trabajo con centros de trabajo y rutas?** — **NATIVO** · media · `mrp.routing.workcenter` + `mrp.workcenter`; operaciones con `time_cycle`/`blocked_by_operation_ids`.
**MRPCAP5. ¿Capacidad/carga de centros de trabajo?** — **NATIVO** (cálculo) · media · `mrp.workcenter → default_capacity`/`workcenter_load`. Planificación con restricciones finitas NO nativa (sería desarrollo).
**MRPCAP6. ¿Subcontratación (fabricación externa)?** — **NATIVO (CE)** · alta · módulo `mrp_subcontracting` (+`_purchase`): BOM tipo subcontract crea PO al subcontratista, gestiona componentes.
**MRPCAP7. ¿Plan maestro de producción (MPS)?** — **NATIVO (EE)** · alta · `mrp_mps → mrp.production.schedule` (forecast por periodo, replenishment).
**MRPCAP8. ¿Controles de calidad en fabricación?** — **NATIVO (EE)** · alta · `quality_mrp`/`quality_mrp_workorder` (`quality.point` en MO/WO). Ver [[quality]].
**MRPCAP9. ¿Subproductos (by-products)?** — **NATIVO** · media · `mrp.bom → byproduct_ids` con `cost_share`.
**MRPCAP10. ¿Desechos (scrap)?** — **NATIVO** · baja · `stock.scrap → production_id`/`workorder_id`.
**MRPCAP11. ¿Fabricación bajo pedido (MTO desde venta)?** — **NATIVO** · media · `stock.rule → _run_manufacture` (procurement SO→MO). Ver playbook MTO.
**MRPCAP12. ¿Gestión de cambios de ingeniería / PLM (ECO)?** — **NATIVO (EE)** · alta · `mrp_plm → mrp.eco` (workflow de cambios sobre BOM/operaciones).
**MRPCAP13. ¿App de taller (shop floor / tablet)?** — **NATIVO (EE)** · alta · `mrp_workorder` (vista tablet `open_tablet_view`, timer, quality inline). Ver [[mrp_workorder]].
**MRPCAP14. ¿Trazabilidad de lotes/serie en producción?** — **NATIVO** · media · move lines (`lot_id`) en MO; reporte de trazabilidad upstream/downstream.
**MRPCAP15. ¿Mantenimiento de máquinas integrado?** — **NATIVO (EE)** · media · `mrp_maintenance` (workcenter↔`maintenance.equipment`, leaves de mantenimiento). Ver [[maintenance]].
**MRPCAP16. ¿Coste real de fabricación (componentes + operaciones)?** — **NATIVO** (módulo `mrp_account`, CE) · alta · coste de componentes + coste de operación (tarifa workcenter); variaciones a contabilidad.
**MRPCAP17. ¿Consumo flexible/backflush de componentes?** — **NATIVO** · media · `mrp.bom → consumption` (flexible/warning/strict); backflush al confirmar.
**MRPCAP18. ¿Reabastecimiento de componentes automático?** — **NATIVO** · media · `stock.rule` (manufacture) + orderpoints con `days_to_prepare_mo`.

---

## Kits (phantom BoM / "packs") — eje consultor (KITCAP)

> Un kit es una `mrp.bom` con `type='phantom'` (`mrp/models/mrp_bom.py:28-31`, `class MrpBom`). NO fabrica: se **explota** en sus componentes al confirmar el movimiento. Verificado contra `mrp`, `sale_mrp`, `mrp_account` v18 (+diffs v19). Gotchas relacionados: MG2, MG9, MG15, MG23.

### EE. Qué es y qué NO es un kit

**KITCAP1. ¿Puedo vender un "pack" que en realidad son varios productos?** — NATIVO · baja · `mrp.bom type='phantom'` · Sí; el kit es un producto con BoM phantom que se descompone en componentes.

**KITCAP2. ¿Un kit crea una orden de fabricación (MO)?** — NO · baja · `stock.move.action_explode` (MG2) · El phantom NO genera MO; se explota directamente en movimientos de componentes.

**KITCAP3. ¿Necesito instalar la app Fabricación para usar kits?** — SÍ · baja · módulo `mrp` (define `mrp.bom`) · El `type='phantom'` vive en `mrp`; sin él no hay kits nativos (alternativa: OCA `sale_product_pack`, distinto modelo).

**KITCAP4. ¿Cómo sé si un producto es un kit?** — NATIVO · baja · `product.is_kits` (compute+search, `mrp/product.py:32, `ProductTemplate.is_kits`,125`) · Campo booleano buscable; True si tiene BoM phantom activa aplicable.

**KITCAP5. ¿Un kit tiene stock propio?** — NO (calculado) · media · `_compute_quantities_dict` override (`mrp/product.py:264`, `ProductProduct._compute_quantities_dict`) · La disponibilidad del kit se **calcula desde los componentes** (cuántos kits completos se pueden montar), no se almacena; es el mínimo por componente.

**KITCAP6. ¿Puedo inventariar un kit como tal (ajuste de stock del kit)?** — NO · media · kit qty es computed · No se ajusta el kit; se ajustan los componentes. El kit no tiene `stock.quant` propio.

### FF. Explosión en venta / entrega

**KITCAP7. ¿Cuándo se descompone el kit?** — NATIVO · media · `stock.move._action_confirm`→`action_explode` (MG2) · Al confirmar el movimiento (entrega): el move del kit se borra y se sustituye por moves de componentes.

**KITCAP8. ¿En el pedido de venta el cliente ve el kit o los componentes?** — NATIVO (kit) · baja · `sale.order.line` (1 línea) · La SO muestra 1 línea = el kit a su precio; el albarán muestra los componentes.

**KITCAP9. ¿Cómo se calcula la cantidad entregada de un kit?** — NATIVO · media · `sale_mrp.sale_order_line._compute_qty_delivered` (`sale_mrp/models/sale_order_line.py:33`, `SaleOrderLine._compute_qty_delivered`) · El `qty_delivered` del kit es el **cuello de botella**: proporción del componente menos entregado (MG-kit). Si falta un componente, el kit no cuenta como entregado.

**KITCAP10. ¿Puedo entregar kits parcialmente?** — NATIVO (con matices) · media · qty_delivered bottleneck · Sí a nivel componente; el kit solo se considera entregado cuando todos sus componentes se entregan en proporción.

**KITCAP11. ¿El kit funciona en compras (recibir un pack del proveedor como componentes)?** — NATIVO · media · phantom explode en recepción · Sí; un PO de un producto kit explota en componentes al recibir (`qty_received` = bottleneck, ver [[playbook P2P]]).

### GG. Precio, facturación, valoración

**KITCAP12. ¿El precio del kit es la suma de sus componentes?** — NO (nativo) · media · `list_price` del producto kit · El precio es el del **producto kit** (su propio `list_price`/pricelist), NO la suma de componentes. "Precio desde BoM" = desarrollo/OCA.

**KITCAP13. ¿Puedo poner precio a los componentes dentro del kit?** — NO (venta) · media · — · Los componentes no aportan precio de venta; solo el kit se factura. Para desglose de precio por componente hace falta custom.

**KITCAP14. ¿Cómo se factura un kit?** — NATIVO · media · `sale_mrp/account_move.py` · Se factura la línea del kit (por `qty_delivered`/`ordered` según policy); anglo-saxon reparte COGS por componente.

**KITCAP15. ¿Cómo se reparte el coste entre el producto principal y los subproductos?** — NATIVO · media · `mrp.bom.byproduct.cost_share` (MG9) · Cada línea de subproducto (byproduct) lleva un % `cost_share` del coste total de producción; los componentes (`mrp.bom.line`) NO tienen `cost_share` y aportan cada uno su propia valoración. Si no se configura, todo el coste recae en el producto principal (gotcha MG9). La suma de `cost_share` de los subproductos debe ser ≤ 100%. Verificado 2026-07-15.

**KITCAP16. ¿La valoración de inventario es del kit o de los componentes?** — NATIVO (componentes) · media · SVL por componente · El kit no tiene valor propio; el valor está en los componentes en stock.

**KITCAP17. ¿El margen/coste del kit sale bien en informes?** — CONFIG · alta · anglo-saxon + cost_share · Depende de `cost_share` bien configurado; mal configurado distorsiona el margen (MG9).

### HH. Estructura del kit

**KITCAP18. ¿Un kit puede contener otro kit (anidado)?** — NATIVO · media · `explode` recursivo (`mrp/models/mrp_bom.py:396`, `MrpBom.explode`, cola BFS) · Sí; la explosión es recursiva vía `_bom_find(..., bom_type='phantom')` sobre los componentes.

**KITCAP19. ¿Un kit puede mezclar productos almacenables y servicios?** — NATIVO (parcial) · media · `get_components` filtra `is_storable` (`mrp/product.py:233`, `ProductProduct.get_components`) · Puede contener no-almacenables, pero solo los almacenables cuentan para disponibilidad/movimiento. Verificado 2026-07-15.

**KITCAP20. ¿Los componentes pueden estar en UoM distinta al kit?** — NATIVO · baja · MG23 · Sí (kit en unidades, componente en L/Kg); la UoM del componente debe ser de la misma categoría que su UoM de stock.

**KITCAP21. ¿Puedo tener un kit por variante concreta?** — NATIVO · media · `mrp.bom.product_id` (variante) vs `product_tmpl_id` (todas) · Sí; BoM phantom ligada a una variante (MRPCAP3, VAG17).

**KITCAP22. ¿Puedo condicionar componentes del kit por atributo/variante?** — NATIVO · media · `bom_line.bom_product_template_attribute_value_ids` + `_skip_bom_line` · Una línea se salta si la variante no coincide (`_skip_bom_line`, usado en `explode`).

**KITCAP23. ¿Cuántos componentes puede tener un kit?** — NATIVO (sin límite) · baja · `bom_line_ids` (O2M) · Sin límite de modelo.

**KITCAP24. ¿Un kit puede tener subproductos (by-products)?** — NATIVO (modelo) / raro · media · `byproduct_ids` · El campo existe, pero by-products en phantom es poco habitual; el uso típico es en MO reales.

### II. Reabastecimiento / MTO / MPS con kits

**KITCAP25. ¿Se abastecen automáticamente los componentes de un kit vendido?** — NATIVO · media · procurement por componente · Al explotar, cada componente dispara sus reglas (MTO/buy/manufacture) según su ruta.

**KITCAP26. ¿Puedo poner una regla de reabastecimiento (orderpoint) sobre el kit?** — NO de forma libre (BLOQUEADO por constraint activo) · media · `mrp.bom.check_kit_has_not_orderpoint` (`mrp/models/mrp_bom.py:339`, `MrpBom.check_kit_has_not_orderpoint` v18 / `:353` v19) · Es un `@api.constrains('product_tmpl_id','product_id','type')` ACTIVO que el ORM dispara en create/write del BoM (NO es código muerto: 0 callers explícitos, pero lo invoca el framework vía el decorador): crear/modificar un BoM phantom para un producto que ya tiene un orderpoint lanza `ValidationError` ('You can not create a kit-type bill of materials for products that have at least one reordering rule.'). Paliativo: el override `stock.warehouse.orderpoint._quantity_in_progress` (`mrp/stock_orderpoint.py:78, `StockWarehouseOrderpoint._quantity_in_progress`+`) explota el kit a componentes para el cálculo de cantidad en curso cuando el orderpoint ya existe. Verificado v18+v19 2026-07-15 (rectifica la versión previa que lo daba por código muerto). (Distinto del MPS, que sí omite kits: KITCAP27/MG15.)

**KITCAP27. ¿El MPS planifica kits?** — NO · media · MG15 · El MPS **omite** los phantom para evitar doble abastecimiento; planificar los componentes.

**KITCAP28. ¿Un kit puede ser MTO (fabricar/comprar bajo pedido)?** — NATIVO (vía componentes) · media · rutas por componente · El kit no se "fabrica"; cada componente hereda su ruta MTO.

### JJ. POS, eCommerce, otros canales

**KITCAP29. ¿Los kits funcionan en POS?** — CONFIG (limitado) · alta · `pos_mrp` · POS soporta phantom en descarga de stock, pero el ticket muestra el kit; sin desglose de componentes nativo en pantalla. Ver [[pos]].

**KITCAP30. ¿Los kits funcionan en eCommerce?** — NATIVO · media · website_sale + sale_mrp · Sí; se vende el producto kit, entrega explota componentes.

**KITCAP31. ¿Diferencia kit vs "producto combo" (POS/v18)?** — NATIVO (distintos) · media · `product.combo` vs `mrp.bom phantom` · El **combo** (nuevo v18, POS/venta) deja elegir 1 de N opciones a precio combo; el **kit** siempre entrega TODOS los componentes fijos. No confundir. Ver `product/core.md`.

**KITCAP32. ¿Diferencia kit vs producto con opciones/accesorios?** — NATIVO (distintos) · baja · `optional_product_ids` vs phantom · Opcionales/accesorios = sugerencias añadibles (líneas propias); kit = descomposición fija de 1 producto.

### KK. Limitaciones y avisos

**KITCAP33. ¿Puedo cambiar las líneas de un kit ya usado en movimientos?** — CONDICIONADO · media · `_onchange...` (`mrp/models/mrp_bom.py:204-207`, `class MrpBom`) · Odoo avisa si ya hay `stock.move` con `bom_line_id` de este kit (cambios afectan a explosiones futuras, no a las hechas).

**KITCAP34. ¿Puedo borrar un kit con MOs en curso?** — NO · baja · `_unlink_except_running_mo` (`mrp/models/mrp_bom.py:346`, `MrpBom._unlink_except_running_mo`) · Bloqueado si hay producciones activas.

**KITCAP35. ¿El kit soporta lotes/números de serie?** — NATIVO (en componentes) · media · move lines de componentes · La trazabilidad es por componente, no por kit.

**KITCAP36. ¿Devoluciones (RMA) de un kit?** — NATIVO (componentes) · alta · return de componentes · Se devuelven los componentes entregados; no hay "devolución de kit" como unidad. Ver [[playbook RMA]].

**KITCAP37. ¿El coste del kit se ve en la ficha del producto kit?** — NO (fiable) · media · `standard_price` del kit ≠ suma componentes · El `standard_price` del producto kit no se mantiene como suma de componentes automáticamente (mrp_account calcula en movimientos, no en la ficha).

### LL. Diferencias v18 → v19 (kits / BoM)

**KITCAP38. ¿Sigue existiendo `type='phantom'` en v19?** — SÍ · baja · `mrp/models/mrp_bom.py:28-31`, `class MrpBom` v19 · Sin cambios en el tipo kit.

**KITCAP39. ¿Qué se añadió en v19 en `mrp.bom`?** — SÍ · baja · `batch_size` + `enable_batch_size` (`mrp/models/mrp_bom.py:96-97`, `class MrpBom` v19) + constraint `_check_valid_batch_size` (`:359`) · Nuevo tamaño de lote para MOs autogeneradas (afecta manufacturing, no la explosión del kit).

**KITCAP40. ¿Cambió la definición de constraints en v19?** — SÍ · baja · `models.Constraint` · `qty_positive`→`_qty_positive` y `_bom_qty_zero` pasan a `models.Constraint` (`mrp/models/mrp_bom.py:99`, `MrpBom._compute_possible_product_template_attribute_value_ids` y `mrp/models/mrp_bom.py:728`, `MrpBomLine._skip_bom_line` v19). Mismo CHECK. Nuevas acciones `action_archive`/`action_unarchive` sustituyen `toggle_active`.

**KITCAP41. ¿`explode`, `_bom_find`, `_skip_bom_line`, `cost_share` cambian en v19?** — NO en comportamiento · baja · diff · `_bom_find`, `_skip_bom_line` y el field `cost_share` son idénticos. Matiz: `explode` NO es solo line-shift — trae un refactor equivalente (`float_round(...precision_rounding=rounding...)` → `product_uom_id.round(...)`) más un guard `if not self: return`, y el hermano `_bom_find_domain` migró al API `Domain`; ninguno altera el resultado de la explosión. Verificado 2026-07-15.

---

## UoM en fabricación (cross-ref `product/capabilities.md` UOMCAP)

**MRPCAP19. ¿Puedo producir en una UoM distinta a la de stock (ej. BoM produce 1 docena)?** — NATIVO · baja · `mrp.bom.product_uom_id` + `product_qty` (`mrp/models/mrp_bom.py:42-48`, `class MrpBom`) · Sí; la BoM define "produce N en esta UoM" y se convierte a la stock UoM al fabricar. "12 uds/lote" y "1 docena/lote" son equivalentes en cálculo pero distintos en UI. En v19 el motor usa los métodos nuevos `product_uom_id.round()`/`.compare()`/`.is_zero()` (KITCAP41).

**MRPCAP20. ¿Cada línea de componente puede ir en su propia UoM?** — NATIVO · baja · `mrp.bom.line.product_uom_id` (required, `mrp/models/mrp_bom.py:46`, `class MrpBom`) · Sí; cada componente se consume en la UoM de la línea, convertible con la UoM del componente. En v19 debe ser una UoM del mismo árbol (raíz común); consumir en UoM de otro árbol daría conversión errónea sin error (ver UOG29).

**MRPCAP21. ¿Fabricaciones parciales con decimales cuadran los componentes?** — RIESGO (redondeo) · media · `unit_factor` scaling (`mrp/models/stock_move.py`) · Con cantidades fraccionarias + conversión de UoM, el redondeo del `should_consume_qty` acumula error; la última entrega/backorder puede salir ligeramente descuadrada. Ver MG6.
