<!-- verified: v18.0 @ 2026-06 (account_asset EE; anclas verificadas vs fuente) -->
# Account Assets (`account_asset`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (ASG*).

**ASCAP1. ¿Activos fijos con amortización?** — **NATIVO** · media · `account.asset` (estados draft/open/paused/close, `depreciation_move_ids`).

**ASCAP2. ¿Métodos de amortización (lineal/degresivo/mixto)?** — **NATIVO** · baja · `account.asset.method` (`linear` / `degressive` / `degressive_then_linear`).

**ASCAP3. ¿Generar el activo automáticamente desde una cuenta/factura?** — **NATIVO** · media · `account.account.create_asset` (`no`/`draft`/`validate`) + `asset_model_ids`; un asset en estado `model` actúa de plantilla.

**ASCAP4. ¿Tabla de asientos de amortización automáticos?** — **NATIVO** · media · `depreciation_move_ids` (`account.move`) generados por `compute_depreciation_board` + `_post`.

**ASCAP5. ¿Venta/disposición del activo (pérdida/ganancia)?** — **NATIVO** · media · `set_to_close` + wizard `asset_modify` (`dispose`/`sell`): calcula `net_gain_on_sale` con `gain_account_id`/`loss_account_id`.

**ASCAP6. ¿Modificar/revalorizar el activo?** — **NATIVO** · media · `asset_modify` (`modify`/"Re-evaluate"); incremento bruto vía `children_ids`/`gross_increase_value`.

**ASCAP7. ¿Prorrateo (prorata temporis)?** — **NATIVO** · baja · `account.asset.prorata_computation_type` (`none`/`constant_periods`/`daily_computation`) + `prorata_date`.

**ASCAP8. ¿Duración en meses/años?** — **NATIVO** · baja · `method_number` (nº de amortizaciones) + `method_period` (`'1'` Months / `'12'` Years).

**ASCAP9. ¿Ingresos/gastos diferidos con el mismo motor?** — **NATIVO** · media · `account.asset` es "Asset/Revenue Recognition"; gestiona deferred revenue invirtiendo importes (no hay `asset_type` separado en v18; el modelo es polivalente).

**ASCAP10. ¿Pausar y reanudar la amortización?** — **NATIVO** · media · estado `paused` + `pause`/`resume_after_pause` (wizard `pause`/`resume`, con `paused_prorata_date`/`asset_paused_days`).
