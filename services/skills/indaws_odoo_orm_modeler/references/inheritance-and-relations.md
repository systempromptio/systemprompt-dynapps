# Herencia y relaciones

Base **v18**. Clases de modelo: `odoo/models.py` (v19: `odoo/orm/models.py` +
`models_transient.py`). Ver `version-diffs.md`.

> Esto es herencia de **MODELOS**. La herencia de **VISTAS** (`inherit_id` + `xpath`/`position`) es otro
> sistema distinto → skill `odoo-view-designer`. Quien busca "herencia" a menudo quiere una de las dos.

## Tipos de herencia

| Mecanismo | Atributos | Crea tabla/modelo nuevo | Cuando usarlo |
|-----------|-----------|--------------------------|---------------|
| Extension (in-place) | `_inherit = 'res.partner'` (sin `_name` o `_name` == `_inherit`) | No (misma tabla) | Anadir campos/metodos a un modelo existente |
| Prototipo (copia) | `_inherit = 'a.model'` + `_name = 'b.model'` distinto | Si (tabla nueva, copia estructura) | Clonar comportamiento en un modelo separado (raro) |
| Delegation | `_inherits = {'product.template': 'product_tmpl_id'}` | Si (tabla propia + FK) | Composicion: el registro "es-un" delegado via FK (p.ej. product.product -> product.template) |
| Modelo nuevo | `_name = 'my.model'` | Si | Entidad nueva |
| AbstractModel | `class X(models.AbstractModel)` | No (mixin reutilizable) | Mixins (`mail.thread`, helpers) |
| TransientModel | `class X(models.TransientModel)` | Si (con vacuum periodico) | Wizards (datos efimeros) |
| Modelo vista SQL | `_auto = False` + `init()` (CREATE VIEW) o `_table_query` | Si (VIEW, no tabla) | Informes/analitica de SOLO LECTURA (`sale.report`, `account.invoice.report`) |

Clases base v18: `Model(AbstractModel)` `odoo/models.py:7458`, `Model`; `TransientModel(Model)` `odoo/models.py:7475`, `TransientModel`.
v19: `Model(AbstractModel)` `odoo/orm/models.py:7049`, `Model`; `TransientModel(Model)` `odoo/orm/models_transient.py:10`, `TransientModel`.

### `_inherit` (extension) — el caso comun
```python
class ResPartner(models.Model):
    _inherit = 'res.partner'
    loyalty_points = fields.Integer()
```
Mismo `_table`, mismos registros. Puedes anadir campos, override de metodos (con `super()`).

### `_inherits` (delegation) — NO confundir
```python
class ProductProduct(models.Model):
    _name = 'product.product'
    _inherits = {'product.template': 'product_tmpl_id'}
    product_tmpl_id = fields.Many2one('product.template', required=True, ondelete='cascade')
```
Los campos del modelo delegado son accesibles transparentemente; al crear un `product.product` se
crea/linka un `product.template`. Es composicion, no extension.

Medido el 2026-08-15 (`tools/bench-orm`) con un `_inherits` mínimo:

- Crear el hijo **crea el padre**: 0 → 1 registros en el modelo delegado, y `parent_id` queda
  puesto.
- El campo del padre **se lee** desde el hijo como si fuera suyo.
- Y **escribirlo desde el hijo propaga**: el registro padre queda con el valor nuevo.

Ampliado el 2026-08-16, mismo banco:

- **Buscar y ordenar por un campo delegado funcionan** desde el hijo (`search([('titulo','=',…)])`,
  `order="titulo"`): el ORM hace el JOIN. No hace falta duplicar el campo para poder filtrar.
- **Borrar el hijo NO borra el padre**: el padre queda **vivo y huérfano**. El `cascade` va en el
  sentido contrario (es el m2o del hijo el que lo declara), así que borrar el padre **sí** se lleva
  al hijo. Consecuencia práctica: una limpieza que borre hijos deja basura acumulándose en el modelo
  delegado, y nadie la ve porque las pantallas se abren por el hijo.

**No confundir con `related`, que se le parece y hace lo contrario.** Un `related` sin
`readonly=False` acepta la escritura y **no propaga nada**
(`references/fields-and-attributes.md`). Delegación: escribes y llega. Related por defecto:
escribes y se pierde.

> **Gotcha delegation:** escribir un campo delegado escribe en el REGISTRO PADRE COMPARTIDO, no en el
> hijo. Si varios hijos comparten el mismo padre (varias `product.product` de una misma
> `product.template`), cambiar un campo delegado desde un hijo lo cambia para TODOS. El campo delegado
> no es "propio" del hijo. Para valor por-hijo, define un campo real en el hijo, no delegues.

### Multiple `_inherit`
```python
class X(models.Model):
    _name = 'my.model'
    _inherit = ['mail.thread', 'mail.activity.mixin']
```
Incorpora mixins (AbstractModel) a un modelo nuevo o existente.

### Modelos respaldados por vista SQL (`_auto = False`)
No crean tabla: mapean sobre una VISTA SQL. Son los modelos de **informe/analitica de solo lectura**
(`sale.report` — `_auto = False`, `sale/report/sale_report.py:11`, `SaleReport._auto`; `account.invoice.report`).
```python
class SaleReport(models.Model):
    _name = 'sale.report'
    _auto = False            # no crea tabla → sin create/write, _log_access off
    _rec_name = 'date'
    # Opcion A (declarativa): _table_query = "SELECT ..."   # v18 models.py:608 · v19 orm/models.py:427
    # Opcion B (clasica):     override init() con tools.drop_view_if_exists(cr, table) + CREATE VIEW
    amount_total = fields.Float(readonly=True)
```
Campos `readonly=True`, sin CRUD. Al portar a v19 el atributo `_table_query` vive en `odoo/orm/models.py:427`, `BaseModel._table_query`.

## Cuando usar cada uno (arbol de decision)

1. ¿Modificas un modelo que **ya existe** (anadir campos/metodos/constraints)? → **`_inherit`** (extension).
   Ej: anadir un campo a `res.partner`, override de `create()` en `sale.order`.
2. ¿Las MISMAS capacidades se repiten en **varios modelos** sin relacion entre si? → **mixin AbstractModel**
   (`_inherit = ['mail.thread', ...]`). Ej: chatter/actividades (`mail.thread`, `mail.activity.mixin`),
   portal (`portal.mixin`), valoraciones (`rating.mixin`). NO copies estructura con prototipo: usa mixin.
3. ¿Entidad **nueva** que ES una especializacion 1:1 de otra, comparte su registro y expone sus campos
   transparentemente? → **`_inherits`** (delegation). Ej (verificados): `res.users` _inherits `res.partner`
   (`odoo/addons/base/models/res_users.py:333`, `Users._inherits`), `product.product` _inherits `product.template` (`product/models/product_product.py:19`, `ProductProduct._inherits`).
4. ¿Tu modelo solo **referencia** a otro (N:1, opcional, con datos propios)? → **`Many2one` normal**, NO
   `_inherits`. Delegar exige 1:1 + FK required + escritura al padre compartido; si no lo quieres, es un M2o.
5. ¿Informe/analitica de solo lectura sobre una consulta SQL? → **`_auto = False`** + `_table_query`/`init()`.
6. ¿Copia estructural pero datos totalmente independientes? → prototipo (`_inherit`+`_name`≠). **Raro**:
   casi siempre lo que querias era un mixin (caso 2) o delegation (caso 3).

**Discriminador rapido:** `_inherit` = *extiendo/reutilizo comportamiento*. `_inherits` = *compongo 1:1 y
delego*. `Many2one` = *solo referencio*. Mixin = *reutilizo en muchos sin tabla ni FK*.

**Anti-patrones:** usar `_inherits` para una relacion N:1 (es Many2one); usar prototipo para "quiero los
mismos campos en varios modelos" (es mixin); delegar cuando no quieres la semantica de escritura-al-padre.

## Clases base: cuando usar cada una

**`models.Model`** (persistente) — el default: tabla real, registros que perduran. Casi todo.

**`models.AbstractModel`** — SIN tabla, no instanciable (no tiene registros). Base reutilizable. Usar para:
- **Mixins** que se añaden a muchos modelos vía `_inherit=[...]`: `mail.thread`, `mail.activity.mixin`,
  `portal.mixin`, `rating.mixin`, `analytic.mixin`, `utm.mixin`. Sus campos materializan en la tabla del
  modelo concreto que los hereda.
- **Lógica/interfaz compartida** sin datos propios (helpers comunes).
- **Modelos de informe QWeb**: el *parser* de un informe es un AbstractModel `report.<módulo>.<template>`
  que override `_get_report_values` (ej. `report.account.report_invoice`, `account/report/account_invoice_report.py:185-200`, `ReportInvoiceWithoutPayment._get_report_values`).
  Distinto del modelo de vista SQL (`_auto=False`): este da el CONTEXTO al QWeb, no una tabla/vista.
- Gotcha: no puedes `create()` sobre un AbstractModel; se vuelve concreto cuando un `Model` lo `_inherit`a.
  Para "los mismos campos en varios modelos" → AbstractModel, NUNCA prototipo.

**`models.TransientModel`** — tiene tabla pero los registros se **purgan** periódicamente (vacuum). Efímero:
- **Wizards** (asistentes de UI) y intermediarios temporales (importadores, cálculos de paso).
- Vacuum por `_transient_max_count`/`_transient_max_hours` (v18 `odoo/models.py:653`, `BaseModel._transient_max_count`, y `odoo/models.py:658`, `BaseModel._transient_max_hours`; v19
  `odoo/orm/models_transient.py:24-26`, `class TransientModel`, desde config `osv_memory_count_limit`/`transient_age_limit`).
- Gotchas: NO guardes nada que debas conservar; NO pongas un FK persistente→transient (queda colgando al
  purgar); acceso relajado (cada usuario ve/crea los suyos).

Contraste de "reutilización": **AbstractModel** = reutilizar sin tabla/FK/datos compartidos · **`_inherits`**
= comparte el registro del padre 1:1 · **`Many2one`** = solo referencia.

## Atributos de modelo utiles

| Atributo | Que hace | Fuente v18 |
|----------|----------|------------|
| `_order` | Orden por defecto (`'name'`, `'date desc, id'`) | `odoo/models.py:613`, `class BaseModel` |
| `_rec_name` | Campo usado como etiqueta (default `name`) | `odoo/models.py:611`, `class BaseModel` |
| `_rec_names_search` | Campos considerados en `name_search` | `odoo/models.py:612`, `class BaseModel` |
| `_log_access` | Crea create/write uid/date (default = `_auto`) | `odoo/models.py:821`, `BaseModel._build_model_attributes` |
| `_auto` | Crea tabla automaticamente | `models.py` |
| `_check_company_auto` | Auto-check de company en campos `check_company=True` | `models.py` |

`_rec_names_search` existe ya en v17 (`odoo17/odoo/models.py:596`, `BaseModel._rec_names_search`); ordenar por un campo requiere que sea
stored.

## Defaults: `default=` vs `_default_x`

```python
# callable inline
active = fields.Boolean(default=True)
date = fields.Date(default=lambda self: fields.Date.today())
# metodo dedicado (logica mas larga / reutilizable)
def _default_currency(self):
    return self.env.company.currency_id
currency_id = fields.Many2one('res.currency', default=_default_currency)
```
`default` puede ser valor literal o callable que recibe `self`. No existe convencion magica
`_default_<field>` automatica: hay que pasarlo a `default=`.

## Relaciones x2many: Command (preferido) vs tuplas legacy

`Command` es `IntEnum` con helpers. v17/v18: `fields.Command` (`odoo/fields.py:4280`, `Command`,
exportado en `odoo/__init__.py:74`, `Command`). v19: `odoo/orm/commands.py:11`, `Command`.
Import: `from odoo import Command`.

| Helper | Tupla legacy | Efecto |
|--------|--------------|--------|
| `Command.create(vals)` | `(0, 0, vals)` | Crea y linka un registro nuevo |
| `Command.update(id, vals)` | `(1, id, vals)` | Actualiza un registro linkado |
| `Command.delete(id)` | `(2, id, 0)` | Borra el registro de la BD |
| `Command.unlink(id)` | `(3, id, 0)` | Quita el link (no borra) |
| `Command.link(id)` | `(4, id, 0)` | Anade un link a existente |
| `Command.clear()` | `(5, 0, 0)` | Quita todos los links |
| `Command.set(ids)` | `(6, 0, ids)` | Reemplaza por exactamente estos ids |

Valores numericos confirmados: v18 `odoo/fields.py:4303-4309`, `Command.CREATE`, v19 `odoo/orm/commands.py:35-41`, `Command.CREATE`.

```python
from odoo import Command
order.write({'line_ids': [
    Command.create({'product_id': p.id, 'qty': 2}),
    Command.unlink(old_line.id),
    Command.set([a.id, b.id]),
]})
```
Usa SIEMPRE `Command.*` en codigo nuevo (legible y estable). Las tuplas siguen funcionando pero son legacy.
