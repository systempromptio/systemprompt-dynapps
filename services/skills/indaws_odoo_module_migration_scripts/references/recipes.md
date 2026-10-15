# Recetas de migracion (idempotentes)

Reglas transversales:
- Todas las recetas asumen que el script puede re-ejecutarse (un `-u` puede
  repetirse). Guarda con `column_exists`/`table_exists`/comprobacion ORM.
- SQL parametrizado SIEMPRE: `cr.execute(query, params)`. `%s` con tupla es
  binding de psycopg2 (seguro). NUNCA f-string de valores.
- NUNCA `cr.commit()` dentro del script.
- Helpers `column_exists`, `rename_column`, etc. en
  `core-util-and-openupgradelib.md`.

## SQL vs ORM: cuando cada uno

| Usa SQL (`cr.execute`) en... | Usa ORM (env) en... |
|------------------------------|---------------------|
| `pre-migrate` (no hay registry fiable) | `post`/`end` cuando necesitas logica de negocio |
| Renames de columna/tabla, backfills masivos | Recompute de campos stored (`add_to_compute`) |
| Cuando el campo/modelo ya no existe en el codigo nuevo | Crear/buscar registros con constraints/defaults |
| Operaciones de millones de filas (rendimiento) | Cuando dependes de `_compute`/`_inverse`/onchange |

## 1. Rename de columna (campo renombrado en el codigo)

Hazlo en **pre** para que `init_models` (que corre despues) reconozca el campo ya
con el nombre nuevo y NO cree una columna vacia ni pierda datos.

```python
from odoo.tools import sql

def migrate(cr, version):
    if sql.column_exists(cr, "res_partner", "old_name") and \
       not sql.column_exists(cr, "res_partner", "new_name"):
        sql.rename_column(cr, "res_partner", "old_name", "new_name")
```

Si solo cambia el `string=` pero NO el nombre tecnico: no hace falta migracion.

## 2. Rename de campo Many2one / cambio de tipo

- Rename: igual que columna (apunta a la misma FK).
- Cambio de tipo de columna: en **pre**, `sql.convert_column(cr, table, col, "new_type")`.

## 3. Rename de modelo (tabla + ir.model + xmlids + FKs)

En **pre** (SQL, porque el modelo viejo ya no esta en el codigo):

```python
def migrate(cr, version):
    # 1) tabla
    cr.execute("ALTER TABLE IF EXISTS old_model RENAME TO new_model")
    # 2) ir_model / ir_model_fields / ir_model_data referencias
    cr.execute("UPDATE ir_model SET model=%s WHERE model=%s", ("new.model", "old.model"))
    cr.execute("UPDATE ir_model_data SET model=%s WHERE model=%s", ("new.model", "old.model"))
    cr.execute("UPDATE ir_model_fields SET model=%s WHERE model=%s", ("new.model", "old.model"))
    # 3) FKs entrantes / ir_attachment / mail_message res_model, etc. segun el caso
```

Nota: un rename de modelo completo (incluida limpieza de actions, views, etc.) es
laborioso; `openupgradelib.rename_models` (externo) lo cubre. Ver
`core-util-and-openupgradelib.md`. Antes de hacerlo, mide el alcance con
`odoo-impact-analyzer`.

## 4. Mover datos entre campos / tablas (backfill)

En **post** (las columnas nuevas ya existen tras `init_models`):

```python
def migrate(cr, version):
    cr.execute("""
        UPDATE sale_order
           SET new_total = amount_total
         WHERE new_total IS NULL
    """)
```

`WHERE new_total IS NULL` lo hace idempotente.

## 5. Recomputar un campo stored

En **post**/**end** via ORM (no recalcules a mano en SQL):

```python
from odoo import api, SUPERUSER_ID

def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    Line = env["sale.order.line"]
    lines = Line.search([("pos_order_line_ids", "!=", False)])
    if lines:
        env.add_to_compute(Line._fields["qty_delivered"], lines)
        env.flush_all()
```

(Patron real: indaws_l10n_es_pos/migrations/18.0.1.1.0/post-migrate.py.)
`add_to_compute` marca para recalculo; `flush_all` lo persiste.

## 6. Borrar datos / vistas / xmlid obsoletos

En **post** o **end** via ORM (respeta cascadas):

```python
from odoo import api, SUPERUSER_ID

def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    data = env["ir.model.data"].search([
        ("module", "=", "mi_modulo"),
        ("name", "in", ["view_obsoleta", "action_vieja"]),
    ])
    data.unlink()  # arrastra el registro destino
```

Para una vista concreta por xmlid:
```python
    view = env.ref("mi_modulo.view_obsoleta", raise_if_not_found=False)
    if view:
        view.unlink()
```

Drop de columna que ya no existe en el codigo (en pre, opcional, para limpiar):
```python
    if sql.column_exists(cr, "mi_tabla", "campo_muerto"):
        cr.execute("ALTER TABLE mi_tabla DROP COLUMN campo_muerto")
```
(Odoo no dropea columnas huerfanas automaticamente; las deja.)

## 7. Set masivo de un valor por defecto en registros existentes

Un campo nuevo `required` o con default necesita backfill en registros viejos.
En **post** (la columna ya existe):

```python
def migrate(cr, version):
    cr.execute("UPDATE account_move SET state_flag = 'pending' WHERE state_flag IS NULL")
```

SQL es preferible a ORM aqui por rendimiento en tablas grandes.

## 7b. Editar el arch XML de una vista por script (lxml)

Manipular `view.arch` (o `arch_db`) desde una migración con **lxml**:

```python
from lxml import etree

def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    for view in env['ir.ui.view'].search([('type', '=', 'form'), ...]):
        root = etree.fromstring(view.arch.encode('utf-8'))   # arch es str → encode
        # mover un nodo entre árboles: parent.replace(viejo, nuevo)
        # quitar wrappers <t>:      etree.strip_tags(root, 't')
        # quitar atributos:         etree.strip_attributes(frag, 'data-oe-id', 'data-oe-xpath')
        view.arch = etree.tostring(root, encoding='unicode')
```

Claves:
- **`ir.ui.view` valida la herencia al escribir**, no al render: un xpath que no ancla lanza excepción
  **en el `write`/`create`**. Para no abortar todo el lote por una vista mala, envuelve cada una:
  ```python
  for view in views:
      try:
          with cr.savepoint():        # revierte SOLO esta vista
              view.arch = new_arch
      except Exception:
          _logger.warning("Vista %s: xpath no ancla, se salta", view.id)
  ```
  Nunca `cr.rollback()` a pelo (tumba toda la migración).
- **Ancla por `hasclass()`/`name=`, no por posición**: `apply_inheritance_specs` soporta la función
  XPath custom `hasclass('page')` (registrada `odoo/addons/base/models/ir_ui_view.py:137`) → `//div[hasclass('page')]` sobrevive
  a cambios del core mejor que `//div[3]`.
- **Edición in-place estilo web/Studio**: `web_editor.save()` / `replace_arch_section` editan una sección
  del arch conservando el resto (patrón COW de website: clona la vista al primer edit). Los helpers de
  diff xpath de Studio (`_node_to_xpath`, `_identify_node`, `normalize`) son **EE-only** →
  `hasattr(env['ir.ui.view'], '_node_to_xpath')` antes de usarlos.

## 8. Idempotencia: checklist

- Rename: `if column_exists(old) and not column_exists(new)`.
- Backfill: `WHERE col IS NULL` / `WHERE NOT EXISTS (...)`.
- Insert: `ON CONFLICT DO NOTHING` o buscar antes con ORM.
- Unlink: `search` filtrado (si ya se borro, devuelve vacio).
- Nunca dependas de que el script corra exactamente una vez.
