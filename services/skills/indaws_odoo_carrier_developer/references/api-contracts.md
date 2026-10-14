# Carrier API Contracts Reference

## Complete Method Signatures

This document details the exact interface contracts that every delivery carrier module must implement. These are derived from analysis of `delivery/models/delivery_carrier.py`, `stock_delivery/models/delivery_carrier.py`, and `stock_delivery/models/stock_picking.py` in Odoo 18.

---

## 1. rate_shipment() - The Dispatcher

**Location:** `delivery/models/delivery_carrier.py`

The base `rate_shipment()` calls `<provider>_rate_shipment()` and then applies post-processing:

```python
def rate_shipment(self, order):
    """Dispatcher - DO NOT override this method.

    Post-processing applied automatically:
    1. Fiscal position tax adjustment via product_id._get_tax_included_unit_price()
    2. Margin application via _apply_margins() (margin % + fixed_margin)
    3. Free shipping check (if free_over=True and order total >= amount)
    """
    res = getattr(self, '%s_rate_shipment' % self.delivery_type)(order)
    # ... post-processing ...
    res['carrier_price'] = res['price']  # Save pre-free-shipping price
    return res
```

### Provider Method: `<provider>_rate_shipment(self, order)`

**Input:**
- `self`: `delivery.carrier` recordset (single record, `ensure_one()`)
- `order`: `sale.order` recordset (single record)

**Output:** `dict` with exact keys:
```python
{
    'success': bool,           # REQUIRED - True if rate obtained successfully
    'price': float,            # REQUIRED - Shipping price (before margins/taxes)
    'error_message': str|False, # REQUIRED - Error text if success=False, else False
    'warning_message': str|False, # REQUIRED - Warning text or False
}
```

**Access to order data:**
```python
order.partner_shipping_id        # Delivery address (res.partner)
order.partner_shipping_id.country_id
order.partner_shipping_id.state_id
order.partner_shipping_id.zip
order.warehouse_id.partner_id    # Ship-from address
order.currency_id                # Sale order currency
order.amount_total               # Order total
order.order_line                  # Order lines
order.shipping_weight            # Estimated weight (Float)
```

**Getting packages for rate calculation:**
```python
packages = self._get_packages_from_order(order, self.<provider>_default_package_type_id)
# Returns: list of DeliveryPackage objects
# Each package has: .weight, .dimension, .commodities, .total_cost, .currency_id
```

**Context variable:** `self.env.context.get('order_weight')` may contain an override weight from the wizard.

---

## 2. send_shipping() - The Dispatcher

**Location:** `stock_delivery/models/delivery_carrier.py`

```python
def send_shipping(self, pickings):
    """Dispatcher - DO NOT override this method."""
    return getattr(self, '%s_send_shipping' % self.delivery_type)(pickings)
```

### Provider Method: `<provider>_send_shipping(self, pickings)`

**Input:**
- `self`: `delivery.carrier` recordset (single record)
- `pickings`: `stock.picking` recordset (can be MULTIPLE records)

**Output:** `list` of `dict` (one per picking, same order):
```python
[
    {
        'exact_price': float,         # REQUIRED - Real shipping cost
        'tracking_number': str|False,  # REQUIRED - Tracking reference or False
    },
    # ... one dict per picking
]
```

**CRITICAL:** The caller `send_to_shipper()` processes the FIRST element `[0]` of the returned list. It then:
1. Sets `picking.carrier_price` (after applying margins)
2. Sets `picking.carrier_tracking_ref` (and propagates to related pickings)
3. Posts a message in the chatter
4. Calls `_add_delivery_cost_to_so()` to update the SO delivery line

**Access to picking data:**
```python
picking.partner_id                    # Delivery destination (res.partner)
picking.picking_type_id.warehouse_id  # Source warehouse
picking.picking_type_id.warehouse_id.partner_id  # Ship-from address
picking.company_id                    # Company
picking.sale_id                       # Related sale order (may be False)
picking.sale_id.currency_id           # Currency
picking.sale_id.name                  # Sale order name (for reference)
picking.name                          # Picking name (e.g., WH/OUT/00001)
picking.origin                        # Origin document reference
picking.weight                        # Total weight (computed from moves)
picking.shipping_weight               # Shipping weight override
picking.carrier_tracking_ref          # Existing tracking ref (empty on first send)
picking.move_ids                      # Stock moves
picking.move_line_ids                 # Stock move lines (with quantities)
```

**Getting packages for shipment:**
```python
packages = self._get_packages_from_picking(picking, self.<provider>_default_package_type_id)
# Returns: list of DeliveryPackage objects
```
Only for the **commodities** case (customs / commercial invoice). It raises `UserError` when the
picking weight is 0.0. For count and weight use the `indaws_base_delivery` helpers instead —
authoritative rule in `workflow-steps.md` § "Package Handling".

**Label attachment pattern:**
```python
import base64

label_binary = base64.b64decode(response_label_b64)
# Or if the API returns raw bytes:
label_binary = response.content

attachment = self.env['ir.attachment'].create({
    'name': '%s-%s.pdf' % (self._get_delivery_label_prefix(), tracking_number),
    'type': 'binary',
    'raw': label_binary,
    'res_model': 'stock.picking',
    'res_id': picking.id,
})
picking.message_post(body=msg, attachment_ids=attachment.ids)
# Or with inline attachments:
picking.message_post(body=msg, attachments=[('LabelShipping.pdf', label_binary)])
```

---

## 3. get_tracking_link() - The Dispatcher

**Location:** `stock_delivery/models/delivery_carrier.py`

```python
def get_tracking_link(self, picking):
    """Dispatcher - DO NOT override this method."""
    return getattr(self, '%s_get_tracking_link' % self.delivery_type)(picking)
```

### Provider Method: `<provider>_get_tracking_link(self, picking)`

**Input:**
- `self`: `delivery.carrier` recordset (single record)
- `picking`: `stock.picking` recordset (single record)

**Output:** `str` (URL) or `False`

**Simple pattern (most common):**
```python
def myprovider_get_tracking_link(self, picking):
    return 'https://tracking.myprovider.com/track?ref=%s' % picking.carrier_tracking_ref
```

**Multiple tracking numbers pattern:**
If `carrier_tracking_ref` contains comma-separated values, return JSON for multi-tracker:
```python
import json

def myprovider_get_tracking_link(self, picking):
    refs = picking.carrier_tracking_ref.split(',')
    if len(refs) > 1:
        return json.dumps([
            [ref.strip(), 'https://tracking.myprovider.com/track?ref=%s' % ref.strip()]
            for ref in refs
        ])
    return 'https://tracking.myprovider.com/track?ref=%s' % picking.carrier_tracking_ref
```

**API-based pattern (like Sendcloud):**
```python
def myprovider_get_tracking_link(self, picking):
    if not picking.<provider>_ref:
        return False
    request = <Provider>Request(self)
    result = request.get_tracking_info(picking.<provider>_ref)
    return result.get('tracking_url', False)
```

**Called from:** `stock.picking._compute_carrier_tracking_url()` which is triggered by `@api.depends('carrier_id', 'carrier_tracking_ref')`.

**Also used by:** `stock.picking.open_website_url()` which is the "Track" button action.

---

## 4. cancel_shipment() - The Dispatcher

**Location:** `stock_delivery/models/delivery_carrier.py`

```python
def cancel_shipment(self, pickings):
    """Dispatcher - DO NOT override this method."""
    return getattr(self, '%s_cancel_shipment' % self.delivery_type)(pickings)
```

### Provider Method: `<provider>_cancel_shipment(self, pickings)`

**Input:**
- `self`: `delivery.carrier` recordset (single record)
- `pickings`: `stock.picking` recordset

**Output:** `None` (raise `UserError` on failure)

**Called from:** `stock.picking.cancel_shipment()` which then:
1. Posts a message: "Shipment {tracking_ref} cancelled"
2. Clears `picking.carrier_tracking_ref = False`

```python
def myprovider_cancel_shipment(self, pickings):
    request = <Provider>Request(self)
    for picking in pickings:
        tracking = picking.carrier_tracking_ref
        if not tracking:
            continue
        result = request.cancel_shipment(tracking)
        if not result.get('success'):
            raise UserError(
                _("Failed to cancel shipment %s: %s", tracking, result.get('error'))
            )
```

---

## 5. DeliveryPackage Object

**Location:** `stock_delivery/models/delivery_request_objects.py`

```python
class DeliveryPackage:
    def __init__(self, commodities, weight, package_type, name=None,
                 total_cost=0, currency=None, picking=False, order=False):
        self.picking_id    # stock.picking or False
        self.order_id      # sale.order or False
        self.company_id    # res.company
        self.commodities   # list of DeliveryCommodity
        self.weight        # float (in user's configured weight UoM)
        self.dimension     # {'length': float, 'width': float, 'height': float}
        self.packaging_type # str (shipper_package_code) or False
        self.name          # str or None
        self.total_cost    # float (total declared value)
        self.currency_id   # res.currency
```

## 6. DeliveryCommodity Object

```python
class DeliveryCommodity:
    def __init__(self, product, amount, monetary_value, country_of_origin):
        self.product_id        # product.product
        self.qty               # int (rounded quantity)
        self.monetary_value    # float (unit price in company currency)
        self.country_of_origin # str (country code, e.g., 'ES')
```

---

## 7. send_to_shipper() Flow (for understanding)

**Location:** `stock_delivery/models/stock_picking.py` lines 148-181

This is the orchestrator called when validating a picking. Understanding this flow is critical:

```python
def send_to_shipper(self):
    self.ensure_one()
    # 1. Call your send_shipping
    res = self.carrier_id.send_shipping(self)[0]

    # 2. Apply free shipping if applicable
    if self.carrier_id.free_over and self.sale_id:
        amount = self.sale_id._compute_amount_total_without_delivery()
        if converted_amount >= self.carrier_id.amount:
            res['exact_price'] = 0.0

    # 3. Set carrier_price (with margins)
    self.carrier_price = self.carrier_id._apply_margins(res['exact_price'])

    # 4. Propagate tracking_number to related pickings
    if res['tracking_number']:
        # Finds all pickings in the chain (via move_orig_ids / move_dest_ids)
        # Sets carrier_tracking_ref on pickings that don't have one yet
        # Appends to existing tracking_ref with comma separator

    # 5. Post chatter message
    msg = "Shipment sent to carrier X for shipping with tracking number Y"
    msg += "Cost: Z EUR"
    self.message_post(body=msg)

    # 6. Update SO delivery line if invoice_policy == 'real'
    self._add_delivery_cost_to_so()
```

---

## 8. _send_confirmation_email() Trigger

**Location:** `stock_delivery/models/stock_picking.py` lines 71-100

This is what triggers the entire shipping flow during picking validation:

```python
def _send_confirmation_email(self):
    for pick in self:
        if (pick.carrier_id
            and pick.carrier_id.integration_level == 'rate_and_ship'
            and pick.picking_type_code != 'incoming'
            and not pick.carrier_tracking_ref
            and pick.picking_type_id.print_label):
            pick.sudo().send_to_shipper()
```

**Key conditions for auto-send:**
1. Picking has a carrier assigned
2. Carrier `integration_level` is `rate_and_ship` (not just `rate`)
3. Picking is outgoing (not incoming/receipt)
4. No tracking ref yet (not already shipped)
5. Picking type has `print_label` enabled
