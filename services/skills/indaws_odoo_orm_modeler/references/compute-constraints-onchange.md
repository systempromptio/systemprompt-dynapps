# Computes, constraints y onchange

Base **v18**. Decoradores en `odoo/api.py`; en **v19** se movieron a
`odoo/orm/decorators.py` (sigue importandose `from odoo import api`). Ver `version-diffs.md`.

## Computed fields

### Stored vs no-stored

| | `store=False` (default en compute) | `store=True` |
|---|---|---|
| Persiste en BD | No (se calcula al vuelo) | Si (columna real) |
| Buscable en domain | Solo si defines `search=` — **y sin el, v18 NO falla: se salta la condicion** (abajo) | Si |
| Ordenable (`_order`) | **No, ni con `search=`** (medido) | Si |
| Agrupable (`_read_group`) | No — `ValueError` | Si |

### «No se puede buscar» es falso en v18: se puede, y miente — MEDIDO

Filtrar por un computed **sin `store` y sin `search=`** no lanza nada en v18. El ORM **loguea un
ERROR y descarta la condicion** (`odoo/osv/expression.py:1179-1186`, `expression.parse`, literal en el codigo: *«Ignore
it: generate a dummy leaf»*), asi que el `search` devuelve **el conjunto entero**. Medido: dominio
que deberia dar 1 registro, devuelve 3.

**v19 lo cerro**: `ValueError: Cannot convert <modelo>.<campo> to SQL because it is not stored`.

Las dos direcciones importan:

- un v18 en produccion que filtre por un no-almacenado **esta dando datos de mas ahora mismo**, y lo
  unico que lo delata es una linea en el log del servidor;
- ese mismo codigo **revienta el dia de la migracion a v19**, y el sintoma («esta pantalla ya no
  abre») no se parece en nada a la causa.

Sonda: `tools/bench-orm/.../test_modeler.py::TestComputedSearchOrder`.

### `search=` habilita BUSCAR, no ORDENAR — MEDIDO

`order="mi_campo"` sobre un computed no almacenado lanza `ValueError: Cannot convert ... to SQL
because it is not stored` **tenga o no `search=`** (`odoo/models.py:2981`, `BaseModel._field_to_sql` v18). El `search=` genera un
dominio; el `ORDER BY` necesita una columna, y no la hay. Para ordenar: `store=True`, o ordena en
Python con `sorted()` sobre el recordset ya leido.
| Recompute | **Una vez por transacción**, no por lectura (ver abajo) | Al cambiar dependencias (`@api.depends`) |
| Coste | CPU por lectura | Espacio + recompute en write |

```python
amount_total = fields.Float(compute='_compute_amount', store=True)

@api.depends('line_ids.subtotal')
def _compute_amount(self):
    for rec in self:
        rec.amount_total = sum(rec.line_ids.mapped('subtotal'))
```

### `store=False` NO se recalcula en cada lectura

Medido el 2026-08-15 con un compute que lleva un contador
(`tools/bench-orm`, `TransactionCase`):

| Lo que se hace | Veces que corre el compute |
|---|---|
| Cinco lecturas seguidas del campo | **1** |
| `invalidate_recordset()` y dos lecturas más | **1** |

El valor se calcula una vez y se sirve de la **caché del entorno** hasta que algo la invalide. La
tabla de arriba decía «cada lectura» y eso lleva a una decisión equivocada: descartar un compute
no almacenado por su coste de CPU cuando dentro de una transacción se paga **una vez por
recordset**, no una por acceso.

Lo que sí cuesta de verdad en un `store=False` es que **no se puede buscar ni ordenar** sin
`search=`, y que cada transacción nueva vuelve a pagarlo. El coste por lectura, no.

### Computed-writable (`inverse`)

Permite escribir un campo computed; el `inverse` propaga el valor a las fuentes.

```python
@api.depends('first_name', 'last_name')
def _compute_full_name(self):
    for r in self: r.full_name = f"{r.first_name} {r.last_name}"

def _inverse_full_name(self):
    for r in self:
        parts = (r.full_name or '').split(' ', 1)
        r.first_name = parts[0]; r.last_name = parts[1] if len(parts) > 1 else ''

full_name = fields.Char(compute='_compute_full_name', inverse='_inverse_full_name', store=True)
```

### Buscable sin store (`search`)

```python
def _search_is_late(self, operator, value):
    # devuelve un domain
    return [('deadline', '<', fields.Date.today())]
is_late = fields.Boolean(compute='_compute_is_late', search='_search_is_late')
```

**v19 cambia lo que le LLEGA a ese metodo** (medido en las dos):

| | operator | value |
|---|---|---|
| v18 | `"="` | `"AAA"` (`str`) |
| v19 | `"in"` | `OrderedSet(["AAA"])` |

El optimizador de dominios de v19 normaliza `=` a `in` **antes** de llamarte. La implementacion
tipica —`value.lower()`, `value.strip()`, comparar con una constante— muere con
`AttributeError: 'OrderedSet' object has no attribute 'lower'`, y **no al instalar el modulo**: el
dia que alguien filtre por ese campo. Escribe el metodo tolerante a los dos:

```python
def _search_is_late(self, operator, value):
    valores = [value] if isinstance(value, str) else list(value)
    ...
```

### `@api.depends` recursivo

Si el compute depende de si mismo (jerarquias), declara `recursive=True` en el campo:
```python
complete_name = fields.Char(compute='_compute_complete_name', recursive=True, store=True)
@api.depends('name', 'parent_id.complete_name')
def _compute_complete_name(self): ...
```

### `@api.depends_context`

Para computes no-stored que dependen del contexto (compania, idioma, etc.). Fuente:
`odoo/api.py:333`, `depends_context`, v19 `odoo/orm/decorators.py:269`, `depends`.
```python
@api.depends_context('company')
def _compute_price(self): ...
```
No combinar con `store=True` (un valor por contexto no cabe en una columna).

### `precompute` (v16+)

Medido el 2026-08-15 (`tools/bench-orm`), registrando qué tipo de `id` ve el compute al correr:

| Campo | Lo que vio | Qué significa |
|---|---|---|
| `compute + store + precompute=True` | `NewId` | corre **antes del INSERT**: el valor entra en la misma sentencia |
| `compute + store` (sin precompute) | **no corrió** durante el `create()` | queda encolado y se resuelve después |

O sea que la afirmación de que `precompute` computa antes del INSERT es cierta, y trae un
corolario que se olvida: **un compute almacenado normal NO se ejecuta durante el `create()`**. Si
tu código lee ese campo inmediatamente después de crear, lo que lo dispara es la lectura o el
flush, no el create.


Computa el valor ANTES del INSERT, evitando un UPDATE posterior. Solo en `compute` + `store=True`.
```python
ref = fields.Char(compute='_compute_ref', store=True, precompute=True)
```
Fuente: `odoo/fields.py:310`, `Field.precompute`, y `odoo/fields.py:456`, `Field._get_attrs`. En no-computed emite warning y se desactiva.

### Control de recompute

- `compute_sudo=True`: recalcula como superusuario (campos visibles para todos pero dependientes
  de datos restringidos).
- Recompute se dispara por `@api.depends`: si está incompleto, el valor queda obsoleto en BD (bug silencioso). **Medido el 2026-08-15**: un compute almacenado cuyo cuerpo lee `base_b` sin declararlo guardó `2`; se cambió `base_b` a 99 —el cuerpo daría 100— y tras vaciar la caché el valor en base de datos **seguía siendo 2**. Ni error ni aviso: el campo queda mintiendo hasta que otra dependencia lo despierte.
- `self.env.add_to_compute(field, records)` / `self.flush_recordset()` para forzar (avanzado; ver odoo-version-api-reference).

## Constraints

### Python: `@api.constrains`

Se ejecuta en `create`/`write` cuando cambian los campos listados. Lanza `ValidationError`.
Fuente: `odoo/api.py:162`, `constrains`, v19 `odoo/orm/decorators.py:88`, `constrains`.

**Medido**, contando ejecuciones con `@api.constrains('a')`:

| Operación | Veces que corrió |
|---|---|
| `create({'a': 1})` | 1 |
| `write({'b': 9})` — campo **no** listado | 0 (sigue en 1) |
| `write({'a': 2})` — campo listado | 1 (sube a 2) |

O sea: corre en cada `create` aunque el campo no venga en los valores, y en `write` **solo** si
toca un campo de la lista. La consecuencia práctica: una constraint que dependa de un campo que no
declaraste **no se re-evalúa** cuando ese campo cambia — el mismo modo de fallo que un
`@api.depends` incompleto.
```python
from odoo.exceptions import ValidationError

@api.constrains('date_start', 'date_end')
def _check_dates(self):
    for r in self:
        if r.date_end and r.date_start > r.date_end:
            raise ValidationError(self.env._("End date must be after start date."))
```
Usar cuando la validacion necesita logica Python o cruza varios campos/modelos.

### SQL: a nivel de BD

Usar para unicidad/CHECK simples (mas rapido, integridad real en BD).

**v17 / v18** — lista `_sql_constraints`:
```python
_sql_constraints = [
    ('code_uniq', 'unique(code)', 'The code must be unique.'),
    ('positive_qty', 'CHECK(qty >= 0)', 'Quantity must be positive.'),
]
```
Fuente v18: `odoo/models.py:609`, `BaseModel._sql_constraints`.

**v19** — `models.Constraint` a nivel de clase (la lista `_sql_constraints` se IGNORA con warning):
```python
_code_uniq = models.Constraint('unique(code)', "The code must be unique.")
_positive_qty = models.Constraint('CHECK(qty >= 0)', "Quantity must be positive.")
```
Fuente v19: clase `Constraint` en `odoo/orm/table_objects.py:79`, `Constraint`; warning de la lista legacy en
`odoo/orm/model_classes.py:162-163`, `add_to_registry`.

> ⚠️ **«Se ignora con warning» se queda corto: la base de datos deja de tener la constraint y
> empieza a aceptar duplicados.** Medido el 2026-08-15 con el mismo modelo en las dos versiones
> (`tools/bench-orm`):
>
> | | v18 | v19 |
> |---|---|---|
> | La constraint en `pg_constraint` | `['dyn_orm_uniq_dyn_uniq_code']` | **`[]`** |
> | Crear un segundo registro con el mismo código | `UniqueViolation` | **aceptado** |
>
> No es que el mensaje de error se pierda: **la restricción no llega a crearse**. Un módulo que
> migra a v19 sin traducir su `_sql_constraints` pierde la unicidad en la propia base de datos, y
> los duplicados entran sin que nada falle. Lo único que avisa es **una línea** al cargar:
> `Model attribute '_sql_constraints' is no longer supported, please define models.Constraint`.
>
> Y como la garantía era de Postgres, la pérdida no la cazan los tests de negocio: hay que mirar
> `pg_constraint`, o intentar el duplicado a propósito.

## Onchange legacy (`@api.onchange`) vs compute

`@api.onchange` solo se dispara en el FORMULARIO (UI), NO en `create`/`write` por RPC/import.
Fuente: `odoo/api.py:259`, `onchange`, v19 `odoo/orm/decorators.py:185`, `ondelete`.

**Medido** (`tools/bench-orm`, 2026-08-15): un `create({'a': 5})` seguido de un `write({'a': 7})`
por ORM dejó el contador del onchange en **0** y el campo que rellena, **vacío**. Ni una vez. Si
tu lógica vive en un onchange, un import, un cron o una llamada RPC la saltan entera y el dato
sale mal sin que nada falle.

| Usa onchange cuando... | Usa compute cuando... |
|---|---|
| Solo necesitas asistir al usuario en el form (rellenar sugerencias, avisos `return {'warning': ...}`) | El valor debe persistir SIEMPRE (tambien por RPC/import/cron) |
| El valor es editable y NO debe recalcularse si el usuario lo cambia | Hay una formula determinista a partir de dependencias |

Regla actual: prefiere **compute (con `inverse` si debe ser editable)** sobre onchange para
cualquier dato que tenga que ser correcto fuera del form. Reserva onchange para UX puntual.
