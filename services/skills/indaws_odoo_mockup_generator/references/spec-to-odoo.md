# Emisor: spec → esqueleto Odoo (modelo Python + vistas XML)

La misma spec de 2 capas (`spec-schema.md`) que genera el mockup produce el **esqueleto real** que
consume `odoo-module-creator`. Consultor diseña → cliente valida el mockup → el dev arranca de aquí.
Prototipo funcional en `assets/mockup-engine.html` (pestaña "Código Odoo").

## Modelo Python (`models/<modelo>.py`)
- **Solo campos `origin:"new"`**; los `std` vienen del `_inherit`. Si no hay `inherit` → `_name` +
  `_inherit=['mail.thread','mail.activity.mixin']` (si la vista tiene chatter) + `_description`.
- **tipo ORM → `fields.*`** (verificado en `odoo/fields.py`):

| type spec | Python |
|---|---|
| char/text/integer/float/boolean/date/datetime/html/binary | `fields.Char/Text/Integer/Float/Boolean/Date/Datetime/Html/Binary` |
| monetary | `fields.Monetary(currency_field='currency_id')` (exige un `currency_id`) |
| selection | `fields.Selection([('a','A'),...])` |
| many2one | `fields.Many2one('<comodel>')` |
| one2many | `fields.One2many('<comodel>', '<inverse>')` (crea el modelo hijo + su M2o inverso) |
| many2many | `fields.Many2many('<comodel>')` |

- `required`/`readonly` → kwargs. **`compute:true`** → `compute='_compute_<name>', store=True` + stub
  `def _compute_<name>` con `@api.depends()` a rellenar. `invisible` es de vista, NO va al campo.
- Cada `one2many` genera además la **clase del comodelo** con sus columnas + `x_orden_id` inverso.

## Vistas XML (`views/<modelo>_views.xml`) — sintaxis por versión (NO de memoria)
Resolver deltas con `odoo-version-api-reference`. Verificado v18:
- **list, no tree**: `<list>` (v18+); `<tree>` solo ≤v17.
- **chatter**: `<chatter/>` (v18+); `<div class="oe_chatter">` en v17.
- **kanban**: `<t t-name="card">` (v18/v19); `kanban-box` deprecado en v18, prohibido en v19.
- **modificadores**: `invisible="..."`, `required="..."`, `readonly="..."` directos (NO `attrs=`/`states=`, eliminados en v17).
- **widget**: `<field name="x" widget="..."/>` solo si difiere del por-defecto del tipo.

Estructura del form: `<form><header>`(botones `type="object"` + `<field name="state" widget="statusbar"/>`)
`<sheet>`(`oe_button_box` con `oe_stat_button`+`statinfo` → `oe_title`/`<h1>` → `<group><group>…</group><group>…</group></group>` 2-col → `<notebook><page><field name="o2m"><list editable="bottom">…`) `<chatter/>`.

## Reglas de oro del emisor
1. **No inventes campos estándar**: si `origin:"std"` no se redeclara (viene del inherit).
2. **Widget válido para el tipo** (matriz `widget-matrix.json`): si no, no lo emitas — corrige el widget.
3. **Lo `compute`/lógico va como stub `# TODO`**, no inventado — el emisor da el esqueleto, no la lógica.
4. **Nombre técnico**: `x_*` = campo/modelo de Studio; sin `x_` = módulo de desarrollo (avisa cuál toca).
5. Entrega el esqueleto a `odoo-module-creator` (que aplica estándares Dynapps, seguridad, tests).
