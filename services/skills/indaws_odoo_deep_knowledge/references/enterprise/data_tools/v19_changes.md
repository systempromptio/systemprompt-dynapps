# Data Tools — v19 changes

Diffs verificados de v18 (`odoo18/addons` CE, `odoo_enterprise/18` EE) vs v19 (`odoo19/addons` CE, `odoo_enterprise/19` EE). La arquitectura (3 subsistemas, distribución de módulos, merge dentro de data_cleaning, 4 crons, 5 bridges) **no cambia**. Los cambios son de framework/API.

---

## Cambios transversales (todos los módulos)

### Clases Python renombradas al patrón `Model_Submodel`
v18 → v19: `DataRecycleModel`→`Data_RecycleModel`, `DataRecycleRecord`→`Data_RecycleRecord`, `DataCleaningModel`→`Data_CleaningModel`, `DataCleaningRule`→`Data_CleaningRule`, `DataMergeModel`→`Data_MergeModel`, `DataMergeGroup`→`Data_MergeGroup`, `DataMergeRecord`→`Data_MergeRecord`, `DataMergeRule`→`Data_MergeRule`, `Partner`→`ResPartner`.
- Irrelevante para data XML (usa `_name`). **Relevante** si un módulo custom hereda por nombre de clase Python o llama `super(DataMergeModel, ...)`.

### `_sql_constraints` → `models.Constraint`
Las constraints declarativas pasan al nuevo estilo atributo de clase:
```python
# v18
_sql_constraints = [('check_notif_freq', 'CHECK(notify_frequency > 0)', 'msg')]
# v19
_check_notif_freq = models.Constraint('CHECK(notify_frequency > 0)', "msg")
```
Afecta a `data_recycle.model`, `data_cleaning.model`, `data_merge.model` (uniq_name + check_notif_freq), `data_merge.rule` (uniq_model_id_field_id).

### `groups_id` → `all_group_ids` en domains de notify_user_ids
El domain de `notify_user_ids` cambia de `[('groups_id', 'in', ...)]` a `[('all_group_ids', 'in', ...)]` en recycle/cleaning/merge model. Refleja el refactor de grupos de usuario de v19 (implied/all groups).

### `self._cr` → `self.env.cr`
Todas las llamadas SQL directas migran de `self._cr.execute/fetchall` a `self.env.cr.execute/fetchall` (cleaning_model, merge_model, merge_record). Cosmético pero rompe overrides que llamen `self._cr` esperando el atributo viejo… (sigue existiendo como alias, pero el código nativo ya no lo usa).

### `notify_author=True` eliminado
Las llamadas `message_notify` en recycle/cleaning/merge ya NO pasan `notify_author=True`. El autor del cron deja de auto-notificarse.

---

## data_recycle (CE)

`data_recycle_model.py`:
- **`config['test_enable']`/`config['test_file']` → `modules.module.current_test`** para detectar entorno de test (l.110).
- **Migración a `odoo.fields.Domain`**: `from odoo.osv import expression` → `from odoo.fields import Domain`. El domain se construye con `Domain(...)`/`Domain.TRUE` y se compone con `&=` en vez de `expression.AND` (l.119,126). Un override custom que use `expression.AND` sobre `rule_domain` debe migrarse.
- `import _` ahora viene de `odoo.tools` (no de `odoo`).
- `__manifest__.py`: añade `'author': 'Odoo S.A.'`.

`data_recycle_record.py`:
- **`toggle_active()` → `action_archive()`** en `action_validate` para la acción archive (l.81). Cambio semántico: `action_archive` solo archiva (no togglea), evita reactivar registros ya activos por error.
- `recycle_model_id` añade `index='btree_not_null'`.

---

## data_cleaning (EE) — cleaning

`data_cleaning_model.py`: solo los cambios transversales (clase, constraint, `all_group_ids`, `env.cr`, sin `notify_author`). Lógica de motor de cleaning **sin cambios funcionales**.

`data_cleaning_rule.py`: `cleaning_model_id` añade `index=True`. `ACTIONS_PYTHON`/`ACTIONS_SQL` y la lógica de acciones **sin cambios**.

`data_cleaning_record.py`: sin cambios relevantes (solo rename de clase si aplica).

---

## data_cleaning (EE) — merge

`data_merge_model.py`:
- **`create_threshold` pierde `groups='base.group_no_one'`** (l.78): ahora el "Suggestion Threshold" es visible sin modo dev.
- **`find_duplicates`: `_where_calc(domain)` → `_search(domain, bypass_access=True)`** (l.193→196). Cambio importante: en v19 la query base de identificación pasa por `_search` con bypass explícito de ACLs. Un override custom que dependa de `_where_calc` debe adaptarse.
- `super(DataMergeModel, self)` → `super()`.

`data_merge_group.py`:
- Migración a `odoo.fields.Domain` en `discard_records` (`expression.AND` → `Domain & Domain`).
- `getattr(model, '_merge_method')` → `model._merge_method` y `getattr(...'_elect_method')` → `self.env[...]._elect_method` (acceso directo). Algoritmo de similitud, master election, merge, cleanup: **sin cambios funcionales**.

`data_merge_record.py`:
- **`_search_company_id`: operadores permitidos cambian** de `ALLOWED_COMPANY_OPERATORS` (8 operadores) a solo `('in', 'ilike', 'like', 'any')` (l.67). Refleja el nuevo motor de Domain de v19 (los `not in`/`!=`/`=` se normalizan a `in`/`any` antes de llegar al search method). Un código que filtrase `data_merge.record` por `company_id` con `!=`/`not in` puede comportarse distinto.
- Migración completa a `odoo.fields.Domain`: `FALSE_DOMAIN`→`Domain.FALSE`, `OR`→`Domain.OR`, retornos `[('id','in',...)]` → `Domain('id','in',...)`.
- `expression([...], Model)` → `Model.with_context(active_test=False)._search([...])` en la construcción de la subquery de company.

`data_merge_rule.py`: `model_id` añade `index=True`. Constraint declarativa al nuevo estilo. Lógica `_available_match_modes`/`_update_default_rules` **sin cambios** (accent sigue dependiendo de unaccent).

`ir_model.py` (clave para bridges):
- **`groups_id` → `group_ids`** en el vals del server action contextual (`action_merge_contextual_enable`, l.52). Un bridge custom v18 que copie ese diccionario fallará en v19.
- `getattr(self.env[model.model], ...)` → `getattr(self.env.get(model.model), ...)` en `_compute_hide_merge_action` (l.30): usa `.get` para tolerar modelos no cargados.

`res_partner.py`: solo rename `Partner` → `ResPartner`. `_merge_method`/`_elect_method` sin cambios.

---

## Bridges (EE) en v19

Los 5 bridges (`data_merge_crm/helpdesk/project/stock_account/utm`) existen en `odoo_enterprise/19/` con idéntica estructura.
- `data_merge_crm/models/crm_lead.py`: **idéntico** v18→v19 (diff vacío).
- El cambio que más impacta a bridges custom es `groups_id`→`group_ids` en `ir_model.action_merge_contextual_enable` y la migración a `Domain`.

---

## Checklist de migración para módulos custom sobre data tools

1. ¿Heredas alguna clase por nombre Python o `super(DataXxx, self)`? → renombra a `Data_Xxx` / `super()`.
2. ¿Usas `_sql_constraints`? → migra a `models.Constraint` (recomendado; el viejo aún funciona con deprecation).
3. ¿Tu domain de `notify_user_ids` usa `groups_id`? → `all_group_ids`.
4. ¿Construyes domains con `odoo.osv.expression` (`AND`/`OR`/`FALSE_DOMAIN`)? → migra a `odoo.fields.Domain`.
5. ¿Override de `find_duplicates` que use `_where_calc`? → `_search(domain, bypass_access=True)`.
6. ¿Bridge que registre contextual merge copiando vals con `groups_id`? → `group_ids`.
7. ¿Filtras `data_merge.record.company_id` con `!=`/`not in`/`not like`? → revisa, el search method solo acepta `in`/`ilike`/`like`/`any`.
8. ¿Llamas `toggle_active()` esperando el comportamiento de recycle archive? → ahora es `action_archive()`.
