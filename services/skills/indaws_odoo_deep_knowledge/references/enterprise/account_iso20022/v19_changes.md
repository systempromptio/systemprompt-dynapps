# account_iso20022 — v19 Changes

Diffs verificados entre `odoo_enterprise/18/account_iso20022` y `odoo_enterprise/19/account_iso20022` (+ `account_sepa_direct_debit`).

---

## Resumen ejecutivo

v19 introduce **soporte US (NACHA/ACH)** vía nuevo code `iso20022_us`, mueve el **charge bearer y la prioridad al nivel de pago** (antes solo en el batch), reemplaza el `EndToEndId` por **UUID** y añade `end_to_end_uuid` a payment y bank statement line. El payment method gana un flag computado `is_iso20022`. El módulo SDD apenas cambia.

---

## Manifest

`19/account_iso20022/__manifest__.py`:
- Nueva vista `views/account_bank_statement_line_views.xml`.
- Añadido `'author': 'Odoo S.A.'`.
- Mantiene `depends`, `post_init_hook`, license.

---

## Ficheros nuevos / eliminados (models)

```
+ account_bank_statement_line.py   (nuevo)
+ account_journal_iso20022_us.py   (nuevo — soporte ACH/NACHA US)
- account_payment_register.py      (ELIMINADO)
```

`account_payment_register.py` desaparece: la validación de bank account al registrar pagos SEPA/ISO se reubicó (ya no como override del wizard en este módulo).

---

## Nuevo code: `iso20022_us` (ACH / NACHA)

`account_payment_method.py` v19 registra `iso20022_us` en `_get_payment_method_information` (`:39`). El journal lo añade a defaults outbound (`account_iso20022/models/account_journal.py:102-103`, `AccountJournal.create_iso20022_credit_transfer_content`).

`account_journal_iso20022_us.py` (nuevo) sobreescribe nodos para US:
- `get_document_namespace` → `pain.001.001.03`.
- `_get_PmtTpInf` → `LclInstrm/Cd` con entry class code NACHA (default `CCD`; usa `nacha_entry_class_code` si el módulo `l10n_us_payment_nacha` está instalado).
- `_get_Dbtr` → `schme_nm="CHID"`, sin Id.
- `_skip_CdtrAgt` → siempre False (US exige CdtrAgt).
- `_get_CdtTrfTxInf` / `_get_RmtInf` → eliminan caracteres reservados de BoA `(* + ~ : ')` de InstrId y Ustrd.
- `_get_FinInstnId` → añade `PstlAdr/Ctry`, `ClrSysMmbId/MmbId` (= `clearing_number`), reordena BIC. ABA routing number vía clearing_number.

---

## Charge bearer y prioridad → nivel de pago

En v18 `iso20022_charge_bearer` vivía en `account.batch.payment` (computed). En v19 se mueve a `account.payment` y `account.journal`:

`account.payment` v19 (`account_payment.py`) define módulo-level:
```python
ISO20022_CHARGE_BEARER_SELECTION   # :7
ISO20022_PRIORITY_SELECTION        # :13  (NORM / HIGH)
ISO20022_PRIORITY_HELP             # :19
```
Nuevos campos en `account.payment`:
- `iso20022_priority` (selection, `:44-49`)
- `iso20022_charge_bearer` (computed `_compute_iso20022_charge_bearer` `:55-68`: `SLEV` si sepa_ct, si no toma `journal.iso20022_charge_bearer`)
- `payment_method_is_iso20022` (related a `payment_method_id.is_iso20022`, `:51`)
- `end_to_end_uuid` (computed `:30-32`, ver abajo)

`account.journal` v19 gana `iso20022_default_priority`, `iso20022_charge_bearer` (default `SHAR`) y `has_iso20022_payment_method` (computed) (`account_iso20022/models/account_journal.py:35-48`, `class AccountJournal`, `:84-90`).

**Consecuencia en el XML**: la agrupación de `PmtInf` ahora es por **(fecha, divisa, prioridad)** (`account_iso20022/models/account_journal.py:161-163`, `AccountJournal.create_iso20022_credit_transfer_content`). El `ChrgBr` se emite a nivel `PmtInf` solo si todos los pagos del grupo comparten valor; si no, baja a nivel `CdtTrfTxInf` (`include_charge_bearer`, `:197-206`, `:294`).

**Prioridad**: `_get_PmtTpInf(payment_method_code, priority)` ahora recibe la prioridad y emite `InstrPrty` con el valor (`:221-225`), en lugar del antiguo hardcode `HIGH` solo para nóminas.

---

## EndToEndId → UUID

v18: `EndToEndId.text = (end_to_end_id or PmtInfId + payment_id)[-30:]` (truncado a 30c → **ISOG13**).

v19: `EndToEndId.text = payment.get('end_to_end_uuid') or uuid4().hex` (`account_iso20022/models/account_journal.py:301`, `AccountJournal._get_CdtTrfTxInf`). Cada pago lleva un UUID hexadecimal único de 32 chars.

`end_to_end_uuid` se computa en `account.payment._compute_end_to_end_uuid` (`account_sepa_direct_debit/models/account_payment.py:106-109`, `AccountPayment.generate_xml`) para todos los codes iso20022* + sepa_ct. Y se añade nuevo campo `end_to_end_uuid` a `account.bank.statement.line` (`account_bank_statement_line.py`) para casar el extracto bancario contra el pago original por E2E — cierra el círculo de conciliación que en v18 fallaba (**ISOG13**).

`_get_payment_vals` del batch v19 propaga `end_to_end_uuid`, `iso20022_charge_bearer`, `iso20022_priority` al dict del pago (`account_iso20022/models/account_batch_payment.py:136-139`, `AccountBatchPayment._get_payment_vals`).

---

## account.payment.method.is_iso20022

Nuevo campo computado (`account_iso20022/models/account_payment_method.py:10-15`, `AccountPaymentMethod._get_payment_method_information`):
```python
is_iso20022 = code and code.startswith('iso20022')
```
Usado por journal/batch/payment para detectar genéricamente cualquier método ISO20022 (`payment_method_is_iso20022` related en varios modelos) en vez de listar codes a mano.

---

## account.batch.payment (v19)

- `iso20022_batch_booking` default cambia de **True a False** (`account_iso20022/models/account_batch_payment.py:15`, `class AccountBatchPayment`) y el help se reescribe.
- Se elimina `iso20022_charge_bearer` computado del batch (movido a payment/journal).
- `payment_method_is_iso20022` related nuevo.
- `_get_methods_generating_files` y `check_payments_for_errors` añaden `iso20022_us`.
- Nuevo `_get_export_file_name_prefix` (`:102-104`) extrae el prefijo (SCT-/PAIN-) a método sobrecargable.
- `create_iso20022_credit_transfer` pierde el arg `charge_bearer` (ahora viaja por pago).
- Para `iso20022_se`: nuevo context `bban` cuando coexisten bban_se/plusgiro/bankgiro (`:109-111`).

---

## Otros cambios menores (core journal v19)

- `sepa_pain_version` string pasa a "SEPA XML Format" (`:28`).
- Defensa de migración: si `sepa_pain_version` tiene un valor inválido (bug de upgrade conocido, PR odoo/upgrade#9771) se resetea a `pain.001.001.09` (`:122-125`).
- Soporte `schema_location` opcional en el `Document` raíz (`:130-133`).
- QR-IBAN: `_get_DbtrAcct` ahora usa `<Othr>` también para QR-IBAN suiza, no solo no-IBAN (`:273`, `_is_bank_account_qr_iban`).
- `_sepa_sanitize_communication` deprecated; llamadas directas a `sanitize_communication` de `sepa_mapping`.
- Imports nuevos: `uuid4`, `get_iban_part` de `base_iban`, constantes de `account_payment`.

---

## account_sepa_direct_debit (v19)

Cambios mínimos. Misma estructura de modelos (sin nuevos ni eliminados respecto a v18). `debit_sepa_pain_version` mantiene las mismas dos opciones (`pain.008.001.02` default, `pain.008.001.08`). La lógica de `sdd.mandate`, expiración 36 meses, pre-notificación y schemes CORE/B2B se mantiene.

---

## Impacto en migración 18→19

1. **EndToEndId cambia de formato** (truncado→UUID): si tenías conciliación o reporting basado en el E2E de pagos antiguos, los nuevos ficheros usan UUID. No retroactivo. Ahora casa contra `account.bank.statement.line.end_to_end_uuid`.
2. **Charge bearer/prioridad por pago**: configuraciones que dependían del campo de batch deben reconfigurarse a nivel journal (`iso20022_charge_bearer` default SHAR, `iso20022_default_priority` default NORM).
3. **batch_booking default False**: revisar batches que asumían True.
4. **`iso20022_us`**: nuevo en zona US; en EU no afecta.
5. **Posible reset de `sepa_pain_version`** por el guard de migración: verificar la versión pain del journal tras migrar → **ISOG5**.
6. Bug de upgrade conocido sobre `sepa_pain_version` fixed en `odoo/upgrade#9771`.

España: sin cambios específicos. Las normas 34/19 siguen sin implementación nativa AEB; el camino OCA/`l10n-spain` permanece igual (ver `integrations.md`).
