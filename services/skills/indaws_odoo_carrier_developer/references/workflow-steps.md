# Carrier Creator Workflow Steps

Complete step-by-step workflow for creating Odoo delivery carrier modules.

---

## STEP 0: Domain Gathering (CRITICAL)

**NEVER proceed without collecting ALL required PROVIDER information.**

> **Split of responsibilities:** target version, technical name, destination/source paths,
> multi-company and brand profile are gathered by **`odoo-module-creator`** in STEP 1 (the
> base) — do **NOT** ask them here. STEP 0 collects **provider/domain info only**.

Use `AskUserQuestion` tool to gather:

### Required Information (Priority 1)

1. **Shipping Provider Name:**
   - Company name (e.g., MRW, SEUR, Correos Express, GLS, Nacex)
   - `delivery_type` technical key (e.g., `mrw`, `seur`, `correos_express`)

3. **Provider API Documentation:**
   - Does the user have API documentation? (URL or file)
   - Authentication method: API key, OAuth2, Basic Auth, Token?
   - API type: REST JSON, REST XML, SOAP?
   - Sandbox/test environment available?

4. **Features Required:**
   - Rate calculation (quote prices)?
   - Shipment creation (send packages)?
   - Label generation (PDF, ZPL, PNG)?
   - Tracking URL?
   - Shipment cancellation?
   - Return labels?
   - Pickup locations?
   - Multiple packages per shipment?
   - Insurance support?
   - **Status/tracking: is there ANY?** Ask all three explicitly — status-query endpoint?
     webhook/push? public tracking portal? "None of the three" is a real and common answer
     (CBL), and it changes the whole design: `get_tracking_link` returns `False` deliberately,
     there is no state-mapping cron, and the module seeds only the states it causes itself
     (see STEP 3 § "Integration with `indaws_delivery_state_tracking`"). Do NOT assume the
     four-method contract implies status retrieval.

5. **Provider-Specific Configuration:**
   - What credentials are needed? (API key, account number, user/password, etc.)
   - Service types available? (express, standard, economy, etc.)
   - Package types/sizes?
   - Weight units? (kg, lb)
   - Special fields? (contract number, client code, etc.)

6. **Approach (domain framing only):**
   - "I have the API docs and a clear plan" -> proceed to STEP 1 scaffold + STEP 2 fill
   - "I need help analyzing the API" -> STEP 0.5 Plan Mode (provider API analysis)

> Destination path, Odoo source/enterprise paths, target version and multi-company are
> asked by `odoo-module-creator` in STEP 1 — do NOT ask them here. Reference-provider
> paths for reading examples (e.g. `delivery_fedex_rest`) are resolved in STEP 1 via
> `references/shared/odoo-source.md`.

**STOP HERE** - Wait for user's response before continuing.

---

## STEP 0.5: Plan Mode (OPTIONAL - If API Analysis Needed)

**Trigger:** User selected Option B or needs help designing the integration.

**Action:** Use `EnterPlanMode` tool.

In Plan Mode:

1. **Analyze Provider API:**
   - Read API documentation (if provided as files)
   - Identify endpoints for: rate, ship, track, cancel
   - Map API authentication flow
   - Identify request/response formats
   - Note error handling patterns

2. **Map to Odoo Interface:**
   - Map rate endpoint -> `<provider>_rate_shipment()`
   - Map ship endpoint -> `<provider>_send_shipping()`
   - Map tracking endpoint -> `<provider>_get_tracking_link()`
   - Map cancel endpoint -> `<provider>_cancel_shipment()`
   - Identify which API fields map to Odoo fields

3. **Design Configuration Fields:**
   - Credentials fields (with `groups="base.group_system"`)
   - Service type selection
   - Package type selection
   - Environment toggle (test/production)

4. **Design the domain layer** (NOT the module skeleton — that's `odoo-module-creator`):
   - Define carrier config fields (credentials, service/package types, env toggle)
   - Plan the `<provider>_request.py` service-class methods
   - Design error handling / label / tracking strategy

5. **Exit Plan Mode** with the domain technical specification (the skeleton spec is
   produced by `odoo-module-creator` in STEP 1).

---

## STEP 1: Scaffold base (delegate) + analyze references

1. **Analyze reference code** (resolve Odoo source via `references/shared/odoo-source.md`):
   - Read `delivery/models/delivery_carrier.py` to understand base fields
   - Read `stock_delivery/models/delivery_carrier.py` to understand dispatch pattern
   - Read `stock_delivery/models/stock_picking.py` to understand `send_to_shipper()` flow
   - Read an existing provider (e.g., `delivery_fedex_rest`) as implementation reference
   - Read `stock_delivery/models/delivery_request_objects.py` for DeliveryPackage/DeliveryCommodity

2. **Delegate the skeleton → `odoo-module-creator`** (BASE). Invoke it (skill/subagent) with:
   - **Module name:** `delivery_<provider>` (e.g., `delivery_mrw`)
   - **Dependencies:** `['stock_delivery', 'mail']` minimum (+ `indaws_delivery_state_tracking` if used)
   - **Target version + brand profile** (module-creator gathers/confirms these in its STEP 0)
   - **Domain model list** so it emits ACL + ir.rules: `delivery.carrier` (`_inherit`),
     `<provider>_request` (pure Python, NOT a model → no ACL), and any of
     `stock.package.type` / `stock.picking` you will extend.
   - It produces: manifest, `__init__`, security (ACL + ir.rules), multi-company, README,
     icon, provenance, test scaffold — and runs module-scanner + version-api-reference.
   - **Its STEP 1 already STOPs for user confirmation** — do not add a second confirmation gate.

3. **Delegation is conditional, never mandatory — there is an inline path.** Some configurations
   forbid subagent fan-out; a step marked "mandatory subagent" then simply does not run and nobody
   finds out. (Observed while building CBL: a scaffold subagent silently chose not to launch the
   two subagents `odoo-module-creator` requires, verified what it needed by hand against
   the real Odoo source, and got it right — invisibly. That could as easily have gone the other way.)
   Prefer loading `odoo-module-creator` as a skill in this context, or a subagent. If you can do
   neither, do it yourself and **say so in the summary**:
   - Read `odoo-module-creator`'s SKILL.md and its scaffold templates, and follow them literally
     (manifest, `__init__`, ACL + ir.rules, multi-company, README, icon, provenance, test scaffold).
   - Instead of `odoo-module-scanner`: `Glob`/`Grep` the sibling carrier module you are cloning and
     list its models, fields and view ids by hand.
   - Instead of `odoo-version-api-reference`: verify every version-specific construct you emit
     against the real Odoo source (resolve it via `references/shared/odoo-source.md` —
     never assume a path) and cite `file:line`.
   - State which generic gates were NOT run, so nobody reads silence as green.

4. Once the skeleton is confirmed and generated, continue to STEP 2 to fill the domain.
   Present the **domain plan** (config fields, request-class methods, label/error strategy)
   as part of module-creator's confirmation so the user sees the whole picture.

---

## STEP 2: Fill domain code (after scaffold)

> `odoo-module-creator` already generated the **skeleton**: `__manifest__.py` (with
> `depends: ['stock_delivery','mail']` and the `data` list), `__init__.py` files,
> `security/ir.model.access.csv` + ir.rules, README, icon, provenance and tests. Do
> **NOT** regenerate them. If a `data`/`views` file you add is missing from the manifest's
> `data` list, ask module-creator to add it (or add the single line) — don't rewrite the manifest.

Generate the **domain files** in this order:

### 1. `models/<provider>_request.py` - Service class (Python pure, NOT an Odoo model)

This class encapsulates ALL HTTP communication with the carrier API. Pattern:

```python
import requests
from odoo import _
from odoo.exceptions import UserError

class <Provider>Request:
    def __init__(self, carrier):
        self.carrier = carrier
        # Extract credentials
        # Setup session/auth

    def _send_request(self, endpoint, method='GET', data=None):
        """Central HTTP request handler with logging."""
        # Build URL, headers, auth
        # Make request
        # Log if debug_logging enabled: carrier.log_xml(...)
        # Handle errors
        # Return parsed response

    def get_shipping_rate(self, ship_from, ship_to, packages, currency):
        """Call rate/quote API endpoint."""

    def create_shipment(self, ship_from, ship_to, packages, **kwargs):
        """Call shipment creation API endpoint.
        Returns: tracking_number, labels, price"""

    def cancel_shipment(self, tracking_ref):
        """Call cancellation API endpoint."""

    def get_label(self, tracking_ref):
        """Download shipping label (if separate endpoint)."""
```

### 2. `models/delivery_<provider>.py` - Carrier model extension

```python
class Delivery<Provider>(models.Model):
    _inherit = 'delivery.carrier'

    # 1. Extend delivery_type selection
    delivery_type = fields.Selection(
        selection_add=[('<provider>', '<Provider Name>')],
        ondelete={'<provider>': lambda recs: recs.write({
            'delivery_type': 'fixed', 'fixed_price': 0
        })}
    )

    # 2. Configuration fields (credentials with groups="base.group_system")
    <provider>_api_key = fields.Char(string="API Key", groups="base.group_system")
    <provider>_account = fields.Char(string="Account Number", groups="base.group_system")
    <provider>_service_type = fields.Selection([...], string="Service Type")
    <provider>_default_package_type_id = fields.Many2one('stock.package.type')
    <provider>_weight_unit = fields.Selection([('KG', 'Kilograms'), ('LB', 'Pounds')])
    <provider>_label_format = fields.Selection([('PDF', 'PDF'), ('ZPL', 'ZPL')])

    # 3. Override computed fields if needed
    def _compute_can_generate_return(self):
        super()._compute_can_generate_return()
        self.filtered(lambda c: c.delivery_type == '<provider>').can_generate_return = True

    def _compute_supports_shipping_insurance(self):
        super()._compute_supports_shipping_insurance()
        self.filtered(lambda c: c.delivery_type == '<provider>').supports_shipping_insurance = True

    # 4. The 4 mandatory methods
    def <provider>_rate_shipment(self, order): ...
    def <provider>_send_shipping(self, pickings): ...
    def <provider>_get_tracking_link(self, picking): ...
    def <provider>_cancel_shipment(self, picking): ...
```

### 3. `models/stock_package_type.py` (if provider has specific package types)

```python
class PackageType(models.Model):
    _inherit = 'stock.package.type'
    package_carrier_type = fields.Selection(
        selection_add=[('<provider>', '<Provider Name>')]
    )
```

### 4. `models/stock_picking.py` (if provider needs extra fields on picking)

Only if the provider requires storing extra data (like Sendcloud's `sendcloud_parcel_ref`):

```python
class StockPicking(models.Model):
    _inherit = 'stock.picking'
    <provider>_shipment_ref = fields.Char("Provider Reference", copy=False)
```

### 5. `data/delivery_<provider>.xml` - Demo/default data

Pre-configured package types and default product:

```xml
<?xml version="1.0" encoding="utf-8"?>
<odoo>
    <record id="product_product_delivery_<provider>" model="product.product">
        <field name="name"><Provider> Shipping</field>
        <field name="type">service</field>
        <field name="list_price">0</field>
        <field name="sale_ok">False</field>
        <field name="purchase_ok">False</field>
        <field name="categ_id" ref="delivery.product_category_deliveries"/>
    </record>

    <!-- Package types if applicable -->
    <record id="<provider>_package_standard" model="stock.package.type">
        <field name="name"><Provider> Standard Box</field>
        <field name="package_carrier_type"><provider></field>
        <field name="shipper_package_code">STANDARD</field>
    </record>
</odoo>
```

### 6. `views/delivery_<provider>.xml` - Form view extension

```xml
<?xml version="1.0" encoding="utf-8"?>
<odoo>
    <record id="delivery_<provider>_form_view" model="ir.ui.view">
        <field name="name">delivery.carrier.form.<provider></field>
        <field name="model">delivery.carrier</field>
        <field name="inherit_id" ref="delivery.view_delivery_carrier_form"/>
        <field name="arch" type="xml">
            <xpath expr="//page[@name='destination']" position="before">
                <page string="<Provider> Configuration"
                      name="<provider>_config"
                      invisible="delivery_type != '<provider>'">
                    <group>
                        <group string="Credentials">
                            <field name="<provider>_api_key" password="True"/>
                            <field name="<provider>_account"/>
                        </group>
                        <group string="Configuration">
                            <field name="<provider>_service_type"/>
                            <field name="<provider>_default_package_type_id"/>
                            <field name="<provider>_weight_unit"/>
                            <field name="<provider>_label_format"/>
                        </group>
                    </group>
                </page>
            </xpath>
        </field>
    </record>
</odoo>
```

**Anchor, verified.** The parent view record is `delivery.view_delivery_carrier_form` — there is
no `delivery.delivery_carrier_view_form`, that id has never existed. The only stable insertion
point is the `destination` page, present in the three supported versions:
`delivery/views/delivery_carrier_views.xml:137`, `view_delivery_carrier_form` (v17), `:147` (v18 `@99f5a33c64`), `:148`
(v19 `@d1d161033`). Do **not** target a `carrier_credential` group: no such group exists in the
base view either. Own `<page>` before `destination`, one `invisible` on the page (not on every
field). Live reference: `indaws_delivery_cbl/views/delivery_carrier_views.xml`.

### Security (delegated)

`security/ir.model.access.csv` and ir.rules are generated by `odoo-module-creator` from the
domain model list you passed in STEP 1. If you add a **new** model here that wasn't in that
list, tell module-creator to emit its ACL + ir.rule — do not hand-write security ad hoc.

---

## STEP 3: Implementation Details

### Integration with `indaws_delivery_state_tracking` (CRITICAL)

When the module depends on `indaws_delivery_state_tracking`, implement these 3 patterns:

> **First decide WHICH states you can populate.** This section is written for carriers that
> expose status codes. A carrier with **no status API at all** (no query endpoint, no webhook,
> no public portal — CBL is one) still needs the state rows: seed the states **you yourself
> cause** — registered / confirmed / cancelled — and write them from your own code paths, rather
> than mapping a provider code space that does not exist. Do not skip the rows: without them
> `_set_delivery_state` finds no target, returns `False`, and every state write is silently
> dropped — the exact failure mode of `indaws_delivery_ontime`. Reference:
> `indaws_delivery_cbl/models/cbl_master_data.py::CBL_TRACKING_STATES` +
> `delivery_carrier.py::_cbl_ensure_tracking_states`.

**a) Tracking states dictionary** -- Define the mapping from carrier codes to generic states:
```python
# If <=20 codes: put in <provider>_request.py
# If >50 codes: create <provider>_master_data.py
TRACKING_STATES = {
    "code": ("generic_state", "Human Name"),
    # generic_state is one of: pending, picked_up, in_transit,
    # out_for_delivery, delivered, incident, returned, cancelled
}
```

**b) Auto-create carrier states** on create/write:
```python
@api.model_create_multi
def create(self, vals_list):
    carriers = super().create(vals_list)
    for carrier in carriers:
        if carrier.delivery_type == '<provider>':
            carrier._<prefix>_ensure_tracking_states()
    return carriers

def write(self, vals):
    res = super().write(vals)
    if vals.get("delivery_type") == "<provider>":
        for carrier in self:
            carrier._<prefix>_ensure_tracking_states()
    return res

def _<prefix>_ensure_tracking_states(self):
    """Create delivery.carrier.state records for this carrier."""
    self.ensure_one()
    State = self.env["delivery.carrier.state"]
    existing_codes = set(
        State.search([("carrier_id", "=", self.id)]).mapped("code")
    )
    seq = 10
    for code, (generic, name) in TRACKING_STATES.items():
        if code not in existing_codes:
            State.create({
                "carrier_id": self.id,
                "code": code,
                "name": name,
                "generic_state": generic,
                "sequence": seq,
            })
        seq += 10
```

**c) Set state after shipping and tracking** -- Call `_set_delivery_state()`:
```python
# In send_shipping, after successful creation:
picking._set_delivery_state("pending_code", note=_("Shipment registered"))

# In tracking_state_update:
picking._set_delivery_state(str(code), note="[%s] %s" % (code, desc))
```

**d) Cancel: parent handles the state** -- `indaws_delivery_state_tracking.cancel_shipment()` automatically sets state to 'cancelled'. The provider only needs to call the carrier cancel API (if it exists) or post a message.

### Missing API Fallbacks

Some carriers lack rate or cancel APIs. Use these patterns:

**No rate API** (e.g., Correos Express):
```python
def <provider>_rate_shipment(self, order):
    return {
        "success": True,
        "price": self.product_id.lst_price,
        "error_message": _(
            "<Provider> does not provide a rate API. "
            "Using the carrier product price as estimate."
        ),
        "warning_message": _(
            "<Provider> does not provide a rate API. "
            "Using the carrier product price as estimate."
        ),
    }
```

**No cancel API** (e.g., Correos Express):
```python
def <provider>_cancel_shipment(self, pickings):
    for picking in pickings.filtered("carrier_tracking_ref"):
        picking.message_post(body=_(
            "<Provider> does not provide an API to cancel "
            "shipments. Create a new shipment if needed."
        ))
    # Parent cancel_shipment() handles state -> 'cancelled'
```

**Cancel that does not cancel** (third case — the one usually missed). Between "there is an API"
and "there is none" sits: *the API accepts the call and only **flags** the shipment; a human at
the carrier has to finish it.* Typically it depends on shipment state — pending shipments really
are deleted, confirmed ones are only marked. Treat it as a **different outcome, not a success**:
the operator must learn from the picking itself that a phone call is still pending, so the
distinction has to reach the **chatter**, not just a log line.

```python
def <provider>_cancel_shipment(self, pickings):
    api = self._<provider>_request()
    for picking in pickings.filtered("<provider>_client_reference"):
        ref = picking.<provider>_client_reference
        if picking.<provider>_confirmed:
            api.delete_confirmed_shipments(ref)      # flags only
            picking.message_post(body=_(
                "Shipment %s flagged as deleted in <Provider>.\n"
                "It was already confirmed, so <Provider> only marks it: contact "
                "your delegation to actually cancel it.", ref))
        else:
            api.delete_pending_shipments(ref)        # really deleted
            picking.message_post(body=_("Pending shipment %s deleted in <Provider>.", ref))
```

Reference: `indaws_delivery_cbl/models/delivery_carrier.py::cbl_cancel_shipment`.

### Label Handling Pattern

Labels can come in different ways depending on the carrier:

| Pattern | Example Carrier | Description |
|---------|----------------|-------------|
| **Inline in creation response** | Correos Express, GLS | Label returned in the shipment creation response |
| **Separate endpoint** | SEUR ATLAS | Must call a second endpoint to fetch labels |
| **Single base64** | SEUR, GLS | Standard `base64.b64decode(data)` |
| **Double base64** | Correos Express | `base64.b64decode(base64.b64decode(data))` |

Attach labels to the picking via `message_post` with `attachments` parameter:

```python
def <provider>_send_shipping(self, pickings):
    res = []
    for picking in pickings:
        request = <Provider>Request(self)
        response = request.create_shipment(...)

        # Attach label(s) - adapt decoding to carrier
        attachments = []
        for idx, label_b64 in enumerate(response.get('labels', [])):
            if is_pdf:
                # Single decode (most carriers):
                raw = base64.b64decode(label_b64)
                # Double decode (e.g., Correos Express):
                # raw = base64.b64decode(base64.b64decode(label_b64))
                attachments.append(("label_%s_%d.pdf" % (tracking, idx + 1), raw))
            else:
                # ZPL: text content
                attachments.append(("label_%s_%d.txt" % (tracking, idx + 1), label_b64.encode("utf-8")))

        picking.message_post(
            body=_("Shipment created with %s. Tracking: %s", self.name, tracking),
            attachments=attachments,
        )

        res.append({
            'exact_price': response.get('price', 0.0),
            'tracking_number': tracking,
        })
    return res
```

**Label reprint button** -- Add a button on stock.picking for reprinting labels:
```xml
<button name="<provider>_get_label" string="Reprint Label"
        type="object"
        invisible="delivery_type != '<provider>' or not carrier_tracking_ref"/>
```

### Package Handling

**Which source wins (this overrides any other reading of this file).** Count and weight ALWAYS
come from the `indaws_base_delivery` helpers (`picking._get_delivery_packages_count()` /
`picking._get_delivery_weight()`): they are the ones that respect the manual overrides.
`_get_packages_from_picking` only if you need **commodities** (customs / commercial invoice), and
never when the weight can be 0: it raises `UserError`
(`stock_delivery/models/delivery_carrier.py:200-204`, `DeliveryCarrier` @d1d161033`) — which is exactly the case the
base's manual weight override exists to cover. If you need per-package detail but not commodities,
walk `picking.move_line_ids.result_package_id` by hand.

Reference implementation: `indaws_delivery_cbl/models/delivery_carrier.py::_cbl_packages`.

The built-in package system from `stock_delivery`, for the commodities case:

```python
def <provider>_send_shipping(self, pickings):
    for picking in pickings:
        packages = self._get_packages_from_picking(
            picking, self.<provider>_default_package_type_id
        )
        for package in packages:
            # package.weight       -> float (user UoM)
            # package.dimension    -> {'length': x, 'width': y, 'height': z}
            # package.commodities  -> [DeliveryCommodity, ...]
            # package.total_cost   -> float (declared value)
            # package.currency_id  -> res.currency
            # package.name         -> str or None
```

### Weight Conversion Pattern

```python
def _<provider>_convert_weight(self, weight):
    """Convert from Odoo UoM to carrier UoM."""
    if self.<provider>_weight_unit == 'KG':
        target = self.env.ref('uom.product_uom_kgm')
    else:
        target = self.env.ref('uom.product_uom_lb')
    source = self.env['product.template']._get_weight_uom_id_from_ir_config_parameter()
    return source._compute_quantity(weight, target, round=False)
```

### Length Conversion Pattern

Dimensions need the same treatment as weight, and they are **more dangerous**, because the error
is silent. Odoo's default length UoM for `stock.package.type` (`length`/`width`/`height`) is the
**millimetre** — `product/models/product_template.py::_get_length_uom_id_from_ir_config_parameter`,
v18 `:359 @99f5a33c64`, v19 `:367 @d1d161033`. A carrier that expects metres and receives the raw
value gets a number 1000× too large: no exception, no traceback, just wrong data (a provider
warning at best). Never pass `package_type.width` straight to the API.

```python
def _<provider>_to_meters(self, value):
    """Convert a length from Odoo's configured length UoM into metres."""
    if not value:
        return 0.0
    source = self.env['product.template']._get_length_uom_id_from_ir_config_parameter()
    target = self.env.ref('uom.product_uom_meter')
    return source._compute_quantity(value, target, round=False)
```

Reference: `indaws_delivery_cbl/models/delivery_carrier.py::_cbl_to_meters`.

### Error Handling Pattern

```python
def <provider>_send_shipping(self, pickings):
    request = <Provider>Request(self)
    res = []
    for picking in pickings:
        try:
            response = request.create_shipment(...)
        except UserError:
            raise
        except Exception as e:
            raise UserError(
                _("Error sending shipment to %s:\n%s", self.name, str(e))
            )
        # ... process response
    return res
```

### Debug Logging Pattern

Use the built-in `log_xml()` from `delivery.carrier`:

```python
class <Provider>Request:
    def __init__(self, carrier):
        self.logger = carrier.log_xml

    def _send_request(self, endpoint, method='GET', data=None):
        self.logger(
            '%s %s\n%s' % (method, url, json.dumps(data, indent=2)),
            '<provider> request %s' % endpoint
        )
        response = self.session.request(method=method, url=url, json=data)
        self.logger(
            '%s %s' % (response.status_code, response.text),
            '<provider> response %s' % endpoint
        )
```

### Auth Retry Pattern (for token/session-based carriers)

When using cached tokens, the session can expire **during** an operation. Add retry logic to `_send_request()`:

```python
def _send_request(self, endpoint, data, retry_auth=True):
    token = self._get_token()
    headers = {'Authorization': 'Bearer %s' % token}
    response = self.session.post(url, json=data, headers=headers, timeout=90)
    result = response.json()

    # Retry once on auth expiry (carrier returns error code for expired session)
    if retry_auth and self._is_auth_error(result):
        self._invalidate_token()
        return self._send_request(endpoint, data, retry_auth=False)

    return result
```

See [provider-patterns.md](provider-patterns.md) for the complete pattern with `_is_auth_error()` and `_invalidate_token()`.

### Extended Features Pattern (POD, Pickup, Manifest, Images)

For carriers with >4 endpoints, organize `delivery_carrier.py` by section. See [provider-patterns.md](provider-patterns.md) "Extended Features" section for templates covering:
- POD (Proof of Delivery) download + attachment
- Pickup creation
- Manifest generation
- Image upload
- Joint delivery/pickup operations

---

## STEP 4: Domain Validation

> Generic gates — manifest correctness, `__init__` completeness, multi-company ir.rules,
> anti-laxity self-audit and unit + E2E flow tests — are run by `odoo-module-creator`. This
> step validates only the **carrier-specific domain**.

1. **Wiring check (domain-relevant only):**
   - `data/`/`views/` files you added are listed in the manifest's `data` (else ask module-creator)
   - `data/` XML files reference correct external IDs
   - Views inherit correct parent views (`delivery.view_delivery_carrier_form`)

2. **Interface contract validation:**
   - `<provider>_rate_shipment()` returns `{'success', 'price', 'error_message', 'warning_message'}`
   - `<provider>_send_shipping()` returns `[{'exact_price', 'tracking_number'}]`
   - `<provider>_get_tracking_link()` returns `str` or `False`
   - `<provider>_cancel_shipment()` returns `None` or raises `UserError`

3. **Security validation (domain-specific):**
   - API credentials use `groups="base.group_system"`
   - No credentials in logs (use `****` masking)
   - (ACL/ir.rules coverage for models is validated by `odoo-module-creator`, not here)

4. **delivery_type consistency:**
   - `delivery_type` selection value matches ALL method prefixes
   - `ondelete` handler defined for the selection value
   - `package_carrier_type` selection value matches (if extending stock.package.type)

5. **Version-specific validation:**
   - Odoo 19.0: `<list>` not `<tree>`, no `<group>` in search views
   - Odoo 18.0+: `_compute_display_name` not `name_get()`, `aggregator` not `group_operator`
   - All versions: Check XML `invisible` attribute syntax per version
   - **Odoo 19.0 delivery/stock_delivery** (see common-pitfalls.md §19–26):
     `stock.quant.package` → `stock.package`; `_pre_put_in_pack_hook` moved to
     `stock.package` (new signature); `_apply_margins(price, order=False)`;
     `available_carriers`/`_match*` take `source` not `order`; new
     `_compute_allowed_carrier_ids` on `stock.picking`; delivery now ships
     payment_provider/payment_transaction/ir_http.

6. **Present summary** of all created files to user.

---

## STEP 5: Install and run the suite in a throwaway container

**Not optional, and not expensive (~5 min). A module whose suite has never been executed is not
green: it is unexecuted.** STEP 4 validates contracts by *reading*. Reading does not find a
fixture in a state the code under test discards, a mocked response desynchronised by a token
cache, or a cron whose time bound the ORM re-stamps under it.

Evidence, not opinion (CBL, 2026-08-09): first run of the suite was **2 failures and 2 errors out
of 85**, and fixing them exposed **three more tests that were passing for the wrong reason** — the
fixture picking was in `draft`, the cron excludes `draft`, so the three cron tests never exercised
the cron. One real code bug (`_check_status` blowing up on a bare JSON array) was found by
*writing* the tests; another (the cron's time bound) only by *running* them.

Recipe, re-executed end to end on 2026-08-09 against `indaws_delivery_cbl` on v19 —
**85 tests, 0 failed, 0 errors, exit 0**. It failed the first time for the reason in trap 2b
below; the version here is the one that actually ran:

```bash
docker network create <net>
docker run -d --name <db> --network <net> \
  -e POSTGRES_USER=odoo -e POSTGRES_PASSWORD=odoo -e POSTGRES_DB=postgres postgres:16-alpine

docker run --rm --network <net> \
  -v $ODOO_SRC/odoo<N>:/home/odoo/odoo:ro \
  -v <module-repo>:/src/carriers:ro \
  -v <deps-symlink-dir>:/src/extra:ro \
  --entrypoint python3 <image> \
  /home/odoo/odoo/odoo-bin -d <db-name> --db_host=<db> --db_user=odoo --db_password=odoo \
  --addons-path=/home/odoo/odoo/addons,/src/extra,/src/carriers \
  -i <module> --test-enable --test-tags /<module> \
  --stop-after-init --workers=0 --max-cron-threads=0
```

Four details that cost real time to discover:

1. **The park's images carry no Odoo source.** `indaws/odoo:19-dai-test` and `test18:1.0.1` bring
   the Python dependencies but **not** Odoo itself — mount your Odoo source (`$ODOO_SRC/odoo<N>`;
   resolve the base via `references/shared/odoo-source.md`, do not assume it). Bypass their
   entrypoint with `--entrypoint python3`.
2. **Do NOT mount `/opt/PROYECTS/indawsodoo/<N>` whole**: it holds **75 modules with
   `auto_install: True`** that install themselves and contaminate the run. Mount it on a path
   *outside* the `addons-path` and build a directory of **symlinks to just the modules you need**
   — the symlinks must point at the **in-container** path (`/src/indawsodoo/<mod>`), which is
   where they resolve.
2b. **Symlink the TRANSITIVE closure, not the direct `depends`.** This is what broke the first
   run: CBL depends on `indaws_base_delivery` + `indaws_delivery_state_tracking`, but
   `indaws_base_delivery` depends on `indaws_base`, which was not linked — Odoo aborts with
   `You try to install module "X" that depends on module "Y". But the latter module is not
   available in your system.` Resolve the closure before starting:

   ```bash
   python3 - <<'EOF'
   import ast, os
   ROOT="/opt/PROYECTS/indawsodoo/<N>"; MOD="<path-to-your-module>"
   def deps(p):
       m=os.path.join(p,"__manifest__.py")
       return ast.literal_eval(open(m).read()[open(m).read().index("{"):]).get("depends",[]) \
              if os.path.exists(m) else []
   seen, queue = set(), list(deps(MOD))
   while queue:
       d = queue.pop(0)
       if d in seen or not os.path.isdir(os.path.join(ROOT, d)): continue
       seen.add(d); queue += deps(os.path.join(ROOT, d)); print(d)
   EOF
   ```
   Note that some of those closure members are themselves `auto_install: True` (`indaws_base`,
   `indaws_base_delivery`). That is fine and is not a contradiction of trap 2: linking them
   explicitly installs exactly them, whereas mounting the whole tree would pull in all 75.
3. **Own names for network and containers — never an existing compose project.** Recorded scar:
   reusing the `docker19` project name takes down the corporate v19 instance.
4. **Delete the containers and the network when done.**

Report the real numbers (`N passed / M failed`). "Tests written" is not "tests green".
