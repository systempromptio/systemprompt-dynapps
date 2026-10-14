# Reporting / BI — Cambios v19 (vs 18.0)

Resumen de diffs verificados contra `odoo19`. La regla general: **el render PDF NO cambia
(sigue wkhtmltopdf), pero la capa de agregación BI sí cambia bastante** (read_group deprecada, ORM
movido a `odoo/orm/`, formatted_read_group + grouping sets nuevos).

Para gotchas que siguen vigentes en v19 ver `gotchas.md` (sección v19, RPTG21-22 + nota de que
RPTG1-20 y la mayoría de RPTG-SQL/BI aplican igual).

---

## 1. QWeb → PDF: prácticamente igual

Verificado en `odoo19/odoo/addons/base/models/ir_actions_report.py`:
- `report_type` idéntico (`qweb-html`/`qweb-pdf`/`qweb-text`, default `qweb-pdf`, líneas 170-174).
- `report.paperformat`: mismos campos/defaults (`odoo/addons/base/models/report_paperformat.py:171-189`, `report_paperformat.default`, v18 (en v19 la clase es `ReportPaperformat`)).
- Mismos estados `install/ok/upgrade/workers/broken`, misma comprobación >=0.12.0 / zoom >=0.12.2.
- **Sigue siendo wkhtmltopdf. NO hay Chromium/Playwright nativo** (RPTG21).

Único cambio estructural relevante (ya en `core.md` §9 y RPTG22):
- `_run_wkhtmltopdf` pasa a helper a nivel de módulo con `subprocess.run(..., check=False)`.
- Nueva rama que lanza `UserError("Tried to convert multiple documents in wkhtmltopdf using
  unpatched QT")` / `unittest.SkipTest` en multi-documento con QT no parcheado (RPTG22). En v18 no
  existe.

---

## 2. ORM movido a `odoo/orm/` (cambio de paths)

En v19 el ORM se reparte en el paquete `odoo/orm/`:
- `_read_group` / `read_group` / `formatted_read_group` → `odoo/orm/models.py` (NO `odoo/models.py`).
- Campos → `odoo/orm/fields.py`, `fields_numeric.py`, `fields_relational.py`, etc.

Impacto al verificar/citar código: las líneas de los gotchas RPTG-BI que en v18 apuntan a
`odoo/models.py` y `odoo/fields.py` en v19 hay que buscarlas en `odoo/orm/models.py` /
`odoo/orm/fields.py`. La lógica es equivalente salvo lo de abajo.

---

## 3. `read_group` DEPRECADA → `formatted_read_group` / `_read_group`

Cambio más importante para BI. `read_group` lleva decorador de deprecación
(`odoo/models.py:2748-2749`, `BaseModel._read_group_format_result_properties`):
```python
@api.deprecated("Since 19.0, read_group is deprecated. Please use _read_group in the backend code "
                "or formatted_read_group for a complete formatted result")
def read_group(self, domain, fields, groupby, offset=0, limit=None, orderby=False, lazy=True):
```
Tres niveles ahora:

| Método | Devuelve | Cuándo | Fuente v19 |
|---|---|---|---|
| `_read_group(domain, groupby, aggregates, having, offset, limit, order)` | lista de **tuplas** | backend, alto rendimiento | `odoo/models.py:1861`, `BaseModel.name_create` |
| `formatted_read_group(domain, groupby, aggregates, having, ...)` | lista de **dicts** formateados para el web client | reemplazo de read_group con formato | `addons/web/models/models.py:790`, `Base.search_panel_select_multi_range` |
| `formatted_read_grouping_sets(...)` | grupos por **múltiples grouping sets** en una llamada | **NUEVO** en v19 | `addons/web/models/models.py:690`, `Base.search_panel_select_multi_range` |

**Diferencias clave de la nueva API vs la vieja `read_group`**:
- Firma con `aggregates=` (lista tipo `'amount:sum'`) y `groupby=` separados, igual que `_read_group`
  (`odoo/models.py:1861-1869`, `BaseModel.name_create`). Ya NO hay parámetro `fields` mezclando medidas y groupbys.
- **NO hay `lazy`**: `formatted_read_group`/`_read_group` agrupan por TODOS los groupbys de una
  (adiós al gotcha RPTG-BI1 de "solo el primero"). El lazy-loading por niveles ahora lo gestiona el
  cliente con `formatted_read_grouping_sets` (un read por grouping set en vez de el truco `__context`).
- Granularidades de fecha ampliadas: además de day/week/month/quarter/year, v19 soporta partes
  enteras `year_number`, `quarter_number`, `month_number`, `iso_week_number`, `day_of_year`,
  `day_of_month` (`addons/web/models/models.py:813-830`, `Base.search_panel_select_multi_range`).

**Migración**: código custom que llama `record.read_group(domain, ['amount:sum'], ['team_id'])`
sigue funcionando (deprecado, warning). Para silenciarlo y ganar la semántica correcta:
`record._read_group(domain, ['team_id'], ['amount:sum'])` (tuplas) o `formatted_read_group(...)`
(dicts). OJO: el ORDEN de args cambia (groupby antes que aggregates en la nueva API).

---

## 4. `aggregator` (ya no `group_operator`) — consolidado

`group_operator` se deprecó en **v18** (no v19); en v19 sigue el shim con DeprecationWarning
(`odoo/fields.py:487-489`, `Field._setup_attrs`):
```python
if 'group_operator' in attrs:
    warnings.warn("Since Odoo 18, 'group_operator' is deprecated, use 'aggregator' instead", ...)
    attrs['aggregator'] = attrs.pop('group_operator')
```
La lógica de agregación en `_read_group` usa `field.aggregator` directamente
(`odoo/models.py:2832-2833`, `BaseModel.read_group`). Sin cambios de comportamiento respecto a v18; solo recordar que en
v19 el código vive en `odoo/orm/fields.py`.

---

## 5. Reports SQL `_auto=False` — sin cambios de mecanismo

Los dos mecanismos (subquery `_table_query` con objetos `SQL`, e `init()`+`CREATE VIEW`) siguen
coexistiendo en v19. `sale.report`/`account.invoice.report` mantienen el patrón de hooks
`_select`/`_from`/`_where`/`_group_by`. Todos los gotchas RPTG-SQL aplican igual.

Cambio colateral relevante para reports: en v19 varios filtros/campos del ecosistema account
cambian (ej. el filtro `deprecated=False` se elimina de todos los domains, `_sql_constraints` →
`models.Constraint`), lo que puede romper reports SQL custom que asumían esas columnas/constraints.
Detalle en `functional/account/v19_changes.md` y `cheatsheet.md`.

---

## 6. account.report (EE) en v19

- Sigue exportando PDF vía `_run_wkhtmltopdf` (mismo motor, mismo RPTG22 multi-doc) y XLSX vía
  xlsxwriter.
- El cálculo de engines (domain/tax_tags/aggregation/account_codes/external/custom) se mantiene; los
  cambios v19 de account afectan más al modelo de datos (extra_tax_data Json, return types) que al
  motor de reporting en sí. Ver `cross-module/account_reports/` (ARG) para el detalle.

---

## Resumen accionable para migrar v18 → v19

1. PDF/QWeb: nada que tocar salvo asegurar wkhtmltopdf con **QT parcheado** para multi-documento
   (RPTG22).
2. Cualquier `record.read_group(...)` custom: migrar a `_read_group`/`formatted_read_group`
   (firma distinta, sin `lazy`, groupby antes que aggregates).
3. Paths al verificar código: `odoo/orm/models.py` y `odoo/orm/fields.py`, no `odoo/models.py`.
4. `group_operator=` en campos: cambiar a `aggregator=` (warning, no error).
5. Reports SQL custom: revisar columnas que dependían de campos account eliminados en v19.
