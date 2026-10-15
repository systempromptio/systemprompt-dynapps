# TransientModel: mecánica, vacuum y seguridad

## Qué es

`models.TransientModel` es la superclase para registros "temporalmente
persistentes y limpiados periódicamente". Se materializa en una tabla real de
PostgreSQL (igual que un modelo normal), pero sus filas se borran solas.

Definición canónica (v18, `odoo18/odoo/models.py::TransientModel`):

```python
class TransientModel(Model):
    """ Model super-class for transient records, meant to be temporarily
    persistent, and regularly vacuum-cleaned. """
    _auto = True                # automatically create database backend
    _register = False           # not visible in ORM registry, python-inherited only
    _abstract = False
    _transient = True           # transient   (models.py:7485)
```

`Model._transient = False` (models.py:567 / 7471) → los modelos normales NO son
transient; solo `TransientModel` pone `_transient = True`.

Esqueleto mínimo de un wizard:

```python
from odoo import models, fields, api

class MyWizard(models.TransientModel):
    _name = 'my.module.do.something'
    _description = 'Do Something Wizard'

    # campos efímeros aquí
```

## Vacuum automático

El método `_transient_vacuum` (v18 `odoo/models.py:7488`, `class TransientModel`) está decorado con
`@api.autovacuum`, así que lo ejecuta el cron `base.autovacuum_job` — y **nadie
más**.

> **Medido** (`tools/bench-orm`, v18 y v19). Dos frases del docstring de Odoo no
> describen lo que hace el código, y esta página las repetía:
>
> - **«se dispara al crear registros»: no.** Crear 10 wizards deja el contador de
>   llamadas a `_transient_vacuum` en **0**. En v18/v19 no queda ninguna llamada
>   desde `create()`: el único disparador es el cron (`grep -rn "_transient_vacuum"`
>   sobre `odoo/` solo devuelve la definición y el re-trigger interno).
> - **«la limpieza real ocurre como mucho cada 5 minutos»: tampoco.** No hay
>   limitador entre ejecuciones; dos `_transient_vacuum()` seguidos limpian los
>   dos. Los 5 minutos son otra cosa: **la edad mínima de la fila**
>   (`seconds = max(seconds, 300)`, `odoo/models.py:7529`, `TransientModel._transient_clean_rows_older_than`).
>
> **Consecuencia práctica, y es la que importa:** el cron
> `base.autovacuum_job` corre **una vez al día**
> (`odoo/addons/base/data/ir_cron_data.xml:8-9`, `autovacuum_job`, `interval_number=1`,
> `interval_type=days`). Así que el «1 hora de vida» de más abajo es el límite de
> edad que el vacuum *aplicará cuando le toque*, no cuánto vive la fila: en una
> base normal un registro de wizard sobrevive **hasta que pase el cron**, o sea
> hasta ~24 h. Sigue siendo efímero, pero no cuentes con la hora.

```python
@api.autovacuum
def _transient_vacuum(self):
    if self._transient_max_hours:
        self._transient_clean_rows_older_than(self._transient_max_hours * 60 * 60)
    if self._transient_max_count:
        self._transient_clean_old_rows(self._transient_max_count)
```

Reglas de limpieza (**v18** `odoo/models.py:7522-7541`, `TransientModel._transient_clean_old_rows`, remedido 2026-08-18 — el rango citado empezaba en `:7521`, que es una línea en blanco; el bloque es `_transient_clean_old_rows` `:7522` + `_transient_clean_rows_older_than` `:7529`, llamados desde `_transient_vacuum` `:7489`. En **v19** el método vive en `orm/models_transient.py`, no en `models.py`):
- **Por edad** (`_transient_max_hours`): borra filas cuya `write_date`/`create_date`
  sea más antigua que el límite. **Nunca borra filas usadas en los últimos 5
  minutos** (`seconds = max(seconds, 300)`).
- **Por cantidad** (`_transient_max_count`): si `count(*) > max_count`, limpia las
  filas de más de 5 minutos.
- Borra en lotes de `GC_UNLINK_LIMIT`; si llega al límite re-dispara el autovacuum.

### Valores por defecto (verificado en fuente)

Las dos propiedades son `lazy_classproperty` que leen la config del servidor
(v18 `odoo/models.py:651-660`, `BaseModel`):

```python
@lazy_classproperty
def _transient_max_count(cls):   # models.py:653
    return config.get('osv_memory_count_limit')
@lazy_classproperty
def _transient_max_hours(cls):   # models.py:658
    return config.get('transient_age_limit')
```

Defaults de esos parámetros de config (v18 `odoo/tools/config.py:316`, `configmanager.__init__` y `:320`; v17 `:311`/`:315`, v19 `:436`/`:440`):

| Propiedad | Parámetro config | `my_default` | Significado |
|-----------|------------------|--------------|-------------|
| `_transient_max_count` | `osv_memory_count_limit` | **0** | 0 = sin límite de número de filas |
| `_transient_max_hours` | `transient_age_limit` | **1.0** | 1 hora de vida máxima por defecto |

**Implicación práctica**: un registro de wizard es EFÍMERO — con la salvedad de
arriba sobre *cuándo* se limpia de verdad. No guardes nada que deba durar, no
construyas FK hacia él, no devuelvas su `id` a un proceso externo que lo lea más
tarde.

`_transient_max_count = 0` significa **sin límite de filas**, no «cero filas»: la
rama por cantidad es un `if self._transient_max_count:` y con 0 no entra.

Se pueden sobrescribir por modelo (raro), y **funciona** (medido: la
`lazy_classproperty` de la clase base se deja tapar por un valor en la subclase):

```python
class MyWizard(models.TransientModel):
    _name = 'my.wizard'
    _transient_max_hours = 0.5   # 30 minutos para este wizard
```

Lo que **no** consigues bajando ese número es limpiar antes de 5 minutos: con
`_transient_max_hours = 0.0001` (0,36 s) la fila recién creada **sigue ahí**
después de un vacuum, y desaparece en cuanto se le envejece la `write_date` 10
minutos. El suelo de `max(seconds, 300)` gana siempre (medido en v18 y v19).

## Seguridad (ACL)

El docstring de `TransientModel` (v18 `odoo/models.py:7479-7482`, `class TransientModel`) dice que hay una
«gestión de permisos simplificada»: todos los usuarios pueden crear registros y
solo ven los que ellos crearon. **Ese docstring está caducado y esta página lo
repetía.** Medido en v18 y v19 (`tools/bench-orm`, `test_wizard.py`):

| Lo que dice el docstring | Lo que hace |
|---|---|
| «all users can create new records» | **No.** Sin `ir.model.access`, un usuario normal se come un `AccessError` al crear. Igual que un modelo normal. |
| «may only access the records they created» | **No.** El usuario B **ve y lee** el registro que creó A si el ACL le da lectura. No hay aislamiento por creador. |

**No queda código que lo implemente, y conviene decirlo de forma que resista un `grep`**
(re-medido 2026-08-17). `_transient` aparece **10 veces** en `odoo18/odoo/models.py` —no 3—, pero
**ninguna de las diez decide un permiso**: el default (`:567`, `:7472`, `:7486`), el registro de
clases (`:782`, `:800-801`), la prohibición de un m2o de Model a TransientModel (`:3063`, `:3100`),
`is_transient()` (`:6113`, `:6119`) y el vacuum. Igual en v19 (`odoo/orm/models.py:386`, `BaseModel._transient`, `:2962`,
`:2999`, `:5738-5744`).

Y ojo con la capa de permisos, porque el grep ingenuo parece desmentir esto: **`ir_model.py` sí
menciona `_transient`** —`:427` lo persiste en el campo `transient` de `ir.model` y `:479` lo lee de
vuelta al reconstruir la clase (v19: `:434`/`:488`)—, pero eso es **guardar una bandera, no conceder
nada**. Quien decide de verdad, `ir_rule.py`, tiene **0 ocurrencias** de `transient` en v18 y en v19.

**Trátalo como un modelo normal para permisos.** Si un wizard no debe ver lo que
metió otro usuario, eso lo pone una `ir.rule` con
`[('create_uid', '=', user.id)]` — no viene de serie.

Sigue haciendo falta un `ir.model.access` explícito o el usuario obtiene
`AccessError` al pulsar el botón.

Patrón habitual: dar CRU (create/read/write, normalmente sin unlink) al grupo
funcional que usa el wizard. Ejemplo real de `account`, **idéntico en v17, v18 y v19 pero en tres
líneas distintas** (`security/ir.model.access.csv`: v17 `:119`, v18 `:128`, v19 `:121` — verificado
2026-08-17; esta cita estaba **sin versión**, y el `:128` solo vale para v18):

```csv
id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
access_account_move_reversal,access.account.move.reversal,model_account_move_reversal,account.group_account_invoice,1,1,1,0
```

`perm_unlink=0` es seguro: el vacuum borra con `sudo()`, no necesita el permiso
del usuario. Delega la generación a `odoo-security-designer`.

## Los Many2one del wizard: el `ondelete` por defecto no es el de siempre

Un wizard casi siempre apunta a los registros sobre los que actúa. Ese m2o
**recibe un `ondelete` distinto del que recibiría en un modelo normal**, y solo
cuando es `required` (`odoo18/odoo/fields.py:3241-3249`, `Many2one.setup_nonrelated`, v19
`odoo/orm/fields_relational.py:274-282`, `Many2one.setup_nonrelated`):

| m2o hacia un modelo normal | En un modelo normal | En un TransientModel |
|---|---|---|
| sin `required` | `set null` | `set null` |
| **con `required=True`** | **`restrict`** | **`cascade`** |

Medido: borrar el registro destino con un wizard `required` apuntándole **pasa** y
se lleva la fila del wizard por delante; con un modelo normal apuntándole, el
mismo borrado revienta con `ForeignKeyViolation` de psycopg2 (no un `UserError`).
Es deliberado — el comentario del ORM dice que las FK desde transients «bloquean
borrados» — y es una razón más para no meter un wizard en medio de un flujo de
datos: **no protege nada de lo que apunta**.

### Al revés está prohibido, y falla al instalar

Un `Many2one` de un modelo **normal** hacia un transient no es un aviso ni una
mala práctica: es un `ValueError` en `Many2one.update_db`
(`odoo18/odoo/fields.py:3263-3267`, `Many2one.update_db`, v19 `odoo/orm/fields_relational.py:296-300`).
Medido en las dos versiones (`tools/bench-orm/probe-m2o-transient.sh`):

```
ValueError: Many2one dyn.bad.normal.wiz_id from Model to TransientModel is forbidden
CRITICAL … Failed to initialize database
```

Falla **al instalar/actualizar el módulo**, no al usarlo: el registro no llega a
cargar y la base se queda a medias. Si aparece ese CRITICAL en un despliegue,
busca el m2o hacia el wizard.
