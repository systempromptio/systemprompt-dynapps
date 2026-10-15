# Correcciones Críticas v2.0 / v2.1 / v2.2

Correcciones descubiertas al validar flujos contra código fuente Odoo 18.0 (v2.0) y tests en vivo (v2.1):

| Área | Error anterior | Corrección | Referencia |
|------|---------------|------------|------------|
| `hr.expense.sheet` | `action_sheet_move_create()` | `action_sheet_move_post()` — create NO EXISTE en 18.0 | `api_compatibility.md` §hr.expense |
| `hr.leave` | `action_draft()`, estado inicial `draft` | `action_reset_confirm()` — estado inicial es `confirm` | `api_compatibility.md` §hr.leave |
| `account.move` | Hash fiscal bloquea `unlink()` | Hash solo bloquea `button_draft()`. Solo audit trail bloquea `unlink()` | `revert/protocol/` |
| `mrp.production` | Solo draft/cancel permiten `unlink()` | `unlink()` auto-cancela en 18.0. Solo `done` irremovible | `api_compatibility.md` |
| `project.task` | 1 solo estado (stage-based) | 6 estados via campo `state` (computed) | `api_compatibility.md` |
| Productos storable | `type='product'` | `is_storable=True` **en 18.0+** — en 17.0 `type='product'` sigue siendo válido (`stock` lo añade por `selection_add`) y `is_storable` no existe | `api_compatibility.md` |
| `fleet.vehicle.log.fuel` | Incluido como modelo | NO EXISTE en 18.0 — usar `fleet.vehicle.log.services` | `flows/extended/` |
| `account.asset` | `pause(pause_date)` via RPC | `pause()` NO funciona via RPC — usar `write({'state': 'paused'})` | `flows/extended/` |
| Suscripciones | `recurrence_id` | NO EXISTE en 18.0 — usar `plan_id` (`sale.subscription.plan`) | `flows/extended/` |
| `mailing.mailing` | `action_draft` | NO EXISTE — `action_cancel` ya resetea a `draft` | `flows/extended/` |
| `survey.survey` | Campo `state` | NO tiene `state` — usar `active` + `action_archive/unarchive` | `flows/extended/` |
| `purchase.requisition` | `action_in_progress` | Método correcto: `action_confirm` | `flows/extended/` |
| `stock.landed.cost` | Sin `split_method` | Campo requerido en líneas (ej: `'equal'`, `'by_quantity'`) | `flows/extended/` |
| `hr.work.entry` | Eliminación libre | Validados NO eliminables via ORM — archivar contrato | `flows/extended/` |
| `hr.contract` | Eliminación sin restricciones | `state='open'` auto-genera work entries | `flows/extended/` |
| `flows/chains/` | `action_sheet_move_create` en 6 lugares | Corregido a `action_sheet_move_post()` | `api_compatibility.md` |
| Cadenas | "(28)" en SKILL.md | Actualizado a (39) con 11 cadenas nuevas | `flows/chains/` |
| `api_compatibility.md` | Faltaba sección 19.0 | Añadida sección 18.0→19.0 preliminar | `api_compatibility.md` |
