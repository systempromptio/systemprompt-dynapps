# Carrier Module Scaffold Checklist

Use this checklist when creating a new carrier module. Replace `<provider>` with the actual provider key.

**Scope: DOMAIN files only.** `__manifest__.py`, the `__init__.py` files, `security/` (ACL +
ir.rules) and the test scaffold are emitted and checked by `odoo-module-creator` — see its own
checklist. Verifying them again here duplicated the base and drifted from it; the only carrier-
specific things to confirm about them are `depends` (`['stock_delivery','mail']`, plus
`indaws_delivery_state_tracking` if used) and that every `data/`/`views/` file below is listed in
the manifest's `data`.

## Files to Create (in order)

### 4. `delivery_<provider>/models/<provider>_request.py`
- [ ] Class is plain Python (NOT `models.Model`)
- [ ] Constructor receives carrier record (sudoed) or credentials
- [ ] Has `_send_request()` central method with debug logging
- [ ] Uses `carrier.log_xml()` for request/response logging
- [ ] Has proper timeout on all requests (recommended: 30-90 seconds)
- [ ] Error handling raises `UserError` with clear messages
- [ ] Supports prod/test URL switching via `carrier.prod_environment` or configurable URL
- [ ] API methods: `create_shipment()`, `cancel_shipment()`, `get_tracking()` minimum
- [ ] If token-based auth: `_get_token()`, `_invalidate_token()`, `_is_auth_error()` methods
- [ ] If token-based auth: auth retry in `_send_request()` (retry once on auth error)
- [ ] Extended methods if needed: `get_label()`, `download_pod()`, `create_pickup()`, `get_manifest()`, `upload_image()`

### 5. `delivery_<provider>/models/delivery_<provider>.py`
- [ ] `_inherit = 'delivery.carrier'`
- [ ] `delivery_type` Selection extended with `selection_add`
- [ ] `ondelete` defined for the new selection value
- [ ] Credential fields have `groups="base.group_system"`
- [ ] All field names prefixed with `<provider>_`
- [ ] `_compute_can_generate_return()` overridden (call super first!)
- [ ] `_compute_supports_shipping_insurance()` overridden (call super first!)
- [ ] `<provider>_rate_shipment()` implemented with correct return dict
- [ ] `<provider>_send_shipping()` implemented with correct return list
- [ ] `<provider>_get_tracking_link()` implemented
- [ ] `<provider>_cancel_shipment()` implemented
- [ ] Phone formatting helper: `+`→`00`, strip spaces/dashes, max 15 chars
- [ ] Address helper with `unidecode` if carrier requires ASCII

### 5b. State Tracking Integration (if using `indaws_delivery_state_tracking`)
- [ ] `TRACKING_STATES` dict defined (code → generic_state + name)
- [ ] `_<prefix>_ensure_tracking_states()` creates `delivery.carrier.state` records
- [ ] `create()` override calls `_ensure_tracking_states()` for new carriers
- [ ] `write()` override calls `_ensure_tracking_states()` when delivery_type changes
- [ ] `send_shipping()` calls `picking._set_delivery_state()` after success
- [ ] `tracking_state_update()` method fetches API status and calls `_set_delivery_state()`
- [ ] If no cancel API: post message + let parent handle state
- [ ] If no rate API: return `product_id.lst_price` with warning message

### 5c. Extended Features (if carrier provides >4 endpoints)
- [ ] POD download: `<provider>_get_pod()` on carrier + button on picking
- [ ] Pickup creation: `<provider>_create_pickup()` on carrier + button on picking
- [ ] Manifest: `<provider>_get_manifest()` on carrier
- [ ] Image upload: `<provider>_upload_image()` on carrier
- [ ] Joint delivery: `<provider>_joint_delivery()` on carrier (if API supports batch ops)
- [ ] Extra picking fields for secondary references (`<provider>_admission_code`, `<provider>_locator`, `<provider>_pickup_code`)
- [ ] Buttons on stock.picking form with correct visibility rules (`delivery_type`, `carrier_tracking_ref`, `delivery_generic_state`)
- [ ] `views/stock_picking_views.xml` added to manifest `data` list

### 5d. Authentication (choose pattern, see references/provider-patterns.md)
- [ ] Pattern chosen: Basic Auth / API Key / OAuth2 / Session Token / SOAP embedded
- [ ] If token-cached: `<provider>_token` + `<provider>_token_expiry` fields (`groups="base.group_system"`, `copy=False`)
- [ ] If token-cached: auth retry logic in `_send_request()` with `retry_auth` parameter
- [ ] If token-cached: `_invalidate_token()` clears both token fields

### 6. `delivery_<provider>/models/stock_package_type.py` (optional)
- [ ] `_inherit = 'stock.package.type'`
- [ ] `package_carrier_type` Selection extended with `selection_add`

### 7. `delivery_<provider>/models/stock_picking.py` (optional)
- [ ] `_inherit = 'stock.picking'`
- [ ] Extra fields have `copy=False`
- [ ] No business logic here (keep in carrier model)

### 8. `delivery_<provider>/data/delivery_<provider>.xml`
- [ ] Default product.product for the carrier (type=service, sale_ok=False)
- [ ] Product category ref: `delivery.product_category_deliveries`
- [ ] Package types if applicable

### 9. `delivery_<provider>/views/delivery_<provider>.xml`
- [ ] Inherits `delivery.view_delivery_carrier_form` (that is the real id — see "Quick Reference: View Inheritance")
- [ ] Own `<page>` inserted with `xpath expr="//page[@name='destination']" position="before"`
- [ ] `invisible="delivery_type != '<provider>'"` on the page, not repeated per field
- [ ] Credentials in separate group with clear label
- [ ] Configuration in separate group

---

## Validation Checklist

The full domain validation is `references/workflow-steps.md` STEP 4 (contracts, `delivery_type`
consistency, security, version-specific) and the 42 entries of `references/common-pitfalls.md`.
Restating them here is how the two lists drifted apart. What follows is only the tick-list you
run at the end; when one fails, the reason is in those two files.

- [ ] The 4 return signatures (STEP 4.2) — dispatcher contracts, verbatim
- [ ] `delivery_type` key == method prefix == `package_carrier_type` value; `ondelete` defined
- [ ] Credentials `groups="base.group_system"`, never logged in clear (STEP 4.3)
- [ ] Robustness: timeout 30-90 s · per-endpoint error handling and key casing (#15-17) ·
      auth retry once (#30) · tracking errors log, never raise (they run in cron) ·
      warnings surfaced, not swallowed (#39) · multi-reference responses stored (#14)
- [ ] Labels: single vs double base64 (#12) · ZPL as text, one file per package, no preview (#42)
- [ ] Dimensions converted from mm (#40); weight via the base helpers (#31)
- [ ] Version-specific constructs correct for the target (pitfalls #18-26)
- [ ] **STEP 5 executed**: suite installed and run, real numbers reported

---

## Quick Reference: View Inheritance

The parent view id is `delivery.view_delivery_carrier_form` (v17 `:34`, v18 `:38` `@99f5a33c64`,
v19 `:38` `@d1d161033`). There is **no** `delivery.delivery_carrier_view_form` and **no**
`carrier_credential` group — neither has ever existed; referencing either makes the module fail
to install with `ParseError`. Insert your own `<page>` before `page[@name='destination']`
(present in all three versions: v17 `:137`, v18 `:147`, v19 `:148`).

Snippet and rationale: `references/workflow-steps.md` §"6. `views/delivery_<provider>.xml`".
