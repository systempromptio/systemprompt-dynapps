## CADENA 18: Flujo Inter-empresa
**ID:** CHAIN_INTERCOMPANY
**Apps:** Ventas + Compras + Multi-empresa
**Descripcion:** Flujo inter-empresa donde una venta en la empresa A genera automaticamente una orden de compra en la empresa B, con recepcion de mercancia en la empresa B.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| sale.order.action_confirm (intercompany) | sale.order (Empresa A) | purchase.order (Empresa B) | `search_records.sh purchase.order '[["origin","=","<SO_name>"]]'` con contexto de Empresa B |
| purchase.order.button_confirm (Empresa B) | purchase.order | stock.picking | `search_records.sh stock.picking '[["origin","=","<PO_name>"]]'` con contexto Empresa B |

### Paso de Variables Entre Flujos
- `{chain.company_a_id}` — ID de la empresa vendedora
- `{chain.company_b_id}` — ID de la empresa compradora
- `{chain.SO_id}` — ID del sale.order en Empresa A
- `{chain.SO_name}` — Nombre del pedido de venta
- `{chain.PO_id}` — ID del purchase.order en Empresa B
- `{chain.PO_name}` — Nombre de la orden de compra
- `{chain.picking_id}` — ID del picking de recepcion en Empresa B

### Precondiciones
- Configuracion inter-company activa: `res.company` con `rule_type='sale_purchase'` en ambas empresas
- Partner de cada empresa configurado como inter-company partner en la otra
- Producto disponible en ambas empresas (o plantilla compartida)
- Usuario con acceso a ambas empresas (multi-company)
- Warehouse configurado en Empresa B para recepciones
- Impuestos mapeados entre empresas (o sin impuestos para simplificar)

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | SALE_CREATE | Crear SO en Empresa A: `create_record.sh sale.order '{"partner_id": <partner_company_b>, "company_id": <company_a_id>, ...}'` | state=draft en Empresa A |
| 2 | SALE_CONFIRM | `execute_method.sh sale.order action_confirm '[[<SO_id>]]' '{}' ` (con contexto allowed_company_ids de Empresa A) | SO.state -> 'sale' |
| 3 | — detectar PO — | `search_records.sh purchase.order '[["origin","=","<SO_name>"],["company_id","=",<company_b_id>]]'` | PO generada automaticamente en Empresa B |
| 4 | PURCHASE_CONFIRM | `execute_method.sh purchase.order button_confirm '[[<PO_id>]]' '{}'` (con contexto Empresa B) | PO.state -> 'purchase' en Empresa B |
| 5 | — detectar picking — | `search_records.sh stock.picking '[["origin","=","<PO_name>"],["company_id","=",<company_b_id>]]'` | Picking recepcion en Empresa B |
| 6 | PICKING_VALIDATE | `execute_method.sh stock.picking button_validate '[[<picking_id>]]' '{}'` (con contexto Empresa B) | picking.state -> 'done' |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | PICKING_RETURN | Wizard stock.return.picking -> validate return (contexto Empresa B) | Devolucion en Empresa B |
| 5 | (limpieza) | Verificar return picking done | |
| 4 | PURCHASE_CANCEL | button_cancel en PO (contexto Empresa B) | Solo si no hay invoices |
| 3 | PURCHASE_DELETE | delete_record PO (contexto Empresa B) | Solo desde cancel |
| 2 | SALE_CANCEL | action_cancel con disable_cancel_warning (contexto Empresa A) | La PO en Empresa B NO se cancela automaticamente — debe cancelarse primero manualmente |
| 1 | SALE_DELETE | delete_record SO (contexto Empresa A) | Solo desde cancel/draft |

### Puntos de No Retorno
- **Paso 2 (SO confirmed)**: La PO se genera automaticamente en Empresa B — cancelar la SO en A no cancela la PO en B
- **Paso 6 (picking done)**: Solo devolucion, no cancelacion
- **Nota**: La cancelacion debe hacerse en orden: primero Empresa B (PO), luego Empresa A (SO). Invertir este orden puede dejar registros huerfanos

---

