## CADENA 38: Ciclo de Vida de Activo Fijo Completo con Reversion
**ID:** CHAIN_ASSET_LIFECYCLE_FULL
**Apps:** Contabilidad (Enterprise)
**Descripcion:** Ciclo completo extendido de un activo fijo: creacion manual, confirmacion, generacion de depreciaciones, publicacion de primer asiento, cierre/disposicion del activo, y reversion completa (set_to_running, cancelar depreciaciones, set_to_draft, eliminar).
**Tiempo estimado:** 2-4 minutos

### Deteccion de Registros Cascade

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| account.asset.validate | account.asset | account.move (depreciaciones) | Campo depreciation_move_ids en el asset |
| account.asset.set_to_close | account.asset | account.move (disposicion) | Ultimo move en depreciation_move_ids |
| account.asset.set_to_cancelled | account.asset | account.move (reversiones) | Reversal entries de las depreciaciones |

### Paso de Variables Entre Flujos
- `{chain.asset_id}` — ID del account.asset
- `{chain.depreciation_move_ids}` — IDs de los asientos de depreciacion
- `{chain.first_depreciation_id}` — ID del primer asiento de depreciacion

### Precondiciones
- Modulo `account_asset` instalado (Enterprise)
- Cuentas contables configuradas: cuenta de activo, cuenta de depreciacion acumulada, cuenta de gasto de depreciacion
- Diario de activos/miscelaneos

### Flujo de Ejecucion

| # | Flujo Atomico | Accion | Verificacion |
|---|--------------|--------|-------------|
| 1 | ASSET_CREATE | Crear account.asset con original_value, account_asset_id, account_depreciation_id, account_depreciation_expense_id, journal_id, method, method_number, method_period, acquisition_date | state='draft' |
| 2 | ASSET_CONFIRM | `execute_method.sh account.asset validate '[[<asset_id>]]' '{}'` | state -> 'open', depreciation_move_ids generados |
| 3 | — verificar depreciaciones — | `search_records.sh account.move '[["asset_id","=",<asset_id>]]' '["id","state","date","depreciation_value"]'` | Asientos de depreciacion creados |
| 4 | — verificar primer asiento — | El primer asiento se publica automaticamente al validar si su fecha <= hoy | Primer asiento state='posted' (si fecha <= hoy) |
| 5 | ASSET_DISPOSE | `execute_method.sh account.asset set_to_close '[[<asset_id>]]' '{"invoice_line_ids": [], "date": "YYYY-MM-DD"}'` | state -> 'close' |

### Plan de Rollback Completo (orden inverso)

| # | Flujo Atomico | Accion | Notas |
|---|--------------|--------|-------|
| 5 | ASSET_REOPEN | `execute_method.sh account.asset set_to_running '[[<asset_id>]]' '{}'` | state -> 'open', asiento de disposicion persiste |
| 4 | (asiento publicado) | Si hay hash -> punto de no retorno | Verificar con `journal_id.restrict_mode_hash_table` |
| 3 | ASSET_CANCEL | `execute_method.sh account.asset set_to_cancelled '[[<asset_id>]]' '{}'` | Revierte asientos posted, elimina drafts, state -> 'cancelled' |
| 2 | ASSET_DRAFT | `execute_method.sh account.asset set_to_draft '[[<asset_id>]]' '{}'` | state -> 'draft' |
| 1 | ASSET_DELETE | `delete_record.sh account.asset '[<asset_id>]'` | Solo desde state='draft' |

### Puntos de No Retorno
- **Paso 2 (validate)**: Los asientos de depreciacion se publican automaticamente si su fecha <= hoy. Si el diario tiene hash (restrict_mode_hash_table), esos asientos son irreversibles
- **Paso 5 (dispose)**: Reversible con set_to_running, pero requiere gestion manual de asientos de disposicion
- **set_to_cancelled**: Metodo mas seguro para reversion total — revierte asientos posted y elimina drafts automaticamente
- **Nota**: Si el activo fue creado desde una factura (original_move_line_ids), cancelar el activo NO cancela la factura

---

