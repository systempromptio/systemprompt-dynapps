# Test E2E de flujo de negocio (scoped al módulo)

El **tour** cubre E2E de UI; esto cubre el **E2E de flujo ORM**: recorrer la cadena de negocio
completa (p.ej. venta→albarán→factura) por API/ORM y **asertar lo que el MÓDULO añade/cambia** en
los pasos que toca. No re-testea Odoo estándar; prueba la contribución del módulo dentro del flujo real.

Es el test que exige el estándar de entrega de Desarrollador 2.0 (Gate 4) para un desarrollo: es la
**piedra angular del control de calidad tras crear un módulo** — demuestra que el módulo hace lo que
dice dentro del flujo real, no que "da verde". Un flujo que corre sin asertar la contribución del
módulo NO es control de calidad (ver `references/shared/delivery-standard.md` §1).

## 1. Qué cadena testear (determinista — NO lo adivines)

Delega en `odoo-flow-coverage-planner` → `scripts/map_module_to_flows.py <ruta_modulo>`: por los
`depends` + modelos `_inherit` del módulo devuelve las `chains` del catálogo que toca. Los pasos de
cada cadena las da el `flows.py` de `odoo-flow-tester` con `--chain <ID>` (CHAIN_SALE_TO_CASH, etc.).
Si el mapper devuelve 0 cadenas → el módulo es técnico/utilitario: E2E de flujo puede no aplicar;
prioriza unit (este mismo skill).

## 2. Estructura del test

- **Fichero:** `<módulo>/tests/test_<cadena>_flow.py` (versionado con el módulo, corre con `-u <módulo> --test-enable`).
- **Clase base:** `TransactionCase` (flujo ORM). Para UI → tour (ver `tour-patterns.md`).
- **Tag OBLIGATORIO:** `@tagged('post_install', '-at_install')` — la cadena cruza módulos, deben estar ya instalados.
- **Precondiciones en `setUp`:** construir los maestros mínimos (partner, producto, diario…) — NO depender
  de datos de la instancia ni de demo ajena. Producto/partner propios del test.

```python
from odoo.tests import TransactionCase, tagged

@tagged('post_install', '-at_install')
class TestRappelSaleToCash(TransactionCase):
    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.partner = cls.env['res.partner'].create({'name': 'E2E Client'})
        cls.product = cls.env['product.product'].create({'name': 'E2E Prod', 'list_price': 100})
        # ... precondiciones específicas del módulo (p.ej. configurar el rappel)

    def test_sale_to_cash_applies_module_contribution(self):
        # --- recorrer la cadena estandar (pasos que da flows.py --chain) ---
        so = self.env['sale.order'].create({
            'partner_id': self.partner.id,
            'order_line': [Command.create({'product_id': self.product.id, 'product_uom_qty': 10})],
        })
        so.action_confirm()                       # → delivery
        so.picking_ids.move_ids.quantity = 10
        so.picking_ids.button_validate()
        inv = so._create_invoices(); inv.action_post()
        # --- ASERTAR LO QUE EL MÓDULO APORTA (no que "la cadena corrió") ---
        self.assertTrue(so.rappel_id, "el módulo debe enganchar el rappel al confirmar")
        self.assertEqual(inv.rappel_amount, expected, "el descuento de rappel debe reflejarse en la factura")
```

## 3. Anti-hueco (obligatorio)

- Aserta la **contribución del módulo** en cada paso que toca (campo calculado, estado, línea creada,
  importe), NO solo `self.assertTrue(so)` / "no lanzó excepción". Aplica las 2 golden rules de
  `test-templates.md` y el detector de `references/hollow-tests.md` (odoo-code-auditor).
- Si el módulo añade un campo computed en `account.move`, el E2E debe **llegar a la factura** y asertar ese campo.

## 4. Qué asertar = lo que el scanner marcó del módulo

Del `scan_module` del módulo: los `_inherit` + métodos override + campos nuevos son los **puntos de
contribución**. Cada uno se traduce en una aserción en el paso de la cadena donde actúa (override de
`action_confirm` → aserta tras `action_confirm`; campo computed en la factura → aserta tras `action_post`).

## 5. Smoke opcional (antes de entregar)

`odoo-flow-tester` en modo ejecución (execute + revert) valida que el escenario corre en vivo sin
dejar rastro. Útil como humo antes de commitear el test. No sustituye al test Python (ese es el entregable).
