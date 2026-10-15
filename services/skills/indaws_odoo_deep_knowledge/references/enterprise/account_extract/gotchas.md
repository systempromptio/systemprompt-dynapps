<!-- verified: v18.0 @ 2026-06 -->
# account_extract — Gotchas (prefijo AEXG)

Comportamientos no obvios del OCR de facturas (`account_extract` / `iap_extract` / `account_invoice_extract`). Verificado contra fuente v18 (`odoo_enterprise/18/`).

---

### AEXG1 — `account_extract` no contiene código; la lógica está en `account_invoice_extract`

**Síntoma**: buscas el flujo OCR en `account_extract` y solo encuentras `__manifest__.py` + `__init__.py` vacío.

`account_extract` es un módulo **glue auto-install** (`account_extract/__manifest__.py:6-8`) que solo declara dependencias. La extensión de `account.move` y todo el OCR de facturas viven en `account_invoice_extract`. La abstracción base (`extract.mixin`) vive en `iap_extract`. Buscar ahí.

---

### AEXG2 — El PDF se envía codificado a un servidor externo de Odoo (privacidad)

**Síntoma**: cliente con datos sensibles pregunta dónde se procesa la factura.

`_upload_to_extract` (`iap_extract/models/extract_mixin.py:255`, `ExtractMixin._upload_to_extract`) hace `'documents': [x.datas.decode('utf-8') for x in attachment]` → el contenido **base64 del PDF/imagen sale de la instancia** hacia `https://extract.api.odoo.com` (`account_invoice_extract/models/account_invoice.py:100`, `AccountMove._contact_iap_extract`). No es OCR local. Implica enviar la factura completa (con todos sus datos) a Odoo S.A. Relevante para RGPD/clientes españoles sensibles. En bases neutralizadas el endpoint se borra (`neutralize.sql`) y deja de funcionar.

---

### AEXG3 — Auto-digitalización por defecto en facturas de proveedor

**Síntoma**: "cada factura de proveedor que entra consume un crédito sin que nadie lo pida".

`res.company.extract_in_invoice_digitalization_mode` tiene **default `auto_send`** (`account_invoice_extract/models/res_company.py:10-15`, `ResCompany.extract_in_invoice_digitalization_mode`). Cualquier `in_invoice` en borrador con attachment principal se envía sola al OCR (vía `register_as_main_attachment` o `_get_edi_decoder`). Las de cliente (`out_invoice`) por defecto son `manual_send` (`account_invoice_extract/models/res_company.py:16-21`, `ResCompany.extract_out_invoice_digitalization_mode`). Para desactivar: poner el modo en `no_send` en Ajustes de Contabilidad.

---

### AEXG4 — El alias de email del diario dispara OCR automáticamente

**Síntoma**: facturas recibidas por email aparecen ya rellenadas (o consumiendo créditos) sin intervención.

Una factura creada por alias de email (vía `account.move.message_new`, definido en `account` base `account/models/account_move.py:6379`, `AccountMove.message_new`, no en este módulo) registra el adjunto como principal y, si el modo es `auto_send`, `_get_edi_decoder` devuelve `_import_invoice_ocr` (`account_invoice_extract/models/account_invoice.py:956-962`, `AccountMove._get_edi_decoder`), que llama `_send_batch_for_digitization`. Es el flujo "buzón de facturas" → OCR end-to-end. Ver `integrations.md`.

---

### AEXG5 — Solo se procesa el `message_main_attachment_id`, no todos los adjuntos

**Síntoma**: subes varios PDFs al chatter pero solo se digitaliza uno.

`_upload_to_extract` usa `self.message_main_attachment_id` (`iap_extract/models/extract_mixin.py:243`, `ExtractMixin._upload_to_extract`), el adjunto **principal** del registro, no la lista completa. Si llegan varios adjuntos, solo el marcado como principal se envía. El botón de envío (`extract_can_show_send_button`) también exige que exista `message_main_attachment_id`.

---

### AEXG6 — El OCR no sobreescribe campos ya rellenados

**Síntoma**: rellenas manualmente el partner o la fecha antes de que llegue el resultado OCR, y el OCR no lo cambia.

`_save_form` solo escribe si el campo está vacío: `if not move_form.partner_id`, `if not move_form.ref`, `if date_ocr and (not move_form.invoice_date or ...)`, `add_lines = not move_form.invoice_line_ids` (`account_invoice_extract/models/account_invoice.py:781`, `AccountMove._save_form` y `account_invoice_extract/models/account_invoice.py:843`, `AccountMove._save_form` y `account_invoice_extract/models/account_invoice.py:852`, `AccountMove._save_form` y `account_invoice_extract/models/account_invoice.py:861`, `AccountMove._save_form`). Diseño intencional: el input del usuario gana. Para forzar re-relleno completo, usar el botón **Reload AI data** (`action_reload_ai_data`, `account_invoice_extract/models/account_invoice.py:72`, `AccountMove`), que resetea los campos primero.

---

### AEXG7 — `extract_state` máquina de estados: `done` no significa "validado bien" sino "feedback enviado"

**Síntoma**: confusión sobre qué estado indica qué.

`done` (`iap_extract/models/extract_mixin.py:122`, `ExtractMixin._cron_validate`) se alcanza tras `_cron_validate` enviar al servidor los campos corregidos por el usuario (aprendizaje del OCR), **después de postear** la factura. No es un estado de éxito de extracción. El estado donde los datos están listos para revisión humana es `waiting_validation`. `to_validate` es un estado transitorio puente que dispara el cron de validación.

---

### AEXG8 — La validación (feedback al OCR) solo ocurre al POSTEAR la factura

**Síntoma**: corriges campos pero el OCR "no aprende".

`_validate_ocr` se dispara desde `_post` (`account_invoice_extract/models/account_invoice.py:274-279`, `AccountMove`). Si la factura nunca se postea (queda en borrador), no se envía feedback y `extract_state` no llega a `done`. Además `_get_validation_domain` exige `state == 'posted'` (`account_invoice_extract/models/account_invoice.py:138-140`, `AccountMove`).

---

### AEXG9 — Al validar se borran las cajas OCR y el prefill data

**Síntoma**: tras postear, el widget de cajas visuales desaparece y no puedes re-seleccionar.

`_cron_validate` override (`account_invoice_extract/models/account_invoice.py:265-272`, `class AccountMove`) hace `validated.mapped('extract_word_ids').unlink()` y `extract_prefill_data = None`. Es intencional (ya no se necesitan), pero significa que no se puede reusar la selección visual tras la validación.

---

### AEXG10 — Detección de partner por "layout" depende de facturas previas del mismo proveedor

**Síntoma**: el primer documento de un proveedor no detecta partner por layout; los siguientes sí.

`_find_partner_from_previous_extracts` (`account_invoice_extract/models/account_invoice.py:340-362`, `class AccountMove`) busca moves con el mismo `extract_detected_layout` o `extract_partner_name` y `extract_state == 'done'`. Solo devuelve partner si hay **exactamente un** partner entre los matches (`len(...partner_id) == 1`). Requiere historial validado previo. No funciona en el primer documento ni si el layout mapea a varios partners.

---

### AEXG11 — Creación automática de proveedor desde VAT usa Partner Autocomplete (otro servicio IAP)

**Síntoma**: el OCR "crea proveedores solos" con dirección/datos enriquecidos, o falla silenciosamente si Partner Autocomplete no está.

`_create_supplier_from_vat` → `_fetch_autocomplete_values` (`account_invoice_extract/models/account_invoice.py:388-433`, `class AccountMove`) llama `iap.autocomplete.api._request_partner_autocomplete('enrich_by_vat', ...)`, que es el servicio **partner_autocomplete** (endpoint distinto, `https://partner-autocomplete.odoo.com`, créditos propios). Si no está instalado → `KeyError` capturado → devuelve None y la creación se desactiva silenciosamente (warning en log).

---

### AEXG12 — `extract_single_line_per_tax` (default True) cambia drásticamente las líneas resultantes

**Síntoma**: una factura con muchas líneas detalladas acaba como 1-2 líneas agregadas por tipo de impuesto.

`company.extract_single_line_per_tax` default `True` (`account_invoice_extract/models/res_company.py:22`, `class ResCompany`). Con True, `_get_invoice_lines` (`account_invoice_extract/models/account_invoice.py:613-652`, `AccountMove._get_invoice_lines`) **fusiona** todas las líneas OCR agrupando por conjunto de impuestos, con descripción genérica `proveedor - fecha`. Para conservar línea-a-línea hay que desactivarlo en Ajustes. Si tras agregar queda una sola línea, usa el subtotal total del OCR (menos propenso a error de redondeo).

---

### AEXG13 — Polling: webhook + cron diario; el webhook es `auth='public'`

**Síntoma**: el documento se queda en `waiting_extraction` mucho tiempo / dudas sobre seguridad del endpoint.

Hay dos rutas de actualización: el webhook `/account_invoice_extract/request_done/<uuid>` (`account_invoice_extract/controllers/main.py:8`, `class AccountInvoiceExtractController`, **`auth='public'`, `csrf=False`**) que el servidor IAP llama al terminar, y el cron diario `ir_cron_update_ocr_status` (`account_invoice_extract/data/crons.xml:4`, `ir_cron_update_ocr_status`) como respaldo. Si el webhook no es alcanzable (instancia sin URL pública / detrás de firewall), el estado solo avanza con el cron diario → percepción de lentitud. El endpoint público busca por `extract_document_uuid` (token), que actúa de credencial.

---

### AEXG14 — `error_invalid_account_token`: borra el token IAP para regenerarlo

**Síntoma**: `extract_state='error_status'` con mensaje "The 'invoice_ocr' IAP account token is invalid".

`_upload_to_extract` (`iap_extract/models/extract_mixin.py:247-250`, `ExtractMixin._upload_to_extract`) pone `error_invalid_account_token` si `account_token.account_token` está vacío. Solución (según el propio mensaje, `iap_extract/models/extract_mixin.py:24-26`, `ERROR_MESSAGES`): borrar el token en Ajustes › IAP para que Odoo genere uno nuevo, o rellenarlo con uno válido.

---

### AEXG15 — `_safe_upload` traga excepciones para no romper el lote (pero deja error_status)

**Síntoma**: en envío en lote, un documento falla pero el resto sí se envía; el fallido queda en `error_status`.

`_safe_upload` (`iap_extract/models/extract_mixin.py:132-147`, `ExtractMixin._safe_upload`) envuelve el upload en un `savepoint` y captura cualquier excepción. Si no es `IntegrityError`/`OperationalError`, marca `extract_state='error_status'`, `extract_status='error_internal'` y manda notificación de error, pero **no propaga**. Diseñado para envío batch. El log queda en warning ("Couldn't upload ...").

---

### AEXG16 — El OCR de facturas y el de extractos bancarios comparten mixin pero NO endpoint

**Síntoma**: confusión al asumir que comparten servicio/versión.

Ambos heredan `extract.mixin`, pero `account.move` usa `/api/extract/invoice/2/` con `OCR_VERSION = 122` (`account_invoice_extract/models/account_invoice.py:17`, `PARTNER_AUTOCOMPLETE_ENDPOINT` y `account_invoice_extract/models/account_invoice.py:100`, `AccountMove._contact_iap_extract`) y `account.bank.statement` usa `/api/extract/bank_statement/1/` con `OCR_VERSION = 100` (`account_bank_statement_extract/models/account_bank_statement.py:4`, `OCR_VERSION` (y `:35`, `AccountBankStatement._contact_iap_extract`)). Distinto endpoint y versión; comparten la cuenta IAP `invoice_ocr` y los créditos. Ver `integrations.md`.

---

### AEXG17 — Las notificaciones (no-credit, error, success) las provee `iap_mail`, no `iap`

**Síntoma**: `AttributeError` sobre `_send_no_credit_notification` si falta `iap_mail`.

`extract_mixin` llama `self.env['iap.account']._send_no_credit_notification(...)`, `_send_error_notification`, `_send_success_notification`, `_send_status_notification`. Estos métodos **no están en el módulo base `iap`**, sino en `iap_mail` (`odoo18/addons/iap_mail/models/iap_account.py`). Por eso `account_extract` depende de `iap_mail` (`account_extract/__manifest__.py:6`). Sin él, las notificaciones bus/email fallan.

---

### AEXG18 — `is_in_extractable_state` solo True en borrador y siendo factura

**Síntoma**: no aparece el botón de digitalización en un documento que no es factura o ya está posteado.

`_compute_is_in_extractable_state` (`account_invoice_extract/models/account_invoice.py:43-45`, `class AccountMove`): `state == 'draft' and is_invoice()`. Entradas de diario, asientos varios o facturas ya posteadas no son extractables. `_upload_to_extract` además hace `if self.is_invoice()` (`account_invoice_extract/models/account_invoice.py:169-173`, `class AccountMove`) para no enviar no-facturas.

---

### AEXG19 — El email de "sin créditos" se manda al usuario id=2, no a los creadores

**Síntoma**: el aviso "Not enough credits for data extraction" llega al admin/usuario base, no a quien procesa facturas.

Pese al comentario "Get the email address of the creators of the records", `_send_no_credit_notification` (`iap_extract/models/extract_mixin.py:316`, `ExtractMixin._send_no_credit_notification`) hace `search_read([('id', '=', 2)], ['email'])` — id=2 es el usuario administrador base (`base.user_admin`), hardcodeado. El email del aviso por crédito agotado va siempre a esa dirección, no al creador de la factura. Si el admin no tiene email configurado, no se envía nada (`if res:`, `iap_extract/models/extract_mixin.py:317`, `ExtractMixin._send_no_credit_notification`). La notificación bus (in-app) sí usa `_send_no_credit_notification` de `iap_mail` con destino más amplio; es solo el **email** el que va al id=2.

---

### AEXG20 — Solo se comprueban 20 documentos por click manual (`check_ocr_status`)

**Síntoma**: con muchas facturas en `waiting_extraction`, al refrescar manualmente no todas avanzan de estado.

`check_ocr_status` (`iap_extract/models/extract_mixin.py:208-225`, `ExtractMixin.check_ocr_status`) comprueba primero los registros del recordset en waiting/not_ready, y luego rellena hasta un **tope de 20** con un `search(..., limit=max(0, 20 - len(records_to_check)))` (`iap_extract/models/extract_mixin.py:216-221`, `ExtractMixin.check_ocr_status`). Es un límite anti-sobrecarga del check interactivo (el que dispara el cliente al abrir la vista). El avance masivo real lo hace el cron diario `check_all_status` (sin límite). No confundir: subir el límite no es configurable sin override.

---

### AEXG21 — Las trazas del relleno OCR aparecen firmadas por OdooBot, no por ti

**Síntoma**: el chatter de la factura muestra los cambios de campos hechos por el OCR como autor "OdooBot", aunque tú tengas la sesión abierta.

`_check_ocr_status` (`iap_extract/models/extract_mixin.py:343`, `ExtractMixin._check_ocr_status`) hace `self._track_set_author(self.env.ref('base.partner_root'))` antes de rellenar, para que el mensaje de tracking (cambios de partner, fecha, etc.) se atribuya a OdooBot y no al usuario que abrió el documento. Intencional: el relleno lo hace el sistema, no la persona. Si buscas "quién cambió el partner" en el log, será OdooBot.

---

### AEXG22 — El texto OCR completo se vuelca en `index_content` del adjunto (búsqueda full-text)

**Síntoma**: tras la digitalización, buscar texto que aparece dentro del PDF encuentra la factura aunque el PDF fuera una imagen escaneada (sin capa de texto).

`_check_ocr_status` (`iap_extract/models/extract_mixin.py:344-345`, `ExtractMixin._check_ocr_status`) escribe `message_main_attachment_id.index_content = ocr_results['full_text_annotation']` cuando el servidor lo devuelve. Esto alimenta el índice de búsqueda full-text de `ir.attachment`. Efecto colateral: el texto reconocido por OCR (incluyendo posibles datos sensibles) queda almacenado en claro en `index_content` y es buscable. Relevante para RGPD/retención.

---

### AEXG23 — `_validate_ocr` solo actúa sobre `waiting_validation`; postear dos veces no re-envía feedback

**Síntoma**: postear → cancelar → re-postear no manda feedback duplicado al OCR.

`_validate_ocr` (`iap_extract/models/extract_mixin.py:324-326`, `ExtractMixin._validate_ocr`) filtra `extract_state == 'waiting_validation'` y los pasa a `to_validate`. Una vez el cron los lleva a `done` (`_cron_validate`), un nuevo `_post` no encuentra docs en `waiting_validation` y no reenvía nada. El feedback al algoritmo OCR es **idempotente respecto a re-posteos**: solo se manda una vez, en la primera validación. El delay del cron es configurable vía context `ocr_trigger_delta` en minutos (`iap_extract/models/extract_mixin.py:329`, `ExtractMixin._validate_ocr`).

---

### AEXG24 — `error_status` por `AccessError` se mapea a "Server not available", enmascarando un problema de conexión IAP

**Síntoma**: `extract_state='error_status'` con mensaje "Server not available. Please retry later" aunque el servidor de Odoo esté operativo.

En `_upload_to_extract`, cualquier `AccessError` al contactar IAP (token rechazado, sin acceso al servicio, problema de red al endpoint) se captura y fija `extract_status='error_no_connection'` (`iap_extract/models/extract_mixin.py:285-287`, `ExtractMixin._upload_to_extract`), que `ERROR_MESSAGES` traduce como "Server not available" (`iap_extract/models/extract_mixin.py:20`, `ERROR_MESSAGES`). El mensaje al usuario sugiere reintentar, pero la causa real puede ser de credenciales/permisos IAP, no de disponibilidad del servidor. Revisar el log (`_logger.warning`) para el error crudo.
