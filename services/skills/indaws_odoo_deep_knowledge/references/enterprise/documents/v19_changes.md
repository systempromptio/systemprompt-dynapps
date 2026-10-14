# Documents — v18 → v19 diff

Source: `diff -rq odoo_enterprise/18/documents odoo_enterprise/19/documents`. Significant additions: virtual folders, async tracking, wizard-driven sharing, embedded actions integration.

## Major new features

### 1. Virtual folder identifiers (`user_folder_id`)

New computed Char field on `documents.document` (line 139):
```python
user_folder_id = fields.Char(compute='_compute_user_folder_id', search='_search_user_folder_id')
```

Returns one of:
- `"MY"` — root of current user's drive (owner_id == user, no parent).
- `"COMPANY"` — root, owned by OdooBot, visible to user's companies.
- `"SHARED"` — root, owned by someone else but accessible.
- `"TRASH"` — `active=False`.
- `"RECENT"` — search-only, returns recently accessed.
- `str(folder.id)` — explicit folder.

Impact: replaces ad-hoc XPath/domain filters; webclient sidebar uses these constants. Custom code reading `folder_id` keeps working but URLs and UI patches now reference `user_folder_id`.

### 2. New transient wizards

- `documents.operation` (`wizard/documents_operation.py`) — unified move / copy / shortcut / "add as document". Replaces a handful of v18 inline actions. Reads `user_folder_id`-style destination.
- `documents.sharing` + `documents.sharing.access` (`wizard/documents_sharing.py`) — modal sharing UI: list of partners with role + expiration, plus access_internal/access_via_link controls in one form. v18 had partial inline edits.

Impact: any v18 custom action button doing move/copy/share inline should be replaced with a launcher for the new wizard.

### 3. `documents.access.tracking` (async audit)

New model (`models/documents_access_tracking.py`):
```python
class DocumentsAccessTracking(models.Model):
    _name = 'documents.access.tracking'
    changes = fields.Json(...)
    documents = fields.Json(...)
    user_id = fields.Many2one('res.users')
```

Workflow:
1. Application code calls `_create_access_tracking({document_id: changes_dict})`.
2. Tracking rows batched by `frozendict(changes)` with `documents.tracking_batch_size` config (default 500).
3. `ir_cron_documents_access_tracking` cron triggered immediately, processes rows, generates the actual tracking entries (mail.tracking.value-style).

Impact: gives audit-grade access logs without slowing user flows. Custom code that previously hooked into write() to log changes can use this batched mechanism. New file `data/ir_cron_data.xml`.

### 4. `ir.embedded.actions` integration

New file `models/ir_embedded_actions.py` extends `ir.embedded.actions`:
- Folder-pinned actions require `write` access on that folder.
- Validation runs on create + write (before AND after super to prevent race).
- New view `views/ir_actions_views.xml` integrates the editing UI.

v18 had pin-to-folder via `is_pinned_folder` Boolean (TODO: removed). v19 elevates to embedded actions.

### 5. Mail-alias-related views

New files:
- `data/mail_alias_data.xml` — bootstrap aliases.
- `views/mail_compose_message_views.xml` — compose messages targeting documents.
- `views/mail_scheduled_message_views.xml` — schedule messages from documents.

### 6. Editor integration

New asset bundle entry: `documents/static/src/editor/**/*`. Documents can now be embedded in the rich text editor (Knowledge integration becomes deeper).

### 7. New demo files

- `demo/files/Acme_Insurance_Contract_Structured.pdf`
- `demo/files/Acme_NDA.pdf`
- `demo/files/email-as-invoice.html`
- `demo/files/Mails_inbox.pdf`

Replace the old `data/files/` set; demos showcase OCR, contracts, email-to-document.

## Constraint refactor

`_sql_constraints` → `models.Constraint` decorator-style (same as other modules):
```python
_attachment_unique = models.Constraint('unique (attachment_id)', "...")
_document_token_unique = models.Constraint('unique (document_token)', "...")
_folder_id_not_id = models.Constraint('check(folder_id <> id)', "...")
_shortcut_document_id_not_id = models.Constraint('check(shortcut_document_id <> id)', "...")
```

## Manifest changes

```diff
+ 'data/mail_alias_data.xml'
+ 'data/ir_cron_data.xml'
+ 'views/documents_templates_thumbnails.xml'
+ 'views/ir_actions_views.xml'
+ 'views/mail_compose_message_views.xml'
+ 'views/mail_scheduled_message_views.xml'
+ 'wizard/documents_operation_views.xml'
+ 'wizard/documents_sharing_views.xml'
- 'data/ir_actions_server_data.xml'
+ 'author': 'Odoo S.A.'
```

## Asset changes

```diff
- 'documents/static/src/model/**/*'
+ 'documents/static/src/editor/**/*'
+ 'documents/static/src/attachments/attachment_list_patch.{js,xml}'
+ 'documents/static/src/attachments/document_file_viewer.scss'
```

The legacy `model/**` bundle was a placeholder; v19 removed it.

## Removed

- `data/files/` directory — moved to `demo/files/`.
- `data/ir_actions_server_data.xml` — server-action defaults migrated to `ir_actions_views.xml` (declarative views).
- `is_pinned_folder` field marked TODO-remove in v18; replaced by embedded actions in v19.
- `documents/static/src/model/**` bundle.

## Behavioural notes

- New shortcut owner field: `shortcut_document_owner_id = fields.Many2one('res.users', related='shortcut_document_id.owner_id', store=True)`. Lets you filter shortcuts by source owner without joining.
- v19 `_compute_user_permission` keeps the same logic but adds the embedded actions check.
- The TRASH "folder" cannot be searched as a parent (`_search_user_folder_id` raises UserError). v18 search by `active=False` still works.

## Migration cookbook

1. Replace `_sql_constraints` with `models.Constraint`.
2. If you have customizations referencing `is_pinned_folder`, port to `ir.embedded.actions` with `parent_res_model='documents.document'`.
3. Custom move/copy/share buttons → launcher for `documents.operation`/`documents.sharing` wizards.
4. To enable async access tracking: install + set `documents.tracking_batch_size`. Hook into business logic via `_create_access_tracking()`.
5. Update assets manifest if you patched `documents/static/src/model/**` (no longer exists).
6. URL-based filtering using `folder_id` works; consider migrating to `user_folder_id` for the new sidebar constants.
7. v19 mail compose views require any module that overrides documents `mail.compose.message` action button to extend the new view.
