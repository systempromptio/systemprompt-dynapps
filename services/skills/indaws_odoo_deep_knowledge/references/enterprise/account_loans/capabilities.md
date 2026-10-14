<!-- verified: v18.0 @ 2026-07 (account_loans EE; anclas verificadas vs fuente) -->
# Préstamos (`account_loans`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**: "¿se puede X?" + **complejidad** + **origen**. Método en [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md)/[`integrations.md`](integrations.md); trampas en [`gotchas.md`](gotchas.md) (LOANG*).
> Veredicto: **EE** (`depends: account_asset` → Enterprise) · **NO**.
> Aplica a España sin problema: es genérico (préstamos bancarios/leasing), no localización fiscal. En CE no existe → el seguimiento de préstamos se haría a mano o con OCA.

**LOANCAP1. ¿Gestión de préstamos con cuadro de amortización?** — **EE** · media · `account_loans/models/account_loan.py` (`account.loan`, `amount_borrowed`/`interest`/`duration`) + `account_loan_line.py` (`account.loan.line`: `principal`/`interest`/`payment`/`outstanding_balance`) · Cada préstamo lleva su cuadro línea a línea.

**LOANCAP2. ¿Cálculo automático del cuadro (francés/lineal, interés compuesto)?** — **EE** · media · `wizard/account_loan_compute_wizard.py` (`interest_rate`, `loan_type` = `annuity`/`linear`, `compounding_method`, `_compute_preview`) · Genera el cuadro por sistema francés (anualidad) o lineal con previsualización.

**LOANCAP3. ¿Importar cuadro de amortización desde fichero?** — **EE** · media · `account_loan.py::action_upload_amortization_schedule` + `action_file_uploaded` (`base_import`) · Si el banco da su propio cuadro, se sube en vez de calcularlo.

**LOANCAP4. ¿Genera asientos (principal + interés) por vencimiento?** — **EE** · media · `account_loan.py::action_confirm` + `_compute_nb_posted_entries`; cada `account.loan.line` publica su asiento de pago (`_compute_is_payment_move_posted`) · Contabiliza el devengo por periodo.

**LOANCAP5. ¿Reclasificación automática corto/largo plazo?** — **EE** · media · `account_loan_line.py::long_term_theoretical_balance`/`short_term_theoretical_balance` (split 12 meses: `filtered_lines[:12]` corto, `[12:]` largo) + cuentas `long_term_account_id`/`short_term_account_id` · Separa la deuda a c/p y l/p como exige el PGCE.

**LOANCAP6. ¿Enlace préstamo ↔ activos financiados?** — **EE** · media · `account_loan.py::asset_group_id` (`account.asset.group`) + relación con `account.asset` · Vincula el préstamo a los activos que financia.

**LOANCAP7. ¿Cierre / cancelación anticipada del préstamo?** — **EE** · baja · `wizard/account_loan_close_wizard.py` + `account_loan.py::action_close`/`action_reset` · Cancela el cuadro pendiente y regulariza.
