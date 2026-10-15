# Analysis Engine Reference

## Objetivo

Cruzar las tres fuentes de informacion para producir un plan de test completo y trazable:

```
Fuente 1: Studio Inventory (Phase 0)    → Lo que EXISTE en la BD
Fuente 2: Documento de testing (Phase 1) → Lo que el CLIENTE quiere probar
Fuente 3: Codigo fuente (Phase 2)        → Como FUNCIONA cada cosa

             ┌─────────────────┐
             │  Studio         │
             │  Inventory      │
             │  (BD real)      │
             └───────┬─────────┘
                     │
          ┌──────────┼──────────┐
          │    ANALYSIS ENGINE  │
          │                     │
          │  1. Cruce           │
          │  2. Trazado         │
          │  3. Plan de test    │
          │                     │
          └──────────┬──────────┘
                     │
    ┌────────────────┼────────────────┐
    │                │                │
┌───┴────┐    ┌──────┴──────┐   ┌────┴──────┐
│Documento│    │ Codigo      │   │ Plan de   │
│entrada  │    │ fuente      │   │ test FINAL│
└─────────┘    └─────────────┘   └───────────┘
```

---

## Paso 1: Indexar el Studio Inventory

Reorganizar el inventario en indices accesibles por clave:

### Indice por modelo

```json
{
  "crm.lead": {
    "custom_fields": ["x_studio_sector", "x_studio_categoria", ...],
    "computed_fields": ["x_studio_total_forecast"],
    "automations": ["Canal ventas auto", "Probabilidad etapa"],
    "server_actions": [],
    "studio_views": ["studio_customization.crm_lead_form_1"],
    "custom_rules": []
  },
  "sale.order": {
    "custom_fields": ["x_studio_field_60PRQ", ...],
    "computed_fields": ["x_studio_fecha_mod_lote"],
    "automations": ["Crear lineas SO", "Calculo fecha validez", "Incoterms tarifa"],
    "server_actions": ["Calculo fecha validez", "Incoterms desde tarifa"],
    "studio_views": [...],
    "custom_rules": []
  }
}
```

### Indice por tipo de personalizacion

```json
{
  "automations": {
    "Canal ventas auto": {
      "model": "crm.lead",
      "trigger": "on_write",
      "trigger_fields": ["partner_id"],
      "effect": "Cambia team_id segun equipo del partner",
      "tested_by": null
    },
    "Crear lineas SO": {
      "model": "sale.order",
      "trigger": "on_create",
      "trigger_fields": ["opportunity_id"],
      "effect": "Crea lineas de SO desde lineas de la oportunidad",
      "tested_by": null
    }
  }
}
```

El campo `tested_by` se rellena en el cruce (Paso 2).

---

## Paso 2: Cruce Documento vs Inventario

### 2.1 Parsear el documento en items testeables

Cada test del documento se convierte en un item estructurado:

```json
{
  "doc_id": "2.1",
  "doc_text": "Crear lead y verificar campos Studio: Sector, Categoria, Producto interes",
  "type": "FIELD_CHECK",
  "area": "03_crm",
  "priority": "A",
  "model": "crm.lead",
  "references": ["x_studio_sector", "x_studio_categoria", "x_studio_producto_interes"],
  "matched_inventory": [],
  "unmatched_references": [],
  "status": "pending_analysis"
}
```

### 2.2 Matching algorithm

Para cada item del documento:

```python
def match_document_item(doc_item, inventory_index):
    model = doc_item["model"]
    refs = doc_item["references"]  # campos, acciones, modelos mencionados

    matched = []
    unmatched = []

    for ref in refs:
        # Buscar en campos custom del modelo
        if ref in inventory_index.get(model, {}).get("custom_fields", []):
            matched.append({"type": "field", "name": ref, "found_in": "inventory"})

        # Buscar en automations
        elif any(ref in a["name"] or ref in a.get("trigger_fields", [])
                 for a in inventory_index.get(model, {}).get("automations", [])):
            matched.append({"type": "automation", "name": ref, "found_in": "inventory"})

        # Buscar en computed fields
        elif ref in inventory_index.get(model, {}).get("computed_fields", []):
            matched.append({"type": "computed", "name": ref, "found_in": "inventory"})

        # Buscar en codigo fuente (modulo custom)
        elif exists_in_source_code(ref, model):
            matched.append({"type": "code", "name": ref, "found_in": "source"})

        else:
            unmatched.append(ref)

    doc_item["matched_inventory"] = matched
    doc_item["unmatched_references"] = unmatched

    # Marcar items del inventario como cubiertos
    for m in matched:
        mark_inventory_item_covered(m, doc_item["doc_id"])

    return doc_item
```

### 2.3 Resultado del cruce

Tres listas:

#### Lista A: Tests del documento CON match en inventario (OK)
```
✓ 2.1 crm.lead: x_studio_sector → Existe (many2one, res.partner.industry)
✓ 2.1 crm.lead: x_studio_categoria → Existe (selection)
✓ 2.2 crm.lead: Accion "Canal ventas auto" → Existe (on_write, partner_id)
```

#### Lista B: Tests del documento SIN match (WARNINGS)
```
⚠ 2.5 crm.lead: x_studio_field_forecast_kg → NO encontrado en inventario
  Posibles causas:
  - El campo no se migro correctamente
  - Nombre tecnico incorrecto en el documento
  - El campo fue renombrado durante la migracion
  Accion: Preguntar al usuario
```

#### Lista C: Items del inventario NO cubiertos por el documento (GAPS)
```
☐ sale.order: x_studio_field_abc123 ("Observaciones internas") → No hay test
☐ sale.order: x_studio_field_def456 ("Ref. almacen") → No hay test
☐ base.automation "Eliminar Stock" en stock.quant → No hay test
☐ ir.cron "Actualizacion estados lote" → No hay test
```

### 2.4 Presentar al usuario

```markdown
## Resultado del Analisis

### Cobertura
- Tests del documento: 241
- Personalizaciones descubiertas: 380
- Tests con match: 215 / 241 (89%)
- Personalizaciones cubiertas: 195 / 380 (51%)
- Personalizaciones NO cubiertas: 185

### Warnings (26 tests sin match)
| # | Test | Referencia | Problema |
|---|------|-----------|----------|
| 2.5 | Forecast Kg en crm.lead | x_studio_field_forecast_kg | Campo no existe en BD |
| ... | ... | ... | ... |

### Gaps (185 personalizaciones sin test)
| Modelo | Tipo | Nombre | Label |
|--------|------|--------|-------|
| sale.order | field | x_studio_field_abc123 | Observaciones internas |
| sale.order | field | x_studio_field_def456 | Ref. almacen |
| stock.quant | automation | Eliminar Stock | - |
| ... | ... | ... | ... |

**¿Quieres que genere tests automaticos para los gaps?**
Opciones:
1. Generar tests para TODOS los gaps
2. Generar solo para gaps de prioridad Alta (campos required, automations activas)
3. Seleccionar manualmente cuales cubrir
4. Ignorar gaps y continuar solo con el documento
```

---

## Paso 3: Trazado de Flujos

### 3.1 Dependency Graph de personalizaciones

Construir un grafo de dependencias entre personalizaciones:

```
x_studio_sector (crm.lead)
  └── Usado por: Accion auto "Canal ventas" (trigger on_write)
        └── Efecto: Cambia team_id
              └── Afecta: Regla de seguridad (segregacion por equipo)
                    └── Impacta: Visibilidad de leads en CRM

x_crm.product.line (modelo custom)
  └── Vinculado a: crm.lead (one2many)
        └── Usado por: Accion auto "Crear lineas SO"
              └── Efecto: Crea sale.order.line desde oportunidad
                    └── Impacta: Importe del presupuesto
```

### Como construir el grafo

```python
def build_dependency_graph(inventory):
    graph = {}

    # 1. Automations dependen de trigger_fields
    for auto in inventory["automations"]:
        for field in auto["trigger_fields"]:
            add_edge(graph, field, auto["name"], "triggers")

    # 2. Computed fields dependen de depends
    for comp in inventory["computed_fields"]:
        for dep in comp["depends"]:
            add_edge(graph, dep, comp["name"], "computes")

    # 3. Related fields dependen de la cadena
    for field in inventory["all_fields"]:
        if field.get("related"):
            add_edge(graph, field["related"], field["name"], "related_to")

    # 4. Server actions afectan modelos target
    for action in inventory["server_actions"]:
        if action.get("crud_model_id"):
            add_edge(graph, action["name"], action["crud_model_id"], "creates/writes")

    # 5. One2many vinculan modelos
    for field in inventory["all_fields"]:
        if field["type"] == "one2many":
            add_edge(graph, field["model"], field["relation"], "has_lines")

    return graph
```

### 3.2 Trazar flujos desde el grafo

Un **flujo** es una cadena de acciones que atraviesa el grafo:

```python
def trace_flow(graph, starting_action, inventory):
    """
    Dado una accion inicial (ej: crear crm.lead),
    trazar todos los efectos encadenados.
    """
    flow = []
    visited = set()
    queue = [starting_action]

    while queue:
        current = queue.pop(0)
        if current in visited:
            continue
        visited.add(current)

        flow.append(current)

        # Buscar que se dispara desde current
        for edge in graph.get(current, []):
            if edge["type"] == "triggers":
                # Una automation se dispara
                auto = find_automation(inventory, edge["target"])
                flow.append(f"AUTO: {auto['name']} fires")
                # El efecto de la automation es el siguiente nodo
                queue.append(auto["effect_target"])

            elif edge["type"] == "computes":
                flow.append(f"COMPUTE: {edge['target']} recalculates")

            elif edge["type"] == "creates/writes":
                flow.append(f"CREATE/WRITE: {edge['target']}")
                queue.append(edge["target"])

    return flow
```

### Ejemplo de flujo trazado

```
Flujo: Lead to Cash con automatizaciones Pharmactive

1. CREATE crm.lead
   ├── Campos custom a rellenar: x_studio_sector, x_studio_categoria,
   │   x_studio_producto_interes, x_studio_cmo
   │
2. WRITE crm.lead.partner_id = Partner Europa
   ├── AUTO: "Canal ventas auto" fires
   │   └── EFFECT: team_id → Europa
   │       └── SECURITY: Regla segregacion equipo aplica
   │
3. WRITE crm.lead.stage_id = New
   ├── AUTO: "Probabilidad etapa" fires (si existe)
   │   └── EFFECT: probability recalculada
   │
4. CREATE x_crm.product.line (lineas de producto)
   ├── Campos: product_id, quantity, price
   │
5. ACTION: Crear cotizacion desde oportunidad
   ├── AUTO: "Crear lineas SO" fires
   │   └── EFFECT: sale.order.line creadas desde x_crm.product.line
   ├── AUTO: "Calculo fecha validez" fires
   │   └── EFFECT: validity_date = +6m o +2m segun categoria
   ├── AUTO: "Incoterms tarifa" fires
   │   └── EFFECT: incoterm_id desde pricelist
   │
6. ACTION: sale.order.action_confirm()
   ├── STANDARD: stock.picking creado
   ├── AUTO: "Fecha modif lote" fires (si se escribe lote)
   │   └── EFFECT: x_studio_fecha_modificacion_lote = now()
   │
7. ACTION: stock.picking.button_validate()
   ├── CHECK: maker_lot_required → lote fabricante obligatorio
   ├── EFFECT: lot state = "Quality pending"
   │
8. WRITE stock.production.lot (analisis)
   ├── Campos: x_studio_compuesto_1..5, x_studio_humedad, x_studio_densidad
   │
9. ACTION: lot.action_approve()
   ├── EFFECT: lot state = "Available"
   │
10. ACTION: sale.order._create_invoices()
    ├── AUTO: "Notificacion opcional" → NO notifica (indaws_mail_optional_follower)
    │
11. ACTION: account.move.action_post()
    ├── AUTO: "VeriFactu" → comunicacion AEAT (si activo)
    │
12. WIZARD: account.payment.register → cobro
    ├── EFFECT: invoice payment_state = paid/in_payment
```

### 3.3 Validaciones por paso del flujo

Cada paso del flujo trazado genera validaciones automaticas:

```python
def generate_validations(flow_step, inventory, graph):
    validations = []

    # 1. Si el paso tiene automations que se disparan
    for auto in get_triggered_automations(flow_step, graph):
        # Validar el efecto de cada automation
        validations.append({
            "type": "automation_effect",
            "description": f"Verificar que '{auto['name']}' se disparo",
            "check": auto["expected_effect"],
            "source": "Studio Inventory"
        })

    # 2. Si el paso escribe campos computed
    for comp in get_affected_computed(flow_step, graph):
        validations.append({
            "type": "computed_field",
            "description": f"Verificar que '{comp['label']}' se recalculo",
            "field": comp["name"],
            "depends_on": comp["depends"],
            "source": "Studio Inventory"
        })

    # 3. Si el paso crea registros (standard Odoo)
    for side_effect in get_standard_side_effects(flow_step):
        validations.append({
            "type": "record_created",
            "description": f"Verificar que {side_effect['model']} se creo",
            "model": side_effect["model"],
            "source": "Odoo Standard"
        })

    # 4. Si el paso tiene constraints (from code analysis)
    for constraint in get_constraints(flow_step):
        validations.append({
            "type": "constraint_respected",
            "description": f"Verificar constraint: {constraint['name']}",
            "source": "Code Analysis"
        })

    return validations
```

---

## Paso 4: Generar Plan de Test Final

### Estructura del plan

```json
{
  "plan": {
    "metadata": {
      "generated_at": "2026-03-07T10:30:00",
      "sources": {
        "document": "Pharmactive_Checklists_Testing_v18.doc",
        "inventory": "280 fields, 8 models, 4 automations, 12 server_actions",
        "code": "51 custom modules"
      },
      "coverage": {
        "document_tests": 241,
        "auto_generated_tests": 45,
        "total_tests": 286,
        "inventory_coverage": "78%"
      }
    },

    "phases": {
      "A_config": {
        "tests": [ ... ],
        "count": 17,
        "batch_safe": true,
        "estimated_time": "34s"
      },
      "B_fields": {
        "tests_by_model": {
          "crm.lead": {"fields": [...], "count": 15},
          "sale.order": {"fields": [...], "count": 22},
          ...
        },
        "total_count": 120,
        "batch_safe": true,
        "estimated_time": "120s"
      },
      "C_data_integrity": {
        "tests": [ ... ],
        "count": 18,
        "baselines_available": false,
        "action_needed": "Ask user for v15 counts"
      },
      "D_flows": {
        "flows": [
          {
            "id": "FLOW-001",
            "name": "Lead to Cash con automatizaciones",
            "steps": 12,
            "automations_involved": 5,
            "models_touched": ["crm.lead", "x_crm.product.line", "sale.order", "stock.picking", "stock.production.lot", "account.move"],
            "custom_validations": 18,
            "standard_validations": 8,
            "source": "document(2.31) + inventory_enriched"
          },
          ...
        ]
      },
      ...
    }
  }
}
```

---

## Paso 5: Enriquecimiento de tests del documento

Cada test del documento se ENRIQUECE con informacion del inventario y el codigo:

### Antes (solo documento)

```
Test 2.2: Asignar cliente con equipo Europa, verificar asignacion automatica del canal
Resultado esperado: El equipo de ventas cambia automaticamente a "Europa"
Origen: Accion auto: Canal ventas
Prioridad: A
```

### Despues (enriquecido)

```json
{
  "doc_id": "2.2",
  "type": "AUTOMATION_CHECK",
  "area": "03_crm",
  "priority": "A",
  "description": "Asignar cliente con equipo Europa, verificar asignacion automatica del canal",

  "automation_detail": {
    "name": "Canal ventas automatico",
    "db_id": 45,
    "model": "crm.lead",
    "trigger": "on_write",
    "trigger_fields": ["partner_id"],
    "filter_pre": "[]",
    "filter_post": "[('partner_id', '!=', False)]",
    "action_type": "code",
    "code": "for rec in records:\n  if rec.partner_id.team_id:\n    rec.team_id = rec.partner_id.team_id",
    "source": "Studio Inventory (base.automation id=45)"
  },

  "execution_plan": {
    "setup": [
      {"action": "search", "model": "res.partner", "domain": "[('team_id.name', '=', 'Europa')]", "purpose": "Find partner with Europa team"},
      {"action": "search", "model": "crm.lead", "domain": "[('id', '>', 0)]", "limit": 1, "purpose": "Find existing lead or create one"}
    ],
    "steps": [
      {"action": "write", "model": "crm.lead", "fields": {"partner_id": "$partner_id"}, "purpose": "Trigger automation"}
    ],
    "validations": [
      {"type": "field_check", "model": "crm.lead", "field": "team_id", "expected": "$europa_team_id", "source": "automation code analysis"},
      {"type": "timing", "max_ms": 2000, "reason": "Automation should fire immediately"}
    ]
  },

  "traceability": {
    "document_ref": "2.2",
    "inventory_ref": "base.automation id=45",
    "code_ref": null,
    "depends_on": ["1.4 (equipos de ventas existen)", "1.11 (acciones auto activas)"]
  }
}
```

---

## Resumen del Analysis Engine

```
Input:
  ├── Studio Inventory (380 items)
  ├── Documento (241 tests)
  └── Codigo fuente (51 modulos)

Processing:
  1. Indexar inventario por modelo y tipo
  2. Cruzar documento vs inventario
     ├── Match: 215 tests → enriquecidos con detalle de BD
     ├── Warnings: 26 tests → referencias no encontradas
     └── Gaps: 185 items → proponer tests automaticos
  3. Trazar flujos con dependency graph
     ├── Cadenas de automations
     ├── Campos computed y sus triggers
     └── Side effects estandar de Odoo
  4. Generar validaciones por paso
  5. Enriquecer cada test con detalle ejecutable

Output:
  └── Plan de test final
      ├── 286 tests (241 doc + 45 auto-generados)
      ├── Organizados en fases A-I
      ├── Cada test con execution_plan detallado
      ├── Validaciones trazables a su origen
      └── Dependency graph para ordenar ejecucion
```
