# Rules — NUNCA / SIEMPRE

## NUNCA

1. Ejecutar flujos sin aprobación explícita del consultor (Paso 1.5 STOP)
2. Crear datos maestros (partners, productos) durante ejecución — solo transaccionales
3. Ejecutar en production sin pre-state snapshot
4. Ejecutar reversión en production sin dry-run aprobado
5. Ejecutar Modo C sin verificar la Safety Checklist completa
6. Cancelar `stock.picking` en estado `done` — usar wizard `stock.return.picking`
7. `button_draft()` en `account.move` con `inalterable_hash` — escalar a inversión o SQL
8. Operar sobre modelo sin verificar `ir.model.access` del usuario API
9. Asumir que operación funcionará sin comprobar `ir.rule` activas
10. Usar `type='product'` en Odoo **18.0+** — usar `is_storable=True` (en 17.0 `type='product'` es válido y `is_storable` no existe)
11. Usar `action_sheet_move_create()` en hr.expense.sheet (no existe en 18.0) ni `action_draft()` en hr.leave. **En 19.0 no usar `hr.expense.sheet` en absoluto: el modelo desapareció**, todo va contra `hr.expense`
12. Referir `fleet.vehicle.log.fuel` — no existe en Odoo 18.0
13. Usar `pause()` en `account.asset` via RPC — usar `write({'state': 'paused'})`
14. Usar `recurrence_id` para suscripciones en 18.0 — usar `plan_id`
15. Usar `action_draft()` en `mailing.mailing` — `action_cancel` ya vuelve a draft
16. Asumir que `survey.survey` tiene campo `state` — usar `active` + `action_archive/action_unarchive`
17. Eliminar `hr.work.entry` validados via ORM — archivar contrato/empleado
18. Crear `hr.contract` con `state='open'` sin planificar cleanup de work entries
19. Escribir `date_from/date_to` en `hr.leave` — usar `request_date_from/request_date_to`. Son computed desde v17 (no es un cambio de 19.0): aplica a **todas** las versiones
20. Escribir campo `state` en `hr.expense` en Odoo 19.0 — es computed, usar `approval_state`
21. Asumir que `account.move.action_post()` devuelve True en Odoo 19.0 — incluir `disable_abnormal_invoice_detection: True`
22. Cancelar/modificar `stock.picking` con `is_locked=True` en Odoo 19.0 sin `action_toggle_is_locked()`

## SIEMPRE

1. Pedir documento de análisis antes de empezar (Paso 0)
2. Usar prefijo `[TEST-YYYYMMDD-HHMM]` en todos los registros de test
3. Activar safe mode antes de ejecutar en production
4. Restaurar safe mode después de ejecutar, incluso si el flujo falló
5. Verificar permisos `ir.model.access` antes de ejecutar flujos (Paso 2)
6. Comprobar record rules (`ir.rule`) activas
7. Añadir timeout por paso (default 30s) para prevenir bloqueos
8. Verificar versión de Odoo y adaptar métodos API (ver `api_compatibility.md`)
9. Usar `is_storable=True` para productos almacenables en Odoo 18.0+ (en 17.0, `type='product'`)
10. Verificar módulos EDI fiscales antes de flujos contables

## OTRAS

- Si algo ambiguo → PREGUNTAR al usuario
- Adaptar flujos a GAPs del cliente
- Si paso falla → PARAR y reportar
- Screenshots → delegar a `odoo-screenshot-generator`
