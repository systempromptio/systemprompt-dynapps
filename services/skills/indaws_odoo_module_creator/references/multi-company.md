# Multi-Company Module Patterns — Odoo 18

**Todos los módulos Dynapps DEBEN ser multi-company compatibles.**
Que un cliente use multi-company o no es independiente del módulo.

## Table of Contents

| § | Sección | Cuándo leer |
|---|---|---|
| 1 | Arquitectura Multi-Company en el ORM | Entender cómo funciona internamente |
| 2 | Campo `company_id` — Cuándo y Cómo | Siempre — clasificar modelos padre/hijo/config/compartido |
| 3 | Record Rules (`ir.rule`) | Siempre — crear reglas de seguridad por empresa |
| 4 | `_check_company_auto` + `check_company` | Siempre — validación ORM en campos Many2one |
| 5 | `company_dependent` | Solo si hay campos con valor por empresa (costes, cuentas) |
| 6 | `with_company()` | Solo si hay lógica que cruza empresas |
| 7 | Jerarquía de Empresas (Branches) | Solo si se usan dominios `parent_of` (contabilidad) |
| 8 | Procurement y Empresa | Solo si el módulo interactúa con stock rules / procurement |
| 9 | Checklist | Siempre — verificación final |
| 10 | Gotchas | Siempre — errores comunes |
| 11 | Tabla de Referencia | Consulta rápida de modelos core |

---

## 1. Arquitectura Multi-Company en el ORM

### Cómo funciona internamente

Odoo resuelve multi-company en 3 capas:

| Capa | Mecanismo | Dónde |
|---|---|---|
| **SQL (row-level)** | `ir.rule` con `company_ids` | `base/models/ir_rule.py:35-50`, `IrRule._eval_context` |
| **ORM (field-level)** | `_check_company_auto` + `check_company=True` | `odoo/models.py:4340-4427`, `BaseModel._check_company` |
| **Contexto** | `with_company()` → `allowed_company_ids` | `odoo/models.py:6333-6365`, `BaseModel.with_company` |

### Variables mágicas en ir.rule

```python
# base/models/ir_rule.py:45-50
def _eval_context(self):
    return {
        'user': self.env.user.with_context({}),
        'time': time,                            # módulo time de Python
        'company_ids': self.env.companies.ids,  # empresas activas del usuario
        'company_id': self.env.company.id,       # empresa actual
    }
```

- `company_ids` = empresas que el usuario tiene activadas (selector superior derecha)
- `company_id` = la primera de esa lista (empresa activa)

### with_company() — qué hace realmente

```python
# odoo/models.py:6333-6365
def with_company(self, company):
    company_id = int(company)
    allowed_company_ids = list(self.env.context.get('allowed_company_ids') or [])
    if company_id in allowed_company_ids:
        allowed_company_ids.remove(company_id)
    allowed_company_ids.insert(0, company_id)
    return self.with_context(allowed_company_ids=allowed_company_ids)
```

Solo manipula `allowed_company_ids` en el contexto. `env.company` y `env.companies` se derivan de esa lista.

---

## 2. Campo `company_id` — Cuándo y Cómo

### Modelos PADRE (sale.order, purchase.order, stock.move, account.move)

El documento raíz es el dueño de la empresa:

```python
company_id = fields.Many2one(
    "res.company",
    required=True,
    index=True,
    default=lambda self: self.env.company,
)
```

**Referencia real:**
- `sale.order` → `sale/models/sale_order.py:76-79`, `SaleOrder.company_id`
- `purchase.order` → `purchase/models/purchase_order.py:146`, `PurchaseOrder.company_id`
- `stock.move` → `stock/models/stock_move.py:41-44`, `StockMove.company_id`
- `mrp.production` → `mrp/models/mrp_production.py:207-209`, `MrpProduction.company_id`

**Caso especial** — `account.move` usa compute/inverse desde journal:
```python
# account/models/account_move.py:196-201
company_id = fields.Many2one(
    compute='_compute_company_id', inverse='_inverse_company_id',
    store=True, readonly=False, precompute=True, index=True,
)
```
Esto es excepcional. No imitarlo salvo necesidad real.

**Caso especial** — `stock.picking` hereda del tipo de operación:
```python
# stock/models/stock_picking.py:633-635
company_id = fields.Many2one(
    related='picking_type_id.company_id', store=True, index=True,
)
```

### Modelos HIJO (sale.order.line, account.move.line, etc.)

Los hijos **heredan** del padre via `related`:

```python
company_id = fields.Many2one(
    related="order_id.company_id",
    store=True,
    index=True,
    precompute=True,  # opcional pero recomendado
)
```

**No poner `required`** — lo hereda del padre.
**No poner `default`** — el valor viene del related.

**Referencia real:**
- `sale.order.line` → `sale/models/sale_order_line.py:45-47`, `SaleOrderLine.company_id`
- `purchase.order.line` → `purchase/models/purchase_order_line.py:49-50`, `PurchaseOrderLine.company_id`
- `account.move.line` → `account/models/account_move_line.py:56-59`, `AccountMoveLine.company_id`

### Modelos de CONFIGURACIÓN

Igual que padre: `required=True, default=lambda self: self.env.company`.
Algunos permiten `company_id = False` para compartir entre empresas.

**Referencia real:**
- `stock.location` → optional, shared (`stock/models/stock_location.py:67-70`, `Location.company_id`)
- `stock.route` → optional, shared (`stock/models/stock_location.py:517-520`, `StockRoute.company_id`)
- `mrp.bom` → optional, shared (`mrp/models/mrp_bom.py:64-66`, `MrpBom.company_id`)
- `stock.warehouse` → required (`stock/models/stock_warehouse.py:39-42`, `Warehouse.company_id`)

### Modelos SIN company_id

| Modelo | Razón | Cómo se maneja la empresa |
|---|---|---|
| `procurement.group` | Cross-company por diseño | Empresa en namedtuple `Procurement` (7º campo) |
| `res.partner` | Compartido | `company_dependent` en campos property |
| `product.product` | Compartido | `company_dependent=True` para `standard_price` etc. |
| `uom.uom` | Global | — |

---

## 3. Record Rules (`ir.rule`)

### Dominios estándar por tipo de modelo

| Tipo | Dominio | Cuándo |
|---|---|---|
| **Estricto** | `[('company_id', 'in', company_ids)]` | Documentos transaccionales (SO, PO, picking, move, invoice) |
| **Compartible** | `[('company_id', 'in', company_ids + [False])]` | Configuración compartible (location, route, BOM, lot) |
| **Jerárquico** | `[('company_id', 'parent_of', company_ids)]` | Contabilidad con branches (account, tax, journal) |
| **Jerárquico + compartible** | `['|', ('company_id', '=', False), ('company_id', 'parent_of', company_ids)]` | Tax repartition, payment terms |

### Template XML

```xml
<?xml version="1.0" encoding="utf-8"?>
<odoo noupdate="1">

    <!-- Documentos transaccionales: estricto -->
    <record id="my_model_comp_rule" model="ir.rule">
        <field name="name">My Model: multi-company</field>
        <field name="model_id" ref="model_my_model"/>
        <field name="domain_force">[('company_id', 'in', company_ids)]</field>
    </record>

    <!-- Configuración compartible: permite company_id = False -->
    <record id="my_config_comp_rule" model="ir.rule">
        <field name="name">My Config: multi-company</field>
        <field name="model_id" ref="model_my_config"/>
        <field name="domain_force">[('company_id', 'in', company_ids + [False])]</field>
    </record>

</odoo>
```

### Reglas GLOBALES vs GRUPO — Diferencia crítica

| Tipo | Quién afecta | Combinación |
|---|---|---|
| **Global** (sin `groups`) | TODOS los usuarios | AND entre ellas |
| **Grupo** (con `groups`) | Solo usuarios del grupo | OR entre las del mismo modelo |

**Evaluación final:** `(AND de globales) AND (OR de grupo)`

### Anti-pattern: regla de grupo sin complementaria

```xml
<!-- ❌ PELIGROSO -->
<record id="my_restrictive_rule" model="ir.rule">
    <field name="model_id" ref="stock.model_stock_move"/>
    <field name="domain_force">[('picking_id.carrier_id.user_id', '=', user.id)]</field>
    <field name="groups" eval="[(4, ref('my_portal_group'))]"/>
</record>
```

Si un usuario interno está en `my_portal_group`, pierde acceso a todos los moves que no sean de su carrier.

**Fix:** regla complementaria para internos:

```xml
<!-- ✓ CORRECTO -->
<record id="my_internal_rule" model="ir.rule">
    <field name="model_id" ref="stock.model_stock_move"/>
    <field name="domain_force">[(1, '=', 1)]</field>
    <field name="groups" eval="[(4, ref('base.group_user'))]"/>
</record>
```

**Patrón Odoo real:** `sale/security/ir_rules.xml` usa `sale_order_personal_rule` (restrictiva) + `sale_order_see_all` (permisiva para managers).

---

## 4. `_check_company_auto` + `check_company`

### Las dos piezas son UNA sola — medido

`_check_company_auto` vale **`False` por defecto** (v18 `odoo/models.py:630`, `BaseModel._check_company_auto`, v19
`odoo/orm/models.py:451`, `BaseModel._check_company_auto`) y es la **única** condición de las dos llamadas a `_check_company()`. Así
que **`check_company=True` en el campo, solo, no lo mira nadie**: se acepta, no valida, y no avisa.

Medido en v18 y v19 con dos modelos idénticos salvo esa línea
(`tools/bench-orm/.../tests/test_company.py`, 9/9 en las dos):

| Modelo | `check_company=True` en el m2o | `_check_company_auto` | Crear en A apuntando a un recurso de B |
|---|---|---|---|
| `dyn.mc.lax` | sí | **no** | **pasa en silencio** |
| `dyn.mc.strict` | sí | sí | `UserError` en `create` **y** en `write` |
| `dyn.mc.strict` (campo sin `check_company`) | no | sí | pasa — el campo no entra en la comprobación |
| `dyn.mc.strict` → recurso con `company_id = False` | sí | sí | pasa (`_check_company_domain` admite el compartido) |

Por eso los pasos 3 y 4 del checklist **no son independientes**: poner uno sin el otro deja el
módulo exactamente igual de desprotegido que no poner ninguno, pero con aspecto de protegido.

### En el modelo padre

```python
class MyModel(models.Model):
    _name = "my.model"
    _check_company_auto = True
```

El ORM llama `_check_company()` automáticamente en `create()` y `write()`. Verifica que todo campo con `check_company=True` apunte a registros de empresa compatible.

**Disparado en:**
- `create()` → `odoo/models.py:5076-5077`, `BaseModel.create`
- `write()` → `odoo/models.py:4860-4861`, `BaseModel.write`, y `check_company` solo se pone a `True` si en los vals hay
  `company_id` o un campo con `check_company` (`:4773-4774`)

### La guardia solo mira cuando tocas el enlace — medido

Corolario de esa condición, y no está en ninguna documentación: la incoherencia **creada desde el
otro lado no la detecta nadie**. Medido: se crea un registro coherente (A → recurso de A), se mueve
el **recurso** a la empresa B, y a partir de ahí `read()` y cualquier `write` sobre otro campo pasan
sin protestar. El enlace entre empresas queda ahí hasta que alguien reescriba **ese** m2o.

O sea que `check_company` es una validación **de escritura del enlace**, no una invariante del dato.
Si el módulo mueve recursos de empresa (o lo hace un usuario desde la interfaz), hace falta una
comprobación propia: el ORM no vuelve a mirar.

### En campos Many2one

```python
warehouse_id = fields.Many2one("stock.warehouse", check_company=True)
journal_id = fields.Many2one("account.journal", check_company=True)
```

Efecto dual:
1. **Python (create/write):** `_check_company()` valida compatibilidad
2. **JS (vista):** genera dominio automático `['|', ('company_id', '=', False), ('company_id', '=', company_id)]`

### Modelos que lo usan en Odoo 18

| Modelo | `_check_company_auto` | Nº campos `check_company` |
|---|---|---|
| `sale.order` | Sí | 8+ |
| `sale.order.line` | Sí | 2 |
| `purchase.order` | No | 3 |
| `stock.picking.type` | Sí | 5 |
| `stock.move` | No | 11+ |
| `mrp.bom` | Sí | 3 |
| `account.move` | Sí | 10+ |
| `account.move.line` | Sí | 6+ |

---

## 5. `company_dependent` — Campos por empresa

### Qué es

Campos cuyo valor depende de la empresa activa. Almacenados como **JSONB**:
```json
{"1": "valor_empresa_1", "2": "valor_empresa_2"}
```

### Cómo se declara

```python
standard_price = fields.Float(company_dependent=True)
property_account_payable_id = fields.Many2one('account.account', company_dependent=True)
```

### Cómo se lee

El ORM extrae el valor para `env.company.id` vía SQL. Para otra empresa:
```python
cost = product.with_company(other_company).standard_price
```

### Principales campos company_dependent

| Campo | Modelo |
|---|---|
| `standard_price` | product.product |
| `property_account_payable_id` | res.partner |
| `property_account_receivable_id` | res.partner |
| `property_payment_term_id` | res.partner |
| `credit_limit` | res.partner |

---

## 6. `with_company()` — Cuándo usar

| Situación | Ejemplo real |
|---|---|
| Crear registros en otra empresa | `env['purchase.order'].with_company(company_id).create(vals)` |
| Leer campos `company_dependent` | `product.with_company(company).standard_price` |
| Computes que dependen de config empresa | `order.with_company(order.company_id)` |
| Generar secuencias | `env['ir.sequence'].with_company(company).next_by_code(...)` |
| Crear stock moves por empresa | `env['stock.move'].sudo().with_company(company_id).create(vals)` |

---

## 7. Jerarquía de Empresas (Branches)

### Campos en `res.company`

```python
parent_id = fields.Many2one('res.company', index=True, ondelete='restrict')
child_ids = fields.One2many('res.company', 'parent_id')
parent_path = fields.Char(index=True)  # materialized path: "1/3/5/"
parent_ids = fields.Many2many(compute='_compute_parent_ids')
root_id = fields.Many2one(compute='_compute_parent_ids')
```

### Dominio `parent_of` en ir.rule

Para modelos contables con herencia de empresa:
```xml
<field name="domain_force">[('company_id', 'parent_of', company_ids)]</field>
```

Permite ver registros de empresa padre si tienes acceso a una branch.

---

## 8. Procurement y Empresa

### Namedtuple Procurement

```python
# stock/models/stock_rule.py:452-453
Procurement = namedtuple('Procurement', [
    'product_id', 'product_qty', 'product_uom', 'location_id',
    'name', 'origin', 'company_id', 'values'
])
```

`company_id` es el 7º campo. `procurement.group` NO tiene campo `company_id`.

### Resolución de empresa en stock rules

```python
# Jerarquía en _get_stock_move_values (stock_rule.py:365):
'company_id': rule.company_id          # 1. empresa de la regla
    or location_src.company_id         # 2. empresa ubicación origen
    or location_dest.company_id        # 3. empresa ubicación destino
    or procurement.company_id          # 4. empresa del procurement
```

---

## 9. Checklist

| # | Paso |
|---|---|
| 1 | Clasificar modelos: padre / hijo / config / compartido |
| 2 | `company_id` — padre: `default=env.company`, hijo: `related`, config: según si compartible |
| 3 | `_check_company_auto = True` en modelos padre |
| 4 | `check_company=True` en Many2one a recursos de empresa |
| 5 | `ir_rules.xml` — regla por modelo con company_id (estricto o compartible) |
| 6 | Si usas reglas de **grupo** → añadir complementaria para `base.group_user` |
| 7 | `with_company()` en lógica que cruza empresas o lee `company_dependent` |
| 8 | Añadir `ir_rules.xml` al manifest |

---

## 10. Gotchas

1. **Regla de grupo sin complementaria** → usuario interno pierde acceso si también está en grupo restrictivo
2. **`check_company=True` sin `_check_company_auto = True`** → **no valida nada, y no avisa**
   (medido v18+v19). Las dos piezas o ninguna: §4.
3. **La guardia solo corre al escribir el enlace** → mover el recurso a otra empresa deja un enlace
   entre empresas que ningún `read`/`write` posterior denuncia (medido, §4)
4. **Modelo con `company_id` y sin `ir.rule`** → **no separa nada**: medido, un usuario cuya única
   empresa es A lee los registros de B con un `search` normal. El campo es dato, la barrera es la regla
5. **Regla estricta (`in company_ids`) sobre un modelo con registros compartidos** → los de
   `company_id = False` **desaparecen para todos**; para eso está el dominio compartible (§3)
6. **`procurement.group` NO tiene `company_id`** → empresa va en namedtuple, no en el modelo
7. **`company_dependent` es JSONB** → no se puede buscar sin contexto de empresa
8. **`parent_of` vs `in`** → contabilidad usa `parent_of`, stock/ventas usa `in`
9. **`_check_company_domain` añade `False`** → no duplicar en `check_company`

---

## 11. Tabla de Referencia

### Documentos (estricto: `in company_ids`)

| Modelo | Tipo | company_id | _check_company_auto |
|---|---|---|---|
| `sale.order` | Padre | `default, required` | Sí |
| `sale.order.line` | Hijo | `related=order_id` | Sí |
| `purchase.order` | Padre | `default, required` | No |
| `purchase.order.line` | Hijo | `related=order_id` | No |
| `stock.picking` | Hijo | `related=picking_type_id` | No |
| `stock.move` | Padre | `default, required` | No |
| `account.move` | Padre | `compute, required` | Sí |
| `account.move.line` | Hijo | `related=move_id` | Sí |
| `mrp.production` | Padre | `default, required` | No |

### Configuración (compartible: `in company_ids + [False]`)

| Modelo | company_id | required |
|---|---|---|
| `stock.location` | `default` | No |
| `stock.route` | `default` | No |
| `stock.rule` | `default` | No |
| `mrp.bom` | `default` | No |
| `mrp.workcenter` | — | No |

### Contabilidad (jerárquico: `parent_of company_ids`)

| Modelo |
|---|
| `account.account` |
| `account.journal` |
| `account.tax` |
| `account.fiscal.position` |

### Sin company_id

| Modelo | Razón |
|---|---|
| `procurement.group` | Cross-company |
| `res.partner` | Compartido (`company_dependent` para properties) |
| `product.product` | Compartido (`company_dependent` para coste) |
