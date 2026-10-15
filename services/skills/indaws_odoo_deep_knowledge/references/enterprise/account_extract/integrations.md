<!-- verified: v18.0 @ 2026-06 -->
# account_extract — Integrations

Cómo el OCR de facturas (`account_extract` → `account_invoice_extract` sobre `extract.mixin` de `iap_extract`) se conecta con otros módulos. Foco en los bridges existentes, el flujo IAP, y los puntos de extensión.

Recordatorio (ver `core.md`): `account_extract` es solo glue auto-install; la lógica vive en `account_invoice_extract`.

---

## 1. Bridge ↔ `account.move` (in_invoice / out_invoice)

`account_invoice_extract` extiende `account.move` con `_inherit = ['extract.mixin', 'account.move']` (`account_invoice_extract/models/account_invoice.py:39-40`, `AccountMove`). Es el bridge principal.

**Trigger automático (3 vías)**:

```
A) Adjunto principal registrado
   ir.attachment.register_as_main_attachment()  [ir_attachment.py:10]
     └─ move._needs_auto_extract() → move._send_batch_for_digitization()

B) Importación / EDI (subida a diario, drag&drop)
   account.move._get_edi_decoder(file_data)  [account_invoice.py:947]
     └─ si pdf/binary y _needs_auto_extract → decoder = _import_invoice_ocr
          └─ _message_set_main_attachment_id + _send_batch_for_digitization

C) Manual (botón "Send for digitization")
   action_send_batch_for_digitization()  [extract_mixin.py:153]
```

**`_needs_auto_extract`** (`account_invoice_extract/models/account_invoice.py:112-129`, `class AccountMove`): `extract_state == 'no_extract_requested'` + modo `auto_send` para el tipo + `is_in_extractable_state` (draft & factura). Nuevos docs: siempre. Docs existentes con adjunto añadido: solo compra.

**Perspectiva** (`_get_user_infos`, `account_invoice_extract/models/account_invoice.py:159-167`, `class AccountMove`): envía al OCR `user_company_VAT`, `user_company_name`, `user_company_country_code` y `perspective = 'supplier' if is_sale_document() else 'client'`. Así el servidor sabe si el "yo" del documento es emisor o receptor.

**Diferencia compra/venta**:
- `in_invoice`: rellena `ref` (referencia proveedor), busca partner con `supplier_rank`, crea `res.partner.bank` desde IBAN.
- `out_invoice`: rellena `name` (si quick_edit), busca partner con `customer_rank`.

---

## 2. Bridge ↔ IAP (`iap.account`, `iap.service`, créditos)

`extract.mixin` consume el servicio IAP **`invoice_ocr`**.

```
iap_extract/data/iap_service_data.xml → iap.service technical_name='invoice_ocr',
                                          unit_name='Documents', integer_balance=True
extract_mixin._get_iap_account()       → iap.account.get('invoice_ocr')  [extract_mixin.py:88]
                                          (filtrado por allowed_company_ids si hay company_id)
account_invoice._contact_iap_extract() → iap_tools.iap_jsonrpc(
                                            endpoint + '/api/extract/invoice/2/' + pathinfo)
                                          con params['account_token'], params['version']=122
endpoint = ir.config_parameter 'iap_extract_endpoint'
           default 'https://extract.api.odoo.com'  [config_parameter_endpoint.xml]
```

**`iap.account`** (`odoo18/addons/iap/models/iap_account.py::IapAccount`): `account_token` (uuid, credencial), `balance` (créditos, dinámico desde servidor IAP), `service_id` → `invoice_ocr`. `get('invoice_ocr')` crea la cuenta si no existe (`iap/models/iap_account.py:158-177`, `IapAccount.get`).

**Créditos**:
- Sin créditos → respuesta `error_no_credit` → `extract_state='not_enough_credit'` + email (`_send_no_credit_notification`, plantilla `iap_extract.iap_extract_no_credit`).
- `buy_credits` (`iap_extract/models/extract_mixin.py:201`, `ExtractMixin.buy_credits`) → `iap.account.get_credits_url('invoice_ocr')` → URL de compra en odoo.com.
- Las **notificaciones** (`_send_no_credit_notification`, `_send_error_notification`, `_send_success_notification`, `_send_status_notification`) las aporta **`iap_mail`** (no el `iap` base) → de ahí la dependencia. Ver AEXG17.

**Servicios IAP distintos en juego** (no confundir):
- `invoice_ocr` — el OCR de digitalización (este módulo).
- `partner_autocomplete` — enriquecimiento de partner desde VAT (`PARTNER_AUTOCOMPLETE_ENDPOINT`, `account_invoice_extract/models/account_invoice.py:17`, `PARTNER_AUTOCOMPLETE_ENDPOINT`), usado en `_create_supplier_from_vat`. Endpoint y créditos propios.

---

## 3. Bridge ↔ mail (alias → factura → OCR)

Flujo "buzón de facturas por email":

```
Email a alias del diario (ej. facturas@empresa.odoo.com)
  └─ account.move.message_new()  [account/models/account_move.py:6303]
       └─ crea account.move (in_invoice draft) + adjunta el PDF del email
            └─ adjunto se marca como main → register_as_main_attachment()  [ir_attachment.py:10]
                 └─ _needs_auto_extract() (modo auto_send) → _send_batch_for_digitization()
                      └─ OCR end-to-end sin intervención humana
```

Esto, combinado con `extract_in_invoice_digitalization_mode='auto_send'` (default), es el caso de uso estrella para clientes ES: el proveedor envía la factura por email y entra digitalizada en borrador lista para validar. Requiere `mail` + alias configurado en el diario de compras. El autor del tracking de los cambios OCR se fija como **OdooBot** (`base.partner_root`, `iap_extract/models/extract_mixin.py:342`, `ExtractMixin._check_ocr_status`).

`account_extract` depende también de **`mail_enterprise`** (`account_extract/__manifest__.py:6`) — necesario para la previsualización del PDF embebida en el formulario (visor de documento sobre el que se superponen las cajas OCR).

---

## 4. Bridge ↔ documents (Odoo Documents)

No hay un módulo bridge dedicado `account_extract_documents`. La integración es indirecta vía el mecanismo estándar de `documents` que crea `account.move` desde un documento y registra el PDF como adjunto principal → dispara el mismo flujo `register_as_main_attachment` → auto-extract. El widget de carga de Documents que genera facturas reutiliza `_get_edi_decoder` / `_import_invoice_ocr`. Es decir, Documents alimenta el OCR a través del mismo punto de entrada que el alias de email y la subida al diario; no hay código específico de OCR en `documents`.

---

## 5. Bridge ↔ `account_bank_statement_extract` (relación, mismo mixin)

Módulo hermano que aplica el **mismo `extract.mixin`** a extractos bancarios.

```
account.bank.statement  _inherit = ['extract.mixin', 'account.bank.statement']
                         [account_bank_statement_extract/models/account_bank_statement.py:7-9]
```

Diferencias clave respecto a facturas:

| Aspecto | account_invoice_extract | account_bank_statement_extract |
|---------|-------------------------|--------------------------------|
| Endpoint | `/api/extract/invoice/2/` | `/api/extract/bank_statement/1/` |
| `OCR_VERSION` | 122 | 100 |
| `is_in_extractable_state` | draft & factura | `not self.line_ids` (extracto vacío) |
| Modo company | `extract_in/out_invoice_digitalization_mode` | `extract_bank_statement_digitalization_mode` (default `auto_send`, solo no_send/auto_send) |
| Disparo | adjunto / EDI / manual | `account.journal._import_bank_statement` si todos los adjuntos son PDF/imagen (`account_bank_statement_extract/models/account_journal.py:32-55`, `AccountJournal._import_bank_statement`) |
| Resultado | partner, importes, líneas, taxes | `balance_start`, `balance_end`, `date`, `line_ids` (transacciones) |
| Post-relleno | `_autopost_bill` | dispara `auto_reconcile_bank_statement_line` cron (`account_bank_statement_extract/models/account_bank_statement.py:61`, `AccountBankStatement._fill_document_with_results`) |

**Relación**: comparten la cuenta IAP `invoice_ocr` y los créditos, el endpoint base (`iap_extract_endpoint`), el patrón de estados (`extract_state`) y el polling (cron + webhook `/account_bank_statement_extract/request_done`). Solo difieren en el path del API, la versión y el mapeo de resultados. `account_journal._import_bank_statement` (`account_bank_statement_extract/models/account_journal.py:32`, `AccountJournal._import_bank_statement`) intercepta la importación: si los adjuntos son PDF/JPEG/PNG, crea `account.bank.statement` y los envía al OCR; si son formatos bancarios (OFX/CAMT/CSV/QIF) delega en el import estándar. **No se permite mezclar** PDF/imagen con otros formatos (UserError, `account_bank_statement_extract/models/account_journal.py:35-36`, `AccountJournal._import_bank_statement`).

---

## Diagrama de integración

```
                  ┌─────────────────────────────────────────────┐
                  │  Servicio OCR de Odoo (IAP, externo)         │
                  │  https://extract.api.odoo.com                │
                  │   /api/extract/invoice/2/{parse,get_result,  │
                  │                            validate}         │
                  └───────▲───────────────────────┬─────────────┘
       base64 PDF/img │   │ params+account_token  │ webhook request_done
       + créditos IAP │   │                       ▼ (auth=public)
                      │   │             /account_invoice_extract/request_done/<uuid>
   ┌──────────────────┴───┴───────────────────────────────────────────────┐
   │  extract.mixin  (iap_extract)                                         │
   │   extract_state · _upload_to_extract · _check_ocr_status · _validate  │
   └──▲───────────────▲────────────────────▲──────────────────▲───────────┘
      │ inherit       │ inherit             │ get('invoice_ocr')│ notif
┌─────┴──────┐  ┌─────┴────────────┐  ┌─────┴──────┐    ┌───────┴────────┐
│account.move│  │account.bank.     │  │ iap.account│    │   iap_mail     │
│(in/out     │  │  statement       │  │ iap.service│    │ _send_*_notif  │
│ invoice)   │  │ (extract)        │  │ (créditos) │    └────────────────┘
└──▲──▲──▲───┘  └──────────────────┘  └─────┬──────┘
   │  │  │                                  │
   │  │  └─ register_as_main_attachment ◄── ir.attachment
   │  └──── message_new (alias email) ◄──── mail / mail_enterprise
   └─────── _import_invoice_ocr / Documents
   │
   └─ partner_autocomplete (IAP aparte, enrich_by_vat) → crea res.partner
```

---

## Modelos (resumen tabla)

| Modelo | Módulo | Rol |
|--------|--------|-----|
| `extract.mixin` | iap_extract | AbstractModel base del flujo OCR (`iap_extract/models/extract_mixin.py:33`, `class ExtractMixin`) |
| `account.move` (ext) | account_invoice_extract | Factura digitalizable (`account_invoice_extract/models/account_invoice.py:39`, `AccountMove`) |
| `account.invoice_extract.words` | account_invoice_extract | Cajas OCR / bounding boxes (`account_invoice_extract/models/account_invoice.py:21`, `AccountInvoiceExtractionWords`) — eliminado v19 |
| `account.bank.statement` (ext) | account_bank_statement_extract | Extracto digitalizable |
| `iap.account` | iap (base) | Cuenta IAP + token + balance |
| `iap.service` | iap (base) | Servicio `invoice_ocr` |
| `res.company` (ext) | account_invoice_extract / _bank_statement_extract | Modos de digitalización |
| `ir.attachment` (ext) | account_invoice_extract | Trigger auto-extract |

---

## Config (puesta en marcha)

1. **Instalar**: con Contabilidad Enterprise instalada, `account_extract` se auto-instala (arrastra `iap_extract`, `iap_mail`, `mail_enterprise`).
2. **Créditos IAP**: Ajustes › Contabilidad › sección Digitalización → comprar créditos del servicio "Document Digitization" (`invoice_ocr`).
3. **Modo por compañía** (`res.config.settings`):
   - Facturas de proveedor (`extract_in_invoice_digitalization_mode`): `no_send` / `manual_send` / `auto_send` (default **auto_send**).
   - Facturas de cliente (`extract_out_invoice_digitalization_mode`): default **manual_send**.
   - `extract_single_line_per_tax` (default **True**): una línea por impuesto vs línea-a-línea.
   - Extractos bancarios (`extract_bank_statement_digitalization_mode`): default **auto_send**.
4. **Alias de email** (opcional, recomendado ES): configurar alias en el diario de compras → facturas por email entran y se digitalizan solas.
5. **URL pública**: para que el webhook `request_done` funcione (polling rápido). Sin ella, solo avanza el cron diario.

---

## Gotchas relevantes a integraciones

Ver `gotchas.md` para el detalle. Especialmente:

- **AEXG2** — privacidad: el PDF sale de la instancia hacia Odoo S.A.
- **AEXG3 / AEXG4** — auto_send por defecto + alias email consumen créditos sin acción del usuario.
- **AEXG11** — la creación de proveedor usa partner_autocomplete (servicio IAP aparte).
- **AEXG13** — webhook `auth='public'`; sin URL pública el polling cae en el cron diario.
- **AEXG16** — facturas y extractos comparten mixin/créditos pero NO endpoint ni versión.
- **AEXG17** — las notificaciones las da `iap_mail`, no el `iap` base.

---

## v19

Cambios estructurales relevantes a integraciones (detalle completo en `v19_changes.md`):

- **Nuevo submodelo `extract.mixin.with.words`** (`iap_extract/models/extract_mixin_with_words.py`): `account.move` hereda de él en lugar de `extract.mixin`. Mueve `extract_attachment_id` + datos de cajas (`extracted_words`, `extracted_numbers`, `extracted_dates` como `Json`) a este nivel. El modelo `account.invoice_extract.words` y `extract_word_ids` **desaparecen**.
- **Envío en postcommit** (`extract_mixin.py` v19, `_send_batch_for_digitization`): los documentos se mandan tras el commit, con cursor nuevo y commit por documento, para no perder créditos IAP si un rollback resetea el estado.
- **`res.partner.is_created_by_ocr`** + índice único `(name, is_created_by_ocr)` (`account_invoice_extract/models/res_partner.py`): evita que el OCR cree partners duplicados en transacciones concurrentes (`ConcurrencyError` → retry).
- **Soporte de recibos** (`include_receipts=True` en todas las comprobaciones `is_invoice/is_purchase_document/is_sale_document`): el OCR ahora maneja `in_receipt`/`out_receipt` además de facturas/notas de crédito.
- **Bridge EDI reescrito**: `_get_edi_decoder` devuelve dict `{decoder, priority}` y prioriza por tipo de fichero (PDF 10, JPG/PNG 5) vía `_get_import_file_type`.
