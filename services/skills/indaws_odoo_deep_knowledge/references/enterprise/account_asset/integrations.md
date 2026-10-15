# Account Asset — Integrations (Odoo 18.0 Enterprise) — EE

Cómo `account_asset` se conecta con el resto de Odoo. Verificado contra `odoo_enterprise/18/account_asset`.

---

## 1. account.move / account.move.line (bridge principal)

`models/account_move.py` (398 líneas). Cada movimiento de depreciación, revaluación o disposal es un `account.move` con `asset_id` set.

### Campos añadidos a account.move
- `asset_id` (M2o, `ondelete='cascade'`) — el activo origen.
- `depreciation_value` (Monetary, computed) — importe depreciado del move. v18: `line_ids[1].debit/credit`; v19: `abs(line_ids[1].balance)` (ver v19_changes.md).
- `asset_move_type` (Selection: depreciation/sale/purchase/disposal/negative_revaluation/positive_revaluation).
- `asset_remaining_value` / `asset_depreciated_value` (cumulativos).
- `asset_value_change` (Boolean) — marca asientos de revaluación (no cuentan como depreciación normal).
- `asset_depreciation_beginning_date` (Date) — fecha base del periodo (clave en `_get_residual_value_at_date`).

### Generación del asiento de depreciación
`_prepare_move_for_asset_depreciation()`. El move se construye con:
- **Partner**: se preserva el de la factura origen sólo si hay un único partner en `original_move_line_ids` (si varios → sin partner).
- **Analytic**: `analytic_distribution` se propaga desde el activo (`account_asset/models/account_asset.py:241-252`, `AccountAsset._compute_analytic_distribution`, `_compute_analytic_distribution` agrega la distribución de las líneas origen). Al escribir `analytic_distribution` en un move draft de depreciación, se propaga a `line_ids` (`account_asset/models/account_asset.py:532-534`, `AccountAsset.create`).
- **Tax**: cuando `move.asset_id` está set, `_get_computed_taxes()` devuelve `self.tax_ids` directamente (NO recalcula por posición fiscal) — evita recomputar impuestos en asientos de depreciación.

### asset_move_type se limpia al desvincular
`account_asset/models/account_asset.py:498`, `AccountAsset.unlink`: cuando una línea deja de pertenecer a un asset, `move.asset_move_type = False`.

### Disposal / Sale move (`_get_disposal_moves`, `account_asset/models/account_asset.py:1115-1180`, `AccountAsset._cancel_future_moves`)
Estructura de líneas (`get_line` pone `balance = -amount`):
```
initial_amount    → account_asset_id (o cuenta única de las líneas origen)
depreciated_amount→ account_depreciation_id
[invoice lines]   → cuentas de las líneas de factura (sólo en venta)
difference        → gain_account_id si difference>0, else loss_account_id
```
- `difference = -initial_amount - depreciated_amount - invoice_amount`.
- **Líneas con `account` falsy se OMITEN** (`if account` en la comprehension, `account_asset/models/account_asset.py:1175`, `AccountAsset._get_disposal_moves`) → si `gain_account_id`/`loss_account_id` no están configuradas, la línea de diferencia desaparece y el asiento queda descuadrado (ASG11).
- Antes del disposal se crea la depreciación hasta la fecha (`_create_move_before_date(disposal_date)`, `account_asset/models/account_asset.py:1140`, `AccountAsset.get_line`).
- `move_type='entry'`, `asset_move_type='disposal'` (sin factura) o `'sale'` (con `invoice_line_ids`).

---

## 2. account.move.line (creación de activos)

- `asset_ids` (M2M vía `asset_move_line_rel`) — líneas origen del activo.
- `turn_as_asset()` — acción manual: abre el form de activo con las líneas seleccionadas como `original_move_line_ids`.
- `non_deductible_tax_value` (computed) — porción no deducible de impuestos (Bélgica 50% coches): se suma al `original_value` del activo.
- v19: nuevo `deductible_amount` (%) que pondera `related_purchase_value` y el non-deductible tax (ver v19_changes.md).

### Auto-creación desde factura (`_auto_create_asset`, account.move._post)
- Sólo líneas con `account.can_create_asset` y `account.create_asset != 'no'`.
- Filtros: `price_total > 0`, no línea de impuesto, no ya vinculada, no cuenta de grupo `asset` en facturas de cliente (evita doble creación con deferred revenue).
- `multiple_assets_per_line` → `int(quantity)` activos separados.
- N modelos en `account.asset_model_ids` → N activos por línea (multiplicativo, ASG19).
- `create_asset == 'validate'` → auto-valida (postea board completo).

---

## 3. account.account (config de activos)

- `create_asset` (Selection: no/draft/validate) — qué hacer al postear una línea en esta cuenta. v19: pasa a computed (auto `'draft'` si tiene `asset_model_ids`).
- `asset_model_ids` (M2M → account.asset state='model') — modelos plantilla aplicables.
- `multiple_assets_per_line`, `can_create_asset` (computed: True para `asset_fixed`/`asset_non_current`).

---

## 4. Analytic

- El activo es `analytic.mixin` → tiene `analytic_distribution`.
- Se hereda de las líneas origen (`_compute_analytic_distribution`, agregando distribuciones de todas las `original_move_line_ids`).
- Cada asiento de depreciación propaga esa distribución a sus `account.move.line` → la depreciación impacta cuentas analíticas (proyectos, centros de coste).
- El reporte Depreciation Schedule soporta filtrado analítico.

---

## 5. account_reports / Depreciation Schedule

`models/account_assets_report.py` (469 líneas). `AssetsReportCustomHandler` (`account.report.custom.handler`).
- SQL crudo (`self._cr.execute`) joinando `account_asset` + `account_move` (ASG23) → campos custom en el activo NO aparecen sin override del SQL.
- Hijos (gross increases, `parent_id != False`) se agregan bajo el padre (ASG24) — no son líneas separadas.
- Columnas: Characteristics (Acquisition Date, First Depreciation [removida en v19], Method, Duration/Rate), Assets (Opening/Additions/Disposals/Closing), Depreciation (idem), Book Value.
- Opciones: agrupar por cuenta o `asset_group_id`, jerarquía de cuentas, draft entries (`all_entries`), filtro por diario.
- v19: clase renombrada a `AccountAssetReportHandler`, display config movido a `_custom_options_initializer`.

---

## 6. account_asset_fleet (bridge opcional)

`odoo_enterprise/18/account_asset_fleet`. Vincula activos a vehículos de flota:
- `account.asset.vehicle_id` (M2o → fleet.vehicle, computed store, readonly=False) — se computa de `original_move_line_ids.vehicle_id`; si hay >1 vehículo distinto, queda vacío (`account_asset/models/account_asset.py:16-18`, `DAYS_PER_YEAR` del bridge).
- `account.move` (bridge, `account_asset/models/account_move.py:11-14`, `AccountMove`): al crear el move de depreciación, si `asset.vehicle_id`, propaga `vehicle_id` a las `line_ids` → la depreciación se imputa al vehículo en los reportes de flota.
- `fleet.vehicle.log.services` (inherit) — gastos de servicio pueden generar activos.

---

## 7. account_loans (bridge inverso)

`account_loans` depende de `account_asset` y vincula préstamos a activos **indirectamente vía `account.asset.group`** (no M2M directo):
- `account.asset.linked_loans_ids` = related de `asset_group_id.linked_loan_ids`.
- `account.loan.asset_group_id` → mismo grupo de activos.
- Caso ES: préstamo que financia un vehículo/maquinaria → mismo `asset_group` para reporting conjunto.
- Ver `enterprise/account_loans/integrations.md`.

---

## 8. asset.modify Wizard (TransientModel, wizard/asset_modify.py, 410 líneas)

| Action | Bridge contable |
|---|---|
| `dispose` | `_create_move_before_date` + disposal move (sin factura) |
| `sell` | igual + líneas de factura de cliente, calcula gain/loss vs precio venta |
| `modify` | crea hijo (gross increase, asiento DR gross/CR counterpart) si sube valor; asiento DR expense/CR depreciation (`asset_value_change=True`) si baja |
| `pause` | depreciación hasta fecha, `state='paused'` |
| `resume` | suma paused days, recomputa board |

Validaciones: no modificar si hay depreciaciones futuras posteadas (revertir primero), no antes de fiscal lock date, no vender activo con gross increases en curso.

---

## 9. Tabla resumen de modelos implicados

| Modelo | Rol | Fichero |
|---|---|---|
| `account.asset` | Activo / deferred rev/exp / modelo plantilla | `models/account_asset.py` (1230) |
| `account.asset.group` | Agrupación + puente con loans | `models/account_asset_group.py` (37) |
| `account.move` (ext) | Asientos de depreciación/disposal | `models/account_move.py` (398) |
| `account.move.line` (ext) | Líneas origen, turn_as_asset, non-deductible tax | `models/account_move.py` |
| `account.account` (ext) | create_asset, asset_model_ids | `models/account.py` (35) |
| `res.company` (ext) | gain_account_id / loss_account_id | `models/res_company.py` (20) |
| `AssetsReportCustomHandler` | Depreciation Schedule | `models/account_assets_report.py` (469) |
| `asset.modify` | Wizard dispose/sell/modify/pause/resume | `wizard/asset_modify.py` (410) |
| `account.asset` (fleet ext) | vehicle_id | `account_asset_fleet/` |
