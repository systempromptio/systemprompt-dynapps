# Customization Overlay Reference

## Concepto

Un **overlay** es una capa de verificaciones custom que se inyecta dentro de un flujo estándar. El flujo base (190+ atómicos) no cambia — el overlay añade checks antes, durante y después de cada paso para verificar que las personalizaciones del cliente funcionan correctamente Y que no rompen el comportamiento estándar.

```
Flujo estándar:        CREATE SO → CONFIRM → PICKING → INVOICE → PAYMENT
                          │           │         │          │          │
Overlay custom:       [pre-check]  [auto-fx]  [field-fx] [auto-fx]  [post-check]
                      campos req   automation  lote custom EDI custom  saldos ok
                      defaults ok  team_id ok  state ok   envío ok    estándar ok
```

## Tres Capas de Personalización

El overlay detecta y testea personalizaciones de **cualquier origen**, no solo Studio:

| Capa | Origen | Cómo detectar | Ejemplos |
|------|--------|---------------|----------|
| **Studio/No-Code** | Odoo Studio UI | `ir.model.fields` con `x_studio_*`, `ir.ui.view` con key `studio_customization`, `base.automation` sin xmlid | Campos custom, automations, vistas modificadas |
| **Código custom** | Módulos Python del partner | `ir.module.module` con `author` ≠ Odoo/OCA, herencias en modelos, server actions con xmlid custom | Modelos heredados, métodos override, constraints |
| **Configuración avanzada** | Setup manual del consultor | Record rules sin xmlid, filtros default, secuencias modificadas, parámetros de sistema custom | Reglas de seguridad, workflows custom, params |

## Discovery Ampliado (Paso 0)

### Discovery de Studio (existente)
Ver `studio-discovery.md` — 10 tipos de consulta para personalizaciones en BD.

### Discovery de Código Custom (NUEVO)

```python
# 1. Módulos custom instalados (no son Odoo base ni OCA)
domain = [
    ["state", "=", "installed"],
    ["author", "not ilike", "Odoo"],
    ["author", "not ilike", "OCA"],
    ["author", "not ilike", "Odoo Community Association"],
    ["name", "not like", "l10n_"],  # excluir localizaciones
    ["name", "not like", "theme_"],  # excluir temas
]
custom_modules = search_read("ir.module.module", domain,
    ["name", "shortdesc", "author", "summary", "depends"])

# 2. Herencias de modelos desde módulos custom
# Para cada módulo custom, buscar sus modelos heredados
for mod in custom_modules:
    domain = [["modules", "like", mod["name"]]]
    models = search_read("ir.model", domain, ["model", "name", "field_id"])

# 3. Campos añadidos por módulos custom (NO x_studio_*)
# Campos que no son x_studio_ pero sí custom
domain = [
    ["modules", "not in", base_module_list],  # no Odoo base
    ["name", "not like", "x_studio_"],
    ["state", "=", "manual"],  # o campos base overrideados
]
custom_code_fields = search_read("ir.model.fields", domain,
    ["name", "model", "ttype", "modules", "compute", "depends", "store"])

# 4. Constraints custom (Python y SQL)
# Buscar _sql_constraints y @api.constrains en código fuente del partner
# (requiere acceso al filesystem del servidor o al repo del partner)

# 5. Server actions con xmlid de módulos custom
domain = [
    ["xml_id", "!=", False],
    ["xml_id", "not like", "base."],
    ["xml_id", "not like", "sale."],
    # ... excluir módulos estándar
]
custom_server_actions = search_read("ir.actions.server", domain,
    ["name", "model_id", "state", "code"])
```

### Discovery de Configuración Avanzada (NUEVO)

```python
# 1. Record rules custom (sin xmlid = creadas manualmente)
domain = [["xml_id", "=", False], ["active", "=", True]]
custom_rules = search_read("ir.rule", domain,
    ["name", "model_id", "domain_force", "groups",
     "perm_read", "perm_write", "perm_create", "perm_unlink"])

# 2. Parámetros de sistema custom
domain = [["key", "not like", "base.%"], ["key", "not like", "mail.%"]]
custom_params = search_read("ir.config_parameter", domain, ["key", "value"])

# 3. Secuencias modificadas
# Secuencias con padding, prefijo o sufijo custom
domain = [["prefix", "!=", False]]
sequences = search_read("ir.sequence", domain,
    ["name", "code", "prefix", "suffix", "padding", "number_next"])
```

## Inventario Unificado

El resultado del discovery ampliado produce un **inventario unificado** que clasifica cada personalización por origen:

```json
{
  "customization_inventory": {
    "discovered_at": "2026-03-27T10:00:00",
    "by_model": {
      "sale.order": {
        "studio": {
          "fields": ["x_studio_estado_s", "x_studio_ref_almacen"],
          "automations": ["Calculo fecha validez"],
          "views": ["studio_customization.sale_order_form_1"]
        },
        "code": {
          "modules": ["partner_custom_sale"],
          "fields": ["custom_delivery_notes"],
          "overrides": ["action_confirm (adds custom validation)"],
          "constraints": ["_check_delivery_date"]
        },
        "config": {
          "rules": ["Segregación por equipo comercial"],
          "sequences": ["SO con prefijo regional"]
        }
      }
    },
    "impact_map": {}
  }
}
```

## Impact Map: Personalización → Flujos Afectados

Después del discovery, mapear cada personalización a los flujos estándar que toca:

```python
def build_impact_map(inventory, standard_flows):
    """
    Para cada personalización, determinar qué flujos estándar afecta.
    """
    impact = {}

    for model, customs in inventory["by_model"].items():
        # Buscar flujos que usan este modelo
        affected_flows = [f for f in standard_flows
                          if model in f["models_touched"]]

        for custom_item in flatten_customs(customs):
            impact[custom_item["id"]] = {
                "item": custom_item,
                "affects_flows": [f["flow_id"] for f in affected_flows],
                "overlay_type": classify_overlay(custom_item),
                "risk": assess_risk(custom_item, affected_flows)
            }

    return impact
```

### Clasificación de Overlay

| Tipo overlay | Cuándo aplica | Qué verifica |
|-------------|---------------|-------------|
| `PRE_CHECK` | Campo required o default custom | Antes de crear: ¿tiene valor por defecto? ¿se puede crear sin él? |
| `POST_CHECK` | Campo computed o automation on_create | Después de crear: ¿se calculó correctamente? |
| `TRIGGER_CHECK` | Automation on_write | Después de write: ¿se disparó? ¿efecto correcto? |
| `SIDE_EFFECT` | Override de método (action_confirm, etc.) | Después de ejecutar: ¿se crearon registros extra? ¿campos extra populados? |
| `CONSTRAINT_CHECK` | _sql_constraints, @api.constrains | Intentar violación: ¿falla correctamente? |
| `SECURITY_CHECK` | Record rules custom | Con usuario limitado: ¿ve lo que debe? ¿no ve lo que no debe? |
| `INTEGRITY_CHECK` | Al final del flujo completo | Estado final: ¿saldos contables correctos? ¿stock coherente? ¿estándar intacto? |

### Evaluación de Riesgo

| Riesgo | Criterio | Acción |
|--------|----------|--------|
| **ALTO** | Override de action_confirm/action_post, automation con code que escribe en otros modelos, constraint que puede bloquear | Test obligatorio, marcar en informe |
| **MEDIO** | Campo required custom, computed stored con depends cross-model, record rule restrictiva | Test recomendado |
| **BAJO** | Campo informativo no-required, vista modificada sin attrs, filtro custom | Test opcional, verificar existencia |

## Generación del Overlay

### Paso 1: Para cada flujo estándar, obtener sus modelos

```python
flow = {
    "flow_id": "CHAIN_SALE_TO_CASH",
    "models_touched": ["sale.order", "stock.picking", "account.move", "account.payment"],
    "steps": [...]
}
```

### Paso 2: Cruzar con impact_map

```python
def generate_overlay(flow, impact_map):
    """
    Genera assertions overlay para un flujo estándar.
    """
    overlay_assertions = []

    for step in flow["steps"]:
        model = step["model"]

        # Buscar personalizaciones que afectan este modelo en este momento
        for custom_id, impact in impact_map.items():
            if model not in get_models(impact["item"]):
                continue

            overlay_type = impact["overlay_type"]

            if overlay_type == "PRE_CHECK" and step["action"] == "create":
                overlay_assertions.append({
                    "inject_at": f"before:{step['id']}",
                    "type": "PRE_CHECK",
                    "description": f"Verificar default de {impact['item']['name']}",
                    "check": {
                        "action": "fields_get",
                        "model": model,
                        "field": impact["item"]["name"],
                        "verify": "default_value_exists"
                    },
                    "source": impact["item"]["origin"],
                    "risk": impact["risk"]
                })

            elif overlay_type == "TRIGGER_CHECK" and step["action"] in ("execute", "update"):
                auto = impact["item"]
                overlay_assertions.append({
                    "inject_at": f"after:{step['id']}",
                    "type": "TRIGGER_CHECK",
                    "description": f"Verificar que automation '{auto['name']}' se disparó",
                    "check": {
                        "action": "read",
                        "model": model,
                        "target": f"{{result.{step['save_as']}}}",
                        "field": auto["effect_field"],
                        "expected": auto["expected_value"]
                    },
                    "source": auto["origin"],
                    "risk": impact["risk"]
                })

            elif overlay_type == "POST_CHECK":
                overlay_assertions.append({
                    "inject_at": f"after:{step['id']}",
                    "type": "POST_CHECK",
                    "description": f"Verificar campo computed {impact['item']['name']}",
                    "check": {
                        "action": "read",
                        "model": model,
                        "field": impact["item"]["name"],
                        "verify": "not_empty_and_correct_type"
                    },
                    "source": impact["item"]["origin"],
                    "risk": impact["risk"]
                })

            elif overlay_type == "SIDE_EFFECT":
                overlay_assertions.append({
                    "inject_at": f"after:{step['id']}",
                    "type": "SIDE_EFFECT",
                    "description": f"Verificar side-effect de override en {step['method']}",
                    "check": impact["item"]["verification"],
                    "source": impact["item"]["origin"],
                    "risk": "HIGH"
                })

    # SIEMPRE añadir INTEGRITY_CHECK al final del flujo
    overlay_assertions.append({
        "inject_at": "end",
        "type": "INTEGRITY_CHECK",
        "description": "Verificar que el flujo estándar no fue alterado por las personalizaciones",
        "checks": [
            {"description": "Estado final del registro principal correcto",
             "verify": "standard_final_state"},
            {"description": "Registros cascade estándar creados (picking, invoice, payment)",
             "verify": "standard_cascade_records"},
            {"description": "Campos estándar con valores esperados",
             "verify": "standard_field_values"},
            {"description": "No hay errores en mail.mail ni registros huérfanos",
             "verify": "no_orphan_records"}
        ],
        "source": "Standard Integrity",
        "risk": "HIGH"
    })

    return overlay_assertions
```

## Flujo Combinado: Estándar + Overlay

### Ejemplo: CHAIN_SALE_TO_CASH con overlay Pharmactive

```yaml
flow:
  base: CHAIN_SALE_TO_CASH
  client: Pharmactive
  overlay_source: "Studio(22 fields, 3 automations) + Code(2 modules) + Config(1 rule)"

  steps:
    # --- STEP 1: Crear SO ---
    - id: step_1_create_so
      base_action: create sale.order
      overlay:
        - type: PRE_CHECK
          desc: "x_studio_estado_s tiene default"
          risk: MEDIUM

        - type: PRE_CHECK
          desc: "custom_delivery_notes existe en el modelo"
          risk: LOW

    # --- STEP 2: Confirmar SO ---
    - id: step_2_confirm_so
      base_action: execute sale.order.action_confirm
      overlay:
        - type: TRIGGER_CHECK
          desc: "Automation 'Calculo fecha validez' se disparó"
          check: validity_date == today + 6months (si categ_id=6)
          risk: HIGH

        - type: SIDE_EFFECT
          desc: "Override action_confirm del módulo partner_custom_sale"
          check: custom_delivery_notes poblado desde partner
          risk: HIGH

        - type: POST_CHECK
          desc: "x_studio_estado_s cambió a valor esperado"
          risk: MEDIUM

    # --- STEP 3: Validar Picking ---
    - id: step_3_validate_picking
      base_action: execute stock.picking.button_validate
      overlay:
        - type: TRIGGER_CHECK
          desc: "Automation 'Fecha modif lote' se disparó"
          check: x_studio_fecha_modificacion_lote == today
          risk: MEDIUM

        - type: POST_CHECK
          desc: "Campos custom de lote (x_studio_humedad, x_studio_densidad) accesibles"
          risk: LOW

    # --- STEP 4: Crear y publicar factura ---
    - id: step_4_invoice
      base_action: execute account.move.action_post
      overlay:
        - type: TRIGGER_CHECK
          desc: "EDI custom no genera envío real (safe mode activo)"
          risk: HIGH

    # --- STEP 5: Registrar pago ---
    - id: step_5_payment
      base_action: wizard account.payment.register
      overlay: []  # Sin personalizaciones en este paso

    # --- INTEGRITY CHECK FINAL ---
    - id: integrity
      type: INTEGRITY_CHECK
      checks:
        - "SO state == 'sale' (estándar intacto)"
        - "Picking state == 'done' (estándar intacto)"
        - "Invoice payment_state == 'paid' (estándar intacto)"
        - "Stock quantities updated correctly"
        - "Account balances consistent"
        - "No orphan records created by automations"
        - "Secuencias no alteradas fuera del incremento esperado"

  summary:
    base_assertions: 8   # del flujo estándar
    overlay_assertions: 11  # de las personalizaciones
    total: 19
    high_risk: 4
    medium_risk: 3
    low_risk: 4
```

## Reporting del Overlay

El informe de ejecución distingue claramente entre resultados estándar y overlay:

```
CHAIN_SALE_TO_CASH — Pharmactive Overlay
═══════════════════════════════════════════

Step 1: Crear SO → sale.order(42)
  [STD] ✅ state='draft'
  [OVL] ✅ PRE_CHECK: x_studio_estado_s default='nuevo' (Studio)
  [OVL] ✅ PRE_CHECK: custom_delivery_notes existe (Code: partner_custom_sale)

Step 2: Confirmar SO → sale.order(42) → state='sale'
  [STD] ✅ state='sale', picking generado
  [OVL] ✅ TRIGGER: 'Calculo fecha validez' → validity_date=2026-09-27 (Studio)
  [OVL] ✅ SIDE_EFFECT: action_confirm override → delivery_notes poblado (Code)
  [OVL] ✅ POST_CHECK: x_studio_estado_s='confirmado' (Studio)

Step 3: Validar Picking → stock.picking(78)
  [STD] ✅ state='done', quantities moved
  [OVL] ✅ TRIGGER: 'Fecha modif lote' → 2026-03-27 (Studio)
  [OVL] ✅ POST_CHECK: campos lote accesibles (Studio)

Step 4: Factura → account.move(156) → state='posted'
  [STD] ✅ state='posted', balance correct
  [OVL] ✅ TRIGGER: EDI custom neutralizado por safe mode (Code)

Step 5: Pago → payment_state='paid'
  [STD] ✅ payment_state='paid'
  [OVL] — (sin overlay en este paso)

INTEGRITY CHECK:
  [STD] ✅ Flujo estándar completo sin alteraciones
  [OVL] ✅ Sin registros huérfanos de automations
  [OVL] ✅ Secuencias incrementadas correctamente (+1 SO, +1 INV, +1 PAY)
  [OVL] ✅ Saldos contables consistentes

═══════════════════════════════════════════
RESULTADO: 8/8 STD ✅ | 11/11 OVL ✅ | 0 ❌ | Riesgo: 4 HIGH passed
```

## Gotchas del Overlay

1. **Automations encadenadas** — Una automation puede disparar otra. El overlay debe seguir la cadena completa, no solo el primer efecto. Usar el dependency graph de `analysis-engine.md`.

2. **Override que cambia el return** — Si un módulo custom overridea `action_confirm` y cambia el valor de retorno (ej: devuelve un wizard), el flujo estándar puede romperse. Verificar tipo de retorno.

3. **Computed fields con store=False** — No se guardan en BD. Solo existen durante la request. El overlay debe leerlos inmediatamente después del write, no en un paso posterior.

4. **Record rules que filtran** — Una record rule custom puede hacer que el usuario API no vea registros que acaba de crear. Verificar permisos ANTES de ejecutar.

5. **Campos required custom sin default** — Si un campo `x_studio_*` es `required=True` sin default, TODOS los creates del flujo estándar fallarán. El overlay PRE_CHECK los detecta y propone valores.

6. **Automations on_time** — Se disparan por cron, no por write. No se pueden verificar en ejecución síncrona. Documentar como MANUAL check o ejecutar el cron manualmente.

7. **Módulos custom que instalan dependencias** — Un módulo del partner puede instalar módulos que cambian comportamiento estándar (ej: instalar `sale_stock_picking_invoice` cambia el flujo de facturación). Verificar `depends` de cada módulo custom.

8. **Personalizaciones mixtas Studio+Code** — Un módulo Python puede crear un campo que Studio después modifica (attrs, domain en vista). El overlay debe considerar AMBAS capas.
