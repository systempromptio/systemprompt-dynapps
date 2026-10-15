# Studio Detection

Como identificar customizaciones Studio en una BD Odoo, por version.

Studio guarda casi todo en BD (no en filesystem) salvo cuando se exporta como modulo.

## Disponibilidad por version

| Odoo version | Studio | Notas |
|--------------|--------|-------|
| 14.0 | ❌ Enterprise only, raro | `studio_scan.sh` retorna `studio_available: false` |
| 15.0 | ❌ Enterprise only, raro | idem |
| 16.0 | ✅ Enterprise | Modulo `web_studio` instalado |
| 17.0 | ✅ Enterprise | idem |
| 18.0 | ✅ Enterprise | idem |
| 19.0 | ✅ Enterprise | idem |

Detectar disponibilidad:
```python
# Studio existe si modulo web_studio esta installed
studio_available = bool(search_records('ir.module.module', [
    ('name', '=', 'web_studio'),
    ('state', '=', 'installed')
]))
```

Si `studio_available=false`, `studio.yaml` se genera con bloque metadata + listas vacias. NO falla.

## Senales de Studio en BD

### Campos custom (mas comun)

```python
# ir.model.fields con state='manual' y nombre x_*
search_records('ir.model.fields', [
    ('state', '=', 'manual'),
    ('name', 'like', 'x_%'),
])
```

Devuelve campos creados desde Studio o desde Settings → Technical → Fields. Filtrar por:

- `name` empieza con `x_studio_` -> creado via Studio UI
- `name` empieza con `x_` (pero no `x_studio_`) -> creado via Studio o Settings o codigo manual
- `state == 'manual'` -> NO viene de codigo (sin `__manifest__.py`)

Para diferenciar Studio de "campo manual via Settings":
- Studio escribe en `field_description` (label) typicamente con prefijo o consistencia
- Studio escribe `tracking` y `index` por defecto consistentes
- En `ir.model.data` el `module='studio'` para registros gestionados via Studio (no siempre)

Heuristica: `state='manual' AND name like 'x_%'` -> tratarlo como Studio (en la practica casi todo lo manual hoy viene de Studio).

### Modelos custom (modelo enteramente Studio)

```python
search_records('ir.model', [
    ('state', '=', 'manual'),
    ('model', 'like', 'x_%'),
])
```

Modelos creados desde Studio empiezan con `x_`. Los reportamos en `studio.yaml -> models[]`.

### Server actions Studio

```python
search_records('ir.actions.server', [
    '|',
    ('xml_id', 'like', 'studio_customization.%'),
    ('xml_id', 'like', 'studio.%'),
])
```

XML ID format depende de version:
- v16: `studio_customization.studio_action_<id>`
- v17/v18/v19: `studio_customization.studio_action_<n>` o sin xml_id (creacion ad-hoc)

Fallback: filtrar por `model_id.model` que empieza con `x_` o por `name` que contenga "Studio" (frágil).

Mejor query combinada via `ir.model.data`:

```python
search_records('ir.model.data', [
    ('model', '=', 'ir.actions.server'),
    ('module', '=', 'studio_customization'),
])
# luego browse(res_id) sobre cada uno
```

### Automated actions

```python
search_records('base.automation', [])  # leer todas
# Filtrar por:
# - tienen xml_id en ir.model.data con module='studio_customization'
# - O fueron creadas por user_id == admin_user (poco fiable)
```

Mejor: cruzar con `ir.model.data` igual que server actions.

### Menus Studio

```python
search_records('ir.ui.menu', [])
# Cruzar con ir.model.data module='studio_customization'
```

### Vistas Studio

Vistas modificadas por Studio tienen:
- `arch_fs == False` (no vienen de filesystem)
- `arch_db` con la modificacion
- `xml_id` con prefijo `studio_customization.` (si existe)

Query:

```python
search_records('ir.ui.view', [
    ('arch_fs', '=', False),
    ('inherit_id', '!=', False),
])
# luego cruzar con ir.model.data module='studio_customization'
```

Tambien hay vistas COMPLETAMENTE custom (no inheritan): `inherit_id == False`. Esas suelen ser para los modelos `x_*`.

## Output a `studio.yaml`

```yaml
metadata:
  scanned_at: "2026-04-25T08:30:00Z"
  odoo_version: 18.0
  studio_available: true
  total_fields: 28
  total_server_actions: 3
  total_automations: 2
  total_models: 1
  total_views: 12
  total_menus: 4

models:
  - name: x_proyecto_obra
    description: "Obras de construccion"
    fields_count: 12
    record_count: 47        # recordcount via search_count
    has_records: true

fields:
  - model: sale.order
    name: x_studio_obra_id
    field_type: many2one
    relation: x_proyecto_obra
    label: "Obra"
    required: false
    in_views: ["studio_customization.studio_view_sale_order_form"]

server_actions:
  - xml_id: studio_customization.studio_action_5
    name: "Marcar obra como entregada"
    model: x_proyecto_obra
    type: code
    code_lines: 12

automated_actions:
  - xml_id: studio_customization.studio_automation_3
    name: "Notificar comercial al confirmar"
    model: sale.order
    trigger: on_create_or_write

menus:
  - xml_id: studio_customization.studio_menu_obras
    name: "Obras"
    parent: base.menu_sales
    action: studio_customization.studio_action_obras

views:
  - model: sale.order
    name: "Studio: sale.order.form"
    type: form
    inherit_id: sale.view_order_form
    studio_origin: true
    arch_fs_empty: true
```

## Casos especiales

### Studio exportado como modulo
Si el cliente uso "Studio → Export" para empaquetar customizaciones, aparece como `studio_customization` en `ir.module.module` con su `__manifest__.py`. Lo tratamos como `custom` en `modules.yaml` Y mantenemos las customizaciones vivas en `studio.yaml` (pueden coexistir si el cliente sigue editando en Studio despues de exportar).

### Campos manuales NO Studio
Algunos clientes crean campos via Settings → Technical sin Studio. Tienen `state='manual'` igualmente. Los reportamos en `studio.yaml` con flag `created_via: manual_settings` (vs `studio` que es el default).

Detectar: si el field NO tiene entrada en `ir.model.data` con `module='studio_customization'` o `module='studio'`, marcarlo como `manual_settings`.

### Studio en modelos creados por modulos custom
Un Studio `x_studio_obra_id` puede vivir en `sale.order` (estandar extendido por indaws_sale) o en `cliente.contrato` (modelo creado por modulo custom). En ambos casos: reportar en `studio.yaml -> fields[]` con su `model`. La colision la calcula `compute_collisions.py` cruzando con `extends_models` de los modulos custom/indaws_core.

### Performance
Si la BD tiene >500 fields manuales, el scan completo tarda. `studio_scan.sh` debe paginar (limit=200, offset=N) y mostrar progreso en stderr. No bloquear.
