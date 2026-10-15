<!-- verified: v18.0 @ 2026-06 (account_bank_statement_import + qif/ofx/csv/camt EE; OCR/sync = módulos aparte; anclas verificadas vs fuente) -->
# Bank Statement Import (`account_bank_statement_import`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (BSIG*).

**BSICAP1. ¿Importar extractos en varios formatos?** — **NATIVO** · baja · `account_journal.py::_get_bank_statements_available_import_formats` (QIF/OFX/CSV/CAMT, cada formato un módulo EE que extiende `_parse_bank_statement_file`).

**BSICAP2. ¿Mapeo de columnas para CSV?** — **CONFIG** · media · `account_bank_statement_import_csv` (hereda `base_import.import`); el usuario empareja columnas a mano en cada importación.

**BSICAP3. ¿Crear statement + líneas automáticamente?** — **NATIVO** · baja · `account_journal.py::_import_bank_statement` → `_create_bank_statements`.

**BSICAP4. ¿Detección de duplicados?** — **NATIVO** · baja · `account.bank.statement.line.unique_import_id` (SQL constraint); `_create_bank_statements` filtra repetidos con notificación.

**BSICAP5. ¿Detectar el diario por la cuenta del fichero?** — **NATIVO** · media · `account_journal.py::_find_additional_data` (+ `_statement_import_check_bank_account`) por `sanitized_acc_number`/IBAN.

**BSICAP6. ¿Multi-divisa en las líneas?** — **CONFIG (solo CAMT)** · media · `account_bank_statement_import_camt::_set_amount_in_currency` (requiere `group_multi_currency`); OFX/QIF/CSV no rellenan importe en divisa extranjera.

**BSICAP7. ¿Importar saldo inicial/final?** — **NATIVO** · baja · `balance_start`/`balance_end_real` cuando el formato lo provee.

**BSICAP8. ¿OCR de extracto en PDF/imagen?** — **NO (módulo aparte)** · — · existe `account_bank_statement_extract` (IAP), fuera de los módulos de import manual.

**BSICAP9. ¿Sincronización bancaria automática?** — **NO (módulo aparte)** · — · `account_online_synchronization` es un sistema distinto del import manual.

**BSICAP10. ¿Conciliación tras importar?** — **NATIVO** · baja · `_import_bank_statement` lanza `_cron_try_auto_reconcile_statement_lines` y abre el widget de conciliación.
