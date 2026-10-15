# Patrón 23 — Test de informe QWeb (render)

**Cuándo:** el módulo añade o hace override de un informe (`ir.actions.report` + plantilla QWeb),
o de un `report.<módulo>.<modelo>` (`_get_report_values`). Un informe roto (un `t-field` a un campo
que no existe, un `object.foo` inválido, un `_get_report_values` que peta) **no lo pilla ningún test
de modelo**: solo explota al imprimir, normalmente en producción. Este test lo hace barato.

No re-testea el motor QWeb ni wkhtmltopdf: comprueba que **tu informe renderiza sin excepción** y que
**aparece el dato que tu módulo aporta**.

## API verificada (estable v16→v19)

`self.env['ir.actions.report']` (source: `base/models/ir_actions_report.py`):
- `_render_qweb_html(report_ref, docids, data=None)` → `(bytes_html, 'html')`
- `_render_qweb_pdf(report_ref, res_ids=None, data=None)` → `(bytes_pdf, 'pdf')`

`report_ref` = el **XML id** del `ir.actions.report` (`'mi_modulo.action_report_x'`). Testea sobre
**HTML** (rápido, sin wkhtmltopdf); reserva PDF para un smoke opcional.

## Estructura

- **Fichero:** `tests/test_report.py` · **Clase:** `Test<Módulo>Report`
- **Base:** `TransactionCase` (o la common del dominio si el doc necesela cadena, p.ej. `AccountTestInvoicingCommon`).
- **Tag:** `@tagged('post_install', '-at_install')` (el informe y su plantilla deben estar instalados).

```python
from odoo.tests import TransactionCase, tagged

@tagged('post_install', '-at_install')
class TestSaleRappelReport(TransactionCase):
    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.partner = cls.env['res.partner'].create({'name': 'E2E Report Client'})
        cls.doc = cls.env['sale.order'].create({
            'partner_id': cls.partner.id,
            'order_line': [(0, 0, {'name': 'Línea', 'price_unit': 100})],
        })

    def test_report_renders_and_shows_contribution(self):
        html, rtype = self.env['ir.actions.report']._render_qweb_html(
            'sale_rappel.action_report_rappel', self.doc.ids)
        self.assertEqual(rtype, 'html')
        content = html.decode() if isinstance(html, bytes) else str(html)
        # NO basta "renderizó": asertar el dato que APORTA el módulo en el informe
        self.assertIn('Rappel', content, "el informe debe mostrar la sección de rappel del módulo")
        self.assertIn(self.partner.name, content)
```

## Anti-hueco (obligatorio)

- Aserta **contenido concreto que tu módulo mete** en el informe (un importe, un bloque, un label
  nuevo), no solo `assertTrue(html)` ni "no lanzó excepción". Aplica las 2 golden rules de
  `adversarial-cases.md`.
- Si el informe usa un modelo `report.<módulo>.<modelo>` con `_get_report_values`, **testea también
  ese método** por separado: `self.env['report.<módulo>.<modelo>']._get_report_values(self.doc.ids)`
  → asertar las claves/valores que construye (es lógica tuya, no de QWeb).
- Con varios `docids` (informe por lote): pasa 2+ ids y asegura que salen ambos.

## Smoke PDF (opcional, antes de entregar)

`_render_qweb_pdf(ref, ids)` valida la cadena completa, pero **requiere wkhtmltopdf** en el entorno y
es lento → no lo pongas como test estándar de CI salvo que el layout PDF sea crítico. El de HTML es el entregable.

## Detección

`ir.actions.report` en XML (`<record model="ir.actions.report">` o `<report .../>`) o un modelo
`report.<módulo>.*` con `_get_report_values` → generar este test. Ref: `detection-matrix.md` §2.4b.
