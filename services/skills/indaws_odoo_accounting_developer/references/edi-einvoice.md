# EDI / e-factura — `account_edi`, builders UBL/CII y contexto España

> Las citas `file::symbol` son recibos: el símbolo manda sobre el número de línea (los números derivan). Snapshot CE v18 y v19 verificado 2026-07 contra fuente local (resolver vía `references/shared/odoo-source.md`; NO asumir rutas de máquina). Lo NO verificado se marca explícitamente; no se ha silenciado nada.

---

## 1. Mapa honesto del paisaje EDI: v18 vs v19

Hay **dos frameworks conviviendo**, y el legacy ya NO es el camino principal:

- **Legacy `account.edi.format` / `account.edi.document`** (módulo `account_edi`): sigue existiendo e `installable=True` en v18 y v19 (`account_edi/__manifest__.py`). **NO es dependencia de `account`** (grep en `account/__manifest__.py` no lo devuelve en ninguna versión) → es opcional, solo lo instalan las localizaciones que lo necesitan. En España **solo lo usa `l10n_es_edi_sii`**.
- **Moderno: `account.move.send` + builders `account.edi.xml.*` + attachment en el propio move**. Es la ruta por defecto para e-facturas UBL/CII/Factur-X y para los formatos españoles nuevos (Facturae, TicketBAI, Veri*factu). Ninguno de esos tres usa `account.edi.format`.

Entre v18 y v19 la **esencia es idéntica** en ambos frameworks: los cambios son modernización ORM (ver §2) y reubicación de un hook de import (ver §3). La lógica de negocio no cambia.

**Corrección de memoria habitual:** en v18/v19 `account.edi.format` está muy adelgazado. **NO existen `_post_invoice_edi` ni `_cancel_invoice_edi`** (eran de v14–v15). No los busques.

---

## 2. Framework legacy: `account.edi.format` / `account.edi.document`

### Hooks "to override" que quedan en `AccountEdiFormat`
`account_edi/models/account_edi_format.py::AccountEdiFormat`:

- **`_get_move_applicability(self, move)`** — núcleo del framework. Devuelve un dict `{post, cancel, post_batching, cancel_batching, edi_content}` mapeando a callables. Devolver `None`/falsy = el formato no aplica a ese move.
- **`_needs_web_services(self)`** — default `False`; si `True`, activa el cron `account_edi.ir_cron_edi_network` (procesamiento asíncrono).
- `_is_compatible_with_journal`, `_is_enabled_by_default_on_journal`.
- **`_check_move_configuration(self, move)`** — devuelve **lista de strings de error** (`AccountEdiFormat._check_move_configuration`).
- `_prepare_invoice_report`, `_format_error_message`.

### Procesamiento en `AccountEdiDocument`
`account_edi/models/account_edi_document.py::AccountEdiDocument`:

- Campos stored: `move_id`, `edi_format_id`, `attachment_id` (`groups='base.group_system'`), `state` (`to_send`/`sent`/`to_cancel`/`cancelled`), `error` (Html), `blocking_level` (`info`/`warning`/`error`).
- `_prepare_jobs` agrupa por `(edi_format, state, company, batching_key)`.
- **`_process_job`** (`AccountEdiDocument._process_job`) llama a `method_to_call` (el callable de `_get_move_applicability`) y postprocesa `edi_result`: dict `{move: {'success', 'attachment', 'error', 'blocking_level'}}`.
- `_process_documents_no_web_services` (síncrono, al postear) y `_process_documents_web_services` + **`_cron_process_documents_web_services(job_count)`** (asíncrono vía cron, `AccountEdiDocument._cron_process_documents_web_services`).
- `_compute_edi_content` usa `_get_move_applicability(...)['edi_content'](move)`.

### Diff v18→v19 (modernización ORM, **con un cambio de bloqueo que NO es cosmético**)
- `_sql_constraints = [...]` → `models.Constraint(...)` (`_unique_code`, `_unique_edi_document_by_move_by_format`).
- `edi_content = fields.Binary(compute=..., compute_sudo=True)` en v18 → sin `compute_sudo` en v19.
- **Locking: cambia la primitiva, no solo la sintaxis.** v18 hace
  `self._cr.execute('… FOR UPDATE NOWAIT')` **dentro de un `with self.env.cr.savepoint(flush=False)`**
  y captura `psycopg2.errors.LockNotAvailable`; v19 llama `documents.lock_for_update()` **sin
  savepoint** y captura `LockError` (excepcion **nueva en v19**, `odoo/exceptions.py:109`, `LockError`). El motivo
  lo dice el propio ORM en un comentario: *"Use SKIP LOCKED instead of NOWAIT because the later
  aborts the transaction and we do not want to use SAVEPOINTS"* (`odoo/orm/models.py:5581-5582`, `BaseModel.lock_for_update`).
  O sea **`NOWAIT` → `SKIP LOCKED`**: en v18 Postgres aborta y el savepoint revierte; en v19 las
  filas bloqueadas **se saltan**, se comparan los conteos y se lanza `LockError` (`:5588-5589`) —
  con lo que **las filas que si se pudieron bloquear quedan bloqueadas en la transaccion**, algo que
  el rollback al savepoint de v18 deshacia. Si extiendes `_process_documents_web_services`, esa es
  la diferencia que importa.
- `self._context` → `self.env.context`.

---

## 3. Camino moderno: `account.move.send` + builders `account.edi.xml.*`

El motor es el servicio **`account.move.send`** (`AbstractModel`, `_name='account.move.send'`, `account/models/account_move_send.py`) — backend de "Send & Print". Hooks que extiende un dev:

- **`_hook_invoice_document_before_pdf_report_render(self, invoice, invoice_data)`** — en el fichero de `account` es no-op (`return`); aquí se genera el XML (`account/models/account_move_send.py::AccountMoveSend._hook_invoice_document_before_pdf_report_render`).
  **PERO en una instalación real NO llega vacío, y esto importa al extender.** `account_edi_ubl_cii` declara **`auto_install: True`** (`account_edi_ubl_cii/__manifest__.py:40` v18 / `account_edi_ubl_cii/__manifest__.py:39` v19) con `depends: ['account']`: instalar `account` lo instala también, y ese módulo **ya overridea el hook** (`account_edi_ubl_cii/models/account_move_send.py:114-118`, `AccountMoveSend._hook_invoice_document_before_pdf_report_render`). Medido en v18 y v19 sobre una base con solo `account`: llamar al hook con un `invoice_data` vacío **no devuelve `None`**, lanza `KeyError: 'invoice_edi_format'` — o sea que hay alguien delante.
  Consecuencia: overridearlo **sin `super()`** no es «pisar un no-op», es **apagar la generación de UBL/CII en silencio** — nadie lanza nada, simplemente deja de adjuntarse el XML. Y el que lo escribió leyó, con razón, que la base no hacía nada.
- `_hook_invoice_document_after_pdf_report_render(self, invoice, invoice_data)` — post-proceso (incrustar PDF en XML, o Factur-X en PDF).
- `_call_web_service_before_invoice_pdf_render(self, invoices_data)` / `_call_web_service_after_invoice_pdf_render` — formatos que llaman a una AEAT / web service (`AccountMoveSend._call_web_service_before_invoice_pdf_render`).
- `_link_invoice_documents(self, invoices_data)` — crea los `ir.attachment` definitivos.
- **`_get_all_extra_edis()`** — registro de EDIs "extra" aplicables (mecanismo `extra_edis`) (`AccountMoveSend._get_all_extra_edis`).
- `_get_invoice_extra_attachments`, `_get_placeholder_mail_attachments_data`, `_get_alerts`.

### Selección del builder por partner
`res.partner._get_edi_builder(invoice_edi_format)` (`account_edi_ubl_cii/models/res_partner.py::ResPartner._get_edi_builder`) devuelve el modelo `account.edi.xml.*` según formato: `ubl_de`→xrechnung, `cii`→facturx/zugferd, `ubl_a_nz`, `ubl_nl`→nlcius, `ubl_bis3`, `ubl_sg`.

El selector **`invoice_edi_format` vive en base `account`** (`account/models/partner.py::ResPartner.invoice_edi_format`): `selection=[]` "to extend", `company_dependent`, con compute `_compute_invoice_edi_format`/inverse. `account_edi_ubl_cii` lo extiende (`account_edi_ubl_cii/models/res_partner.py::ResPartner.invoice_edi_format`) con `selection_add=[facturx, ubl_bis3, zugferd, xrechnung, nlcius, ubl_a_nz, ubl_sg]`.

### Jerarquía de builders (todos AbstractModel)
`account.edi.common` (`account_edi_common.py`) → `account.edi.ubl` (`account_edi_ubl.py`) → `account.edi.xml.ubl_20` → `ubl_21` → `ubl_bis3`/`ubl_nl`/`ubl_de`/`ubl_a_nz`/`ubl_sg`/`ubl_efff`/`ubl_xrechnung`. En paralelo `account.edi.xml.cii` (Factur-X/ZUGFeRD, `account_edi_xml_cii_facturx.py`).

### El hook real de generación
`account.edi.xml.ubl_20._export_invoice(self, invoice, convert_fixed_taxes=True)` (`account_edi_ubl_cii/models/account_edi_xml_ubl_20.py::_export_invoice`) → devuelve la tupla **`(xml_bytes, set(errors))`**:

```python
vals = self.with_context(...)._export_invoice_vals(invoice.with_context(lang=...))
errors = [c for c in self._export_invoice_constraints(invoice, vals).values() if c]
xml_content = self.env['ir.qweb']._render(vals['main_template'], vals)
return etree.tostring(cleanup_xml_node(xml_content), ...), set(errors)
```

Los **tres métodos que se override** para un formato:
- `_export_invoice_vals(invoice)` — construye el dict de valores.
- `_export_invoice_constraints(invoice, vals)` — validación → **dict de mensajes de error** (`account_edi_xml_ubl_20.py::_export_invoice_constraints`).
- `_export_invoice_filename(invoice)`.

CII tiene su propio `_export_invoice` (`account_edi_xml_cii_facturx.py::_export_invoice`).

### Import
`account.edi.common._import_invoice_ubl_cii(self, invoice, file_data, new=False)` (`account_edi_ubl_cii/models/account_edi_common.py::_import_invoice_ubl_cii`, ~línea 436). El dispatcher es `account.move._get_edi_decoder` que, vía `_get_ubl_cii_builder_from_xml_tree(tree)` (mira `CustomizationID`/`UBLVersionID`/tag CII, `account_edi_ubl_cii/models/account_move.py::AccountMove._get_ubl_cii_builder_from_xml_tree`), devuelve el callable `builder._import_invoice_ubl_cii`. Rellenado en `_import_fill_invoice` / `_import_retrieve_partner_vals`. **NO verificado:** el detalle del import CII (`account.edi.xml.cii._import_invoice`) — solo se leyó su `_export_invoice`.

---

## 4. Dónde se adjunta el XML y cómo afloran los errores

En el move hay dos campos dedicados (`account_edi_ubl_cii/models/account_move.py::AccountMove`):
- `ubl_cii_xml_file = fields.Binary(attachment=True, ...)` (contenido) — `AccountMove.ubl_cii_xml_file`.
- `ubl_cii_xml_id = fields.Many2one('ir.attachment', compute=_compute_linked_attachment_id)` (el adjunto).

Flujo de adjuntado (`account_edi_ubl_cii/models/account_move_send.py::AccountMoveSend`):
1. **`_hook_invoice_document_before_pdf_report_render`**: si `invoice._need_ubl_cii_xml(fmt)` (`AccountMove._need_ubl_cii_xml`), llama `builder._export_invoice(invoice)`.
   - Si `errors` → `invoice_data['error'] = {'error_title':…, 'errors':…}` + `invoice_data['error_but_continue']=True`.
   - Si OK → `invoice_data['ubl_cii_xml_attachment_values'] = {'name','raw','mimetype':'application/xml','res_model':'account.move','res_id':invoice.id,'res_field':'ubl_cii_xml_file'}`.
2. **`_hook_invoice_document_after_pdf_report_render`**: incrusta el PDF dentro del XML (`_postprocess_invoice_ubl_xml`) y/o incrusta Factur-X dentro del PDF (siempre genera un Factur-X silencioso para inter-portabilidad).
3. **`_link_invoice_documents`**: `self.env['ir.attachment'].with_user(SUPERUSER_ID).create(attachments_vals)` y luego `invalidate_recordset(['ubl_cii_xml_id','ubl_cii_xml_file'])`.

Los errores del framework **legacy (SII)** afloran distinto: en `account.edi.document.error` (Html) + `blocking_level`, visibles en el move vía `edi_document_ids`.

### Diff v18→v19 en la ruta moderna
- **`_get_edi_decoder` se movió** en base `account`: v18 en `account/models/account_move.py::AccountMove._get_edi_decoder` (~línea 4350); v19 en el nuevo mixin `account/models/account_document_import_mixin.py::_get_edi_decoder` (~línea 370). El override en `account_edi_ubl_cii/models/account_move.py::AccountMove._get_edi_decoder` sigue existiendo en ambas.
- `res_partner` v19 añade `sanitize_peppol_endpoint()` (`account_edi_ubl_cii/models/res_partner.py::sanitize_peppol_endpoint`), `_get_ubl_cii_edi_format()`, validación de endpoint por EAS (`PEPPOL_ENDPOINT_INVALID_CHARS_RE_BY_EAS`) y `_get_peppol_endpoint_value(…, eas)` (firma nueva con `eas`).
- `account_move_send` (ubl_cii) v19: `_get_document_type_code_vals` → renombrado `_get_document_type_code_node`; usa `dict_to_xml` (`odoo.addons.account.tools`); `SUPPORTED_FILE_TYPES` importado de `account_edi_common` (no de `account.models.ir_attachment`); `_postprocess_invoice_ubl_xml` reescrito con anclas por tipo (Invoice/CreditNote/DebitNote); `_get_placeholder_mail_attachments_data` gana kwarg `pdf_report`.
- `account_move.py` (ubl_cii) v18 ya tiene `_export_invoice_new`/`_get_document_template`/`_get_document_nsmap` → arquitectura "dict_to_xml" en transición; en v19 consolidada.
- **NO verificado:** la versión saas exacta en que se removió `_get_document_type_code_vals` (el comentario en fuente dice "removed in saas-18.4", tomado del comentario, no contrastado con changelog).

---

## 5. España — módulos `l10n_es_edi_*` (CE v18 y v19)

**Presentes** en CE `addons/` (set idéntico en v18 y v19):
`l10n_es_edi_facturae`, `l10n_es_edi_sii`, `l10n_es_edi_tbai`, `l10n_es_edi_tbai_pos`, `l10n_es_edi_verifactu`, `l10n_es_edi_verifactu_pos`.

- **NO existe** un `l10n_es_facturae` viejo separado — es `l10n_es_edi_facturae`.
- En **EE** (v18 y v19) NO hay ningún `l10n_es_edi_*` (solo `l10n_es_reports*`, `l10n_es_intrastat` en v19, etc.). Los EDI españoles son **CE**.
- **NO verificado (OCA):** un `grep l10n_es_*` en la fuente OCA v18 no devolvió módulos EDI → no se ha verificado que existan `l10n_es_edi` en OCA en este entorno.

### `l10n_es_edi_sii` — ÚNICO que usa el framework legacy
`depends=['certificate','l10n_es','account_edi']`. `models/account_edi_format.py`:
- **`_get_move_applicability(move)`** (`l10n_es_edi_sii/models/account_edi_format.py::_get_move_applicability`): si `code=='es_sii'` y `move.l10n_es_edi_is_required` → `{'post': self._l10n_es_edi_sii_post_invoices, 'post_batching': lambda inv:(inv.move_type, inv.l10n_es_edi_csv), 'edi_content': self._l10n_es_edi_sii_xml_invoice_content, 'cancel': self._l10n_es_edi_sii_cancel_invoices}`.
- **`_needs_web_services()`** → `code=='es_sii'` (cron, asíncrono).
- Web services por agencia: `_l10n_es_edi_web_service_{aeat,bizkaia,gipuzkoa,navarra}_vals`, `_l10n_es_edi_call_web_service_sign_common`.
- **SII usa JSON SOAP, no XML UBL**: `_l10n_es_edi_sii_xml_invoice_content` → `json.dumps(...)` (`l10n_es_edi_sii/models/account_edi_format.py::_l10n_es_edi_sii_xml_invoice_content`).

### `l10n_es_edi_facturae` — NO usa `account_edi`
`depends=['certificate','l10n_es']`. Ruta `account.move.send`:
- `models/account_move_send.py::_hook_invoice_document_before_pdf_report_render`: si aplica, `xml_content, errors = invoice._l10n_es_edi_facturae_render_facturae()` y guarda `invoice_data['l10n_es_edi_facturae_attachment_values']` con `res_field='l10n_es_edi_facturae_xml_file'`.
- Campos en move: `l10n_es_edi_facturae_xml_file` (Binary) / `l10n_es_edi_facturae_xml_id` (M2o attachment) (`l10n_es_edi_facturae/models/account_move.py::l10n_es_edi_facturae_xml_file`).
- Entry de generación: `account.move._l10n_es_edi_facturae_render_facturae()`. Habilitación: `_l10n_es_edi_facturae_get_default_enable()`.
- **NO verificado:** el cuerpo de `_l10n_es_edi_facturae_render_facturae` (firma XAdES) — confirmado que se invoca desde el hook, no se leyó la implementación.

### `l10n_es_edi_tbai` (TicketBAI, País Vasco) — NO usa `account_edi`
`depends=['l10n_es','certificate']`. Modelo documento dedicado **`l10n_es_edi_tbai.document`** (`models/l10n_es_edi_tbai_document.py`): `state` (`to_send`/…), `chain_index`, `is_cancel`, y:
- **`_post_to_web_service(values)`** → `_post_to_agency(env, is_sale)` (`l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py::_post_to_web_service`).
- **`_post_to_agency`** despacha por agencia: Araba/Gipuzkoa `_prepare_post_params_ar_gi`, Bizkaia `_prepare_post_params_bi` (`::_post_to_agency`).
- Enganche al envío: `models/account_move_send.py::_call_web_service_before_invoice_pdf_render` (mira `'es_tbai' in invoice_data['extra_edis']`), registrado vía `_get_all_extra_edis`.

### `l10n_es_edi_verifactu` (Veri*factu) — NO usa `account_edi`
`depends=['l10n_es','certificate']`. Modelo documento dedicado **`verifactu_document`** (`models/verifactu_document.py`). Enganche: `models/account_move_send.py::_call_web_service_before_invoice_pdf_render` (`'es_verifactu' in extra_edis`), `_is_es_verifactu_applicable`, `_get_all_extra_edis`, `_hook_if_errors`.
- **NO verificado:** los métodos internos de firma/hash/QR de `verifactu_document.py` — solo se verificó que el modelo existe y los hooks de `account_move_send`.

### Resumen España
| Formato | Framework | Enganche | Estado/cadena |
|---|---|---|---|
| **SII** | `account.edi.format` (legacy) + cron | `_get_move_applicability`, `_needs_web_services` | `account.edi.document` (JSON SOAP) |
| **Facturae** | `account.move.send` | `_hook_invoice_document_before_pdf_report_render` + render a Binary | campo `l10n_es_edi_facturae_xml_file` |
| **TicketBAI** | `account.move.send` + `extra_edis` | `_call_web_service_before_invoice_pdf_render` | `l10n_es_edi_tbai.document` |
| **Veri*factu** | `account.move.send` + `extra_edis` | `_call_web_service_before_invoice_pdf_render` | `verifactu_document` |

Ninguno de los tres nuevos (Facturae/TBai/Verifactu) usa `account.edi.format`.

- **NO verificado:** diff exhaustivo v18 vs v19 dentro de los módulos `l10n_es_*` — solo se comprobó presencia de ficheros, `depends` idénticos y hook Facturae idéntico; no se hizo diff línea a línea de cada módulo español.

---

## 6. Qué hook override según el caso (decisión rápida)

- **Formato XML "offline" (UBL/CII/Facturae) — v18 y v19:** override `account.edi.xml.<tuformato>._export_invoice_vals` + `_export_invoice_constraints` (validación → dict de errores) + `_export_invoice_filename`; declara el formato en `res.partner.invoice_edi_format` (`selection_add`) y en `res.partner._get_edi_builder`. El servicio `account.move.send._hook_invoice_document_before_pdf_report_render` invoca `builder._export_invoice(invoice)` → `(xml, errors)` y adjunta al move. Import: `_import_invoice_ubl_cii` / `_import_fill_invoice`.
- **Formato con web service / AEAT (estilo nuevo, TBai/Verifactu) — v18 y v19:** implementa `account.move.send._get_all_extra_edis` + `_is_<x>_applicable` + `_call_web_service_before_invoice_pdf_render` y un modelo documento propio (`*_document` con estado/cadena).
- **Formato con web service estilo legacy (SII):** `account.edi.format._get_move_applicability` (dict `post/cancel/edi_content`) + `_needs_web_services=True` + `_check_move_configuration`; el cron `account_edi.ir_cron_edi_network` procesa. **Patrón antiguo — para módulos nuevos Odoo ya no lo usa salvo SII heredado.**

**Dónde ve el usuario los errores:**
- Ruta moderna → `invoice_data['error']` en el Send&Print (se muestra en el wizard; no bloquea si `error_but_continue`).
- Ruta legacy → `account.edi.document.error` / `blocking_level` en el move.
