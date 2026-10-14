# Esquema del inventario de capacidades

`extract_inventory.py` produce un JSON por entorno (read-only, solo SELECTs):

```json
{
  "meta":   { "label": "old|migrated|oracle", "version": "16", "db": "..." },
  "modules": { "<module>": { "author": "...", "website": "...", "state": "installed" } },
  "elements": { "<categoria>": [ { "id": "...", ... }, ... ] },
  "counts":  { "<categoria>": 123 }
}
```

`modules` es el mapa que permite **atribuir el dueño** de cada elemento
(custom / studio / oca / core / vendor) por el autor/website del módulo, sin
depender de un prefijo fijo.

## Categorías e identificadores (estables entre versiones)

| Categoría | Fuente | `id` (estable) | Campos extra |
|---|---|---|---|
| `models` | `ir_model` (+ `ir_model_data`, `pg_class`) | `model` | `module` (dueño), `rows` (aprox., null si abstracto) |
| `fields` | `ir_model_fields` | `model::name` | `ttype`, `state` (`manual`=custom/Studio), `stored` |
| `views` | `ir_ui_view` + `ir_model_data` | `module.name` (o `noxmlid:<id>`) | `module`, `type`, `model`, `active` |
| `menus` | `ir_model_data` (ir.ui.menu) | `module.name` | `module` |
| `actions` | `ir_model_data` (act_window/server/report/client) | `module.name` | `module`, `kind` |
| `reports` | `ir_actions_report` | `report_name` | `model` |
| `access` | `ir_model_access` | `model|grupo-xmlid|perms(rwcu)` | — |
| `rules` | `ir_model_data` (ir.rule) | `module.name` | `module` |
| `filters` | `ir_filters` | `name::model` | `model`, `blank_domain` |
| `crons` | `ir_model_data` (ir.cron) | `module.name` | `module` |
| `automations` | `ir_model_data` (base.automation) | `module.name` | `module` |
| `mail_templates` | `ir_model_data` (mail.template) | `module.name` | `module` |
| `groups` | `ir_model_data` (res.groups) | `module.name` | `module` |

## Por qué estos identificadores

- **`xmlid` (`module.name`)** es el identificador que sobrevive a un major
  upgrade para registros de datos (vistas, menús, acciones, informes, plantillas,
  crons, grupos, reglas). Comparar por id numérico o por etiqueta traducida
  produce ruido falso.
- **`model::campo`** para campos y **`model`** para modelos son estables por
  definición.
- **`active`** en vistas permite distinguir *desaparecida* de *presente pero
  desactivada* (regresión típica del Upgrade Service).
- **`blank_domain`** en filtros marca un `domain` required dejado vacío (`NULL`
  o `''`) por el upgrade — `'[]'` es un dominio vacío VÁLIDO, no cuenta.
- **`rows`** en modelos es el conteo aproximado de filas (`pg_class.reltuples`,
  O(1), sin escaneo). Es la señal de **pérdida de datos**: un modelo que
  desaparece con `rows != 0` se llevó datos. `null` = modelo abstracto/transient
  (sin tabla física). Es una estimación: si la BD no está `ANALYZE`-ada puede
  quedar obsoleta — para el gate de pérdida de datos conviene un `ANALYZE` previo.

## Notas de robustez

- Cada categoría degrada a `[]` si su tabla/columna no existe en esa versión
  (no aborta la extracción).
- Los campos `state='manual'` cubren tanto Studio como campos custom por código
  añadidos manualmente; para vistas Studio, el dueño es el módulo
  `studio_customization`.
- La extracción **no arranca Odoo**: solo necesita la BD. Reserva el runtime
  para la Capa 2 (comportamiento).

## `view_fields` — qué campos hay DENTRO de cada vista

Añadido el 2026-08-15. Id: `<xmlid de la vista>::<nombre del campo>`.

Sin esta categoría, «un campo que ya no está en el formulario» —el ejemplo con el que se explica
esta capa— **no se detectaba**: la vista sigue existiendo y el campo sigue en `ir_model_fields`, así
que nada cambiaba. Medido en `tools/bench-parity`.

Se extrae del `arch_db`, que es un `jsonb {idioma: xml}`: hay que **decodificarlo** antes de buscar,
porque en el texto crudo las comillas van escapadas (`name=\"x\"`) y una expresión regular ingenua
devuelve cero campos sin dar ningún error. Se busca con regex y no con un parser XML a propósito:
una vista con arch roto degrada en vez de tumbar el inventario.

**Coste**: es la categoría más numerosa. Comparando dos majors con solo `base` instalado aporta un
**28 %** más de hallazgos, casi todos INTER_VERSION si hay oráculo, y UNKNOWN si no lo hay. Si el
triage se hace inmanejable, el oráculo deja de ser «recomendado» y pasa a ser necesario.
