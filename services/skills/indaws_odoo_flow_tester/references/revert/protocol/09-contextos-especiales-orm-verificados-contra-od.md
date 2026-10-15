## Contextos Especiales ORM (verificados contra Odoo 18.0)

| Contexto | Modelo | Efecto | Referencia |
|----------|--------|--------|------------|
| `force_delete=True` | account.move | Bypass de `_unlink_forbid_parts_of_chain` y `_unlink_account_audit_trail_except_once_post` | account_move.py L3500, L3510 |
| `disable_cancel_warning=True` | sale.order | Salta el wizard de cancelacion | sale_order.py L1322 |
| `tracking_disable=True` | Todos | Evita tracking en chatter | ORM base |
| `mail_create_nolog=True` | Todos | Evita log de mail | mail module |
| `mail_notrack=True` | Todos | Evita tracking de cambios | mail module |
| `skip_invoice_sync=True` | account.move | Evita sync con facturas | account_move.py L3521 |
| `dynamic_unlink=True` | account.move | Permite borrado dinamico | account_move.py L3521 |
| `leave_skip_date_check=True` | hr.leave | Salta validacion de fechas en unlink | hr_leave.py L853 |
| `skip_activity=True` | mrp.production | Salta log de actividad en cancel | mrp_production.py L1760 |
| `prefetch_fields=False` | Varios | Evita prefetch de campos en operaciones masivas de borrado | Optimizacion de rendimiento |

---

