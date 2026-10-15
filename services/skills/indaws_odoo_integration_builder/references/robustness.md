# Robustez — Idempotencia, Async, Secretos, Logging

## Idempotencia

### Campo de referencia externa

```python
class MyModel(models.Model):
    _name = "my.model"

    external_ref = fields.Char(
        "External Reference",
        index=True,
        copy=False,
        help="Unique ID in the external system. Prevents duplicate creation.",
    )

    _sql_constraints = [
        ("external_ref_uniq", "unique(external_ref)", "External reference must be unique"),
    ]
```

### Patrón create-or-update idempotente

```python
def _upsert_from_api(self, data):
    """Idempotent: create if new, update if exists."""
    external_ref = data["id"]
    existing = self.search([("external_ref", "=", external_ref)], limit=1)
    vals = self._map_api_to_vals(data)
    if existing:
        existing.write(vals)
        return existing
    vals["external_ref"] = external_ref
    return self.create(vals)
```

### Timestamp de última sincronización

```python
last_sync = fields.Datetime("Last Sync", readonly=True)

def _upsert_from_api(self, data):
    vals = self._map_api_to_vals(data)
    vals["last_sync"] = fields.Datetime.now()
    ...
```

---

## queue.job (OCA)

### Requisito en `__manifest__.py`

```python
"depends": ["base", "queue_job"],   # el addon instalable OCA es 'queue_job'
```

### Encolar un método (NO hay decorador `@job` en v14+)

Cualquier método se ejecuta como job llamándolo con `.with_delay()` (definido en
`queue_job/models/base.py:22`, `Base.with_delay` v14, `:21` v19). El decorador
`@job` (`from odoo.addons.queue_job.job import job`) NO existe en queue_job v14+.

Verificado contra `/opt/OCA/v{14..19}/queue/queue_job`: no hay ningún `def job` en el addon, y
`queue_job/__init__.py:10` (bloque de re-exports, sin simbolo envolvente) solo re-exporta `identity_exact` de `.job`, y el `post_init_hook` esta en `:6` (la `:9` es el comentario
`# shortcuts`) — el nombre `job` no está en la superficie pública de ninguna versión. Y el propio
addon lo cuenta: *"`@job` and `@related_action` deprecated, any method can be delayed, and configured
using `queue.job.function` records"* (`queue_job/readme/HISTORY.rst`, entrada **Next** de la rama que
migró desde 13.0), y `USAGE` avisa de que *"in earlier versions, jobs could be configured using the
`@job` decorator"*.

**Corroboración por el lado del consumidor — remedida 2026-08-17, y la cuenta anterior estaba mal en
las dos mitades** (decía *"un módulo en v14"* y *"0 en v16+"*). Módulos de OCA que aún importan
`from odoo.addons.queue_job.job import job`:

| | v14 | v15 | v16 | v17 | v18 | v19 |
|---|---|---|---|---|---|---|
| ficheros | **4** | **4** | **4** | 0 | **1** | 0 |

Y **importa quiénes son, no cuántos**: en v14/v15/v16 dos de los cuatro son de `l10n-spain` —
`l10n_es_ticketbai_batuz/models/lroe_operation.py` y `l10n_es_aeat_sii_oca/models/account_move.py` —
o sea **AEAT y TicketBAI**, no un rincón exótico. Los otros son
`edi/edi_voxel_oca/models/voxel_mixin.py` (v14/v15) y
`server-ux/document_quick_access_folder_auto_classification` (v14→v18, el único superviviente en
v18). Sus imports van dentro de un `try` precisamente porque fallan (p.ej.
`edi_voxel_oca/models/voxel_mixin.py:17` (OCA `edi`, repo no clonado en este disco), con los usos en `:73`, `:136`, `:171`).

Si tocas uno de esos módulos, no te sorprendas de ver `@job`: **está muerto pero escrito**, y el
`try` lo tapa.

La vía documentada es `with_delay()`, y **el fichero cambia de formato**: `readme/USAGE.rst:11` en
v14-v16, **`readme/USAGE.md:9`** en v17-v19 (todo el `readme/` de queue_job pasó de `.rst` a `.md`
en v17).

```python
class MyModel(models.Model):
    _name = "my.model"

    def action_sync_all(self):
        """Enqueue one job per record."""
        for record in self:
            record.with_delay(priority=10, description=f"Sync {record.name}")._sync_record()

    def _sync_record(self):
        """Executed by queue worker. Must be idempotent."""
        svc = MyApiService(self.env)
        data = svc.get_order(self.external_ref)
        self._upsert_from_api(data)
```

### `with_delay()` parámetros útiles

| Parámetro | Uso |
|-----------|-----|
| `priority` | 0-100; menor = antes. Default 10 |
| `max_retries` | Reintentos si el job falla. Default 5 |
| `description` | Texto visible en la cola |
| `eta` | `datetime` o segundos — ejecutar después de |
| `identity_key` | Clave de deduplicación; evita jobs duplicados |

### Deduplicación con identity_key

```python
from odoo.addons.queue_job.job import identity_exact

record.with_delay(
    identity_key=identity_exact,   # mismos args = mismo job, no duplicado
    description=f"Sync {record.external_ref}",
)._sync_record()
```

---

## Cron periódico

```xml
<!-- data/cron.xml -->
<record id="cron_sync_orders" model="ir.cron">
    <field name="name">Sync Orders from MyAPI</field>
    <field name="model_id" ref="model_my_model"/>
    <field name="state">code</field>
    <field name="code">model._cron_sync_orders()</field>
    <field name="interval_number">1</field>
    <field name="interval_type">hours</field>
    <field name="active">True</field>
    <!-- NO usar numbercall (eliminado en v18) -->
</record>
```

```python
@api.model
def _cron_sync_orders(self):
    """Called by cron. Fetches incremental updates since last sync."""
    last = (
        self.env["ir.config_parameter"]
        .sudo()
        .get_param("my_module.last_sync_ts", default="")
    )
    svc = MyApiService(self.env)
    orders = svc.get_orders_since(last or "2000-01-01T00:00:00Z")
    for order_data in orders:
        self.sudo()._upsert_from_api(order_data)
    self.env["ir.config_parameter"].sudo().set_param(
        "my_module.last_sync_ts",
        fields.Datetime.now().isoformat(),
    )
```

---

## Secretos en `ir.config_parameter`

### Registrar parámetros en XML (valores vacíos por defecto)

```xml
<!-- data/config_parameters.xml -->
<record id="param_api_key" model="ir.config_parameter">
    <field name="key">my_module.api_key</field>
    <field name="value"/>
</record>
<record id="param_api_url" model="ir.config_parameter">
    <field name="key">my_module.api_url</field>
    <field name="value">https://api.example.com/v1</field>
</record>
```

### Leer parámetros en código

```python
def _get_config(self, key, required=True):
    value = self.env["ir.config_parameter"].sudo().get_param(f"my_module.{key}", "")
    if required and not value:
        raise UserError(_(f"Missing configuration: my_module.{key}"))
    return value
```

### Parámetros desde config file (para ambientes Docker/CI)

```python
# En service/__init__.py o models
import os
API_KEY = os.environ.get("MY_MODULE_API_KEY") or \
    lambda env: env["ir.config_parameter"].sudo().get_param("my_module.api_key")
```

---

## Rate limiting

### Limitar requests por ventana de tiempo

```python
import time

class MyApiService:
    _RATE_LIMIT_RPS = 5   # máx 5 requests/segundo

    def __init__(self, env):
        self._last_call_ts = 0.0
        self._min_interval = 1.0 / self._RATE_LIMIT_RPS

    def _request(self, method, endpoint, **kwargs):
        elapsed = time.monotonic() - self._last_call_ts
        if elapsed < self._min_interval:
            time.sleep(self._min_interval - elapsed)
        self._last_call_ts = time.monotonic()
        # ... hacer la petición
```

---

## Logging y trazabilidad

### Niveles de log recomendados

| Nivel | Cuándo |
|-------|--------|
| `DEBUG` | Payload completo request/response (solo en dev) |
| `INFO` | Inicio/fin de sincronización, registros creados/actualizados |
| `WARNING` | Registro ignorado por duplicado, campo inesperado |
| `ERROR` | Fallo de conexión, error HTTP 4xx/5xx, excepción no recuperada |

### Patrón logging estructurado

```python
_logger = logging.getLogger(__name__)

# Al inicio de cron/job
_logger.info("Starting sync [%s] from=%s", self._name, last_sync_ts)

# Por registro procesado
_logger.debug("Processing external_ref=%s data=%s", external_ref, data)

# Resultado de cron
_logger.info("Sync complete: created=%d updated=%d errors=%d", created, updated, errors)

# Error recuperable
_logger.warning(
    "Skipping external_ref=%s: %s", external_ref, exc, exc_info=False
)

# Error crítico
_logger.error(
    "API call failed [%s %s]: %s", method, url, exc, exc_info=True
)
```

### Loggear sin exponer secretos

```python
safe_headers = {k: "***" if "key" in k.lower() or "auth" in k.lower() else v
                for k, v in headers.items()}
_logger.debug("Request headers: %s", safe_headers)
```

---

## Reproceso de jobs fallidos

Con `queue.job`, los jobs fallidos quedan en estado `failed` en la cola. Se pueden reprocesar
desde la UI (`Queue Jobs > Failed`) o programáticamente:

```python
@api.model
def _requeue_failed_jobs(self, channel="root.my_module"):
    failed = self.env["queue.job"].search([
        ("state", "=", "failed"),
        ("channel", "=", channel),
    ])
    failed.requeue()
    _logger.info("Requeued %d failed jobs", len(failed))
```

---

## Transaccionalidad

- Los webhooks procesan en su propia transacción. Si el procesado falla, la respuesta al
  proveedor ya fue enviada (HTTP 200). Diseñar para reproceso desde logs o cola.
- Para garantizar que el webhook se guarda aunque falle el procesado:

```python
def webhook_receive(self):
    payload = request.get_json_data()
    # 1. Guardar el raw payload siempre (en un modelo de log o queue.job)
    self.env["my.webhook.log"].sudo().create({
        "payload": json.dumps(payload),
        "event": payload.get("event"),
        "state": "pending",
    })
    # 2. El cron o job lo procesa después
    return {"status": "queued"}
```
