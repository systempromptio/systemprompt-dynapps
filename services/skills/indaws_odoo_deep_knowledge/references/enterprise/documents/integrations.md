# Documents — Cross-module integrations (Odoo 18.0/19.0 EE)

`documents` (OEEL-1) es un **hub de archivos**: por sí solo no hace OCR, ni firma, ni contabilidad. Toda la potencia viene de los **bridges** que cuelgan de `documents.mixin` y de carpetas-destino configuradas por compañía. Este fichero cubre solo los bridges; el modelo central está en [[core]] y las trampas en [[gotchas]].

Todos los módulos citados son **Enterprise (EE)** salvo `attachment_indexation` (CE, dependencia directa para full-text).

---

## 1. Catálogo de bridges (v18)

`ls -d odoo_enterprise/18/documents_*` → 21 bridges:

| Bridge | Depends | Qué conecta |
|--------|---------|-------------|
| `documents_account` | `documents`, `account_reports` | Carpeta → diario contable → factura/extracto (OCR vía `account_extract`) |
| `documents_account_peppol` | `documents_account`, `account_peppol` | Documentos entrantes Peppol → bandeja Documents |
| `documents_project` | `documents`, `project` | Carpeta por proyecto; adjuntos de tarea ↔ documento |
| `documents_project_sale` | `documents_project`, `sale_project` | Carpeta auto al confirmar SO con proyecto |
| `documents_project_sign` | `documents_project`, `documents_sign` | Firmar desde la carpeta del proyecto |
| `documents_sign` | `documents`, `sign` | Documento → `sign.template`/`sign.request` |
| `documents_hr` | `documents`, `hr` | `documents_hr_folder` por compañía; carpeta-empleado |
| `documents_hr_contract`/`_expense`/`_holidays`/`_payroll`/`_recruitment` | `documents_hr` + módulo HR | Contratos, gastos, ausencias, nóminas, CVs |
| `documents_l10n_*_hr_payroll` | `documents_hr_payroll` + l10n | OCR de nóminas localizadas (BE/CH/HK/KE) |
| `documents_fleet` | `documents`, `fleet` | Documentos de vehículo |
| `documents_fsm` | `documents`, `industry_fsm` | **Desactiva** Documents en proyectos FSM (solo ACL, no workflow) — ver [[fsm/integrations]] |
| `documents_product` | `documents`, `product` | Ficha-producto ↔ documento |
| `documents_spreadsheet` | `documents`, `spreadsheet_edition` | Hojas de cálculo como documentos |
| `documents_approvals` | `documents`, `approvals` | Adjuntos de solicitudes de aprobación |

`auto_install` en casi todos: si están instaladas ambas dependencias, el bridge se instala solo. Por eso instalar `documents` + `sign` activa silenciosamente `documents_sign`.

---

## 2. OCR / digitalización — el camino real

**`documents` NO incluye OCR.** El motor es la cadena IAP:

```
iap_extract (CE)          → extract.mixin (extract_state, _contact_iap_extract)
   └─ extract_mixin.py:39  extract_state ∈ {no_extract_requested, waiting_upload,
                              waiting_extraction, extract_not_ready, waiting_validation,
                              done, error_status, not_enough_credit, ...}
account_extract (EE)      → engancha extract.mixin a account.move
documents_account (EE)    → mapea carpeta→diario y llama al flujo account
```

Flujo de una factura subida a Documents:
1. Usuario suelta PDF en la carpeta contable (`documents.account.folder.setting` mapea `folder_id` → `journal_id`, `documents_account/models/documents_account_folder_setting.py:7`, `DocumentsFolderSetting._name`).
2. Acción `account_create_account_move` (`documents_account/models/documents_document.py:85`, `DocumentsDocument.account_create_account_move`) crea el `account.move` llamando a `journal.create_document_from_attachment` (`account/models/account_journal.py:984`, `AccountJournal._create_document_from_attachment`).
3. `account.move` hereda `extract.mixin`; al postear/cargar dispara `_contact_iap_extract` (`iap_extract/models/extract_mixin.py:100`, `ExtractMixin._contact_iap_extract`) que sube el adjunto al servidor IAP de Odoo.
4. Cron `check_all_status` (`iap_extract/models/extract_mixin.py:95`, `ExtractMixin.check_all_status`) recorre los registros y llama `_try_to_check_ocr_status` → `_check_ocr_status` (`:332`), que consulta el resultado al IAP (`get_result`); al recibirlo `extract_state='waiting_validation'` (`:339`) y rellena líneas/importes vía `_fill_document_with_results`. (El paso a `extract_state='done'` (`:122`) lo hace otro cron, `_cron_validate`, al validar/postear.)

Consecuencias:
- Sin créditos IAP (`extract_state='not_enough_credit'`) la factura se crea **vacía**, sin error visible en Documents.
- El OCR es del lado `account.move`, no del documento: el `documents.document` solo es el contenedor del adjunto inicial.
- `documents_l10n_*_hr_payroll` reutilizan el mismo `extract.mixin` para nóminas.

---

## 3. Sign (`documents_sign`)

`documents_sign/models/documents_document.py`:
- `document_sign_create_sign_template_x(create_model, folder_id)` (línea 9) convierte el documento en `sign.template` (modo `sign.template.new`) o lanza firma directa (`sign.template.direct`, exige `len(self)==1`, línea 12).
- Crea el `sign.template` con `create()` (línea 29) y abre la vista de plantilla; `go_to_custom_template(sign_directly_without_mail=...)` (línea 43) para firma inmediata.
- **GC**: `_get_gc_clear_bin_domain` (línea 47) **excluye** `res_model='sign.request'` del vaciado de papelera → los documentos generados por firmas NO se purgan con el autovacuum estándar (a diferencia del resto). Trampa de retención: las firmas se acumulan.
- `documents_project_sign` añade firmar directamente desde la carpeta del proyecto (depende de ambos: `documents_project`+`documents_sign`).

---

## 4. Project / HR — carpetas-destino por entidad

Patrón común: cada bridge añade un campo `*_folder_id` (M2O `documents.document`, `domain=[('type','=','folder')]`) y vuelca los adjuntos ahí.

**`documents_project`** (`models/project_project.py`):
- `use_documents` (Bool, default True, línea 15) y `documents_folder_id` (línea 16, dominio `type='folder'` + sin shortcut).
- `_archive_folder_on_projects_unlinked` (línea 29): al borrar todos los proyectos que comparten una carpeta, **archiva** la carpeta (no la borra).
- `_check_company_is_folders_company` (línea 37): la carpeta debe ser de la misma compañía que el proyecto, o sin compañía. Cruzar compañías → ValidationError.
- Adjuntos de `project.task` se reflejan como documentos vía `documents.mixin`.

**`documents_hr`** (`models/res_company.py`):
- `documents_hr_folder` (M2O, `check_company=True`, línea 12) computado con default `documents_hr.document_hr_folder` (línea 18). Carpeta-raíz de RRHH por compañía.
- `_get_used_folder_ids_domain` (línea 21) marca carpetas "en uso" para que no se puedan reasignar libremente.
- Submódulos (`_contract`, `_expense`, `_holidays`, `_payroll`, `_recruitment`) cuelgan documentos de cada modelo HR usando esta carpeta.

Trampa transversal: si borras o archivas la carpeta-destino de un bridge, los nuevos adjuntos dejan de crearse como documentos **sin error** (vuelven a ser meros `ir.attachment`).

---

## 5. Spreadsheet (`documents_spreadsheet`)

Bridge entre `documents` y `spreadsheet_edition` (EE). Ver detalle completo en [[functional/spreadsheet/gotchas]] (SPRGE*).

`documents_spreadsheet/models/`:
- `documents_document.py` — el documento tipo hoja-de-cálculo guarda los datos en `spreadsheet_binary_data` (res_field del adjunto).
- `spreadsheet_template.py`, `spreadsheet_contributor.py`, `spreadsheet_cell_thread.py` — plantillas, colaboradores (presencia) y comentarios de celda.
- `documents_access.py` (v18) / `ir_http.py` — el control de acceso de la hoja **reutiliza** el de Documents (`documents.access`): compartir la hoja = compartir el documento.

Implicación: la edición colaborativa (revisiones, snapshots cada 2h, GC de revisiones 365 días) la gobierna `spreadsheet_edition`, pero **el ACL es de Documents**. Un usuario con `documents.access` role=`view` sobre la hoja no puede editar aunque el editor de spreadsheet lo permita en abstracto.

---

## 6. Mapa de ecosistema

```
                          documents (hub, documents.mixin)
                                  │
   ┌───────────┬─────────────────┼──────────────┬───────────────┐
   │           │                 │              │               │
documents_account  documents_sign  documents_project  documents_hr  documents_spreadsheet
   │ (folder→journal) │ (sign.template) │ (project folder) │ (hr folder) │ (spreadsheet_edition ACL)
   │                  │                 │                  │
   └─ account_extract └─ sign           └─ documents_project_sale  └─ _contract/_expense/
        │  (IAP OCR)                        documents_project_sign      _holidays/_payroll/
        └─ iap_extract (extract.mixin)                                  _recruitment
                                                                            │
   documents_fsm ──╳── workflow   (solo desactiva docs en proyectos FSM)   └─ documents_l10n_*_hr_payroll
   documents_fleet / documents_product / documents_approvals                    (OCR nóminas localizadas)
   documents_account_peppol ── account_peppol (B2B entrantes)
```

---

## 7. Notas v19

- `documents_spreadsheet` añade `documents_spreadsheet_survey` (`=ODOO.SURVEY`) — ver [[functional/spreadsheet/v19_changes]].
- El control de acceso de todos los bridges se reescribe sobre el nuevo modelo `documents.access` + `documents.access.tracking` (audit async). Cualquier bridge que en v18 leyera campos de acceso directos debe adaptarse — ver [[v19_changes]].
- `documents_account` se beneficia de los nuevos asistentes `documents.operation`/`documents.sharing` para mover/compartir las facturas digitalizadas.
