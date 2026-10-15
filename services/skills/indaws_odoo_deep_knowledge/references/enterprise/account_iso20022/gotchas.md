<!-- verified: v18.0 @ 2026-06 -->
# account_iso20022 — Gotchas

Prefijo **ISOG**. Verificado contra fuente (`odoo_enterprise/18/account_iso20022` y `odoo_enterprise/18/account_sepa_direct_debit`). Citas `path:line` relativas a esos dos módulos.

---

## ISOG1 — `sepa_ct` desaparece del wizard de pago si la moneda no es SEPA-elegible

`_get_payment_method_codes_to_exclude` (`account_iso20022/models/account_payment.py:45-52`, `AccountPayment._get_payment_method_codes_to_exclude`) excluye `sepa_ct` cuando la moneda del pago no está en la whitelist (EUR, GBP, CHF, NOK, SEK, DKK, PLN, CZK, HUF, RON, BGN, HRK, ISK, GIP, XPF).

**Síntoma**: "No aparece la opción SEPA Credit Transfer al registrar un pago." Casi siempre es porque la factura/pago está en una moneda fuera de la lista (p.ej. USD). El método existe y está bien configurado en el journal, pero el wizard lo oculta.

**Fix**: usar el payment method `iso20022` genérico (pensado para no-EUR/no-SEPA) o cambiar la moneda.

---

## ISOG2 — `iso20022` genérico se auto-excluye de companies en EUR al instalar

`_get_payment_method_domain` (`account_iso20022/models/account_payment_method.py:33-44`, `AccountPaymentMethod._get_payment_method_domain`) añade un domain que impide que `iso20022` se añada a journals cuya company tenga divisa EUR. Es deliberado: en zona euro debe usarse `sepa_ct`, no el genérico.

**Síntoma**: "Activé el módulo pero el método ISO20022 no aparece en mi journal." En company EUR es el comportamiento esperado: aparece `sepa_ct`, no `iso20022`. Para forzar ISO20022 genérico en EUR habría que añadir la payment.method.line manualmente.

---

## ISOG3 — No hay validación XSD del fichero generado

Los `.xsd` están en `schemas/` (SCT: `pain.001.001.03.xsd`, `pain.001.001.09.xsd`; SDD: `pain.008.001.02.xsd`, `pain.008.001.08.xsd`, `EPC131-08_2019_V1.0_pain.008.001.02.xsd`) pero el código **nunca** los usa para validar (no hay `etree.XMLSchema(...).assertValid()` en ninguno de los dos módulos). La conformidad es por construcción de nodos y por la regex de BIC.

**Síntoma**: "El XML pasó en Odoo pero el banco lo rechaza." Odoo no valida contra esquema; un BIC raro, una dirección incompleta, o un campo fuera de orden pueden colarse. Validar el fichero externamente contra el XSD del banco antes de subirlo.

---

## ISOG4 — Sin BIC se emite `NOTPROVIDED` en lugar de fallar

`_get_FinInstnId` (`account_iso20022/models/account_journal.py:309-319`, `AccountJournal._get_CdtTrfTxInf`): si la cuenta del acreedor no tiene BIC, en vez de error genera `<Othr><Id>NOTPROVIDED</Id>`. Además `_skip_CdtrAgt` (`:465-466`) puede omitir todo el bloque `CdtrAgt` si no hay BIC (`not partner_bank.bank_id.bic`).

**Síntoma**: "El banco rechaza el SEPA por falta de BIC" / "el destinatario no recibe el pago". El fichero se genera sin error pero con `NOTPROVIDED`. Alemania **fuerza** el bloque CdtrAgt aunque no haya BIC: `_skip_CdtrAgt` override en `account_iso20022/models/account_journal_sepa_ct_de.py:7-10`, `AccountJournal._skip_CdtrAgt` devuelve `False` cuando `sepa_pain_version == "pain.001.001.03.de"`. (Cuidado: el override del mismo nombre en `account_iso20022/models/account_journal_sepa_ct.py:23-26`, `AccountJournal._get_ReqdExctnDt_content` compara contra `"pain_de"`, valor que NO existe en el Selection → rama muerta, no fuerza nada.) Rellenar el BIC en `res.partner.bank`.

---

## ISOG5 — La versión pain por defecto cambió a 09 y puede romper bancos legacy

`_compute_sepa_pain_version` (`account_iso20022/models/account_journal.py:36-56`, `class AccountJournal`) pone `pain.001.001.09` por defecto salvo DE/AT. Muchos bancos (sobre todo fuera de zona euro core) aún esperan `pain.001.001.03`.

**Síntoma**: "Tras actualizar, el banco rechaza el fichero con error de formato/namespace." El namespace pasó a `...:pain.001.001.09`. El campo es `readonly=False`: cambiarlo manualmente a `pain.001.001.03` en el journal. Nota: el compute respeta el cambio manual a 03 (`:52-56`), pero NO respeta cambios manuales a otros valores frente a recomputes.

---

## ISOG6 — Una `PmtInf` por (fecha, divisa): un batch mixto explota en bloques

`create_iso20022_credit_transfer_content` agrupa por `(required_payment_date, currency_id)` (`account_iso20022/models/account_journal.py:121-128`, `AccountJournal.create_iso20022_credit_transfer_content`). Pagos con distinta fecha o distinta divisa en el mismo batch generan múltiples bloques `PmtInf`.

**Síntoma**: "El banco procesa solo parte del fichero" o "los importes no cuadran con el CtrlSum del header". Cada `PmtInf` tiene su propio `CtrlSum`/`NbOfTxs`; algunos bancos limitan nº de `PmtInf` por fichero. Agrupar por fecha/divisa homogénea.

---

## ISOG7 — `required_payment_date = max(payment_date, today)`: fechas pasadas se silencian

`account_iso20022/models/account_journal.py:125`, `AccountJournal.create_iso20022_credit_transfer_content`. Si un pago tiene fecha anterior a hoy, se reemplaza por hoy sin avisar.

**Síntoma**: "Generé el fichero con fecha de ejecución X pero el banco ejecutó hoy." Esperado: SEPA no permite fechas en pasado y Odoo las corrige en silencio.

---

## ISOG8 — SDD `SeqTp` siempre `RCUR`, nunca FRST/OOFF/FNAL

`account_sepa_direct_debit/models/account_payment.py:189-191`, `AccountPayment._sdd_xml_gen_payment_group` (módulo SDD) hardcodea `SeqTp = 'RCUR'`. El comentario lo confirma: "refiere a la colección, no al tipo de mandato, solo informativo".

**Síntoma**: "El banco exige FRST en la primera remesa de un mandato nuevo." Algunos bancos/esquemas (esp. esquemas antiguos) discriminan FRST vs RCUR. Odoo solo emite RCUR. Si el banco lo exige, requiere desarrollo o módulo OCA.

---

## ISOG9 — Mandato `revoked` bloquea el XML; `closed` no

`sdd_xml_gen_payment` (`account_sepa_direct_debit/models/account_payment.py:228-229`, `AccountPayment.sdd_xml_gen_payment`) lanza UserError solo si el mandato está `revoked`. Un mandato `closed` (expirado) **no** se chequea aquí — el filtro de validez está antes, en `validate_batch` (`account_sepa_direct_debit/models/account_batch_payment.py:195-207`, `AccountBatchPayment.validate_batch`).

**Síntoma**: "Pude generar el XML con un mandato vencido." Si el pago se enlazó antes de cerrar el mandato y se salta `validate_batch`, puede colarse. El gate fiable es `_update_and_partition_state_by_validity` en validación del batch.

---

## ISOG10 — Cron cierra mandatos a los 36 meses de la última colección

`_get_expiry_date_per_mandate` (`account_sepa_direct_debit/models/sdd_mandate.py:202-224`, `SDDMandate._get_expiry_date_per_mandate`) + `cron_update_mandates_states` (`:350-357`). Regla SEPA: 36 meses sin colectar → expira.

**Síntoma**: "Un mandato activo apareció de pronto en estado closed." El cron lo cerró porque la última colección `paid` fue hace >36 meses (o se pasó el `end_date`). Hay aviso por email 30 días antes (`expiration_warning_already_sent`).

---

## ISOG11 — `pain.008.001.08` exige dirección estructurada y ciudad/país

`_sdd_xml_gen_address` (`account_sepa_direct_debit/models/account_payment.py:151-166`, `AccountPayment._sdd_xml_gen_address`): en pain.008.001.08 city y country son **obligatorios** (UserError si faltan: city en `:160`, country en `:166`). En `pain.008.001.02` (default) la dirección es libre (`AdrLine`) y no se exige city/country.

**Síntoma**: "UserError: The debtor and creditor city name is a compulsary information..." (o ...country...). El journal está en `pain.008.001.08` y un partner (o la company) no tiene ciudad o país. O completar la dirección, o volver a `pain.008.001.02` (default).

---

## ISOG12 — `one_off` mandato se cierra automáticamente tras el primer pago

`account_iso20022/models/account_payment.py:59-60`, `AccountPayment._compute_iso20022_uetr` (SDD): al pasar el pago a `in_process`, si el mandato es `one_off` se ejecuta `action_close_mandate()`.

**Síntoma**: "El mandato se cerró solo tras una sola remesa." Es la semántica de `one_off` (marcado oculto en UI por defecto, `account_sepa_direct_debit/models/sdd_mandate.py:48-51`, `class SDDMandate`). Para domiciliaciones recurrentes NO marcar one_off.

---

## ISOG13 — `EndToEndId` truncado a 30 caracteres (no 35)

`account_iso20022/models/account_journal.py:266`, `class AccountJournal`: `EndToEndId.text = (... )[-30:]`. La especificación SEPA permite 35, pero Odoo trunca a 30 y por la derecha (`[-30:]`).

**Síntoma**: "La conciliación bancaria no casa por referencia E2E." Si tu referencia tiene >30c se pierde el prefijo. En v19 esto cambia a un UUID hex (ver `v19_changes.md`).

---

## ISOG14 — `iso20022_uetr` solo se genera para el code `iso20022`, no para `sepa_ct`

`_compute_iso20022_uetr` (`account_iso20022/models/account_payment.py:54-60`, `class AccountPayment`) filtra `payment_method_id.code == 'iso20022'`. El UETR solo se emite en pain.09 forzado (`account_iso20022/models/account_journal_sepa_ct.py:73-76`, `AccountJournal._get_CdtTrfTxInf`).

**Síntoma**: "Falta el UETR en mi fichero SEPA CT." Esperado: el UETR (rastreo end-to-end SWIFT gpi) no se genera para `sepa_ct`, solo para ISO20022 genérico con `force_iso_20022_pain_09`.

---

## ISOG15 — `iso20022_ch` con `iso20022_ch_force_sepa` reescribe el code por divisa

`create_iso20022_credit_transfer_content` (`account_iso20022/models/account_journal.py:142-149`, `AccountJournal.create_iso20022_credit_transfer_content`): si el config param `iso20022_ch_force_sepa` está activo, dentro de cada grupo el code cambia dinámicamente a `sepa_ct` (EUR), `iso20022` (no-CHF) o se queda `iso20022_ch` (CHF).

**Síntoma**: "Configuré pago suizo pero el fichero salió como SEPA estándar." El switch Suiza-trata-SEPA-como-subconjunto está activo. Comportamiento intencional para bancos suizos multi-divisa.

---

## ISOG16 — Charge bearer: `sepa_ct` ignora el valor del batch (siempre SLEV)

`account_iso20022/models/account_journal_sepa_ct.py:45-50`, `AccountJournal._get_ChrgBr`: para `sepa_ct` el `ChrgBr` es siempre `'SLEV'`, ignorando `iso20022_charge_bearer` del batch.

**Síntoma**: "Cambié Charge Bearer a SHAR en el batch pero el XML dice SLEV." Esperado para SEPA CT (SLEV = Service Level, las comisiones se reparten según reglas SEPA). El campo del batch solo afecta a ISO20022 genérico.

---

## ISOG17 — `sdd_creditor_identifier` obligatorio y validado con Mod-97; sin él no hay SDD

`res_company.validate_sdd_creditor_identifier` (`account_sepa_direct_debit/models/res_company.py:19-39`, `ResCompany.validate_sdd_creditor_identifier`) valida formato + check digits Mod 97-10. `validate_batch` (`account_sepa_direct_debit/models/account_batch_payment.py:171-179`, `AccountBatchPayment.validate_batch`) lanza RedirectWarning a Ajustes si falta.

**Síntoma**: "RedirectWarning: Your company must have a creditor identifier." Configurar el identificador de acreedor SEPA (el que da el banco, formato `ESxx ZZZ <id>` en España) en Contabilidad → Ajustes.

---

## ISOG18 — No se puede borrar una `res.partner.bank` ligada a mandato activo

`res_partner_bank._unlink_except_linked_to_mandate` (`account_sepa_direct_debit/models/res_partner_bank.py:7-10`, `class ResPartnerBank`, SDD): bloquea unlink si hay un `sdd.mandate` activo apuntando a esa cuenta.

**Síntoma**: "UserError al intentar eliminar una cuenta bancaria de un cliente." Revocar/cerrar primero el mandato.

---

## ISOG19 — `NbOfTxs` limitado a 15 dígitos (no al nº real de transacciones)

`account_iso20022/models/account_journal.py:111-112`, `AccountJournal.create_iso20022_credit_transfer_content`: `ValidationError("Too many transactions for a single file")` si `len(str(len(payments))) > 15`. Es un límite teórico absurdo (10^15 pagos); el límite real lo pone el banco.

**Síntoma**: relevante solo como aclaración: el límite práctico de transacciones por fichero lo impone el banco, no Odoo. Odoo no avisa de exceder el máximo del banco (salvo `check_payments_for_errors` por importe, `account_iso20022/models/account_batch_payment.py:93-112`, `AccountBatchPayment.check_payments_for_errors`).

---

## ISOG20 — Initiating party name se fija una vez en `create` y no se recomputa

`res_company.create` (`account_iso20022/models/res_company.py:16-24`, `class ResCompany`) copia `company.name` a `iso20022_initiating_party_name` solo si no viene en vals. No es computed: si luego renombras la company, el initiating party name **no** cambia.

**Síntoma**: "Cambié el nombre de la empresa pero el SEPA sigue mostrando el viejo." Editar manualmente `iso20022_initiating_party_name` en Ajustes de Contabilidad (campo "Your Company Name").

---

## ISOG21 — `EndToEndId` en SCT nunca usa `end_to_end_id`: siempre `PmtInfId+id`

`_get_CdtTrfTxInf` (`account_iso20022/models/account_journal.py:266`, `class AccountJournal`) hace `(payment.get('end_to_end_id') or PmtInfId.text + str(payment['id']))[-30:]`. Pero `_get_payment_vals` (`account_iso20022/models/account_batch_payment.py:133-147`, `AccountBatchPayment._get_payment_vals`) **no** rellena la clave `end_to_end_id` (mete `iso20022_uetr`, otra cosa). Por tanto el `.get('end_to_end_id')` siempre es `None` y el E2E es siempre `PmtInfId+id` truncado a 30.

**Síntoma**: "Quiero personalizar la referencia E2E del pago y no se respeta." En SCT no hay forma estándar de inyectarla vía el payment template; el campo `end_to_end_id` es un hook latente sin emisor. La referencia real va en `RmtInf` (memo), no en E2E.

---

## ISOG22 — `MsgId` = `str(time.time())`: dos ficheros en el mismo segundo pueden colisionar

`account_iso20022/models/account_journal.py:105`, `AccountJournal.create_iso20022_credit_transfer_content` (SCT) y `account_iso20022/models/account_payment.py:126`, `class AccountPayment` (SDD): `MsgId = str(time.time())`. Es un float con microsegundos, pero el `PmtInfId` SCT deriva de `(MsgId+journal.id+count)[-30:]` (`:131`).

**Síntoma**: "El banco rechaza por MsgId duplicado." Improbable (time.time tiene resolución sub-microsegundo) pero posible si automatizas la generación de muchos batches sin pausa. No hay secuencia ni unicidad garantizada por BD; el identificador depende del reloj.

---

## ISOG23 — El nombre del acreedor (`Cdtr/Nm`) prioriza `acc_holder_name` sobre el partner

`account_iso20022/models/account_journal.py:289-291`, `AccountJournal._get_CdtTrfTxInf`: `Nm = acc_holder_name or partner.name or commercial_partner_id.name or '/'`, truncado a 70 y sanitizado. Si la `res.partner.bank` tiene un titular distinto al partner del pago, manda el titular de la cuenta.

**Síntoma**: "El SEPA sale con un nombre de beneficiario que no es el del cliente." Es porque la cuenta bancaria tiene `acc_holder_name` rellenado. Vaciar ese campo para que caiga al nombre del partner. (Mismo patrón en SDD para el `Dbtr/Nm`, `account_sepa_direct_debit/models/account_payment.py:249`, `AccountPayment.sdd_xml_gen_payment`.)

---

## ISOG24 — Pagos de nómina (`sepa_payroll_sala`) cambian el XML: prioridad HIGH + CtgyPurp SALA

`_get_PmtTpInf`/`_get_CtgyPurp` (`account_iso20022/models/account_journal.py:181-220`, `AccountJournal._get_CtrlSum`): si el contexto trae `sepa_payroll_sala`, se añade `<CtgyPurp><Cd>SALA</Cd>`; y con `l10n_be_hr_payroll_sepa_salary_payment` también `<InstrPrty>HIGH</InstrPrty>` y prefijo `/A/ ` en `Ustrd` (`account_iso20022/models/account_journal_sepa_ct.py:116-123`, `class AccountJournal`).

**Síntoma**: "El mismo journal genera XML distinto según de dónde se lance." Esperado: el módulo de nóminas inyecta ese contexto. Un batch normal no lleva `SALA` ni `HIGH`; uno de nómina sí. No es configurable desde el batch.

---

## ISOG25 — `LclInstrm` en ISO20022 genérico solo aparece con el config param `local_instrument_code`

`_get_LclInstrm` (`account_iso20022/models/account_journal.py:206-211`, `AccountJournal._get_PmtTpInf`): solo emite `<LclInstrm><Cd>` para `payment_method_code == 'iso20022'` y si existe el `ir.config_parameter` `account_iso20022.local_instrument_code`. Sin ese param no hay nodo.

**Síntoma**: "Mi banco exige un Local Instrument code y no aparece en el fichero ISO20022." Setear el config param `account_iso20022.local_instrument_code`. No hay campo en UI; es parámetro de sistema.

---

## ISOG26 — Empleados sin país: ValidationError en `get_postal_address`, no UserError silencioso

`get_postal_address` (`account_iso20022/models/account_journal.py:450-463`, `AccountJournal._get_SvcLvlText`): si el partner no tiene ninguna dirección con país, lanza `ValidationError` ("Partner X has no country code defined" para companies, "Employee X has no country" para personas). Además, entre direcciones con ciudad, prioriza la de `contact_type == 'employee'` (`:461`).

**Síntoma**: "ValidationError al generar el SEPA sobre un partner aparentemente completo." Le falta el país en la dirección (o todas sus direcciones carecen de él). En pagos de nómina, ojo: se elige la dirección de tipo empleado por delante de la primera disponible.
