# Enterprise Tools Gotchas — Odoo 18.0 Enterprise

<!-- verified: v18.0 @ 2026-06 (ETG33-ETG41 verified against odoo18/addons crm_iap_*; ETG1-32 Sign/VoIP/Data carried from prior pass — internals now own dedicated domains) -->

> **Note**: Sign (ETG1-10,23-26), VoIP (ETG11-12,27-28), Data Cleaning/Merge (ETG13-20,29-32) gotchas below are kept for convenience but the canonical, deeper versions live in `enterprise/sign`, `enterprise/voip`, `enterprise/data_tools`. The **unique** gotchas of this domain are the **Lead Intelligence / IAP** ones, ETG33-ETG41.

## Sign

**ETG1. Access tokens are security-critical — regenerated on cancel/reassign** — Tokens are UUID4, grant full signing access. Regenerated on: cancel, send refusal, signer reassignment. `access_via_link` tracks portal vs authenticated access. Impact: NEVER leak tokens in logs or URLs. _ES: El enlace de firma que envié dejó de funcionar tras cancelar o reasignar el firmante._

**ETG2. Multi-signer requires explicit ordering** — `mail_sent_order` determines sequential flow. Order=1 sends immediately. Next orders only after ALL current order signers complete. Refusing cancels ALL subsequent orders. Impact: parallel signing requires setting all items to same order (default 1). _ES: Los firmantes no reciben el documento a la vez; al siguiente no le llega hasta que firman todos los del turno anterior._

**ETG3. PDF attachment auto-copied if shared** — When sign.template created: if attachment already has res_model/res_id (attached elsewhere), it's COPIED. Otherwise, attachment's res_model/res_id set to template. Deleting template does NOT delete attachment. Impact: storage bloat from copied PDFs. _ES: Se duplican los PDF y ocupan espacio; al borrar la plantilla de firma el adjunto sigue ahí._

**ETG4. Sign item positions normalized 0-1** — All posX/posY/width/height are fractions of page (0.0 to 1.0). Pages are 1-indexed. Alignment only applies to text types. Impact: raw API field creation needs normalized coordinates. _ES: Los campos de firma salen descolocados en el PDF al crearlos por API con coordenadas sin normalizar._

**ETG5. Signature images are Base64 PNG with transparency fix** — Frame values = `data:image/png;base64,...`. Processed with PIL `_fix_image_transparency()`. PDF rendering uses ReportLab with custom TTF fonts. Impact: custom signature processing must handle PNG transparency. _ES: La imagen de la firma sale con fondo o transparencia rara en el PDF final._

**ETG6. SMS authentication is IAP credit-based** — Role `auth_method='sms'` requires SMS credits. `_send_no_credits_email` notifies creator if exhausted. No credits = signing fails unless `signed_without_extra_auth=True`. Impact: SMS auth silently breaks when credits run out. _ES: La firma con verificación por SMS deja de funcionar sin avisar cuando se agotan los créditos._

**ETG7. Requests auto-expire after 6 months** — Default validity = 6 months from create_date. Cron `_cron_reminder` auto-cancels expired. Shared documents (state='shared') have NO role restriction — any signer fills any field. Impact: long-lived requests expire silently. _ES: Las solicitudes de firma antiguas se cancelan solas a los seis meses sin previo aviso._

**ETG8. Shared documents bypass role restrictions** — `state='shared'` + single request_item (no partner). Anyone with link fills ANY field. Multi-role templates cannot be shared (ValidationError). Impact: shared mode removes all access control per role. _ES: En un documento compartido por enlace cualquiera puede rellenar cualquier campo, sin control por roles._

**ETG9. Template tag uniqueness is case-sensitive** — SQL UNIQUE constraint on name, not LOWER(name). "Invoice" and "invoice" = different tags. Impact: duplicate-looking tags in UI. _ES: Salen etiquetas duplicadas que parecen iguales (Factura/factura) en las plantillas de firma._

**ETG10. Signer reassignment has strict conditions** — Only if: state='sent' AND no prior signing AND role.change_authorized=True. Creates new access_token. Posts chatter message. Impact: can't reassign after partial signing starts. _ES: No me deja reasignar el firmante una vez que alguien ya ha empezado a firmar._

## VoIP

**ETG11. Phone matching is context-sensitive** — Short numbers (<3 digits): exact match. International (+, 00): normalized search. US "1" prefix: treated as +1. Partner auto-linked AFTER call state change. Impact: matching depends on number format and country context. _ES: El teléfono de la llamada no enlaza con el contacto correcto según el formato o el país del número._

**ETG12. User settings auto-created on VoIP field write** — Setting VoIP field on res.users auto-creates res.users.settings via inverse method. `_find_or_create_for_user()` is lazy. Impact: unexpected settings records created by field writes. _ES: Se crean registros de configuración de usuario inesperados al tocar campos de VoIP._

## Data Cleaning

**ETG13. SQL injection risk in cleaning actions** — `_clean_records` uses `SQL()` literals for composable expressions. Whitelisted actions only (trim|upper|lower|remove_accents|format_phone|replace_char). Impact: NEVER pass custom action strings. _ES: Al usar una acción de limpieza fuera de las permitidas da error en lugar de aplicarla._

**ETG14. Batch commits manual and mode-dependent** — `batch_commits=True` enables per-batch `env.cr.commit()`. Automatic=5K batches, manual=50K. Default False = single transaction, rollback on timeout. Impact: large datasets need explicit batch commit setup. _ES: Al limpiar muchos registros la operación se cae por timeout y no guarda nada._

**ETG15. Phone formatting requires country context** — format_phone action uses record's country_id or company fallback. Missing country = formatting error/skip. Impact: records without country may not get phone cleaned. _ES: El formateo de teléfonos no se aplica en contactos que no tienen país asignado._

## Data Merge

**ETG16. Similarity only compares char/text fields** — `similarity = matching_char_fields / total_char_fields`. Numeric, date, relational fields IGNORED. Records with no char fields = similarity 0. Impact: unexpected groupings for models with few text fields. _ES: La detección de duplicados agrupa registros de forma rara en modelos con pocos campos de texto._

**ETG17. FK redirect uses raw SQL — no constraint handling** — `_update_foreign_keys()` uses UPDATE SQL without cascade checks. ON DELETE RESTRICT violations logged but don't block merge. Impact: custom models with complex FK constraints may need custom `_merge_method()`. _ES: Tras fusionar registros quedan referencias rotas o incoherentes en modelos con relaciones complejas._

**ETG18. Requires PostgreSQL unaccent extension** — `find_duplicates()` uses `lower(unaccent(field))`. Crashes without extension. Impact: must run `CREATE EXTENSION IF NOT EXISTS unaccent` before using. _ES: La búsqueda de duplicados casca con un error de base de datos por faltar la extensión unaccent._

**ETG19. Master election extensible per model** — Default `_elect_method()` returns oldest record. Models can override `_elect_method(records)` staticmethod. Impact: custom master selection logic possible but must return record from input list. _ES: Al fusionar duplicados siempre elige el registro más antiguo como principal._

**ETG20. Cross-company dedup controlled by mix_by_company** — `mix_by_company=False` groups by company before search. Models WITHOUT company_id: all included. Company search uses LEFT JOIN + OR. Impact: cross-company merge can create data ownership issues. _ES: La fusión de duplicados mezcla registros de compañías distintas._

## Additional — Sign

**ETG23. Document integrity is a hash CHAIN, not a single hash** — `sign.log` chains SHA256 over `(previous_hash + str(body))`, seeding the first entry from the template PDF data (`sign/models/sign_log.py:98-116`, `SignLog._get_or_check_hash`); `_check_document_integrity()` (`:129-140`) walks the chain. Impact: a single tampered/deleted log row breaks every subsequent hash — you cannot "fix" one entry, the whole audit trail reads as compromised. This is detection-only (post-hoc), it does not block the write. _ES: Si se borra o altera un registro del historial de firma, todo el rastro de auditoría queda marcado como comprometido._

**ETG24. Reminder cron uses raw SQL interval + `last_reminder` defaulting to today** — `_cron_reminder` (`sign/models/sign_request.py:444-467`, `SignRequest._cron_reminder`) selects where `sr.last_reminder + sr.reminder * '1 day'::interval <= today`. `last_reminder` defaults to creation day. Impact: a request created and given a reminder interval on the same day can skip its first reminder window; reminders are also coupled to the same cron that auto-expires at 6 months. _ES: Una solicitud creada hoy con recordatorio no envía el primer aviso cuando tocaba._

**ETG25. Frame hash binds signer email to DB uuid** — `frame_hash = sha256((signer_email + db_uuid).encode()).hexdigest()` (`sign/models/sign_request.py:918-924`, `SignRequestItem.partner_id`). Impact: copying a signed PDF's frame to another database (different `database_uuid`) invalidates the visible frame integrity check; restoring a DB dump under a new uuid silently breaks frame verification on historical signatures. _ES: Tras restaurar una copia de la base de datos, la verificación de firmas antiguas aparece como inválida._

**ETG26. Template attachment `copy_data` does not re-stamp `res_model`** — On duplicate, `copy_data` copies `attachment_ids` (`sign/models/sign_request.py:198-204`, `SignRequest.copy_data`) but only the create hook sets `res_model = sign.request` (`:184`). Impact: a copied template's attachment can still reference the OLD record, so deleting the source can orphan or mis-target the PDF. _ES: Al duplicar una plantilla de firma y borrar la original, el PDF apunta al registro equivocado o desaparece._

## Additional — VoIP

**ETG27. Call activity refresh is bus-driven and category-gated** — On `mail.activity` create the model filters call activities and fires `_bus_send("refresh_call_activities")` (`voip/models/mail_activity.py:28-35`, `MailActivity.create`). Impact: changing an activity's type so its category is no longer `phonecall` will NOT refresh the softphone activity list — the call panel shows stale activities until manual reload. _ES: El panel del softphone muestra actividades de llamada desactualizadas hasta que recargo manualmente._

**ETG28. Settings auto-create ignores explicit company context** — `_find_or_create_for_user()` (`voip/models/res_users.py:132-144`, `ResUsers._reflect_change_in_res_users_settings`) creates `res.users.settings` lazily on any VoIP field write. Impact: in a multi-company write without a pinned `env.company`, the settings row can be created against the wrong company, and VoIP provider resolution then picks the wrong `voip_provider_id`. _ES: En multicompañía el VoIP coge el proveedor equivocado porque crea la configuración en la compañía errónea._

## Additional — Data Cleaning

**ETG29. Automatic mode applies suggestions to PRODUCTION with no review** — In `cleaning_mode='automatic'`, `_cron_clean_records` creates the batch and immediately calls `.action_validate()` (`data_cleaning/models/data_cleaning_model.py:170-171`, `DataCleaningModel._clean_records`). Impact: trim/upper/lower/phone rewrites hit live records unattended; a mis-scoped rule (e.g. `upper` on a name field) silently mutates the whole table on the next cron tick. There is no undo — only the chatter trail of changed records. _ES: La limpieza automática modifica registros reales sin revisión y no hay forma de deshacerlo._

## Additional — Data Merge

**ETG30. FK redirect swallows IntegrityError/UniqueViolation as a WARNING** — `_update_foreign_keys()` wraps each raw `UPDATE` in a savepoint and on `UniqueViolation`/`IntegrityError` only logs a warning, never aborts (`data_cleaning/models/data_merge_record.py:445-463`, `DataMergeRecord._update_foreign_keys`). Impact: a merge can "succeed" while silently leaving some foreign keys pointing at the discarded duplicate — partial merges with no error surfaced to the user. _ES: La fusión dice que fue bien pero quedan referencias apuntando al registro descartado._

**ETG31. Merge reassigns attachments/activities/messages with no ownership check** — `_merge_additional_models` (`data_cleaning/models/data_merge_record.py:472-495`, `DataMergeRecord._merge_additional_models`) rewrites `res_id` on `ir.attachment`, `mail.activity`, `mail.message`, `ir.model.data` for the master. Impact: documents/notes authored under other modules get reassigned to the surviving record without verifying they belonged to the merged set — cross-module data bleed. _ES: Al fusionar registros se arrastran adjuntos, notas y mensajes de otros módulos que no eran del grupo fusionado._

**ETG32. Similarity baseline is the FIRST record, order-dependent** — `data_merge_group` computes similarity via set-intersection seeded from the first record's items (`data_cleaning/models/data_merge_group.py:49-60`, `DataMergeGroup._compute_similarity`), char fields only. Impact: the "master" suggestion and the divergent-field list depend on record ordering, not on a majority vote — re-running dedup after new records arrive can reshuffle which record is proposed as master. _ES: Al reejecutar la detección de duplicados cambia cuál registro propone como principal._

## v19 Changes

**ETG21. Sign depends on 'certificate' module in v19** — New dependency for crypto/signing enhancements. Template preview wizard added. Company-level sign config added. Impact: v18→v19 migration needs certificate module installed. _ES: Tras migrar a la v19 el módulo de firma no arranca porque falta el módulo de certificados._

**ETG22. Sign editor components lazy-loaded in v19** — `sign_request_activity` and file_viewer are lazy-loaded OWL components. `web.assets_unit_tests` filter changed. Impact: custom sign JS extensions may need refactoring. _ES: En la v19 mis personalizaciones JS del editor de firma dejan de cargar o funcionar._

## Lead Intelligence / IAP (UNIQUE to this domain)

**ETG33. `_notify_no_more_credit` emails creators ONCE, then goes silent** — `crm.iap.lead.helpers._notify_no_more_credit(service, model, param)` (`crm_iap_mine/models/crm_iap_lead_helpers.py:9-30`, `CRMHelpers._notify_no_more_credit`) checks an `ir.config_parameter` flag; if set, it returns immediately. Reveal sets `reveal.already_notified=True` after the first credit-exhaustion mail and only resets it to `False` on the next successful run. Impact: if credits stay at zero, users get exactly ONE "out of credits" email — subsequent days fail silently with no further notice until credits are topped up and a run succeeds. _ES: Solo llega un email de 'sin créditos' de enriquecimiento; luego falla en silencio cada día sin avisar._

**ETG34. Lead enrichment is auto-triggered on EVERY `crm.lead` create (default ON)** — `crm_iap_enrich/models/crm_lead.py:41-49`, `Lead.create`: `create()` reads `ir.config_parameter 'crm.iap.lead.enrich.setting'` (default `'auto'`) and, if auto, fires the hourly cron via `_trigger()`. Impact: installing `crm_iap_enrich` silently turns on automatic IAP credit consumption for all new leads with a non-generic email domain — there is no install-time opt-in. Toggle is Settings → CRM (`lead_enrich_auto` writes the cron's `active`, NOT the config param read by `create` — see ETG35). _ES: Tras instalar el enriquecimiento de leads se empiezan a gastar créditos IAP solos en cada lead nuevo sin haberlo activado._

**ETG35. Two DIFFERENT switches govern enrichment auto-mode (config param vs cron.active)** — `create()` keys off `ir.config_parameter 'crm.iap.lead.enrich.setting'`, but the Settings UI (`res_config_settings.py`) toggles the **cron's `active`** field via `lead_enrich_auto`. They are not the same store. Impact: disabling the cron stops the scheduled batch but `create()` still calls `cron._trigger()` (a no-op if cron inactive); a custom flow that sets the config param to `'manual'` won't change what the Settings screen shows. Don't assume one switch. _ES: Desactivo el enriquecimiento automático en ajustes pero sigue comportándose distinto de lo esperado._

**ETG36. Enrich only fills EMPTY lead fields — never overwrites** — `_iap_enrich_from_response` (`crm_iap_enrich/models/crm_lead.py:118-165`, `Lead._iap_enrich_from_response`) writes a field only `if not lead[field]`. Impact: a lead with a wrong/placeholder partner_name/city/etc. will NOT be corrected by enrichment; IAP data is silently discarded for already-populated fields. `iap_enrich_done=True` is set regardless, so it won't be retried. _ES: El enriquecimiento no corrige datos ya rellenos aunque estén mal, y aun así marca el lead como procesado._

**ETG37. Generic email domains are skipped and marked done (credit-saving, but no enrichment)** — domains in `iap_tools._MAIL_PROVIDERS` (gmail, etc.) set `iap_enrich_done=True` + a "not found" note WITHOUT calling IAP (`crm_iap_enrich/models/crm_lead.py:78-85`, `Lead.iap_enrich`). Impact: leads with personal-email contacts never get enriched and silently look "processed". Expected behaviour, but surprising when debugging "why wasn't this lead enriched". _ES: Los leads con email personal (gmail, etc.) nunca se enriquecen y aparecen como procesados._

**ETG38. Enrich credit-exhaustion ABORTS the whole run, not just the failed batch** — on `InsufficientCreditError` the loop `break`s out of all remaining 50-lead batches (`crm_iap_enrich/models/crm_lead.py:92-99`, `Lead.iap_enrich`). Impact: a single mid-run credit exhaustion leaves the tail of the queue unenriched until the next cron tick; combined with per-batch `cr.commit()` (ETG39) you get a partially-enriched queue. _ES: Cuando se acaban los créditos a mitad de proceso, el resto de leads de la cola se quedan sin enriquecer._

**ETG39. Enrich commits per batch — partial results survive a later crash** — `iap_enrich` does `env.cr.commit()` after each 50-lead batch (unless in test mode) "to avoid losing credits" (`crm_iap_enrich/models/crm_lead.py:114-116`, `Lead.iap_enrich`). Impact: a transaction wrapping a bulk lead import that also triggers enrichment will have those commits escape the outer transaction — you cannot fully roll back an import once enrichment has committed a batch. _ES: No puedo revertir una importación de leads porque el enriquecimiento ya confirmó parte de los cambios._

**ETG40. Reveal runs ONLY for public (not-logged-in) visitors and ONLY on 200 page responses** — `ir.http._serve_page` (`website_crm_iap_reveal/models/ir_http.py:15-44`, `IrHttp._serve_page`) gates on `request.env.user._is_public()` and `status_code==200`, and skips if the visitor already has a lead. Impact: testing reveal while logged in as a backend user produces NOTHING; staging behind auth/4xx never reveals. Also it needs working **GeoIP** (`request.geoip.country_code`) — no country code = no reveal view, no error. _ES: Al probar la revelación de visitantes logueado como usuario no pasa nada y no crea ningún lead._

**ETG41. Reveal dedups per-IP for 6 months and per-clearbit_id forever** — `_unlink_unrelevant_reveal_view` drops views whose IP produced a lead within `reveal.lead_month_valid` (default 6 months); `_create_lead_from_response` also `search_count` on `reveal_id` and skips if a lead with that clearbit_id already exists (`website_crm_iap_reveal/models/crm_reveal_rule.py:226-242`, `CRMRevealRule._unlink_unrelevant_reveal_view` y `website_crm_iap_reveal/models/crm_reveal_rule.py:354-385`, `CRMRevealRule._create_lead_from_response`). Impact: revisiting from the same IP or a company already revealed creates no new lead — expected, but explains "why didn't my test visit create a second lead". The unique index `(reveal_rule_id, reveal_ip)` further blocks duplicate pending views. _ES: Volver a visitar la web desde la misma IP o empresa ya revelada no crea un segundo lead._
