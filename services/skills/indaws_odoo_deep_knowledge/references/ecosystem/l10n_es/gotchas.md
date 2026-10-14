<!-- verified: v18.0 @ 2026-06 (l10n_es_edi_* core + OCA l10n-spain; cite/method checks clean, 15/15 sample CONFIRMED vs source) -->
# Spanish Localization Gotchas — Odoo 18.0

Prefix `ESG` (Es-Spain Gotchas). All grounded in source code under `odoo18/addons/l10n_es*`, `odoo_enterprise/18/l10n_es*`, `OCA/v18/l10n-spain/`.

---

**ESG1. `l10n_es_type='ignore'` excludes BOTH base AND tax** — Unlike `no_sujeto` (which still reports base), `ignore` removes the line entirely from SII desglose AND mod 303. Source: `l10n_es_edi_sii/models/account_edi_format.py:41-46`, `_l10n_es_edi_get_invoices_tax_details_info.filter_to_apply` filters out invoice lines where ALL taxes are `ignore`. Impact: use `ignore` only for non-fiscal positions like fee/round-off taxes, never for real exempt operations. _ES: Una operación exenta desaparece del 303 y del SII: ni base ni cuota aparecen en la declaración._

**ESG2. Recargo de equivalencia is detected globally per invoice, not per line** — `l10n_es_edi_sii/models/account_edi_format.py:62-65`, `AccountEdiFormat._l10n_es_edi_get_invoices_tax_details_info`: "Since only a single combination tax + recargo is allowed on the same invoice, this can be deduced globally." Impact: an invoice with TWO different sujeto+recargo combinations will silently fail SII validation. Split into two invoices. _ES: Factura con dos tipos distintos de recargo de equivalencia es rechazada por el SII sin explicación clara._

**ESG3. SII intracom detection uses mod_303 tags 10/11, not l10n_es_type** — `l10n_es_edi_sii/models/account_edi_format.py:268-271`, `AccountEdiFormat._l10n_es_edi_get_invoices_info`: _ES: Una factura intracomunitaria de compra sale como régimen general (01) en vez de intracom (09) y Hacienda la rechaza._
```python
mod_303_10 = self.env.ref('l10n_es.mod_303_casilla_10_balance')._get_matching_tags()
mod_303_11 = self.env.ref('l10n_es.mod_303_casilla_11_balance')._get_matching_tags()
intracom = bool(tax_tags & (mod_303_10 + mod_303_11))
```
Impact: if a vendor bill is intracom but the tax has wrong tags, SII regimen code goes 01 (regular) instead of 09 (intra) → AEAT rejects.

**ESG4. `_l10n_es_get_main_tax_types` excludes retencion AND recargo** — `l10n_es/models/account_tax.py:62`, `AccountTax._l10n_es_get_main_tax_types`: _ES: Una línea con dos retenciones o dos recargos rompe el envío al SII._
```python
{'exento', 'sujeto', 'sujeto_agricultura', 'sujeto_isp', 'no_sujeto', 'no_sujeto_loc', 'no_deducible'}
```
Retención + recargo are SECONDARY taxes that always coexist with a main tax. Impact: line validation `_check_move_configuration:606-621` allows ONE main + ONE recargo + ONE retencion. A line with TWO retenciones or TWO recargos breaks SII.

**ESG5. mod 347 threshold uses fiscal year, not declaration period** — Enterprise `l10n_es_reports/models/aeat_tax_reports.py:949-951`, `SpanishMod347TaxReportCustomHandler._report_custom_engine_threshold_all_operations`: _ES: En el modelo 347 no aparece un cliente que superó el umbral en un mes porque se compara con el total anual._
```python
from_fy_dates = self.env.company.compute_fiscalyear_dates(...)
if from_fy_dates == to_fy_dates:
    fy_options['date'].update({...})  # expand to full FY
```
Impact: when generating mod 347 for a partial period, partners are filtered against full-year aggregate (3.005,06€). A partner under threshold YTD won't show even if January alone exceeded.

**ESG6. mod 347 EXCLUDES retentions in subquery filtering** — `l10n_es_reports/models/aeat_tax_reports.py:71-101`, `MOD_347_CUSTOM_ENGINES_DOMAINS` shows `MOD_347_CUSTOM_ENGINES_DOMAINS` includes: _ES: En el 347 un proveedor sube o baja del umbral de golpe al tocar importes de retenciones._
```python
'|', ('account_type', '=', 'liability_payable'),
     ('tax_line_id.l10n_es_type', '=', 'retencion'),
```
Both account-type lines AND retencion tax lines are accepted. Impact: changing the AML balance excluded by AEAT (cuotas retenidas) silently moves partners above/below threshold.

**ESG7. Tax-tag conflict between SII and mod 303** — Enterprise SII derives intracom from `mod_303_casilla_10_balance` tag; Enterprise mod 303 also uses these tags for casillas 10/11. If consultant fixes mod 303 by REMOVING a tag, SII suddenly classifies the invoice as regimen 01 instead of 09 → past invoices need re-submission with `TipoComunicacion='A1'`. _ES: Al arreglar el modelo 303 quitando una etiqueta de impuesto, facturas antiguas del SII cambian de régimen y hay que reenviarlas._

**ESG8. `csv_number` batching prevents mixing A0 and A1 in same envelope** — `l10n_es_edi_sii/models/account_edi_format.py:428`, `AccountEdiFormat._l10n_es_edi_call_web_service_sign_common`: _ES: Enviar de golpe muchas facturas al SII (nuevas y modificadas juntas) provoca fallos en el envío._
```python
'TipoComunicacion': 'A1' if csv_number else 'A0'
```
And `post_batching: lambda invoice: (invoice.move_type, invoice.l10n_es_edi_csv)` (line 584). Impact: a new invoice (no CSV) and a modified invoice (has CSV) sent in the same SOAP envelope ALWAYS fails — Odoo prevents this by batching, but importing many invoices then sending in bulk needs careful timing.

**ESG9. SII `RegistroDuplicado` is silent success** — `l10n_es_edi_sii/models/account_edi_format.py:547-554`, `AccountEdiFormat._l10n_es_edi_call_web_service_sign_common`: if AEAT says duplicate AND state is 'Correcta', Odoo treats as success (no error message). Impact: if you accidentally send the same invoice twice, you won't notice; the first send is the authoritative one. _ES: Se envía dos veces la misma factura al SII y no salta ningún aviso de duplicado._

**ESG10. SII error 1117 triggers automatic IDOtro retry** — `l10n_es_edi_sii/models/account_edi_format.py:556-557`, `AccountEdiFormat._l10n_es_edi_call_web_service_sign_common`: error 1117 (datos sujeto inválidos) → retry recursively with `error_1117=True` context, switching from `NIF` to `IDOtro` partner identifier. Impact: error 1117 followed by silence in the chatter is normal — the retry succeeded. _ES: El SII da un error de datos del sujeto y luego silencio en el chatter: parece que falló pero en realidad se reenvió bien._

**ESG11. Vendor bill matching by `ref` is fuzzy** — `l10n_es_edi_sii/models/account_edi_format.py:512-538`, `AccountEdiFormat._l10n_es_edi_call_web_service_sign_common`: same `ref` for two vendor bills with different partners → falls back to `candidates[0]` with a warning comment. Impact: SII responses may be applied to the WRONG bill if vendor numbering collides — NEVER reuse vendor refs across partners. _ES: Una respuesta del SII se aplica a la factura de proveedor equivocada porque dos proveedores usan la misma referencia._

**ESG12. Bizkaia/Gipuzkoa SOAP requires DH cipher downgrade** — `l10n_es_edi_sii/models/account_edi_format.py:15`, `EUSKADI_CIPHERS`: _ES: El envío al SII de Bizkaia o Gipuzkoa falla con error de SSL/certificado al conectar._
```python
EUSKADI_CIPHERS = "DEFAULT:!DH"
```
Applied via `CertificateAdapter(ciphers=EUSKADI_CIPHERS)`. Impact: modern systems with FIPS-compliant OpenSSL may STILL fail because `:!DH` only excludes DH ciphers, not other weak ones. Symptom: `requests.exceptions.SSLError` on connection — solution is to weaken local OpenSSL config or use `urllib3.util.ssl_.create_urllib3_context()`.

**ESG13. mod 303 `marca_sepa` of `'0'` (empty) blocks both SEPA and AEAT account** — OCA `l10n_es_aeat_mod303/models/mod303.py:299-314`, `L10nEsAeatMod303Report._compute_marca_sepa`: marca_sepa is computed from `partner_bank_id.bank_id.country`. Impact: if the bank record has no country set, marca_sepa stays `'0'` and AEAT rejects domiciliated payments. Symptom: result_type='I' (manual entry) instead of 'U' (auto debit). _ES: El modelo 303 sale como pago manual en vez de domiciliado y Hacienda rechaza la domiciliación por falta de país en el banco._

**ESG14. mod 303 cuota_compensar carry-over is manual** — OCA `l10n-spain/l10n_es_aeat_mod303/models/mod303.py:320-344`, `L10nEsAeatMod303Report._compute_exception_msg`: if previous quarter had `result_type='C'` (compensate) and current report has `cuota_compensar=0`, system raises a WARNING but doesn't auto-fill. Impact: small businesses lose IVA carry-over by forgetting to fill casilla 67. _ES: Se pierde el IVA a compensar del trimestre anterior porque no se rellenó la casilla y el sistema no lo arrastra solo._

**ESG15. Annual regularization (casilla 68) only on 4T/12** — OCA `l10n-spain/l10n_es_aeat_mod303/models/mod303.py:363-368`, `L10nEsAeatMod303Report._compute_regularizacion_anual`: _ES: En el 303 de trimestres Q1-Q3 se borra sola la regularización anual que había introducido._
```python
if record.period_type not in ("4T", "12"):
    record.regularizacion_anual = 0
```
Impact: a manual value entered on Q1-Q3 is silently zeroed. Don't try to regularize early.

**ESG16. mod 390 exoneration must match all four mod 303 returns** — OCA `l10n-spain/l10n_es_aeat_mod303/models/mod303.py:168-195`, `L10nEsAeatMod303Report.exonerated_390`: exonerated_390 + has_operation_volume + has_347 are stored on EACH mod 303. If Q1 says exonerated and Q4 says not, the company is contradictory — AEAT cross-checks. Impact: keep these fields consistent across the year. _ES: El modelo 390 da contradicciones porque los datos de exoneración no coinciden entre los cuatro trimestres del 303._

**ESG17. mod 390 needs `mod_390_casilla_667` ref** — Enterprise `l10n_es_reports/models/aeat_tax_reports.py:150`, `SpanishTaxReportCustomHandler.open_boe_wizard`: _ES: La generación del modelo 390 falla con un aviso pidiendo actualizar el módulo l10n_es._
```python
if not self.env.ref('l10n_es.mod_390_casilla_667', raise_if_not_found=False):
    raise RedirectWarning("Please update the l10n_es module")
```
Impact: BOE generation fails with redirect to Apps. The 667 casilla was added late — old DBs need module update.

**ESG18. mod 349 monthly/quarterly switch is irreversible mid-year** — Threshold: 50.000€/year. Once you exceed → must declare monthly THAT month onwards (not next year). Odoo doesn't enforce this; consultant must change `period_type` manually. Impact: continuing quarterly past the threshold = AEAT inspection. _ES: Al superar el umbral del modelo 349 hay que pasar a mensual ese mismo mes; Odoo no lo cambia solo y arriesga inspección._

**ESG19. mod 130 v18 lives in TWO different modules (CE + Enterprise)** — `l10n_es_modelo130` (CE) + `l10n_es_reports_modelo130` (Enterprise). v19: absorbed into `l10n_es` core. Impact: when migrating v18→v19, must uninstall the CE/EE addon BEFORE upgrading `l10n_es` (otherwise duplicate report records). _ES: Al migrar de v18 a v19 aparecen informes del modelo 130 duplicados._

**ESG20. REAGYP compensation is NOT a real tax — it's a refund of farmer's IRPF** — `l10n_es_type='sujeto_agricultura'` (12% / 10.5%) compensates the agricultural producer. Reported in mod 303 [42][43] as input "VAT" but it's actually IRPF withholding offset. SII puts it in `PorcentCompensacionREAGYP` field, NOT `TipoImpositivo`. Impact: don't merge with regular VAT taxes. Also auto-detected as `TipoFactura='F6'` for vendor bills. _ES: El REAGYP de agricultura se mezcla mal con el IVA normal y sale en la casilla o campo del SII equivocado._

**ESG21. ROI (Operador Intracomunitario) registration is NOT in Odoo** — Membership in the ROI registry must be confirmed manually with AEAT (mod 036). Odoo just sets the partner's intracom VAT prefix and uses it for SII/mod 349 → if the partner is NOT actually in ROI, AEAT rejects with error 4102 (operador no encontrado en VIES). Impact: validate VAT via VIES (`base_vat`) before declaring intracom. _ES: Una operación intracomunitaria es rechazada por Hacienda porque el proveedor no está dado de alta en el ROI/VIES._

**ESG22. `tax_scope` is required for foreign customers (consu vs service)** — SII `_check_move_configuration:622-626`: _ES: Factura a un cliente extranjero sin scope de impuesto configurado se rechaza en el SII por no distinguir entrega de servicio._
```python
if move.is_inbound() and move.commercial_partner_id._l10n_es_is_foreign() and not any(t.tax_scope for t in total_taxes):
    res.append("...you need to configure the tax scope on taxes")
```
TBAI same. Impact: Customer in Portugal with a tax that has no scope → SII can't decide between `Entrega` and `PrestacionServicios` → manual rejection.

**ESG23. TBAI chain freezes if previous invoice rejected** — `l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:127-129`, `L10nEsEdiTbaiDocument._check_can_post`. Impact: rejecting invoice N blocks N+1 forever until N is either accepted, cancelled, or chain manually reset. Symptom: "Cannot post invoice while chain head has not been posted" — solution: cancel the failed TBAI doc or fix the issue and resend. _ES: Una factura rechazada en TicketBAI bloquea todas las siguientes hasta desatascar la cadena._

**ESG24. TBAI chain_index gaps on send failure** — `l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:179`, `L10nEsEdiTbaiDocument._post_to_web_service` assigns chain_index BEFORE send. Line 194: if send fails, `chain_index = 0` (releases the slot). Impact: between assignment and rollback, another concurrent send may grab the same index → race condition. The `FOR UPDATE NOWAIT` lock at `l10n_es_edi_tbai/models/account_move.py:182`, `AccountMove._l10n_es_tbai_lock_move` mitigates but cannot prevent kernel-level interruption. _ES: Bajo envíos simultáneos de TicketBAI se producen huecos o colisiones en el número de cadena._

**ESG25. Bizkaia Batuz wraps TBAI inside LROE — different protocol** — `l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:259-297`, `L10nEsEdiTbaiDocument._prepare_post_params_bi` `_prepare_post_params_bi`. Bizkaia uses gzip + JSON metadata header `eus-bizkaia-n3-data`. Impact: don't reuse Araba/Gipuzkoa code samples for Bizkaia; signature happens BEFORE LROE wrapping, not on the LROE envelope. _ES: El código de TicketBAI de Álava o Gipuzkoa no funciona en Bizkaia porque usa otro protocolo (Batuz/LROE)._

**ESG26. TBAI freelancer in Bizkaia requires `epigrafe` config parameter** — `l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:115-120`, `L10nEsEdiTbaiDocument._check_can_post`: _ES: Un autónomo de Bizkaia no puede enviar ningún TicketBAI porque falta configurar el epígrafe._
```python
if self.company_id._l10n_es_freelancer() and not env['ir.config_parameter'].get_param('l10n_es_edi_tbai.epigrafe', False):
    return _("...need to configure the Epigrafe...")
```
Impact: a self-employed in Bizkaia with no epigrafe parameter set → all TBAI sends fail. Codes are in `batuz_lroe_lista_epigrafes_v1_0_3.xlsx` (external file).

**ESG27. TBAI refund reason `R5` is mutually exclusive with non-simplified** — `l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:154-157`, `L10nEsEdiTbaiDocument._check_can_post`: _ES: Cambiar el flag de factura simplificada tras crear un abono rompe TicketBAI por el motivo de rectificación R5._
```python
if is_simplified and refund_reason != 'R5':
    return _('Refund reason must be R5 for simplified invoices')
if not is_simplified and refund_reason == 'R5':
    return _('Refund reason cannot be R5 for non-simplified invoices')
```
Impact: changing simplified flag after creating refund silently breaks TBAI on next send.

**ESG28. SII strips l10n_es_exempt_reason if l10n_es_type != 'exento'** — `l10n_es_edi_sii/models/account_edi_format.py:32`, `_l10n_es_edi_get_invoices_tax_details_info.grouping_key_generator`: _ES: El SII no reporta la causa de exención porque el impuesto no está marcado como exento._
```python
'l10n_es_exempt_reason': tax.l10n_es_exempt_reason if tax.l10n_es_type == 'exento' else False
```
Impact: a tax with `l10n_es_type='no_sujeto'` AND `l10n_es_exempt_reason='E2'` will report no exempt reason. To use code 02 (exportación) regime, the tax MUST be `exento` with reason E2.

**ESG29. There is NO TicketBAI module in OCA** — Confirmed by grep — no `l10n_es_ticketbai*` exists in `OCA/v18/l10n-spain/`. Only `l10n_es_edi_tbai` (Enterprise/CE-licensed in Odoo source). Impact: companies with strict CE-only policies have no community alternative for País Vasco compliance — must accept Enterprise license OR pay for TBAI-specific commercial modules. _ES: No existe ningún módulo comunitario (OCA) de TicketBAI: solo la versión Enterprise para el País Vasco._

**ESG30. Facturae signing happens on canonicalized SubjectMatter, not raw XML** — `xml_utils.py` _canonicalize_node + _reference_digests. Impact: copying signed XML and modifying ANY whitespace (including line endings) invalidates the signature. Always use Odoo's regenerator, never edit by hand. _ES: Editar a mano el XML de Facturae (aunque sea un espacio o salto de línea) invalida la firma._

**ESG31. Facturae reason code default '10' (Detalle Operación) is generic** — `l10n_es_edi_tbai/models/account_move.py:117`, `AccountMove._compute_show_reset_to_draft_button`: _ES: Los abonos de Facturae salen con motivo genérico y los auditores prefieren un código de rectificación específico._
```python
default='10'
```
Impact: Facturae invoices with refunds default to "operation detail" reason — passes FACe but auditors prefer codes 03/04/05/06 for proper traceability. Always set explicitly for credit notes.

**ESG32. Facturae OCA has FACe submission, Enterprise does NOT** — `l10n_es_facturae_face` (OCA) wraps the signed XML in a SOAP envelope with WSSE signature and POSTs to FACe. Enterprise `l10n_es_edi_facturae` only generates the signed XML; user uploads to FACe portal manually. Impact: high-volume B2G clients NEED the OCA stack for automation. _ES: Facturae Enterprise no envía automáticamente a FACe; hay que subirlo a mano salvo que se use el módulo OCA._

**ESG33. Veri*Factu chain-index cannot be deleted** — `l10n_es_edi_verifactu/models/verifactu_document.py:180-184`, `L10nEsEdiVerifactuDocument._never_unlink_chained_documents`: _ES: No se puede borrar un documento Veri*Factu encadenado; solo cancelar y volver a registrar._
```python
@api.ondelete(at_uninstall=False)
def _never_unlink_chained_documents(self):
    if document.chain_index:
        raise UserError("You cannot delete Veri*Factu Documents that are part of the chain")
```
Impact: hash chain integrity is enforced. Fix-by-deletion not possible — must cancel + re-register.

**ESG34. Veri*Factu has 60-second wait between batches** — `l10n_es_edi_verifactu/models/verifactu_document.py:90-91`, `L10nEsEdiVerifactuDocument`: _ES: Facturación por encima de 1000 registros por minuto en Veri*Factu se retiene con esperas de 60 segundos entre lotes._
```python
# The waiting time is usually 60 seconds.
# In case we cannot send the records directly a cron will be triggered at the next possible time.
```
And `BATCH_LIMIT = 1000` (line 27). Impact: high-frequency invoicing > 1000/min has natural backpressure. The `l10n_es_edi_verifactu_next_batch_time` company field gates the next send.

**ESG35. Prorrata IVA has no Enterprise module** — Only OCA `l10n_es_vat_prorate` + `l10n_es_aeat_mod303_vat_prorate` provide prorrata logic. Enterprise `l10n_es_reports` does NOT compute prorrata coefficient. Impact: clients with mixed taxed/exempt outputs (e.g. medical practices billing both insurance + private) MUST use OCA stack on v18, AND have NO migration path to v19 (NOT PORTED). _ES: La prorrata de IVA solo existe en módulos OCA, no en Enterprise, y no hay ruta de migración a v19._

**ESG36. mod 303 OCA result_type computed every save** — OCA `l10n_es_aeat_mod303/models/mod303.py:466-501`, `L10nEsAeatMod303Report._compute_result_type`, depends on 6 fields. Impact: changing the bank account on a posted mod 303 silently flips result_type from `'I'` to `'U'` — re-validate before submitting BOE. Worse: changing it AFTER submission still updates the field (no archived state). _ES: Cambiar la cuenta bancaria en un modelo 303 ya validado cambia el tipo de resultado sin avisar._

**ESG37. `l10n_es_aeat.res_partner_aeat` is a hardcoded counterpart** — OCA `l10n-spain/l10n_es_aeat/models/l10n_es_aeat_report_tax_mapping.py:188`, `L10nEsAeatReportTaxMapping._prepare_counterpart_move_line`: _ES: Los asientos de regularización AEAT fallan si se borró el registro del partner de demo en producción._
```python
'partner_id': self.env.ref('l10n_es_aeat.res_partner_aeat').id
```
Used for regularization journal entries. Impact: if the demo data is removed in production, regularization moves fail. Always keep that XML record.

**ESG38. account_tax_balance dependency required by mod 347 OCA** — OCA `l10n_es_aeat_mod347/__manifest__.py:18` depends on `account_tax_balance`. Without it, the partner-aggregation by trimester breaks. Impact: when migrating from non-OCA setup, install `account_tax_balance` BEFORE mod 347. _ES: El modelo 347 OCA no agrega por trimestre si no está instalado el módulo account_tax_balance._

**ESG39. SII OSS clave 17 detection happens on repartition lines** — `l10n_es/models/account_tax.py:46-50`, `AccountTax._l10n_es_get_regime_code`: _ES: En abonos con OSS el SII reporta la clave equivocada porque solo la detecta en las líneas de venta._
```python
oss_tag = self.env.ref('l10n_eu_oss.tag_oss', raise_if_not_found=False)
if oss_tag and oss_tag in self.invoice_repartition_line_ids.tag_ids:
    return '17'
```
Impact: OSS clave is detected ONLY on `invoice_repartition_line_ids` (sales side), NOT on refund_repartition_line_ids. Refund OSS invoices may report wrong clave.

**ESG40. Spanish state codes (province) hardcoded in ENTERPRISE** — `l10n_es_reports/models/aeat_tax_reports.py:15-68`, `SPANISH_PROVINCES_REPORT_CODES` `SPANISH_PROVINCES_REPORT_CODES` maps state code → 2-digit numeric code (e.g. `'B': '08'` for Barcelona). OCA has parallel mapping in `spanish_states_mapping.py`. Impact: Ceuta/Melilla (CE/ME = 51/52) and special cases must match. If `res.country.state.code` is wrong, BOE rejects. _ES: El BOE rechaza el informe si el código de provincia del estado está mal (Ceuta, Melilla, casos especiales)._

## Veri\*Factu (`l10n_es_edi_verifactu` CE + `_pos`) — ESG41-55

Núcleo en [`verifactu.md`](verifactu.md). Sistema antifraude AEAT de registro encadenado (territorio común; País Vasco usa [TicketBAI](ticketbai.md)).

**ESG41. Los registros encadenados NUNCA se borran.** `verifactu_document.py::_never_unlink_chained_documents` (`:181`) lanza `UserError` en cualquier `unlink` de un documento con `chain_index` asignado. La cadena es inmutable por diseño legal; no hay "borrar y rehacer", solo anular.

**ESG42. Cancelar una factura Veri\*Factu NO la borra: crea una ANULACIÓN encadenada.** `account_move.py::l10n_es_edi_verifactu_button_cancel` (`:237`) genera un documento `cancellation` (RegistroAnulacion) que entra en la cadena como uno más; el asiento solo se cancela tras aceptarse en AEAT (`_cancel_after_sending`, `:1204`). El "cancelar" estándar de Odoo queda interceptado.

**ESG43. `chain_index` solo se asigna si el JSON se generó sin error.** `_create_for_record` (`:420`): si la serialización del registro falla, no se consume índice de la secuencia `no_gap`. Evita huecos, pero significa que un documento puede existir en estado intermedio sin índice (no enviable).

**ESG44. La secuencia de cadena es `no_gap` → NO hay encadenado en paralelo.** El índice usa `ir.sequence` `no_gap`, que **bloquea la fila** hasta el commit. Dos facturas posteadas concurrentemente para la misma compañía se serializan a nivel de BD; bajo carga, contención y esperas. No se puede paralelizar el alta de documentos de una misma compañía.

**ESG45. La huella encadena con el documento ANTERIOR; corromper uno invalida toda la cola.** `_fingerprint` (`:862`) mete la `Huella` previa (`l10n_es_edi_verifactu/models/res_company.py:89`, `ResCompany._l10n_es_edi_verifactu_get_last_document` `_..._get_last_document`). Si un documento se queda en estado inconsistente, los siguientes encadenan contra él y la AEAT puede rechazar en cascada. El orden importa: `trigger_next_batch` ordena por `chain_index`.

**ESG46. `test_environment` viene activado por DEFECTO (`default=True`).** `l10n_es_edi_verifactu/models/res_company.py:16`, `ResCompany.l10n_es_edi_verifactu_test_environment`. Una compañía recién configurada manda a los endpoints de **pruebas** (`prewww*.aeat.es`); si no lo desactivas, "funciona" pero NO presenta nada real ante Hacienda. Hay que ponerlo a False explícitamente para producción (`_..._get_endpoints`, `:43`).

**ESG47. Sin certificado de scope `verifactu`, falla en creación Y en envío.** `certificate/models/certificate.py:7`, `x509` define el scope. La dependencia es dura: no hay modo degradado. Configurar Veri\*Factu sin subir el certificado correcto rompe el posteo de facturas, no solo el envío.

**ESG48. El envío es por LOTES con límite de 1000 y espera impuesta por AEAT.** `trigger_next_batch` (`:885`) / `_send_batch` (`:955`): `BATCH_LIMIT=1000`; entre lotes respeta `l10n_es_edi_verifactu_next_batch_time` (`l10n_es_edi_verifactu/models/res_company.py:27`, `class ResCompany`, ~60s). El registro NO es inmediato al postear; hay latencia y agrupación. No asumas confirmación síncrona.

**ESG49. Un timeout de red = posible DUPLICADO en AEAT.** Si el SOAP se envía pero la respuesta se pierde, la AEAT puede haber registrado el alta mientras Odoo lo ve como pendiente. La huella+IDFactura permiten a Hacienda deduplicar, pero el reintento debe respetar la cadena; no reenvíes manualmente con otro índice.

**ESG50. `state` lo fija la respuesta de la AEAT, no el usuario.** `l10n_es_edi_verifactu/models/verifactu_document.py:154`, `class L10nEsEdiVerifactuDocument`: `accepted` (Correcto) / `registered_with_errors` (AceptadoConErrores) / `rejected`. **`registered_with_errors` SÍ cuenta como presentado** ante Hacienda (aceptado con errores ≠ rechazado); no lo trates como fallo a reintentar.

**ESG51. El QR de cotejo apunta a un entorno según test/producción.** `_get_qr_code_img_url` (`:293`) construye la URL `ValidarQR`; en modo test el QR valida contra el entorno de pruebas. Un ticket impreso en pruebas lleva un QR que NO valida en la web real de la AEAT.

**ESG52. El POS genera un documento por TICKET al cobrar, no al facturar.** `l10n_es_edi_verifactu_pos` (`pos_order.py::action_pos_order_paid`): cada cobro sin factura emite su propio registro (ticket simplificado). Volumen de POS = volumen de documentos encadenados; la cadena crece por cada venta de caja.

**ESG53. Un ticket de POS NO se puede Veri\*Factu-cancelar.** No hay "anulación de ticket": la vía correcta es facturar el ticket (lo que genera la factura encadenada) y luego anular la factura. Devoluciones en POS = nuevo documento, no anulación del original.

**ESG54. El envío hace COMMIT por documento.** Para no perder el progreso de un lote ante un fallo a mitad, el proceso confirma transacción por documento aceptado. Efecto: no puedes envolver el envío en una transacción atómica que revierta todo; un lote parcialmente enviado deja documentos ya presentados.

**ESG55. Veri\*Factu NO manda emails ni notifica al cliente.** El sistema solo presenta a la AEAT; no hay envío automático de la factura ni del justificante al partner. Cualquier comunicación al cliente es flujo aparte (`account_move_send` / plantilla), independiente del estado del registro.
