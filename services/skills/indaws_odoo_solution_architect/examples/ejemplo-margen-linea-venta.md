# Ejemplo end-to-end — Margen por línea en el pedido de venta

Muestra el flujo completo del skill: Specify → Design → Skeleton → Handoff. Odoo 18.0.

> Nota: las citas de este ejemplo están **resueltas contra fuente** (2026-08-07). Hasta esa fecha iban
> como `[verificar]` y **apuntaban al fichero equivocado** (`sale_margin/models/sale_order.py`, cuando
> el campo está en `sale_order_line.py`): un placeholder también se copia. En un caso real haz el grep
> y pon la cita; si no la tienes, escribe que no la tienes — no inventes la ruta.

---

## Step 0 — Specify

- **Necesidad**: el director comercial necesita ver el % de margen en cada línea del pedido para
  aprobar descuentos sin sacar Excel.
- **Escenarios**: (1) coste 50 / precio 100 → 50%. (2) cambio precio a 75 → recalcula 33,3%.
  (3) precio 0 → 0% (no dividir por cero).
- **Versión**: 18.0.

## Step 1 — ¿Se debe construir?

`dev-assessment`: Studio permite un campo, pero el cálculo depende de `purchase_price` y debe
recomputar → lógica no trivial → **módulo gobernado (Custom horizontal)**, no Studio. Útil a >1
cliente → candidato a Core. Confirmado: se diseña.

## Step 2 — Diseño (ADR)

- **Mecanismo central (tracer bullet)**: de dónde sale el coste. En `sale.order.line` el coste está
  en `purchase_price` (campo estándar del módulo `sale_margin`) —
  `addons/sale_margin/models/sale_order_line.py:15`, `SaleOrderLine.purchase_price` (verificado 2026-08-07; misma línea en 17.0, 18.0
  y 19.0). **Todo el diseño depende de que ese campo exista y esté
  poblado** → si `sale_margin` no está instalado, no hay coste. Decisión: `depends` de `sale_margin`.
- **Patrón**: herencia `_inherit = "sale.order.line"` — extender, no tocar core.
- **Frontera**: 1 módulo, 1 problema (mostrar margen). Core intacto.
- **Alternativa descartada**: campo `related` a un coste calculado a mano → descartada, `sale_margin`
  ya resuelve el coste y reinventarlo genera deuda.
- **Slices**: (1) campo + compute + verlo en la línea [contiene tracer bullet]. (2) formato/umbral
  visual de margen bajo (opcional, fase abierta).
- **Walking skeleton**: crear un pedido con una línea → el margen se muestra y recalcula.

## Step 3 — Esqueleto (contrato)

`__manifest__.py`:
```python
{
    "name": "sale_line_margin_percent",
    "version": "18.0.1.0.0",           # CONTRACT
    "depends": ["sale_margin"],         # CONTRACT — verificado: aporta purchase_price
    "data": ["views/sale_order_views.xml"],
    "license": "LGPL-3",
}
```

`models/sale_order_line.py`:
```python
class SaleOrderLine(models.Model):
    _inherit = "sale.order.line"

    margin_percent = fields.Float(     # CONTRACT — nombre/tipo/store fijos
        string="Margen %",
        compute="_compute_margin_percent",
        store=True,
        # Regla: (price_subtotal - purchase_price*qty) / price_subtotal * 100
        # Invariante: price_subtotal == 0 → 0.0
    )

    @api.depends("price_subtotal", "purchase_price", "product_uom_qty")  # CONTRACT
    def _compute_margin_percent(self):
        # TRACER BULLET (pseudo + fuente) — el riesgo era "¿de dónde sale el coste?".
        # Verificado: el coste está en purchase_price — sale_margin/models/sale_order_line.py:15.
        # PSEUDO (el dev pone el cemento):
        #   para cada línea:
        #     base = price_subtotal
        #     coste = purchase_price * product_uom_qty
        #     margin_percent = 0.0 si base == 0, si no (base - coste) / base * 100
        ...
```

Criterios de aceptación (QUÉ verificar — el dev escribe los tests):
```
- Cuando coste 50 y precio 100 → margin_percent = 50.0
- Cuando cambio precio a 75      → recalcula 33.3
- Cuando precio 0                → margin_percent = 0.0 (no div/0)

Requisito (DoD): tests unitarios de los 3 criterios + flujo e2e
(crear pedido → añadir línea → ver/recalcular margen), todo en verde.
```

Vista (`odoo-view-designer`): añadir `margin_percent` a la `<list>` de líneas (v18 `<list>`, no
`<tree>`) con `groups=` si el margen es sensible (`odoo-security-designer`).

## Step 4/5 — Handoff

- **Contrato fijo**: firma de `_compute_margin_percent`, `depends`, nombre/tipo del campo, los 3
  criterios de aceptación.
- **Libertad del dev**: formato/redondeo, el widget de la vista, umbral visual de la fase 2, y los
  tests (unit + e2e) que cubren los criterios.
- **DoR**: cumplida. **DoD**: tests unitarios (3 criterios) + flujo e2e en verde, margen visible,
  impacto cubierto (¿registros existentes? → recompute en post-init de las líneas ya creadas).
- Se pasa a `dev-task-writer` para empaquetar. Nivel dev sugerido: Junior (planos + pseudocódigo
  hechos, el cemento es acotado). Estimación: probable ~3h.
