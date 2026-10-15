# 8. HttpCase Patterns

### 8.1 Authenticated URL Open

```python
from odoo.tests import tagged, HttpCase

@tagged('post_install', '-at_install')
class TestControllers(HttpCase):

    def test_authenticated_endpoint(self):
        self.authenticate('admin', 'admin')
        response = self.url_open('/my/endpoint', data={'key': 'value'})
        self.assertEqual(response.status_code, 200)
        self.assertIn('expected_content', response.text)
```

### 8.2 Portal Access Test

```python
def test_portal_pdf_report(self):
    """Test portal user can download PDF report with token."""
    self.authenticate(None, None)

    # Without token -> redirect
    resp = self.url_open('/my/orders/%d?report_type=pdf' % self.order.id,
                         allow_redirects=False)
    self.assertEqual(resp.status_code, 303)

    # With valid token -> OK
    token = self.order.access_token
    resp = self.url_open(
        '/my/orders/%d?access_token=%s&report_type=pdf' % (self.order.id, token),
        allow_redirects=False,
    )
    self.assertEqual(resp.status_code, 200)
```

### 8.3 JSON-RPC Controller Test

```python
import json

def test_json_rpc_endpoint(self):
    """Test JSON-RPC style controller."""
    self.authenticate('admin', 'admin')
    payload = json.dumps({
        'jsonrpc': '2.0',
        'method': 'call',
        'params': {'model': 'res.partner', 'method': 'search_count', 'args': [[]]},
    })
    resp = self.url_open('/web/dataset/call_kw', data=payload,
                         headers={'Content-Type': 'application/json'})
    result = resp.json()
    self.assertIn('result', result)
```
