## CADENA 35: Venta Inter-empresa (Plantilla de Operaciones Paralelas)
**ID:** CHAIN_INTERCOMPANY_SALE
**Apps:** Ventas + Compras + Multi-empresa
**Descripcion:** Plantilla de flujo inter-empresa que muestra operaciones paralelas en dos empresas. La venta en Empresa A genera automaticamente una PO en Empresa B. Se documenta con marcadores de empresa para cada operacion, facilitando la ejecucion con cambios de contexto `allowed_company_ids`.
**Tiempo estimado:** 4-6 minutos

### Estructura de Contexto Multi-empresa

Cada paso debe incluir el contexto de empresa para operar correctamente:

```json
{
  "context": {
    "allowed_company_ids": ["{data.company_a_id}"],
    "force_company": "{data.company_a_id}"
  }
}
```

### Paso de Variables Entre Flujos
- `{data.company_a_id}` — ID de la empresa vendedora (definido en data section)
- `{data.company_b_id}` — ID de la empresa compradora
- `{data.partner_company_b}` — ID del partner que representa a la Empresa B
- `{data.partner_company_a}` — ID del partner que representa a la Empresa A
- `{chain.SO_id}` — ID del sale.order en Empresa A
- `{chain.SO_name}` — Nombre del SO
- `{chain.PO_id}` — ID del purchase.order auto-generado en Empresa B
- `{chain.PO_name}` — Nombre del PO
- `{chain.picking_out_id}` — ID del picking de salida (Empresa A)
- `{chain.picking_in_id}` — ID del picking de recepcion (Empresa B)

### Precondiciones
- Configuracion inter-company activa: ambas empresas con `rule_type='sale_purchase'`
- Partners cruzados: cada empresa registrada como partner en la otra
- Producto disponible en ambas empresas
- Usuario con `allowed_company_ids` incluyendo ambas empresas
- Almacenes configurados en ambas empresas

### Flujo de Ejecucion (operaciones paralelas por empresa)

| # | Empresa | Flujo Atomico | Accion | Verificacion |
|---|---------|--------------|--------|-------------|
| 1 | Cia A | SALE_CREATE | Crear SO con partner = Empresa B | state=draft |
| 2 | Cia A | SALE_CONFIRM | action_confirm (ctx: Cia A) | state=sale, PO auto-generada en Cia B |
| 3 | Cia B | PO_DETECT | `search_records.sh purchase.order '[["origin","=","<SO_name>"],["company_id","=",<company_b_id>]]'` | PO encontrada |
| 4 | Cia B | PURCHASE_CONFIRM | button_confirm (ctx: Cia B) | PO.state=purchase |
| 5 | Cia A | PICKING_OUT_DETECT | Buscar picking de salida en Cia A | Picking OUT detectado |
| 6 | Cia A | PICKING_OUT_VALIDATE | button_validate (ctx: Cia A) | picking.state=done |
| 7 | Cia B | PICKING_IN_DETECT | Buscar picking de recepcion en Cia B | Picking IN detectado |
| 8 | Cia B | PICKING_IN_VALIDATE | button_validate (ctx: Cia B) | picking.state=done |

### Ejemplo JSON con contextos de empresa

```json
{
  "flow_id": "CHAIN_INTERCOMPANY_SALE",
  "flow_name": "Venta Inter-empresa",
  "data": {
    "company_a_id": 1,
    "company_b_id": 2,
    "partner_company_b": 10,
    "product_id": 25
  },
  "steps": [
    {
      "step": "1", "action": "create", "model": "sale.order", "save_as": "so",
      "values": {"partner_id": "{data.partner_company_b}", "company_id": "{data.company_a_id}", "order_line": [[0, 0, {"product_id": "{data.product_id}", "product_uom_qty": 5}]]},
      "context": {"allowed_company_ids": ["{data.company_a_id}"]}
    },
    {
      "step": "2", "action": "execute", "model": "sale.order", "method": "action_confirm",
      "ids": ["{result.so}"],
      "context": {"allowed_company_ids": ["{data.company_a_id}"]}
    },
    {
      "step": "3", "action": "search", "model": "purchase.order",
      "domain": [["origin", "=", "{result.so.name}"], ["company_id", "=", "{data.company_b_id}"]],
      "fields": ["id", "name", "state", "origin"],
      "save_as": "po",
      "context": {"allowed_company_ids": ["{data.company_b_id}"]}
    },
    {
      "step": "4", "action": "execute", "model": "purchase.order", "method": "button_confirm",
      "ids": ["{result.po}"],
      "context": {"allowed_company_ids": ["{data.company_b_id}"]}
    },
    {
      "step": "5", "action": "search", "model": "stock.picking",
      "domain": [["origin", "=", "{result.so.name}"], ["company_id", "=", "{data.company_a_id}"], ["picking_type_code", "=", "outgoing"]],
      "fields": ["id", "name", "state"],
      "save_as": "picking_out",
      "context": {"allowed_company_ids": ["{data.company_a_id}"]}
    },
    {
      "step": "6", "action": "execute", "model": "stock.picking", "method": "button_validate",
      "ids": ["{result.picking_out}"],
      "context": {"allowed_company_ids": ["{data.company_a_id}"]}
    },
    {
      "step": "7", "action": "search", "model": "stock.picking",
      "domain": [["origin", "=", "{result.po.name}"], ["company_id", "=", "{data.company_b_id}"], ["picking_type_code", "=", "incoming"]],
      "fields": ["id", "name", "state"],
      "save_as": "picking_in",
      "context": {"allowed_company_ids": ["{data.company_b_id}"]}
    },
    {
      "step": "8", "action": "execute", "model": "stock.picking", "method": "button_validate",
      "ids": ["{result.picking_in}"],
      "context": {"allowed_company_ids": ["{data.company_b_id}"]}
    }
  ]
}
```

### Plan de Rollback (orden inverso — Cia B primero, luego Cia A)

| # | Empresa | Accion | Notas |
|---|---------|--------|-------|
| 8 | Cia B | PICKING_IN_RETURN | Devolucion picking recepcion (ctx: Cia B) |
| 7 | Cia B | (detectado) | |
| 6 | Cia A | PICKING_OUT_RETURN | Devolucion picking salida (ctx: Cia A) |
| 5 | Cia A | (detectado) | |
| 4 | Cia B | PURCHASE_CANCEL | button_cancel PO (ctx: Cia B) |
| 3 | Cia B | PURCHASE_DELETE | delete PO (ctx: Cia B) |
| 2 | Cia A | SALE_CANCEL | action_cancel con disable_cancel_warning (ctx: Cia A) |
| 1 | Cia A | SALE_DELETE | delete SO (ctx: Cia A) |

### Puntos de No Retorno
- **Paso 2 (SO confirm)**: PO auto-generada — cancelar SO no cancela PO
- **Paso 6/8 (pickings done)**: Solo devolucion
- **IMPORTANTE**: Rollback en orden estricto: Cia B primero, Cia A despues

### Notas para el Consultor
- El contexto `allowed_company_ids` es **obligatorio** en cada paso para operar en la empresa correcta
- La deteccion de PO (paso 3) puede necesitar un breve delay si el cron inter-company no es sincrono
- Si la generacion automatica de PO no ocurre, verificar `res.company.rule_type` == 'sale_purchase' en ambas empresas

---

