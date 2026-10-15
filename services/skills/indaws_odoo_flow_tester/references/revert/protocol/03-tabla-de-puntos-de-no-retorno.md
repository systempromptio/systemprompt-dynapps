## Tabla de Puntos de No Retorno

Referencia rapida de estados irreversibles por modelo, verificada contra el codigo fuente de Odoo 18.0.

| Modelo | Estado/Condicion | Por que es irreversible | Referencia fuente | Alternativa |
|--------|-----------------|------------------------|-------------------|-------------|
| `account.move` | `inalterable_hash IS NOT NULL` | Hash SHA-256 encadenado protege integridad fiscal. `_hash_moves()` (L4046-4058) se ejecuta al publicar si journal.restrict_mode_hash_table=True. Encadena hashes: cada asiento usa el hash del anterior. `show_reset_to_draft_button = False` (L1786-1793). **Solo bloquea button_draft, NO bloquea unlink** — unlink sigue disponible si el move esta en draft. | account_move.py L325, L1791, L4007-4012 | Crear asiento inverso via `_reverse_moves()` |
| `account.move` | `posted_before=True` + `company.check_account_audit_trail=True` | Audit trail impide borrado via ORM. `_unlink_account_audit_trail_except_once_post()` (L3508-3517) lanza UserError. `_unlink_or_reverse()` solo cancela (no borra). **Bypass: `context={'force_delete': True}`** | account_move.py L3508-3517 | Solo cancelar (`button_cancel`), no borrar. O usar `force_delete` ctx |
| `account.move` | Exchange diff entry | `_check_draftable()` lanza UserError. Es un asiento de diferencia de cambio generado por reconciliacion (via `account_full_reconcile.exchange_move_id` y `account_partial_reconcile.exchange_move_id`). | account_move.py L5569 | Se revierte automaticamente al desreconciliar |
| `account.move` | Cash basis entry | `_check_draftable()` lanza UserError por `tax_cash_basis_rec_id` o `tax_cash_basis_origin_move_id`. | account_move.py L5569 | Se revierte automaticamente al desreconciliar |
| `account.move` | `date <= fiscal_lock_date` | Periodo cerrado. `_check_fiscal_lock_dates()` (L2503-2518) bloquea edicion/borrado. Hard lock no bypasseable ni por administrador. | account_move.py L2503-2518 | Pedir al contable que abra periodo temporal |
| `account.move` | `need_cancel_request = True` | Hook de localizacion e-factura (L708, L1711-1723). Si devuelve True, `button_draft()` lanza UserError (L5528-5529). Se necesita solicitar cancelacion al gobierno (ej: CFDI Mexico, SII Espana). | account_move.py L708, L5528-5529 | Solicitar cancelacion EDI al gobierno |
| `stock.picking` | `state=done` | `stock.move._action_cancel()` (L1997-1999) lanza `UserError` si `move.state == 'done' and not move.scrapped`. `picking.unlink()` (L1152) llama `move._action_cancel()` que falla. **stock.picking NO tiene @api.ondelete** — no valida estado directamente. | stock_move.py L1997-1999, stock_picking.py L1152-1155 | **Crear devolucion** (return picking) - Tier 1 preferido |
| `stock.move` | `state=done` (no scrapped) | `_action_cancel()` (L1997-1999) verifica `move.state == 'done' and not move.scrapped` y lanza UserError. `_unlink_if_draft_or_cancel()` (L2130-2133) impide borrado si tiene `move_orig_ids` o `move_dest_ids` en estado no-draft/cancel. | stock_move.py L1997-1999, L2130-2133 | Devolucion via `stock.return.picking` |
| `mrp.production` | `state=done` | No se puede cancelar (`action_cancel()` L1743-1744: ERROR si done) NI borrar (`_unlink_if_not_done()` L1062-1065 y `_unlink_except_done()` L1365-1370: ERROR si done). Doble check en ondelete. | mrp_production.py L1062-1065, L1365-1370, L1743-1744 | Crear orden de desmontaje (`mrp.unbuild`) |
| `account.payment` | Reconciliado con factura | Partials bloquean cancelacion directa. | account_payment.py L1115 | `action_draft()` (L1124) desreconcilia automaticamente via `move.button_draft()` que llama `line_ids.remove_move_reconcile()` |
| `pos.order` | Sesion cerrada con `move_id` posted | No tiene `action_cancel` estandar. `@api.ondelete` (L501-504) solo permite borrar en draft/cancel. | pos_order.py L501-504 | Crear refund o sesion rescue |
| `hr.payslip` | `move_id` posted con hash | Asiento contable protegido por hash. | — | Cancelar/revertir move primero |
| `mailing.mailing` | `state=done` | Emails ya entregados a destinatarios. | — | Solo borrar registro; emails irrecuperables |
| `account.asset` | Asientos depreciacion posted | Asientos de amortizacion ya publicados. | — | Revertir asientos primero, luego `set_to_draft` |
| `stock.scrap` | `state=done` | Move de desecho en state=done. | — | Crear unbuild o devolucion manual |
| `sale.order` (suscripcion) | Facturas con hash fiscal | Periodos facturados con hash. | — | Neutralizar suscripcion |
| Factura enviada EDI (SII/TicketBAI) | `need_cancel_request = True` | Dato fiscal ya comunicado al gobierno. | account_move.py L708, L1711-1723 | Solo anular con factura rectificativa |

### Hash fiscal: Localizaciones afectadas

En Odoo 18.0, el hash se activa **por diario** (`account.journal.restrict_mode_hash_table`). Cuando esta activado, al publicar un asiento (`action_post`) se ejecuta `_hash_moves()` que calcula SHA-256 encadenado.

| Localizacion | Comportamiento hash | Notas |
|-------------|-------------------|-------|
| **Cualquier pais** (base) | Activacion manual por diario (checkbox "Secure Posted Entries with Hash") | El usuario decide. Una vez activado y con asientos hasheados, no se puede desactivar. |
| **Espana (l10n_es)** con VeriFactu OCA | `restrict_mode_hash_table` se fuerza a True y readonly en diarios de venta/compra si VeriFactu esta activo | Modulo `l10n_es_verifactu_oca`. Campo `restrict_mode_hash_table_readonly` impide desactivar. |
| **Mexico (l10n_mx)** | Hace visible el campo hash en todos los diarios (sin `groups` restriction) | No fuerza pero facilita activacion. |
| **Uruguay (l10n_uy_edi)** | Referencia hash en vistas de diarios | Facturacion electronica. |
| **Alemania (l10n_de)** | Activa `check_account_audit_trail=True` por defecto | No hash directo, pero audit trail impide borrado de moves con `posted_before=True`. |
| **Dinamarca (l10n_dk)** | Activa `check_account_audit_trail=True` por defecto | Idem Alemania. |

**Implicacion para reversion:** Si el diario tiene hash activo, `button_draft()` siempre fallara para asientos hasheados. La unica opcion ORM es `_reverse_moves()`. Si se necesita borrar fisicamente, se requiere SQL directo (Tier 3).

---

