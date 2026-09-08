"""Small JSON/SSE MCP client used by the demo seeder and verification harness."""
import json
import urllib.request


class McpClient:
    def __init__(self, url, token, context_headers=None, timeout=120):
        self.timeout = timeout
        self.context_headers = context_headers or {}
        self.url = url
        self.token = token
        self.session = None
        self.sequence = 0
        self.rpc('initialize', {'protocolVersion': '2025-06-18', 'capabilities': {},
                               'clientInfo': {'name': 'astound-requirements-demo', 'version': '1.0'}})
        self.rpc('notifications/initialized', {}, notification=True)

    def rpc(self, method, params, notification=False):
        self.sequence += 1
        body = {'jsonrpc': '2.0', 'method': method, 'params': params}
        if not notification:
            body['id'] = self.sequence
        headers = {'Authorization': 'Bearer '+self.token, 'Content-Type': 'application/json',
                   'Accept': 'application/json, text/event-stream', 'MCP-Protocol-Version': '2025-06-18'}
        headers.update(self.context_headers)
        if self.session:
            headers['Mcp-Session-Id'] = self.session
        req = urllib.request.Request(self.url, data=json.dumps(body).encode(), headers=headers)
        with urllib.request.urlopen(req, timeout=self.timeout) as response:
            self.session = response.headers.get('Mcp-Session-Id', self.session)
            raw = response.read().decode()
        if not raw.strip():
            return {}
        if raw.lstrip().startswith('{'):
            message = json.loads(raw)
        else:
            messages = [json.loads(line[5:].strip()) for line in raw.splitlines() if line.startswith('data:') and line[5:].strip() and line[5:].strip() != '[DONE]']
            message = next((m for m in reversed(messages) if m.get('id') == body.get('id')), {})
        if 'error' in message:
            raise RuntimeError(json.dumps(message['error']))
        return message.get('result', {})

    def tools(self):
        result, cursor = [], None
        while True:
            page = self.rpc('tools/list', {'cursor': cursor} if cursor else {})
            result.extend(page.get('tools', []))
            cursor = page.get('nextCursor')
            if not cursor:
                return result

    def call(self, name, arguments):
        result = self.rpc('tools/call', {'name': name, 'arguments': arguments})
        if result.get('isError'):
            raise RuntimeError(json.dumps(result.get('content', []))[:1000])
        value = result.get('structuredContent')
        if value is None:
            texts = [v['text'] for v in result.get('content', []) if v.get('type') == 'text']
            for text in texts:
                try:
                    value = json.loads(text)
                    break
                except json.JSONDecodeError:
                    continue
        if value is None:
            raise RuntimeError('No structured response from '+name)
        if isinstance(value, dict) and value.get('artifact_type') == 'text':
            try:
                return json.loads(value['content'])
            except json.JSONDecodeError:
                return value
        return value.get('data', value) if isinstance(value, dict) else value
