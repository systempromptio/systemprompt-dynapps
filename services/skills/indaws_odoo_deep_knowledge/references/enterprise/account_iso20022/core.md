<!-- verified: v18.0 @ 2026-06 -->
# account_iso20022 — Core

Generación de ficheros XML de pagos según normas SEPA / ISO 20022. Cubre **SEPA Credit Transfer** (SCT, `pain.001`) e **ISO20022 genérico**. La variante **SEPA Direct Debit** (SDD, `pain.008`) vive en un módulo separado, `account_sepa_direct_debit`, también documentado aquí.

Histórico: el antiguo `account_sepa` se fusionó en `account_iso20022` desde v17. Por eso el `code` del payment method sigue siendo `sepa_ct` aunque el módulo se llame `iso20022`.

Módulo EE, `OEEL-1`. Manifest 18: `depends: ['account_batch_payment', 'base_iban']`, `post_init_hook: 'init_initiating_party_names'` (`account_iso20022/__manifest__.py:9`).

---

## Modelos (herencias, no modelos nuevos)

`account_iso20022` **no crea modelos nuevos**; todo es herencia. El único modelo nuevo del dominio (`sdd.mandate`) lo aporta `account_sepa_direct_debit`.

| Modelo | Fichero | Aporta |
|--------|---------|--------|
| `account.journal` | `models/account_journal.py` | toda la generación XML SCT/ISO20022, `sepa_pain_version`, `has_sepa_ct_payment_method` |
| `account.batch.payment` | `models/account_batch_payment.py` | orquestación del fichero, `iso20022_batch_booking`, `iso20022_charge_bearer`, validaciones |
| `account.payment` | `models/account_payment.py` | `iso20022_uetr`, `sepa_pain_version` (related), constraints EUR/IBAN |
| `account.payment.method` | `models/account_payment_method.py` | registra los 4 codes y sus currency/type |
| `account.payment.register` | `models/account_payment_register.py` | exige bank account al registrar pago |
| `res.company` | `models/res_company.py` | `iso20022_orgid_id`, `iso20022_orgid_issr`, `iso20022_initiating_party_name`, `iso20022_lei` |
| `res.partner` | `models/res_partner.py` | `iso20022_lei` + constraint LEI |
| `account.journal` (dashboard) | `models/account_journal_dashboard.py` | contador `num_sepa_ct_to_send` |

Journals por país (overrides de los nodos XML): `account_journal_sepa_ct.py` (SEPA core), `account_journal_sepa_ct_de.py` (Alemania), `account_journal_sepa_ct_at.py` (Austria), `account_journal_iso20022_se.py` (Suecia), `account_journal_iso20022_ch.py` (Suiza). No hay override para US.

---

## Payment methods (codes)

Definidos en `data/account_payment_method.xml`, todos `payment_type=outbound`:

- `sepa_ct` — SEPA Credit Transfer (`account_payment_method_sepa_ct`)
- `iso20022` — ISO20022 genérico (no-EUR)
- `iso20022_se` — Swedish ISO20022 (SEK)
- `iso20022_ch` — Swiss ISO20022 (CHF)

Restricciones de moneda/tipo en `_get_payment_method_information` (`account_iso20022/models/account_payment_method.py:10-31`, `AccountPaymentMethod._get_payment_method_information`):
- `sepa_ct` → `currency_ids = EUR`, `type=('bank',)`, `mode='multi'`
- `iso20022_se` → SEK, `iso20022_ch` → CHF
- `iso20022` → sin currency_ids (genérico)

**Gotcha de instalación**: `iso20022` lleva un domain extra en `_get_payment_method_domain` (`:33-44`) que lo **excluye** de journals en company EUR. Evita que se añada ISO20022 a journals donde ya entra SEPA al instalar el módulo.

`sepa_ct` además se excluye dinámicamente del wizard de pago si la moneda no está en la lista de divisas SEPA permitidas (`account_iso20022/models/account_payment.py:45-52`, `AccountPayment._get_payment_method_codes_to_exclude`: BGN, HRK, CZK, DKK, GIP, HUF, ISK, CHF, NOK, PLN, RON, SEK, GBP, EUR, XPF).

---

## Generación XML — SCT / pain.001 (Credit Transfer)

Punto de entrada: `account.batch.payment._generate_export_file` (`account_iso20022/models/account_batch_payment.py:116-131`, `AccountBatchPayment._generate_export_file`) → `journal.create_iso20022_credit_transfer(payment_dicts, code, batch_booking, charge_bearer)`.

`create_iso20022_credit_transfer` (`account_iso20022/models/account_journal.py:87-90`, `AccountJournal.create_iso20022_credit_transfer`) → `create_iso20022_credit_transfer_content` (`:92-167`) que construye el árbol `lxml`:

```
Document (nsmap = urn:iso:std:iso:20022:tech:xsd:<pain_version>)
└─ CstmrCdtTrfInitn
   ├─ GrpHdr
   │   ├─ MsgId      = str(time.time())          [:105]
   │   ├─ CreDtTm    = ISO datetime
   │   ├─ NbOfTxs    = len(payments)  (UserError si > 15 dígitos)  [:111]
   │   ├─ CtrlSum    = sum(amount) float_repr 2dec  [:114]
   │   └─ InitgPty   (company PartyIdentification32)
   └─ PmtInf  (UNO por (payment_date, currency_id))   [:128]
       ├─ PmtInfId   = (MsgId + journal.id + count)[-30:]
       ├─ PmtMtd     = 'TRF'
       ├─ BtchBookg  = batch_booking
       ├─ NbOfTxs / CtrlSum  (del grupo)
       ├─ PmtTpInf   (SvcLvl/Cd, CtgyPurp, LclInstrm)
       ├─ ReqdExctnDt
       ├─ Dbtr / DbtrAcct (IBAN o Othr) / DbtrAgt (FinInstnId)
       ├─ ChrgBr
       └─ CdtTrfTxInf  (UNO por payment)
           ├─ PmtId (InstrId 35c + EndToEndId 30c)
           ├─ Amt/InstdAmt Ccy=...
           ├─ CdtrAgt (FinInstnId: BIC o Othr/NOTPROVIDED)
           ├─ Cdtr/Nm (+ PstlAdr)
           ├─ CdtrAcct (IBAN o Othr)
           └─ RmtInf (Strd estructurado o Ustrd libre)
```

**Agrupación**: una `PmtInf` por combinación **(fecha de ejecución, divisa)** (`account_iso20022/models/account_journal.py:121-128`, `AccountJournal.create_iso20022_credit_transfer_content`). `required_payment_date = max(payment_date, today)` — nunca permite fechas pasadas.

**Versión pain**: campo `sepa_pain_version` (`account_iso20022/models/account_journal.py:18-30`, `class AccountJournal`), computed con `readonly=False` (`_compute_sepa_pain_version` `:36-56`):
- `pain.001.001.09` — default actual (recomendado, estructurado)
- `pain.001.001.03` — legacy
- `pain.001.001.03.de` (Alemania), `pain.001.001.03.austrian.004` (Austria) — por país, derivado del prefijo IBAN del banco o del `account_fiscal_country_id`.

El namespace lo decide `get_document_namespace` (`:413-417`): `urn:iso:std:iso:20022:tech:xsd:pain.001.001.{03|09}`.

Diferencias 03 vs 09 (override en `account_journal_sepa_ct.py`):
- **09** usa `<ReqdExctnDt><Dt>` anidado (`:10-21`); 03 usa texto directo.
- **09** dirección estructurada `StrtNm/PstCd/TwnNm/Ctry` (`:52-67`); 03 usa `AdrLine` libres (`account_iso20022/models/account_journal.py:434-448`, `AccountJournal._get_schema_location`).
- **09** tag BIC = `BICFI`; 03 = `BIC` (`:129-137` / `:419-421`).
- **09** regex BIC distinto (`:139-147`).
- **09** soporta `LEI` en `OrgId` y en `CdtrAgt` (`account_iso20022/models/account_journal.py:391-394`, `AccountJournal._get_company_PartyIdentification32`, `account_iso20022/models/account_journal_sepa_ct.py:28-43`, `AccountJournal._skip_CdtrAgt`).
- `iso20022` genérico puede forzarse a pain.09 con el config param `account_iso20022.force_iso_20022_pain_09`, y entonces añade `UETR` (`account_iso20022/models/account_journal_sepa_ct.py:69-84`, `AccountJournal._get_CdtTrfTxInf`).

**SvcLvl/Cd** (`_get_SvcLvlText`): `'SEPA'` para `sepa_ct`, `'NURG'` para `iso20022`/`iso20022_se` (`account_iso20022/models/account_journal_sepa_ct.py:132-135`, `AccountJournal._get_SvcLvlText`, `account_iso20022/models/account_journal.py:429-432`, `AccountJournal._get_schema_location`).

**ChrgBr**: `sepa_ct` fuerza `'SLEV'` (Service Level, `account_iso20022/models/account_journal_sepa_ct.py:45-50`, `AccountJournal._get_ChrgBr`); ISO20022 usa el valor del batch o `'SHAR'` por defecto (`account_iso20022/models/account_journal.py:254-257`, `class AccountJournal`).

**Referencia estructurada (RmtInf)** (`_get_RmtInf` `:342-371`): detecta tipo de referencia por país (BE/CH/FI/NO/SE/ISO) usando `account.tools.structured_reference`; si reconoce una referencia estructurada genera `<Strd>` (con `CdtrRefInf` / `Cd=SCOR` / `Issr=ISO`), si no, comunicación libre en `<Ustrd>`. CH detecta QR-IBAN (`_is_qr_iban` `:495-515`, rango IID 30000-31999).

---

## Generación XML — SDD / pain.008 (Direct Debit) [account_sepa_direct_debit]

Punto de entrada: `account.batch.payment._generate_export_file` (`account_sepa_direct_debit/models/account_batch_payment.py:252-262`, `AccountBatchPayment._generate_export_file`) → `payment_ids.generate_xml(company, required_collection_date, batch_booking)`.

`account.payment.generate_xml` (`account_sepa_direct_debit/models/account_payment.py:93-113`, `AccountPayment.generate_xml`) construye con helpers `create_xml_node` / `create_xml_node_chain` (de `odoo.tools.xml_utils`):

```
Document (nsmap = urn:iso:std:iso:20022:tech:xsd:<debit_sepa_pain_version>)
└─ CstmrDrctDbtInitn
   ├─ GrpHdr (MsgId, CreDtTm, NbOfTxs, CtrlSum, InitgPty/Id/OrgId/Othr/Id = creditor identifier)
   └─ PmtInf  (UNO por journal — _group_payments_per_bank_journal)
       ├─ PmtInfId = MsgId + '/' + counter
       ├─ PmtMtd = 'DD'
       ├─ BtchBookg, NbOfTxs, CtrlSum
       ├─ PmtTpInf
       │   ├─ SvcLvl/Cd = 'SEPA'
       │   ├─ LclInstrm/Cd = sdd_scheme  (CORE | B2B)
       │   └─ SeqTp = 'RCUR'   (siempre, informativo)
       ├─ ReqdColltnDt
       ├─ Cdtr/Nm (+ PstlAdr si pain.008.001.08)
       ├─ CdtrAcct/Id/IBAN
       ├─ CdtrAgt/FinInstnId (BIC/BICFI o Othr/NOTPROVIDED)
       ├─ CdtrSchmeId/Id/PrvtId/Othr/Id = creditor identifier (+ SchmeNm/Prtry = SEPA)
       └─ DrctDbtTxInf  (UNO por payment — sdd_xml_gen_payment)
           ├─ PmtId/EndToEndId
           ├─ InstdAmt Ccy=...
           ├─ DrctDbtTx/MndtRltdInf (MndtId = mandate.name, DtOfSgntr = mandate.start_date)
           ├─ DbtrAgt/FinInstnId
           ├─ Dbtr/Nm (+ PstlAdr, + Id/PrvtId|OrgId si debtor_id_code)
           ├─ DbtrAcct/Id/IBAN
           └─ RmtInf/Ustrd (memo)
```

**Versión SDD**: `account.journal.debit_sepa_pain_version` (`account_iso20022/models/account_journal.py:10-18`, `float_repr`), required, default `pain.008.001.02`. Alternativa `pain.008.001.08` (2023, dirección **estructurada obligatoria** + city/country obligatorios, `account_sepa_direct_debit/models/account_payment.py:151-166`, `AccountPayment._sdd_xml_gen_address`).

`SeqTp = RCUR` **siempre**, hardcoded (`account_sepa_direct_debit/models/account_payment.py:189-191`, `AccountPayment._sdd_xml_gen_payment_group`); comentario explícito: refiere a la colección, no al tipo de mandato. No emite FRST/OOFF/FNAL.

---

## sdd.mandate (modelo nuevo) [account_sepa_direct_debit]

`_name = 'sdd.mandate'`, hereda `mail.thread.main.attachment` + `mail.activity.mixin` (`account_sepa_direct_debit/models/sdd_mandate.py:18-22`, `SDDMandate._name`). Representa el consentimiento del cliente para domiciliar.

Estados (`:29-45`): `draft → active → (cancelled | revoked | closed)`.
- `draft`: validar antes de usar.
- `active`: válido para colectar.
- `cancelled`: nunca validado.
- `closed`: expirado o cerrado manualmente; transacciones previas siguen válidas.
- `revoked`: fraudulento; facturas previas podrían necesitar reembolso. **No puede reusarse** (UserError al generar XML, `account_sepa_direct_debit/models/account_payment.py:228-229`, `AccountPayment.sdd_xml_gen_payment`).

Campos clave:
- `name` (Identifier, unique sql constraint, default timestamp microseg)
- `partner_id`, `partner_bank_id` (IBAN del cliente; debe ser tipo iban para validar)
- `sdd_scheme`: `CORE` | `B2B` (B2B exige `partner_id.is_company`, `:320-321`)
- `start_date` (= fecha de firma, va a `DtOfSgntr`), `end_date`
- `one_off`: mandato de un solo uso → pasa a `closed` tras primer pago (`account_iso20022/models/account_payment.py:59-60`, `AccountPayment._compute_iso20022_uetr`)
- `pre_notification_period` (default 2, mín 2 días `SDD_MIN_PRENOT_PERIOD`)
- `debtor_id_code` (máx 35c)

`_sdd_get_usable_mandate(company, partner, date)` (`:122-137`): busca el primer mandato `active`, dentro de rango de fechas, para esa company+partner.

**Expiración** (`_get_expiry_date_per_mandate` `:202-224`): regla SEPA "el mandato expira 36 meses tras la última colección iniciada". Cron `cron_update_mandates_states` (`:350-357`) cierra los caducados y avisa 30 días antes.

Pre-notificación a 5 días para mandatos nuevos (`SDD_FIRST_MIN_PRENOT_PERIOD`), 2 días si todos los mandatos del batch ya se usaron (`account_iso20022/models/account_batch_payment.py:46-68`, `AccountBatchPayment.validate_batch`).

---

## account.batch.payment — integración

Es el orquestador. `_get_methods_generating_files` añade los codes SCT/ISO/SDD (`account_iso20022/models/account_batch_payment.py:35-38`, `AccountBatchPayment._get_methods_generating_files` y SDD `:132-135`). El flujo es: registrar pagos → agruparlos en un batch → `validate_batch` → `_generate_export_file` produce el XML descargable.

Campos SCT (`account_iso20022/models/account_batch_payment.py:13-25`, `class AccountBatchPayment`):
- `iso20022_batch_booking` (default True)
- `iso20022_charge_bearer` (computed: `SLEV` si sepa_ct, si no `SHAR`)

Campos SDD (`account_sepa_direct_debit/account_batch_payment.py:20-31`, `AccountBatchPayment.sdd_required_collection_date`):
- `sdd_required_collection_date` (computed, fecha de cobro; no puede ser < hoy + pre-notificación)
- `sdd_batch_booking`, `sdd_scheme` (todos los pagos del batch deben compartir scheme)

**Nombre de fichero**: SCT → `SCT-<journal>-<timestamp>.xml`; ISO20022 → `PAIN-...`; SDD → `PAIN008<timestamp>.xml` (`account_iso20022/models/account_batch_payment.py:126-129`, `AccountBatchPayment._generate_export_file` / SDD `:258`).

---

## Validación XSD, BIC, IBAN

**XSD**: el módulo trae los schemas en `schemas/` (`pain.001.001.03.xsd`, `pain.001.001.09.xsd`). SDD **sí** trae XSD: `account_sepa_direct_debit/schemas/` contiene `pain.008.001.02.xsd`, `pain.008.001.08.xsd` y `EPC131-08_2019_V1.0_pain.008.001.02.xsd`. **Importante**: NO hay validación automática del XML contra el XSD en el flujo de generación — los schemas están presentes pero el código no llama a `etree.XMLSchema().assertValid()` en ninguno de los dos módulos. La conformidad se garantiza por construcción de nodos y por la regex de BIC.

**IBAN**: depende de `base_iban`. SCT/SDD exigen `bank_account_id.acc_type == 'iban'` tanto en el journal como en las cuentas de los partners (validado en `validate_batch`, `account_iso20022/models/account_batch_payment.py:53-75`, `AccountBatchPayment.validate_batch`). Para no-IBAN se emite `<Othr><Id>` con el `sanitized_acc_number` (`account_iso20022/models/account_journal.py:248-252`, `AccountJournal._get_DbtrAcct`, `:331-340`).

**BIC**: `_get_cleaned_bic_code` (`account_iso20022/models/account_journal.py:471-490`, `AccountJournal.get_postal_address`) valida contra regex (8 u 11 chars). Si no hay BIC se emite `<Othr><Id>NOTPROVIDED</Id>`. `_skip_CdtrAgt` (`:465-466`) puede omitir el agente del acreedor si no hay BIC; solo **Alemania** lo fuerza (`account_iso20022/models/account_journal_sepa_ct_de.py:7-10`, `AccountJournal._skip_CdtrAgt`, devuelve `False` con `pain.001.001.03.de`). No hay override para US. (El override homónimo en `account_iso20022/models/account_journal_sepa_ct.py:23-26`, `AccountJournal._get_ReqdExctnDt_content` compara contra `"pain_de"`, valor inexistente en el Selection → rama muerta.) Tag del BIC: `BIC` (pain 03) o `BICFI` (pain 09).

---

## Identificación de la company (InitgPty / Dbtr)

`res.company` (`res_company.py`):
- `iso20022_initiating_party_name` (máx 70c) — nombre que aparece como iniciador. Se inicializa al nombre de la company en `create` (`:16-24`) y por el `post_init_hook` `init_initiating_party_names` para companies existentes (`account_sepa_direct_debit/models/__init__.py:6`, `account_payment` -11`).
- `iso20022_orgid_id` / `iso20022_orgid_issr` — Identification/Issuer del `OrganisationIdentification`. Default belga: issuer `KBO-BCE`, id derivado del VAT (`_compute_iso20022_orgid` `:26-40`).
- `iso20022_lei` (related a `partner_id.iso20022_lei`).

`res.partner.iso20022_lei` (`account_iso20022/models/res_partner.py:15-26`, `class ResPartner`): Legal Entity Identifier, 20 chars (18 alfanum + 2 dígitos), validado por regex `LEI_REGEX`.

`iso20022_get_company_name` (`account_iso20022/models/account_journal.py:423-427`, `AccountJournal.get_document_namespace`): usa 35c si hay initiating_party_name, 70c si cae al `company.name`.

---

## Sanitización de comunicaciones

`sepa_mapping.sanitize_communication(text, size)` (`account_batch_payment/models/sepa_mapping.py:5`, `sanitize_communication`): reemplaza caracteres no permitidos por el European Payment Council (set restringido SEPA), colapsa `//`, y trunca por bytes. SCT lo expone como `_sepa_sanitize_communication` (deprecated wrapper, `account_journal.py` / `account_iso20022/models/account_journal_sepa_ct.py:125-127`, `AccountJournal._get_bic_tag`); SDD como `split_node` que parte por límite de bytes UTF-8 (`account_iso20022/models/account_payment.py:64-72`, `AccountPayment._compute_iso20022_charge_bearer`).
