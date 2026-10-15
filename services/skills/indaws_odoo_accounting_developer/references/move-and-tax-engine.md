# account.move flow + account.tax compute engine

> Las citas `file::symbol` son **recibos**: si el número de línea no cuadra, manda el símbolo (el archivo se refactoriza, el símbolo aguanta). Todas las rutas son relativas a `addons/account/` salvo indicación. Snapshot **v18 / v19 CE verificado 2026-07** contra fuente local (resolver vía `references/shared/odoo-source.md`; NO asumir rutas de máquina). Cuando algo NO se verificó, se dice explícitamente en línea.

---

## (a) account.move — máquina de estados y posting

### Estado (`state`) — IDÉNTICO v18/v19

```python
state = fields.Selection(
    [('draft','Draft'), ('posted','Posted'), ('cancel','Cancelled')],
    required=True, readonly=True, copy=False, tracking=True, default='draft')
```

`account_move.py::state` (v18 `:148` / v19 `:129`, definición byte-a-byte igual). **Solo tres estados: no hay paso intermedio de aprobación en el core** — si necesitas un flujo de validación lo añades tú (no lo simules con un cuarto valor de `state`, rompe reporting/EDI que asume estos tres). `move_type` también idéntico: `entry / out_invoice / out_refund / in_invoice / in_refund / out_receipt / in_receipt`.

### `_post(self, soft=True)` — v18 `account/models/account_move.py:5101`, `AccountMove._unlink_or_reverse` / v19 `:5524`

Firma idéntica. Cuerpo leído en v18; en v19 confirmé firma y ubicación pero **NO leí el cuerpo línea a línea** — asumo equivalente por igualdad de firma y estructura. Lo que hace `_post` (v18):

1. **Control de acceso** (`:5115`): si `not env.su` y el usuario no está en `account.group_account_invoice` → `AccessError`.
2. **Validaciones** que se acumulan en `validation_msgs` y se lanzan como un **único `UserError`** (`:5214`): quick_edit total ≠ amount_total; cuenta bancaria archivada/no confiable (`RedirectWarning`/`UserError`); total negativo (obliga a nota de crédito); partner obligatorio en venta/compra; `invoice_date` (se pone hoy en ventas si falta, obligatorio en compras); no permite journal archivado, moneda inactiva, cuenta deprecada, cuentas de otra compañía, cuenta analítica archivada.
3. **Lock dates** (`:5235-5239`): por cada asiento, `move._get_violated_lock_dates(move.date, affects_tax_report)`; si hay violación reajusta `move.date = move._get_accounting_date(...)`.
4. **Side-effects** (`:5242`+): `to_post.line_ids._create_analytic_lines()`, copia de recurrentes, corrección de partner en líneas.
5. **Transición real de estado** (`:5260`): `to_post.write({'state':'posted','posted_before':True})`. Tras ello: `_reconcile_reversed_moves`, `line_ids._reconcile_marked()`, `message_subscribe`, incrementa `customer_rank`/`supplier_rank`, dispara `_invoice_paid_hook()` para facturas de importe cero. Devuelve `to_post`.
6. **`soft=True`**: asientos con `date` futura se marcan `auto_post='at_date'` y NO se postean aún (`to_post = self - future_moves`).

**Numeración (`name`)**: NO se asigna dentro de `_post`. Sale del campo calculado `name` / mecanismo de secuencia (`sequence_prefix` + `sequence_number`, `_set_next_made_sequence_gap`). **NO VERIFICADO** el instante exacto de asignación del `name` al postear (no abrí `_compute_name` ni `_get_last_sequence`).

### `_compute_amount` — v18 `:1132` / v19 `:1166`

Alimenta `amount_untaxed / amount_tax / amount_total / amount_residual` y sus `_signed`. Recorre `move.line_ids` y clasifica por `display_type`: `'tax'` (o `'rounding'` con `tax_repartition_line_id`) → impuesto; `'product'`/`'rounding'` → base; `'payment_term'` → residual. Aplica `move.direction_sign`. Todos los `amount_*` son `compute='_compute_amount', store=True, readonly=True`; `amount_total` además tiene `inverse='_inverse_amount_total'` (`:512`).

---

## (b) `line_ids` vs `invoice_line_ids` — lo que más se equivoca la gente

`line_ids` = **conjunto COMPLETO** de apuntes contables (producto + impuestos + término de pago + secciones/notas):

```python
line_ids = fields.One2many('account.move.line', 'move_id',
                            string='Journal Items', copy=True)
```
`account_move.py::line_ids` (v18 `:203` / v19 `:181`, idéntico).

`invoice_line_ids` = **subconjunto** que edita el usuario en la pestaña de factura. Comentario literal en fuente: `# /!\ invoice_line_ids is just a subset of line_ids.`
- v18 `:339`: `domain=[('display_type','in',('product','line_section','line_note'))]`, `copy=False`.
- v19 `:365`: dominio AMPLIADO con `line_subsection` → `('product','line_section','line_subsection','line_note')`.

**v19: nuevo `display_type` `line_subsection`** (subsecciones dentro de la factura). Si escribes código que enumera display_types válidos, en v19 tienes uno más.

**v19: nuevo campo `journal_line_ids`** (`account/models/account_move.py:189`, `AccountMove.journal_id`), ya marcado DEPRECATED en la propia fuente: `# /!\ Deprecated! Kept in stable, but will be removed in master.` Dominio `('display_type','not in',('line_section','line_subsection','line_note'))`, `exportable=False`. **No existe en v18. No lo uses** aunque aparezca.

### Regla de oro

- Para leer/sumar el asiento completo → `line_ids`.
- Para inyectar líneas de producto como lo haría el usuario → `invoice_line_ids`.
- **NUNCA crees a mano las líneas de impuesto / término de pago** en `line_ids`: las materializa la sincronización dinámica (abajo). Si las metes tú, o se duplican o el balance salta.

---

## (c) Sincronización de líneas dinámicas — no la puentees

El método es `_sync_dynamic_lines(self, container)` (**generator**) en AMBAS versiones, y **sigue en `addons/account/models/account_move.py`**: la reestructuración del ORM a `odoo/orm/` en v19 afecta al framework base, NO a los addons de negocio como `account`.

Es lo que convierte tus `invoice_line_ids` en el asiento equilibrado: crea/actualiza líneas de impuesto (`_sync_tax_lines`), término de pago, redondeo, descuentos y EPD, y equilibra descuadres.

- **v18** `:3258`: implementación explícita. Abre `_disable_recursion(container,'skip_invoice_sync')`; si ya está deshabilitado → `yield; return`. Materializa por orden con `ExitStack`: term (`_sync_dynamic_line(line_type='payment_term')`), `_sync_unbalanced_lines`, `_sync_rounding_lines`, discount_allocation, `_sync_tax_lines(tax_container)` (`:3288`), epd, `_sync_invoice`. Helpers de bajo nivel: `_sync_dynamic_line(existing_key_fname, needed_vals_fname, needed_dirty_fname, line_type, container)` (`:3126`), `_sync_tax_lines` (`:2963`), `_sync_dynamic_line_needed_values` (`:2931`).
- **v19** `:3747`: **REFACTORIZADO** — el qué/en-qué-orden se delega a `stack_list, update_containers = self._get_sync_stack(container)`, luego `stack_list.sort()` y `stack.enter_context(contextmgr)` por cada `(_seq, contextmgr)`. **Mismo contrato externo** (generator + `_disable_recursion('skip_invoice_sync')`), distinta organización interna. Si extiendes esto en v19, engancha en `_get_sync_stack`, no en el cuerpo.

Se invoca desde `create` (v18 `:3395`→`:3400`), `write` (`:3413`→`:3474`) y `onchange` (`:3587`), siempre envolviendo el bloque en `with ... self._sync_dynamic_lines(container)`. `_sanitize_vals` (v18 `:3355`) fusiona `invoice_line_ids` dentro de `line_ids` cuando llegan ambos en los mismos vals.

### Cómo sobreescribir `_post` con seguridad

```python
def _post(self, soft=True):
    posted = super()._post(soft=soft)
    # posted = SOLO los asientos que realmente pasaron a 'posted'
    # (con soft=True los futuros quedan fuera). Actúa sobre 'posted', no sobre 'self'.
    for move in posted.filtered(lambda m: m.is_sale_document()):
        move._do_your_thing()
    return posted
```

Reglas: (1) opera sobre el **return** (`posted`), no sobre `self` — con `soft=True` no todo `self` se postea. (2) Tus validaciones extra van **antes** del `super()` si deben impedir el posting, y lanza un `UserError` claro. (3) Si necesitas tocar líneas por efecto de un impuesto/cálculo, hazlo dentro de la sincronización dinámica (abajo), no en `_post`.

### Añadir un efecto de impuesto/línea calculada

No metas la línea en `_post` ni en un `@api.depends`. Extiende la sincronización dinámica: en **v18** engancha en el helper correspondiente (`_sync_dynamic_line` / `_sync_tax_lines`); en **v19** añade tu manager al stack vía override de `_get_sync_stack`. Toda línea calculada que añadas debe llevar `display_type` coherente para que `_compute_amount` la clasifique bien.

---

## (d) account.tax — `compute_all`

### Firma — IDÉNTICA v18/v19

`account_tax.py::compute_all` (v18 `:4177` / v19 `:4960`):

```python
def compute_all(self, price_unit, currency=None, quantity=1.0, product=None,
                partner=None, is_refund=False, handle_price_include=True,
                include_caba_tags=False, rounding_method=None):
```

En v18/v19 es un **wrapper legacy sobre el nuevo motor genérico de impuestos**: construye `base_line` con `_prepare_base_line_for_taxes_computation(...)`, luego `_add_tax_details_in_base_line(base_line, company, rounding_method=rounding_method)` y `_add_accounting_data_to_base_line_tax_details(...)`, y traduce `tax_details['taxes_data']` al formato antiguo. **NO leí el cuerpo interno de ese motor genérico** — solo confirmé que `compute_all` lo invoca y las claves que devuelve.

`special_mode`: si `force_price_include` está en contexto → `'total_included'`/`'total_excluded'`; si `handle_price_include=False` → `'total_excluded'`; si no → `False`.

### Dict de retorno — idéntico v18/v19 (v19 `:5070`)

```python
{
  'base_tags': [ids],          # a nivel RAÍZ
  'taxes': [ { ... } ],        # una entrada por impuesto aplicado
  'total_excluded': float,
  'total_included': float,
  'total_void': float,
}
```

Cada entrada de `taxes`: `id, name, amount, base, sequence, account_id, analytic, use_in_tax_closing, is_reverse_charge, price_include, tax_exigibility, tax_repartition_line_id, group, tag_ids, tax_ids`.

**Discrepancia docstring vs código (verificada en el código)**: el docstring lista `refund_account_id` por impuesto y `base_tags` dentro de cada tax; el **código real** emite `base_tags` a nivel raíz y **NO emite `refund_account_id`** por tax. Además emite `use_in_tax_closing`/`is_reverse_charge` que el docstring NO lista. Fíate del código.

**Redondeo de totales** solo si `context.get('round_base', True)` → `currency.round(...)`. v18 lo lee con `self._context.get('round_base', ...)`, v19 con `self.env.context.get('round_base', ...)` (mismo efecto).

### `amount_type` (Tax Computation) — v18 `:104`, idéntico v19

`group`, `fixed`, `percent`, `division` (`'Percentage Tax Included'`). Modificadores: `include_base_amount` (el impuesto se suma a la base de los impuestos posteriores) e `is_base_affected` (a este impuesto le afecta la base de los anteriores).

---

## (e) price_include & redondeo

### price_include — RENOMBRADO/REESTRUCTURADO (clave al portar de v17)

`price_include` **ya NO es un Boolean almacenado editable**: es `compute='_compute_price_include', search='_search_price_include'` (v18 `:127` / v19 `:137`), **no store**.

El campo real editable es **`price_include_override`**:

```python
price_include_override = fields.Selection(
    [('tax_included','Tax Included'), ('tax_excluded','Tax Excluded')])
```
(v18 `:132` / v19 `:142`). Más `company_price_include = fields.Selection(related="company_id.account_price_include")`.

Lógica `_compute_price_include` (v18 `:262` / v19 `:302`):

```python
price_include = (price_include_override == 'tax_included') \
    or (company_price_include == 'tax_included' and not price_include_override)
```

Es decir: **el override del impuesto gana**; si no hay override, hereda el default de compañía. Si en v17 escribías `price_include=True` en datos XML/tests, en v18/v19 escribe `price_include_override='tax_included'`.

### Repartition lines (`account.tax.repartition.line`)

Dos One2many en `account.tax`: `invoice_repartition_line_ids` (`document_type='invoice'`) y `refund_repartition_line_ids` (`document_type='refund'`), ambos `compute=..., store=True, readonly=False`, `inverse_name='tax_id'` (v18 `:170`/`:178`). Por defecto el `_compute_*` crea una línea `repartition_type='base'` y otra `'tax'`.

Modelo `account.tax.repartition.line` (v18 `:4578`, `_order='document_type, repartition_type, sequence, id'`):
- `factor_percent = Float(default=100, digits=(16,12), required=True)` — el % que edita el usuario.
- `factor = Float(compute='_compute_factor')` = `factor_percent / 100.0` (`_compute_factor` v18 `:4627`).
- `repartition_type = Selection([('base','Base'),('tax','of tax')], required=True, default='tax')`.
- `document_type = Selection([('invoice','Invoice'),('refund','Refund')], required=True)`.
- `tag_ids` (Tax Grids — casillas del modelo/303 en l10n_es), `account_id`, `use_in_tax_closing` (compute, store, readonly=False).
- **Reverse charge / ±100**: `has_negative_factor` (compute) detecta líneas `repartition_type='tax'` con `factor < 0`. Así se modela el ISP (inversión del sujeto pasivo): dos líneas de impuesto +100/-100.

### Redondeo — `tax_calculation_rounding_method`

En `res.company` (`company.py` v18 `:126` / v19 `:131`):

```python
tax_calculation_rounding_method = fields.Selection(
    [('round_per_line','Round per Line'), ('round_globally','Round Globally')],
    default='round_per_line')
```

Espejado en `res_config_settings`, `account.move` y `account.move.line`. Idéntico v18/v19. `round_per_line` (default) redondea cada línea; `round_globally` acumula y redondea el total — cambia céntimos en facturas multilínea, ojo al comparar contra un ERP externo.

---

## (f) Widget tax_totals — punto de extensión

### Campo `tax_totals` — v18 `account/models/account_move.py:551`, `class AccountMove` / v19 `:593`

```python
tax_totals = fields.Binary(string="Invoice Totals",
                           compute='_compute_tax_totals',
                           inverse='_inverse_tax_totals', exportable=False)
```
Idéntico. Es un `Binary` (dict serializado), no un valor contable — no lo persistas, se recalcula.

### `_compute_tax_totals` — v18 `:1687` / v19 `:1819` (cuerpo idéntico)

Solo en facturas (`is_invoice(include_receipts=True)`); en no-facturas → `None`. Obtiene `base_lines, _ = move._get_rounded_base_and_tax_lines()` y llama al **motor nuevo**:

```python
self.env['account.tax']._get_tax_totals_summary(
    base_lines=..., currency=..., company=...,
    cash_rounding=move.invoice_cash_rounding_id)
```
Añade `display_in_company_currency`. `_get_tax_totals_summary` está en v19 `account/models/account_tax.py:2715`, `AccountTax._get_tax_totals_summary` (**no leí su cuerpo**, solo el punto de llamada y que produce el dict que consume el widget).

### Widget JS — v18/v19 idéntico

`static/src/components/tax_totals/tax_totals.js`:
- `export class TaxTotalsComponent extends Component` (v18 `:125` / v19 `:123`), `static template = "account.TaxTotalsField"`, `static components = { TaxGroupComponent }`.
- Registro: `registry.category("fields").add("account-tax-totals-field", taxTotalsComponent)` (v18 `:175` / v19 `:173`).

**La clave de registro del widget es `account-tax-totals-field`** — es lo que pones en la vista y lo que heredas/reemplazas para personalizar el bloque de totales. El template QWeb `account.TaxTotalsField` **NO lo verifiqué** (solo la clase JS y su registro); si lo extiendes, confírmalo en el XML del componente.

---

## Contexto de entorno (verificado presente en disco, no inspeccionado por dentro)

- EDI CE v18: `account_edi`, `account_edi_proxy_client`, `account_edi_ubl_cii`, `account_edi_ubl_cii_tax_extension`.
- Localización ES presente: `l10n_es`, `l10n_es_edi_facturae`, `l10n_es_edi_sii`, `l10n_es_edi_tbai(_pos)`, `l10n_es_edi_verifactu(_pos)`, `l10n_es_modelo130`, `l10n_es_pos`; y OCA en `l10n-spain` (v18). **No inspeccioné el interior** de los `l10n_es_edi_*` (TicketBAI / Verifactu / Facturae) — solo confirmé presencia.


## `_sync_dynamic_lines` es un context manager, no un método que se llame

Verificado 2026-08-07. La firma real:

```python
# odoo18/addons/account/models/account_move.py:3260-3261 · odoo19/…:3746-3747
@contextmanager
def _sync_dynamic_lines(self, container):
    with self._disable_recursion(container, 'skip_invoice_sync') as disabled:
        if disabled:
            yield
            return
```

Consecuencias, y son las que explican los dos gotchas de `line_ids`:

- **Envuelve** las escrituras: se entra desde `create` (`:3879` v19), `write` (`:3958`) y otros caminos
  (`:4876`, `:5969`). No es un paso que puedas saltarte llamando a otra cosa — **no existe una vía de
  escritura al modelo que no pase por dentro**. Por eso lo que pongas a mano en `line_ids` se pierde:
  no es que "algo lo sobrescriba después", es que la regeneración es parte de la propia escritura.
- La escotilla es `skip_invoice_sync`, y **Odoo la usa para sí mismo**: en el borrado —con el
  comentario *"no need to sync to delete everything"* (`:4057`)— y en las reversas (`:5479-5482`). Que
  el propio core la use en esos dos casos es la mejor señal de cuándo es legítima: cuando vas a
  destruir o reconstruir el conjunto entero, no cuando quieres retocar una línea.

Made with odoo-accounting-developer · v0.1.2
