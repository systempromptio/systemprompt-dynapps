## Grafo de Dependencias FK (verificado contra Odoo 18.0)

### Relaciones con CASCADE (borrar padre borra hijos automaticamente en PostgreSQL)

| Hijo (tabla) | Campo FK | Padre (tabla) | Archivo fuente | Linea |
|--------------|----------|---------------|----------------|-------|
| `sale_order_line` | `order_id` | `sale_order` | sale/models/sale_order_line.py | L41 |
| `sale_order_line` | `linked_line_id` | `sale_order_line` (auto-ref) | sale/models/sale_order_line.py | L140 |
| `purchase_order_line` | `order_id` | `purchase_order` | purchase/models/purchase_order_line.py | L47 |
| `account_move_line` | `move_id` | `account_move` | account/models/account_move_line.py | L40 |
| `pos_order_line` | `order_id` | `pos_order` | point_of_sale/models/pos_order.py | L1401 |
| `pos_payment` | `pos_order_id` | `pos_order` | point_of_sale/models/pos_payment.py | L20 |
| `product_packaging` (sale) | `sale_order_line_id` | `sale_order_line` | sale/models/product_product.py | L105 |
| `product_packaging` (pos) | `pos_order_line_id` | `pos_order_line` | point_of_sale/models/product.py | L285 |
| `mrp_bom_line` | `bom_id` | `mrp_bom` | mrp/models/mrp_bom.py | L619 |
| `mrp_bom_byproduct` | `bom_id` | `mrp_bom` | mrp/models/mrp_bom.py | L797 |
| `mrp_routing_workcenter` | `bom_id` | `mrp_bom` | mrp/models/mrp_routing.py | L25 |
| `stock_move_line` | `product_id` | `product_product` | stock/models/stock_move_line.py | L28 |

### Relaciones SIN CASCADE (ondelete='set null' o 'restrict' o default)

| Hijo (tabla) | Campo FK | Padre (tabla) | ondelete | Notas |
|--------------|----------|---------------|----------|-------|
| `stock_move` | `picking_id` | `stock_picking` | set null (default) | NO cascade — borrar picking NO borra moves automaticamente |
| `stock_move_line` | `move_id` | `stock_move` | set null (default) | NO cascade — hay que borrar manualmente |
| `stock_move_line` | `picking_id` | `stock_picking` | set null (default) | NO cascade |
| `stock_valuation_layer` | `stock_move_id` | `stock_move` | set null (default) | NO cascade |
| `stock_valuation_layer` | `account_move_id` | `account_move` | set null (default) | NO cascade |
| `stock_valuation_layer` | `stock_valuation_layer_id` | `stock_valuation_layer` (auto-ref) | set null (default) | Auto-referencia |
| `account_payment` | `move_id` | `account_move` | set null (default) | NO cascade — payment.unlink() borra su move explicitamente |
| `hr_expense` | `sheet_id` | `hr_expense_sheet` | set null (default) | NO cascade |
| `mrp_workorder` | `production_id` | `mrp_production` | restrict (default) | NO cascade — production.unlink() borra workorders explicitamente |

---

