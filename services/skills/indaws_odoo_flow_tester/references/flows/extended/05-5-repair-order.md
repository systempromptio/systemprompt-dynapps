## 5. repair.order

### Estados
```python
state = fields.Selection([
    ('draft', 'New'),
    ('confirmed', 'Confirmed'),
    ('under_repair', 'Under Repair'),
    ('done', 'Repaired'),
    ('cancel', 'Cancelled')
], default='draft')
```

### Campos Minimos para create()
| Campo | Tipo | Requerido | Default | Notas |
|-------|------|-----------|---------|-------|
| name | Char | Si | 'New' (auto-secuencia) | Referencia auto-generada |
| company_id | Many2one | Si | env.company | Compania |
| picking_type_id | Many2one (stock.picking.type) | Si | _default_picking_type_id | Tipo de operacion repair_operation |
| location_id | Many2one (stock.location) | Si | computed from picking_type_id | Ubicacion origen componentes |
| product_location_src_id | Many2one | Si | computed from picking_type_id | Ubicacion origen producto |
| product_location_dest_id | Many2one | Si | computed from picking_type_id | Ubicacion destino producto |
| location_dest_id | Many2one | Si | related picking_type_id | Ubicacion destino partes |
| parts_location_id | Many2one | Si | related picking_type_id | Destino partes removidas |
| recycle_location_id | Many2one | Si | computed from picking_type_id | Ubicacion reciclaje |
| schedule_date | Datetime | Si | Datetime.now | Fecha programada |

**En practica, basta con:** `{}` (todos los required tienen defaults o computes) o `{'product_id': <id>}` para reparar un producto especifico.

### Flujos Atomicos

#### REPAIR_CREATE
- **Modelo:** repair.order
- **Accion:** create
- **Campos minimos:** `{}` (todos los required tienen defaults)
- **Resultado:** state='draft', name auto-secuenciado
- **Verificacion:** id > 0, state == 'draft', name != 'New'

#### REPAIR_CONFIRM
- **Modelo:** repair.order
- **Accion:** action_validate() -> _action_repair_confirm()
- **Precondicion:** state == 'draft'
- **Resultado:** state='confirmed', moves confirmados
- **Verificacion:** state == 'confirmed'

#### REPAIR_START
- **Modelo:** repair.order
- **Accion:** action_repair_start()
- **Precondicion:** state == 'confirmed' (si no, auto-confirma)
- **Resultado:** state='under_repair'
- **Verificacion:** state == 'under_repair'

#### REPAIR_DONE
- **Modelo:** repair.order
- **Accion:** action_repair_end() -> action_repair_done()
- **Precondicion:** state == 'under_repair'
- **Resultado:** state='done', move_id creado, moves en state 'done'
- **Restriccion:** Si product_id con tracking != 'none', requiere lot_id
- **Verificacion:** state == 'done'

#### REPAIR_CANCEL
- **Modelo:** repair.order
- **Accion:** action_repair_cancel()
- **Precondicion:** state != 'done' (raise UserError si done)
- **Resultado:** state='cancel', moves cancelados
- **Verificacion:** state == 'cancel'

#### REPAIR_CANCEL_TO_DRAFT
- **Modelo:** repair.order
- **Accion:** action_repair_cancel_draft()
- **Precondicion:** cualquier estado (auto-cancela si no cancelado)
- **Resultado:** state='draft', moves en 'draft'
- **Verificacion:** state == 'draft'

#### REPAIR_DELETE
- **Modelo:** repair.order
- **Accion:** unlink
- **Restricciones:** `@api.ondelete(at_uninstall=False) _unlink_except_confirmed` - Si state no es 'draft' ni 'cancel', auto-cancela antes de eliminar.
- **Verificacion:** no existe en BD

### Registros Cascade
| Modelo hijo | Campo FK | ondelete |
|------------|----------|----------|
| stock.picking.type | code selection | cascade para 'repair_operation' |

---

