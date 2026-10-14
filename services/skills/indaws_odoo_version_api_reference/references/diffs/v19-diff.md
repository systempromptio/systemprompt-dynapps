<!-- verified: v19.0 @ 2026-06 (auditado vs $ODOO_SRC/odoo19; corregidos _cr/_context/_uid, active_id, models.Index, rpc) -->
# v19 Diff — Changes from v18 to v19

Everything in `v18-base/` is the canonical reference. This file documents ONLY what changes in Odoo 19.

## Breaking Changes (code will fail if not updated)

### 1. `@api.returns` removed

```python
# v18 (works)
@api.returns('self')
def copy(self, default=None):
    ...

# v19 (AttributeError: module 'odoo.api' has no attribute 'returns')
# Remove the decorator, return values normally
def copy(self, default=None):
    ...
```

### 2. `res.groups` privilege system overhaul

```python
# v18: groups use category_id
<field name="category_id" ref="base.module_category_sales"/>

# v19: new intermediate model res.groups.privilege
<record id="privilege_sales" model="res.groups.privilege">
    <field name="name">Sales</field>
</record>
<field name="privilege_id" ref="privilege_sales"/>
```

Field renames on `res.groups`:
- `category_id` → `privilege_id`
- `users` → `user_ids`

Field rename on `res.users`:
- `groups_id` → `group_ids`

> **Not just a rename — a semantic split (gets access checks wrong if missed).**
> In v16–v18, `groups_id` held **both** directly-assigned and implied groups. v19
> splits them:
> - `group_ids` — groups assigned **directly** to the user.
> - `all_group_ids` — direct **+ implied** (the transitive closure).
>
> For an **authorisation check** use `all_group_ids`, not `group_ids`. A check that
> allows e.g. `base.group_user` (which most internal users hold *implicitly*) will
> wrongly deny everyone if it tests `group_ids` — this shipped as a real 403 bug.
> ```python
> # WRONG in v19 — misses implied groups → false denials
> if allowed_group in user.group_ids:
> # CORRECT in v19 — direct + implied
> if allowed_group in user.all_group_ids:
> ```
> Pure rename sites (an `ir.rule` referencing direct assignment, a form editing the
> user's direct groups) map to `group_ids`. The test of intent: *editing* direct
> membership → `group_ids`; *deciding whether a user is allowed* → `all_group_ids`.

### 3. `expression.AND/OR` → `Domain` class

```python
# v18
from odoo.osv.expression import AND, OR
domain = AND([domain1, domain2])

# v19 (new, preferred)
from odoo.fields import Domain
domain = Domain(domain1) & Domain(domain2)
domain = Domain(domain1) | Domain(domain2)
# Old AND/OR still work but are deprecated
```

### 4. `_sql_constraints` → `models.Constraint`

```python
# v18
_sql_constraints = [
    ('name_uniq', 'unique(name)', 'Name must be unique!'),
]

# v19 (new, preferred) — class attribute with `_` prefix, NOT a list
class MyModel(models.Model):
    _name = 'my.model'

    name = fields.Char(required=True)

    _name_uniq = models.Constraint(
        'unique(name)',
        'Name must be unique!',
    )
# ⚠ In v19 the legacy `_sql_constraints` LIST is NO LONGER APPLIED — it only logs
#   a warning ("Model attribute '_sql_constraints' is no longer supported") and the
#   constraint is NOT created. You MUST migrate to models.Constraint.
#   Verified contra CE19 @d1d161033: odoo19/odoo/orm/model_classes.py:162-164 (warns, does
#   not process the list). El path `odoo/orm/` ya es v19-only, pero se explicita.
# Each constraint is a separate class attribute; the constraint name
# (used internally by Postgres) is taken from the attribute name with
# the leading `_` stripped → `name_uniq`.
```

### 5. `base_import.Import` class renamed

```python
# v18
from odoo.addons.base_import.models.base_import import Import

# v19
from odoo.addons.base_import.models.base_import import Base_ImportImport as Import
```

### 6. `record._cr/_context/_uid` deprecated (NOT removed)

```python
# v19: siguen existiendo como @property @api.deprecated (odoo/orm/models.py:5909-5923)
# → FUNCIONAN con DeprecationWarning, NO lanzan AttributeError.
record._cr, record._context, record._uid

# Migrar a (sin warning):
self.env.cr, self.env.context, self.env.uid
```

> ⚠ No marcar el uso de `_cr/_context/_uid` como código roto en v19 — es un warning, no un error. Verificado contra CE19 `@d1d161033`: `odoo/orm/models.py:5911-5923`, `BaseModel._cr`.

---

## XML View Changes

### 7. `<tree>` → `<list>` — **NO es un diff de v19: el corte es v17→v18** (corregido 2026-08-07)

Esta entrada decía "v18 (both work)" y es **falso**. Evidencia:

| Versión | `ir.ui.view.type` | Ficheros de `addons/*/views/*.xml` con `<tree ` |
|---|---|---|
| 17.0 | `('tree','Tree')` — `odoo/addons/base/models/ir_ui_view.py:163`, `View.type`, v17 | **322** |
| 18.0 | `('list','List')` — `odoo/addons/base/models/ir_ui_view.py:153`, `View.type`, v18 | **0** |
| 19.0 | `('list','List')` — `:149` | 0 |

No hay shim: `'tree'` no aparece en el `ir_ui_view.py` de v18 ni de v19. Así que **en v18 `<list>` ya
es obligatorio**, no opcional. Se deja aquí porque quien venga de v17 lo vive como novedad, pero
**no cuenta como cambio v18→v19** (es la segunda entrada mal clasificada de este fichero; ver §26).

```xml
<!-- v17 (y anteriores) -->
<tree string="Orders" editable="top">

<!-- v18 y v19 -->
<list string="Orders" editable="top">
```

Note: `edit="1"` attribute removed in v19. Use `editable="top"` or `editable="bottom"`.

### 8. Search view `<group>` removed

```xml
<!-- v18 -->
<search>
    <group expand="0" string="Group By">
        <filter name="by_state" string="State" context="{'group_by': 'state'}"/>
    </group>
</search>

<!-- v19: remove <group> wrapper, keep filters at root -->
<search>
    <filter name="by_state" string="State" context="{'group_by': 'state'}"/>
</search>
```

### 9. Kanban `kanban-box` → `card` — **el cambio empieza en v18; lo de v19 es que deja de perdonar**

> **Corregido 2026-08-17.** El ejemplo anterior presentaba `kanban-box` como «lo de v18», y en v18
> ya está **deprecado y avisado por log**: `_validate_tag_kanban` suelta
> *"'kanban-box' is deprecated, define a 'card' template instead"*
> (`odoo18/odoo/addons/base/models/ir_ui_view.py:1515-1517`, `View._validate_tag_kanban`). Y `card` **ya es lo que v18 genera**
> por su cuenta (`kanban_card = E.t(field, {'t-name': "card"})`, `:2530`; v19 `:2847`). Escribir
> `kanban-box` en un módulo v18 nuevo es escribir código deprecado desde el minuto uno.

```xml
<!-- v17: kanban-box, sin aviso -->
<t t-name="kanban-box">

<!-- v18: kanban-box FUNCIONA pero avisa en el log; card ya se acepta -> usa card -->
<!-- v19: solo card (sin card, la vista falla) -->
<t t-name="card">
```

### 10. ~~Form context `active_id` → `id`~~ (FALSO — no aplica)

`active_id` sigue siendo el estándar en contextos de acciones/botones en v19
(verificado: `addons/sale/views/crm_team_views.xml`, `account/views/account_views.xml`
lo usan masivamente). **NO migrar `active_id`→`id`** — rompería los contextos. No existe
tal rename como regla general.

### 11. `numbercall` — **NO es un cambio de v19**: ya no existe en v18

> **Corregido 2026-08-17.** Esta entrada estaba mal ubicada y su ejemplo ensenaba lo contrario de la
> realidad: mostraba `<field name="numbercall">-1</field>` como el «antes» de **v18**, dando a
> entender que alli es valido. **No lo es.** Medido: `numbercall` y `doall` tienen **0 ocurrencias**
> en CE18 y en CE19, y existen en v17 (`odoo/addons/base/models/ir_cron.py:67`, `ir_cron`, v17). El corte es **v17->v18**.

```xml
<!-- v17: valido -->
<field name="numbercall">-1</field>

<!-- v18 Y v19: la linea no existe; ponerla da Invalid field 'numbercall' -->
```

Ver el diff de v18 para el cambio real y su sustituto (`_notify_progress(..., deactivate=True)`).

---

## JavaScript Changes

### 12. RPC: servicio → función importable (NO fetch crudo)

```javascript
// v18 (frontend/portal): servicio vía hook
this.rpc = useService("rpc");
const data = await this.rpc("/api/endpoint", params);

// v19: rpc es una FUNCIÓN que se importa (verificado web/static/src/core/network/rpc.js:93;
// usada en addons/portal/static/src/interactions/*.js)
import { rpc } from "@web/core/network/rpc";
const data = await rpc("/api/endpoint", params);
```

Nota: NO migrar a `fetch` crudo. En backend (OWL) se inyecta igualmente; el cambio es
que ya no es un service vía `useService("rpc")` sino la función importada.

### 13. `publicWidget.include()` removed

```javascript
// v18
publicWidget.registry.MyWidget.include({...});

// v19: use patch()
import { patch } from "@web/core/utils/patch";
patch(MyWidget.prototype, {...});
```

### 14. Legacy `odoo.define()` — **NO está eliminado en v19** (corregido 2026-08-17)

> **Esta entrada decía «removed» y es falso.** Medido contra CE19 `@d1d161033`: el transpiler
> **sigue tratándolo** —`legacy_odoo_define = get_aliased_odoo_define_content(module_path, content)`
> (`odoo/tools/js_transpiler.py:29`, `transpile_javascript`), `wrap_with_odoo_define` (`odoo/tools/js_transpiler.py:50`, `transpile_javascript`), la función definida en
> `:731`— y el propio Odoo **lo emite**: `odoo/addons/base/models/assetsbundle.py:327`, `def js` genera
> `odoo.define("<bundle>.bundle.xml", ["@web/core/templates"], function(require) {...})`. El
> docstring del módulo lo describe como el formato de los *"classic Odoo modules"* (`:3`).
>
> Lo que cambia es **convención, no disponibilidad**: en v19 se escribe ES6 y la ruta decide el
> nombre del módulo (ver §sobre `@odoo-module`). Un `odoo.define` heredado no revienta al migrar —
> queda como código legacy que nadie mantiene, que es otro problema y merece otra frase.

```javascript
// v17/v18/v19: sigue funcionando (transpiler + alias)
odoo.define('module.name', function (require) {...});

// v19 (y v18): lo que se escribe hoy — ES modules
import { Component } from "@odoo/owl";
```

### 15b. `publicWidget.Widget` → `Interaction` class

New lightweight class for frontend/portal components:

```javascript
// v18
import publicWidget from "@web/legacy/js/public/public_widget";
publicWidget.registry.MyWidget = publicWidget.Widget.extend({
    selector: '.my-selector',
    events: { 'click .btn': '_onClick' },
    start() { return this._super(...arguments); },
    _onClick(ev) { /* handle */ },
});

// v19: Interaction class (powered by new Colibri framework)
import { Interaction } from "@web/public/interaction";
import { registry } from "@web/core/registry";

export class MyWidget extends Interaction {
    static selector = ".my-selector";
    dynamicContent = {
        '.btn': { 't-on-click': this.onClick },
    };
    setup() { /* replaces start() */ }
    onClick(ev) { /* handle */ }
}
registry.category("public.interactions").add("my_module.MyWidget", MyWidget);
```

Key differences:
- `events: {}` → `dynamicContent = {}` with QWeb-like directives
- `start()` → `setup()` (sync) + `willStart()` (async)
- `this._super()` → `super.method()`
- `publicWidget.registry` → `registry.category("public.interactions")`
- Access services via `this.services['http']` instead of `useService()`
- Built-in helpers: `this.debounced()`, `this.locked()`, `this.waitFor()`

### 15c. Registry category renamed

```javascript
// v18
registry.category("public_components").add("my_comp", { Component });

// v19
registry.category("public.interactions").add("my_comp", MyInteraction);
```

### 15d. Import path changes

| v18 require/import | v19 import |
|---|---|
| `require('web.public.widget')` | `import { Interaction } from "@web/public/interaction"` |
| `require('web.Widget')` | `import { Component } from "@odoo/owl"` |
| `require('web.core')` | `import { _t } from "@web/core/l10n/translation"` |
| `require('web.rpc')` | `import { rpc } from "@web/core/network/rpc"` (backend) |
| `require('web.Dialog')` | `useService("dialog")` |
| `require('portal.PortalSidebar')` | `import { ... } from "@portal/interactions/..."` |

### 15e. `this._super()` → `super.method()`

```javascript
// v18 (.include/.extend pattern)
start() {
    return this._super.apply(this, arguments);
}

// v19 (patch/class pattern)
setup() {
    super.setup();
}
```

**Warning**: `super` only works in regular methods, NOT in arrow functions.

---

## SCSS/Theme Changes

### 15. Theme variable prefix required

```scss
// v18
$headings-font-weight: 700;

// v19: must use $o-theme- prefix
$o-theme-headings-font-weight: 700;
```

### 16. Color palette requires menu/footer/copyright

```scss
// v19: these keys are MANDATORY in color palettes
$o-color-palettes: map-merge($o-color-palettes, (
    'my_theme': (
        'o-color-1': #124F81,
        // ...
        'menu': 1,        // REQUIRED
        'footer': 4,      // REQUIRED
        'copyright': 5,   // REQUIRED
    ),
));
```

### 17. Font config requires properties section

```scss
// v19: properties section mandatory
$o-theme-font-configs: map-merge($o-theme-font-configs, (
    'Inter': (
        'family': ('Inter', sans-serif),
        'url': 'Inter:300,400,500,600,700&display=swap',
        'properties': (           // REQUIRED in v19
            'base': (
                'font-size-base': 1rem,    // rem, not px
            ),
        ),
    ),
));
```

### 18. Units: px → rem

```scss
// v18
'btn-padding-y': 12px,

// v19
'btn-padding-y': 0.75rem,
```

---

## Test Framework Changes

### 19. `safe_eval()` `nocopy` parameter removed

```python
# v18
safe_eval(expr, globals_dict, nocopy=True)

# v19: remove nocopy parameter
safe_eval(expr, globals_dict)
```

### 20. Form class strictness

In v19, `Form()` raises errors for accessing invisible/missing fields. Tests that access fields hidden by `invisible` conditions must set the conditions first.

### 21. Model field removals — break code AND test setups (verified 2026-07)

- **`res.partner.mobile` REMOVED.** Reading it as a field (`partner.mobile`) raises
  `AttributeError`; creating/writing it raises `ValueError: Invalid field 'mobile' in
  'res.partner'`. Version-safe access: `partner.phone or getattr(partner, "mobile", "") or ""`.
  **Common test trap:** `res.partner` created in `setUpClass` with `"mobile": ...` → the whole
  test class ERRORs at setUp. Remove `mobile` from partner `create()` in tests.
- **`stock.move.name` REMOVED** (was `fields.Char('Description', required=True)`; v19 only has
  `description_picking`). **Test trap:** `env["stock.move"].create({"name": ...})` → invalid
  field. Use `description_picking`. Version-safe: `if "name" in env["stock.move"]._fields: ...`.
- **`env["procurement.group"]` raises `KeyError` in v19** (model not resolvable that way, even
  with stock installed). To link a picking↔sale order in a test, don't build the group by hand:
  confirm the SO (`order.action_confirm()`) and use `order.picking_ids` (`picking.sale_id` is set).

These bite carrier/stock modules and their tests especially. See the migration checklist below.

### 22. Stock / UoM / packaging / FSM (verified against source + docker 2026-07-13)

Config-heavy area with several silent behavioural traps — verify against source, do NOT design from memory.

- **`stock.quant.package` → `stock.package`** (model renamed). `env["stock.quant.package"]` → `KeyError`. Quant still has `package_id` / `lot_id`.
- **`uom.uom` uses `relative_factor` + `relative_uom_id`** (NOT `factor` / `factor_inv`). `factor` is now a **computed, read-only** absolute quantity. Create a "bigger" UoM with `{'relative_uom_id': litre.id, 'relative_factor': 5.0}`. Also: `uom.category_id` removed (no hierarchical category).
- **`product.template` UoM fields: `uom_id` + `uom_ids`** (M2M "Packagings" for purchase/sale). Fields `allowed_uom_ids`, `packaging_uom_id` and `uom_po_id` **do NOT exist**. To buy in a non-reference UoM add it to `uom_ids`.
- **`stock.package.type` has NO volume field** (`max_weight` + `height`/`width`/`packaging_length`/`base_weight` only). Package capacity validates by **weight**, not volume.
- **A UoM never creates packages.** Receiving "10 × Drum-5L" enters 50 L as one quant, NOT 10 `stock.package`. Per-container identity requires **Put in Pack** on receipt (or a packaging flow), not the UoM.
- **`stock.warehouse.orderpoint` triggers on `qty_on_hand < product_min_qty` (strict).** `Min=0` **never** fires; even `Min=0.001` did not procure in test (needs a real threshold, e.g. `Min=1`). An orderpoint procures a *quantity to reach max* — it does NOT "move one whole package". Don't use it for "one open container at a time" logic (`stock_orderpoint.py` `_compute_qty_to_order`).
- **`product.template.invoice_policy` value is `'delivery'`** (label "Delivered quantities"), NOT `'delivered'`. Same in v17/v18 — a common from-memory error.
- **FSM (`industry_fsm_*`):** material is created in `product_product._inverse_fsm_quantity` (and the lot wizard `fsm_stock_tracking`), **NOT** in `_fsm_create_sale_order_line` (that one is timesheets only). `qty_delivered` is set by `industry_fsm_stock._validate_stock`. A SQL CHECK (`project_project._material_imply_billable`) forces `allow_material ⇒ allow_billable`. `sale_project._check_sale_line_type` forbids linking `task.sale_line_id` to a **goods** line (must be a service). `task.under_warranty = True` sets material `price_unit = 0` on both paths.

### 23. `res.partner.mobile` and friends — see section 21

### 24. `account.move.line.display_type == 'product'` — **la trampa NO es de v19: viene de v16** (corregido 2026-08-17)

> **Esta entrada estaba mal fechada.** El valor `'product'` está en la `selection` de
> `account.move.line.display_type` en **v16, v17, v18 y v19** — medido en los cuatro árboles
> (`account/models/account_move_line.py`: v16 `:281`, v17 `:291`, v18 `:306`, v19 `:329`), y el campo
> es `required=True, store=True, precompute=True` en todos. O sea que **el guard roto lleva roto
> desde v16**, y quien esté en v17 o v18 leyendo este fichero concluye *"esto es cosa de v19, yo
> estoy a salvo"* — que es exactamente el error caro.

**La trampa (v16+):** en las líneas de factura, las líneas de producto reales llevan
`display_type='product'` — una cadena **truthy**, no `False`. Las de sección/nota son
`'line_section'` / `'line_note'`. Así que el guard habitual `if line.display_type: continue`
—escrito para saltarse secciones y notas— **se salta TODAS las líneas**, productos incluidos, y el
bucle se convierte en un no-op silencioso. Lo correcto es comprobar el valor:
`if line.display_type in ('line_section', 'line_note'): continue`, o mejor
`if line.display_type != 'product': continue` según lo que quieras.

**Lo que SÍ cambia en v19** es que la `selection` crece. Valores por major:

| | v16 | v17 / v18 | v19 |
|---|---|---|---|
| comunes | `product` `cogs` `tax` `rounding` `payment_term` `line_section` `line_note` `epd` | + `discount` | + `discount` |
| nuevos | — | `discount` (v17) | **`line_subsection`, `non_deductible_product_total`, `non_deductible_product`, `non_deductible_tax`** |

Si tu código enumera valores de `display_type` a mano, **v19 añade cuatro** que no contemplabas —
y `line_subsection` es de las que se comportan como sección.

```python
# WRONG desde v16 (no solo en v19) — se salta tambien las lineas de producto
for line in move.invoice_line_ids:
    if line.display_type:            # 'product' → truthy → skips everything
        continue
    ...

# RIGHT — match the real line type explicitly
for line in move.invoice_line_ids:
    if line.display_type != 'product':   # or: in ('line_section','line_note','line_subsection')
        continue
    ...
```

Trap: `sale.order.line.display_type` **sigue siendo `False`** en las lineas de producto reales —
su `selection` solo tiene secciones y notas, verificado en los tres majors:
`sale/models/sale_order_line.py:65-69`, `SaleOrderLine.display_type` (v17), `:67-71` (v18), `:63-68` (v19, que anade
`line_subsection`). O sea que **el mismo idiom `if line.display_type` es correcto en
`sale.order.line` y esta MAL en `account.move.line`** — y lo esta desde v16, no desde v19. No
arrastres el patron de un modelo al otro.

### 25. ~~`amount_type='code'` (Python-code tax) REMOVED in v19~~ — **FALSO, corregido 2026-08-17**

> ⚠️ **Esta entrada era falsa y el consejo que daba era caro.** Decía que el valor `'code'` y el
> módulo `account_tax_python` estaban *"gone — not in CE, Enterprise 19, nor OCA v19"*. Medido hoy
> contra CE19 `@d1d161033`:
>
> - **`odoo19/addons/account_tax_python` EXISTE** (`__manifest__.py`: *"Define Taxes as Python
>   Code"*), con sus `models/`, `views/`, `tests/` y `tools/`.
> - **Y sigue añadiendo el valor**: `selection_add=[('code', "Custom Formula")]`
>   (`account_tax_python/models/account_tax.py:16`, `AccountTax.amount_type`, v19; en v18 es `:23`), con su `ondelete` que
>   degrada a `percent` al desinstalar (`:17`).
> - Lo que sí es cierto —y era el grano de verdad que confundió— es que **el core de `account` no
>   trae `'code'` en ninguna versión**: su selection es `('group','fixed','percent','division')` en
>   v17 (`account/models/account_tax.py:101-102`, `AccountTax.amount_type`), v18 (`account/models/account_tax.py:104-105`, `AccountTax.amount_type`) y v19 (`account/models/account_tax.py:84-85`, `AccountTax.amount_type`). El valor **siempre** vino del
>   módulo, nunca del core.
>
> **O sea: los impuestos por código Python siguen existiendo en v19.** Si una skill o un diseño
> concluyó *"hay que rediseñar los impuestos calculados, ya no existen"*, la premisa era falsa.
> Lo único que cambia entre v18 y v19 en ese módulo son líneas, no la funcionalidad.

### 26. `@http.route(auth='none')` es `readonly=True` por defecto — **desde v18, NO es un diff de v19** (corregido 2026-08-07)

⚠️ **Esta entrada estaba mal clasificada.** Se marcó "verified vs source 2026-07" comprobando v19 y
sin comprobar v18. El mecanismo es **idéntico en las dos versiones**:

| Versión | Línea |
|---|---|
| v17 | **no existe**: `readonly` es una clave interna fijada a `False` (`odoo/http.py:880`, `_generate_routing_rules`, v17), sin `:param readonly` en el docstring (0 ocurrencias) |
| v18 | `default_mode = submethod.original_routing.get('readonly', default_auth == 'none')` — `odoo/http.py:890`, `_check_and_complete_route_definition`, v18, y el param documentado en `odoo/http.py:728-730`, `route` |
| v19 | lo mismo, `odoo/http.py:974`, `_check_and_complete_route_definition`, v19 |

Así que **el corte real es v17→v18**. Se deja aquí porque el comportamiento sigue vigente en v19 y
porque quien venga de v17 lo vive como novedad, pero **no cuenta como cambio v18→v19**.

El comportamiento (válido en v18 y v19): **una ruta con `auth='none'` corre en transacción READ-ONLY salvo que declares lo contrario**. Un endpoint `auth='none'` que ESCRIBE (o llama a algo que escribe) falla con `"cannot execute UPDATE in a read-only transaction"` y Odoo hace un auto-retry en modo escritura (lento y confuso).

```python
# v18 y v19 — endpoint auth='none' que escribe: DEBE marcar readonly=False explícito
@http.route('/webhook/ingest', type='json', auth='none', csrf=False,
            methods=['POST'], readonly=False)   # ← sin esto, UPDATE peta
def ingest(self, **kw):
    request.env['my.model'].sudo().create({...})
```

Ejemplo en core: `auth_oauth` marca `readonly=False` en sus rutas de login que escriben sesión. Rutas `auth='user'`/`'public'` siguen el default histórico (escritura permitida) salvo que pongas `readonly=True`.

---

## New in v19 (additions, not breaking)

### Type hints

```python
from odoo.api import DomainType, Self
# Type aliases available for better IDE support
```

### Json2 dispatcher

New JSON-RPC 2.0 dispatcher as primary external protocol. The old JSON-RPC dispatcher still works.

### `models.Index` / `models.UniqueIndex`

```python
# Se declaran como ATRIBUTOS DE CLASE (como models.Constraint), NO en una lista.
# El argumento es UN string de definición SQL (no args separados).
# Verificado contra CE19 @d1d161033: odoo19/odoo/orm/table_objects.py:125-200; usos en
# odoo19/addons/hr_recruitment/models/hr_applicant.py:146 y odoo19/addons/mail/models/mail_message.py:205.
# OJO: esas dos lineas son OTRA COSA en v18 (`similar_candidates = ...` y `user = self.env.user`).
_partner_date_idx = models.Index("USING btree (partner_id, date)")
_name_company_uniq = models.UniqueIndex("(name, company_id)", "El nombre debe ser único")
```

### `@api.private` — **NO es de v19: existe desde v17** (corregido 2026-08-17)

Methods decorated with `@api.private` cannot be called via RPC (replaces the `_` prefix convention
for new code).

> **Pero no es una novedad de v19.** El decorador está definido en **v17** (`odoo/api.py:384`, `private`) y
> **v18** (`odoo/api.py:448`, `private`); lo que v19 hace es **mudarlo de fichero**, a `odoo/orm/decorators.py:328`, `private`
> (reexportado por `odoo/api/__init__.py`, así que `@api.private` sigue escribiéndose igual).
> Se queda en este documento **solo por el cambio de ruta**, que afecta a quien cite la fuente —
> no al que lo use.

### `self.env._()` for translations — **disponible desde v18, no es de v19** (corregido 2026-08-17)

> **Medido**: `env._(` aparece **0 veces en el core de v17, 34 en el de v18 y 121 en el de v19**.
> O sea que en v18 **ya está disponible y el propio Odoo lo usa**; lo que cambia en v19 es que se
> generaliza. Si tu target es v18, **puedes usarlo ya** — esta entrada daba a entender que no.

```python
# v17: unica via
from odoo import _
message = _("Hello %(name)s", name=user.name)

# v18 Y v19 (preferido — lazy + lang-aware); en v18 el core ya lo usa 34 veces
message = self.env._("Hello %(name)s", name=user.name)

# In controllers (v19)
message = request.env._("Hello %(name)s", name=user.name)
```

The legacy `_()` function still works but evaluates the translation eagerly using the lang available at call time without proper request scoping. `self.env._()` evaluates against `self.env.context['lang']`, which is the correct lang for the current request/user. Pylint_odoo flags the legacy form via the `prefer-env-translation` rule.

For new code in v19+, default to `self.env._(...)`. For legacy code, migrate progressively when touching the file.

---

## Migration checklist v18 → v19

- [ ] Remove `@api.returns` decorators
- [ ] Replace `category_id` → `privilege_id` in group definitions
- [ ] Replace `users` → `user_ids` on `res.groups` XML
- [ ] Replace `groups_id` → `group_ids` on `res.users` references
- [ ] Replace `<tree>` → `<list>` in all XML views
- [ ] Remove `<group>` wrappers from search views
- [ ] Replace `kanban-box` → `card` in kanban templates
- [ ] Remove `numbercall` from cron definitions
- [ ] Frontend: `import { rpc } from "@web/core/network/rpc"` (función, no servicio; NO fetch crudo)
- [ ] Replace `publicWidget.Widget.extend()` with `Interaction` class
- [ ] Replace `publicWidget.include()` / `.extend()` with `patch()`
- [ ] Replace `events: {}` with `dynamicContent = {}` in Interactions
- [ ] Replace `start()` with `setup()` in Interactions
- [ ] Replace `this._super()` with `super.method()`
- [ ] Replace `registry.category("public_components")` with `"public.interactions"`
- [ ] Convert remaining `odoo.define()` to ES modules
- [ ] Update legacy `require()` imports to ES `import` paths
- [ ] Update theme SCSS variables (prefix, palette keys, rem units)
- [ ] Migrate `record._cr/_context/_uid` → `self.env.*` (deprecation warning, NO rompe)
- [ ] Update `base_import.Import` → `Base_ImportImport`
- [ ] Remove `nocopy` from `safe_eval()` calls
- [ ] Test Form-based tests for invisible field strictness
- [ ] Replace top-level `_(...)` with `self.env._(...)` (or `request.env._(...)` in controllers) for proper lazy/lang-aware translation
- [ ] Remove `res.partner.mobile` usage (code + test partner `create`); use `phone`/`getattr`
- [ ] Replace `stock.move` `name` with `description_picking` (code + test move `create`)
- [ ] Stop using `env["procurement.group"]` in tests; confirm SO → `picking_ids` instead
- [ ] Replace `stock.quant.package` → `stock.package`
- [ ] UoM: use `relative_factor`/`relative_uom_id` (not `factor`/`factor_inv`); product UoMs in `uom_ids` (not `allowed_uom_ids`/`packaging_uom_id`/`uom_po_id`)
- [ ] `invoice_policy` value is `'delivery'` (not `'delivered'`)
- [ ] Stock/UoM/FSM config: validate mechanism with a rollback probe before declaring "config-only" (orderpoint Min=0 doesn't fire; UoM doesn't create packages; package.type has no volume)
