<!-- verified: v18.0 @ 2026-07 (account_saft EE + account_saft_import; anclas verificadas vs fuente) -->
# SAF-T (`account_saft`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**: "¿se puede X?" + **complejidad** + **origen**. Método en [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`core.md`](core.md)/[`integrations.md`](integrations.md); trampas en [`gotchas.md`](gotchas.md) (SAFTG*).
> Veredicto: **EE** (base, `depends: account_reports`) · **EE-país** (requiere `l10n_<país>_saft`) · **NO**.
> ⚠️ **No aplica a España.** SAF-T es obligatorio en AT/DK/LT/LU/NO/PL/RO; **España NO usa SAF-T** (va por SII + modelos AEAT, ver [[l10n_es]]). Relevante solo en clientes con filial/obligación en esos países.

**SAFTCAP1. ¿Genera fichero SAF-T (Standard Audit File for Tax)?** — **EE (base) / EE-país (fichero real)** · media · motor en `account_saft/models/account_general_ledger.py::_saft_prepare_report_values` sobre `account_reports`. El módulo base **no produce un fichero válido solo**: define la estructura y la rellena el módulo de país.

**SAFTCAP2. ¿Incluye libro mayor + apuntes en el SAF-T?** — **EE** · baja · `_saft_fill_report_general_ledger_accounts` + `_saft_fill_report_general_ledger_entries` · Cuentas y asientos del periodo en el XML.

**SAFTCAP3. ¿Detalle de impuestos en el SAF-T?** — **EE** · baja · `_saft_fill_report_tax_details_values` · Desglose de bases/cuotas por impuesto.

**SAFTCAP4. ¿Ledger de terceros (clientes/proveedores)?** — **EE** · baja · `_saft_fill_report_partner_ledger_values` · Saldos y movimientos por partner en el fichero.

**SAFTCAP5. ¿Funciona sin módulo de país?** — **NO** · — · requiere `l10n_at_saft`/`l10n_lt_saft`/`l10n_no_saft`/`l10n_ro_saft`/`l10n_pl_reports_account_saft`/`l10n_lu_reports`… que aportan namespace, versión de esquema y campos obligatorios del país. Sin ellos, el export es incompleto/rechazado.

**SAFTCAP6. ¿Importar datos contables desde ficheros SAF-T?** — **EE-hermano** · media · `account_saft_import/wizard/import_wizard.py` (`account.saft.import.wizard`, `_get_cleaned_namespace`) · Importación inversa (migración/auditoría) en módulo aparte; existen variantes país (`l10n_ro_saft_import`, `l10n_lt_saft_import`, `l10n_dk_saft_import`).

**SAFTCAP7. ¿Aplica a España (obligación AEAT)?** — **NO** · — · España no tiene régimen SAF-T; la obligación equivalente se cubre con SII (tiempo real), TicketBAI/Verifactu y modelos AEAT (ver [[l10n_es]] ESCAP1-11). No ofrecer SAF-T como cumplimiento fiscal español.
