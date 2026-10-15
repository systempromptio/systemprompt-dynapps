# account_extract — v19 Changes

Diffs verificados entre `odoo_enterprise/18/` y `odoo_enterprise/19/` para `iap_extract` y `account_invoice_extract`. El manifest de `account_extract` no cambia (sigue glue auto-install; v19 solo añade `author: 'Odoo S.A.'`). Crons y `res.company` de `account_invoice_extract` **sin cambios** (diff vacío).

---

## 1. Nuevo submodelo `extract.mixin.with.words`

Cambio arquitectónico mayor. v18 metía las cajas OCR en un modelo SQL aparte (`account.invoice_extract.words`) con un One2many; v19 las mueve a un mixin intermedio con campos `Json`.

`iap_extract/models/extract_mixin_with_words.py` (nuevo):

```python
class ExtractMixinWithWords(models.AbstractModel):
    _name = 'extract.mixin.with.words'
    _inherit = ['extract.mixin']
    extract_attachment_id = fields.Many2one('ir.attachment', ...)   # subido desde account.move
    extracted_words   = fields.Json()
    extracted_numbers = fields.Json()
    extracted_dates   = fields.Json()
```

- `account.move` ahora hace `_inherit = ['extract.mixin.with.words', 'account.move']` (antes `['extract.mixin', 'account.move']`).
- **Eliminados de v19**: el modelo `account.invoice_extract.words`, el campo `extract_word_ids` (One2many), y el campo `extract_attachment_id` directo en `account.move` (subió al mixin).
- Las cajas ya no son registros SQL sino JSON (`extracted_words/numbers/dates`), poblados en el nuevo hook `_on_ocr_results` (`iap_extract/models/extract_mixin.py:350`, `ExtractMixin._check_ocr_status` v19 llama `self._on_ocr_results(ocr_results)` dentro de `_check_ocr_status`).
- `get_boxes()` reescrito (`iap_extract/models/extract_mixin_with_words.py:26-52`, `ExtractMixinWithWords.get_boxes`): devuelve estructura agrupada por tipo (`word`/`number`/`date`) → página → cajas con `minX/midX/maxX/minY/midY/maxY/width/height/angle`. v18 devolvía lista plana por `extract_word_ids`.

---

## 2. UI de corrección: de "cajas por campo" a "selección por texto"

v18 tenía `set_user_selected_box(id)` (clic en una caja → asigna a su campo). v19 lo sustituye por métodos basados en texto libre:

- **`get_partner_from_text(text)`** (`account_invoice_extract/models/account_invoice.py:259-275`, `AccountMove._get_validation` v19): dado un texto seleccionado, decide si parece VAT (`_could_be_vat_number`, heurística: 6-15 dígitos y ≥30% del texto son dígitos) → busca/crea partner por VAT; si no, busca por nombre.
- **`get_partner_create_data(text)`** (antes `get_partner_create_data(context)`): rellena `default_vat` o `default_name` según `_could_be_vat_number(text)`.
- **`get_currency_from_text(text)`** (`iap_extract/models/extract_mixin_with_words.py:54-63`, `ExtractMixinWithWords.get_currency_from_text`): nuevo, devuelve currency por label/name/symbol.
- Eliminados los campos de caja con su lógica de `user_selected`/`ocr_selected` por registro.

---

## 3. Envío en postcommit (no perder créditos IAP)

`_send_batch_for_digitization` reescrito (`extract_mixin.py` v19):

```python
record_ids = self.ids
@self.env.cr.postcommit.add
def send_batch_for_digitization():
    with self.env.registry.cursor() as cr:
        records = self.env(cr=cr)[self._name].sudo().browse(record_ids)
        for record in records:
            record._safe_upload()
            cr.commit()        # commit por documento
```

v18 hacía `for rec in self: rec._safe_upload()` en la transacción actual con savepoints. **Motivo del cambio** (comentario en fuente): si se enviaba en la transacción actual, un rollback posterior reseteaba el estado de documentos ya enviados al OCR → pérdida de créditos IAP. Ahora cada documento se confirma individualmente tras el commit.

Consecuencia: `_safe_upload` v19 ya no usa `savepoint` ni distingue `IntegrityError/OperationalError` (siempre marca `error_status`/`error_internal` en fallo). `_try_to_check_ocr_status` v19 tampoco usa savepoint.

---

## 4. `res.partner.is_created_by_ocr` + índice único anti-duplicados

`account_invoice_extract/models/res_partner.py` (nuevo):

```python
is_created_by_ocr = fields.Boolean(default=False)
_ocr_name_uniq = models.UniqueIndex(
    '(name, is_created_by_ocr) WHERE is_created_by_ocr = TRUE AND active = TRUE',
    "The OCR can't create multiple partners with the same name.")
```

`_create_supplier_from_vat` (`account_invoice_extract/models/account_invoice.py:370-377`, `AccountMove._create_supplier_from_vat`, v19) marca `values['is_created_by_ocr'] = True` y captura `UniqueViolation` → relanza `ConcurrencyError("Duplicated OCR partner")` para que el mecanismo de retry de Odoo reintente y vea el partner creado en la transacción concurrente. Evita duplicados cuando varias facturas del mismo proveedor nuevo entran a la vez (ej. lote por email).

---

## 5. Soporte de recibos (`include_receipts=True`)

Todas las comprobaciones de tipo de documento añaden `include_receipts=True`:
`is_invoice(include_receipts=True)`, `is_purchase_document(include_receipts=True)`, `is_sale_document(include_receipts=True)` (decenas de sitios en `account_invoice.py` v19).

Detección de tipo en `_fill_document_with_results` (`account_invoice_extract/models/account_invoice.py:643-653`, `AccountMove._fill_document_with_results`, v19) ahora maneja `receipt` además de `refund`:

```python
detected_move_type = ocr_results.get('type')
if detected_move_type == 'receipt':
    in_invoice  → in_receipt
    out_invoice → out_receipt (si ir.config_parameter account.show_sale_receipts)
elif detected_move_type == 'refund':
    action_switch_move_type()
```

`is_in_extractable_state` v19: `state == 'draft' and is_invoice(include_receipts=True)`.

---

## 6. Bridge EDI reescrito (prioridad por tipo de fichero)

`_get_edi_decoder` (`account_invoice_extract/models/account_invoice.py:885-890`, `AccountMove._get_edi_decoder`, v19) devuelve dict con prioridad:

```python
if file_data['import_file_type'] in {'pdf', 'jpg', 'png'} and file_data['attachment']:
    return {'decoder': self._import_invoice_ocr,
            'priority': 10 if pdf else 5}
```

Nuevo `_get_import_file_type` (v19 `account_invoice_extract/models/account_invoice.py:864-872`, `AccountMove._save_form`): fuerza tipo `jpg`/`png` según mimetype/extensión (formatos soportados por el OCR).

`_import_invoice_ocr` v19 añade guards: si ya hay `invoice_line_ids` → no decodifica; si no `_needs_auto_extract` → mensaje "Automatic OCR does not apply".

Nuevo `_disable_discount_precision` (contextmanager, `account_invoice_extract/models/account_invoice.py:876`, `AccountMove._save_form`): no desactiva la precisión de descuento cuando el contexto `from_ocr=True` está activo. El contexto `from_ocr=True` se propaga en `_save_form`, `_fill_document_with_results`, `action_reload_ai_data` (antes solo `skip_is_manually_modified`).

---

## 7. Cambios menores en `extract.mixin`

- Nuevo error: `'error_unsupported_version': "Version is unsupported"` (`ERROR_MESSAGES`).
- `_inherit = ['mail.thread.main.attachment']` (lista en vez de string).
- `OCR_VERSION` factura: 122 → **123**.
- `_get_iap_account().sudo().account_token` (añade `.sudo()` al leer el token).
- `buy_credits`: `get_credits_url(service_name='invoice_ocr')` (sin `base_url=''`).
- Nuevo hook vacío `_on_ocr_results(ocr_results)` en `extract.mixin` para extensión (lo implementa `extract.mixin.with.words`).

---

## 8. Líneas OCR: estructura de respuesta simplificada

v18 leía taxes de líneas con `il.get('taxes', {}).get('selected_values', [])` y `_get_ocr_selected_value(il, 'total', ...)`. v19 las lee directamente del dict: `il.get('total', 0.0)`, `il.get('taxes', [])`, con `taxes_type_ocr = ['percent'] * len(taxes)` (`account_invoice_extract/models/account_invoice.py:560-563`, `AccountMove._get_taxes_record` y `account_invoice_extract/models/account_invoice.py:599-605`, `AccountMove._get_currency` v19). Refleja un cambio en el formato de respuesta del servicio OCR para las líneas.

---

## Resumen de impacto para migración v18→v19

| Cambio | Impacto |
|--------|---------|
| `account.invoice_extract.words` eliminado | Código custom que lea `extract_word_ids` o `get_boxes()` (formato lista) rompe. Migrar a `extracted_words/numbers/dates` (Json) y al nuevo `get_boxes()` agrupado. |
| `extract_attachment_id` sube a mixin | Sigue accesible en `account.move` por herencia; sin cambio funcional. |
| `set_user_selected_box` → `get_partner_from_text` | Overrides del widget de selección visual deben reescribirse. |
| postcommit batch | Tests que asumían envío síncrono en la misma transacción fallan; el upload ocurre tras commit. |
| `is_created_by_ocr` + índice único | Nueva columna en `res.partner`; posibles `ConcurrencyError` esperados en creación concurrente. |
| `include_receipts=True` | Recibos (`in_receipt`/`out_receipt`) ahora entran en el flujo OCR. |
