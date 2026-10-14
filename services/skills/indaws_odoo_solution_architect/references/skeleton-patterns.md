# Patrones de Esqueleto Odoo (Step 3)

Planos, no cemento. El arquitecto entrega **firmas + docstrings de contrato + pseudocódigo**; el dev
escribe el código real (`# PSEUDO`). **Nada lleva Python ejecutable** — ni el núcleo de riesgo, que
va como pseudocódigo detallado + cita a fuente.

> **Sintaxis version-específica: NO de memoria.** v18 es la base de esta skill, pero el alcance
> declarado es v17-v19 y **no todo lo de v18 vale en v17**. Verificado 2026-08-07:
>
> | Punto | 17.0 | 18.0 | 19.0 |
> |---|---|---|---|
> | Etiqueta de la vista lista | **`<tree>`** — `('tree','Tree')` en `odoo17/odoo/addons/base/models/ir_ui_view.py:163`, `View.type`. `<list>` NO existe | `<list>` (`:153`) | `<list>` (`:149`) |
> | `attrs=` / `states=` | **ya eliminados**: `ValidationError('Since 17.0, the "attrs" and "states" attributes are no longer used…')` en `odoo/addons/base/models/ir_ui_view.py:393`, `View._check_xml` | igual (`:387`) | igual |
> | `Command.*` vs tuplas legacy | `Command.*` (v15+) | igual | igual |
> | `res.groups` | `category_id` | `category_id` (`odoo/addons/base/models/res_users.py:191`, `class Groups`) | **`privilege_id`** → `res.groups.privilege` (`odoo/addons/base/models/res_groups.py:36`, `class ResGroups`; el modelo se muda a su propio fichero) |
> | Constraints SQL | `_sql_constraints` lista | igual | **`models.Constraint`** (`odoo/orm/table_objects.py:79`, `Constraint`); la lista solo avisa (`odoo/orm/model_classes.py:162-163`, `def add_to_registry`) |
> | **Chatter** (todo modelo con `mail.thread`) | **`<div class="oe_chatter">`** — el compilador registra `selector: "div.oe_chatter"` (`mail/static/src/views/web/form/form_compiler.js:74`, `chatter_compiler`). **0** `<chatter/>` en las vistas del core, 77 `oe_chatter` | **`<chatter/>`** — `selector: "chatter"` (`mail/static/src/chatter/web/form_compiler.js:45-46`, `chatter_compiler`). 85 usos en el core | igual que v18 (85 usos, **0** `oe_chatter`) |
> | **Kanban** | `kanban-box`, sin aviso | `kanban-box` **avisa en log** (`odoo/addons/base/models/ir_ui_view.py:1515-1517`, `View._validate_tag_kanban`) y `card` ya funciona → **emite `card`** | **solo `card`** |
> | **Cron XML** | `numbercall`/`doall` válidos (`odoo/addons/base/models/ir_cron.py:67`, `ir_cron`) | **`Invalid field 'numbercall'` → el módulo no carga** (0 ocurrencias). «Una pasada» = `_notify_progress(..., deactivate=True)` (`:734-749`) | igual que v18 |
>
> Ampliada 2026-08-17 con las tres últimas filas: son elementos que **un esqueleto emite de verdad**
> —chatter en cualquier modelo con `mail.thread`, kanban, cron en `data/`— y las tres cambian en el
> corte **v17→v18**, no en el de v19.
>
> O sea: **si el target es 17.0, el esqueleto lleva `<tree>` y `<div class="oe_chatter">`**. Emitir
> `<list>` contra v17 da un tipo de vista inexistente, y `<chatter/>` en v17 **no lo compila nadie**
> (el selector es otro): la vista carga y el chatter simplemente no aparece — **falla en silencio**.
> Al revés, un `numbercall` heredado de v17 **sí revienta** en v18. El resto de la tabla v18 transfiere
> a v17.
>
> Verificar contra la fuente objetivo o `odoo-version-api-reference` antes de emitir. Delegar la
> generación real de cada parte a su builder (ver `delegation-map.md`).

---

## Marcadores del contrato

- `# CONTRACT — no cambiar`: firma/interfaz que el dev NO puede tocar sin romper el diseño.
- `# PSEUDO: <pasos>`: el cuerpo descrito en pseudocódigo; el dev lo convierte en código real.
- `# TRACER BULLET (pseudo + fuente)`: el núcleo de riesgo, en pseudocódigo detallado + cita `file:line`.
- `# DECIDE-DEV: <qué queda a su criterio>`: libertad explícita del dev.

---

## Modelo — campos con firma, sin lógica

```python
class SaleOrderLine(models.Model):
    _inherit = "sale.order.line"

    # CONTRACT — campo del contrato. El dev NO cambia nombre/tipo/store.
    margin_percent = fields.Float(
        string="Margen %",
        compute="_compute_margin_percent",
        store=True,
        # Regla (contrato): (price_subtotal - purchase_price) / price_subtotal * 100.
        # Invariante: si price_subtotal == 0 → 0.0 (no dividir por cero).
        # Recompute: al cambiar price_unit, product_id o purchase_price.
    )

    @api.depends("price_subtotal", "purchase_price")  # CONTRACT — depends fijo
    def _compute_margin_percent(self):
        # PSEUDO (el dev escribe el código):
        #   para cada línea:
        #     si price_subtotal == 0  -> margin_percent = 0.0  (evitar div/0)
        #     si no -> margin_percent = (price_subtotal - coste) / price_subtotal * 100
        # DECIDE-DEV: cómo redondear/formatear internamente.
        ...  # sin cemento: el cuerpo real lo pone el dev
```

## Método — firma + docstring de contrato (Design by Contract)

```python
def _apply_volume_discount(self, qty, price):
    """Devuelve el precio con descuento por volumen aplicado.

    CONTRATO:
      Entradas:  qty (float > 0), price (float >= 0)
      Salida:    float — precio unitario con descuento, siempre <= price
      Invariante: nunca devuelve negativo; qty<=0 → ValueError
      Efectos:   ninguno (función pura, no escribe en BD)
      Lanza:     ValueError si qty <= 0
    """
    # PSEUDO:
    #   validar qty > 0 (si no, ValueError)
    #   buscar el tramo de descuento aplicable a qty
    #   devolver price * (1 - descuento_del_tramo)
    # Ref de patrón: <módulo similar file:line>
    ...
```

## Núcleo de riesgo (tracer bullet) — PSEUDOCÓDIGO + fuente, NO código

```python
def _fsm_hook_precio(self, task):
    # TRACER BULLET (pseudo + fuente) — mecanismo del que depende todo el diseño.
    # Verificado contra fuente v18: <addon/file.py:NNN> (o probe con rollback: <resultado>).
    # PSEUDO:
    #   1. localizar la sale.order.line generada por la task de FSM
    #   2. leer el precio de <campo verificado en fuente>
    #   3. aplicar <regla> y escribir con record.update / write
    # NOTA: no se pone cemento; lo que de-risquea es la cita a fuente, no un cuerpo ejecutable.
    ...
```

## Criterios de aceptación + requisito de testing (NO escribimos el test)

Damos el QUÉ verificar; el dev escribe y ejecuta los tests. Formato "Cuando X, entonces Y":

```
Criterios de aceptación (comportamiento verificable):
  - Cuando coste 50 y precio 100  → margin_percent = 50.0
  - Cuando cambio precio a 75       → recalcula a 33.3
  - Cuando precio 0                 → margin_percent = 0.0 (no div/0)

Requisito de entrega (DoD, innegociable):
  - Tests UNITARIOS que cubran los 3 criterios.
  - Flujo END-TO-END: crear pedido → añadir línea → ver/recalcular margen.
  - Todo en verde. Los tests son del dev (el "cómo verificar").
```

## `__manifest__.py` — deps mínimas verificadas

```python
{
    "name": "...",
    "version": "18.0.1.0.0",          # CONTRACT — versión objetivo
    "depends": ["sale"],               # CONTRACT — mínimas y verificadas (existen y aportan)
    "data": ["security/ir.model.access.csv", "views/sale_order_views.xml"],
    "license": "LGPL-3",
    # DECIDE-DEV: orden de carga interno, autor, category si no está fijado.
}
```

## Árbol de módulo (el esqueleto físico)

Delegar el scaffolding a `odoo-module-creator`. El árbol refleja SRP y separation of concerns (ver
`solution-design-guide.md`). Solo crear las carpetas que el diseño usa — no el árbol completo por
defecto si no hay JS/report/wizard.

---

## Regla de oro

Si te descubres escribiendo Python ejecutable en cualquier cuerpo (incluido el tracer bullet) →
para: eso es cemento. Los planos definen QUÉ y con qué contrato, y describen el CÓMO en pseudocódigo;
el dev pone el cemento (el código real).
