## Deteccion de Cascadas (verificado contra Odoo 18.0)

Muchos registros de Odoo se crean automaticamente cuando se ejecutan acciones sobre otros registros.
Estos registros "cascade" deben detectarse y revertirse ANTES que los registros padres.

### Mapa de creacion automatica

| Registro Padre | Accion | Registros Cascade Creados |
|----------------|--------|---------------------------|
| `sale.order` | `action_confirm` | `stock.picking`, `stock.move`, `stock.move.line`, `procurement.group` |
| `sale.order` | `_create_invoices` | `account.move`, `account.move.line` |
| `purchase.order` | `button_confirm` | `stock.picking`, `stock.move`, `stock.move.line` |
| `purchase.order` | `action_create_invoice` | `account.move`, `account.move.line` |
| `account.move` | `action_post` | `account.move.line` (reconciliacion), `mail.message`, hash si journal.restrict_mode_hash_table |
| `account.payment` | `action_post` | `account.move`, `account.move.line`, `account.partial.reconcile` (si reconcilia factura) |
| `stock.picking` | `button_validate` | `stock.move.line` (detalle), `stock.valuation.layer` (si valoracion automatica), `account.move` (asiento contable valoracion) |
| `mrp.production` | `button_mark_done` | `stock.move.line`, `stock.valuation.layer`, `mrp.workorder` (si routing) |
| `hr.expense.sheet` | `action_sheet_move_create` | `account.move`, `account.move.line` |
| `hr.payslip` | `action_payslip_done` | `account.move`, `account.move.line` |
| `pos.order` | `action_pos_order_paid` | `pos.payment`, `account.move` (si factura), `stock.picking` |
| `sale.order` (suscripcion) | `_create_recurring_invoice` | `account.move`, `account.move.line` |
| `account.asset` | `_generate_moves` (cron) | `account.move` (asientos de depreciacion) |
| `mailing.mailing` | `action_send_mail` | `mailing.trace`, `mail.mail` |

### Diagrama de Dependencias (resumen visual — verificado)

```
                    account.partial.reconcile
                           │
                           ▼
    account.payment ──► account.move.line ──► account.move
                                                   │
                    stock.valuation.layer ──────────┘
                           │
                           ▼
    stock.move.line ──► stock.move ──► stock.picking
                                          │
                    stock.scrap ───────────┘
                    stock.package_level ───┘

    mrp.workcenter.productivity ──► mrp.workorder ──► mrp.production
    mrp.unbuild ────────────────────────────────────►

    pos.payment ──► pos.order ──► (account.move + stock.picking)
    pos.order.line ─┘

    sale.order.line ──► sale.order     (cascade)
    purchase.order.line ──► purchase.order  (cascade)

    hr.expense ──► hr.expense.sheet ──► account.move
    hr.leave (independiente, pero afecta resource.calendar.leaves)
```

### Deteccion especifica de cascadas criticas

```bash
# --- stock.move.line (critico: vinculado a stock.quant via triggers) ---
# Detectar move lines de un picking
search_records.sh stock.move.line '[["picking_id","=",<picking_id>]]' '["id","product_id","qty_done","lot_id","location_id","location_dest_id"]' 0

# Detectar move lines de un stock.move
search_records.sh stock.move.line '[["move_id","=",<move_id>]]' '["id","product_id","qty_done"]' 0

# --- account.partial.reconcile (critico: bloquea cancelacion) ---
# Detectar reconciliaciones de un move
search_records.sh account.partial.reconcile '["|",["debit_move_id.move_id","=",<move_id>],["credit_move_id.move_id","=",<move_id>]]' '["id","debit_move_id","credit_move_id","amount","full_reconcile_id"]' 0

# --- account.full.reconcile (cascada de partial) ---
# Detectar fulls huerfanos (sin partials)
# NOTA: Los fulls se borran automaticamente al borrar sus partials. No borrar manualmente.
search_records.sh account.full.reconcile '[["partial_reconcile_ids","=",false]]' '["id"]' 0

# --- stock.valuation.layer (critico: afecta valoracion de inventario) ---
# Detectar capas de valoracion de un stock.move
search_records.sh stock.valuation.layer '[["stock_move_id","=",<move_id>]]' '["id","value","quantity","remaining_qty","account_move_id"]' 0

# Detectar capas vinculadas a un account.move
search_records.sh stock.valuation.layer '[["account_move_id","=",<move_id>]]' '["id","value","stock_move_id"]' 0

# --- Deteccion general ---
# Detectar pickings creados por un SO
search_records.sh stock.picking '[["origin","=","<so_name>"]]' '["id","name","state"]' 0

# Detectar facturas creadas por un SO
search_records.sh account.move '[["invoice_origin","=","<so_name>"]]' '["id","name","state"]' 0

# Detectar asientos creados por un pago
search_records.sh account.move '[["payment_id","=",<payment_id>]]' '["id","name","state"]' 0

# Detectar asientos de depreciacion de un asset
search_records.sh account.move '[["asset_id","=",<asset_id>]]' '["id","name","state"]' 0

# Detectar trazas de un mailing
search_records.sh mailing.trace '[["mass_mailing_id","=",<mailing_id>]]' '["id"]' 0
```

### Orden de reversion de cascade

**Regla: Siempre revertir registros hoja primero, luego subir hacia el padre.**

```
Ejemplo para cadena SO → Picking → Invoice → Payment:

1. Revertir payment (action_cancel → action_draft → delete)
   → Sus account.partial.reconcile se borran al llamar action_draft (que llama button_draft → remove_move_reconcile)
   → account.full.reconcile huerfanos se borran automaticamente
   → Exchange diffs se revierten automaticamente
2. Revertir invoice (button_draft → button_cancel → delete)
   → Sus account.move.line se borran en cascada
3. Revertir picking done (crear devolucion → validar devolucion)
   → stock.move.line de devolucion se crea automaticamente
   → stock.valuation.layer de devolucion se crea automaticamente
   → stock.quant se actualiza automaticamente (triggers internos)
4. Cancelar y borrar SO (action_cancel → delete)
   → sale.order.line se borra en cascada (FK CASCADE)
   → stock.move sin picking se puede quedar huerfano — verificar
```

---

