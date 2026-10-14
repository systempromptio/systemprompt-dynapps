# 24. Hardware / WebSocket-Dependent Flows (IoT print, bus, external devices)

Flows that end in a physical peripheral (IoT printer, scale, POS terminal) or a
`bus.bus._sendone(...)` push can't reach real hardware in a test. Don't skip them —
**capture the outgoing message and assert the routing + payload**. The device is a
detail; what you test is *which* target got *what*.

Applies to: IoT report printing (`ir.actions.report.render_and_send`), any
`bus.bus._sendone`, webhook/websocket emitters, hardware driver calls.

### 24.1 Capture the websocket/bus send instead of firing it

`render_and_send` loops the devices and calls `self._send_websocket(msg)` per device.
Patch that one seam to record what would have been sent:

```python
import base64
from unittest.mock import patch
from odoo.tests.common import TransactionCase


class TestIoTPrintRouting(TransactionCase):
    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        box = cls.env['iot.box'].create({
            'name': 'Fake Box', 'identifier': 'fake-mac',
            'company_id': cls.env.company.id,
        })
        cls.printer = cls.env['iot.device'].create({
            'iot_id': box.id, 'name': 'P1', 'type': 'printer',
            'identifier': 'device-p1',   # identifier is readonly in UI, settable via ORM
        })
        cls.report = cls.env['ir.actions.report'].create({
            'name': 'R', 'model': 'res.partner',
            'report_name': 'my_module.test_report',
            'device_ids': [(6, 0, cls.printer.ids)],
        })
        cls.partner = cls.env['res.partner'].create({'name': 'X'})

    def test_routes_to_device(self):
        devices = self.report.get_devices_from_ids(self.report.device_ids.ids)
        sent = []
        # data_base64 in context => render_and_send SKIPS _render (no wkhtmltopdf needed)
        report = self.report.with_context(data_base64=base64.b64encode(b'%PDF-fake'))
        with patch.object(type(report), '_send_websocket',
                          lambda self, msg: sent.append(msg) or True):
            report.render_and_send(devices, self.partner.ids, data={}, print_id='job-1')
        self.assertEqual(len(sent), 1)
        self.assertEqual(sent[0]['device_identifier'], self.printer.identifier)
        self.assertEqual(sent[0]['document'], base64.b64encode(b'%PDF-fake'))
```

### 24.2 Assert the PDF is actually rendered (when that's the point)

To prove the report is *rendered* (not a pre-supplied blob), **omit** `data_base64`
and patch `_render` to a sentinel — then assert it was called and its bytes were sent:

```python
def test_renders_and_sends_pdf(self):
    devices = self.report.get_devices_from_ids(self.report.device_ids.ids)
    rendered, sent = [], []

    def _fake_render(self, report_ref, res_ids, data=None):
        rendered.append(report_ref)
        return (b'%PDF-real', 'pdf')

    with patch.object(type(self.report), '_render', _fake_render), \
            patch.object(type(self.report), '_send_websocket',
                         lambda self, msg: sent.append(msg) or True):
        self.report.render_and_send(devices, self.partner.ids, data={}, print_id='j')

    self.assertEqual(rendered, [self.report.report_name])   # PDF was rendered
    self.assertEqual(sent[0]['document'], base64.b64encode(b'%PDF-real'))  # ...and sent
```

Patching `_render` also avoids wkhtmltopdf, which is often missing/flaky in CI.

### 24.3 Simulate "press Print" end to end (mirror the JS handler)

The frontend `iot_report_action_handler` orchestrates the print; there is no Python
entrypoint that does the whole thing. Reproduce its steps in the test to cover the
routing decision (see IoT gotcha IOTG33 in `odoo-deep-knowledge`):

```python
def _simulate_press_print(self, user):
    report = self.report.with_user(user)
    printer_ids = report.get_user_printer_ids()          # custom per-user hook, if any
    if not printer_ids:
        printer_ids = report.device_ids.ids              # fallback: report's own config
    devices = report.get_devices_from_ids(printer_ids)
    report = report.with_context(data_base64=base64.b64encode(b'%PDF'))
    sent = []
    with patch.object(type(report), '_send_websocket',
                      lambda self, msg: sent.append(msg) or True):
        report.render_and_send(devices, self.partner.ids, data={}, print_id='j')
    return sent
```

### Gotchas

- **`data_base64` in context is the skip-render switch** — with it, `render_and_send`
  never calls `_render`; without it, it renders. Choose per what you're asserting.
- **Patch `type(record)`, not the record** — `patch.object(type(report), ...)` targets
  the registry class so the override applies to every recordset in the test.
- **`iot.device.identifier` is readonly in the UI but settable via ORM `create`** — set
  it explicitly or your assertions compare empty strings.
- **The message is dual-keyed** — use the flat `device_identifier`/`iot_identifier`
  keys (newer boxes), not only the nested `iotDevice.*` (see `iot/core.md`).
- **IoT print only runs as an internal user** (IOTG30) — build test users in
  `base.group_user`, not portal/public.
- This is the general shape for **any** `bus.bus._sendone` flow: patch `_sendone` (or
  the model method wrapping it), assert channel + payload.
