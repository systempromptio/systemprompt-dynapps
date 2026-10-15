# Outbound Clients — Patrón Clase Service

## Estructura de ficheros

```
my_module/
├── services/
│   ├── __init__.py
│   └── my_api_service.py   # Clase service (NO es un modelo Odoo)
├── models/
│   └── my_model.py         # Modelo que llama al service
```

## Clase Service Base

```python
import logging
import time
import hmac
import hashlib
from requests import Session
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

_logger = logging.getLogger(__name__)

DEFAULT_TIMEOUT = (5, 30)      # (connect_timeout, read_timeout) segundos
MAX_RETRIES = 3
BACKOFF_FACTOR = 0.5           # espera: 0.5, 1.0, 2.0 s


class MyApiService:
    """Client for MyAPI. Instantiate per-call or reuse within a transaction."""

    BASE_URL_PROD = "https://api.example.com/v1"
    BASE_URL_TEST = "https://sandbox.example.com/v1"

    def __init__(self, env, prod=True):
        self.env = env
        self.base_url = self.BASE_URL_PROD if prod else self.BASE_URL_TEST
        self._session = self._build_session()
        self._token = None

    # ------------------------------------------------------------------
    # Session setup
    # ------------------------------------------------------------------
    def _build_session(self):
        session = Session()
        retry = Retry(
            total=MAX_RETRIES,
            backoff_factor=BACKOFF_FACTOR,
            status_forcelist=[429, 500, 502, 503, 504],
            allowed_methods=["GET", "POST", "PUT"],
            raise_on_status=False,
        )
        adapter = HTTPAdapter(max_retries=retry)
        session.mount("https://", adapter)
        session.mount("http://", adapter)
        return session

    # ------------------------------------------------------------------
    # Auth helpers
    # ------------------------------------------------------------------
    def _get_token(self):
        """Fetch/refresh bearer token. Override for OAuth2 flows."""
        if self._token:
            return self._token
        api_key = (
            self.env["ir.config_parameter"]
            .sudo()
            .get_param("my_module.api_key", default="")
        )
        if not api_key:
            raise ValueError("my_module.api_key not configured in System Parameters")
        self._token = api_key
        return self._token

    def _headers(self):
        return {
            "Authorization": f"Bearer {self._get_token()}",
            "Content-Type": "application/json",
            "Accept": "application/json",
        }

    # ------------------------------------------------------------------
    # Low-level request
    # ------------------------------------------------------------------
    def _request(self, method, endpoint, **kwargs):
        url = f"{self.base_url}/{endpoint.lstrip('/')}"
        kwargs.setdefault("timeout", DEFAULT_TIMEOUT)
        kwargs.setdefault("headers", self._headers())
        try:
            response = self._session.request(method, url, **kwargs)
            response.raise_for_status()
            return response.json() if response.content else {}
        except Exception as exc:
            _logger.error(
                "MyAPI %s %s failed: %s", method.upper(), url, exc, exc_info=True
            )
            raise

    # ------------------------------------------------------------------
    # Public methods
    # ------------------------------------------------------------------
    def get_order(self, external_id):
        return self._request("GET", f"orders/{external_id}")

    def post_order(self, payload):
        return self._request("POST", "orders", json=payload)
```

## Auth: variantes comunes

### API Key en header
```python
def _headers(self):
    key = self.env["ir.config_parameter"].sudo().get_param("my_module.api_key")
    return {"X-Api-Key": key, "Content-Type": "application/json"}
```

### Basic Auth
```python
from requests.auth import HTTPBasicAuth

def _request(self, method, endpoint, **kwargs):
    user = self.env["ir.config_parameter"].sudo().get_param("my_module.user")
    pwd  = self.env["ir.config_parameter"].sudo().get_param("my_module.password")
    kwargs["auth"] = HTTPBasicAuth(user, pwd)
    ...
```

### OAuth2 Client Credentials
```python
def _get_token(self):
    """Fetch token with client_credentials grant."""
    client_id     = self.env["ir.config_parameter"].sudo().get_param("my_module.client_id")
    client_secret = self.env["ir.config_parameter"].sudo().get_param("my_module.client_secret")
    token_url     = self.env["ir.config_parameter"].sudo().get_param("my_module.token_url")
    resp = self._session.post(
        token_url,
        data={"grant_type": "client_credentials", "scope": "read write"},
        auth=(client_id, client_secret),
        timeout=DEFAULT_TIMEOUT,
    )
    resp.raise_for_status()
    self._token = resp.json()["access_token"]
    return self._token
```

## Retry / Backoff manual (sin Retry adapter)

Para endpoints que no devuelven 5xx pero retornan `{"status": "pending"}`:

```python
def _wait_for_result(self, job_id, max_attempts=10, wait=2):
    for attempt in range(max_attempts):
        result = self.get_job(job_id)
        if result.get("status") == "done":
            return result
        if result.get("status") == "error":
            raise RuntimeError(f"Job {job_id} failed: {result}")
        time.sleep(wait * (attempt + 1))   # backoff lineal
    raise TimeoutError(f"Job {job_id} not done after {max_attempts} attempts")
```

## Mapeo a modelos Odoo

Separar mapeo de la llamada HTTP. Hacerlo en el modelo, no en el service:

```python
# models/sale_order.py
class SaleOrder(models.Model):
    _inherit = "sale.order"

    external_ref = fields.Char("External Ref", index=True, copy=False)

    def _sync_from_api(self):
        svc = MyApiService(self.env, prod=not self.env["ir.config_parameter"].sudo().get_param("my_module.test_mode"))
        data = svc.get_order(self.external_ref)
        # Mapeo explícito — no usar **data directamente
        self.write({
            "x_external_status": data.get("status"),
            "x_external_updated": data.get("updated_at"),
        })
```

## SOAP / XML

Usar `zeep` si está disponible en el virtualenv de Odoo, o `requests` con `lxml`:

```python
from lxml import etree

def _soap_request(self, action, body_xml):
    envelope = f"""
    <soapenv:Envelope xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/">
      <soapenv:Body>{body_xml}</soapenv:Body>
    </soapenv:Envelope>"""
    resp = self._session.post(
        self.base_url,
        data=envelope.encode(),
        headers={"Content-Type": "text/xml", "SOAPAction": action},
        timeout=DEFAULT_TIMEOUT,
    )
    resp.raise_for_status()
    return etree.fromstring(resp.content)
```

## Rate limiting

```python
import time

_last_call = 0.0
_min_interval = 0.2   # 5 req/s máximo

def _request(self, method, endpoint, **kwargs):
    global _last_call
    elapsed = time.monotonic() - _last_call
    if elapsed < _min_interval:
        time.sleep(_min_interval - elapsed)
    _last_call = time.monotonic()
    # ... resto de la lógica
```

## Error handling — convención

```python
from odoo.exceptions import UserError

def post_order(self, payload):
    try:
        return self._request("POST", "orders", json=payload)
    except Exception as exc:
        raise UserError(f"Error calling MyAPI: {exc}") from exc
```

No propagar excepciones técnicas al usuario final; envolver con `UserError` cuando se llama
desde un wizard o botón interactivo. En cron/job, loggear y relanzar para que el job quede
en estado `failed`.
