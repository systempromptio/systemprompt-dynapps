# Tipos de campo y atributos

Base **v18**. Fuente de atributos base: `odoo/fields.py:300-330`, `Field` (clase `Field`).
Diffs de version en `version-diffs.md`.

## Tipos de campo

| Categoria | Tipos | Fichero v18 | Fichero v19 |
|-----------|-------|-------------|-------------|
| Numericos | `Integer`, `Float`, `Monetary` | `fields.py` | `orm/fields_numeric.py` |
| Texto | `Char`, `Text`, `Html` | `fields.py` | `orm/fields_textual.py` |
| Booleano/Seleccion | `Boolean`, `Selection` | `fields.py` | `orm/fields_selection.py` |
| Temporal | `Date`, `Datetime` | `fields.py` | `orm/fields_temporal.py` |
| Binario/Imagen | `Binary`, `Image` | `fields.py` | `orm/fields_binary.py` |
| Relacional | `Many2one`, `One2many`, `Many2many`, `Many2oneReference` | `fields.py` | `orm/fields_relational.py` |
| Referencia | `Reference` | `fields.py` | `orm/fields_reference.py` |
| Properties | `Properties`, `PropertiesDefinition` | `odoo/fields.py:3522`, `class Properties` | `odoo/orm/fields_properties.py:32`, `Properties` |
| Misc | `Json`, `Id` | `fields.py` | `orm/fields_misc.py` |

## Atributos comunes (clase base `Field`)

Confirmados en `odoo/fields.py:300-330`, `Field`.

| Atributo | Default | Significado |
|----------|---------|-------------|
| `store` | `True` | Persiste en columna BD. Computed: por defecto `False` salvo que lo fuerces |
| `compute` | `None` | Nombre del metodo `compute(recs)` |
| `compute_sudo` | `False` | Recomputa como superusuario. **Medido**: con él, `env.su` es `True` dentro del compute; sin él, `False` |
| `precompute` | `False` | Computa ANTES del INSERT (solo compute+store; warning si no-computed). Existe desde v16 |
| `inverse` | `None` | Metodo `inverse(recs)` -> hace el campo computed escribible |
| `search` | `None` | Metodo `search(recs, operator, value)` -> hace buscable un computed no-stored |
| `related` | `None` | Secuencia de nombres de campo (`'partner_id.country_id'` se declara como `related='partner_id.country_id'`). **Sin `readonly=False` la escritura se acepta y no hace nada** — ver abajo |
| `recursive` | `False` | El campo depende de si mismo (computes recursivos) |
| `readonly` | `False` | Solo lectura en UI |
| `required` | `False` | NOT NULL / requerido |
| `index` | `None` | Indexacion: `True`/`'btree'`, `'btree_not_null'`, `'trigram'`, `False` |
| `copy` | `True` | Se copia en `copy()`. o2m y campos unicos suelen `copy=False` |
| `default` | `None` | Valor o callable `default=lambda self: ...` |
| `groups` | `None` | CSV de xml ids de grupos que ven el campo |
| `company_dependent` | `False` | Valor por compania (almacenado como property, no columna normal). `odoo/fields.py:314`, `class Field` |
| `aggregator` | `None` | Funcion de agregacion en read_group (`'sum'`, `'avg'`, `'max'`, `'min'`, `'bool_or'`...). **v18+**; en v17 es `group_operator` |
| `group_expand` | `None` | Metodo para expandir grupos vacios en read_group |
| `prefetch` | `True` | Grupo de prefetch (`False` = sin prefetch) |
| `groups` | `None` | Restriccion de visibilidad por grupos |

### Atributos que NO son del core ORM

- **`tracking`**: NO es atributo de la clase `Field`. Lo consume el modulo `mail` via
  `getattr(field, 'tracking', None)` (`mail/models/ir_model_fields.py:21`, `IrModelField.tracking`). Solo
  funciona en modelos que heredan `mail.thread`. Acepta `True` o un entero (orden/secuencia).
- **`depends`**: no se pasa como kwarg; se declara con el decorador `@api.depends(...)` sobre el
  metodo compute (interno `_depends`). Ver `compute-constraints-onchange.md`.

## Atributos ESPECIFICOS por tipo (verificado v18 `fields.py`)

Los de arriba son comunes; estos son los que de verdad se usan por tipo:

- **Char / Text** — `translate` (def `False`, `:276/1804`): traducible por idioma. `size` (Char) longitud
  máx **DEPRECADO** (`:1805`, no usar). `trim` (def `True`, `:2092`) recorta espacios (solo cliente web).
- **Html** — `sanitize`(True)/`sanitize_attributes`(True)/`sanitize_form`(True)/`strip_style`(False)/
  `strip_classes`(False) (`:2154+`): limpieza del HTML. `translate` también aplica.
- **Float / Monetary** — `digits`: precisión, tupla `(16, 2)` o nombre de *Decimal Precision*
  (`digits='Product Price'`). **Monetary** `currency_field` (def `'currency_id'`, `:1718`).
- **Selection** — `selection`: lista de tuplas o nombre de método. `selection_add` (`:2838`) EXTIENDE la
  selección al `_inherit`; exige `ondelete` por valor (`selection_add=[('x','X')], ondelete={'x':'set default'}`).
  Es el patrón para **añadir un valor a una Selection del core** sin pisar los existentes — p.ej. registrar
  un motor de informe nuevo: `report_type = fields.Selection(selection_add=[('my-engine','My Engine')], ondelete={'my-engine':'set default'})` en un `_inherit` de `ir.actions.report`.
- **Many2one** (`:3218+`) — `comodel_name` (1º posicional), `ondelete` (`'set null'`def/`'restrict'`/`'cascade'`),
  `domain`, `context`, `auto_join` (def `False`; genera JOIN en search — ojo con record rules),
  `check_company` (def `False`), **`delegate=True`** (`:3220`) = azúcar de delegación (declara el M2o **y**
  activa `_inherits` sobre ese campo — alternativa a escribir `_inherits`).
- **One2many** (`:4620`) — `comodel_name`, **`inverse_name` (obligatorio**, el M2o del otro lado), `domain`, `context`.
- **Many2many** (`:4934+`) — `comodel_name`; `relation`/`column1`/`column2` **necesarios si hay 2+ m2m entre
  los MISMOS modelos** (si no, colisionan en la tabla puente autogenerada).
- **Binary** (`:2520`) — `attachment` (def **`True`**): se guarda como `ir.attachment`, no en columna;
  `attachment=False` fuerza columna. **Image** añade `max_width`/`max_height` (redimensiona al escribir).

## aggregator vs group_operator (CRITICO)

```python
# v17
total = fields.Float(group_operator='sum')
# v18 / v19  (group_operator emite DeprecationWarning y se mapea a aggregator)
total = fields.Float(aggregator='sum')
```
Fuente warning: `odoo/fields.py:482-483`, `Field._get_attrs` (v18); `odoo/orm/fields.py:487-489`, `Field._get_attrs` (v19).

## company_dependent

```python
price = fields.Float(company_dependent=True)
```
El valor depende de la compania activa (lees el de `self.env.company`; usa `with_company(c)` para otra).

**Almacenamiento — CAMBIO v17→v18 (verificado):**
- **v16/v17**: se guarda como registros del modelo **`ir.property`** (`addons/base/models/ir_property.py`).
- **v18+**: `ir.property` **ELIMINADO**; el valor se guarda como **columna `jsonb`** en la propia tabla, con
  el `company_id` como clave (`odoo/fields.py:180`, `Field`, y `odoo/fields.py:280`, `Field._column_type`). O sea, en v18+ SÍ hay columna (jsonb), no
  tabla `ir.property` aparte. Solo tipos permitidos (`COMPANY_DEPENDENT_FIELDS`, `odoo/fields.py:467`, `Field._get_attrs`); no puede
  ser `required` ni `translate`. Firmas/matices → `odoo-version-api-reference`.

**Qué quiere decir «afecta a search» — MEDIDO** (v18 y v19, `bench-orm::TestCompanyDependent`): las
dos empresas viven en **la misma celda** (`{"1": "de A", "2": "de B"}` en una fila), así que **el
mismo dominio da resultados distintos según quién lo lance**. `search([('nota','=','de A')])`
devuelve el registro con la empresa A activa y **nada** con la B, sin error ni aviso. Consecuencias
prácticas: un dominio de `ir.rule`, un filtro guardado o un cron que corra con otra empresa activa
**no ven lo mismo**; y ningún índice normal sirve (la columna es jsonb).

## Properties (v16+, NO v17+)

Campo dinamico cuyo esquema lo define otro registro via un campo `PropertiesDefinition`.

**Existe desde v16**, no desde v17: `class Properties(Field)` en `odoo16/odoo/fields.py:3232`, `Properties` y
`class PropertiesDefinition(Field)` en `:3787`, y el core de v16 ya los usa
(`odoo16/addons/crm/models/crm_team.py:58`, `Team.lead_properties_definition`, `odoo16/addons/project/models/project.py:370`, `class Project`). En v15 no
existen (0 ocurrencias en `odoo15/odoo/fields.py`). Lo que si cambia en v17 es la **adopcion**: pasa
de 2 ficheros del core a 11. Verificado 2026-08-07.
Declaracion tipica (un `Properties` apunta a la definicion via `definition='parent_id.field_def'`):

```python
properties_definition = fields.PropertiesDefinition('Properties definition')
properties = fields.Properties('Properties', definition='parent_id.properties_definition', copy=True)
```
Fuente v18: `odoo/fields.py:3522`, `Properties` (`Properties`), `:4076` (`PropertiesDefinition`)
(remedidas 2026-08-17 `@99f5a33c64`; decían `:3524`/`:4078`).
v19: `odoo/orm/fields_properties.py:32`, `Properties` y `:844`. Para la lista completa de subatributos
(`comodel_name`, tipos soportados) [delega] en `odoo-version-api-reference`.


## Escribir en un `related` sin `readonly=False` no falla: no hace nada

Medido el 2026-08-15 (`tools/bench-orm`):

| Declaración | Escribir en el campo | El registro de destino queda |
|---|---|---|
| `related='parent_id.label'` | **aceptada, sin excepción** | `'original'` — **no propaga** |
| `related='parent_id.label', readonly=False` | aceptada | `'cambiado'` — propaga |

Un `related` es readonly por defecto, y asignarle un valor **no lanza nada**: el ORM se lo traga
y el dato no llega al modelo de destino. Es el mismo modo de fallo que en el cliente web, donde un
campo `readonly` escrito por un widget se descarta al guardar
(`developer:odoo-owl-component-builder` → `references/view-layer.md`): **el sistema no te avisa de
que tu escritura se ha perdido**.

Si el campo tiene que ser escribible, `readonly=False` — y entonces el ORM escribe en el registro
apuntado, no en este.

## No existe la convención `_default_<campo>`

Comprobado en el mismo banco: un método `_default_magic()` definido en el modelo, sin `default=`
en el campo, **no se usa**. El registro se crea con el campo vacío y nadie avisa. El default se
pasa siempre por `default=`, con literal o callable.


## `ondelete` en un Many2one, medido

Banco `tools/bench-orm`, 2026-08-15, borrando el registro apuntado:

| `ondelete=` | Qué le pasa al registro que apunta | Qué recibe quien borra |
|---|---|---|
| `'set null'` (por defecto) | **sobrevive**, con el campo vacío | nada, el borrado sale bien |
| `'restrict'` | sobrevive | **`ForeignKeyViolation` de psycopg2** |
| `'cascade'` | **se borra también** | nada |

Lo que conviene saber del `'restrict'`: lo que sale **no es un `UserError` de Odoo**, es el error
crudo de Postgres (`violates foreign key constraint ... on table ...`). En la interfaz Odoo lo
traduce, pero en un script, una migración o un test lo que atrapas es la excepción de la base de
datos — y en Postgres una sentencia fallida deja la transacción abortada hasta el rollback.
Envuelve el borrado si necesitas seguir trabajando después.
