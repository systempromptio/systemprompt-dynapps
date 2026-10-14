# Data Tools — Integrations

Cómo `data_merge` (dentro de `data_cleaning`, EE) se conecta con CRM, Helpdesk, Project, Stock/Accounting y UTM mediante 5 bridges oficiales. Todos dependen de `data_cleaning` + el módulo funcional, **ninguno depende de un `data_merge` base** (que no existe; ver `core.md`).

---

## 1. Bridges oficiales — qué aporta cada uno

Dos mecanismos de extensión que usan los bridges:
- **(A) Predefinir `data_merge.model` + `data_merge.rule`** vía data XML → reglas de deduplicación listas para el cron.
- **(B) Registrar contextual merge action** vía `<function model="ir.model" name="action_merge_contextual_enable" eval="[[<model_refs>], '<module>']"/>` → botón "Merge" en list view (server action por modelo).
- **(C) Sobrescribir `_merge_method` / `_elect_method`** en el modelo funcional para una fusión específica.

### `data_merge_crm` — dedup de CRM
Source: `data_merge_crm/models/crm_lead.py`, `data_merge_crm/data/data_merge_data.xml`, `data_merge_crm/data/ir_model_data.xml`.
- **(A) data_merge.model predefinidas**:
  - `crm.lead` ("Lead/Opportunity"), domain `[('probability','<',100)]`. Rules: `partner_id`, `email_from`, `partner_name`, `contact_name` (exact).
  - `crm.tag`: rule por `name`.
  - `crm.lost.reason`: rule por `name`.
- **(B) Contextual merge** habilitado para `crm.lost.reason` y `crm.tag`.
- **(C)** `crm.lead._merge_method` → `merge_opportunity(auto_unlink=False)`; bloquea fusionar oportunidades ganadas (DTG20). `_elect_master` = `_sort_by_confidence_level`. `crm.lead` tiene `_disable_data_merge=True` (no merge genérico).

### `data_merge_helpdesk` — dedup de Helpdesk
Source: `data_merge_helpdesk/models/helpdesk_ticket.py`, `data_merge_helpdesk/data/ir_model_data.xml`.
- **(B) Contextual merge** para `helpdesk.ticket` y `helpdesk.tag`. (No predefine data_merge.model/rule; el ticket se fusiona vía menú contextual).
- **(C)** `helpdesk.ticket._merge_method`: reapunta FKs (`_update_foreign_keys`) y consolida los `sla_status_ids` quedándose el de **deadline mínimo por SLA** (`min(status, key=deadline or datetime.max)`), y recompone `sla_ids`. `post_merge=True`, `log_chatter=True`.

### `data_merge_project` — dedup de Project
Source: `data_merge_project/data/ir_model_data.xml` (¡sin models/!).
- **(B) Contextual merge** para `project.tags` y `project.task`. Es el bridge más simple: solo registra el botón "Merge" para estos dos modelos. Usa el `_merge_method` genérico (reapuntar FKs). No predefine reglas ni sobrescribe lógica.

### `data_merge_stock_account` — warning de valoración
Source: `data_merge_stock_account/models/data_merge_model.py`.
- **No predefine reglas ni contextual action**. Su única función es un `@api.onchange('res_model_id')` sobre `data_merge.model`: si eliges `product.product` o `product.template`, muestra un **warning** avisando de que fusionar productos impacta la valoración de inventario y puede requerir rectificación manual. Es un guardarraíl, no funcionalidad de merge.

### `data_merge_utm` — dedup de UTM
Source: `data_merge_utm/data/data_merge_data.xml`, `data_merge_utm/data/ir_model_data.xml`.
- **(A) data_merge.model predefinidas** para `utm.campaign`, `utm.medium`, `utm.source` (cada una con rule por `name`, exact).
- **(B) Contextual merge** para los tres modelos UTM.
- Llama `_update_default_rules` para promocionar las reglas a `accent` si hay unaccent.

### Resumen comparativo

| Bridge | data_merge.model predefinida | Contextual action | `_merge_method` custom | Particularidad |
|---|---|---|---|---|
| crm | crm.lead, crm.tag, crm.lost.reason | lost.reason, crm.tag | crm.lead (merge_opportunity) | bloquea won; `_disable_data_merge` en lead |
| helpdesk | — | ticket, tag | helpdesk.ticket (SLA consolidation) | min deadline por SLA |
| project | — | task, tags | genérico | el más simple, solo botón |
| stock_account | — | — | — | solo warning valoración en product |
| utm | campaign, medium, source | campaign, medium, source | genérico | rules promocionadas a accent |

---

## 2. Diagrama

```
                    ┌──────────────────────────────┐
                    │  data_recycle (CE, app)      │  archive/unlink registros viejos
                    │  + menu_data_cleaning_root   │
                    └──────────────┬───────────────┘
                                   │ auto_install + depends
                    ┌──────────────▼───────────────┐
                    │  data_cleaning (EE)          │
                    │  ├─ data.cleaning.* (formato)│
                    │  └─ data.merge.*   (dedup)   │  ← merge vive AQUÍ
                    └──────────────┬───────────────┘
                                   │ depends: data_cleaning + módulo funcional
        ┌──────────┬──────────────┼──────────────┬─────────────┐
        ▼          ▼              ▼              ▼             ▼
   data_merge_  data_merge_   data_merge_   data_merge_    data_merge_
      crm        helpdesk       project      stock_account     utm
   (A)(B)(C)     (B)(C)         (B)          (onchange)      (A)(B)
   crm.lead      ticket+SLA     task/tags    product warning  campaign/
   crm.tag       tag                                          medium/source
   lost.reason

  (A)=predefine data_merge.model+rule  (B)=contextual merge action  (C)=_merge_method custom
```

Flujo de un merge:
```
cron 01:00 _cron_find_duplicates  →  find_duplicates() por modelo
   ├─ SQL GROUP BY rule.field HAVING COUNT>1  →  arrays de IDs
   ├─ merge_common_lists()  →  grupos transitivos
   ├─ crea data_merge.group + data_merge.record
   ├─ _elect_master_record()  (modelo._elect_method o genérico=más antiguo)
   ├─ _compute_similarity (campos char idénticos / total char)
   └─ si automatic y similarity*100 ≥ merge_threshold:
        merge_records() → _merge_method (modelo o genérico)
           ├─ _update_foreign_keys (reapunta FKs vía SQL)
           ├─ _merge_additional_models (attachments, activities, messages...)
           ├─ _post_merge (archive/delete source según removal_mode)
           └─ _log_merge (chatter del master, si mail.thread)
cron 01:30 _cron_cleanup  →  borra groups/records huérfanos
```

---

## 3. Modelos clave (extensiones de los bridges)

- `crm.lead` (`_inherit`, data_merge_crm): `_disable_data_merge=True`, `_merge_method`, `_elect_master`.
- `helpdesk.ticket` (`_inherit`, data_merge_helpdesk): `_merge_method` con consolidación SLA.
- `data_merge.model` (`_inherit`, data_merge_stock_account): `_onchange_res_model_id` con warning de productos.
- `ir.model` (`_inherit`, en data_cleaning, no en bridges): `action_merge_contextual_enable/disable`, `hide_merge_action`, `is_merge_enabled`, `ref_merge_ir_act_server_id`. Es el modelo que los bridges invocan vía `<function>` para registrar el botón.
- `res.partner` (`_inherit`, en data_cleaning base): `_disable_data_merge=True`, `_merge_method` vía wizard estándar, `_elect_method` (activo + más antiguo).

---

## 4. Config

- **Reglas predefinidas**: data XML de cada bridge crea `data_merge.model` + `data_merge.rule`. Tras instalar, aparecen en Settings → Technical → Data Merge (admin only).
- **`_update_default_rules`** (`data_cleaning/models/data_merge_rule.py:33-35`, `DataMergeRule._update_default_rules`): función llamada por data XML para promocionar reglas a `match_mode='accent'` si la BD tiene unaccent. Sin unaccent quedan en `exact`.
- **Habilitar/deshabilitar contextual merge** manualmente: botón en el form de `ir.model` (`action_merge`), o `action_merge_contextual_enable([model_ids], module)` vía código. Crea un `ir.actions.server` bindeado a la list view.
- **Parámetros de sistema** (`ir.config_parameter`):
  - `data_merge.merge_lists` (default `True`): activa el agrupado transitivo `merge_common_lists`.
  - `data_merge.compute_references` (default `True`): activa el cómputo de `used_in` (caro; ver DTG25).
- **Por data_merge.model**: `merge_threshold` (auto-merge ≥), `create_threshold` (descarta ≤, dev only), `removal_mode` (archive/delete), `merge_mode` (manual/automatic), `mix_by_company` (cross-company).

---

## 5. Gotchas de integración (DTG relevantes)

Detalle completo en `gotchas.md`:
- **DTG1** — no existe módulo `data_merge`; vive en `data_cleaning`.
- **DTG2** — `data_cleaning` auto-instala `data_recycle` (CE).
- **DTG14/DTG15** — similitud NO usa las reglas de match; es exact-match de campos char, no fuzzy. Afecta a todos los bridges en automatic.
- **DTG16** — `accent` requiere extensión PG `unaccent` (utm/crm promocionan a accent).
- **DTG18** — master genérico = más antiguo; crm/helpdesk/partner lo sobrescriben (DTG19).
- **DTG19** — res.partner y crm.lead no usan merge genérico (botón oculto).
- **DTG20** — crm.lead no fusiona oportunidades ganadas.
- **DTG24** — reapuntado de FKs tolera fallos en silencio.
- **DTG32** — el botón contextual crea data_merge.model inactivas que se acumulan.
- **stock_account específico**: fusionar productos solo avisa con warning; la valoración de inventario hay que rectificarla a mano (no hay lógica de revaloración).

---

## 6. v19

Ver `v19_changes.md` para el detalle. Resumen de impacto en bridges:
- Todos los bridges siguen existiendo en `odoo_enterprise/19/data_merge_*` con la misma arquitectura (A/B/C).
- `crm_lead.py` del bridge crm es **idéntico** v18→v19 (sin cambios).
- El cambio que más afecta a los bridges es en `ir_model.py`: `groups_id` → `group_ids` en el server action contextual (`action_merge_contextual_enable`). Un bridge custom v18 que copie ese vals fallará en v19.
- Renombrado de clases Python (`DataMergeModel` → `Data_MergeModel`, etc.) y `_sql_constraints` → `models.Constraint`: irrelevante para data (XML usa `_name`), relevante si un bridge hereda por clase.
