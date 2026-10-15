# Provider Implementation Patterns

## Pattern 1: Simple REST API (Most Common)

Used by: FedEx REST, DHL REST, UPS REST, Shiprocket, Envia, **Correos Express (OCA)**.

> Don't write this from scratch. **Copy the closest real provider** (see
> `migration-v18-to-v19.md` §2 for the catalog) and adapt. The full method
> signatures/return shapes are in `api-contracts.md`. Below is only the shape +
> the non-obvious bits.

### Structure
```
delivery_<provider>/
├── models/
│   ├── delivery_carrier.py      # _inherit delivery.carrier  → the 4 methods
│   └── <provider>_request.py    # plain Python HTTP class (NOT an Odoo model)
```

### Request class (`<provider>_request.py`) — the shape
A plain class instantiated as `<Provider>Request(self, self.log_xml)`. It holds
auth, payload building and one central `_send_request()`. Non-obvious points:

- `__init__`: pick `BASE_URL_PROD`/`BASE_URL_TEST` from `carrier.prod_environment`;
  read credentials with `carrier.sudo().<field>` (they are `groups="base.group_system"`).
- `_send_request(endpoint, method, data, params)`: **wrap in try/except** for
  `requests.exceptions.Timeout` and `ConnectionError` → `UserError`; set
  `timeout=30`; `self.carrier.log_xml(...)` before and after; on
  `status_code >= 400` call a `_handle_error()` that parses the carrier's error body.
- API methods return plain dicts the carrier maps to Odoo (`get_rate` →
  `{'price'}`, `create_shipment` → `{'tracking_number', 'label_data', 'price'}`,
  `cancel_shipment` → `{'success'}`).
- `_prepare_address(partner)` / `_prepare_package(package)`: convert `res.partner`
  and `DeliveryPackage` to the API payload. Weight via
  `carrier._<provider>_convert_weight(package.weight)`.

> Auth, per-endpoint error handling, response-casing and SOAP variants are in the
> sections below (real Spanish-carrier patterns).

### Carrier model (`delivery_carrier.py`) — the shape
```python
class DeliveryCarrier(models.Model):
    _inherit = 'delivery.carrier'

    delivery_type = fields.Selection(
        selection_add=[('<provider>', '<Name>')],
        ondelete={'<provider>': lambda recs: recs.write(
            {'delivery_type': 'fixed', 'fixed_price': 0})})
    # credentials → groups="base.group_system"; service_type/label_format/weight_unit configs

    # Optionally override _compute_can_generate_return / _compute_supports_shipping_insurance
    # Helper: _<provider>_convert_weight(weight)  (UoM → carrier unit)

    # The 4 dispatch methods (names are the contract — NO leading underscore):
    def <provider>_rate_shipment(self, order):     # → {'success','price','error_message','warning_message'}
    def <provider>_send_shipping(self, pickings):  # → list[{'exact_price','tracking_number'}]; labels via picking.message_post(attachments=[(name, bytes)])
    def <provider>_get_tracking_link(self, picking):  # → str URL (or json.dumps(list))
    def <provider>_cancel_shipment(self, pickings):   # cancel + raise UserError on failure
```

Full bodies + return contracts: `api-contracts.md`. Working examples to copy:
EE `delivery_shiprocket`/`delivery_easypost`, OCA `delivery_correos_express`.

## Pattern 2: Provider with Extra Models

Used by: Sendcloud (has `sendcloud.shipping.product` model)

When the carrier API has entities that need to be stored locally (shipping products, service lists, etc.):

```python
# models/my_provider_service.py
class MyProviderService(models.Model):
    _name = 'my_provider.service'
    _description = 'My Provider Shipping Service'

    name = fields.Char(required=True, readonly=True)
    code = fields.Char(required=True, readonly=True)
    carrier_name = fields.Char(readonly=True)
    max_weight = fields.Float(readonly=True)
    # ...
```

The carrier then has a Many2one to this model instead of a simple Selection for service type.

**Remember:** If you create new models, you MUST add `security/ir.model.access.csv`.

---

## Pattern 3: Provider with Cron Job

Used by: Starshipit (price polling)

When you need periodic updates from the carrier API:

```xml
<!-- data/ir_cron_data.xml -->
<odoo>
    <record id="ir_cron_<provider>_update" model="ir.cron">
        <field name="name"><Provider>: Fetch Updates</field>
        <field name="model_id" ref="stock.model_stock_picking"/>
        <field name="state">code</field>
        <field name="code">model._cron_<provider>_fetch_updates()</field>
        <field name="interval_number">1</field>
        <field name="interval_type">hours</field>
        <field name="active">True</field>
    </record>
</odoo>
```

```python
# In stock_picking.py extension:
class StockPicking(models.Model):
    _inherit = 'stock.picking'

    def _cron_<provider>_fetch_updates(self):
        pickings = self.search([
            ('carrier_id.delivery_type', '=', '<provider>'),
            ('carrier_tracking_ref', '!=', False),
            # ... additional filters
        ])
        for picking in pickings:
            # Call API, update fields
            pass
```

---

## Pattern 4: Provider with Webhook

Not natively implemented in Odoo 18, but if you need real-time status updates:

```python
# controllers/main.py
from odoo import http
from odoo.http import request

class <Provider>Controller(http.Controller):

    # csrf=False aqui es decorativo: JsonRPCDispatcher no lee csrf (solo lo hace HttpDispatcher,
    # odoo18/odoo/http.py:2282). Si el transportista postea form-data en vez de JSON-RPC,
    # la ruta tiene que ser type='http' y AHI si es obligatorio.
    @http.route('/<provider>/webhook', type='json', auth='public', methods=['POST'], csrf=False)
    def webhook(self, **kwargs):
        data = request.get_json_data()
        # Verify webhook signature/token
        # Find picking by tracking ref
        # Update status
        return {'status': 'ok'}
```

**Remember:** Add `'controllers/__init__.py'` import and add controller to manifest if needed.

---

## Pattern 5: Multi-Package Shipment

When a picking has multiple packages (via put-in-pack):

```python
def <provider>_send_shipping(self, pickings):
    res = []
    for picking in pickings:
        # commodities case only; see workflow-steps.md § "Package Handling" for the rule
        packages = self._get_packages_from_picking(picking, self.<provider>_default_package_type_id)

        all_tracking = []
        total_price = 0.0

        for package in packages:
            result = request.create_shipment_package(
                ship_from=...,
                ship_to=...,
                package=package,
            )
            all_tracking.append(result['tracking_number'])
            total_price += result.get('price', 0.0)

            # Attach individual label
            # ...

        res.append({
            'exact_price': total_price,
            'tracking_number': ','.join(all_tracking),
        })
    return res
```

---

## Authentication Patterns

Real-world examples from Spanish carrier implementations:

### HTTP Basic Auth (Correos Express)

Simplest pattern. No token caching needed:
```python
class CorreosExpressRequest:
    def __init__(self, carrier):
        self.carrier = carrier

    def _get_auth(self):
        return (self.carrier.cex_username, self.carrier.cex_password)

    def _send_request(self, url, data):
        res = requests.post(
            url=url, auth=self._get_auth(), json=data, timeout=60
        )
```

### OAuth2 Bearer with Token Cache (SEUR ATLAS)

Token cached on carrier record with expiry field. Check before every request:
```python
# Fields on delivery.carrier:
seur_token = fields.Char(copy=False)
seur_token_expiry = fields.Datetime(copy=False)

# In request class:
def _get_token(self):
    now = fields.Datetime.now()
    # Reuse cached token if not expired (60s safety margin)
    if (self.carrier.seur_token
            and self.carrier.seur_token_expiry
            and self.carrier.seur_token_expiry > now + timedelta(seconds=60)):
        return self.carrier.seur_token
    # Request new token
    response = requests.post(self.token_url, data={
        'grant_type': 'password',  # or 'client_credentials'
        'client_id': self.carrier.sudo().seur_client_id,
        'client_secret': self.carrier.sudo().seur_client_secret,
        'username': self.carrier.sudo().seur_username,
        'password': self.carrier.sudo().seur_password,
    })
    token_data = response.json()
    expiry = now + timedelta(seconds=token_data['expires_in'])
    self.carrier.sudo().write({
        'seur_token': token_data['access_token'],
        'seur_token_expiry': expiry,
    })
    return token_data['access_token']
```

### Daily Token — calendar expiry, not a TTL (CBL)

Some carriers issue a token that is valid **for the calendar day**, with no `expires_in`. Two
traps, both silent:

1. **Do not fake a TTL.** There is no countdown to cache; key the cache by **date**. Store the
   token plus a `Date` field and reuse it only while that date equals today.
2. **Today according to WHOM.** Use the **provider's timezone**, not UTC and not the user's. With
   UTC, between 22:00 UTC and midnight (Spain in DST) you are still on "yesterday" while the
   carrier has already rolled over — you keep sending a dead token and every call fails, once a
   day, for two hours. Pin the carrier's zone as a module constant.

```python
# Fields on delivery.carrier:
<provider>_daily_token = fields.Char(copy=False, groups="base.group_system")
<provider>_daily_token_date = fields.Date(copy=False, groups="base.group_system")

# In the request class:
PROVIDER_TIMEZONE = "Europe/Madrid"   # the CARRIER's zone, not the server's

@staticmethod
def _<provider>_today():
    """Return today's date as the carrier sees it."""
    return datetime.now(pytz.timezone(PROVIDER_TIMEZONE)).date()

def _get_daily_token(self, force=False):
    carrier = self.carrier.sudo()
    if (not force and carrier.<provider>_daily_token
            and carrier.<provider>_daily_token_date == self._<provider>_today()):
        return carrier.<provider>_daily_token
    ...  # fetch, then store token + date
```

Combine with the 401 retry below (`force=True` invalidates the cache and retries **once**).
Reference: `indaws_delivery_cbl/models/cbl_request.py::_get_daily_token` + `_cbl_today`.

### API Key (Header)
```python
self.session.headers['Authorization'] = 'Bearer %s' % carrier.sudo().<provider>_api_key
```

### Session Token + Login Endpoint (Enlaza GTS)

Similar to OAuth2 but simpler: login with user/password, get a token + expiry. Token sent as Bearer.
Includes **auth retry** — if any endpoint returns auth error, invalidate token and retry once.

```python
# Fields on delivery.carrier:
<provider>_username = fields.Char(groups="base.group_system")
<provider>_password = fields.Char(groups="base.group_system")
<provider>_token = fields.Char(groups="base.group_system", copy=False)
<provider>_token_expiry = fields.Datetime(groups="base.group_system", copy=False)

# In request class:
def _get_token(self):
    carrier = self.carrier
    if carrier.<provider>_token and carrier.<provider>_token_expiry:
        if fields.Datetime.now() < carrier.<provider>_token_expiry:
            return carrier.<provider>_token
    # Login
    resp = self.session.post(self.base_url + '/login', json={
        'usuario': carrier.<provider>_username,
        'password': carrier.<provider>_password,
    }, timeout=self.timeout)
    data = resp.json()
    token = data.get('token') or data.get('access_token')
    expires_in = data.get('expires_in', 1800)
    expiry = fields.Datetime.now() + timedelta(seconds=max(expires_in - 60, 60))
    carrier.write({'<provider>_token': token, '<provider>_token_expiry': expiry})
    return token

def _invalidate_token(self):
    self.carrier.write({'<provider>_token': False, '<provider>_token_expiry': False})

def _send_request(self, path, data, retry_auth=True):
    token = self._get_token()
    headers = {'Authorization': 'Bearer %s' % token, 'Content-Type': 'application/json'}
    resp = self.session.post(self.base_url + path, json=data, headers=headers, timeout=self.timeout)
    result = resp.json()
    # Retry once on auth expiry
    if retry_auth and self._is_auth_error(result):
        self._invalidate_token()
        return self._send_request(path, data, retry_auth=False)
    return result

def _is_auth_error(self, result):
    """Detect auth expiry in any response format (dict or list)."""
    if isinstance(result, dict):
        code = result.get('codigo_error') or result.get('CODIGO_ERROR')
        if str(code) == '1':
            return True
    if isinstance(result, list):
        return any(str(item.get('codigo_error', '')) == '1' for item in result)
    return False
```

### Auth Pattern Summary Table

| Type | Example | Cache | Retry | Complexity |
|------|---------|-------|-------|------------|
| Embedded in payload | GLS (UID in XML) | No | No | Low |
| HTTP Basic Auth | Correos Express | No | No | Low |
| API Key (Header) | Generic REST | No | No | Low |
| OAuth2 (password grant) | SEUR ATLAS | Yes (carrier fields) | No | Medium |
| Session token + login | Enlaza GTS | Yes (carrier fields) | Yes (on error code) | Medium |
| Daily token (calendar) | CBL | Yes (keyed by **date**, carrier TZ) | Yes (401 → force) | Medium |
| SOAP with credentials | GLS ASM | No | No | High |

---

### SOAP with raw requests (GLS ASM)

For legacy SOAP APIs, avoid zeep/suds — use `requests` with XML templates:
```python
import xml.etree.ElementTree as ET
from xml.sax.saxutils import escape

def _build_soap_envelope(self, method, params):
    body = ''.join(
        '<v1:{k}>{v}</v1:{k}>'.format(k=k, v=escape(str(v)))
        for k, v in params.items()
    )
    return (
        '<?xml version="1.0" encoding="UTF-8"?>'
        '<soapenv:Envelope xmlns:soapenv="...">'
        '<soapenv:Body><v1:{method}>{body}</v1:{method}>'
        '</soapenv:Body></soapenv:Envelope>'
    ).format(method=method, body=body)

def _send_soap(self, method, params):
    xml = self._build_soap_envelope(method, params)
    res = requests.post(self.url, data=xml, headers={
        'Content-Type': 'text/xml; charset=utf-8',
        'SOAPAction': method,
    }, timeout=60)
    return ET.fromstring(res.content)
```

---

## Standard Helper Patterns

These helpers appear in virtually every carrier implementation:

### Phone Formatting
```python
@staticmethod
def _<prefix>_format_phone(phone):
    """Format phone: +→00, remove spaces/dashes, max 15 chars."""
    if not phone:
        return ""
    return phone.replace("+", "00").replace(" ", "").replace("-", "")[:15]
```

### Address Sanitization (with unidecode)
```python
from unidecode import unidecode

@staticmethod
def _<prefix>_get_streets(partner):
    """Join street + street2, remove accents."""
    parts = []
    if partner.street:
        parts.append(unidecode(partner.street))
    if partner.street2:
        parts.append(unidecode(partner.street2))
    return " ".join(parts)
```

### National vs International Detection
```python
sender = picking.picking_type_id.warehouse_id.partner_id
national = partner.country_id.code == sender.country_id.code
# Use national CP field or international CP field accordingly
```

---

## Error Handling: Per-Endpoint Patterns

Carriers often use **different error formats per endpoint**. Handle each separately:

```python
# Correos Express example — 3 different error formats:
def _check_shipment_error(self, result):
    code = result.get("codigoRetorno")  # Number, 0=OK
    if code and code != 0:
        raise UserError(...)

def _check_label_error(self, result):
    code = result.get("codErr")  # Number, 0=OK
    if code and code != 0:
        raise UserError(...)

def _check_tracking_error(self, result):
    code = result.get("error")  # Number, 0=OK
    if code and code != 0:
        _logger.warning(...)  # Don't raise, just log
```

**Rule:** Tracking errors should **log warnings**, not raise UserError (avoid blocking cron jobs).

---

## JSON Payload Patterns

### Flat Structure (Correos Express)
All fields at root level:
```python
payload = {
    "codRte": "...", "nomRte": "...", "dirRte": "...",
    "codDest": "...", "nomDest": "...", "dirDest": "...",
    "numBultos": "1", "kilos": "1.000", "producto": "63",
}
```

### Nested Structure (SEUR ATLAS)
Grouped by entity:
```python
payload = {
    "sender": {
        "name": "...",
        "address": {"streetName": "...", "cityName": "...", "postalCode": "..."},
    },
    "receiver": {
        "name": "...",
        "address": {"streetName": "...", "cityName": "...", "postalCode": "..."},
    },
    "parcels": [{"weight": 1.0, "height": 10, "length": 20, "width": 15}],
}
```

---

## Response Key Casing Inconsistency

Some carriers use **different casing per endpoint**:
- Endpoint A: `resultado`, `numero_envio`, `codigo_error` (lowercase)
- Endpoint B: `RESULTADO`, `ETIQUETA`, `CODIGO_ERROR` (UPPERCASE)
- Endpoint C: Mix of both

**Rule:** Each request class method must parse its own response format. Never assume uniform casing.

```python
# Good: per-method parsing
def create_shipment(self, vals):
    result = self._send_request(self.SHIPMENT_PATH, {"DOCUMENTAR_ENVIO": [vals]})
    if isinstance(result, list):
        result = result[0]
    if result.get("resultado") == "ERROR":  # lowercase
        raise UserError(result.get("mensaje", ""))
    return {"tracking": result.get("numero_envio", "")}

def get_label(self, client, center, expedition, fmt):
    result = self._send_request(self.LABEL_PATH, {"ETIQUETA": [...]})
    if isinstance(result, list):
        result = result[0]
    if result.get("RESULTADO") == "ERROR":  # UPPERCASE
        raise UserError(result.get("MENSAJE", ""))
    return {"label": result.get("ETIQUETA", "")}
```

---

## Multi-Reference Shipments

Some carriers return **multiple identifiers** per shipment, each with a different purpose:

| Reference | Odoo Field | Purpose |
|-----------|-----------|---------|
| Primary (expedition/tracking) | `carrier_tracking_ref` (standard) | Main tracking reference |
| Secondary (barcode) | `<provider>_public_tracking_ref` | Public tracking URL |
| Tertiary (admission code) | `<provider>_admission_code` | Label reprint, API queries |
| Locator | `<provider>_locator` | Customer-facing locator |
| Pickup code | `<provider>_pickup_code` | Associated pickup |

**Pattern:** Store the primary in `carrier_tracking_ref`, create provider-specific fields for the rest:
```python
# In stock_picking.py:
class StockPicking(models.Model):
    _inherit = 'stock.picking'
    <provider>_admission_code = fields.Char(copy=False, readonly=True)
    <provider>_locator = fields.Char(copy=False, readonly=True)
    <provider>_pickup_code = fields.Char(copy=False, readonly=True)
```

**In send_shipping:**
```python
picking.carrier_tracking_ref = response['tracking_number']  # primary
picking.<provider>_admission_code = response.get('admission_code', '')
picking.<provider>_locator = response.get('locator', '')
```

---

## API Version Parameters

Some carriers offer versioned endpoints where higher versions return more detail:

```python
# Always use the highest available version
payload = {
    "VERSION": 6,  # v6 = all fields; v1 = basic only
    "DETALLE_EXPEDICION": [{
        "CLIENTE": client,
        "CENTRO": center,
        "EXPE_NUMERO": expedition_number,
    }],
}
```

**Gotcha:** Fields that only exist in higher versions (e.g., GPS coordinates, amounts, package details) will be missing in v1 responses. Always check the API docs for which version includes which fields.

---

## Error Codes at Scale

For carriers with many endpoints and error codes (e.g., Enlaza: ~220 codes across 8 endpoints), organize error dicts per endpoint with a cascading lookup helper:

```python
# In <provider>_master_data.py:

SHIPMENT_VALIDATION_ERRORS = {
    "1": "Authentication required",
    "2": "Shipment already exists",
    # ... validation codes (1-99)
}

SHIPMENT_BUSINESS_ERRORS = {
    "1001": "Incorrect origin client",
    "1003": "Recipient name required",
    # ... business codes (1000+)
}

CANCEL_ERROR_CODES = {
    "1": "Authentication required",
    "3": "Expedition number required",
    # ... per-endpoint codes
}

LABEL_ERROR_CODES = { ... }
TRACKING_ERROR_CODES = { ... }
PICKUP_ERROR_CODES = { ... }
MANIFEST_ERROR_CODES = { ... }
POD_ERROR_CODES = { ... }
IMAGE_ERROR_CODES = { ... }


def get_shipment_error(code):
    """Cascading lookup across validation + business dicts."""
    code_str = str(code)
    return (
        SHIPMENT_VALIDATION_ERRORS.get(code_str)
        or SHIPMENT_BUSINESS_ERRORS.get(code_str)
        or "Unknown error (code %s)" % code_str
    )
```

---

## Extended Features (Beyond 4 Mandatory Methods)

For carriers with >4 endpoints, organize `delivery_carrier.py` by section:

```python
# delivery_carrier.py layout:
# 1. Rate (mandatory) — <provider>_rate_shipment()
# 2. Send (mandatory) — <provider>_send_shipping()
# 3. Tracking (mandatory) — <provider>_get_tracking_link(), <provider>_tracking_state_update()
# 4. Cancel (mandatory) — <provider>_cancel_shipment()
# 5. Labels (optional) — <provider>_get_label()
# 6. PODs (optional) — <provider>_get_pod()
# 7. Pickups (optional) — <provider>_create_pickup()
# 8. Manifests (optional) — <provider>_get_manifest()
# 9. Images (optional) — <provider>_upload_image()
# 10. Joint Ops (optional) — <provider>_joint_delivery()
```

### POD (Proof of Delivery) Download

```python
def <provider>_get_pod(self, picking):
    """Download proof of delivery and attach to picking."""
    if not picking.carrier_tracking_ref:
        raise UserError(_("No tracking reference on this picking."))
    request = self._get_<provider>()
    pods = request.download_pod(picking.carrier_tracking_ref)
    if not pods:
        raise UserError(_("No proof of delivery available."))
    for pod in pods:
        self.env['ir.attachment'].create({
            'name': pod.get('name', 'pod_%s.pdf' % picking.carrier_tracking_ref),
            'datas': base64.b64encode(pod['bytes']),
            'res_model': 'stock.picking',
            'res_id': picking.id,
            'mimetype': 'application/pdf',
        })
    picking.message_post(body=_("POD downloaded (%d file(s)).", len(pods)))
```

**Button on stock.picking:**
```xml
<button name="<provider>_download_pod" string="Download POD" type="object"
        icon="fa-file-pdf-o"
        invisible="delivery_type != '<provider>' or not carrier_tracking_ref
                   or delivery_generic_state != 'delivered'"/>
```

### Pickup Creation

```python
def <provider>_create_pickup(self, picking, pickup_date=None):
    """Create a pickup request with the carrier."""
    request = self._get_<provider>()
    sender = picking.picking_type_id.warehouse_id.partner_id or picking.company_id.partner_id
    vals = {
        'CLIENT': self.<provider>_client_code,
        'CENTER': self.<provider>_center_code,
        'NAME': sender.name,
        'ADDRESS': self._<provider>_format_address(sender)[:60],
        'CITY': sender.city,
        'ZIP': sender.zip,
        'COUNTRY': sender.country_id.code or 'ES',
        'DATE': pickup_date or fields.Date.context_today(self).strftime('%d/%m/%Y'),
        'PACKAGES': picking.number_of_packages or 1,
    }
    result = request.create_pickup(vals)
    if result.get('pickup_code'):
        picking.<provider>_pickup_code = result['pickup_code']
        picking.message_post(body=_("Pickup created: %s", result['pickup_code']))
    return result.get('pickup_code', '')
```

### Manifest Generation

```python
def <provider>_get_manifest(self, pickup_code):
    """Download manifest PDF for a pickup."""
    request = self._get_<provider>()
    pdf_bytes = request.get_manifest(
        self.<provider>_client_code, self.<provider>_center_code, pickup_code
    )
    if not pdf_bytes:
        raise UserError(_("Could not generate manifest."))
    return pdf_bytes  # Caller creates attachment or returns to wizard
```

### Image Upload

```python
def <provider>_upload_image(self, picking, image_data, image_format='jpg'):
    """Upload an image (e.g., delivery photo) to the carrier for this shipment."""
    if not picking.carrier_tracking_ref:
        raise UserError(_("No tracking reference on this picking."))
    request = self._get_<provider>()
    request.upload_image(
        entity='EXPEDITION',
        number=picking.carrier_tracking_ref,
        images=[{'FORMAT': image_format.upper(), 'BYTES': image_data}],
    )
    picking.message_post(body=_("Image uploaded to %s.", self.name))
```

### stock.picking Buttons for Extended Features

```xml
<!-- After cancel_shipment button -->
<button name="<provider>_get_label" string="Reprint Label" type="object"
        class="oe_highlight" icon="fa-print"
        invisible="delivery_type != '<provider>' or not carrier_tracking_ref"/>
<button name="<provider>_update_tracking" string="Update Tracking" type="object"
        icon="fa-refresh"
        invisible="delivery_type != '<provider>' or not carrier_tracking_ref"/>
<button name="<provider>_download_pod" string="Download POD" type="object"
        icon="fa-file-pdf-o"
        invisible="delivery_type != '<provider>' or not carrier_tracking_ref
                   or delivery_generic_state != 'delivered'"/>
<button name="<provider>_request_pickup" string="Request Pickup" type="object"
        icon="fa-truck"
        invisible="delivery_type != '<provider>' or not carrier_tracking_ref
                   or <provider>_pickup_code"/>
```
