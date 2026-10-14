# indaws_base: Server Action Extensions

## What indaws_base Adds

`indaws_base` extends `ir.actions.server._get_eval_context()` with:

```python
# indaws_base/models/ir_actions_server.py
eval_context.update({
    'json': safe_eval.json,                    # JSON garantizado (sin depender de mail)
    'open': open,                              # Acceso a ficheros del servidor
    'wrap_module': _wrap_module,               # Import seguro (solo metodos listados)
    'import_module': _import_module,           # Import completo (modulo entero)
})
```

## Capabilities Unlocked

| Need | How | Example |
|------|-----|---------|
| **Regex** (`re`) | `wrap_module('re', ['findall', 'sub'])` | `re.findall(r'\d+', text)` |
| **HTTP** (`requests`) | `import_module('requests', ['get', 'post'])` | `requests.get(url)` |
| **CSV** | `wrap_module('csv', ['reader', 'writer'])` | `csv.reader(f)` |
| **Files** | `open()` directo | `with open('/tmp/file.csv') as f:` |
| **JSON** | Guaranteed (no mail dependency) | `json.loads(data)` |
| Any Python lib | `import_module('lib', [...])` | Full module access |

## wrap_module vs import_module

| Function | Behavior | When to use |
|----------|----------|-------------|
| `wrap_module(name, methods)` | Returns object with ONLY listed methods | When you need specific functions (safer) |
| `import_module(name, fromlist)` | Returns complete module | When you need the full module API |

## Detection

- Ask the user, or check if target module depends on `indaws_base`
- In Indaws/Dynapps projects: **always assume indaws mode**
