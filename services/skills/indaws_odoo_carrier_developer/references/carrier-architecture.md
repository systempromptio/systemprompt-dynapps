# Carrier Architecture Reference

## Which module owns each method (and why 16.0 is out of scope)

The picking-side contract moved module in **17.0**, when `stock_delivery` was created
(`addons/stock_delivery` exists in 17.0/18.0/19.0, **not in 16.0**). Verified 2026-08-07:

| Method | 16.0 | 17.0 / 18.0 / 19.0 |
|--------|------|--------------------|
| `rate_shipment` | `delivery/models/delivery_carrier.py:180`, `DeliveryCarrier`, v16 — el `def rate_shipment` de esa version | `delivery/…:206` (17), `:252` (18), `:306` (19) |
| `send_shipping` | `delivery/models/delivery_carrier.py:215`, `DeliveryCarrier`, v16 — el `def send_shipping` de esa version | **`stock_delivery/models/delivery_carrier.py:35`** |
| `get_tracking_link` | `delivery/…:241` | `stock_delivery/…` |
| `cancel_shipment` | `delivery/…:251` | `stock_delivery/…` |
| `get_return_label` | `delivery/…:233` | `stock_delivery/…` |

Practical consequence: on 16.0 the manifest depends on `delivery` alone and all four picking-side
methods are overridden on the model that `delivery` brings. On 17.0+ implementing `send_shipping`
**requires `stock_delivery` in `depends`**. Only `rate_shipment` stayed put.

What DOES transfer to 16.0 is the dynamic-dispatch pattern itself:
`getattr(self, '%s_rate_shipment' % self.delivery_type)` is identical in all four versions
(`delivery/models/delivery_carrier.py:191-192`, `DeliveryCarrier` in 16.0, `:321-322` in 19.0). So the architecture below reads fine
against 16.0; the file paths and the `depends` do not.

## Complete Model Map

### Fields on `delivery.carrier` (base module: `delivery`)

```
IDENTITY & CONFIG
├── name                    Char        "Delivery Method" (required, translate)
├── active                  Boolean     default=True
├── sequence                Integer     display order, default=10
├── delivery_type           Selection   'fixed' | 'base_on_rule' | '<provider>'
├── integration_level       Selection   'rate' | 'rate_and_ship'
├── prod_environment        Boolean     test vs production toggle
├── debug_logging           Boolean     log API requests to ir.logging
├── company_id              Many2one    res.company (related from product_id)
├── product_id              Many2one    product.product (REQUIRED, ondelete=restrict)
├── tracking_url            Char        template URL with <shipmenttrackingnumber>
└── currency_id             Many2one    res.currency (related from product_id)

PRICING
├── invoice_policy          Selection   'estimated' | 'real' (real added by stock_delivery)
├── margin                  Float       percentage added to price (>= -1)
├── fixed_margin            Float       fixed amount added to price
├── free_over               Boolean     free if order above threshold
├── amount                  Float       threshold for free shipping
├── fixed_price             Float       (computed/inverse from product_id.list_price)
└── price_rule_ids          One2many    delivery.price.rule (for base_on_rule type)

GEOGRAPHIC RESTRICTIONS
├── country_ids             Many2many   res.country
├── state_ids               Many2many   res.country.state
└── zip_prefix_ids          Many2many   delivery.zip.prefix

PRODUCT RESTRICTIONS
├── max_weight              Float
├── max_volume              Float
├── must_have_tag_ids       Many2many   product.tag
└── excluded_tag_ids        Many2many   product.tag

RETURNS
├── can_generate_return             Boolean  (computed, override per provider)
├── return_label_on_delivery        Boolean  auto-generate return on delivery
├── get_return_label_from_portal    Boolean  customer can download from portal
├── supports_shipping_insurance     Boolean  (computed, override per provider)
└── shipping_insurance              Integer  percentage 0-100

ROUTES (added by stock_delivery)
└── route_ids               Many2many   stock.route (shipping_selectable=True)
```

### Fields on `stock.picking` (added by `stock_delivery`)

```
├── carrier_id              Many2one    delivery.carrier
├── carrier_price           Float       actual shipping cost
├── carrier_tracking_ref    Char        tracking number(s), comma-separated
├── carrier_tracking_url    Char        (computed from carrier_id.get_tracking_link())
├── delivery_type           Selection   (related from carrier_id.delivery_type)
├── weight                  Float       (computed from move weights)
├── weight_uom_name         Char        (computed)
├── is_return_picking       Boolean     (computed: has origin_returned_move_id)
├── return_label_ids        One2many    ir.attachment (computed: return labels)
└── destination_country_code Char       (related from partner_id.country_id.code)
```

### Fields on `sale.order` (added by `delivery`)

```
├── carrier_id              Many2one    delivery.carrier
├── pickup_location_data    Json        pickup point address data
├── delivery_message        Char        carrier messages
├── delivery_set            Boolean     (computed: has delivery line)
├── recompute_delivery_price Boolean    flag for price refresh
├── is_all_service          Boolean     (computed: all products are service)
└── shipping_weight         Float       (computed from order lines)
```

### Fields on `sale.order.line` (added by `delivery`)

```
├── is_delivery             Boolean     marks delivery cost line
├── product_qty             Float       (computed: converted to product UoM)
└── recompute_delivery_price Boolean    (related from order_id)
```

### Fields on `stock.package.type` (added by `stock_delivery`)

```
├── shipper_package_code    Char        carrier-specific code
└── package_carrier_type    Selection   'none' | '<provider>'
```

> **Odoo 19.0 — package model renamed.** The package model is now `stock.package`
> (was `stock.quant.package` ≤18.0). `stock_delivery` inherits `stock.package` and
> adds `weight`, `weight_uom_name`, `package_carrier_type`. Reach the packages of a
> picking with `picking.move_line_ids.result_package_id` (a `stock.package`
> recordset). The `_pre_put_in_pack_hook` also moved from `stock.picking` to
> `stock.package` with a new signature
> (`package_id, package_type_id, package_name, from_package_wizard`). See
> common-pitfalls.md §19–20.

### Fields on `product.template` (added by `stock_delivery`)

```
├── hs_code                 Char        harmonized system code (customs)
└── country_of_origin       Many2one    res.country
```

### Fields on `res.partner` (added by `delivery`)

```
└── property_delivery_carrier_id  Many2one  delivery.carrier (company_dependent)
```

---

## Module Dependency Graph

```
                    sale
                     │
                 delivery              ← Base carrier model + SO integration
                     │
    sale_stock ──────┤
                     │
              stock_delivery           ← auto_install, picking integration
                     │
           ┌─────────┼─────────┐
           │         │         │
    delivery_    delivery_   delivery_   ← Enterprise carrier implementations
    fedex_rest   sendcloud   dhl_rest
```

---

## The Availability Filtering Chain

When a user selects a carrier in the SO wizard, this filtering chain runs:

```python
# 1. Domain filter: company match
carriers = self.env['delivery.carrier'].search(
    self.env['delivery.carrier']._check_company_domain(company)
)

# 2. Partner/order match via available_carriers()
available = carriers.available_carriers(partner, order)

# 3. Each carrier checks _match() which chains:
def _match(self, partner, order):
    return (
        self._match_address(partner)        # country, state, zip prefix
        and self._match_must_have_tags(order) # product tags inclusion
        and self._match_excluded_tags(order)  # product tags exclusion
        and self._match_weight(order)         # total weight <= max_weight
        and self._match_volume(order)         # total volume <= max_volume
    )

# 4. For 'base_on_rule' type, also validates against price rules
if self.delivery_type == 'base_on_rule':
    return self.rate_shipment(order).get('success')
```

---

## Price Calculation Pipeline

```
Provider returns raw price
        │
        ▼
Fiscal position adjustment (taxes)
  product_id._get_tax_included_unit_price()
        │
        ▼
Margin application
  price * (1.0 + margin%) + fixed_margin
        │
        ▼
Free shipping check
  if free_over and order_total >= amount:
      price = 0.0
        │
        ▼
Final price returned to wizard/caller
```

---

## Existing Provider Implementations Summary

### Provider Field Patterns

Each provider adds configuration fields to `delivery.carrier`. The naming convention is:
`<delivery_type>_<field_name>`

| Provider | delivery_type | Credential Fields | Config Fields |
|----------|--------------|-------------------|---------------|
| FedEx REST | `fedex_rest` | `fedex_rest_developer_key`, `fedex_rest_developer_password`, `fedex_rest_account_number`, `fedex_rest_access_token` | `fedex_rest_service_type`, `fedex_rest_droppoff_type`, `fedex_rest_weight_unit`, `fedex_rest_label_stock_type`, `fedex_rest_label_file_type`, `fedex_rest_duty_payment`, `fedex_rest_documentation_type`, `fedex_rest_residential_address` |
| Sendcloud | `sendcloud` | `sendcloud_public_key`, `sendcloud_secret_key` | `sendcloud_shipping_id` (M2o), `sendcloud_return_id` (M2o), `sendcloud_shipping_rules`, `sendcloud_product_functionalities`, `sendcloud_use_batch_shipping` |
| DHL REST | `dhl_rest` | `dhl_rest_api_key`, `dhl_rest_account_number` | `dhl_rest_service_type`, `dhl_rest_package_type`, `dhl_rest_weight_unit` |
| UPS REST | `ups_rest` | `ups_rest_client_id`, `ups_rest_client_secret`, `ups_rest_account_number`, `ups_rest_access_token` | `ups_rest_service_type`, `ups_rest_package_type`, `ups_rest_weight_unit`, `ups_rest_label_file_type` |

### Service Class Patterns

| Provider | Class Name | Instantiation |
|----------|-----------|---------------|
| FedEx REST | `FedexRequest` | `FedexRequest(self)` (receives carrier recordset) |
| Sendcloud | `SendCloud` | `SendCloud(public_key, secret_key, log_xml)` |
| DHL REST | `DHLProvider` | `DHLProvider(self)` |
| UPS REST | `UPSRequest` | `UPSRequest(self)` |
| Easypost | `EasypostRequest` | `EasypostRequest(self)` |
| bpost | `BpostRequest` | `BpostRequest(self)` |

### Extra Models Created by Providers

| Provider | Model | Purpose |
|----------|-------|---------|
| Sendcloud | `sendcloud.shipping.product` | Stores shipping products fetched from Sendcloud API |
| Sendcloud | `sendcloud.shipping.wizard` | Wizard to choose shipping products |
| Sendcloud | `stock.picking` (fields) | `sendcloud_parcel_ref` (Json), `sendcloud_return_parcel_ref` (Json) |
| Starshipit | `stock.picking` (fields) | `starshipit_parcel_reference` (Char), `starshipit_return_parcel_reference` (Char) |

### Cron Jobs

| Provider | Cron | Purpose | Frequency |
|----------|------|---------|-----------|
| Starshipit | `_cron_starshipit_fetch_and_update_prices` | Poll real shipping prices | Every 1 hour |
| (No other provider has cron jobs) | | | |

---

## Tracking URL Patterns by Provider

| Provider | Pattern | Type |
|----------|---------|------|
| FedEx REST | `https://www.fedex.com/wtrk/track/?trknbr={ref}` | Static URL construction |
| DHL REST | `https://www.dhl.com/en/express/tracking.html?AWB={ref}` | Static URL construction |
| UPS REST | `https://www.ups.com/track?tracknum={ref}` | Static URL construction |
| Sendcloud | API call to `GET /parcels/{id}` → extract `tracking_url` | Dynamic API call |
| Easypost | Returns `tracking_url` from API response | Stored from API |
| Fixed/Rule | `tracking_url` field with `<shipmenttrackingnumber>` placeholder | Template replacement |

---

## What Odoo Does NOT Handle (gaps for custom development)

1. **No delivery status tracking** - No `tracking_state` or similar field
2. **No status polling** - No cron to check if package was delivered
3. **No webhook receivers** - No endpoint to receive carrier push notifications
4. **No delivery date tracking** - No `delivered_at` or `estimated_delivery` fields
5. **No multi-carrier per picking** - One carrier per picking only
6. **No rate comparison view** - Must use wizard one-carrier-at-a-time
7. **No automatic carrier selection** - No rules engine to auto-pick cheapest

### `indaws_delivery_state_tracking` — Fills gaps 1 and 2

This custom Indaws module addresses the first two gaps:
- **`delivery.carrier.state`** model: carrier-specific states with generic mapping (pending, in_transit, delivered, incident, returned, cancelled, etc.)
- **`delivery.tracking.history`** model: audit trail of all state changes
- **`picking.delivery_state_id`**: Many2one to current carrier state
- **`picking.delivery_generic_state`**: Selection related field for cross-carrier filtering
- **`picking._set_delivery_state(code, note, date)`**: Main method to update state
- **`picking._set_delivery_state_by_generic(generic_state, note, date)`**: Fallback by generic state
- **`cancel_shipment()` override**: Auto-sets 'cancelled' generic state

When creating carrier modules that depend on this module, use `indaws_delivery_state_tracking` instead of `stock_delivery` as dependency (it already depends on stock_delivery).
