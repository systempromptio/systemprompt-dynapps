<!-- verified: v18.0 @ 2026-06 -->
# Data Tools — Core

Trío de herramientas de administración de datos en Odoo 18. **Atención a la distribución de módulos**, que es contraintuitiva:

| Subsistema | Modelos | Módulo | Edición | License |
|---|---|---|---|---|
| **Recycle** (archivar/borrar) | `data_recycle.model`, `data_recycle.record` | `data_recycle` | **CE** | LGPL-3 |
| **Cleaning** (formato de campos) | `data_cleaning.model`, `data_cleaning.rule`, `data_cleaning.record` | `data_cleaning` | **EE** | OEEL-1 |
| **Merge** (deduplicación) | `data_merge.model`, `data_merge.rule`, `data_merge.group`, `data_merge.record` | `data_cleaning` (¡NO existe módulo `data_merge` base!) | **EE** | OEEL-1 |

**No hay módulo `data_merge` independiente**: los modelos `data.merge.*` viven dentro de `data_cleaning` (EE). Los `data_merge_crm/helpdesk/project/stock_account/utm` son bridges que dependen de `data_cleaning`, no de un `data_merge` base. Ver `integrations.md`.

Cadena de dependencias:
- `data_recycle` → `mail` (CE, `application=True`, es la app que aporta el menú raíz).
- `data_cleaning` → `data_recycle`, `phone_validation`, `mail` (EE, `auto_install=['data_recycle']`). Reusa el menú raíz `data_recycle.menu_data_cleaning_root`.

Source: `data_recycle/__manifest__.py:6-27`, `data_cleaning/__manifest__.py:9-39`.

---

## 1. Recycle (CE) — archivar / borrar registros viejos

### `data_recycle.model` — la regla
`data_recycle/models/data_recycle_model.py:20-210`, `DataRecycleModel`. Campos clave:
- `res_model_id` (ir.model) + `res_model_name` (related store).
- `recycle_mode`: `manual` (crea `data_recycle.record` para revisión) | `automatic` (ejecuta directo). Default `manual` (l.34-37).
- `recycle_action`: `archive` | `unlink`. **Default `unlink`** (l.38-41). Un `@api.constrains` (l.77-81) impide `archive` si el modelo destino no tiene campo `active` → `UserError`.
- **Regla de selección** = `domain` (Char, default `'[]'`) + filtro temporal opcional: `time_field_id` (un date/datetime store del modelo) + `time_field_delta` (Integer, default 1) + `time_field_delta_unit` (days/weeks/months/years, default months). El domain temporal se compone como `[(time_field, '<=', now - delta)]` (l.120-126).
- `include_archived` (Boolean): si True, busca con `active_test=False` (l.128-129).

### `_recycle_records()` — el motor (l.107-148)
1. `env.flush_all()`.
2. Lee los `data_recycle.record` existentes (con `active_test=False`) para no duplicar (`mapped_existing_records` por res_id).
3. Por cada modelo: compone domain (rule + temporal), `search()` en el modelo destino, filtra los res_id ya registrados.
4. Si `automatic`: crea en lotes de **5000** (`DR_CREATE_STEP_AUTO`) y llama `action_validate()` inmediatamente (commit por lote si `batch_commits and not is_test`).
5. Si `manual`: solo crea `data_recycle.record` (lotes de **50000**, `DR_CREATE_STEP_MANUAL`) para revisión humana.

### `data_recycle.record` — el candidato (l.9-88)
- `res_id` (Integer, index) + `recycle_model_id`. **No referencia real**, solo guarda model+id.
- `_original_records()` (l.50-64): resuelve el registro real con `active_test=False` + `sudo()` + `.exists()`. Si fue borrado, `name = '**Record Deleted**'`.
- `action_validate()` (l.66-84): agrupa por acción. `archive` → `toggle_active()` (v18) sobre el registro real; `unlink` → `unlink()`. Luego borra los `data_recycle.record` procesados.
- `action_discard()` (l.86-87): marca `active=False` (no vuelve a aparecer hasta nuevo barrido, pero **no se reactiva**; ver DTG).

### Notificación (manual)
`_notify_records_to_recycle()` (l.150-166) + `_send_notification()` (l.168-193): notifica a `notify_user_ids` (default = usuario actual, dominio restringido a `base.group_system`) cada `notify_frequency` × `notify_frequency_period`. Usa `mail.thread.message_notify` con la plantilla QWeb `data_recycle.notification`.

---

## 2. Cleaning (EE) — formatear campos texto

### `data_cleaning.model` + `data_cleaning.rule`
`data_cleaning/models/data_cleaning_model.py:16`, `DataCleaningModel` + `data_cleaning/models/data_cleaning_rule.py:35`, `class DataCleaningRule`. Una `data_cleaning.model` apunta a `res_model_id` y agrupa N `rule_ids`. `cleaning_mode`: `manual`|`automatic`.

`data_cleaning.rule`:
- `field_id`: solo campos `char/text/html` store (domain l.47-50).
- `action` (l.52-59): `trim` | `case` | `phone` | `html`.
  - `action_trim`: `all` (quita TODOS los espacios) | `superfluous` (colapsa espacios y hace strip).
  - `action_case`: `first` (INITCAP/title) | `upper` | `lower`.
- `sequence` (orden de aplicación encadenada).
- `action_technical` (computed, l.80-96): combina `action` + sub-acción → `trim_all`, `trim_superfluous`, `case_first`, `case_upper`, `case_lower`, `phone`, `html`.

### Acciones — doble implementación (clave)
`data_cleaning/models/data_cleaning_rule.py:11-32`, `ACTIONS_PYTHON`. Cada acción existe en **SQL** (para detectar candidatos en masa) y en **Python** (para renderizar el valor sugerido):

| action_technical | ACTIONS_PYTHON | ACTIONS_SQL (operator, expr, composable) |
|---|---|---|
| `trim_all` | `value.replace(' ', '')` | `<>`, `REPLACE(%s,' ','')`, composable |
| `trim_superfluous` | `re.sub(r'\s+',' ',value.strip())` | `<>`, `TRIM(REGEXP_REPLACE(%s,'\s+',' ','g'))`, composable |
| `case_first` | `value.title()` | `<>`, `INITCAP(%s)`, composable |
| `case_upper` | `value.upper()` | `<>`, `UPPER(%s)`, composable |
| `case_lower` | `value.lower()` | `<>`, `LOWER(%s)`, composable |
| `html` | `html2plaintext(value)` | `~`, `'<[a-z]+.*>'`, NO composable |
| `phone` | `record._phone_format(... INTERNATIONAL)` | `(False, 'format_phone', False)` — caso especial Python |

- **composable**: las acciones composables se anidan en una sola expresión SQL (`_action_to_sql` l.98-122: `action = action % field_action['action']`), de modo que varias rules sobre el mismo campo se aplican en cadena dentro de un único SELECT.
- **phone** tiene `operator=False`: no se puede expresar en SQL puro → `_clean_records()` (l.127-130) detecta el caso especial y llama a `_clean_records_format_phone()` (l.79-107), que itera en Python usando `_phone_format` con el país del registro (`_get_country_id`). Requiere el módulo Python `phonenumbers` (validación en `_onchange_action`, l.128-134).

### `_clean_records()` — el motor (l.110-181)
Por cada modelo, monta un SELECT por campo: busca registros donde `field <operator> <expr_limpia>` (es decir, el valor actual difiere del valor limpio) Y que no tengan ya un `data_cleaning.record`. Respeta `_active_name` (solo registros activos). Crea `data_cleaning.record` (manual) o los valida directo (automatic, lotes de 5000).

### `data_cleaning.record` — el candidato (l.8-127)
- `current_value` / `suggested_value` (computed sudo, l.57-81): renderiza el valor sugerido encadenando las acciones Python (`_render_value`, l.46-55, respeta `sequence`).
- `action_validate()` (l.111-123): escribe `suggested_value` en el campo del registro real (`original_record[0].update({field_name: suggested_value})`).
- Para trim, muestra los espacios como ` ` (non-breaking) para que sean visibles en UI (l.67-70).

---

## 3. Merge (EE, dentro de data_cleaning) — deduplicación

### `data_merge.model` + `data_merge.rule`
`data_cleaning/models/data_merge_model.py:41`, `DataMergeModel` + `data_cleaning/models/data_merge_rule.py:7`, `class DataMergeRule`. Una `data_merge.model` apunta a `res_model_id`, tiene `domain` (registros elegibles), N `rule_ids`, y parámetros de merge.

`data_merge.rule`:
- `field_id`: campos `char/text/many2one` store (l.14-16). Para m2o se compara por el `_rec_name` del modelo relacionado.
- `match_mode` (l.17-19, selección dinámica `_available_match_modes` l.26-31):
  - `exact`: igualdad exacta.
  - `accent`: "Case/Accent Insensitive" — **solo disponible si la extensión PostgreSQL `unaccent` está instalada** (`registry.has_unaccent`). Aplica `unaccent(lower(field))`.
- Constraint único `(model_id, field_id)`: un campo no puede repetirse en reglas del mismo modelo.

Parámetros del `data_merge.model`:
- `removal_mode`: `archive` (default) | `delete` — qué hacer con los duplicados tras fusionar.
- `merge_mode`: `manual` | `automatic`.
- `merge_threshold` (default **75**): en automatic, grupos con similitud ≥ umbral se fusionan solos.
- `create_threshold` (default 0, `groups_no_one`): grupos con similitud ≤ umbral NO se crean/sugieren.
- `mix_by_company` (default False): si False y hay multi-company, agrupa duplicados POR company (no cruza companies).

### `find_duplicates()` — identificación (l.177-293)
1. Por cada rule, ejecuta un SQL `GROUP BY <field> HAVING COUNT > 1` sobre el modelo, devolviendo arrays de IDs que comparten ese valor (l.215-237). Para m2o hace JOIN al modelo relacionado y compara su `_rec_name` (l.195-199). Para `accent` envuelve en `unaccent(lower(...))` (l.201-203). Multi-company + `not mix_by_company` añade `company_id` al GROUP BY (l.205-209).
2. **`merge_common_lists()`** (l.21-38): fusiona listas con elementos comunes en grupos transitivos. Ej. `[[a,b],[b,c],[d,e]]` → `[[a,b,c],[d,e]]`. Controlable por config `data_merge.merge_lists` (default True). Si False, cada match de regla es su propio grupo (l.252-256).
3. Salta grupos que sean **subconjunto** de un grupo ya existente (l.272-273) — evita duplicar groups. Compara contra `done_groups_res_ids` (todos los grupos ya creados, l.242-248).
4. Crea `data_merge.group` + sus `data_merge.record`, elige master (`_elect_master_record`), aplica `create_threshold` (borra el grupo si está por debajo) y, en automatic, fusiona si supera `merge_threshold`.

### Algoritmo de similitud — `data_merge.group._compute_similarity` (l.41-60)
**OJO: la similitud NO usa las reglas de match.** Se calcula sobre TODOS los campos `char` del modelo (`_get_similarity_fields` l.36-39):
```
similarity = min(1, |campos_char_idénticos_en_todos| / |total_campos_char|)
```
- Hace intersección de los `set(record.items())` de todos los registros del grupo (l.54-56).
- `divergent_fields` = campos char que NO son idénticos (l.58-59), usado para mostrar diferencias.
- Es un coeficiente de "cuántos campos texto coinciden exactamente", **no** una distancia de Levenshtein/fuzzy. No considera unaccent (hay un comentario YTI/TODO al respecto, l.53). Grupo de 1 record → similitud = 1.

### Master record — `_elect_master_record` (l.75-96)
Busca un `_elect_method` en el modelo destino; si no, usa el genérico `_elect_method` (l.98-106) = **el registro más antiguo** (`sorted('create_date')[0]`). Modelos como `res.partner` y `crm.lead` lo sobrescriben.

### Merge — `merge_records()` (l.118-172)
1. `master_record` = el marcado `is_master` o el primero.
2. Busca `_merge_method` en el modelo; si no, usa el genérico (l.200-213) = `_update_foreign_keys(destination=master, source=otros)`.
3. `_update_foreign_keys` (`data_cleaning/models/data_merge_record.py:370-469`, `class DataMergeRecord`): vía SQL, reapunta TODAS las FKs (`pg_constraint`) que apuntaban a los source hacia el master, incluyendo M2M (detecta tabla relación por nº columnas) y campos company-dependent (jsonb merge). Maneja `UniqueViolation`/`IntegrityError` con savepoint + log warning (no aborta).
4. `_merge_additional_models` (l.472-540): reapunta `ir_attachment`, `mail_activity`, `ir_model_data`, `mail_message` (por res_id+model). Fusiona followers sin duplicar (`_merge_mail_followers`).
5. `_post_merge` (l.215-223): archiva (`removal_mode=archive`) o borra (`delete`) los source.
6. Loguea snapshot en el chatter del master (`_log_merge`, solo si el modelo es `mail.thread`).

### Contextual merge action — botón "Merge" en list view
`data_cleaning/models/ir_model.py:6-89`, `IrModel`. Cada `ir.model` puede tener un server action de merge contextual:
- `action_merge_contextual_enable()` (l.46-87): crea un `ir.actions.server` (state=code) con binding list view, código `env['data_merge.record'].action_deduplicates(records)`, restringido a `base.group_system`. Le da un xml_id para que no cuente como customization en cloc.
- `_disable_data_merge` (class attr): modelos con merge propio (res.partner, crm.lead) lo ponen True → `hide_merge_action`.
- `action_deduplicates()` (`data_cleaning/models/data_merge_record.py:589-669`, `DataMergeRecord.action_deduplicates`): al pulsar el botón con ≥2 registros seleccionados, crea on-the-fly una `data_merge.model` con `is_contextual_merge_action=True` (`active=False`) + group + records, y redirige a la vista deduplicate.

---

## Crons (3 en EE + 1 en CE)

| Cron | Módulo | Hora | Método |
|---|---|---|---|
| Data Recycle: Clean Records | data_recycle (CE) | 03:00 diario | `data_recycle.model._cron_recycle_records()` |
| Data Cleaning: Clean Records | data_cleaning (EE) | 03:00 diario | `data_cleaning.model._cron_clean_records()` |
| Data Merge: Find Duplicate Records | data_cleaning (EE) | 01:00 diario | `data_merge.model._cron_find_duplicates()` |
| Data Merge: Cleanup Records | data_cleaning (EE) | 01:30 diario | `data_merge.group._cron_cleanup()` |

Source: `data_recycle/data/ir_cron_data.xml`, `data_cleaning/data/data_cleaning_cron.xml`.

- `_cron_*` llaman al motor con `batch_commits=True` y luego notifican.
- `_cron_cleanup` (`data_cleaning/models/data_merge_group.py:228-284`, `DataMergeGroup._cron_cleanup`) borra en lotes de 100 (`DM_CRON_BATCH_SIZE`): records cuyo original fue borrado/archivado, y groups con ≤1 record sin descartados.

---

## Seguridad

- **Todo restringido a `base.group_system`** (admin de sistema). No hay grupo funcional intermedio. Source: `data_recycle/security/ir.model.access.csv`, `data_cleaning/security/ir.model.access.csv`.
- Reglas multi-company globales en `data_cleaning/security/ir_rule.xml`: `data_cleaning.record` y `data_merge.record` filtran por `company_id in company_ids + [False]`.
- Los motores corren todo en `sudo()` (search, validate, merge) — la regla `_disable_data_merge` y `_prevent_merge` son la única barrera funcional.

## Report bonus: ir.attachment.report (EE)
`data_cleaning/report/data_storage.py`: vista SQL `_auto=False` que agrega `file_size` de `ir_attachment` (type=binary) por `res_model`+`res_id`. Es el report "Storage" para detectar registros que consumen mucho almacenamiento (no es estrictamente recycle/clean/merge, pero vive en data_cleaning).
