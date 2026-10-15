# Safety Checklist (Obligatoria antes de Modo C)

Antes de ejecutar cualquier flujo en Modo C, verificar y confirmar cada punto:

1. **Safe mode activado**: Se ha ejecutado `safe_mode.py activate` (desactiva mail servers, integraciones, crons peligrosos, fetchmail)
2. **Mail servers verificados**: NO hay mail servers activos que puedan enviar emails reales
3. **Automatizaciones pausadas**: Se han desactivado las automatizaciones peligrosas (RED category)
4. **Pre-state snapshot capturado**: Se ha ejecutado `snapshot.py capture`
5. **Prefijo de test configurado**: Se ha generado `[TEST-YYYYMMDD-HHMM]`
6. **Dry-run de reversión aprobado**: Se ha ejecutado el dry-run y el usuario lo ha aprobado
7. **Localización fiscal verificada**: Se han detectado módulos SII, TicketBAI, Verifactu u otros EDI fiscales
8. **Permisos de acceso verificados**: `ir.model.access` y `ir.rule` del usuario API confirmados
9. **Productos storable verificados**: Usar `is_storable=True` (no `type='product'`) en Odoo **18.0+**; en 17.0 es al revés
10. **Módulos EDI fiscales verificados**: Detectados módulos que puedan generar envíos fiscales reales
11. **Odoo 19.0 adaptaciones**: `request_date_from/to` en hr.leave, `disable_abnormal_invoice_detection` en facturas, `is_locked` en pickings

**Si algún punto NO se cumple, NO proceder con la ejecución.**
