# Operadores de dominio PROPIOS (v19)

Cómo añadir un **operador de dominio no-estándar** (p.ej. `geo_dwithin` para PostGIS). El mecanismo
**cambió respecto a v16/v18**: ya no basta con override-ar algo en el `Field`. Verificado contra
`odoo/orm/domains.py` y `odoo/orm/models.py` (2026-07).

## Por qué no funciona "a la vieja"

Un operador no-estándar **nunca** llega a `Field.condition_to_sql`. Hay dos asertos que lo cortan:

- **El optimizador a nivel FULL** exige que el operador ya sea estándar:
  ```python
  # odoo19/odoo/orm/domains.py::DomainCondition._optimize (~:981)
  if self.operator not in STANDARD_CONDITION_OPERATORS and level == OptimizationLevel.FULL:
      self._raise("Not standard operator left")
  ```
- **`DomainCondition._to_sql`** vuelve a aseverarlo (`domains.py:~1089`:
  `assert operator in STANDARD_CONDITION_OPERATORS`).

`STANDARD_CONDITION_OPERATORS` es un `frozenset` (`odoo/orm/domains.py:81`, `STANDARD_CONDITION_OPERATORS`) — inmutable, no lo toques.
Existe además `CONDITION_OPERATORS = set(STANDARD_CONDITION_OPERATORS)  # modifiable (for
optimizations only)` (`odoo/orm/domains.py:118`, `CONDITION_OPERATORS`): es el set que `checked()` consulta para ACEPTAR un
operador (`domains.py:~822`: `if operator not in CONDITION_OPERATORS: self._raise("Invalid
operator")`).

## Mecanismo soportado (2 pasos)

**1. Registrar el operador** con el decorador `@operator_optimization([...])`. El decorador hace
`CONDITION_OPERATORS.update(operators)` (`odoo/orm/domains.py:1114`, `def operator_optimization`) → así `checked()` acepta tu operador en
el dominio de entrada, y registra una optimización que lo **reescribe** antes del nivel FULL.

**2. Reescribir la condición a `Domain.custom(to_sql=...)`** (que produce SQL directo y está marcado
`_opt_level = FULL`, así que ya no se le exige ser estándar):

```python
from odoo.orm.domains import Domain, DomainCondition, operator_optimization
from odoo.tools import SQL

@operator_optimization(['geo_dwithin'])
def _optimize_geo_dwithin(condition, model):
    fname = condition.field_expr
    geojson, meters = condition.value            # valor = (geojson, metros)
    def to_sql(model, alias, query):
        col = model._field_to_sql(alias, fname, query)   # columna cruda del campo
        return SQL(
            "ST_DWithin(%s::geography, ST_GeomFromGeoJSON(%s)::geography, %s)",
            col, geojson, float(meters),
        )
    return Domain.custom(to_sql=to_sql)
```

- **`Domain.custom(to_sql=callable)`** (`odoo/orm/domains.py:289`, `Domain.custom`) devuelve un `DomainCustom` (`odoo/orm/domains.py:738`, `class DomainCustom`).
  La firma del callable es **`(model, alias, query) -> SQL`**.
- **La columna se obtiene con `model._field_to_sql(alias, fname, query)`** (v18 `odoo/models.py:2956`, `BaseModel._field_to_sql`;
  en v19 el metodo desaparece del modelo y la conversion vive en `Field.to_sql`,
  `odoo/orm/fields.py:1209`, `Field.to_sql`), NO
  accediendo a la columna a mano — así respeta joins/related/alias de la query.
- **`SQL(template, *args)`** parametriza (evita inyección); los `%s` son placeholders de `odoo.tools.SQL`.
- SQL real verificado por dump en dyngeogis:
  `ST_DWithin(col::geography, 'SRID=4326;...'::geometry::geography, 5000.0)`.

## Ejemplo real en el core (patrón idéntico)

`analytic/models/analytic_mixin.py:112-124`, `AnalyticMixin._condition_to_sql` usa exactamente
`Domain.custom(to_sql=lambda model, alias, query: SQL(..., model._field_to_sql(alias,
'analytic_distribution', query), ...))` dentro de un search method. Es la referencia canónica del
patrón en fuente v19.

## Alternativa sin operador nuevo: search method

Si NO necesitas un operador nuevo con nombre propio y te vale reinterpretar operadores estándar,
un campo con `search=lambda ...` (o `_search_<field>`) que devuelva `Domain.custom(...)` basta —
es lo que hace `analytic_mixin`. Registra un operador propio (`@operator_optimization`) solo cuando
la semántica **no encaja** en ningún operador estándar (como `geo_dwithin`).
