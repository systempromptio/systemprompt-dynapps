## Tabla Completa de Reversion por Modelo (verificada contra Odoo 18.0)

| Modelo | Estados | Cancelar | Volver a Draft | Eliminar (ORM) | Restricciones principales |
|--------|---------|----------|----------------|-----------------|---------------------------|
| **sale.order** | draft, sent, sale, cancel | `action_cancel()` (L1273) — wizard si no draft; `disable_cancel_warning` ctx. `_action_cancel()` (L1311) — cancela facturas draft | No tiene `button_draft` | Solo `draft`/`cancel` (L1004-1010) | Locked orders no se cancelan (L1281). Pickings done o facturas posted → revertir primero |
| **sale.order.line** | — | Via SO | — | Cascade con sale.order | `order_id` ondelete=cascade (L41) |
| **purchase.order** | draft, sent, to approve, purchase, done, cancel | `button_cancel()` (L553) — requiere facturas en draft/cancel | `button_draft()` (L534) — sin restricciones | Solo `cancel` (L295-299) | No se puede cancelar con facturas posted (L554-556) |
| **purchase.order.line** | — | Via PO | — | Cascade con purchase.order | `order_id` ondelete=cascade (L47) |
| **account.move** | draft, posted, cancel | `button_cancel()` (L5615) — posted→draft→cancel auto. Cancela payments | `button_draft()` (L5525) — solo posted/cancel. `_check_draftable()` (L5569). Elimina analiticas, desreconcilia | Solo draft. `force_delete` ctx bypasses chain y audit trail | **inalterable_hash** bloquea draft (L1791). **Lock dates**. **need_cancel_request** en e-factura. **Audit trail** impide borrar posted_before |
| **account.move.line** | — | Via account.move | — | `unlink()` (L1737) — verifica reconciliacion, lock dates | `move_id` ondelete=cascade (L40). Reconciliadas no se borran (L1315-1320) |
| **account.payment** | draft, posted, in_process, paid, canceled | `action_cancel()` (L1115) — borra moves draft, cancela moves posted | `action_draft()` (L1124) — llama move.button_draft() | `unlink()` (L938) — pasa move a draft, borra move, borra payment | Hereda restricciones de account.move |
| **stock.picking** | draft, confirmed, assigned, done, cancel | `action_cancel()` (L1198) — llama `move_ids._action_cancel()` | No tiene button_draft | `unlink()` (L1152) — cancela moves, borra moves, sin @api.ondelete | Moves done no se cancelan (stock.move L1998) |
| **stock.move** | draft, confirmed, partially_available, assigned, done, cancel | `_action_cancel()` (L1997) — **ERROR si done y no scrapped** | No tiene button_draft | `unlink()` (L2135) — borra move_lines. ondelete (L2130-2133): solo draft/cancel si tiene links | **Done + no scrapped = irreversible** (L1998). Crear devolucion |
| **stock.move.line** | — | Via stock.move | — | `unlink()` (L567) — libera reservas en stock.quant, limpia package_levels | Modifica stock.quant al borrar |
| **stock.valuation.layer** | — | — | — | ORM estandar | Auto-referencia: `stock_valuation_layer_id` (L34). FK a stock.move y account.move |
| **mrp.production** | draft, confirmed, progress, to_close, done, cancel | `action_cancel()` (L1740) — **ERROR si done** (L1743-1744) | No tiene button_draft | `unlink()` (L1055) — action_cancel() primero. **ERROR si done** (L1062-1065, L1365-1370) | **Done = irreversible**. Alternativa: `mrp.unbuild` |
| **mrp.workorder** | pending, waiting, ready, progress, done, cancel | Via mrp.production | — | Solo si no done (via production.unlink) | `production_id` FK restrict a mrp.production |
| **pos.order** | draft, paid, done, invoiced, cancel | No tiene action_cancel estandar | No tiene button_draft | Solo `draft`/`cancel` (L501-504) | FK a stock.picking y account.move. Requiere sesion POS |
| **pos.order.line** | — | Via pos.order | — | Solo si order en draft/cancel (L1536-1539). `unlink()` (L1756) — tracking | `order_id` ondelete=cascade (L1401) |
| **hr.expense** | draft, submitted, approved, done | — | — | `unlink()` (L693). ERROR si done/approved (L609-613) | `sheet_id` FK sin cascade (set null, L100-107) |
| **hr.expense.sheet** | draft, submit, approve, post, done | `action_reset_expense_sheets()` (L614) — revierte moves | Via action_reset | ERROR si post/done (L525-529) | Requiere revertir account.move antes |
| **hr.leave** | confirm, validate1, validate, refuse, cancel | `action_refuse()` (L1146) — solo confirm/validate/validate1 | `action_reset_confirm()` (L1005) — solo cancel/refuse | (L835-849): usuarios normales con restricciones. HR managers solo cancel/confirm | Validadas en el pasado = problematicas |

---

