<!-- verified: v18.0 @ 2026-06 -->
# Documents — Core Reference (Odoo 18.0/19.0 EE)

**Enterprise (EE), licencia OEEL-1.** No existe en Community.
Module path v18: `odoo_enterprise/18/documents/`
Module path v19: `odoo_enterprise/19/documents/`
Depends on `mail`, `digest`, `attachment_indexation`, `web`.

> **Ruptura de arquitectura v17 → v18.** Documents se reescribió por completo. Modelos de v17 **eliminados** en v18: `documents.folder` (carpetas eran un modelo propio → ahora son `documents.document` con `type='folder'`), `documents.facet` (facetas de tags → tags planos), `documents.share` (enlaces compartidos como modelo con token consteq → ahora el propio documento lleva `access_via_link`/`document_token`), `documents.workflow.rule` + `documents.workflow.action` (motor de reglas → ahora simples `ir.actions.server`). Quien venga de v17: `folder_id` ya NO apunta a `documents.folder`, apunta a otro `documents.document`. Ver bridges en [[integrations]] y diffs v18→v19 en [[v19_changes]].

## Models

| Model | File | Role |
|---|---|---|
| `documents.document` | models/documents_document.py (2397 LoC) | The main record: file, folder, URL or shortcut |
| `documents.access` | models/documents_access.py | Per-partner ACL (viewer/editor + expiration) |
| `documents.tag` | models/documents_tag.py | Flat tags (M2M with documents) |
| `documents.mixin` | models/documents_mixin.py | Abstract: auto-create document on attachment |
| `documents.unlink.mixin` | models/documents_unlink_mixin.py | On unlink → trash (active=False) related document |
| `documents.redirect` | models/documents_redirect.py | URL shortener for legacy share links |
| `ir.attachment` (inherit) | models/ir_attachment.py | adds `_pdf_split` helper |
| `mail.activity` (inherit) | models/mail_activity.py | request-document activities (`_action_done` cierra la solicitud) |
| `mail.activity.type` (inherit) | models/mail_activity_type.py | flagging activity types as document-related |
| `kpi.provider` (inherit) | models/kpi_provider.py | Documents KPIs for the digest |
| `documents.access.tracking` (v19 only) | models/documents_access_tracking.py | Async cron-batched audit log (NO existe en v18, verificado) |
| `ir.embedded.actions` | (consumido en v18; sin `_inherit`) | folder-pinned actions ya se LEEN en v18 (`_get_folder_embedded_actions`, `:602`); v19 añade el `_inherit` con `_check_documents_can_pin` |

## Document types

`documents.document.type` (Selection):
- `binary` — actual file backed by `attachment_id`.
- `url` — bookmark; `url` field + auto-fetched OG preview. `name`/`url_preview_image` are a stored compute (`_compute_name_and_preview`, `:307-321`) that fetches OG metadata over HTTP via `mail.tools.link_preview.get_link_preview_from_url` (one shared `requests.Session`, `timeout=3`, failures swallowed).
- `folder` — container; uses `_parent_store` (parent_path).

Plus `shortcut_document_id` (M2O ondelete=cascade) creates a "symlink" — a document whose data is sourced from another. `shortcut_ids` = inverse one2many.

## Folder hierarchy

`folder_id` (M2O `documents.document`, `domain=[('type','=','folder')]`) + `parent_path` (`_parent_store=True`). Single tree across companies; `company_id` per document determines visibility scope.

Root folders have `folder_id=False`. `folder_id` is `ondelete="set null"` (`documents/models/documents_document.py:131`, `class Document`) → deleting a folder orphans its children to root, it does NOT cascade-delete them.

`is_pinned_folder` (stored compute, `:141,295-304`) marks OdooBot-owned root folders and is referenced by the security rule `documents_document_write_base_rule`, but it is flagged **TODO: remove in master** and `toggle_is_pinned_folder` is already a no-op (`:1115-1117`). Folder-pinned ACTIONS use `ir.embedded.actions` — and that is already consumed in v18 (`_get_folder_embedded_actions`, `:602`), not a v19 novelty; v19 only adds the `_check_documents_can_pin` write-access enforcement.

## Access model

Three layers, evaluated in `_get_permission_without_token_multi()`:

1. **Owner** (`owner_id`): always `edit` (`:377-378`). Default owner = **current user** (`:97`), NOT OdooBot.
2. **`access_ids`** (`documents.access`): explicit per-partner role (`view`/`edit`) with optional `expiration_date`. The row is immutable in `partner_id`/`document_id` (`documents/models/documents_access.py:33-35`, `DocumentAccess.write`) — to re-target, delete & recreate. Expired rows are physically purged by `_gc_expired` autovacuum (`:40-42`). A row may have `role=NULL` if `last_access_date` is set (audit-only entry; CHECK constraint `:22`).
3. **`access_internal`** (Selection: `view`/`edit`/`none`): role granted to ALL internal users (`not user.share`). **Caveat:** a `documents.group_documents_manager` user is upgraded to `edit` on any `access_internal != 'none'` doc (`:413-416`), regardless of whether internal is set to `view`.
4. **`access_via_link`** (Selection: `view`/`edit`/`none`): role for anyone who has the URL.
5. **`is_access_via_link_hidden`** (Boolean): if True, `access_via_link` requires direct invite; if False, parent-folder access propagates ONE level (`:353-360`).

`user_permission` is the computed final permission (`_compute_user_permission`, `compute_sudo=True`, `@api.depends_context('uid','allowed_company_ids')`, `:327-331`). `documents.group_documents_system` bypasses everything (admin) but is restricted to non-disabled companies (`:332-342`).

## Tokens

- `document_token`: stable UUID v4 per document, base64-urlsafe; default strips trailing `==` (`:105-107`); `document_token_unique` constraint (`:178`).
- `access_token` (computed, `:108,190-193`): `f"{document_token}o{id or 0:x}"`. `access_url` (`:196-198`) = `<base>/odoo/documents/<quoted access_token>`.
- Portal sign-up flow uses HMAC over `(access.id, partner.id)` — `documents_access._get_member_signup_token()` (`:56-71`), validated with `consteq` (`:78`). Only available if `_get_signup_invitation_scope() == 'b2c'` and the partner has no user yet (`_is_signup_available`, `:48-54`) — otherwise `_get_signup_url` returns `''` (DOG49).

## Attachment & versioning

- `attachment_id` is unique per document (`attachment_unique` constraint, `:177`); `ondelete='cascade'` (`:47`) — deleting the attachment deletes the document.
- `previous_attachment_ids` — M2M to `ir.attachment` storing versions (`:69`). No automatic pruning (DOG20).
- "Replace file" assigns a new `attachment_id` and pushes the old one into `previous_attachment_ids` (`:2024`). Because `name` is a stored compute keyed on `attachment_id` (`:72,312`), replacing the file **recomputes `name` to the new file's name** (DOG21).
- `_pdf_split()` (`:2102-2104`) splits a multi-page PDF into N new documents using `ir.attachment._pdf_split`.

## Document creation flow

1. User uploads file via `/documents/upload` route.
2. `ir.attachment` created.
3. If `documents.mixin._check_create_documents()` returns True OR upload is direct, `documents.document` is created.
4. `_get_document_vals()` (`documents/models/documents_mixin.py:22-42`, `DocumentMixin._get_document_vals`) injects: folder, owner, tags, partner, access_rights. In the **mixin/bridge** path the default owner is **OdooBot** (`_get_document_owner` → `base.user_root`, `:60-67`) to avoid granting full access; in a **direct** `documents.document.create()` the owner default is the **current user** (`documents/models/documents_document.py:97`, `class Document`).
5. Owner-of-portal-user constraint: portal users (`owner_id.share`) can't own ROOT documents — `_check_portal_cant_own_root_document` (`:280-285`), only docs inside a folder.
6. Default `access_rights` from the mixin are the most restrictive: `access_via_link='none'`, `access_internal='none'`, `is_access_via_link_hidden=True` (`documents/models/documents_mixin.py:54-58`, `DocumentMixin._get_document_vals_access_rights`) — relies on folder-permission inheritance.

## Email-to-document (mail.alias)

`documents.document` inherits `mail.alias.mixin`. Folders (`type='folder'`, non-shortcut) accept inbound email via `alias_name`. `_check_alias` constraint (`:287-293`) blocks aliases on non-folder types AND on shortcuts-to-folders (DOG15). The built-in `inbox` alias is force-set on the Internal folder by `_add_inbox_alias_if_not_used` (`:2326-2334`, a stable-migration shim).

`message_new()` (`:1429-1457`) sets `active=False` and tags `with_context(document_message_new=True)`; `_message_post_after_hook` (`:1477-1513`) then creates one document PER attachment (DOG16). The flag bypasses the `no_document=True` guard normally forced by `message_post` (`:1475`) — see DOG17. The folder's `alias_tag_ids` (`:166`) are auto-applied to incoming docs.

## Trash & deletion

- "Archive" (`action_archive`, `:1740`) sets `active=False` and logs a chatter message with the deletion ETA (`:1758-1759`).
- `deletion_delay` config param `documents.deletion_delay` (default 30 days, `get_deletion_delay` `:1340-1341`; CHECK ≥ 0 in `documents/models/res_config_settings.py:13`, `class ResConfigSettings`).
- `_gc_clear_bin` (autovacuum, `:2314-2316`; ojo: el método se llama así, no "_gc_archived_documents", que no existe en la fuente) hard-deletes via `_get_gc_clear_bin_domain` (`:2319`): `active=False AND write_date <= now - deletion_delay`. The clock is on **write_date**, so any later write resets it.
- `documents.unlink.mixin.unlink()` (`documents/models/documents_unlink_mixin.py:11-26`, `DocumentUnlinkMixin.unlink`): deleting a source record keeps its document but archives it and re-targets to itself (`res_model='documents.document', res_id=self.id, active=False`).

## Tags

`documents.tag` (flat, no facets in v18+v19). `tag_name_unique` constraint on `name` GLOBALLY — **no `company_id` field** (`documents/models/documents_tag.py:19-27`, `class Tags`), so two companies cannot create same-named tags (DOG22). `_unlink_except_used_in_server_action` (`:45-50`) prevents deleting tags referenced by `ir.actions.server`, but checks ONLY tags that have an external_id (`_get_external_ids`) — UI-created tags without xmlid bypass the guard (DOG23). `name` is `translate=True`; `tooltip` is deprecated.

## Server-action workflows

`ir.actions.server` extended:
- `action_open_documents_server_action_view` opens a list filtered to `model_name='documents.document'`.
- The "workflow" is just a server action with `default_update_path='tag_ids'` — apply tags / move to folder / change owner / send email through the standard server-action engine.
- v19 adds `views/ir_actions_views.xml` for richer action editing UI.

## Activities

`mail.activity` extension (`mail_activity.py`): `request_activity_id` (M2O, `documents/models/documents_document.py:101`, `class Document`) on `documents.document` is the "request a document from someone" activity — ONE outstanding request per document (DOG24). The requestee receives a portal link; uploading marks the activity done.

- Created via `documents.request_wizard` (`wizard/documents_request_wizard.py`): if the requestee has a user, they get a time-boxed `documents.access` `edit`; if NOT, the doc's `access_via_link` is set to `edit` so the anonymous link can upload.
- On completion `_action_done` (`documents/models/mail_activity.py:31-52`, `MailActivity._action_done`): if `access_via_link == 'edit'` it is DOWNGRADED to `view`, and `requestee_partner_id`/`request_activity_id` are cleared. The upload runs under `no_document=True` to avoid a recursive document creation.
- `requestee_partner_id` (`:69`) may be a NON-internal partner (DOG25) — portal/public upload via token routes.

`mail.activity.type` (`mail_activity_type.py`) flag (`category == 'upload_file'`) tells which activity types create documents on completion. Chained `upload_file` → `upload_file` activities auto-spawn the next request document (`_prepare_next_activity_values`, `:14-29`).

## OCR / extraction

Standalone `documents` does NOT include OCR. Bridge modules:
- `documents_account` — invoice OCR via `account.move._extract_*` chain.
- `documents_l10n_be_hr_payroll` — payroll OCR.
- `documents_hr_recruitment` — CV upload to applicant.
- `documents_project` — drag to task.
- `documents_fleet`, `documents_sign`, `documents_spreadsheet`, `documents_product`, `documents_approvals`.
- `documents_fsm` — does NOT integrate; it DISABLES Documents on FSM projects (DOG41).

(`documents_helpdesk` does NOT exist in EE18.) Each bridge inherits `documents.mixin` on its model and overrides `_get_document_folder`/`_get_document_owner`/etc. Full bridge map → [[integrations]].

## Search panel & UI

- Custom search panel (`static/src/components/`) renders folder tree + tag chips + filters.
- `search_panel_select_range` override (`:2118`) exposes folders, tags, owners.
- `last_access_date_group` (`:169-174`) is a SQL-CTE-derived Selection (`3_day`/`2_week`/`1_month`/older, `:648-667`) for filtering, with its own `search` method (no stored value).

## Multi-company

`company_id` on `documents.document` (with `store=True, readonly=False, index=True`). `_compute_user_permission` (`:331-360`) and `_get_permission_without_token_multi` (`:366-419`) check, in order:
- **Archived company** (`document.company_id and not company_id.active`) → `none` for non-system users (`:347-348`).
- **Disabled company** (`is_user_company AND company_id not in env.companies`) → `none` (`:370-374`); the doc is hidden until the user switches companies. This short-circuits even the owner check.
- `company_id is False` (cross-company doc) → no company gate; visible subject to the access layers.
- `access_internal` only applies when `not document.company_id OR company_id in env.companies` (`:411`).
- System admins: `edit` on any doc whose company is active or not in their `company_ids`; `none` only on disabled-but-owned companies (`:332-342`).
- Re-companying a folder cascades the new `company_id` to all descendant docs + shortcuts via raw SQL (`_update_company`, `:1057-1093`) — see DOG26. It does NOT touch `attachment_id.company_id`.

## Permission matrix summary

| Source | Target | Result |
|---|---|---|
| Group `documents.group_documents_system` | any | `edit` |
| `owner_id == user` | document | `edit` |
| `documents.access` (partner=user.partner) role=`edit`/`view` (not expired) | document | role (or `access_via_link` if role=NULL) |
| `access_internal` != none, user is internal, `group_documents_manager` | document | `edit` (forced) |
| `access_internal` != none, user is internal (plain user) | document | `access_internal` role |
| `access_via_link` != none, user knows URL | document | role |
| Parent folder permission != none, link not hidden | document | falls back to `access_via_link` |
| Otherwise | document | `none` |

## Search index

`init()` (`:183-188`) creates a compound index `documents_document_res_model_res_id_idx` on `(res_model, res_id)` — the most common filters (every bridge queries by source record).

## Reports & batch ops

No QWeb PDF reports in core. v19 adds wizards `documents.operation` (move/copy/shortcut) and `documents.sharing` (modal-based sharing UI replacing v18 ad-hoc actions).
