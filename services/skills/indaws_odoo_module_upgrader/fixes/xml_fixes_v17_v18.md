# XML Fix Templates: Odoo 17 → 18

Scripts automatizados para las transformaciones XML del salto v17→v18.

## 1. t-raw → t-out

```python
import re

def convert_t_raw_to_t_out(xml_content):
    """Reemplaza t-raw por t-out (eliminado en v18)."""
    return re.sub(r't-raw=', 't-out=', xml_content)
```

## 2. Bootstrap 5 class migration

```python
BOOTSTRAP_REPLACEMENTS = {
    'float-left': 'float-start',
    'float-right': 'float-end',
    'text-left': 'text-start',
    'text-right': 'text-end',
    'ml-0': 'ms-0', 'ml-1': 'ms-1', 'ml-2': 'ms-2', 'ml-3': 'ms-3',
    'ml-4': 'ms-4', 'ml-5': 'ms-5', 'ml-auto': 'ms-auto',
    'mr-0': 'me-0', 'mr-1': 'me-1', 'mr-2': 'me-2', 'mr-3': 'me-3',
    'mr-4': 'me-4', 'mr-5': 'me-5', 'mr-auto': 'me-auto',
    'pl-0': 'ps-0', 'pl-1': 'ps-1', 'pl-2': 'ps-2', 'pl-3': 'ps-3',
    'pl-4': 'ps-4', 'pl-5': 'ps-5',
    'pr-0': 'pe-0', 'pr-1': 'pe-1', 'pr-2': 'pe-2', 'pr-3': 'pe-3',
    'pr-4': 'pe-4', 'pr-5': 'pe-5',
    'font-weight-bold': 'fw-bold',
    'font-weight-normal': 'fw-normal',
    'font-italic': 'fst-italic',
    'badge-primary': 'bg-primary',
    'badge-secondary': 'bg-secondary',
    'badge-success': 'bg-success',
    'badge-danger': 'bg-danger',
    'badge-warning': 'bg-warning text-dark',
    'badge-info': 'bg-info',
}

def convert_bootstrap_classes(content):
    """Reemplaza clases Bootstrap 4 por Bootstrap 5."""
    import re
    changes = 0
    for old, new in BOOTSTRAP_REPLACEMENTS.items():
        # Solo reemplazar como clase CSS (precedido/seguido por espacio, comilla, o inicio/fin)
        pattern = r'(?<=["\s])' + re.escape(old) + r'(?=["\s])'
        new_content = re.sub(pattern, new, content)
        if new_content != content:
            changes += content.count(old)
            content = new_content
    return content, changes
```

## 3. layout_document_title removal

```python
def remove_layout_document_title(xml_content):
    """Elimina t-set="layout_document_title" (no existe en v18)."""
    import re
    pattern = r'\s*<t\s+t-set="layout_document_title"[^/]*/>\s*'
    content = re.sub(pattern, '\n', xml_content)
    pattern2 = r'\s*<t\s+t-set="layout_document_title"[^>]*>.*?</t>\s*'
    content = re.sub(pattern2, '\n', content, flags=re.DOTALL)
    return content
```

## 4. http_routing removal from depends

```python
import re

def fix_manifest_depends(manifest_content):
    """Elimina http_routing de depends (absorbido en core v18)."""
    # Eliminar 'http_routing' de la lista de depends
    content = re.sub(r"'http_routing',?\s*", '', manifest_content)
    content = re.sub(r'"http_routing",?\s*', '', content)
    # Limpiar comas dobles o trailing commas
    content = re.sub(r',\s*,', ',', content)
    content = re.sub(r',\s*\]', ']', content)
    return content
```

## 5. QWeb report field → t-out en t-foreach

```python
import re

def fix_field_in_foreach(xml_content):
    """Detecta <field t-field="line.x"> dentro de t-foreach y sugiere t-out.

    NO hace reemplazo automático — solo detecta, porque el formato puede variar.
    Retorna lista de líneas que necesitan revisión manual.
    """
    lines_to_fix = []
    in_foreach = False
    foreach_var = None

    for i, line in enumerate(xml_content.splitlines(), 1):
        # Detectar inicio de t-foreach
        match = re.search(r't-foreach="[^"]*"\s+t-as="(\w+)"', line)
        if match:
            in_foreach = True
            foreach_var = match.group(1)

        # Detectar fin del foreach (simplificado)
        if in_foreach and '</t>' in line and foreach_var:
            in_foreach = False
            foreach_var = None

        # Detectar t-field con variable del foreach
        if in_foreach and foreach_var:
            if re.search(rf't-field="{foreach_var}\.', line):
                lines_to_fix.append((i, line.strip()))

    return lines_to_fix
```

## 6. Procesador batch v17→v18

```python
import os
import glob

def process_module_v17_to_v18(module_path):
    """Aplica todas las transformaciones v17→v18 a un módulo."""
    xml_files = glob.glob(os.path.join(module_path, '**/*.xml'), recursive=True)
    py_files = glob.glob(os.path.join(module_path, '**/*.py'), recursive=True)

    stats = {'t_raw': 0, 'bootstrap': 0, 'layout_title': 0, 'field_foreach': []}

    for filepath in xml_files:
        with open(filepath) as f:
            content = original = f.read()

        # 1. t-raw → t-out
        if 't-raw=' in content:
            stats['t_raw'] += content.count('t-raw=')
            content = convert_t_raw_to_t_out(content)

        # 2. Bootstrap classes
        content, changes = convert_bootstrap_classes(content)
        stats['bootstrap'] += changes

        # 3. layout_document_title
        if 'layout_document_title' in content:
            stats['layout_title'] += 1
            content = remove_layout_document_title(content)

        # 4. field en t-foreach (solo detección)
        issues = fix_field_in_foreach(content)
        if issues:
            stats['field_foreach'].extend([(filepath, ln, line) for ln, line in issues])

        if content != original:
            with open(filepath, 'w') as f:
                f.write(content)

    # 5. Manifests — eliminar http_routing
    for filepath in py_files:
        if filepath.endswith('__manifest__.py'):
            with open(filepath) as f:
                content = original = f.read()
            if 'http_routing' in content:
                content = fix_manifest_depends(content)
                with open(filepath, 'w') as f:
                    f.write(content)

    print(f"=== {os.path.basename(module_path)} v17→v18 ===")
    print(f"  t-raw→t-out: {stats['t_raw']}")
    print(f"  Bootstrap classes: {stats['bootstrap']}")
    print(f"  layout_document_title removed: {stats['layout_title']}")
    if stats['field_foreach']:
        print(f"  ⚠ {len(stats['field_foreach'])} field-in-foreach (revisión manual):")
        for path, ln, line in stats['field_foreach']:
            print(f"    {os.path.basename(path)}:{ln}: {line[:80]}")
```

## Gotchas

1. **Bootstrap class replacement debe respetar boundaries**: No reemplazar `ml-` dentro de `html-` o nombres de clase custom. El regex usa word boundaries.
2. **t-raw en herencia de vistas**: Si la vista hereda de otra con `t-raw`, el fix solo aplica al fichero XML actual. La vista padre puede seguir con `t-raw` si es de otro módulo.
3. **layout_document_title en reportes custom**: Algunos reportes custom dependen de este campo para el PDF filename. Verificar si hay lógica Python que lea este valor.
4. **http_routing en depends transitivos**: Si modulo A depende de modulo B que depende de http_routing, quitar http_routing de A no es suficiente — B también debe actualizarse.
