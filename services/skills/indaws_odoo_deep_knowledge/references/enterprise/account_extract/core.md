<!-- verified: v18.0 @ 2026-06 -->
# account_extract — OCR de facturas vía IAP

El módulo `account_extract` (`odoo_enterprise/18/account_extract/`) es **solo un glue/auto-install** (`account_extract/__manifest__.py:6-8`): no contiene código, solo declara `depends: ['account', 'iap_extract', 'iap_mail', 'mail_enterprise', 'account_bank_statement_import']` con `auto_install=True`. Su única función es activar la pila OCR completa cuando `account` + Enterprise están instalados.

**La lógica real de OCR de facturas vive en `account_invoice_extract`** (`odoo_enterprise/18/account_invoice_extract/`), que extiende `account.move`. La abstracción base de digitalización vive en `iap_extract` (modelo `extract.mixin`).

```
account_extract (glue, vacío)
  └─ iap_extract → extract.mixin (AbstractModel, flujo OCR genérico)
       └─ account_invoice_extract → account.move hereda extract.mixin (facturas)
       └─ account_bank_statement_extract → account.bank.statement hereda extract.mixin (extractos)
       └─ hr_expense_extract / hr_recruitment_extract (otros consumidores)
```

`iap_extract`, `account_invoice_extract`, `account_bank_statement_extract` son `OEEL-1` (Enterprise).

## Propósito (contexto ES)

Digitalizar facturas de proveedor (PDF escaneado o imagen) enviándolas al servicio OCR de Odoo (IAP), que devuelve los campos detectados (proveedor, NIF/VAT, importes, fechas, líneas, impuestos, IBAN) y rellena la `account.move` en borrador para validación del usuario. Especialmente útil para automatizar la entrada masiva de facturas de proveedor de clientes españoles (alias de email + auto-digitalización).

**Privacidad / IAP**: el PDF/imagen se **codifica en base64 y se envía al servicio externo de Odoo** (`https://extract.api.odoo.com`). Consume **créditos IAP** del servicio `invoice_ocr`. Ver nota completa en `integrations.md`.

## Modelos

| Modelo | Fichero | Propósito |
|--------|---------|-----------|
| `extract.mixin` | `iap_extract/models/extract_mixin.py:33`, `ExtractMixin` | AbstractModel base. Hereda `mail.thread.main.attachment`. Define `extract_state`, polling, upload, validación. |
| `account.move` (extendido) | `account_invoice_extract/models/account_invoice.py:39`, `AccountMove` | `_inherit = ['extract.mixin', 'account.move']`. Implementa `_fill_document_with_results`, búsqueda de partner/taxes/currency. |
| `account.invoice_extract.words` | `account_invoice_extract/models/account_invoice.py:21`, `AccountInvoiceExtractionWords` | Cajas OCR detectadas (coords de bounding box por campo) para el widget de selección visual. **Eliminado en v19** (ver `v19_changes.md`). |
| `iap.account` | `odoo18/addons/iap/models/iap_account.py::IapAccount` | Cuenta IAP con `account_token`, balance de créditos, servicio `invoice_ocr`. |
| `iap.service` | `odoo18/addons/iap/models/iap_service.py` | Define el servicio `invoice_ocr` (`iap_extract/data/iap_service_data.xml`). |
| `ir.attachment` (extendido) | `account_invoice_extract/models/ir_attachment.py:7`, `IrAttachment` | Override `register_as_main_attachment` → dispara auto-extract. |
| `res.company` (extendido) | `account_invoice_extract/models/res_company.py:7`, `ResCompany` | Modos de digitalización (in/out invoice) + `extract_single_line_per_tax`. |

## Campos clave de `extract.mixin`

`iap_extract/models/extract_mixin.py:39-55`, `class ExtractMixin`:

- **`extract_state`** (Selection, default `no_extract_requested`, `copy=False`): máquina de estados del flujo OCR. Valores:
  - `no_extract_requested` — aún no enviado.
  - `not_enough_credit` — falta de créditos IAP.
  - `error_status` — error (mensaje en `extract_status`).
  - `waiting_extraction` — enviado, esperando que el servidor procese.
  - `extract_not_ready` — el servidor respondió "processing" (aún no listo).
  - `waiting_validation` — resultados recibidos y campos rellenados; pendiente de validación del usuario.
  - `to_validate` — marcado para enviar feedback de validación al OCR (transitorio, lo pone `_validate_ocr`).
  - `done` — flujo completado (feedback enviado).
- **`extract_status`** (Char): código de estado/error crudo del servicio. `_compute_error_message` lo mapea a texto vía `ERROR_MESSAGES` (`iap_extract/models/extract_mixin.py:16-30`, `ERROR_MESSAGES`).
- **`extract_document_uuid`** (Char, readonly): token del documento en el servidor IAP-OCR. Clave para polling y webhook.
- **`extract_can_show_send_button`** (compute): True si se puede mostrar el botón "Send for digitization" (`_compute_show_send_button`, `iap_extract/models/extract_mixin.py:73-81`, `class ExtractMixin`): requiere `_get_ocr_option_can_extract()`, attachment principal, estado `no_extract_requested` y `is_in_extractable_state`.
- **`is_in_extractable_state`** (compute store): meant-to-override. En `account.move`: `state == 'draft' and is_invoice()` (`account_invoice_extract/models/account_invoice.py:43-45`, `AccountMove._compute_is_in_extractable_state`).
- **`extract_state_processed`** (Boolean, compute store, `iap_extract/models/extract_mixin.py:55`, `ExtractMixin.extract_state_processed` y `iap_extract/models/extract_mixin.py:68-71`, `ExtractMixin._compute_extract_state_processed`): True solo cuando `extract_state == 'waiting_extraction'`. Campo auxiliar de UI/dominio para distinguir "ya enviado y procesándose" del resto de estados.
- **`extract_error_message`** (Text, compute `_compute_error_message`, `iap_extract/models/extract_mixin.py:51`, `ExtractMixin.extract_error_message` y `iap_extract/models/extract_mixin.py:57-66`, `ExtractMixin._compute_error_message`): traduce `extract_status` vía `ERROR_MESSAGES`; vacío si `extract_status in ('success','processing')`.

Campos extra en `account.move` (`account_invoice_extract/models/account_invoice.py:64-70`, `class AccountMove`): `extract_prefill_data` (Json, datos para crear partner), `extract_word_ids` (One2many a las cajas), `extract_attachment_id`, `extract_detected_layout` (Integer — layout detectado para reconocer mismo proveedor), `extract_partner_name`.

## Flujo OCR completo

### 1. Subida (upload → parse)

`_upload_to_extract` (`iap_extract/models/extract_mixin.py:238-291`, `ExtractMixin._upload_to_extract`):

```
attachment = self.message_main_attachment_id
account_token = self._get_iap_account()      # iap.account.get('invoice_ocr')
if not account_token.account_token: → extract_state='error_status', status='error_invalid_account_token'
params = {
  'dbuuid', 'documents': [att.datas.decode('utf-8')],   # ← base64 del PDF/imagen al servidor
  'user_infos', 'webhook_url': baseurl + '/account_invoice_extract/request_done'
}
result = self._contact_iap_extract('parse', params)
  → POST a endpoint + '/api/extract/invoice/2/parse'   (account_invoice.py:96-100)
status == 'success'      → extract_state='waiting_extraction', extract_document_uuid = result['document_token']
status == 'error_no_credit' → _send_no_credit_notification(); extract_state='not_enough_credit'
otro                     → extract_state='error_status'
```

Disparadores de la subida:
- **Manual**: botón → `action_send_batch_for_digitization` (`iap_extract/models/extract_mixin.py:153`, `ExtractMixin.action_send_batch_for_digitization`) → `_send_batch_for_digitization` → `_safe_upload` (con savepoint para no romper la transacción en lote).
- **Auto**: al registrar el attachment principal, `ir.attachment.register_as_main_attachment` (`account_invoice_extract/models/ir_attachment.py:10-19`, `IrAttachment.register_as_main_attachment`) llama `move._needs_auto_extract()` y si procede `_send_batch_for_digitization()`.
- **EDI / importación**: `_get_edi_decoder` (`account_invoice_extract/models/account_invoice.py:956-962`, `AccountMove._get_edi_decoder`) devuelve `_import_invoice_ocr` para ficheros `pdf`/`binary` si `_needs_auto_extract`. Así, una factura creada por alias de email o subida al diario entra automáticamente.

`_needs_auto_extract` (`account_invoice_extract/models/account_invoice.py:112-129`, `class AccountMove`): requiere `extract_state == 'no_extract_requested'`, modo `auto_send` para el tipo de documento, y `is_in_extractable_state`. Documentos nuevos siempre se auto-extraen; en documentos existentes con attachment añadido, solo facturas de compra.

### 2. Polling de estado (extract_state)

Dos mecanismos:

- **Webhook (push)**: el servidor llama `/account_invoice_extract/request_done/<uuid>` (`account_invoice_extract/controllers/main.py:8-16`, `class AccountInvoiceExtractController`, `auth='public'`, `csrf=False`). Busca el move por `extract_document_uuid` en estado `waiting_extraction`/`extract_not_ready` y llama `_check_ocr_status()`.
- **Cron (pull)**: `ir_cron_update_ocr_status` (`account_invoice_extract/data/crons.xml:4`, `ir_cron_update_ocr_status` -11`, diario) ejecuta `model.check_all_status()` → recorre `_get_to_check_domain()` (`iap_extract/models/extract_mixin.py:381-383`, `ExtractMixin._get_to_check_domain`: `is_in_extractable_state=True` y estado en waiting/not_ready) y llama `_try_to_check_ocr_status()` por registro.

`_check_ocr_status` (`iap_extract/models/extract_mixin.py:332-354`, `ExtractMixin._check_ocr_status`):

```
result = self._contact_iap_extract('get_result', {'document_token': uuid})
status == 'success'    → extract_state='waiting_validation'
                          self._fill_document_with_results(result['results'][0])
                          autor del tracking = OdooBot (base.partner_root)
                          index_content del attachment = full_text_annotation
status == 'processing' → extract_state='extract_not_ready'
otro                   → extract_state='error_status'
bus.bus._sendone(canal, 'state_change', {...})   # actualiza UI en vivo
```

### 3. Relleno de campos (`_fill_document_with_results`)

`account_invoice_extract/models/account_invoice.py:691-759`, `AccountMove` (+ `_save_form` `account_invoice_extract/models/account_invoice.py:761-934`, `class AccountMove`). Solo actúa si `state == 'draft'`. Pasos:

1. `extract_detected_layout` ← `detected_layout_id` (para reconocer mismo proveedor en futuras facturas).
2. Si OCR detecta `type == 'refund'` y move es `in_invoice`/`out_invoice` → `action_switch_move_type()` (a nota de crédito; nunca al revés).
3. **`extract_prefill_data`** (Json): country, email, website, phone, mobile, vat — para crear partner si no existe.
4. **Partner** (`_get_partner`, `account_invoice_extract/models/account_invoice.py:490-518`, `class AccountMove`): orden de prioridad → VAT exacto → layout previo (`_find_partner_from_previous_extracts`) → IBAN → nombre fuzzy (`_find_partner_id_with_name`, SequenceMatcher ratio > 0.8) → crear desde VAT (partner autocomplete IAP).
5. **Fecha** (`invoice_date`), **fecha vencimiento** (`invoice_date_due`); si fecha = vencimiento y el partner tiene `property_supplier_payment_term_id`, usa el plazo de pago en su lugar.
6. **Referencia**: en compra → `ref`; en venta con quick_edit → `name`.
7. **Moneda** (`_get_currency`, `account_invoice_extract/models/account_invoice.py:582-598`, `class AccountMove`): coincidencia por label/name/symbol; prioriza la del último invoice del partner o la de la compañía.
8. **Líneas** (`_get_invoice_lines`, `account_invoice_extract/models/account_invoice.py:601-672`, `class AccountMove`): dos modos según `company.extract_single_line_per_tax` — una línea por impuesto (merge) o una línea por línea OCR. Detección de impuestos `_get_taxes_record` busca taxes ya usados con ese proveedor.
9. **IBAN/SWIFT** (`_get_bank_account_vals`) → crea/asocia `res.partner.bank`.
10. **QR-bill** (Suiza): parsea el contenido del QR para rellenar partner+IBAN.
11. **Cajas OCR** (`extract_word_ids`): crea registros `account.invoice_extract.words` con coords para el widget visual (solo v18).
12. `_autopost_bill()` al final.

### 4. Validación del usuario y feedback al OCR

Al **postear** la factura (`_post`, `account_invoice_extract/models/account_invoice.py:274-279`, `class AccountMove`), se llama `_validate_ocr()` (`iap_extract/models/extract_mixin.py:324-330`, `ExtractMixin._validate_ocr`): marca los docs `waiting_validation` → `to_validate` y dispara el cron `ir_cron_ocr_validate`.

`_cron_validate` (`iap_extract/models/extract_mixin.py:104-123`, `class ExtractMixin` + override `account_invoice_extract/models/account_invoice.py:265-272`, `class AccountMove`): por cada doc en `to_validate`, envía a `'validate'` los campos corregidos por el usuario (`_get_validation_fields`, `account_invoice_extract/models/account_invoice.py:142-146`, `class AccountMove`: total, subtotal, total_tax_amount, date, due_date, invoice_id, partner, VAT_Number, currency, payment_ref, iban, SWIFT_code, merged_lines, invoice_lines). Esto **mejora el algoritmo OCR** (aprendizaje). Tras enviar: `extract_state = 'done'` y se borran `extract_word_ids` + `extract_prefill_data` (ya no se necesitan).

`action_reload_ai_data` (`account_invoice_extract/models/account_invoice.py:72-93`, `class AccountMove`): resetea los campos rellenados (partner, fechas, líneas, ref…) y vuelve a llamar `_check_ocr_status()` para re-rellenar.

## Créditos IAP

- Servicio `invoice_ocr` (`iap_extract/data/iap_service_data.xml`): `unit_name='Documents'`, `integer_balance=True`.
- `_get_iap_account` (`iap_extract/models/extract_mixin.py:88-92`, `ExtractMixin._get_iap_account`): `iap.account.get('invoice_ocr')` (filtrado por compañía).
- Sin créditos → `error_no_credit` → `extract_state='not_enough_credit'` + email de aviso (`_send_no_credit_notification`, `iap_extract/models/extract_mixin.py:293-322`, `ExtractMixin._send_no_credit_notification`, plantilla `iap_extract.iap_extract_no_credit`). Para no spamear cada hora, usa `ir.config_parameter iap_extract.already_notified`.
- `buy_credits` (`iap_extract/models/extract_mixin.py:201-206`, `ExtractMixin.buy_credits`): abre `iap.account.get_credits_url(service_name='invoice_ocr')`.
- Endpoint configurable: `ir.config_parameter iap_extract_endpoint` (default `https://extract.api.odoo.com`, `iap_extract/data/config_parameter_endpoint.xml`). En bases neutralizadas se borra (`neutralize.sql`).

## Config (res.company)

`account_invoice_extract/models/res_company.py`:

- `extract_in_invoice_digitalization_mode` (facturas de proveedor): `no_send` / `manual_send` / `auto_send` — **default `auto_send`**.
- `extract_out_invoice_digitalization_mode` (facturas de cliente): **default `manual_send`**.
- `extract_single_line_per_tax` (default `True`): agrupa líneas por impuesto en lugar de una línea por línea detectada.

Expuestos en Ajustes vía `res.config.settings` (`res_config_settings.py`).
