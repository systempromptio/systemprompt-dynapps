# Diffs de versión: TransientModel v17 → v18 → v19

Base de esta skill: **v18**. Para firmas/parámetros generales de API entre
versiones consulta `odoo-version-api-reference`.

## v17 vs v18: prácticamente idéntico

`TransientModel`, `_transient`, `_transient_vacuum`, `_transient_max_count` y
`_transient_max_hours` viven en el MISMO path en ambas versiones:

- v17: `$ODOO_SRC/odoo17/odoo/models.py`
- v18: `$ODOO_SRC/odoo18/odoo/models.py`

Defaults idénticos, pero **la línea NO es la misma en las dos** (verificado):
`osv_memory_count_limit` → `my_default=0` (sin límite de filas) y
`transient_age_limit` → `my_default=1.0` (1 hora), en `tools/config.py` **v17 `:311`/`:315`,
v18 `:316`/`:320`** (v19: `:436`/`:440`). Las propiedades son `@lazy_classproperty` que devuelven
`config.get(...)` sin cast en las dos, aunque **la forma difiere**: v17 es una lambda de una línea
(`odoo/models.py:636`, `BaseModel._allow_sudo_commands`/`:638`), v18 son métodos decorados (`:653-660`).

No hay cambios relevantes de API de wizard entre v17 y v18: `default_get`,
`act_window` con `target="new"`, botones que devuelven dicts y `<footer>` en la
vista funcionan igual.

## v19: el ORM se movió a odoo/orm/ (cambio estructural)

En v19 el ORM monolítico `odoo/models.py` se partió en el paquete `odoo/orm/`.
`TransientModel` ya **no** está en `odoo/models.py`:

- v19: `$ODOO_SRC/odoo19/odoo/orm/models_transient.py` — `class TransientModel`
  (línea 10), `_transient = True` (línea 21), `_transient_vacuum` (línea 30).
- `Model` base: `$ODOO_SRC/odoo19/odoo/orm/models.py`.

`from odoo import models, fields, api` sigue funcionando en el código de usuario
(el namespace público se mantiene); solo cambia DÓNDE está el código fuente del
framework. Relevante al leer/depurar el core o al hacer imports internos.

### Diff de implementación en v19

Las propiedades por defecto ahora hacen cast explícito
(`odoo/orm/models_transient.py:24`, `class TransientModel` y `:26`):

```python
# v17 (models.py:636,638): lambda de una línea, sin cast
_transient_max_count = lazy_classproperty(lambda _: config.get('osv_memory_count_limit'))
_transient_max_hours = lazy_classproperty(lambda _: config.get('transient_age_limit'))

# v18 (models.py:653-660): método decorado, sin cast — MISMO valor, otra forma
@lazy_classproperty
def _transient_max_count(cls):
    return config.get('osv_memory_count_limit')

# v19 (orm/models_transient.py:24-26): castea a int/float
_transient_max_count = lazy_classproperty(lambda _: int(config.get('osv_memory_count_limit')))
_transient_max_hours = lazy_classproperty(lambda _: float(config.get('transient_age_limit')))
```

Los defaults de config (0 filas, 1.0 horas) y la mecánica del vacuum (limpieza
por edad/cantidad, ventana mínima de 5 minutos) no cambian. La API de wizard
(apertura, default_get, botones, vistas) tampoco cambia para el código de usuario.

## Resumen 1 línea por versión
- **v17**: igual que v18; `TransientModel` en `odoo/models.py`; defaults 0 / 1.0h.
- **v18 (base)**: `TransientModel` en `odoo/models.py:7475`, `TransientModel`; props sin cast.
- **v19**: ORM movido a `odoo/orm/`; `TransientModel` en
  `odoo/orm/models_transient.py`; props con cast `int()`/`float()`.
