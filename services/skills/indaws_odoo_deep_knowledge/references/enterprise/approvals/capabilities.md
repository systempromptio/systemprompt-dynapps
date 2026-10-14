<!-- verified: v18.0 @ 2026-06 (approvals EE + bridge approvals_purchase; routing por umbral negativo; anclas verificadas vs fuente) -->
# Approvals (`approvals`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (APPG*); cambios v19 en [`v19_changes.md`](v19_changes.md).

**APPRCAP1. ¿Tipos de aprobación configurables?** — **NATIVO** · baja · modelo `approval.category` (`approval_category.py`). Registros creados por el usuario con sus reglas, campos y aprobadores.

**APPRCAP2. ¿Mínimo de aprobaciones + múltiples aprobadores?** — **NATIVO** · baja · `approval.category.approval_minimum` + `approver_ids` (O2M a `approval.category.approver`). Fija cuántas aprobaciones hacen falta y admite varios aprobadores.

**APPRCAP3. ¿Aprobadores por usuario o por grupo/rol?** — **NATIVO (solo por usuario; + manager dinámico)** · baja · `approval.category.approver.user_id` + `approval_category.py::manager_approval`. Se define por usuario individual o "manager del empleado"; **no** hay asignación nativa por grupo/rol.

**APPRCAP4. ¿Flujo secuencial vs paralelo?** — **CONFIG** · media · `approval.category.approver_sequence` (Boolean) + `approval_request.py::_update_next_approvers`. El flag activa orden secuencial por `sequence`; sin él, paralelo.

**APPRCAP5. ¿Campos activables por tipo (importe, fecha, partner, producto, ubicación…)?** — **NATIVO** · baja · `approval_category.py` selecciones `has_amount`/`has_date`/`has_period`/`has_quantity`/`has_reference`/`has_partner`/`has_payment_method`/`has_location`/`has_product` (no/optional/required). Cada campo se activa por categoría.

**APPRCAP6. ¿Adjuntar documentos/justificantes (obligatorios)?** — **NATIVO** · baja · `approval.category.requirer_document` + `approval_request.py::action_confirm` (exige adjunto si `requirer_document=='required'`). Adjuntos vía `attachment_ids`, opcionales u obligatorios.

**APPRCAP7. ¿Una aprobación genera un pedido de compra?** — **NATIVO** (módulo `approvals_purchase`) · media · `approvals_purchase/models/approval_request.py::action_create_purchase_orders` + `approval_type='purchase'`. La categoría tipo "Create RFQ's" genera órdenes de compra al aprobar.

**APPRCAP8. ¿Notificaciones/actividades automáticas a los aprobadores?** — **NATIVO** · baja · `approval_request.py::ApprovalApprover._create_activity` (`activity_schedule`) + `message_notify` en approve/refuse. Crea actividad al aprobador en turno y notifica al solicitante.

**APPRCAP9. ¿Routing condicional por umbral de importe?** — **NO** · — · `approval.category.has_amount` solo controla visibilidad/obligatoriedad del campo `amount`, no condiciona aprobadores; no hay lógica de umbral en `_compute_approver_ids`/`action_confirm`. Requeriría desarrollo.

**APPRCAP10. ¿Estados del flujo y quién puede qué?** — **NATIVO** · baja · `approval.request.request_status` (`new`/`pending`/`approved`/`refused`/`cancel`, `_compute_request_status` desde los approvers) + `approval.approver.status`. Aprobar/rechazar se restringe al `user_id` del approver; `group_approval_user` controla edición.

## Líneas, compras y alcance (profundización)

**APPRCAP11. ¿Líneas de producto en la solicitud (artículos + cantidades)?** — **NATIVO** · baja · `approval.product.line` (`models/approval_product_line.py`: `product_id`, `quantity`, `product_uom_category_id`) · La solicitud puede detallar qué se pide, no solo un importe.

**APPRCAP12. ¿Aprobador automático = responsable/manager del empleado?** — **NATIVO** · baja · `approval_category.py::manager_approval` (`Is Approver` / `Is Required Approver`) + `approver_sequence` (el manager va primero en cadena) · El jefe entra como aprobador sin configurarlo caso a caso.

**APPRCAP13. ¿Una solicitud aprobada genera pedido de compra?** — **EE-hermano** · media · `approvals_purchase/models/approval_product_line.py::_get_purchase_order_values` → `approval.request.action_create_purchase_orders` (agrupa por proveedor) · Requiere `approvals_purchase`; convierte las líneas en PO.

**APPRCAP14. ¿Recepción de stock tras aprobar la compra?** — **EE-hermano** · media · `approvals_purchase_stock/models/approval_product_line.py::_get_picking_type` (`_default_warehouse_id`) · Con `approvals_purchase_stock` la compra aprobada genera su albarán de entrada al almacén elegido.

**APPRCAP15. ¿Aprobar documentos estándar (PO/gasto/factura) o solo su propio modelo?** — **SOLO SU MODELO** · — · `approval.request` es un objeto **autónomo** (`_name='approval.request'`); NO envuelve `purchase.order`/`hr.expense`/`account.move`. Para aprobar un PO real se usa la **doble validación de Compras** (`po_double_validation`), no Approvals. No prometer que Approvals mete un flujo de visto bueno sobre los documentos estándar existentes.
