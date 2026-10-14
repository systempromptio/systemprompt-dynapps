## CADENA 15: Ciclo de Vida de Activo Fijo
**ID:** CHAIN_ASSET_LIFECYCLE
**Apps:** Contabilidad (Enterprise)
**Descripcion:** Ciclo completo de un activo fijo: creacion desde factura de compra, generacion del activo, depreciacion progresiva y disposicion final del activo.

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| account.move.action_post (con asset) | account.move | account.asset | `search_records.sh account.asset '[["original_move_line_ids.move_id","=",<invoice_id>]]'` |
| account.asset depreciate | account.asset | account.move (depreciacion) | Campo depreciation_move_ids en el asset |
| account.asset dispose | account.asset | account.move (disposicion) | Ultimo move en depreciation_move_ids |

### Paso de Variables Entre Flujos
- `{chain.invoice_id}` — ID de la factura de compra (in_invoice)
- `{chain.asset_id}` — ID del account.asset generado
- `{chain.asset_name}` — Nombre del activo
- `{chain.depreciation_move_ids}` — IDs de los asientos de depreciacion
- `{chain.dispose_move_id}` — ID del asiento de disposicion

### Precondiciones
- Modulo `account_asset` instalado (Enterprise)
- Modelo de activo (account.asset.group) configurado con metodo de depreciacion, duracion y cuentas
- Cuenta de activo, cuenta de depreciacion y cuenta de gastos configuradas
- Producto con categoria que tiene asset_category_id asignada (o configuracion manual en linea de factura)
- Diario de activos (o diario de compras/miscelaneos)

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | INVOICE_CREATE | Crear account.move tipo 'in_invoice' con linea que tenga asset_category o cuenta de activo | state=draft |
| 2 | INVOICE_POST | `execute_method.sh account.move action_post '[[<invoice_id>]]' '{}'` | state -> 'posted', activo generado automaticamente |
| 3 | — detectar asset — | `search_records.sh account.asset '[["original_move_line_ids.move_id","=",<invoice_id>]]'` | Activo creado, state='draft' |
| 4 | ASSET_CONFIRM | `execute_method.sh account.asset validate '[[<asset_id>]]' '{}'` | asset.state -> 'open' |
| 5 | ASSET_DEPRECIATE | `execute_method.sh account.asset action_post '[[<asset_id>]]' '{}'` (publica las lineas de depreciacion computadas) | Asientos de depreciacion posted |
| 6 | ASSET_DISPOSE | `execute_method.sh account.asset set_to_close '[[<asset_id>]]' '{"invoice_id": false}'` -> genera asiento de cierre | asset.state -> 'close', asiento de disposicion creado |

### Plan de Rollback (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 6 | ASSET_REOPEN | `execute_method.sh account.asset set_to_running '[[<asset_id>]]' '{}'` | Reabre el activo, pero el asiento de disposicion persiste -> debe cancelarse manualmente |
| 5 | DEPRECIATION_REVERSE | Los asientos de depreciacion posted deben revertirse con button_draft -> button_cancel individualmente | Solo si no tienen inalterable_hash |
| 4 | ASSET_DRAFT | `execute_method.sh account.asset set_to_draft '[[<asset_id>]]' '{}'` | Solo si state='open' y no hay depreciaciones posted |
| 3 | (eliminado con la factura) | | |
| 2 | INVOICE_DRAFT -> CANCEL -> DELETE | button_draft -> button_cancel -> delete_record | Verificar restricciones hash/audit; al cancelar la factura el activo se desvincula |
| 1 | (eliminada en paso 2) | | |

### Puntos de No Retorno
- **Paso 2 (invoice posted)**: Si journal tiene hash -> no se puede volver a draft
- **Paso 5 (depreciaciones posted)**: Cada asiento de depreciacion con hash es irreversible
- **Paso 6 (disposicion)**: Reversible tecnicamente con set_to_running, pero los asientos de cierre requieren tratamiento manual

---

