<!-- verified: v18.0 @ 2026-06 (account_iso20022 + account_batch_payment EE; SDD = módulo aparte; anclas verificadas vs fuente) -->
# ISO20022 / SEPA Payments (`account_iso20022`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**, anclado a fuente. **EE**. Método: [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md); trampas en [`gotchas.md`](gotchas.md) (ISOG*). Motor de pago en [[payment]].

**ISOCAP1. ¿Generar fichero ISO20022/SEPA (pain.001)?** — **NATIVO** · baja · `account_batch_payment.py::_generate_export_file` → `account.journal.create_iso20022_credit_transfer` cuando el método es `sepa_ct`/`iso20022`.

**ISOCAP2. ¿Lote de pagos?** — **NATIVO** · baja · `account.batch.payment` con `payment_ids` (One2many) agrupa varios pagos.

**ISOCAP3. ¿SEPA Credit Transfer como método?** — **NATIVO** · baja · `account_payment_method.py::_get_payment_method_information` registra `'sepa_ct'` (`mode='multi'`, EUR).

**ISOCAP4. ¿Varias versiones de pain.001?** — **NATIVO** · media · `account.journal.sepa_pain_version` (`pain.001.001.03`, `.03.de`, `.austrian.004`, `.001.09`) + submódulos `_de/_at/_ch/_se`; namespace por `get_document_namespace`.

**ISOCAP5. ¿Validación IBAN/BIC?** — **NATIVO** · baja · `validate_batch` exige IBAN (compañía y pagos); `_get_cleaned_bic_code` valida el BIC contra el regex del XSD (UserError si no).

**ISOCAP6. ¿Descargar el XML?** — **NATIVO** · baja · `account.batch.payment.export_file` (Binary) + `export_batch_payment` adjunta el fichero (`message_post`).

**ISOCAP7. ¿Por diario bancario de la compañía?** — **NATIVO** · baja · `account.batch.payment.journal_id`; `_get_DbtrAcct`/`_get_Dbtr` usan `bank_account_id`/`company_id`.

**ISOCAP8. ¿Multi-divisa en el fichero?** — **PARCIAL** · media · ISO20022 genérico agrupa `PmtInf` por fecha **y divisa**; pero `sepa_ct` fuerza EUR (`validate_batch` rechaza no-EUR).

**ISOCAP9. ¿Domiciliación / adeudo SEPA (SDD)?** — **NO (módulo aparte)** · — · estos módulos solo generan transferencias (crédito); el SDD (pain.008) vive en `account_sepa_direct_debit`.

**ISOCAP10. ¿Propósito / referencia estructurada del pago?** — **NATIVO** · media · `_get_CtgyPurp` (CtgyPurp/Cd) + `_get_RmtInf` (referencia estructurada BE/CH/FI/NO/SE/ISO con `Strd`, fallback `Ustrd`).
