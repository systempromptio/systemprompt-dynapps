# Helpers util: `odoo.tools.sql` y `openupgradelib`

## 1. `odoo.upgrade` NO esta vacio: es un PUNTO DE MONTAJE

> **Corregido 2026-08-07.** Esta seccion decia que `odoo/upgrade/` esta "(casi) vacio" y que
> `odoo.upgrade.util` lo provee un repo **PRIVADO** que "NO viene en CE". Las dos cosas son falsas y
> llevan a la conclusion equivocada.

`odoo/upgrade/` es un **namespace package vacio a proposito**: su unico fichero en v19 es un
`.gitkeep` cuyo contenido es, literalmente, `(python namespace)`. Su `__path__` **se rellena al
arrancar** con lo que traiga `--upgrade-path` (con `base/maintenance/migrations` como legado por
defecto):

```python
# odoo19/odoo/modules/module.py:154-171 — "hook odoo.upgrade on upgrade-path"
for up in tools.config['upgrade_path'] or [legacy_upgrade_path]:
    if up not in odoo.upgrade.__path__:
        odoo.upgrade.__path__.append(up)
```

Y **`odoo/upgrade-util` es un repo PUBLICO**: `github.com/odoo/upgrade-util`, LGPL-3.0 (verificado por
API 2026-08-07: `private: false`). Es la libreria que Odoo usa para los scripts de sus propios modulos.
No viene **incluida** en el tarball, que es otra cosa que "no existe para CE".

Instalarla: clonar el repo y pasarlo en `--upgrade-path`, con `pip`, o via `requirements.txt` en
Odoo.sh. Con eso `from odoo.upgrade import util` **funciona en Community**.

**Por que importa la diferencia.** Si crees que no puedes tenerlo, escribes el `rename_field` a mano en
SQL. Si sabes que es un `pip install`, usas la libreria que ya resuelve **las referencias indirectas**
al renombrar modelos o campos (`ir_model_data`, `ir_model_fields`, dominios guardados, vistas), que es
justo la parte cara y donde se pierden datos. `odoo.tools.sql` **no es el equivalente**: es la caja de
herramientas SQL de bajo nivel, sin nada de eso.

Si no la instalas —o no puedes—, entonces si: la fuente fiable de helpers es **`odoo.tools.sql`** (§2),
y hay que asumir a mano el trabajo de referencias indirectas.

## 2. `odoo.tools.sql` — helpers REALES (verificados)

Import: `from odoo.tools import sql` (o `from odoo.tools.sql import column_exists, ...`).

Firmas verificadas en `$ODOO_SRC/odoo18/odoo/tools/sql.py`:

| Funcion | Firma | Linea v18 | Que hace |
|---------|-------|-----------|----------|
| `table_exists` | `(cr, tablename)` | :206 | Tabla existe -> bool |
| `create_model_table` | `(cr, tablename, comment=None, columns=())` | :265 | Crea tabla base |
| `table_columns` | `(cr, tablename)` | :290 | dict de columnas |
| `column_exists` | `(cr, tablename, columnname)` | :305 | Columna existe (rowcount) |
| `create_column` | `(cr, tablename, columnname, columntype, comment=None)` | :315 | Anade columna |
| `rename_column` | `(cr, tablename, columnname1, columnname2)` | :333 | Renombra columna |
| `convert_column` | `(cr, tablename, columnname, columntype)` | :344 | Cambia tipo |
| `convert_column_translatable` | `(cr, tablename, columnname, columntype)` | :350 | Conversion a/de jsonb traducido |
| `drop_depending_views` | `(cr, table, column)` | :378 | Dropea vistas SQL que dependen |
| `set_not_null` / `drop_not_null` | `(cr, tablename, columnname)` | :407 / :421 | NOT NULL on/off |
| `constraint_definition` | `(cr, tablename, constraintname)` | :430 | Devuelve definicion |
| `add_constraint` / `drop_constraint` | `(cr, tablename, constraintname[, definition])` | :442 / :463 | Constraints |
| `add_foreign_key` | `(cr, tablename1, columnname1, tablename2, columnname2, ondelete)` | :476 | FK |
| `index_exists` / `check_index_exist` | `(cr, indexname)` | :538 / :544 | Indices |
| `create_index` | `(cr, indexname, tablename, expressions, method='btree', where='')` | :548 | Crea indice |
| `create_unique_index` | `(cr, indexname, tablename, expressions)` | :563 | Indice unico |
| `drop_index` | `(cr, indexname, tablename)` | :576 | Dropea indice |
| `drop_view_if_exists` | `(cr, viewname)` | :582 | Dropea vista SQL |
| `escape_psql` | `(to_escape)` | :590 | Escapa para LIKE |
| `make_index_name` | `(table_name, column_name)` | :729 | Nombre canonico de indice |

NO hay en `odoo.tools.sql` un `rename_table` ni un `rename_model`: para tabla usa
`ALTER TABLE ... RENAME TO ...` a mano (ver recipes.md #3).

Estos helpers usan internamente el wrapper `SQL(...)` para identificar columnas/
tablas con seguridad (`SQL.identifier(...)`), asi que son seguros frente a
inyeccion cuando les pasas nombres de tabla/columna.

## 3. `openupgradelib` (EXTERNO — solo si esta instalado)

`openupgradelib` es una libreria de la OCA, NO es Odoo core. Solo esta disponible
si el entorno la tiene instalada (`pip install openupgradelib`). Su API NO debe
asumirse de memoria; consulta la version instalada. Helpers tipicos (orientativos,
verifica contra la version real):

- `openupgrade.rename_columns(cr, {table: [(old, new)]})`
- `openupgrade.rename_fields(env, [(model, table, old, new)])`
- `openupgrade.rename_models(cr, [(old_model, new_model)])`
- `openupgrade.rename_tables(cr, [(old_table, new_table)])`
- `openupgrade.delete_records_safely_by_xml_id(env, [xmlids])`
- `openupgrade.load_data(...)`, `openupgrade.logged_query(cr, sql)`

Import habitual: `from openupgradelib import openupgrade`.

Decision: para custom sobre CE sin garantia de tener la libreria, prefiere
`odoo.tools.sql` + SQL plano. Usa `openupgradelib` solo si el proyecto ya la
declara como dependencia.
