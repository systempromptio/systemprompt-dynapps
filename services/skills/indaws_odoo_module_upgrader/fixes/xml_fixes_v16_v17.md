# XML Fix Templates: Odoo 16 → 17

Scripts automatizados para las transformaciones XML más comunes del salto v16→v17.

## 1. attrs → atributos separados (el más complejo)

```python
import re
import ast

def parse_attrs_domain(domain_str):
    """Convierte un domain Odoo a expresión Python v17.

    Ejemplos:
      [('state', '=', 'draft')] → "state == 'draft'"
      [('type', 'in', ['a','b'])] → "type in ('a', 'b')"
      ['|', ('a', '=', 1), ('b', '=', 2)] → "a == 1 or b == 2"
    """
    try:
        domain = ast.literal_eval(domain_str.strip())
    except (ValueError, SyntaxError):
        return None  # No se puede parsear — requiere fix manual

    if not isinstance(domain, list) or not domain:
        return None

    def _leaf_to_expr(leaf):
        if not isinstance(leaf, (list, tuple)) or len(leaf) != 3:
            return None
        field, op, value = leaf
        if op == '=':
            if value is True:
                return str(field)
            elif value is False:
                return f"not {field}"
            elif isinstance(value, str):
                return f"{field} == '{value}'"
            else:
                return f"{field} == {value}"
        elif op == '!=':
            if value is True:
                return f"not {field}"
            elif value is False:
                return str(field)
            elif isinstance(value, str):
                return f"{field} != '{value}'"
            else:
                return f"{field} != {value}"
        elif op == 'in':
            vals = ', '.join(f"'{v}'" if isinstance(v, str) else str(v) for v in value)
            return f"{field} in ({vals})"
        elif op == 'not in':
            vals = ', '.join(f"'{v}'" if isinstance(v, str) else str(v) for v in value)
            return f"{field} not in ({vals})"
        elif op in ('>', '<', '>=', '<='):
            if isinstance(value, str):
                return f"{field} {op} '{value}'"
            return f"{field} {op} {value}"
        return None

    def _parse_domain(domain, pos=0):
        if pos >= len(domain):
            return None, pos
        item = domain[pos]
        if item == '&':
            left, pos = _parse_domain(domain, pos + 1)
            right, pos = _parse_domain(domain, pos)
            if left and right:
                return f"{left} and {right}", pos
            return None, pos
        elif item == '|':
            left, pos = _parse_domain(domain, pos + 1)
            right, pos = _parse_domain(domain, pos)
            if left and right:
                return f"{left} or {right}", pos
            return None, pos
        elif item == '!':
            expr, pos = _parse_domain(domain, pos + 1)
            if expr:
                return f"not ({expr})", pos
            return None, pos
        else:
            expr = _leaf_to_expr(item)
            return expr, pos + 1

    # Si no hay operadores explícitos, todo es AND implícito
    has_operators = any(x in ('&', '|', '!') for x in domain)
    if not has_operators:
        parts = [_leaf_to_expr(leaf) for leaf in domain]
        if all(parts):
            return ' and '.join(parts) if len(parts) > 1 else parts[0]
        return None

    expr, _ = _parse_domain(domain)
    return expr


def convert_attrs_to_expressions(xml_content):
    """Convierte attrs={'invisible': [...], 'readonly': [...]} a atributos separados.

    Retorna: (converted_content, manual_fixes_needed: list)
    """
    manual_fixes = []

    # Regex para capturar attrs="{'key': [domain], ...}"
    pattern = r'''attrs\s*=\s*"(\{[^"]+\})"'''

    def replace_attrs(match):
        attrs_str = match.group(1)
        # Reemplazar comillas simples escapadas
        attrs_str = attrs_str.replace('&quot;', '"')

        try:
            attrs = ast.literal_eval(attrs_str)
        except (ValueError, SyntaxError):
            manual_fixes.append(f"Cannot parse attrs: {attrs_str[:80]}...")
            return match.group(0)

        new_attrs = []
        for key, domain in attrs.items():
            if key not in ('invisible', 'readonly', 'required', 'column_invisible'):
                manual_fixes.append(f"Unknown attrs key '{key}': {domain}")
                continue
            expr = parse_attrs_domain(str(domain))
            if expr:
                new_attrs.append(f'{key}="{expr}"')
            else:
                manual_fixes.append(f"Cannot convert {key} domain: {domain}")
                new_attrs.append(f'{key}="TODO_MANUAL_FIX"')

        return ' '.join(new_attrs)

    result = re.sub(pattern, replace_attrs, xml_content)
    return result, manual_fixes
```

### Uso
```python
content = open('views/sale_order_view.xml').read()
new_content, fixes = convert_attrs_to_expressions(content)
if fixes:
    print(f"  ⚠ {len(fixes)} attrs requieren fix manual:")
    for f in fixes:
        print(f"    - {f}")
open('views/sale_order_view.xml', 'w').write(new_content)
```

## 2. states → invisible expression

```python
import re

def convert_states_to_invisible(xml_content):
    """Convierte states="draft,sent" a invisible="state not in ('draft', 'sent')"."""
    manual_fixes = []

    def replace_states(match):
        states_str = match.group(1)
        states = [s.strip() for s in states_str.split(',')]

        if len(states) == 1:
            return f'''invisible="state != '{states[0]}'"'''
        else:
            vals = ', '.join(f"'{s}'" for s in states)
            return f'invisible="state not in ({vals})"'

    # states="value1,value2"
    result = re.sub(r'states="([^"]+)"', replace_states, xml_content)
    return result, manual_fixes
```

## 3. tree → list (batch)

```python
import re

def convert_tree_to_list(xml_content):
    """Convierte <tree> a <list>, elimina edit="1"."""
    content = xml_content
    # tree → list
    content = re.sub(r'<tree(\s|>|/)', r'<list\1', content)
    content = re.sub(r'</tree>', '</list>', content)
    # Eliminar edit="1" o edit="0"
    content = re.sub(r'\s+edit="[01]"', '', content)
    return content
```

## 4. Procesador batch completo

```python
import os
import glob

def process_module_v16_to_v17(module_path):
    """Aplica todas las transformaciones v16→v17 a un módulo."""
    xml_files = glob.glob(os.path.join(module_path, '**/*.xml'), recursive=True)

    total_attrs = 0
    total_states = 0
    total_tree = 0
    all_manual = []

    for filepath in xml_files:
        with open(filepath) as f:
            content = original = f.read()

        # 1. attrs → expressions
        if 'attrs=' in content:
            content, manual = convert_attrs_to_expressions(content)
            total_attrs += len(re.findall(r'attrs=', original))
            all_manual.extend([(filepath, m) for m in manual])

        # 2. states → invisible
        if re.search(r'states="', content):
            content, manual = convert_states_to_invisible(content)
            total_states += len(re.findall(r'states="', original))
            all_manual.extend([(filepath, m) for m in manual])

        # 3. tree → list
        if '<tree' in content:
            content = convert_tree_to_list(content)
            total_tree += original.count('<tree')

        if content != original:
            with open(filepath, 'w') as f:
                f.write(content)

    print(f"=== {os.path.basename(module_path)} v16→v17 ===")
    print(f"  attrs convertidos: {total_attrs}")
    print(f"  states convertidos: {total_states}")
    print(f"  tree→list: {total_tree}")
    if all_manual:
        print(f"  ⚠ {len(all_manual)} fixes manuales necesarios:")
        for path, msg in all_manual:
            print(f"    {os.path.basename(path)}: {msg}")
```

## Gotchas

1. **attrs con `context.get()`**: El parser no los maneja — requieren fix manual
2. **attrs con domain referencing parent**: `parent.field_id` en domain necesita `parent.field_id` en la expresión
3. **Campos que no están en la vista**: Si `invisible="other_field == 'x'"` y `other_field` no tiene `<field>` en la vista, Odoo lanzará error. Añadir `<field name="other_field" invisible="1"/>`.
4. **states en botones vs campos**: Ambos se convierten igual, pero los botones con `states` a veces tienen lógica adicional en Python (`button_confirm` chequea state). Verificar que no haya duplicación.
