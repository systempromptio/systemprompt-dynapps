# Inventory Schema

Esquemas completos de los ficheros que viven en `.ai-memory/inventory/`.

## modules.yaml

Indice maestro. Lista de modulos. Cada entrada:

```yaml
- name: indaws_sale_rappel              # technical name (ir.module.module.name)
  type: indaws_core                     # standard | oca | indaws_core | custom | studio
  subtype: indaws                       # ce | enterprise | oca | oca_fork | indaws | project | unknown
  version: 18.0.1.0.2                   # latest_version del manifest
  state: installed                      # installed | to upgrade | to install | uninstalled
  source_path: /opt/PROYECTS/indawsodoo/18/indaws_sale_rappel  # ruta filesystem (null si scan_source=remote_only)
  hash: sha256:abc123...                # sha256(manifest + py + xml + csv + js + scss). 'unknown' si no accesible
  hash_algo: v2                         # version del algoritmo del hash (v1=py only, v2=py+xml+csv+js+scss)
  last_modified: "2026-04-25T12:30:00Z" # max(mtime) de .py + .xml — detecta modulos sin tocar
  author: Indaws                        # author del manifest
  category: Sales                       # category del manifest
  depends:                              # dependencias declaradas en manifest
    - sale
    - indaws_sale
  auto_install: false
  application: false
  extends_models:                       # modelos extendidos via _inherit en codigo Python
    - sale.order
    - sale.order.line
  new_models:                           # modelos nuevos (_name)
    - rappel.line
  fields_added: 7                       # nº de fields.X(...) en codigo
  views_inherited: 3                    # nº de records con inherit_id en views/
  views_new: 1                          # nº de records sin inherit_id en views/
  has_cron: true                        # tiene ir.cron declarados
  has_server_actions: false             # tiene ir.actions.server custom
  has_automated_actions: false          # tiene base.automation
  has_security: true                    # tiene ir.model.access.csv o security.xml
  loc: 1245                             # lineas de codigo Python (sin comentarios/blank)
  js_loc: 230                           # lineas no-comentario de .js (OWL components, etc.)
  scss_loc: 45                          # lineas de .scss / .css / .sass
  application: false                    # apps principales: aparecen en menu Apps
  reverse_deps_count: 4                 # # de modulos que dependen de este
  migrations:                           # presencia de scripts /migrations/<ver>/
    present: true
    versions: ["18.0.1.0.0", "18.0.2.0.0"]
    scripts: 3
  monkey_patches: 0                     # # de hallazgos de "ClassName.attr = ..." top-level
  monkey_patch_findings: []             # primeros 10 hallazgos (file, line, snippet)
  risk: medium                          # low | medium | high (ver risk-scoring.md)
  risk_score: 45                        # 0-100 (incluye reverse_deps boost)
  purpose: "Calcula rappels sobre lineas de venta agrupadas por cliente"
  deep_dive: deep/indaws_sale_rappel.md # null si type in [standard, oca]
  last_scanned: "2026-04-25T08:30:00Z"
  scan_source: filesystem               # filesystem | remote_only (sin acceso al codigo)
```

Modulos `standard` y `oca`: solo `name`, `type`, `version`, `state`, `author`, `category`, `purpose` (resumen del manifest). Sin deep-dive, sin `extends_models`, sin hash (no aporta).

## studio.yaml

Customizaciones Studio. Estructura:

```yaml
metadata:
  scanned_at: "2026-04-25T08:30:00Z"
  odoo_version: 18.0
  studio_available: true                 # false en v14-v15

models:                                  # modelos custom Studio (x_*)
  - name: x_proyecto_obra
    description: "Obras de construccion"
    created_via: studio
    fields_count: 12
    has_records: true
    record_count: 47

fields:                                  # campos custom (state=manual + name like x_%)
  - model: sale.order
    name: x_studio_obra_id
    field_type: many2one
    relation: x_proyecto_obra
    label: "Obra"
    required: false
    created_via: studio
    in_views: [sale_order_form]
  - model: x_proyecto_obra
    name: x_studio_metros_cuadrados
    field_type: float
    label: "m²"

server_actions:                          # ir.actions.server con xml_id studio_*
  - xml_id: studio_action_5
    name: "Marcar obra como entregada"
    model: x_proyecto_obra
    type: code
    code_lines: 12
    triggered_by: button                 # button | manual | automation

automated_actions:                       # base.automation con xml_id studio_*
  - xml_id: studio_automation_3
    name: "Notificar comercial al confirmar"
    model: sale.order
    trigger: on_create_or_write
    action_type: code

menus:                                   # ir.ui.menu creados via Studio
  - xml_id: studio_menu_obras
    name: "Obras"
    parent: base.menu_sales
    action: studio_action_obras

views:                                   # ir.ui.view modificadas via Studio (arch_fs vacio)
  - model: sale.order
    name: "Studio: sale.order.form"
    type: form
    inherit_id: sale.view_order_form
    studio_origin: true
```

Si `studio_available: false` (v14-v15 o sin Studio): bloque `metadata` + listas vacias.

## collisions.yaml

Modelos extendidos por >1 modulo no-estandar (custom + indaws_core + studio). Auto-calculado desde `modules.yaml` y `studio.yaml`.

```yaml
collisions:
  - model: sale.order
    extended_by:
      - module: indaws_sale
        type: indaws_core
        fields_added: 4
        risk_contribution: medium
      - module: indaws_sale_rappel
        type: indaws_core
        fields_added: 2
        risk_contribution: low
      - module: cliente_x_sale_custom
        type: custom
        fields_added: 1
        risk_contribution: medium
      - module: studio
        type: studio
        fields_added: 3
        risk_contribution: high
    total_extensions: 4
    severity: high                       # high si >=3 extensores no-estandar o studio presente
    notes: "Studio anade campos sobre modelo ya extendido por 3 modulos."

  - model: account.move
    extended_by:
      - module: indaws_account
        type: indaws_core
      - module: cliente_x_factura
        type: custom
    total_extensions: 2
    severity: medium
```

`severity`: `low` (2 extensores, sin Studio) | `medium` (3 extensores o 2 + Studio) | `high` (>=4 o cualquier numero con Studio sobre modelos criticos: sale.order, account.move, stock.picking, mrp.production, res.partner, product.template).

## SNAPSHOT.md

Resumen humano. Es lo primero que cualquier agente lee.

```markdown
# Inventory Snapshot

**Last scan**: 2026-04-25 08:30:00 UTC
**Odoo version**: 18.0
**Database**: production_xyz
**Global hash**: sha256:def789...
**Mode**: target indaws_sale_rappel

## Resumen

| Tipo | Total | Con cambios desde ultimo scan |
|------|-------|------------------------------|
| standard | 142 | 0 |
| oca | 18 | 1 (oca_account_invoice_x ↑ 18.0.1.2.0) |
| indaws_core | 47 | 2 |
| custom | 6 | 1 |
| studio | 1 (28 fields, 3 server actions) | true |
| **Total instalados** | **214** | |

## Delta vs scan anterior (2026-04-22)

- **Nuevos instalados (1)**: cliente_x_extra_report
- **Actualizados (3)**: indaws_sale (18.0.2.0.1 → 18.0.2.0.3), oca_account_invoice_x, indaws_sale_rappel
- **Desinstalados (0)**: ninguno
- **Studio**: +2 campos en sale.order, +1 server action

## Top-N modulos por riesgo

1. `cliente_x_sale_custom` — risk: high — toca sale.order + account.move, sin tests
2. `indaws_purchase_stock_picking_invoice` — risk: high — modulo bridge crítico
3. `indaws_sale_rappel` — risk: medium — depende de cron + estado de facturas
4. ...

## Colisiones criticas

- `sale.order` — 4 extensores (3 modulos + Studio) — severity: high
- `account.move` — 3 extensores — severity: medium

## Modulos sin scan posible

(Solo si scan_source=remote_only o hash=unknown)

- `cliente_x_obscure` — sin acceso al filesystem del addons_path
```

## Convenciones

- Todos los YAML: sangria 2 espacios, sin tabs, encoding UTF-8.
- Fechas: ISO-8601 UTC.
- Hashes: `sha256:` prefix obligatorio.
- Nombres de modelo: notacion punto Odoo (`sale.order`, no `SaleOrder`).
- Lists vs maps: top-level es siempre map para `studio.yaml` y `collisions.yaml`. `modules.yaml` es lista directa (no envuelta en map) para iteracion eficiente.
