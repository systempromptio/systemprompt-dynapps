# Autoría de un tipo de campo/columna PROPIO (v19)

Cómo crear un **`fields.Field` nuevo con su propio tipo de columna Postgres** (p.ej. un campo
geométrico PostGIS `geometry(Point,4326)`), no solo configurar atributos de campos estándar
(eso está en `fields-and-attributes.md`). Verificado contra `odoo/orm/fields.py` y
`odoo/tools/sql.py` (2026-07). Base v19; el mecanismo cambió respecto a v16/v18.

## 1. Declarar el tipo de columna

El tipo de columna es una tupla `(udt_name, ddl_type)` expuesta por la property `column_type`,
que por defecto devuelve `self._column_type` (atributo de clase).

```python
from odoo.orm.fields import Field

class Geometry(Field):
    type = 'geometry'
    _column_type = ('geometry', 'geometry(Point,4326)')  # (udt_name, ddl_type)
```

- **`ddl_type` (índice 1) se emite VERBATIM** en el `ALTER TABLE ... ADD COLUMN`. Confirmado:
  `sql.create_column(...)` hace `SQL(columntype)` sin escapar (`tools/sql.py::create_column`,
  ~`:326`). → permite **typmods** como `geometry(Point,4326)` sin DDL a mano.
- **`udt_name` (índice 0)** es lo que Postgres reporta en `information_schema`; Odoo lo compara
  para decidir si recrea/convierte la columna (`fields.py::update_db_column`, compara
  `column['udt_name'] == self.column_type[0]`, ~`:1140`).
- **NO existe `_slots`** en v19 (0 ocurrencias). Los atributos del campo son atributos de clase +
  los kwargs del `__init__`, guardados en `_args__` / `_extra_keys__` (`odoo/fields.py:261`, `Field`, y `odoo/fields.py:267`).
  Para un kwarg propio (p.ej. `srid=`), decláralo como atributo de clase con default y Odoo lo
  recogerá de `_args__`.

## 2. Índices no-btree (GIST/GIN): FUERA DE BANDA, no `index=`

**NO se puede pedir un índice GIST/GIN con `index=`.** `registry.check_indexes` ASEVERA que el
índice sea uno de los estándar:

```python
# odoo19/odoo/orm/registry.py::check_indexes (~:833)
assert index in ('btree', 'btree_not_null', 'trigram', True, False, None)
```

Un índice especial se crea **fuera de banda** override-ando **`Field.update_db`** (`odoo/fields.py:1094`, `Field.update_db`)
y emitiendo el `CREATE INDEX ... USING gist (...)` a mano, con un **nombre PROPIO distinto** del que
genera `sql.make_index_name(table, column)` (`odoo/tools/sql.py:779`, `make_index_name`) — p.ej. `{tabla}__{campo}_gist`.
Así `check_indexes` no lo reconoce como "esperado ni sobrante conocido" y lo **respeta**:

```python
# odoo19/odoo/orm/registry.py::check_indexes (~:892)
_schema.info("Keep unexpected index %s on table %s", indexname, tablename)
```

Si le pusieras el nombre de `make_index_name`, `check_indexes` lo trataría como el índice btree
esperado y lo borraría/recrearía.

## 3. Read path v19: override `_insert_cache`, NO `convert_to_cache`

**El camino de lectura v19 NO pasa por `convert_to_cache`.** `_fetch_query` mete el valor CRUDO de
la BD en caché llamando `field._insert_cache(fetched, values)`:

```python
# odoo19/odoo/orm/models.py::_fetch_query (~:3931)
for field, values in zip(column_fields, column_values, strict=True):
    field._insert_cache(fetched, values)   # values = valor crudo de la query SQL
```

- Si el valor en BD **no es directamente usable** (p.ej. PostGIS devuelve **EWKB hex**), override
  **`_insert_cache`** (`odoo/orm/fields.py:1595`, `Field._insert_cache`) para convertir ahí (hex→GeoJSON/objeto) antes del
  `setdefault` en caché. NO basta con `convert_to_cache` (no se llama en lectura).
- El SQL de la columna lo construye `model._field_to_sql(table, fname, query)`; puede seguir
  devolviendo la **columna cruda** — los operadores SQL (ver `custom-domain-operators.md`) la
  necesitan tal cual. Si quieres que la lectura traiga ya un formato cómodo (p.ej.
  `ST_AsEWKB(col)`), hazlo en el SQL de lectura, no rompas el de los operadores.
- `convert_to_column` / `convert_to_cache` **sí** se usan en el camino de **escritura**; impleméntalos
  para serializar el valor Python → literal Postgres del INSERT/UPDATE.

## 4. Campo custom COMPUTED STORE: funciona sin fricción

Un campo de tu tipo nuevo declarado `compute=... store=True` funciona con el ciclo estándar
(`convert_to_column`/`convert_to_cache`). Regla: mantén el **`compute` PURO** — sin red ni
side-effects (Odoo lo puede reejecutar en batch/recompute). Lo que necesite red/servicios externos
(geocoding, routing…) va en una **acción o wizard**, nunca en el compute.

## Checklist mínimo de un tipo de campo nuevo

1. Subclase de `Field`, `type = '...'`, `_column_type = (udt_name, ddl_type)`.
2. `convert_to_column` / `convert_to_cache` (escritura) + `_insert_cache` (lectura, si el crudo BD
   no es usable).
3. Índice especial → override `update_db` con nombre propio (no `make_index_name`).
4. Registrar el widget de UI aparte (v17+ objeto descriptor en `registry.category("fields")`,
   ver `odoo-owl-component-builder`); el tipo ORM y el widget web son cosas distintas.
5. Extensión Postgres (PostGIS…) instalada en la BD antes de crear la columna.
