<!-- verified: v18.0 @ 2026-06 (documents EE base + bridges account/sign/spreadsheet/project/fsm + OCR chain iap_extract/account_extract; anclas verificadas vs fuente) -->
# Documents (`documents`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (DOG*); cambios v19 en [`v19_changes.md`](v19_changes.md).

## Módulo base `documents`

**DOCAP1. ¿Capturar email entrante como documento (alias → workspace)?** — **NATIVO** · baja · `documents_document.py::message_new` (`documents.document` hereda `mail.alias.mixin`; folder con `alias_name`/`alias_tag_ids`, `_alias_get_creation_values` fija `alias_defaults={'folder_id': self.id}`). Cada carpeta puede tener alias; el correo entrante crea un documento ahí y aplica sus tags.

**DOCAP2. ¿Acciones/flujos automáticos sobre documentos (workspace actions)?** — **NATIVO** · media · `ir_actions_server.py::action_open_documents_server_action_view` + `data/ir_actions_server_data.xml` (p.ej. "Send To Finance", "Create Activity", "Remove all tags"). Son `ir.actions.server` sobre `documents.document`, embebibles en la carpeta vía `documents_document.py::action_folder_embed_action`; incluyen mover, etiquetar, crear actividad o código Python. "Crear factura"/"pedir firma" NO están en el base — los aportan los puentes (ver DOCAP11/DOCAP13).

**DOCAP3. ¿Versionado de ficheros (subir nueva versión con histórico)?** — **NATIVO** · baja · `documents.document.previous_attachment_ids` + `documents_document.py::action_delete_from_history`. Al reemplazar el adjunto, el anterior pasa al histórico, restaurable o eliminable.

**DOCAP4. ¿Solicitar un documento a alguien (request/placeholder a subir)?** — **NATIVO** · baja · `documents_request_wizard.py::request_document` (`documents.request_wizard`) + `documents.document.requestee_partner_id`/`request_activity_id`. Crea un documento placeholder con actividad `upload_file` para el destinatario; el filtro "Requested Documents" los lista.

**DOCAP5. ¿Carpetas/workspaces anidados?** — **NATIVO** · baja · `documents.document._parent_name='folder_id'` + `_parent_store=True` (folder_id self-M2O, `children_ids` O2M, constraint anti-bucle). Las carpetas son el propio modelo `documents.document` con `type='folder'`; jerarquía recursiva por `parent_path`.

**DOCAP6. ¿Tags/facetas y reglas automáticas de etiquetado?** — **CONFIG** · media · modelo `documents.tag` (`documents_tag.py`); auto-tagging por `documents.document.alias_tag_ids` (vía email) o server actions `object_write` sobre `tag_ids`. Tags planos: en v18 NO hay modelo de "facet"/categoría ni motor de reglas dedicado — el etiquetado automático se monta con alias o acciones de servidor.

**DOCAP7. ¿Buscar dentro del contenido del documento (full-text)?** — **NATIVO** · baja · `documents.document.index_content` (`related='attachment_id.index_content'`), expuesto en la search view `document_view_search`. Busca sobre el texto indexado por `ir.attachment` (depende del indexado del core). Para PDF escaneados necesita OCR (ver DOCAP12).

**DOCAP8. ¿Split / merge de PDF desde la interfaz?** — **NATIVO** · baja · controlador `/documents/pdf_split` (`controllers/documents.py::pdf_split`) → `documents_document.py::_pdf_split` → `ir_attachment.py::_pdf_split`. La herramienta de la UI separa/combina páginas generando nuevos documentos.

**DOCAP9. ¿Dónde se almacenan los ficheros y hay limpieza automática?** — **NATIVO** · baja · `documents.document.attachment_id` → `ir.attachment` (filestore/DB estándar); papelera con `documents_document.py::_gc_clear_bin` (`@api.autovacuum`) y `documents.deletion_delay` (default 30 días, `get_deletion_delay`). Los archivados se purgan tras el delay por el autovacuum del cron base.

**DOCAP10. ¿Generar un documento desde un registro de Odoo (adjuntar informe a un workspace)?** — **NATIVO** · media · `documents.mixin` (`documents_mixin.py::_get_document_vals`/`_get_document_folder`) + `ir_attachment.py::_create_document`. Cualquier modelo que herede `documents.mixin` y defina folder/owner/tags genera automáticamente un documento al adjuntarle un fichero (p.ej. el PDF de un informe).

## Puentes (módulos `documents_*` + cadena OCR)

**DOCAP11. ¿Convertir un PDF en factura de proveedor directamente?** — **NATIVO** (módulo `documents_account`) · baja · `documents_account/models/documents_document.py::account_create_account_move`. La acción de workspace "Create Vendor Bill" crea un `account.move` tipo `in_invoice` desde el adjunto vía `journal._create_document_from_attachment`. (También Customer Invoice/Credit Note/Vendor Receipt/Misc Entry/Import Bank Statement.)

**DOCAP12. ¿OCR/digitalización automática del PDF para rellenar la factura?** — **CONFIG** (servicio IAP de pago) · media · `iap_extract/models/extract_mixin.py::_contact_iap_extract` (cadena `iap_extract`→`account_extract`→`account_invoice_extract`). NO lo aporta `documents_account`: el OCR es un servicio IAP que requiere crédito/contrato (estado `not_enough_credit` en `extract_mixin.extract_state`).

**DOCAP13. ¿Pedir firma electrónica desde un documento del workspace?** — **NATIVO** (módulo `documents_sign`) · baja · `documents_sign/models/documents_document.py::document_sign_create_sign_template_x`. La acción "Sign" crea un `sign.template` desde el adjunto PDF y abre el flujo de firma; lanza `UserError` si el mimetype no es PDF. Ver [[sign]].

**DOCAP14. ¿Crear una hoja de cálculo nativa dentro de Documents?** — **NATIVO** (módulo `documents_spreadsheet`) · baja · `documents_spreadsheet/models/documents_document.py::action_open_new_spreadsheet`. Crea un `documents.document` con `handler='spreadsheet'` y abre el editor o-spreadsheet; depende de `spreadsheet_edition`. (Se invoca por acción cliente JS, NO registra `ir.actions.server` en data.) Ver [[spreadsheet]].

**DOCAP15. ¿Vincular documentos a proyecto/tareas, con workspace automático?** — **NATIVO** (módulo `documents_project`) · baja · `documents_project/models/project_project.py::_create_missing_folders`; campos `project.project.documents_folder_id` y `project.task.documents_folder_id`. Con `use_documents=True` el proyecto crea su carpeta al `create`; las tareas heredan la carpeta y sus adjuntos se añaden como documentos. Ver [[project]].

**DOCAP16. ¿Documents integrado con Field Service (FSM)?** — **NO (el puente lo DESACTIVA)** · baja · `documents_fsm/models/project_project.py::_compute_use_documents` (`use_documents = not is_fsm`). El puente `documents_fsm` no añade flujos de Documents para FSM; su única función es desactivar Documents por defecto en proyectos FSM. Ver [[fsm]].
