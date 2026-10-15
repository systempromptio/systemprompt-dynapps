<!-- verified: v18.0 @ 2026-07 (l10n_es core + l10n_es_reports EE + OCA l10n-spain v18; anclas verificadas vs fuente) -->
# Localización España (`l10n_es`) — Capacidades (¿Odoo puede…?)

> Eje **consultor**: "¿se puede X?" + **complejidad** + **origen**, anclado a fuente. Método en [`../../_index/HOW-TO-EXTEND.md`](../../_index/HOW-TO-EXTEND.md). Mecánica en [`overview.md`](overview.md)/[`sii.md`](sii.md)/[`ticketbai.md`](ticketbai.md)/[`facturae.md`](facturae.md)/[`aeat_models.md`](aeat_models.md); trampas en [`gotchas.md`](gotchas.md) (ESG*). Diffs v19 en [`v19_diff.md`](v19_diff.md).
> Veredicto: **NATIVO** (core CE) · **EE** (nativo Enterprise) · **OCA** (módulo comunidad `l10n-spain`, de facto en ES) · **CONFIG** (activar flag/dato) · **DESARROLLO** (no existe, hay punto de extensión) · **NO**.
> Regla ES: media localización fiscal vive en **OCA** o **Enterprise**, no en core CE. Cada fila dice el origen para no prometer "de fábrica" lo que exige addon.

## Suministro a AEAT / factura electrónica

**ESCAP1. ¿SII (Suministro Inmediato de Información)?** — **NATIVO** · media · `l10n_es_edi_sii/models/account_edi_format.py::_l10n_es_edi_sii_post_invoices` + `res_company.py::l10n_es_sii_tax_agency` · Envío inmediato a AEAT/foral; requiere certificado (`l10n_es_sii_certificate_ids`) y agencia. Alternativa OCA `l10n_es_aeat_sii_oca`. **v19**: el envío pasa al framework `account_move_send` (ver `v19_diff.md`).

**ESCAP2. ¿TicketBAI (País Vasco)?** — **NATIVO** · media · `l10n_es_edi_tbai/models/account_move.py::l10n_es_tbai_send_bill` · Álava/Bizkaia/Gipuzkoa; encadenado de facturas (`_l10n_es_tbai_lock_move`). En POS con `l10n_es_edi_tbai_pos`.

**ESCAP3. ¿Verifactu (sistema antifraude AEAT)?** — **NATIVO** · media · `l10n_es_edi_verifactu/models/verifactu_document.py` + `account_move_send.py` · Registro de facturación Verifactu con certificado; en POS con `l10n_es_edi_verifactu_pos`.

**ESCAP4. ¿Factura electrónica Facturae (B2G / FACe)?** — **NATIVO (genera XML) / DESARROLLO (envío FACe)** · media · `l10n_es_edi_facturae/models/account_move.py` · Genera y firma el XML Facturae 3.2.x; el **envío directo a FACe** no es nativo (se descarga/sube manual o addon externo).

## Modelos / declaraciones AEAT

**ESCAP5. ¿Modelo 303 (IVA trimestral)?** — **EE** (o **OCA**) · baja · casillas en core `l10n_es/data/mod303.xml`; export BOE en `l10n_es_reports/data/mod303.xml` + `wizard/aeat_boe_export_wizards.py`. Alternativa OCA `l10n_es_aeat_mod303`. El **cálculo de casillas** es core; la **presentación/BOE** es EE u OCA.

**ESCAP6. ¿Modelo 390 (resumen anual IVA)?** — **EE** (o **OCA**) · baja · `l10n_es_reports/data/mod390.xml` / OCA `l10n_es_aeat_mod390`. Casillas base en core `l10n_es/data/mod390/`.

**ESCAP7. ¿Modelo 347 (operaciones con terceros >3.005 €)?** — **EE** (o **OCA**) · baja · `l10n_es_reports/data/mod347.xml` / OCA `l10n_es_aeat_mod347`. No existe en core CE.

**ESCAP8. ¿Modelo 349 (operaciones intracomunitarias)?** — **EE** (o **OCA**) · baja · `l10n_es_reports/data/mod349.xml` / OCA `l10n_es_aeat_mod349`.

**ESCAP9. ¿Modelos 111 / 115 / 190 (retenciones IRPF)?** — **111/115 EE, 190 solo OCA** · media · casillas 111/115 en core `l10n_es/data/mod111.xml`,`mod115.xml` + export `l10n_es_reports/data/mod111.xml`,`mod115.xml`; el **190** (resumen anual) NO está en EE → OCA `l10n_es_aeat_mod190`.

**ESCAP10. ¿Modelo 130 (pago fraccionado estimación directa)?** — **NATIVO+EE** (o **OCA**) · baja · core `l10n_es_modelo130` + export `l10n_es_reports_modelo130`. Alternativa OCA `l10n_es_aeat_mod130`.

**ESCAP11. ¿Libros registro de IVA (exportación)?** — **EE** (o **OCA**) · media · `l10n_es_reports` (BOE/libros, `tests/test_libros_export.py`) / OCA `l10n_es_vat_book`. No en core CE.

## IVA / impuestos

**ESCAP12. ¿Plan contable español (PGCE)?** — **NATIVO** · baja · `l10n_es/models/template_es_pymes.py::@template('es_pymes')` + `template_es_full.py::@template('es_full')` (+ asociaciones). Se elige al instalar la compañía ES; PGCE pymes/normal completos de fábrica.

**ESCAP13. ¿Retenciones IRPF en factura de proveedor?** — **NATIVO** · baja · impuestos plantilla `p_irpf19`/`p_irpf7`… en `l10n_es/data/template/account.tax-es_common.csv` (grupo `tax_group_retenciones_*`, tag `+mod111[08]`). Retención de compra lista de fábrica.

**ESCAP14. ¿Recargo de equivalencia?** — **NATIVO** · baja · impuestos `s_req*` en `account.tax-es_common_mainland.csv` (grupo `tax_group_recargo_*`, tags mod303). Todos los tipos RE de fábrica; se aplican vía posición fiscal del cliente.

**ESCAP15. ¿IVA de caja (RECC — criterio de caja)?** — **CONFIG parcial / OCA** · media · el mecanismo de exigibilidad **on payment** es nativo de `account` (impuestos cash-basis); el **régimen RECC completo** (libro, marca 7.º sección, límites) requiere OCA (`l10n_es_vat_book`) o desarrollo. No prometer "RECC de fábrica".

**ESCAP16. ¿OSS / ventanilla única (One Stop Shop)?** — **CONFIG (core `l10n_eu_oss`) / OCA (modelos)** · media · el cálculo OSS es genérico UE (`l10n_eu_oss`); los **modelos 369 y variantes OSS** son OCA (`l10n_es_aeat_mod369`, `l10n_es_aeat_mod303_oss`, `l10n_es_vat_book_oss`).

**ESCAP17. ¿Prorrata de IVA?** — **OCA** · media · `l10n_es_aeat_mod303_vat_prorate` / `l10n_es_aeat_mod390_vat_prorate` / `l10n_es_vat_prorate`. No nativo core ni EE.

## Cobros / pagos / banca

**ESCAP18. ¿Remesas SEPA (adeudos 19.14 / transferencias 34.14)?** — **EE (SDD) / OCA** · media · adeudos directos en `account_sepa_direct_debit` (EE) + `payment_sepa_direct_debit`; transferencias SEPA CT vía `account_batch_payment` (EE). FSDD B2B y variantes ES en OCA `l10n_es_account_banking_sepa_fsdd`.

**ESCAP19. ¿Importación extracto bancario Norma 43 (N43)?** — **OCA** · baja · `l10n_es_account_statement_import_n43`. No nativo (core solo OFX/CAMT/CSV genéricos).

**ESCAP20. ¿Pagarés / confirming / efectos comerciales?** — **OCA / DESARROLLO** · media · confirming en OCA `l10n_es_payment_order_confirming_aef`/`_sabadell`, pagaré CaixaBank `account_promissory_note_caixabank`. Efectos/pagarés genéricos NO nativos; caso a caso por banco.

## Validación, rectificación, numeración, informes

**ESCAP21. ¿Validación NIF / VIES?** — **NATIVO** · baja · `base_vat/models/res_partner.py::simple_vat_check` (formato NIF/CIF) + `vies_valid` (`_compute_vies_valid`, chequeo VIES online). De fábrica al depender `l10n_es` de `base_vat`.

**ESCAP22. ¿Facturas rectificativas con motivo (R1–R4)?** — **NATIVO** · media · abono estándar `account.move` (reversal); Facturae mapea el original en `l10n_es_edi_facturae/models/account_move.py::_l10n_es_edi_facturae_get_refunded_invoices` y exige tipo/motivo de rectificación. Motivo R1–R4 se informa en el EDI (Facturae/TBAI/SII).

**ESCAP23. ¿Numeración por serie / registro fiscal?** — **NATIVO (secuencias por diario) / CONFIG** · baja · numeración correlativa por diario nativa de `account`; `l10n_es_edi_sii` añade `l10n_es_registration_date` (fecha registro contable). Series fiscales = configurar diarios; no hay "libro de series" adicional.

**ESCAP24. ¿Informes financieros ES (balance PGCE, PyG)?** — **EE** · baja · `l10n_es_reports/data/pymes_balance_sheet_report_data.xml` + `pymes_profit_and_loss_report_data.xml` (+ full/assoc). Balance y PyG con estructura PGCE oficial; sobre el motor `account_reports` (Enterprise). En CE, solo informes contables genéricos.
