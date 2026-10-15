## Reconciliacion: Flujo de Desreconciliacion Correcto

La reconciliacion en Odoo 18.0 involucra 3 modelos:

| Modelo | Tabla SQL | Descripcion |
|--------|----------|-------------|
| `account.partial.reconcile` | `account_partial_reconcile` | Emparejamiento parcial entre debit_move_id y credit_move_id |
| `account.full.reconcile` | `account_full_reconcile` | Agrupa partials cuando las lineas estan totalmente reconciliadas |
| `account.move.line` | `account_move_line` | Tiene campos `matched_debit_ids`, `matched_credit_ids`, `full_reconcile_id` |

### Secuencia correcta de desreconciliacion

```
1. Borrar account.partial.reconcile
   → Su unlink() automaticamente:
     a) Identifica full_reconcile_id asociados
     b) Borra el super() (el partial)
     c) Borra full_reconcile huerfanos (que a su vez revierten exchange moves)
     d) Revierte CABA entries (tax_cash_basis_rec_id)
     e) Revierte exchange diff entries (exchange_move_id)
     f) Actualiza payment states (to_update_payments)

2. O mas seguro via ORM:
   line_ids.remove_move_reconcile()
   → Hace (matched_debit_ids + matched_credit_ids).unlink()
   → Que dispara todo lo anterior
```

### Secuencia para revertir factura reconciliada con pago

```
1. Desreconciliar (Opcion A: via payment)
   execute_method.sh account.payment action_draft '[[<payment_id>]]'
   → Llama move.button_draft() → line_ids.remove_move_reconcile()
   → Borra partials, fulls, exchange diffs, CABA entries automaticamente

2. Cancelar payment
   execute_method.sh account.payment action_cancel '[[<payment_id>]]'

3. Borrar payment
   delete_record.sh account.payment '[<payment_id>]'

4. Ahora la factura esta libre de reconciliacion
   execute_method.sh account.move button_draft '[[<invoice_id>]]'
   execute_method.sh account.move button_cancel '[[<invoice_id>]]'
   delete_record.sh account.move '[<invoice_id>]'
```

```
Alternativa (Opcion B: via partial directo)
1. Buscar partials
   search_records.sh account.partial.reconcile '["|",["debit_move_id.move_id","=",<move_id>],["credit_move_id.move_id","=",<move_id>]]' '["id"]' 0

2. Borrar partials (dispara cascada de full_reconcile, exchange, CABA)
   delete_record.sh account.partial.reconcile '[<partial_ids>]'

3. Ahora cancelar pago y factura
```

### ADVERTENCIA sobre account.full.reconcile.unlink()

En Odoo 18.0, `account.full.reconcile.unlink()` revierte automaticamente `exchange_move_id` (crea un asiento inverso). Si se borra un full_reconcile manualmente sin borrar sus partials primero, se pueden crear asientos inversos inesperados.

**Regla:** Siempre borrar partials (que disparan borrado de fulls). NUNCA borrar fulls directamente.

---

