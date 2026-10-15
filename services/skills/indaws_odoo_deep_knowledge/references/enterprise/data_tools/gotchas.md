<!-- verified: v18.0 @ 2026-06 -->
# Data Tools — Gotchas (DTG)

Prefijo **DTG**. Marcado por subsistema/edición:
- **[CE-Recycle]** = `data_recycle` (CE).
- **[EE-Clean]** = `data_cleaning` rules de formato (EE).
- **[EE-Merge]** = `data_merge.*` dentro de `data_cleaning` (EE).

Todo verificado contra fuente v18 (rutas relativas a `odoo18/addons/` para CE y `odoo_enterprise/18/` para EE).

---

### DTG1 — [EE-Merge] No existe módulo `data_merge` independiente
Los modelos `data_merge.model/group/record/rule` viven dentro del módulo **`data_cleaning`** (EE), no en un módulo `data_merge`. El directorio `odoo_enterprise/18/data_merge` **no existe** (los únicos `data_merge_*` son los bridges: crm, helpdesk, project, stock_account, utm). Para tener deduplicación basta instalar `data_cleaning`. Source: `data_cleaning/models/data_merge_model.py`, `data_cleaning/__manifest__.py:11-29` (las vistas `data_merge_*_views.xml` se cargan desde el propio data_cleaning).
**Síntoma**: buscas el módulo "Data Merge" en Apps y no aparece; o un manifest hace `depends:['data_merge']` y falla.

### DTG2 — [EE-Clean][EE-Merge] data_cleaning auto-instala data_recycle (CE)
`data_cleaning` tiene `auto_install=['data_recycle']` y `depends` incluye `data_recycle`. Instalar Data Cleaning arrastra Data Recycle. El menú raíz lo aporta `data_recycle` (`menu_data_cleaning_root`), que es el `application=True`. Source: `data_cleaning/__manifest__.py:18,39`, `data_recycle/__manifest__.py`.
**Síntoma**: aparece "Data Recycle" instalado aunque solo querías cleaning/merge.

### DTG3 — [CE-Recycle] recycle_action default es `unlink` (BORRA, no archiva)
`recycle_action` default = `'unlink'` (no `archive`). Una regla de recycle creada sin tocar este campo **borra registros físicamente**. Source: `data_recycle/models/data_recycle_model.py:38-41`, `class DataRecycleModel`.
**Síntoma**: registros desaparecen de la BD tras el cron de las 03:00, no aparecen como archivados.

### DTG4 — [CE-Recycle] recycle_mode `automatic` ejecuta sin revisión humana
Con `recycle_mode='automatic'`, `_recycle_records` llama `action_validate()` directamente (archiva/borra en el acto, lotes de 5000). No se crean `data_recycle.record` para revisar. Combinado con DTG3 (`unlink` default) = borrado silencioso diario. Source: `data_recycle/models/data_recycle_model.py:136-142`, `DataRecycleModel._recycle_records`.
**Síntoma**: configuras una regla "para ver qué pillaría" y borra de verdad.

### DTG5 — [CE-Recycle] `archive` falla si el modelo no tiene `active`
`@api.constrains('recycle_action')` lanza `UserError` si pones `archive` sobre un modelo sin campo `active`. Source: `data_recycle/models/data_recycle_model.py:77-81`, `class DataRecycleModel`.
**Síntoma**: "This model doesn't manage archived records. Only deletion is possible." al guardar la regla.

### DTG6 — [CE-Recycle] desactivar la regla BORRA todos sus candidatos pendientes
`data_recycle.model.write({'active': False})` hace `unlink()` de todos los `data_recycle.record` asociados (no los archiva, los borra). Lo mismo en cleaning. Source: `data_recycle/models/data_recycle_model.py:195-198`, `DataRecycleModel.write`, `data_cleaning/models/data_cleaning_model.py:232-235`, `DataCleaningModel.write`.
**Síntoma**: archivas una regla y pierdes la lista de candidatos que habías revisado a medias.

### DTG7 — [CE-Recycle] el filtro temporal exige campo date/datetime store
`time_field_id` está limitado por domain a campos `date`/`datetime` con `store=True`. No puedes filtrar por un campo computed no almacenado. Sin `time_field_id` solo cuenta el `domain` manual. Source: `data_recycle/models/data_recycle_model.py:45-48`, `class DataRecycleModel`.

### DTG8 — [CE-Recycle] include_archived no es por defecto
Si los registros candidatos ya están archivados (active=False), no se detectan salvo `include_archived=True` (que aplica `active_test=False`). Source: `data_recycle/models/data_recycle_model.py:55`, `DataRecycleModel.include_archived` y `data_recycle/models/data_recycle_model.py:128-129`, `DataRecycleModel._recycle_records`.
**Síntoma**: regla archive→unlink en dos pasos: archivas y luego nunca los borra porque la segunda regla no ve los archivados.

### DTG9 — [EE-Clean] Format Phone requiere el paquete Python `phonenumbers`
La acción `phone` valida en `_onchange_action` que `phonenumbers` esté instalado; si no, `ValidationError`. Sin el paquete, la acción phone no funciona. Source: `data_cleaning/models/data_cleaning_rule.py:128-134`, `class DataCleaningRule`.
**Síntoma**: "The Python module `phonenumbers` is not installed."

### DTG10 — [EE-Clean] `trim_all` quita TODOS los espacios, también los internos
`trim_all` = `value.replace(' ', '')` → "John Doe" se convierte en "JohnDoe". Para emails/VAT está bien; para nombres NO. Usa `trim_superfluous` para nombres. La data demo usa `all` en email/vat/phone y `superfluous` en name. Source: `data_cleaning/models/data_cleaning_rule.py:12`, `ACTIONS_PYTHON` y `data_cleaning/models/data_cleaning_rule.py:23`, `ACTIONS_SQL`, `data_cleaning_data.xml`.
**Síntoma**: nombres de contacto quedan pegados sin espacios tras el cron.

### DTG11 — [EE-Clean] la acción phone se procesa en Python (lento), no en SQL
A diferencia de trim/case/html (SQL masivo), `phone` itera registro a registro en `_clean_records_format_phone` porque depende del país. Con muchos registros, el cron de cleaning puede ser lento. Source: `data_cleaning/models/data_cleaning_model.py:79-107`, `DataCleaningModel._clean_records_format_phone`, `data_cleaning/models/data_cleaning_rule.py:31`, `ACTIONS_SQL`.

### DTG12 — [EE-Clean] el país del teléfono se infiere del registro, no del número
`_phone_format` usa `record.country_id` o `record.company_id.country_id` (`_get_country_id`). Si el contacto no tiene país y la company tampoco, el formateo internacional puede fallar o producir un valor inesperado. Source: `data_cleaning/models/data_cleaning_record.py:30-37`, `class DataCleaningRecord`, `data_cleaning/models/data_cleaning_model.py:98-99`, `DataCleaningModel._clean_records_format_phone`.
**Síntoma**: teléfonos correctos quedan sin sugerencia, o con prefijo país equivocado.

### DTG13 — [EE-Clean] solo campos char/text/html; m2o/selection no se limpian
`data_cleaning.rule.field_id` domain = `ttype in ('char','text','html')`. No puedes normalizar selections, m2o, etc. con cleaning (para eso está merge en m2o, o Studio). Source: `data_cleaning/models/data_cleaning_rule.py:47-50`, `class DataCleaningRule`.

### DTG14 — [EE-Merge] la similitud NO usa las reglas de match
`merge_threshold` se compara contra `data_merge.group.similarity`, que se calcula sobre **todos los campos char del modelo** (cuántos coinciden exactamente), no sobre los campos de las `data_merge.rule`. Una regla de match por `email` agrupa, pero la similitud puede salir baja si el resto de campos char difieren → en automatic puede no fusionar pese a coincidir el email. Source: `data_cleaning/models/data_merge_group.py:36-39`, `DataMergeGroup._get_similarity_fields` (el cálculo, en `data_cleaning/models/data_merge_group.py:41-60`, `DataMergeGroup._compute_similarity`) vs el SQL de agrupación en `data_cleaning/models/data_merge_model.py:191-237`, `DataMergeModel.find_duplicates`.
**Síntoma**: duplicados con el mismo email no se fusionan automáticamente; el % de similitud parece arbitrario.

### DTG15 — [EE-Merge] similitud = exact-match de campos char, no fuzzy
No hay distancia de Levenshtein ni trigram. "Acme" vs "Acme " (con espacio) cuentan como campos DISTINTOS → bajan la similitud. No considera unaccent en el cálculo (TODO en el código). Source: `data_cleaning/models/data_merge_group.py:53-60`, `DataMergeGroup._compute_similarity`.

### DTG16 — [EE-Merge] match_mode `accent` requiere extensión PostgreSQL `unaccent`
El modo "Case/Accent Insensitive" solo aparece en el selector si `registry.has_unaccent`. Si la rule usa accent y falta la extensión, `find_duplicates` lanza `UserError('Missing required PostgreSQL extension: unaccent')`. Source: `data_cleaning/models/data_merge_rule.py:26-31`, `DataMergeRule._available_match_modes`, `data_cleaning/models/data_merge_model.py:201-203,231-234`, `DataMergeModel.find_duplicates`.
**Síntoma**: el cron de find duplicates falla; o no aparece la opción accent al crear la regla.

### DTG17 — [EE-Merge] merge automático irreversible y silencioso
Con `merge_mode='automatic'` + `merge_threshold` superado, `find_duplicates` fusiona y borra el grupo en el cron de la 01:00, sin intervención. El `removal_mode='delete'` además BORRA los duplicados (no archiva). Source: `data_cleaning/models/data_merge_model.py:288-291`, `DataMergeModel.find_duplicates`, `data_cleaning/models/data_merge_group.py:215-223`, `DataMergeGroup._post_merge`.
**Síntoma**: registros desaparecen/se fusionan de madrugada; FKs reapuntadas sin aviso.

### DTG18 — [EE-Merge] el master genérico es el registro MÁS ANTIGUO
Sin `_elect_method` custom, el master = `sorted('create_date')[0]`. Los datos del más antiguo prevalecen (los demás solo aportan sus FKs). res.partner y crm.lead lo sobrescriben con criterios propios. Source: `data_cleaning/models/data_merge_group.py:98-106`, `DataMergeGroup._elect_method`, `data_cleaning/res_partner.py:28-31`.
**Síntoma**: tras merge, gana el registro viejo con datos obsoletos en vez del más completo/reciente.

### DTG19 — [EE-Merge] res.partner y crm.lead NO usan el merge genérico
Ambos tienen `_disable_data_merge=True` y un `_merge_method` propio: res.partner delega en `base.partner.merge.automatic.wizard`; crm.lead en `merge_opportunity`. La acción de merge genérica del menú contextual se oculta (`hide_merge_action`). Source: `data_cleaning/models/res_partner.py:9-26`, `class Partner`; `data_merge_crm/models/crm_lead.py:8-25`, `class CrmLead`; `data_cleaning/models/ir_model.py:24-30`, `IrModel._compute_hide_merge_action`.
**Síntoma**: el botón "Merge" no aparece en res.partner/crm.lead; o el merge de partners abre el wizard estándar en vez del flujo data_merge.

### DTG20 — [EE-Merge] crm.lead: no se pueden fusionar oportunidades ganadas
`crm.lead._merge_method` lanza `UserError('Won opportunities cannot be merged.')` si queda ≤1 lead con `probability < 100`. La `data_merge.model` demo de crm filtra `domain=[('probability','<',100)]`. Source: `data_merge_crm/models/crm_lead.py:16-20`, `CrmLead._merge_method`, `data_merge_crm/data/data_merge_data.xml`.

### DTG21 — [EE-Merge] el atributo de clase "_prevent_merge" bloquea la dedup en un modelo
Si un modelo define el atributo de clase `_prevent_merge = True`, crear una `data_merge.model` para él lanza `ValidationError('Deduplication is forbidden on the model: %s')` en el `@api.constrains('res_model_id')` `_check_prevent_merge` (verifica vía `hasattr(...)` sobre ese atributo + valor truthy). Es un hook: **ningún modelo de Odoo 18 lo define por defecto** (es un punto de extensión para módulos custom). Distinto del atributo `_disable_data_merge` (que solo oculta el botón contextual, no bloquea crear la `data_merge.model`; lo activa p.ej. `data_cleaning/models/res_partner.py:12`, `Partner._disable_data_merge`). Source: `data_cleaning/models/data_merge_model.py:299`, `DataMergeModel._check_prevent_merge` (`_check_prevent_merge`), `:302` (el `hasattr`). Nota: en EE18 el módulo es `data_cleaning`, no `data_merge`.

### DTG22 — [EE-Merge] cross-company off por defecto: no agrupa entre companies
`mix_by_company=False` (default) + multi-company → `find_duplicates` añade `company_id` al GROUP BY, así que duplicados en companies distintas NO se sugieren. Hay que activar "Cross-Company" para detectarlos. Source: `data_cleaning/models/data_merge_model.py:63`, `DataMergeModel.mix_by_company`, y el GROUP BY en `data_cleaning/models/data_merge_model.py:205-209`, `DataMergeModel.find_duplicates`.
**Síntoma**: contactos duplicados entre dos companies no salen como duplicados.

### DTG23 — [EE-Merge] `create_threshold` borra grupos por debajo del umbral
Subir `create_threshold` (write) borra inmediatamente todos los `data_merge.group` con `similarity <= threshold/100`. En `find_duplicates`, los grupos por debajo se descartan al vuelo. Es un campo `groups='base.group_no_one'` (solo modo dev). Source: `data_cleaning/models/data_merge_model.py:78`, `DataMergeModel.create_threshold` y `data_cleaning/models/data_merge_model.py:284-286`, `DataMergeModel.find_duplicates` y `data_cleaning/models/data_merge_model.py:313-314`, `DataMergeModel.write`.
**Síntoma**: subes el "Suggestion Threshold" y desaparecen grupos ya detectados.

### DTG24 — [EE-Merge] reapuntado de FKs tolera fallos silenciosamente
`_update_foreign_keys` y `_merge_additional_models` capturan `UniqueViolation`/`IntegrityError` con savepoint y solo logean warning; el merge continúa. Una FK que no se pudo reapuntar queda colgando del registro a borrar/archivar. Source: `data_cleaning/models/data_merge_record.py:458-463`, `DataMergeRecord._update_foreign_keys` y `data_cleaning/models/data_merge_record.py:511-516`, `DataMergeRecord._merge_additional_models`.
**Síntoma**: tras merge, referencias perdidas o registros source con FKs residuales; warnings en el log pero sin error visible.

### DTG25 — [EE-Merge] `used_in` (References) puede ser caro de computar
`_compute_usage` recorre todas las FK indexadas que apuntan al modelo y hace `_read_group` por cada una. Desactivable con `ir.config_parameter` `data_merge.compute_references=False`. Source: `data_cleaning/models/data_merge_record.py:227-291`, `DataMergeRecord._get_references`.
**Síntoma**: la vista de deduplicación tarda en cargar con muchos duplicados.

### DTG26 — [EE-Merge] `merge_common_lists` agrupa transitivamente (puede sobre-agrupar)
A→B y B→C agrupan A,B,C en un solo grupo aunque A y C no compartan nada directamente. Controlable con `ir.config_parameter` `data_merge.merge_lists` (default True). Con muchos matches encadenados, un grupo puede crecer demasiado. Source: `data_cleaning/models/data_merge_model.py:21-38`, `merge_common_lists` y `data_cleaning/models/data_merge_model.py:252-256`, `DataMergeModel.find_duplicates`.
**Síntoma**: grupo de duplicados gigante mezclando registros poco relacionados.

### DTG27 — [EE-Merge] subconjuntos no crean grupo nuevo
Si {A,B} ya tiene grupo {A,B,C}, un nuevo match {A,B} se ignora (es subset). Pero {A,D} sí crea grupo nuevo. Source: `data_cleaning/models/data_merge_model.py:267-273`, `DataMergeModel.find_duplicates`.

### DTG28 — [CE-Recycle][EE-Clean][EE-Merge] todo es admin-only (group_system)
No hay grupo funcional para data tools: todos los modelos requieren `base.group_system`. Un usuario funcional no puede ver ni gestionar reglas/candidatos. Source: `data_recycle/security/ir.model.access.csv`, `data_cleaning/security/ir.model.access.csv`.

### DTG29 — [EE-Clean][EE-Merge] candidatos "discard" se reactivan en el siguiente barrido
`data_cleaning.record.action_discard` / `data_merge.group.discard_records` marcan `active=False`/`is_discarded`, pero el motor (`_clean_records`, `find_duplicates`) excluye por `NOT EXISTS data_cleaning_record` o por groups existentes — un descarte de cleaning solo persiste mientras el `data_cleaning.record` exista; tras `_cleanup` puede volver a crearse. En merge, el discard se respeta mientras el group sobreviva al cleanup. Source: `data_cleaning/models/data_cleaning_record.py:125-126`, `DataCleaningRecord.action_discard`, `data_cleaning/models/data_cleaning_model.py:140-145`, `DataCleaningModel._clean_records`, `data_cleaning/models/data_merge_group.py:62-70`, `DataMergeGroup.discard_records` y `:238-284`, `DataMergeGroup._cleanup`, `DataMergeGroup.discard_records`.
**Síntoma**: registros que ya descartaste reaparecen como candidatos.

### DTG30 — [EE-Clean] el motor de cleaning solo toca registros activos
`_clean_records` añade `active_cond` (`AND <active_name>`) según `_active_name`. Registros archivados no se limpian. Source: `data_cleaning/models/data_cleaning_model.py:118-121`, `DataCleaningModel._clean_records` y `data_cleaning/models/data_cleaning_model.py:146`, `DataCleaningModel._clean_records`.

### DTG31 — [EE-Merge] _log_merge solo si el modelo es mail.thread
El snapshot del merge en el chatter solo se escribe si el modelo destino hereda `mail.thread`. Modelos sin chatter pierden la traza del merge. Source: `data_cleaning/models/data_merge_group.py:174-196`, `DataMergeGroup._log_merge`.

### DTG32 — [EE-Merge] contextual merge crea data_merge.model inactivas ocultas
Cada vez que usas el botón "Merge" en list view sobre un modelo sin merge.model previo, se crea una `data_merge.model` con `is_contextual_merge_action=True`, `active=False`. Se acumulan ocultas (active=False). Source: `data_cleaning/models/data_merge_record.py:627-639`, `DataMergeRecord.action_deduplicates`.

### DTG33 — [CE-Recycle] el filtro temporal usa `now` del servidor según ttype
Si `time_field_id` es `date`, compara contra `fields.Date.today()`; si `datetime`, contra `fields.Datetime.now()`. Mezclar zonas horarias en datetime puede incluir/excluir registros límite. Source: `data_recycle/models/data_recycle_model.py:120-126`, `DataRecycleModel._recycle_records`.

### DTG34 — [EE-Merge] el reapuntado de FKs M2M NO está protegido por savepoint (a diferencia de las FKs normales)
DTG24 dice que `_update_foreign_keys` tolera fallos en silencio, pero eso **solo aplica a la rama de FKs normales** (`else`, `data_cleaning/models/data_merge_record.py:444-463`, `DataMergeRecord._update_foreign_keys`), que envuelve cada `UPDATE` en `self._cr.savepoint()` + `try/except UniqueViolation/IntegrityError`. La rama de **tablas relación M2M** (`column_count == 2`, `:418-440`) ejecuta el `UPDATE` con su `NOT EXISTS` anti-duplicado **sin savepoint ni try/except**: si esa query falla por cualquier `psycopg2.Error` no contemplado, la excepción propaga y aborta TODO el merge (no se logea warning, no continúa). El `NOT EXISTS` evita el duplicado de la combinación (master ya tiene el tag), pero no cubre otros errores de integridad de la tabla relación.
**Síntoma**: un merge que reapunta M2M revienta con error SQL en vez de logear warning como hacen las FKs normales.

### DTG35 — [EE-Merge] con `removal_mode='archive'` pero modelo SIN campo active, `_post_merge` BORRA igualmente
`_post_merge` (`data_cleaning/models/data_merge_group.py:215-223`, `DataMergeGroup._post_merge`) hace `if self.model_id.removal_mode == 'delete' or not origins._active_name: origins.unlink()`. Es decir, aunque la `data_merge.model` esté configurada como `archive`, si el modelo destino **no tiene `_active_name`** (sin campo `active`/`active_test`), los registros source se **borran físicamente** (`unlink`), no se archivan. No hay aviso: el "archive" se degrada silenciosamente a "delete".
**Síntoma**: configuras removal_mode=archive esperando recuperar los duplicados, pero el modelo no soporta archivado y desaparecen de la BD.

### DTG36 — [EE-Merge] el master debe EXISTIR o el merge se aborta en silencio (return sin error)
`merge_records` (`data_cleaning/models/data_merge_group.py:142-144`, `DataMergeGroup.merge_records`) comprueba `if not master_record._original_records(): _logger.warning('The master record does not exist'); return`. Si el registro elegido como master fue borrado entre la detección del grupo (cron 01:00) y el merge, la fusión **no se ejecuta y no lanza error** — solo un warning en log. El grupo queda sin fusionar hasta el siguiente barrido. También: `merge_records` reconsulta los records con `active_test=False` y `order='id'`, y si quedan ≤1 hace `return` (no merge). Source: `data_cleaning/models/data_merge_group.py:135-144`, `DataMergeGroup.merge_records`.
**Síntoma**: un grupo de duplicados marcado para auto-merge no se fusiona y no hay error visible; solo un warning "master record does not exist" en el log.

### DTG37 — [EE-Merge] `_log_merge` solo escribe la nota de archivado en el source si `removal_mode == 'archive'`
En `_log_merge` (`data_cleaning/models/data_merge_group.py:194-196`, `DataMergeGroup._log_merge`), la nota "merged" se postea en el chatter del **source** SOLO si `removal_mode == 'archive'` (el source sigue existiendo archivado). Con `removal_mode == 'delete'` el source se borra y su chatter desaparece: la única traza del merge queda en el chatter del master (`data_merge_main`). Si necesitas auditoría completa de qué se fusionó en el registro eliminado, usa `archive`, no `delete`. Source: `data_cleaning/models/data_merge_group.py:174-196`, `DataMergeGroup._log_merge`.

### DTG38 — [EE-Merge] crm.lead define `_elect_master` (typo): NUNCA se llama, gana el genérico
El bridge `data_merge_crm` define en `crm.lead` un método `_elect_master(self, records)` (que ordena por `_sort_by_confidence_level`), pero el motor de elección de master (`_elect_master_record`, `data_cleaning/models/data_merge_group.py:83-84`, `DataMergeGroup._elect_master_record`) busca **`_elect_method`** (no `_elect_master`). Como crm.lead **no** define `_elect_method`, para crm.lead se usa el genérico = registro más antiguo (`create_date`). El `_elect_master` del bridge es código muerto / mal nombrado. (Contraste: `res.partner` SÍ define `_elect_method` correctamente, `data_cleaning/res_partner.py:28`, `Partner._elect_method`.) Source: `data_merge_crm/models/crm_lead.py:27-28`, `CrmLead._elect_master` vs `data_cleaning/models/data_merge_group.py:75-96`, `DataMergeGroup._elect_master_record`.
**Síntoma**: esperas que el lead "ganador" sea el de mayor probabilidad/confianza pero el master elegido es el más antiguo; da igual que crm.lead tenga `_elect_master`.

### DTG39 — [EE-Merge] `_disable_data_merge` también en mailing.list y account.analytic.line (no solo partner/lead)
Además de `res.partner` y `crm.lead`, ponen `_disable_data_merge=True`: `mailing.list` (`mass_mailing/models/mailing_list.py:18`, `MassMailingList._disable_data_merge`, `MassMailingList._disable_data_merge`, **CE**) y `account.analytic.line` (`timesheet_grid/models/analytic.py:20`, EE). En esos modelos el botón "Merge" contextual queda oculto (`hide_merge_action`). Ojo: a diferencia de partner/lead, mailing.list y account.analytic.line **NO** definen un `_merge_method` propio, así que si fuerzas una `data_merge.model` sobre ellos (vía menú, no contextual) usarían el merge genérico de FKs. Source: `mass_mailing/models/mailing_list.py:18`, `MassMailingList._disable_data_merge`, `timesheet_grid/models/analytic.py:20`, `data_merge/models/ir_model.py:24-30`, `IrModel._compute_hide_merge_action`.

### DTG40 — [EE-Merge] res.partner `_elect_method` prioriza ARCHIVADOS al final, no por "más completo"
El `_elect_method` de res.partner ordena por `(not p.active, create_date)` y toma `[0]`: prefiere registros **activos** y, entre ellos, el **más antiguo**. No elige por número de campos rellenos ni por completitud. Un partner activo viejo gana a uno activo reciente con más datos. Source: `data_cleaning/res_partner.py:28-31`, `Partner._elect_method`.
**Síntoma**: tras dedup de contactos, el master es el contacto viejo con la ficha a medias en vez del recién actualizado.

### DTG41 — [EE-Merge] res.partner merge devuelve `post_merge=False`: el archivado/borrado lo hace el wizard, no data_merge
El `_merge_method` de res.partner delega en `base.partner.merge.automatic.wizard.action_merge()` y devuelve `{'post_merge': False, 'log_chatter': True}`. Como `post_merge=False`, `merge_records` **NO** llama a `_post_merge` (`data_cleaning/models/data_merge_group.py:160-161`, `DataMergeGroup.merge_records`): no se aplica el `removal_mode` (archive/delete) de la `data_merge.model`. Es el wizard estándar de partners quien decide qué pasa con los duplicados (los borra). Configurar `removal_mode=archive` en la `data_merge.model` de res.partner **no tiene efecto**. Source: `data_cleaning/res_partner.py:14-26`, `Partner._merge_method`, `data_cleaning/models/data_merge_group.py:156-161`, `DataMergeGroup.merge_records`.

### DTG42 — [EE-Merge] crm.lead merge devuelve `log_chatter=False`: sin snapshot data_merge en el chatter
El `_merge_method` de crm.lead devuelve `{'log_chatter': False, 'post_merge': True}`. Con `log_chatter=False`, `merge_records` **NO** llama a `_log_merge` (`data_cleaning/models/data_merge_group.py:157-158`, `DataMergeGroup.merge_records`): el snapshot tipo data_merge (`data_merge_main`/`data_merge_merged`) no se postea. La traza del merge en el chatter es la que genere `merge_opportunity` del módulo crm, no la de data_merge. Source: `data_merge_crm/models/crm_lead.py:13-25`, `CrmLead._merge_method`, `data_cleaning/models/data_merge_group.py:157-158`, `DataMergeGroup.merge_records`.

### DTG43 — [EE-Merge] `merge_records` SIEMPRE borra los data_merge.record al final, aunque el merge real falle
Tras llamar al `_merge_method`, `merge_records` ejecuta `(master_record + to_merge).unlink()` (`data_cleaning/models/data_merge_group.py:164`, `DataMergeGroup.merge_records`) de forma incondicional: borra los `data_merge.record` del grupo pase lo que pase en el merge subyacente (salvo que el merge haya lanzado una excepción que aborte la transacción). Si el `_merge_method` custom "se traga" un fallo y no propaga excepción, los `data_merge.record` desaparecen igualmente y el grupo se da por fusionado. Source: `data_cleaning/models/data_merge_group.py:156-164`, `DataMergeGroup.merge_records`.

### DTG44 — [EE-Merge] `data_merge.record.create` valida que el registro original EXISTA (con active_test implícito)
El override `create` de `data_merge.record` (`data_cleaning/models/data_merge_record.py:554-564`, `class DataMergeRecord`) hace `browse(res_id).exists()` y lanza `ValidationError('The referenced record does not exist')` si no existe. **Pero usa el modelo SIN `active_test=False`**: si el registro candidato está archivado, `browse().exists()` aún lo encuentra (exists no aplica active_test, consulta por id en tabla) — el chequeo pasa. El riesgo real es que entre el `find_duplicates` y el `create` del record el original se borre físicamente → entonces el create del data_merge.record peta con ValidationError y aborta el lote. Source: `data_cleaning/models/data_merge_record.py:554-564`, `class DataMergeRecord`.

### DTG45 — [CE-Recycle][EE-Clean] `_send_notification` notifica al AUTOR (`notify_author=True`) y por defecto solo a ti mismo
Tanto recycle como cleaning llaman `message_notify(..., notify_author=True, partner_ids=notify_user_ids.partner_id.ids)`. `notify_user_ids` por defecto = `self.env.user` (el que crea la regla). Combinado con `notify_author=True`, recibirás la notificación de "datos a reciclar/limpiar" aunque seas tú quien la disparó. Merge en cambio tiene `notify_user_ids` **sin default** (vacío): si no añades usuarios manualmente, el cron de duplicados **no notifica a nadie**. Source: `data_recycle/models/data_recycle_model.py:62-65`, `DataRecycleModel.notify_user_ids` y `data_recycle/models/data_recycle_model.py:178-193`, `DataRecycleModel._send_notification`, `data_cleaning/models/data_cleaning_model.py:38-42,201-227`, `class DataCleaningModel`, `data_merge/models/data_merge_model.py:66-68,138-168`, `class DataMergeModel`.
**Síntoma**: cleaning/recycle te spamean a ti solo; merge nunca avisa porque olvidaste poner Notify Users.
