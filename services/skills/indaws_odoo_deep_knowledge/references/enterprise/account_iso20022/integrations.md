# account_iso20022 — Integrations

Cómo `account_iso20022` (SCT/ISO20022) y `account_sepa_direct_debit` (SDD) se conectan con el resto del stack contable de Odoo. Todo es herencia; no hay modelos propios salvo `sdd.mandate`.

---

## Diagrama de bridges

```
                         ┌────────────────────────┐
                         │   account.batch.payment │  (account_batch_payment)
                         │   ORQUESTADOR del fichero│
                         └───────────┬─────────────┘
                                     │ _generate_export_file()
                  ┌──────────────────┴───────────────────┐
                  │                                       │
        payment_method_code in                  payment_method_code in
        [sepa_ct, iso20022, _se, _ch]           [sdd]
                  │                                       │
                  ▼                                       ▼
   journal.create_iso20022_credit_transfer   payment_ids.generate_xml
        (account_journal.py)                  (sepa_direct_debit/account_payment.py)
        ── pain.001 (SCT) ──                   ── pain.008 (SDD) ──
                  │                                       │
                  ▼                                       ▼
         account.payment (outbound)            account.payment (inbound)
         partner_bank_id (acreedor)            sdd_mandate_id ──► sdd.mandate
                  │                                       │            │
                  ▼                                       ▼            ▼
         res.partner.bank (IBAN/BIC)           res.partner.bank   res.partner
                  │                             (debtor IBAN)      (debtor)
                  ▼
         account.journal (bank) ── bank_account_id (company IBAN) + sepa_pain_version
                  │
                  ▼
         res.company (orgid, initiating party, LEI, creditor identifier)
```

Pipeline común: `account.move` (factura) → `account.payment.register` (wizard) → `account.payment` (con method line SEPA/SDD) → agrupados en `account.batch.payment` → `validate_batch` → `_generate_export_file` → XML descargable → subida manual al banco → conciliación al volver el extracto.

---

## 1. Bridge ↔ account_batch_payment

`account_batch_payment` es **dependencia dura** de ambos módulos (manifests). Es el contenedor que agrupa pagos y dispara la generación.

**Puntos de extensión** (todos overrides de `account.batch.payment`):
- `_get_methods_generating_files` → registra los codes SCT/ISO/SDD para que el batch sepa que produce fichero (`account_iso20022/models/account_batch_payment.py:35-38`, `AccountBatchPayment._get_methods_generating_files`; SDD `:132-135`).
- `_generate_export_file` → produce `{filename, file(base64)}` con el XML (`:116-131`; SDD `:252-262`).
- `validate_batch` → gates pre-fichero (IBAN, EUR, creditor identifier, mandatos válidos, fecha de cobro) (`:40-82`; SDD `:165-236`).
- `check_payments_for_errors` → lista de problemas no bloqueantes mostrados al usuario (sin bank account, importe excesivo, schemes mixtos) (`:84-114`; SDD `:302-318`).

**Campos añadidos al batch**:
- SCT: `iso20022_batch_booking`, `iso20022_charge_bearer` (`:13-25`).
- SDD: `sdd_required_collection_date`, `sdd_batch_booking`, `sdd_scheme`, `sdd_min_required_collection_date` (`account_sepa_direct_debit/account_batch_payment.py:20-31`, `AccountBatchPayment.sdd_required_collection_date`).

**Dashboard del journal** (de `account_batch_payment`): contadores `num_sepa_ct_to_send` (`account_iso20022/models/account_journal_dashboard.py:6-15`, `class account_journal`) y `sdd_payments_to_send_number` (`account_sepa_direct_debit/models/account_journal.py:26-34`, `AccountJournal._get_journal_dashboard_data_batched`), filtrando pagos `in_process`, `is_sent=False`, `is_matched=False`.

**Gotcha**: ISOG6 (un `PmtInf` por fecha/divisa); el batch puede contener pagos heterogéneos y el fichero se fragmenta.

---

## 2. Bridge ↔ account.payment

`account.payment` es donde vive cada transacción individual.

**Campos añadidos** (SCT, `account_payment.py`):
- `iso20022_uetr` (computed, solo code `iso20022`) — rastreo end-to-end SWIFT.
- `sepa_pain_version` (related a journal).

**Campos añadidos** (SDD, `sepa_direct_debit/account_payment.py`):
- `sdd_mandate_id` (computed store, `_compute_sdd_mandate_id` `:34-38`) — busca mandato usable según partner+date.
- `sdd_mandate_usable`, `sdd_mandate_scheme` (related).

**Hooks de método de pago**:
- `_get_method_codes_using_bank_account` / `_get_method_codes_needing_bank_account` (`account_iso20022/models/account_payment.py:19-29`, `class AccountPayment`): declaran que SCT/ISO necesitan cuenta bancaria → la UI muestra el campo `partner_bank_id`.
- `_get_payment_method_codes_to_exclude` (`:45-52`): oculta `sepa_ct` si moneda no SEPA → **ISOG1**.

**Constraints**:
- `_check_sepa_bank_account` (`:31-37`): journal debe tener IBAN para sepa_ct.
- `_check_sepa_currency` (`:39-43`): sepa_ct solo EUR.
- `_validate_sdd_mandate_id` (SDD `:82-86`): el mandato debe pertenecer al `commercial_partner_id`.

**`write` override SDD** (`account_sepa_direct_debit/models/account_payment.py:52-61`, `AccountPayment.write`): al pasar a `in_process`, propaga el `sdd_mandate_id` a las facturas conciliadas y cierra mandatos `one_off` → **ISOG12**.

---

## 3. Bridge ↔ bank journal (account.journal)

El journal de tipo banco es donde se configura todo y donde viven los métodos de generación XML.

**Configuración por journal**:
- `bank_account_id` (la cuenta IBAN de la company, emisor/acreedor).
- `sepa_pain_version` (SCT, `account_iso20022/models/account_journal.py:18-30`, `class AccountJournal`) → **ISOG5**.
- `debit_sepa_pain_version` (SDD, `account_sepa_direct_debit/models/account_journal.py:10-18`, `AccountJournal.debit_sepa_pain_version`) → pain.008.001.02 | .08, **ISOG11**.
- payment.method.lines: `_default_outbound_payment_methods` añade sepa_ct/iso20022 (`account_iso20022/models/account_journal.py:70-81`, `AccountJournal._default_outbound_payment_methods`); `_default_inbound_payment_methods` añade sdd (`account_sepa_direct_debit/models/account_journal.py:20-24`, `AccountJournal._default_inbound_payment_methods`).

**Métodos XML** (todos en `account.journal`, sobrecargables por país):
- `create_iso20022_credit_transfer_content` y los `_get_*` por nodo (Dbtr, CdtTrfTxInf, RmtInf, FinInstnId...).
- Overrides país: `_sepa_ct.py` (core SEPA), `_de.py`, `_at.py`, `_iso20022_se.py`, `_iso20022_ch.py`.

**Patrón de extensión país**: heredar `account.journal` y sobreescribir el `_get_<Node>` específico, ramificando por `payment_method_code` y/o `sepa_pain_version`, llamando a `super()` para el resto. Ejemplo mínimo (`account_iso20022/models/account_journal_sepa_ct_de.py:7-10`, `AccountJournal._skip_CdtrAgt`): solo override de `_skip_CdtrAgt` para forzar el CdtrAgt en pain DE.

---

## 4. Bridge ↔ SDD mandates (sdd.mandate)

Único modelo nuevo del dominio. El mandato es el consentimiento legal del deudor.

**Relaciones**:
- `sdd.mandate.partner_id` → `res.partner` (cliente/deudor).
- `sdd.mandate.partner_bank_id` → `res.partner.bank` (IBAN del deudor, debe ser tipo iban).
- `account.payment.sdd_mandate_id` → `sdd.mandate` (qué mandato autoriza el cobro).
- `sdd.mandate.payment_ids` / `paid_invoice_ids` (computed via `_read_group` sobre payments con `state in (in_process, paid)`, `account_sepa_direct_debit/models/sdd_mandate.py:139-163`, `SDDMandate._compute_from_moves`).

**Selección de mandato**: `_sdd_get_usable_mandate(company, partner, date)` (`account_sepa_direct_debit/models/sdd_mandate.py:123-137`, `SDDMandate._sdd_get_usable_mandate`) — primer mandato `active`, en rango de fechas, para company+partner. Se invoca desde `account.payment.get_usable_mandate` (`account_sepa_direct_debit/models/account_payment.py:115-120`, `AccountPayment.get_usable_mandate`) usando siempre `commercial_partner_id`.

**Ciclo de vida**: draft→active (action_validate, exige IBAN), active→{cancelled/revoked/closed}. Cron de expiración a 36 meses → **ISOG10**. one_off → cierre auto → **ISOG12**. revoked bloquea XML → **ISOG9**.

**Pre-notificación**: `_send_after_validation` (`account_sepa_direct_debit/models/account_batch_payment.py:264-300`, `AccountBatchPayment._send_after_validation`) envía email al deudor (template `email_template_sdd_pre_notification`) con IBAN anonimizado, ref de mandato, importe, creditor identifier y fecha de cobro. Requisito del rulebook SDD. Desactivable con config param `account_sepa_direct_debit.disable_sdd_pre_notification`.

---

## 5. Bridge ↔ res.partner.bank

La cuenta bancaria es el nodo IBAN/BIC tanto del emisor (journal) como del receptor (partner).

**Lo que el dominio lee de `res.partner.bank`**:
- `sanitized_acc_number` → tag `<IBAN>` (`account_iso20022/models/account_journal.py:241-242`, `AccountJournal._get_DbtrAcct`, `:325-326`).
- `acc_type == 'iban'` → decide IBAN vs `<Othr>` (`:238-240`).
- `bank_bic` / `bank_id.bic` → tag `<BIC>`/`<BICFI>` o `NOTPROVIDED` (`:309-319`, `_get_cleaned_bic_code` `:471-490`) → **ISOG4**.
- `acc_holder_name` → `<Nm>` del acreedor/deudor (`:289-291`; SDD `account_sepa_direct_debit/models/account_payment.py:249`, `AccountPayment.sdd_xml_gen_payment`).

**Override SDD** (`account_sepa_direct_debit/models/res_partner_bank.py:7-10`, `class ResPartnerBank`): `_unlink_except_linked_to_mandate` bloquea borrado si hay mandato activo → **ISOG18**.

**Validaciones IBAN en batch** (`account_iso20022/models/account_batch_payment.py:53-75`, `AccountBatchPayment.validate_batch`): journal y todos los partners deben tener IBAN para sepa_ct; RedirectWarning con la lista de pagos sin IBAN.

---

## 6. Otros bridges relevantes

### ↔ res.company / res.config.settings
`res.company` aporta la identidad del iniciador: `iso20022_initiating_party_name`, `iso20022_orgid_id/issr`, `iso20022_lei`, y (SDD) `sdd_creditor_identifier`. Expuestos en Ajustes vía `res.config.settings` (`res_config_settings.py`). **ISOG17** (creditor id), **ISOG20** (initiating name no recomputa).

### ↔ account.payment.register (wizard)
SCT override `_create_payment_vals_from_batch` (`account_iso20022/models/account_payment_register.py:8-21`, `AccountPaymentRegister._create_payment_vals_from_batch`): exige bank account al registrar pagos SEPA/ISO. **En v19 este fichero desaparece** (la lógica se movió, ver v19_changes).

### ↔ account.move (factura) [SDD]
El bridge SDD añade `account_invoice.py` y propaga `sdd_mandate_id` a las facturas conciliadas en `account.payment.write` (`account_sepa_direct_debit/models/account_payment.py:56-58`, `AccountPayment.write`). Permite ver desde la factura qué mandato la cobró.

### ↔ structured_reference (account.tools)
SCT usa `odoo.addons.account.tools.structured_reference` para detectar referencias estructuradas BE/CH/FI/NO/SE/ISO y generar `<Strd>` en vez de `<Ustrd>` (`account_iso20022/models/account_journal.py:342-371`, `AccountJournal._get_CdtrAcctIdOthr`, `:468-469`).

### ↔ hr_payroll (l10n_be) — pago de nóminas
Contexto `sepa_payroll_sala` / `l10n_be_hr_payroll_sepa_salary_payment` activa `InstrPrty=HIGH`, `CtgyPurp/Cd=SALA` y prefijo legal belga `/A/` en RmtInf (`account_iso20022/models/account_journal.py:181-220`, `AccountJournal._get_CtrlSum`, `account_iso20022/models/account_journal_sepa_ct.py:96-123`, `AccountJournal._get_RmtInf`). Es un bridge implícito vía context, sin dependencia de manifest.

---

## Modelos tocados (resumen)

| Modelo | SCT (account_iso20022) | SDD (account_sepa_direct_debit) |
|--------|------------------------|----------------------------------|
| `account.journal` | XML SCT/ISO, sepa_pain_version | XML SDD, debit_sepa_pain_version, dashboard |
| `account.batch.payment` | orquestación, batch_booking, charge_bearer | collection_date, scheme, pre-notif |
| `account.payment` | uetr, constraints | sdd_mandate_id, write→propagación |
| `account.payment.method` | 4 codes + currency rules | code `sdd` + `_get_sdd_payment_method_code` |
| `account.payment.register` | exige bank account | wizard mandato |
| `res.company` | orgid, initiating party, LEI | sdd_creditor_identifier |
| `res.partner` | LEI | — |
| `res.partner.bank` | (lectura) | unlink guard |
| `account.move` | — | sdd_mandate_id en factura |
| `sdd.mandate` | — | **modelo nuevo** |

---

## Config (mínima viable)

**SCT / ISO20022**:
1. Journal banco con `bank_account_id` IBAN válido.
2. Añadir payment.method.line `SEPA Credit Transfer` (auto si moneda EUR).
3. Company: `iso20022_initiating_party_name` (auto = nombre), opcional orgid/LEI.
4. Verificar `sepa_pain_version` que pide el banco (09 default, 03 legacy) → **ISOG5**.
5. Partners con IBAN + BIC en `res.partner.bank` → **ISOG4**.

**SDD**:
1. Company: `sdd_creditor_identifier` (lo da el banco) → **ISOG17**.
2. Journal con IBAN + `debit_sepa_pain_version`.
3. Por cliente: crear `sdd.mandate` (IBAN deudor, scheme CORE/B2B), validar a `active`, enviar al cliente.
4. Registrar pagos inbound con method `sdd`; el mandato se enlaza solo.
5. Batch con `sdd_required_collection_date` ≥ hoy + pre-notificación.

Config params relevantes:
- `account_iso20022.force_iso_20022_pain_09` — fuerza pain.09 + UETR en code genérico.
- `account_iso20022.local_instrument_code` — `LclInstrm/Cd` en ISO20022 genérico.
- `iso20022_ch_force_sepa` — Suiza trata SEPA como subconjunto → **ISOG15**.
- `account_sepa_direct_debit.disable_sdd_pre_notification` — desactiva email de pre-notificación.

---

## Gotchas que cruzan módulos

- **ISOG1** sepa_ct desaparece del wizard por moneda no-SEPA.
- **ISOG2** iso20022 genérico se auto-excluye en companies EUR.
- **ISOG4** sin BIC → `NOTPROVIDED`, banco puede rechazar.
- **ISOG6** un PmtInf por (fecha, divisa); batch mixto se fragmenta.
- **ISOG9 / ISOG12** mandatos revoked/one_off; gates en batch vs en XML.
- **ISOG17** sin creditor identifier no hay SDD.
- **ISOG18** no borrar res.partner.bank con mandato activo.

(Lista completa en `gotchas.md`, ISOG1–ISOG20.)

---

## Contexto España (Normas 34/19)

**SCT/SDD son el estándar de remesas para bancos españoles**. La correspondencia con las normas del Consejo Superior Bancario / AEB:
- **Norma 34.14** (transferencias/pagos) ↔ `pain.001` (SCT).
- **Norma 19.14** (adeudos/recibos domiciliados) ↔ `pain.008` (SDD).

**Odoo nativo (EE) NO implementa los cuadernos AEB 34.14/19.14 como tal.** Genera `pain.001`/`pain.008` ISO20022 estándar SEPA, que los bancos españoles aceptan vía su pasarela SEPA (la mayoría de bancos ES admiten pain.001.001.03 / pain.008.001.02 directamente). El `sdd_creditor_identifier` en España tiene formato `ESxx ZZZ <NIF>` y lo valida el Mod-97 de `res_company.validate_sdd_creditor_identifier` → **ISOG17**.

**Cuándo NO basta el nativo**:
- Si el banco exige el formato fijo de registro AEB (Norma 34.14/19.14 "clásica", texto plano de longitud fija) en vez de XML pain → requiere **OCA** o `l10n_es`.
- Anticipos de crédito / confirming españoles → OCA `l10n_es_account_banking_sepa_fsdd` ("FSDD — Anticipos de crédito", depende de `account_banking_sepa_direct_debit`).

**Alternativa OCA** (`OCA/v18/bank-payment`): `account_banking_sepa_credit_transfer` y `account_banking_sepa_direct_debit` (sobre `account_banking_pain_base` + `account_banking_mandate`). Stack distinto al nativo (usa `account.payment.mode` / `account.payment.order` en lugar de `account.batch.payment`). **No mezclar** el motor nativo EE con el OCA: son arquitecturas paralelas. Para clientes ES con requisitos de cuaderno AEB estricto o confirming, el camino habitual es OCA + `l10n-spain`.
