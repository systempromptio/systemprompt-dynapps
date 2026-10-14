<!-- verified: v18.0 @ 2026-06 -->
# Tools — Essential Utilities (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/tools/` (~40 modules)

This reference covers the functions developers use most frequently. Organized by module.

## Table of Contents

1. [Float Utilities](#float-utilities)
2. [Date Utilities](#date-utilities)
3. [Mail & HTML](#mail--html)
4. [Image Processing](#image-processing)
5. [Safe Eval](#safe-eval)
6. [Formatting & Locale](#formatting--locale)
7. [ORM Cache](#orm-cache)
8. [SQL Utilities](#sql-utilities)
9. [Collections & Misc](#collections--misc)
10. [Translation](#translation)
11. [File I/O](#file-io)
12. [JSON](#json)
13. [Profiling](#profiling)
14. [Data Population](#data-population)

---

## Float Utilities

`from odoo.tools import float_round, float_compare, float_is_zero, float_repr`

**Never use `==` or `round()` on floats in Odoo.** Always use these:

```python
float_round(value, precision_digits=None, precision_rounding=None, rounding_method='HALF-UP')
```
Round to N decimal places. `rounding_method`: `'HALF-UP'` (default), `'HALF-DOWN'`, `'HALF-EVEN'`, `'UP'`, `'DOWN'`.

```python
float_compare(value1, value2, precision_digits=None, precision_rounding=None) -> int
```
Returns `-1` (v1 < v2), `0` (equal), `1` (v1 > v2) within precision.

```python
float_is_zero(value, precision_digits=None, precision_rounding=None) -> bool
```
True if value is zero within precision.

```python
float_repr(value, precision_digits) -> str
```
String with fixed decimal places: `float_repr(1.5, 3)` → `'1.500'`.

```python
float_split(value, precision_digits) -> (units:int, cents:int)   # float_utils.py:240
```
Parte el valor en entero y decimales según la precisión (p.ej. `float_split(3.14, 2)` → `(3, 14)`).

```python
# Usage with currency
float_compare(amount, 0, precision_rounding=currency.rounding)
float_is_zero(amount, precision_rounding=currency.rounding)
float_round(amount, precision_rounding=currency.rounding)
```

---

## Date Utilities

`from odoo.tools import date_utils`

```python
date_utils.start_of(value, granularity)  # 'year','quarter','month','week','day','hour'
date_utils.end_of(value, granularity)
```
Returns first/last moment of the period. Works with both `date` and `datetime`.

```python
date_utils.get_month(date)       # (first_day, last_day) of month
date_utils.get_quarter(date)     # (first_day, last_day) of quarter
date_utils.get_quarter_number(d) # 1-4
date_utils.get_fiscal_year(date, day=31, month=12)  # (start, end) customizable
```

```python
date_utils.add(value, **kwargs)       # add relativedelta: add(date, months=1, days=5)
date_utils.subtract(value, **kwargs)  # subtract relativedelta
```

```python
date_utils.date_range(start, end, step)  # generator yielding dates at intervals
# step is a relativedelta: date_range(date(2024,1,1), date(2024,12,31), relativedelta(months=1))
```

```python
date_utils.get_timedelta(qty, granularity)  # date_utils.py:89 → relativedelta
# granularity: 'hour'|'day'|'week'|'month'|'year'. Ej: get_timedelta(2, 'month')
```
Atajo para construir un `relativedelta` por cantidad+granularidad (útil en crons/plazos configurables).

---

## Mail & HTML

`from odoo.tools import html_sanitize, html2plaintext, plaintext2html, email_normalize, email_split`

### HTML processing

```python
html_sanitize(src, silent=True, sanitize_tags=True, sanitize_attributes=False,
              sanitize_style=False, sanitize_form=True, sanitize_conditional_comments=True,
              strip_style=False, strip_classes=False, output_method="html")
```
Removes unsafe tags/attributes. **Always use on user-provided HTML.**
NOTE: `sanitize_attributes` defaults to `False` (not `True`).

```python
html2plaintext(html)           # convert HTML to readable plain text
html_to_inner_content(html)    # → texto interno colapsado a una línea (mail.py:430; para previews/índices)
plaintext2html(text)           # convert plain text to HTML (wraps in <p>)
is_html_empty(html_content)    # True if HTML has no visible content
append_content_to_html(html, content)    # append to HTML body
prepend_html_content(html_body, content) # prepend to HTML body
```

### Email utilities

```python
email_normalize(text, strict=True)  # lowercase, strip, validate → 'user@domain.com' or False
email_split(text)              # split comma-separated emails → ['a@b.com', 'c@d.com']
email_split_tuples(text)       # → [('Name', 'email@domain.com'), ...]
email_split_and_format(text)   # → ['"Name" <a@b.com>', ...] formateado listo para cabecera (mail.py:700)
email_domain_extract(email)    # → 'domain.com'
email_domain_normalize(domain) # → lowercase domain
formataddr((name, email))      # → '"Name" <email@domain.com>'
```

---

## Image Processing

```python
from odoo.tools import image_process            # re-exported from odoo.tools
# The rest are NOT re-exported by odoo.tools — import from odoo.tools.image:
from odoo.tools.image import (
    base64_to_image, image_to_base64, image_data_uri, image_guess_size_from_field_name,
)
```

```python
image_process(source, size=(0, 0), verify_resolution=False, quality=0, expand=False,
              crop=None, colorize=False, output_format='', padding=False)
```
Main entry point. `source` is base64 bytes. `size=(width, height)`, 0=auto. Returns base64 bytes.

```python
base64_to_image(base64_source)         # → PIL.Image
image_to_base64(image, output_format, **params)  # PIL.Image → base64 bytes (2nd arg is output_format)
image_data_uri(base64_source)          # → 'data:image/png;base64,...' for CSS/HTML embedding
image_guess_size_from_field_name('image_512')  # → (512, 512)
```

Clase OO `ImageProcess` (pipeline; `image_process` es su atajo) — `odoo/tools/image.py:60`, `class ImageProcess`:
```python
from odoo.tools.image import ImageProcess
img = ImageProcess(source_b64, verify_resolution=True)   # :62
img.resize(max_width=1024, max_height=0, expand=False)   # :170  (0 = auto por ratio)
img.crop_resize(max_width=200, max_height=200, center_x=0.5, center_y=0.5)  # :205
img.image_quality(quality=80, output_format='WEBP')      # :109 → base64 bytes final
```
Helpers extra (`from odoo.tools.image import ...`):
```python
binary_to_image(source)            # bytes → PIL.Image (image.py:429)
image_fix_orientation(image)       # corrige EXIF orientation (:392)
is_image_size_above(b64_a, b64_b)  # ¿a supera el límite de tamaño de b? (:513)
average_dominant_color(colors)     # color dominante (usos kanban) (:328)
hex_to_rgb('#1f2a3b') / rgb_to_hex((31,42,59))   # (:596/:601)
```

---

## Query (constructor SQL — overrides de `_search` / `_where_calc` / `read_group`)

`from odoo.tools.query import Query` — el objeto que Odoo pasa/usa para construir el SQL de búsqueda.
Se toca al override `_search`, `_where_calc`, `_apply_ir_rules` o al añadir joins/where custom.

```python
q.add_where(where_clause, where_params=())      # AÑADE una condición WHERE (query.py:106)
q.add_join(kind, alias, table, condition)       # JOIN genérico (:91)
alias = q.left_join(lhs_alias, lhs_col, rhs_table, rhs_col, link)   # LEFT JOIN, devuelve alias (:130)
alias = q.join(lhs_alias, lhs_col, rhs_table, rhs_col, link)       # INNER JOIN (:111)
q.order = SQL('...')                            # fija el ORDER BY (property, :144/148)
q.is_empty()                                    # → True si ya se sabe que no hay resultados (:176)
q.get_result_ids()                              # ejecuta y devuelve los ids (:221)
q.set_result_ids(ids, ordered=True)             # inyecta ids precalculados (evita ejecutar) (:229)
sql = q.select('id') / q.subselect('id')        # → SQL para usar como subconsulta
```
Regla: para filtros dinámicos en un computed `search=`, devuelve un dominio; toca `Query` solo cuando
necesitas SQL/joins que el dominio no expresa.

---

## Safe Eval

`from odoo.tools.safe_eval import safe_eval, test_python_expr`

```python
safe_eval(expr, globals_dict=None, locals_dict=None, mode='eval', nocopy=False,
          locals_builtins=False, filename=None)
```
Evaluates untrusted Python expression in a sandboxed environment. Blocks imports, attribute assignment, and dangerous operations.

```python
# Allowed in safe_eval:
safe_eval("record.name + ' - ' + record.code", {'record': record})
safe_eval("[x for x in range(10)]")
safe_eval("{'key': value}", {'value': 42})

# NOT allowed — the raised exception type depends on what is rejected:
safe_eval("import os")           # SyntaxError (statement in mode='eval')
safe_eval("__import__('os')")    # NameError  (forbidden builtin name)
safe_eval("open('/etc/passwd')") # NameError  (forbidden name)
# (forbidden bytecode raises ValueError; a code object raises TypeError)
```

```python
test_python_expr(expr, mode='eval')  # validate expression safety without executing
const_eval(expr)                     # evaluate ONLY literals (no variables, no functions)
```

---

## Formatting & Locale

`from odoo.tools.misc import formatLang, format_amount, format_date, format_duration, human_size`

```python
formatLang(env, value, digits=2, grouping=True, monetary=SENTINEL, dp=None,
           currency_obj=None, rounding_method='HALF-EVEN', rounding_unit='decimals')
```
Format number per user's locale: `formatLang(env, 1234.5)` → `'1,234.50'` (en_US) or `'1.234,50'` (es_ES).
`digits` defaults to `2`. `monetary` is deprecated (since 13.0). `dp` (decimal-precision
name) or `currency_obj` override `digits`. `rounding_unit`: `'decimals'|'units'|'thousands'|'lakhs'|'millions'`.

```python
format_amount(env, amount, currency)     # monetary formatting with currency symbol
format_date(env, date, date_format=None) # date per locale
format_duration(value)                    # seconds → '1:30:00'
human_size(bytes)                         # → '1.5 MB', '3 GB'
format_decimalized_amount(amount, currency=None)  # → '1.2k', '3.4M' (compacto, misc.py:1620)
```

---

## ORM Cache

`from odoo.tools import ormcache, ormcache_context`

```python
class MyModel(models.Model):
    @ormcache('self.env.uid', 'partner_id')
    def _get_partner_data(self, partner_id):
        # expensive computation
        return self.env['res.partner'].browse(partner_id).read()[0]
```

Cache is per-registry (per-database). Automatically invalidated on registry reload. Cache keys are the decorator arguments evaluated.

```python
@ormcache_context('self.env.uid', keys=('lang',))
def _get_translated(self):
    # cached per user + language
    ...
```

Clear cache: `self.env.registry.clear_cache()`. (`Model.clear_caches()` still
exists but is **deprecated in v18** — it warns and calls `registry.clear_all_caches()`.)

---

## SQL Utilities

`from odoo.tools import SQL`
`from odoo.tools.sql import table_exists, column_exists, create_column, create_index`

### SQL class (safe query building)

```python
from odoo.tools import SQL

query = SQL("SELECT id FROM %s WHERE name = %s", SQL.identifier(table_name), name)
# Parameters are properly escaped
```

### Schema operations (for migration scripts)

```python
table_exists(cr, 'my_table')                          # → bool
column_exists(cr, 'my_table', 'my_column')            # → bool
index_exists(cr, 'my_index_name')                     # → bool

create_column(cr, 'table', 'column', 'varchar')       # add column
rename_column(cr, 'table', 'old_name', 'new_name')    # rename
convert_column(cr, 'table', 'column', 'int4')         # change type

create_index(cr, 'idx_name', 'table', ['col1', 'col2'], method='btree', where='')
create_unique_index(cr, 'idx_name', 'table', ['col1'])
drop_index(cr, 'idx_name', 'table')

add_constraint(cr, 'table', 'constraint_name', 'CHECK (amount >= 0)')
drop_constraint(cr, 'table', 'constraint_name')

add_foreign_key(cr, 'table1', 'col1', 'table2', 'id', 'set null')
set_not_null(cr, 'table', 'column')
drop_not_null(cr, 'table', 'column')

drop_view_if_exists(cr, 'my_view')
```

Introspección / helpers (útiles en migraciones, `tools/sql.py` v18):
```python
constraint_definition(cr, 'table', 'constraint_name')     # → definición SQL de la constraint (:430)
get_foreign_keys(cr, t1, c1, t2, c2, ondelete)            # → FKs que cumplen ese patrón (:489)
check_index_exist(cr, 'index_name')                        # → assert de existencia de índice (:544)
make_index_name('table', 'column')                         # → nombre de índice canónico (:729)
get_depending_views(cr, 'table', 'column')                 # → vistas SQL que dependen (recrear tras ALTER) (:389)
escape_psql(value)                                         # → escapar % y _ para LIKE/ILIKE (:590)
increment_fields_skiplock(records, *fields)                # → +1 atómico sin bloqueo (contadores concurrentes) (:622)
```

---

## Collections & Misc

`from odoo.tools import frozendict, OrderedSet, split_every, topological_sort, unique, groupby, flatten`

### Contexto / entorno (muy usados)
```python
clean_context(context)   # quita default_* y active_* del contexto (misc.py:959) — usar antes de
                         # pasar self.env.context a create()/write() de OTRO modelo (evita defaults heredados)
get_lang(env, lang_code=None)   # → LangData del idioma activo/dado (misc.py:1315): fecha/decimal/dirección
submap(mapping, keys)           # subconjunto de un dict por claves (misc.py:1236)
```

### Data structures

```python
frozendict({'key': 'value'})   # immutable dict (used for env.context)
OrderedSet([3, 1, 2, 1])      # → OrderedSet([3, 1, 2]) — set with insertion order
```

### Iteration utilities

```python
split_every(n, iterable)       # chunk into groups of n: split_every(3, range(7)) → [(0,1,2),(3,4,5),(6,)]
unique(iterable)               # remove duplicates preserving order
flatten([[1,2],[3,[4]]])       # → [1, 2, 3, 4] (flattens FULLY/recursively).
                               # DEPRECATED in v18 (emits a warning). Avoid in new code.
groupby(iterable, key=func)    # like itertools.groupby BUT aggregates ALL elements
                               # with the same key (not just consecutive). Does NOT
                               # sort; uses a defaultdict. Returns dict_items of (key, [elems]).
topological_sort(elements)     # sort by dependencies: {a: [b, c]} → [b, c, a]
partition(pred, elems)          # → (matching, non_matching) lists
reverse_enumerate(lst)          # enumerate from end: [(2, 'c'), (1, 'b'), (0, 'a')]
```

### String & validation

```python
str2bool('yes')                # → True (accepts: 1, true, yes, on)
remove_accents('café')         # → 'cafe'
html_escape('<script>')        # → '&lt;script&gt;'
mod10r('12345')                # → MOD10 checksum (banking)
hmac(env, scope, message, hash_function=hashlib.sha256)  # signed digest; NOTE: scope arg is required
consteq(a, b)                  # constant-time comparison (= hmac.compare_digest)
```

### File utilities

```python
clean_filename('my file (1).txt')  # → 'my_file__1_.txt' (safe for filesystem)
```

---

## Translation

`from odoo import _`
`from odoo.tools.translate import LazyTranslate, html_translate, xml_translate`

```python
_("Untranslated message")                    # mark for translation + translate at runtime
_("Hello %(name)s", name=partner.name)       # with named placeholders (NOT % or .format())

_lt = LazyTranslate('module_name')
STATES = [('draft', _lt("Draft")), ('done', _lt("Done"))]  # translated at access time, not import
```

```python
# For field translate parameter:
name = fields.Char(translate=True)                    # entire value translated
body = fields.Html(translate=html_translate)           # extract terms from HTML
arch = fields.Text(translate=xml_translate)            # extract terms from XML
```

---

## File I/O

`from odoo.tools.misc import file_path, file_open`

```python
file_path('module_name/static/file.txt')    # → absolute path (validates it's in addons)
file_open('module_name/static/file.txt')    # → file object (validates path)
```

Both functions validate that the path is within Odoo's addons directories (security).

---

## JSON

`from odoo.tools.json import scriptsafe`

```python
scriptsafe.dumps(data)   # JSON with XSS-safe escaping (escapes <, >, &, U+2028, U+2029)
scriptsafe.loads(text)    # standard JSON parsing
```

```python
from odoo.tools.json import json_default
# Custom encoder for Odoo types: datetime → ISO string, lazy → evaluated, frozendict → dict
import json
json.dumps(data, default=json_default)
```

---

## Profiling

`from odoo.tools.profiler import Profiler`

```python
with Profiler(db=env.cr.dbname, description='my operation'):
    # code to profile
    ...
# Results visible in Settings > Technical > Profiling
```

---

## Data Population

`from odoo.tools.populate import populate_models, populate_model`

Used by the `odoo populate` CLI command to generate test data.

> **CHANGED in v18 — the populate engine was rewritten to be SQL-based.**
> `odoo/tools/populate.py` now exposes `populate_models(model_factors, separator_code)`,
> `populate_model(...)`, `populate_field(...)`, `get_field_variation(...)` and related
> helpers — it generates data directly with SQL series.
>
> The old factory helpers (`populate.compute`, `populate.randomize`,
> `populate.randfloat`, etc.) and the model hooks `_populate_sizes`,
> `_populate_dependencies`, `_populate_factories` are **NOT present in v18 core**
> (`odoo/models.py` has no `_populate*` attributes). Do not rely on them for v18.
> If you need the legacy factory pattern, verify it against the actual target
> version before use.
