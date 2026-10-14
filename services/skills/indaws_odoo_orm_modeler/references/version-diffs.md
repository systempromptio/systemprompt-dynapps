> ⚠️ **v19 rechaza un módulo cuyo manifiesto declare otro major, y solo lo dice con un WARNING.**
> `check_version` (`odoo/modules/module.py:569-579`, `check_version`) exige que `version` empiece por
> `19.0.`; si no, `odoo/modules/module.py:465-467`, `_load_manifest`, pone `installable=False` y suelta
> *"The module X has an incompatible version, setting installable=False"*. El módulo simplemente
> **deja de existir** para el instalador. Ese chequeo **no está en v17 ni en v18** (`check_version`
> no existe ahí), así que un módulo con `"version": "18.0.1.0.0"` que funcionaba se vuelve
> invisible al levantar v19 — sin error, sin traza, y con los tests reportando *0 tests*.
> Medido el 2026-08-15 montando el banco ORM.

# Diffs del ORM: v17 -> v18 -> v19

Todos los puntos verificados contra fuente local (`fichero:linea`). Base de comparacion: **v18**.

## Reestructuracion del modulo ORM en v19 (el gran cambio)

En v17/v18 el ORM vive en ficheros sueltos bajo `odoo/`:
`odoo/fields.py`, `odoo/models.py`, `odoo/api.py`.

En **v19** el ORM se movio a un paquete `odoo/orm/`:

| v17/v18 | v19 |
|---------|-----|
| `odoo/api.py` (decoradores) | codigo en `odoo/orm/decorators.py`; **`odoo/api.py` ya no es un modulo: es un paquete `odoo/api/__init__.py`** que re-exporta desde `odoo/orm/*` |
| `odoo/fields.py` (todos los campos) | `odoo/orm/fields.py` + `fields_numeric.py`, `fields_textual.py`, `fields_selection.py`, `fields_temporal.py`, `fields_binary.py`, `fields_relational.py`, `fields_reference.py`, `fields_properties.py`, `fields_misc.py` |
| `fields.Command` (en `fields.py`) | `odoo/orm/commands.py` |
| `odoo/models.py` | `odoo/orm/models.py` + `models_transient.py`, `model_classes.py`, `table_objects.py`, `registry.py` |

Comprobado: `ls $ODOO_SRC/odoo19/odoo/orm/` lista esos ficheros.
**El API publico de import NO cambia**: sigues escribiendo `from odoo import api, fields, models, Command`.
Solo cambia donde reside el codigo (importante al citar fuente o leer internals).

### El matiz que se suele contar mal: `api.py` y `fields.py` no desaparecen, se vuelven paquetes

Decir "en v19 `odoo/api.py` no existe" es cierto **como fichero** y engañoso como conclusion. En v19
hay `odoo/api/__init__.py` y `odoo/fields/__init__.py`: **paquetes cuya unica funcion es re-exportar**
la superficie publica desde `odoo/orm/*`. El propio fichero dice el motivo:

```
# odoo19/odoo/api/__init__.py:1-3
# ruff: noqa: F401
# Exports features of the ORM to developers.
# This is a `__init__.py` file to avoid merge conflicts on `odoo/api.py`.
```

Consecuencias practicas:

- **Para saber que es publico**, el sitio a leer es `odoo/api/__init__.py` (lista explicita de
  decoradores exportados: `autovacuum, constrains, depends, depends_context, deprecated, model,
  model_create_multi, onchange, ondelete, private, readonly`) y `odoo/fields/__init__.py`. Lo que no
  esta ahi es internals, aunque exista en `odoo/orm/`.
- **Para citar fuente**, la linea real esta en `odoo/orm/*.py`, no en el paquete.
- Un `ls odoo/api.py` que falla NO prueba que `odoo.api` haya muerto. Verificado 2026-08-07 sobre
  `odoo19 @d1d161033`.

## v17 -> v18

| Tema | v17 | v18 | Fuente |
|------|-----|-----|--------|
> **Citas de `odoo/fields.py` remedidas el 2026-08-17 (CE18 `@99f5a33c64`).** Cinco estaban
> **2 lineas altas** y una 1 linea alta, y el patron dice que fue la rama y no el que midio: **todo lo
> que esta antes de `:481` cae exacto** (`:180`, `:280`, `:314`) y **todo lo posterior sobra
> exactamente 2**, con un escalon intermedio de 1. O sea **dos borrados de una linea** en 18.0
> despues de la medicion, no una cuenta inventada — si fuera aritmetica, el error seria proporcional,
> no un escalon. Las citas de `odoo/models.py` de este mismo documento (`:609`, `:611`,
> `:613`, `:821`, `:653`, `:658`, `:2981`, `:7458`, `:7475`) **siguen todas exactas**: la deriva es
> por fichero, no por version.

| Agregacion en read_group | `group_operator=` | `aggregator=` (`group_operator` deprecado, warning) | warning: `odoo/fields.py:481-482`, `Field._get_attrs`; v17 attr: `odoo/fields.py:323`, `Field.group_operator`, v17 |
| `_description_*` del aggregator | `_description_group_operator` (property) | `_description_aggregator` (metodo) | `odoo17/odoo/fields.py:869`, `Field._description_group_operator` vs `odoo/fields.py:928`, `Field._description_aggregator` |
| `_related_*` | `_related_group_operator` | `_related_aggregator` | `odoo17/odoo/fields.py:733`, `Field._related_group_operator` vs `odoo/fields.py:769`, `Field._related_aggregator` |
| Nombre para mostrar | `name_get()` presente pero deprecado (`odoo17/odoo/models.py:1677`, `BaseModel.name_get`) | `name_get` **ELIMINADO**; solo `_compute_display_name` | `_compute_display_name` existe **al menos desde v14** (`odoo14:1727`, `odoo15:1829`, `odoo16:1541`, `odoo17:1661`, `odoo18:1793`); lo que v18 quita es `def name_get` |
| Almacenamiento `company_dependent` | registros `ir.property` (`odoo17/addons/base/models/ir_property.py`) | **columna `jsonb`** en la tabla, clave = company_id; `ir.property` ELIMINADO | `odoo/fields.py:180`, `Field`, y `odoo/fields.py:280`, `Field._column_type`; `ir_property.py` ausente en v18 |

> **`name_get` → `_compute_display_name`:** `name_get` sobrevivió deprecado en v16/v17 y se
> **eliminó en v18**. Al portar a v18+, cualquier override de `name_get` debe reescribirse como
> `_compute_display_name` (asignar `record.display_name = ...`).
>
> **Ojo con la fecha de `_compute_display_name`**: el método **existe al menos desde v14**
> (`odoo14/odoo/models.py:1727`, `BaseModel._compute_display_name`, `odoo15:1829`, `odoo16:1541`) — esta nota decía "desde v16" y era
> falso, corregido 2026-08-07. Lo que cambia en v18 no es que aparezca el método, es que
> **desaparece la alternativa**: en v14-v17 `_compute_display_name` existía pero su implementación
> por defecto delegaba en `name_get()` (el docstring de v15 lo dice: *"In general `display_name` is
> equal to calling `name_get()[0][1]`"*), así que el punto de override natural era `name_get`. En v18
> `name_get` ya no está y el único punto es `_compute_display_name`.

`precompute`, `_sql_constraints` (lista), `_rec_names_search`, `Properties` y los decoradores
existen IGUAL en v17 y v18 (sin cambio de nombre/firma relevante para el modelado).

## v18 -> v19

| Tema | v18 | v19 | Fuente |
|------|-----|-----|--------|
| Ubicacion decoradores | `odoo/api.py` | `odoo/orm/decorators.py` (`odoo/api/` queda como paquete de re-export) | `odoo/api.py:308`, `depends` vs `odoo/orm/decorators.py:248`, `depends` |
| Ubicacion campos | `odoo/fields.py` (monolitico) | `odoo/orm/fields*.py` (dividido) | `ls odoo19/odoo/orm/` |
| Ubicacion Command | `fields.Command` (`odoo/fields.py:4280`, `class Command`) | `odoo/orm/commands.py:11`, `Command` | confirmado |
| SQL constraints | `_sql_constraints = [(name, sql, msg)]` | `models.Constraint(definition, message)`; la lista legacy **no crea la constraint en Postgres** (medido: `pg_constraint` vacío y duplicados aceptados) y solo suelta un warning | lista: `odoo/models.py:609`, `BaseModel._sql_constraints`; clase Constraint: `odoo/orm/table_objects.py:79`, `Constraint`; warning: `odoo/orm/model_classes.py:162-163`, `add_to_registry` |
| Properties | `odoo/fields.py:3522`, `Properties` | `odoo/orm/fields_properties.py:32`, `Properties` | confirmado |
| Filtrar por un computed **sin store ni `search=`** | **NO falla**: loguea un ERROR y **descarta la condicion** → devuelve todo | `ValueError: Cannot convert ... to SQL because it is not stored` | medido; v18 `odoo/osv/expression.py:1179-1186`, `expression.parse` |
| Lo que recibe un metodo `search=` | `("=", "AAA")` — `str` | `("in", OrderedSet(["AAA"]))` | medido (`bench-orm::TestComputedSearchOrder`) |

`aggregator`, `precompute`, `company_dependent`, `_inherit/_inherits`, `_rec_names_search`,
`@api.depends(_context)`, `@api.constrains`, `@api.onchange` se mantienen sin cambio de nombre en v19
(solo cambia su fichero de residencia).

## Resumen de "lo que rompe al portar"

1. **v19**: cualquier **cita a fuente** de `odoo/api.py` o `odoo/fields.py` monolitico debe reapuntar a
   `odoo/orm/*.py`. Los **imports** siguen valiendo: `odoo/api/` y `odoo/fields/` existen como paquetes
   de re-export (ver seccion anterior).
2. **v19**: `_sql_constraints` lista deja de aplicarse -> migrar a `models.Constraint`.
3. **v18+**: `group_operator` -> `aggregator` (warning, sigue funcionando temporalmente).
4. **v19**: todo metodo `search=` escrito antes de v19 hay que revisarlo — recibe `("in", OrderedSet)`
   donde recibia `("=", str)`. Rompe **en ejecucion, al filtrar**, no al instalar.
5. **v19**: los filtros por computed no almacenado que en v18 «funcionaban» (devolviendo de mas, en
   silencio) ahora lanzan `ValueError`. Buscarlos ANTES de migrar: son a la vez un bug latente en la
   v18 y un reventon garantizado en la v19.

Para firmas exhaustivas y matices de cada API/version, [delega] en `odoo-version-api-reference`.
