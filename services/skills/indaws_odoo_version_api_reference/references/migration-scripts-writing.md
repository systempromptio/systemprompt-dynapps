# Migration Scripts: Cómo Escribirlos Correctamente

Best practices, helpers, anti-patrones y testing para `pre-migrate.py`, `post-migrate.py` y `end-migrate.py`.

**Prerequisito**: leer `migration-scripts-decision.md` (cuándo) y `v18-base/modules.md` (estructura).

---

## 1. Signature y entorno

```python
def migrate(cr, version):
    """
    cr      — raw psycopg2 cursor (autocommit gestionado por Odoo)
    version — string con la versión ANTERIOR instalada (e.g. "18.0.1.0.0")
              None si es fresh install (nunca debería llegar aquí)
    """
    if not version:
        return
```

### Acceso al ORM (solo post-migrate y end-migrate)

```python
from odoo import api, SUPERUSER_ID

def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    # Ahora puedes usar env['model.name'].search(...)
```

> **NUNCA** construyas `env` en `pre-migrate` — el schema es viejo, los modelos ORM esperan el nuevo.

---

## 2. `odoo.tools.sql` — Helpers oficiales

Preferir estos helpers sobre SQL directo. Son idempotentes y gestionan edge cases.

### Consulta de schema

```python
from odoo.tools import sql

sql.table_exists(cr, 'my_table')                    # → bool
sql.column_exists(cr, 'my_table', 'my_column')       # → bool
sql.table_columns(cr, 'my_table')                    # → {col: (type, nullable, comment)}
sql.index_exists(cr, 'my_index_name')                # → bool
sql.existing_tables(cr, {'table_a', 'table_b'})      # → set of existing
sql.table_kind(cr, 'my_table')                       # → TableKind.Regular|View|Materialized|None
```

### Modificación de columnas

```python
sql.create_column(cr, 'my_table', 'new_col', 'varchar')
sql.rename_column(cr, 'my_table', 'old_col', 'new_col')
sql.convert_column(cr, 'my_table', 'col', 'integer')        # cambia tipo
sql.set_not_null(cr, 'my_table', 'col')                      # añade NOT NULL
sql.drop_not_null(cr, 'my_table', 'col')                     # quita NOT NULL
```

### Constraints y foreign keys

```python
sql.add_constraint(cr, 'table', 'constraint_name', 'CHECK(amount > 0)')
sql.drop_constraint(cr, 'table', 'constraint_name')
sql.add_foreign_key(cr, 'child_table', 'parent_id', 'parent_table', 'id', 'cascade')
sql.fix_foreign_key(cr, 'child', 'parent_id', 'parent', 'id', 'set null')  # cambia ondelete
```

### Índices

```python
sql.create_index(cr, 'idx_name', 'table', ['col1', 'col2'])
sql.create_unique_index(cr, 'uniq_name', 'table', ['col1', 'col2'])
sql.drop_index(cr, 'idx_name', 'table')
```

### Vistas

```python
sql.drop_view_if_exists(cr, 'my_report_view')       # para SQL report views
sql.drop_depending_views(cr, 'table', 'column')      # antes de ALTER COLUMN
```

---

## 3. Patrones correctos

### 3.1 Idempotency (OBLIGATORIO)

El script puede ejecutarse más de una vez (retry tras error). Todo debe ser idempotente.

```python
# ✅ CORRECTO — guard antes de actuar
def migrate(cr, version):
    if sql.column_exists(cr, 'sale_order', 'old_field'):
        sql.rename_column(cr, 'sale_order', 'old_field', 'new_field')

# ❌ INCORRECTO — falla en segunda ejecución
def migrate(cr, version):
    cr.execute("ALTER TABLE sale_order RENAME COLUMN old_field TO new_field")
```

### 3.2 Logging (OBLIGATORIO)

```python
import logging
_logger = logging.getLogger(__name__)

def migrate(cr, version):
    if sql.column_exists(cr, 'sale_order', 'old_field'):
        sql.rename_column(cr, 'sale_order', 'old_field', 'new_field')
        _logger.info("Renamed sale_order.old_field → new_field")
    else:
        _logger.info("sale_order.old_field not found, skipping rename")
```

### 3.3 Batch processing (datos masivos)

```python
# ✅ CORRECTO — batch con LIMIT para tablas grandes
def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    batch_size = 1000
    while True:
        records = env['my.model'].search(
            [('state', '=', 'old_value')], limit=batch_size,
        )
        if not records:
            break
        records.write({'state': 'new_value'})
        _logger.info("Migrated batch of %d records", len(records))
        # flush para liberar memoria
        env.cr.flush()
        records.invalidate_recordset()

# ✅ ALTERNATIVA — SQL puro para millones de registros
def migrate(cr, version):
    cr.execute("""
        UPDATE my_table
        SET state = 'new_value'
        WHERE state = 'old_value'
    """)
    _logger.info("Updated %d rows", cr.rowcount)
```

### 3.4 Preservar datos antes de DROP

```python
# pre-migrate: guardar datos antes de que ORM borre la columna
def migrate(cr, version):
    if sql.column_exists(cr, 'sale_order', 'old_field'):
        # Crear columna temporal que ORM no conoce
        if not sql.column_exists(cr, 'sale_order', '_old_field_bak'):
            sql.create_column(cr, 'sale_order', '_old_field_bak', 'varchar')
        cr.execute("UPDATE sale_order SET _old_field_bak = old_field")

# post-migrate: restaurar datos en nuevo campo
def migrate(cr, version):
    if sql.column_exists(cr, 'sale_order', '_old_field_bak'):
        cr.execute("UPDATE sale_order SET new_field = _old_field_bak")
        cr.execute("ALTER TABLE sale_order DROP COLUMN _old_field_bak")
```

### 3.5 Recompute de stored computed fields

```python
# post-migrate: forzar recompute
def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    records = env['sale.order.line'].search([])
    records._compute_amount()  # método compute específico
    # O para todos los campos de un modelo:
    env.add_to_compute(
        env['sale.order.line']._fields['amount_total'],
        records,
    )
```

### 3.6 Actualizar ir_model_data (XML IDs)

```python
# pre-migrate: renombrar XML ID antes de que ORM cree duplicado
def migrate(cr, version):
    cr.execute("""
        UPDATE ir_model_data
        SET name = %s
        WHERE module = %s AND name = %s
    """, ('new_xml_id', 'my_module', 'old_xml_id'))
```

### 3.7 Migrar M2O → M2M

```python
# pre-migrate: guardar la relación M2O
def migrate(cr, version):
    if sql.column_exists(cr, 'my_table', 'partner_id'):
        if not sql.column_exists(cr, 'my_table', '_partner_id_bak'):
            sql.create_column(cr, 'my_table', '_partner_id_bak', 'integer')
        cr.execute("UPDATE my_table SET _partner_id_bak = partner_id")

# post-migrate: insertar en tabla rel del M2M
def migrate(cr, version):
    if sql.column_exists(cr, 'my_table', '_partner_id_bak'):
        cr.execute("""
            INSERT INTO my_table_res_partner_rel (my_table_id, res_partner_id)
            SELECT id, _partner_id_bak FROM my_table
            WHERE _partner_id_bak IS NOT NULL
            ON CONFLICT DO NOTHING
        """)
        cr.execute("ALTER TABLE my_table DROP COLUMN _partner_id_bak")
```

---

## 4. Anti-patrones

### 4.1 ORM en pre-migrate

```python
# ❌ FATAL — ORM espera schema nuevo, tabla tiene schema viejo
def migrate(cr, version):
    env = api.Environment(cr, SUPERUSER_ID, {})
    env['my.model'].search([])  # → puede fallar con column not found
```

### 4.2 No verificar existencia

```python
# ❌ Falla si la columna ya fue renombrada (retry, otro módulo, etc.)
def migrate(cr, version):
    cr.execute("ALTER TABLE x RENAME COLUMN a TO b")
```

### 4.3 Hardcoded IDs

```python
# ❌ El ID 5 puede ser otro registro en otra BD
def migrate(cr, version):
    cr.execute("UPDATE my_table SET category_id = 5")

# ✅ Buscar por XML ID
def migrate(cr, version):
    cr.execute("""
        UPDATE my_table t
        SET category_id = (
            SELECT res_id FROM ir_model_data
            WHERE module = 'my_module' AND name = 'category_default'
        )
        WHERE t.category_id IS NULL
    """)
```

### 4.4 cr.commit()

```python
# ❌ NUNCA — Odoo gestiona la transacción. Un commit parcial deja
#    la BD en estado inconsistente si el script falla después.
def migrate(cr, version):
    cr.execute("UPDATE ...")
    cr.commit()  # ← PROHIBIDO
```

### 4.5 Sin logging

```python
# ❌ Si falla en producción, no sabes qué pasó ni por dónde iba
def migrate(cr, version):
    cr.execute("UPDATE ...")
    cr.execute("ALTER TABLE ...")
    cr.execute("DELETE FROM ...")
```

### 4.6 SQL injection

```python
# ❌ Concatenación de strings
def migrate(cr, version):
    cr.execute("UPDATE %s SET field = 1" % table_name)

# ✅ Usar sql.SQL o %s para valores, sql.Identifier para nombres
from psycopg2 import sql as psql
def migrate(cr, version):
    cr.execute(
        psql.SQL("UPDATE {} SET field = %s").format(psql.Identifier(table_name)),
        [1],
    )
```

### 4.7 Ignorar tablas heredadas

```python
# ❌ Si el modelo usa _inherits (delegation), los campos están en OTRA tabla
# sale.order._inherits = {'account.move': 'move_id'} ← ejemplo ficticio
# Los campos de account.move están en account_move, NO en sale_order
```

---

## 5. Testing de migration scripts

### 5.1 Test manual (el más común)

```bash
# 1. Backup de la BD
pg_dump -Fc mydb > backup_pre_migration.dump

# 2. Bump versión en manifest y ejecutar upgrade
./odoo-bin -d mydb -u my_module --stop-after-init

# 3. Si falla → restaurar y arreglar
pg_restore -d mydb backup_pre_migration.dump
```

### 5.2 Test automatizado (TransactionCase)

```python
@tagged('post_install', '-at_install')
class TestMigration(TransactionCase):

    def test_post_migrate_selection_values(self):
        """Verify old selection values were migrated."""
        records = self.env['my.model'].search([('state', '=', 'old_value')])
        self.assertFalse(records, "Old selection value 'old_value' should not exist")

    def test_post_migrate_data_preserved(self):
        """Verify data was preserved during field rename."""
        record = self.env.ref('my_module.demo_record')
        self.assertTrue(record.new_field, "Data should have been migrated to new_field")
```

### 5.3 Verificación SQL directa

```python
def test_column_removed(self):
    """Verify backup column was cleaned up."""
    self.env.cr.execute("""
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'my_table' AND column_name = '_old_field_bak'
    """)
    self.assertFalse(self.env.cr.fetchone(), "Backup column should be removed")
```

---

## 6. Checklist del migration script

- [ ] `if not version: return` al inicio
- [ ] Guards de existencia antes de cada operación (`column_exists`, `table_exists`)
- [ ] Logger con `_logger.info()` para cada operación significativa
- [ ] Sin `cr.commit()` — nunca
- [ ] Sin ORM en `pre-migrate`
- [ ] Sin IDs hardcodeados — buscar por XML ID o dominio
- [ ] Parámetros SQL con `%s`, nunca concatenación de strings
- [ ] Batch processing si tabla tiene >100K registros
- [ ] Columnas temporales limpiadas en `post-migrate`
- [ ] Script funciona en segunda ejecución (idempotente)
