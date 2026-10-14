# Flow Translation — Documento de cliente → YAML

## Cuándo se activa
Cuando el usuario proporciona un documento (texto, markdown, PDF, Excel) que describe flujos de negocio para testear.

## Proceso

1. **Leer documento completo** — Identificar todos los flujos
2. **Cargar translate_flow.py** — Usar como diccionario de patrones:
   - `FLOW_PATTERNS`: descripciones comunes → flujos atómicos
   - `VERB_MAP`: verbos es/en → acciones Odoo
   - `ENTITY_MAP`: entidades de negocio → modelos Odoo
   - `CHAIN_TEMPLATES`: cadenas predefinidas
3. **Para cada flujo**:
   a. Descomponer en pasos atómicos
   b. Mapear a atomic flow más cercano (`scripts/flows.py --list`)
   c. Si coincide con cadena conocida, usarla como base
   d. Adaptar datos del cliente
   e. Resolver dependencias (`{result.step_X.id}`)
4. **Generar YAML** siguiendo `flow_schema.md`
5. **Validar** con `validate_flow.py`
6. **Presentar** para aprobación

## Reglas

- Paso no mapea → PREGUNTAR, nunca asumir
- Flujo custom → usar `action: custom_method`
- Módulos custom → verificar existencia con odoo-pilot
- Siempre `cleanup_strategy: auto`
- Siempre contexto anti-email: `{tracking_disable: true, mail_notrack: true}`
- Datos del cliente con `{search: "domain"}`, nunca IDs hardcodeados

## Ejemplo

**Input:**
> "Crear pedido para Acme Corp con 10 unidades REF-001, confirmar, entrega parcial 5 unidades, facturar lo entregado, registrar pago."

**Output YAML:**
```yaml
flow:
  name: "Venta completa Acme Corp - entrega parcial"
  target_version: "18.0"
  cleanup_strategy: auto

  preconditions:
    modules: [sale_management, stock, account]
    data:
      partner: {model: res.partner, domain: [["name", "=", "Acme Corp"]]}
      product: {model: product.product, domain: [["default_code", "=", "REF-001"]]}

  steps:
    - id: step_1
      action: create
      model: sale.order
      context: {tracking_disable: true, mail_notrack: true}
      data: {partner_id: "{pre.partner.id}", order_line: [[0, 0, {product_id: "{pre.product.id}", product_uom_qty: 10}]]}

    - id: step_2
      action: confirm
      model: sale.order
      target: "{result.step_1.id}"

    - id: step_3
      action: deliver
      model: stock.picking
      target: "{result.step_2.picking_ids[0]}"
      params: {partial: true, quantities: {"{pre.product.id}": 5}}

    - id: step_4
      action: invoice
      model: sale.order
      target: "{result.step_1.id}"

    - id: step_5
      action: pay
      model: account.move
      target: "{result.step_4.invoice_ids[0]}"

  assertions:
    - step: step_2, field: state, expected: "sale"
    - step: step_3, model: sale.order, target: "{result.step_1.id}", field: delivery_status, expected: "partial"
    - step: step_5, model: account.move, target: "{result.step_4.invoice_ids[0]}", field: payment_state, expected: "paid"
```
