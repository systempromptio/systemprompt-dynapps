<!-- verified: v18.0 @ 2026-06 -->
# account_loans Gotchas — Odoo 18.0 Enterprise — EE

Prefijo único: **LOANG**. Path raíz: `odoo_enterprise/18/account_loans`. Módulo Enterprise (OEEL-1), `auto_install: True`, depende de `account_asset`+`base_import`.

---

## Generación y posteo de asientos

**LOANG1: el préstamo NO tiene cron propio — depende del cron del core `account`**
`action_confirm` crea los moves futuros con `auto_post='at_date'` (`account_loans/models/account_loan.py:209`, `AccountLoan.action_confirm`) pero sólo postea los de fecha `<= hoy` (`account_loans/models/account_loan.py:284`, `AccountLoan.post_moves`). El resto los postea `account.ir_cron_auto_post_draft_entry` → `_autopost_draft_entries()` (`odoo18/addons/account/data/service_cron.xml:3-9`, `ir_cron_auto_post_draft_entry`). Si ese cron del core está desactivado, las cuotas nunca se contabilizan solas y el loan se queda en `running` indefinidamente. No hay log ni aviso en el loan.

**LOANG2: el asiento de pago se fecha siempre a FIN DE MES, no en `line.date`**
El payment move usa `date = line.date + relativedelta(day=31)` (`account_loans/models/account_loan.py:211`, `AccountLoan.action_confirm`). Aunque la línea del cuadro venza el día 5, el asiento se contabiliza el último día del mes. Esto descuadra la conciliación si el banco carga la cuota a mitad de mes y esperabas la fecha exacta de la línea.

**LOANG3: el signo del asiento de pago es contraintuitivo (debe en deuda LP)**
El payment move hace DEBE en `long_term_account_id` por el principal y HABER en `short_term_account_id` por la cuota total (`account_loans/models/account_loan.py:214-225`, `AccountLoan.action_confirm`). No es un asiento de "pago" clásico contra banco: refleja la amortización + el devengo del interés, dejando un saldo en la cuenta corto plazo que luego se concilia con el cargo bancario. Si esperabas un apunte contra la 572 directamente, no está: la conciliación bancaria es manual contra la cuenta short_term.

**LOANG4: reclasificación LT→ST sólo cubre 12 meses fijos y vive 1 solo día**
Por cada línea se reclasifica el principal de las **próximas 12 cuotas** de largo a corto plazo (`account_loans/models/account_loan.py:237`, `AccountLoan.action_confirm`, `line_ids[i+1:i+13]`) y al día siguiente se revierte (`account_loans/models/account_loan.py:267`, `AccountLoan.action_confirm`, `+relativedelta(days=1)`). El "12" está hardcoded; si tu definición de corto plazo no es 12 meses, no es configurable. La reclasificación sólo existe el último día del mes (entre el move y su reverse del día 1 siguiente), justo para el corte del reporte.

**LOANG5: la reversión NO usa `_reverse_moves()` — se crea a mano**
Por rendimiento, el move de reversión se construye manualmente con apuntes invertidos (`account_loans/models/account_loan.py:265-281`, `AccountLoan.action_confirm`) y luego se enlaza con `reversed_entry_id` (`account_loans/models/account_loan.py:292`, `AccountLoan.action_confirm`). Implicación: no pasa por la lógica estándar de reversión de Odoo (no hereda tax_tags, ni analytic, ni reconciliación automática). Si extiendes el reverse, hazlo en este bloque, no en `_reverse_moves`.

**LOANG6: la última línea NO genera reclasificación**
`if line == loan.line_ids[-1]: break` (`account_loans/models/account_loan.py:235-236`, `AccountLoan.action_confirm`). La última cuota sólo crea el payment move, no reclasificación (ya no hay 12 meses futuros). Correcto, pero si tu cuadro tiene una sola línea no habrá ningún asiento de reclasificación.

---

## skip_until_date

**LOANG7: `skip_until_date` salta líneas SILENCIOSAMENTE y afecta al saldo pendiente**
Al confirmar, las líneas con `date < skip_until_date` no generan ningún asiento (`account_loans/models/account_loan.py:202`, `AccountLoan.action_confirm`, `continue`). Además `_compute_outstanding_balance` resta su principal como si estuvieran pagadas (`account_loans/models/account_loan.py:164`, `AccountLoan._compute_outstanding_balance`). Si lo informas por error, perderás los asientos de esas cuotas y el saldo pendiente saldrá menor del real, sin warning. Pensado sólo para préstamos cuyas primeras cuotas ya se contabilizaron a mano. **Ojo a la comparación**: es estrictamente `<` en los 3 sitios (`account_loans/models/account_loan.py:202`, `AccountLoan.action_confirm`, `:164`, `account_loans/models/account_move.py:23`, `AccountMove._post`), pero el `help` del campo dice "up to this date (**included**)" (`account_loans/models/account_loan.py:56`, `class AccountLoan`). Es decir, una línea con `date == skip_until_date` SÍ genera asiento y NO se considera saltada, contra lo que sugiere el texto de ayuda. Para incluir la cuota de esa fecha hay que poner `skip_until_date` un día después.

**LOANG8: el cierre automático cuenta las líneas skipped como pagadas**
`account.move._post` cierra el loan cuando todas las líneas están `is_payment_move_posted` **o** son anteriores a `skip_until_date` (`account_loans/models/account_move.py:23`, `AccountMove._post`). Una línea skipped nunca tendrá payment move, pero igual cuenta para cerrar. Coherente con LOANG7 pero hay que tenerlo presente al auditar.

---

## Validaciones al confirmar

**LOANG9: confirmación bloqueada si el cuadro no cuadra al céntimo con las cabeceras**
`action_confirm` exige `amount_borrowed_difference==0`, `interest_difference==0` y `duration_difference==0` (`account_loans/models/account_loan.py:182-192`, `AccountLoan.action_confirm`). Es decir: `sum(line.principal)` debe igualar `amount_borrowed`, `sum(line.interest)` el `interest`, y `len(line_ids)` la `duration`, todo redondeado a la moneda. Si importas un cuadro de un banco con redondeos distintos, la confirmación falla con UserError. Hay que ajustar la última línea o las cabeceras.

**LOANG10: `is_wrong_date` exige que `loan.date` sea <= primera línea**
`_compute_is_wrong_date` (`account_loans/models/account_loan.py:127-130`, `class AccountLoan`) marca error si `loan.date` está vacío o si alguna línea tiene `date < loan.date`. La "Loan Date" es la fecha de disposición, debe ser anterior o igual al primer vencimiento. Importar un cuadro y olvidar fijar `date` bloquea la confirmación.

---

## Cuadro de amortización (líneas)

**LOANG11: corto/largo plazo se calcula por posición (12 líneas), NO por fecha real**
`_compute_theoretical_balances` (`account_loans/models/account_loan_line.py:60-65`, `class AccountLoanLine`) toma las líneas futuras y parte `[:12]` (corto) / `[12:]` (largo) por ÍNDICE, asumiendo cadencia mensual. Si tu cuadro es trimestral o irregular, las "12 líneas" no equivalen a 12 meses y la clasificación corto/largo será incorrecta (mostrará como corto plazo cuotas que vencen a >1 año).

**LOANG12: `outstanding_balance` de la línea es TEÓRICO, el del loan es REAL**
`account.loan.line.outstanding_balance` se calcula sobre el cuadro (`amount_borrowed - sum principal hasta la fecha`, `account_loans/models/account_loan_line.py:55-58`, `class AccountLoanLine`), independiente de lo posteado. En cambio `account.loan.outstanding_balance` se basa en asientos posteados (`account_loans/models/account_loan.py:158-166`, `class AccountLoan`). Pueden divergir si hay cuotas vencidas sin postear. No confundir al reconciliar.

**LOANG13: `payment`, `long_term_/short_term_theoretical_balance` están stored "para pivot"**
Estos computados están `store=True` con el comentario `# stored for pivot view` (`account_loans/models/account_loan_line.py:24`, `AccountLoanLine.payment` y `account_loans/models/account_loan_line.py:33`, `AccountLoanLine.long_term_theoretical_balance` y `account_loans/models/account_loan_line.py:38`, `AccountLoanLine.short_term_theoretical_balance`). Si modificas el cuadro vía SQL/ORM masivo sin disparar el recompute, el pivot de análisis (`action_view_account_loans_analysis`) mostrará valores obsoletos.

---

## Cancelación / borrado / audit trail

**LOANG14: cancelar o borrar usa `_unlink_or_reverse()` — moses posteados se REVIERTEN, no se borran**
`action_cancel`, `action_set_to_draft` y el `ondelete` del loan llaman `_unlink_or_reverse()` (`account_loans/models/account_loan.py:393`, `AccountLoan.action_cancel` y `account_loans/models/account_loan.py:397`, `AccountLoan.action_set_to_draft` y `account_loans/models/account_loan.py:425`, `AccountLoan._unlink_loan`). Los moves draft se borran, pero los posteados se revierten (asientos de reversión adicionales). Con audit trail activo no se borra nada. Esto genera moves extra en la contabilidad al resetear un préstamo.

**LOANG15: tras reconfirmar con audit trail, hay varios payment moves por línea**
`_compute_is_payment_move_posted` contempla que, con audit trail, una línea pueda tener >1 move de pago (tras cerrar/cancelar y reconfirmar). Filtra el que NO tiene `reversal_move_ids` (`account_loans/models/account_loan_line.py:78-79`, `AccountLoanLine._compute_is_payment_move_posted`). Si tu código cuenta `generated_move_ids` crudo para saber si está pagado, contarás de más.

**LOANG16: borrar el loan hace `ondelete='cascade'` desde el move generado**
`account.move.generating_loan_line_id` tiene `ondelete='cascade'` (`account_loans/models/account_move.py:14`, `class AccountMove`). Pero el flujo de borrado del loan primero llama `_unlink_or_reverse()` (`account_loans/models/account_loan.py:422-426`, `class AccountLoan`). No borres `account.loan.line` directamente por SQL: dejarías moves huérfanos sin línea generadora.

---

## Bridges / config

**LOANG17: `expense_account_id` no admite cuentas de tipo `income` ni `asset` por domain**
Domain `[('account_type', 'in', ('expense', 'expense_depreciation'))]` (`account_loans/models/account_loan.py:66`, `class AccountLoan`). En ES la cuenta de gasto financiero (662) es de tipo expense, OK. Pero si tu plan mapea los intereses a una cuenta con otro `account_type`, no aparecerá en el selector. (En v19 se añade `expense_other` al domain — ver v19_changes.md.)

**LOANG18: el vínculo préstamo↔activos es indirecto vía `asset_group_id`**
`account.loan.linked_assets_ids` NO es un campo directo: es `asset_group_id.linked_asset_ids` (`account_loans/models/account_loan.py:168-172`, `class AccountLoan`). Y `account.asset.linked_loans_ids` es related de `asset_group_id.linked_loan_ids` (`account_asset.py`). Para vincular un préstamo a un activo, ambos deben compartir el mismo `account.asset.group`. No hay relación M2M directa préstamo-activo.

**LOANG19: `journal_id` restringido a `type='general'`**
Domain `[('type', '=', 'general')]` (`account_loans/models/account_loan.py:71`, `class AccountLoan`). No puedes usar un diario tipo bank/cash/sale/purchase para los asientos del préstamo. Los asientos son misceláneos; la conciliación con banco se hace después contra la cuenta short_term.

**LOANG20: `loan_properties` depende de la definición en el diario**
`loan_properties = fields.Properties(definition='journal_id.loan_properties_definition')` (`account_loans/models/account_loan.py:74`, `class AccountLoan`). Si no fijas `journal_id` o el diario no tiene `loan_properties_definition` configurada, las properties no aparecen. Cambiar de diario cambia el conjunto de properties disponibles.

## Wizards (compute / close)

**LOANG21: el wizard de cierre sólo borra moves DRAFT posteriores a la fecha — los posteados quedan**
`account.loan.close.wizard.action_save` (`account_loans/wizard/account_loan_close_wizard.py:20`, `AccountCloseWizard.action_save`) hace `unlink()` sólo de los `generated_move_ids` con `generating_loan_line_id.date > self.date AND state == 'draft'`. Si cierras un préstamo a una fecha pasada, los pagos YA POSTEADOS posteriores a esa fecha NO se revierten ni se tocan: siguen en la contabilidad. El cierre es "administrativo" (cambia state a closed y postea mensaje), no deshace asientos posteados. Para deshacer posteados usa cancel/set-to-draft (`_unlink_or_reverse`, LOANG14).

**LOANG22: `loan_type` lo decide la presencia de interés, no el usuario**
`account.loan.compute.wizard.action_save` (`account_loans/wizard/account_loan_compute_wizard.py:99`, `AccountLoanComputeWizard._get_loan_payment_schedule`): `loan_type='annuity' if self.interest_rate else 'linear'`. Si pones `interest_rate=0` esperando una cuota fija con interés cero, pyloan genera amortización **lineal** (principal constante), no una annuity de interés 0. No hay selector explícito de tipo de préstamo.

**LOANG23: `first_payment_date` por defecto es el día 1 del mes siguiente, y sólo se usa con `at_anniversary`**
El default de `first_payment_date` es `today.replace(day=1) + relativedelta(months=1)` (`account_loans/wizard/account_loan_compute_wizard.py:44`, `class AccountLoanComputeWizard`). Pero ese valor sólo se pasa a pyloan si `payment_end_of_month == 'at_anniversary'` (`:96`); con `end_of_month` (default) se ignora y pyloan calcula a fin de mes. Configurar `first_payment_date` "a mano" no tiene efecto en modo end_of_month.

## Reclasificación / reporte

**LOANG24: la reclasificación LT→ST y su reversión NO heredan analítica ni tax_tags**
El move de reversión se construye a mano (LOANG5) y el de reclasificación también: ambos sólo mueven principal entre 170 y 520. No llevan `analytic_distribution` ni `tax_tags`. Si necesitas que la deuda a corto/largo plazo aparezca desglosada por analítica en tus reportes, estos asientos no la aportan — sólo el payment move (que tampoco propaga analítica salvo que la añadas). El propósito de la reclasificación es exclusivamente el corte de balance corriente/no corriente del último día de mes.

**LOANG25: `_compute_is_wrong_date` tiene la dependencia INCOMPLETA — no recomputa al editar la fecha de una línea**
`@api.depends('date')` (`account_loans/models/account_loan.py:127`, `class AccountLoan`) pero el cuerpo lee `loan.line_ids.mapped('date')` (`:130`). Falta `line_ids.date` en el depends. Si cambias la fecha de una `account.loan.line` (p.ej. corriges un vencimiento) sin tocar `loan.date`, `is_wrong_date` NO se recalcula automáticamente. Como `action_confirm` lo usa de guard (`:180`), puedes acabar confirmando un cuadro con una línea anterior a `loan.date` si el flag quedó cacheado a False, o ver el aviso desfasado en la vista. Forzar recompute (recargar/escribir `date`) antes de fiarte del flag.

**LOANG26: re-ejecutar el wizard de cálculo APILA líneas, no las reemplaza**
`account.loan.compute.wizard.action_save` (`account_loans/wizard/account_loan_compute_wizard.py:132-141`, `AccountLoanComputeWizard.action_save`) hace `create()` de las nuevas `account.loan.line` sin borrar las existentes. Si abres el wizard de cómputo sobre un préstamo que ya tenía cuadro (calculado o importado) y guardas, las líneas nuevas se SUMAN a las viejas → cuadro duplicado, `duration_difference`/`amount_borrowed_difference` descuadrados y confirmación bloqueada. Hay que `action_reset` (borra `line_ids`, `account_loans/models/account_loan.py:374-376`, `AccountLoan.action_reset`) antes de recalcular. La importación de fichero sí limpia (`action_upload_amortization_schedule` hace `loan.line_ids.unlink()`, `:309`); el wizard de cómputo NO.

**LOANG27: `loan_term` del wizard está en AÑOS, no en meses ni en nº de cuotas**
`loan_term` (Integer) del compute wizard se pasa tal cual a `pyloan.Loan(loan_term=...)` y la validación usa `start_date + relativedelta(years=self.loan_term)` (`account_loans/wizard/account_loan_compute_wizard.py:81`, `AccountLoanComputeWizard._onchange_preview`, y `account_loans/wizard/account_loan_compute_wizard.py:94`, `AccountLoanComputeWizard._get_loan_payment_schedule`). Es la duración en **años**. Un préstamo a 5 años con cuota mensual genera ~60 líneas, no 5. No confundir con `account.loan.duration`, que sí es el nº de cuotas/líneas (`account_loans/models/account_loan.py:53`, `AccountLoan.duration`, validado contra `len(line_ids)`).

**LOANG28: `interest_difference` se guarda CON signo (sin `abs`), a diferencia de `amount_borrowed_difference`**
`_compute_amount_borrowed_difference` usa `abs(...)` (`account_loans/models/account_loan.py:136`, `AccountLoan._compute_amount_borrowed_difference`) pero `_compute_interest_difference` NO: `interest - round(sum(line.interest))` (`account_loans/models/account_loan.py:144`, `AccountLoan._compute_interest_difference`), puede ser negativo. La validación de `action_confirm` usa `float_compare(..., 0) != 0` (`:189`), así que el signo no afecta al bloqueo; pero si muestras `interest_difference` en una vista o lo usas en lógica propia, puede salir negativo cuando el cuadro tiene más interés que la cabecera. No asumas que es siempre >= 0.
