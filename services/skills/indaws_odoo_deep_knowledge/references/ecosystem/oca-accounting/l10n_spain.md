# OCA l10n-spain — Odoo 18.0

Complete Spanish tax/accounting localization. 53+ modules.
Source: `OCA/v18/l10n-spain/`


## 1. AEAT Framework (Base + 12 Models)

### Base: `l10n_es_aeat`

Declaration lifecycle: **draft → calculated → done → posted → cancelled**

Core models: `l10n.es.aeat.report` (abstract base), `l10n.es.aeat.tax.line`, `l10n.es.aeat.map.tax`, `aeat.model.export.config`, `aeat.certificate`

Flow: User creates declaration → selects period → **Calculate** (queries account.move by date range + tax mapping) → populates tax_lines → **Post** (optional journal entry) → **Submit** to AEAT.

### Declaration Models

| Module | Mod# | Name | Freq | What |
|--------|------|------|------|------|
| `l10n_es_aeat_mod303` | 303 | IVA trimestral | Q/Y | **IVA principal** — obligatorio para todos |
| `l10n_es_aeat_mod390` | 390 | Resumen anual IVA | Annual | Resumen del 303 — depende de mod303 |
| `l10n_es_aeat_mod111` | 111 | Retenciones IRPF | Quarterly | Retenciones trabajo/profesionales |
| `l10n_es_aeat_mod115` | 115 | Retenciones alquileres | Quarterly | Retenciones arrendamientos |
| `l10n_es_aeat_mod123` | 123 | Retenciones capital | Quarterly | Retenciones capital mobiliario |
| `l10n_es_aeat_mod130` | 130 | Pago fraccionado IRPF | Quarterly | Autónomos en estimación directa |
| `l10n_es_aeat_mod190` | 190 | Resumen retenciones | Annual | Resumen anual de 111/115/123 |
| `l10n_es_aeat_mod216` | 216 | No residentes | Annual | Retenciones a no residentes |
| `l10n_es_aeat_mod296` | 296 | Resumen no residentes | Annual | Resumen anual de 216 |
| `l10n_es_aeat_mod347` | 347 | Operaciones >3.005€ | Annual | Declaración de terceros |
| `l10n_es_aeat_mod349` | 349 | Intracomunitarias | Annual | Operaciones EU B2B |
| `l10n_es_aeat_mod369` | 369 | OSS (ventanilla única) | M/Q | IVA B2C no residente EU |

**Variants:** `_mod303_oss`, `_mod303_vat_prorate`, `_mod390_oss`, `_mod390_vat_prorate` — auto-install.


## 2. SII (Suministro Inmediato de Información)

| Module | What |
|--------|------|
| `l10n_es_aeat_sii_oca` | **Core:** envío facturas a AEAT en <3 días. SOAP/zeep. Tracking estado por factura. Claves registro. Test/prod endpoints. |
| `l10n_es_aeat_sii_match` | Verifica facturas enviadas vs registros AEAT (deepdiff). Detecta discrepancias. |
| `l10n_es_aeat_sii_oss` | SII + OSS (ventas B2C EU). Auto-install con `l10n_eu_oss_oca`. |
| `l10n_es_aeat_sii_invoice_summary` | Facturas simplificadas POS: agrupa ventas diarias en factura resumen para SII. |

**vs Odoo nativo:** OCA pre-existe, más modular, community-maintained. Indaws tiene `indaws_l10n_es_edi_sii` como bridge/mejora.


## 3. Verifactu

`l10n_es_verifactu_oca` — Nueva regulación 2024+: verificación electrónica de facturas.

Models: `verifactu.map`, `verifactu.registration.keys`, `verifactu.invoice.entry`, `verifactu.chaining`, `verifactu.invoice.entry.response`

Workflow: factura → submission → reception → validation → acceptance. Cron para polling estado.

**Indaws:** `indaws_l10n_es_edi_verifactu` como bridge complementario.


## 4. Facturae (Factura Electrónica)

| Module | What |
|--------|------|
| `l10n_es_facturae` | Genera XML Facturae (UBL Spain). Firma digital (xmlsig). Exporta desde factura. |
| `l10n_es_facturae_face` | Upload a FACe (portal B2G). EDI auto-envío + polling estado. |
| `l10n_es_facturae_sale_stock` | Añade datos de entrega (picking) al XML Facturae. |

**Obligatorio para:** facturación a administración pública española.


## 5. Libro de IVA (VAT Book)

| Module | What |
|--------|------|
| `l10n_es_vat_book` | Libros registro de facturas emitidas/recibidas. XLSX export. Tax mapping configurable. |
| `l10n_es_vat_book_invoice_summary` | Facturas resumen (POS) en libro IVA. |
| `l10n_es_vat_book_oss` | Columnas OSS en libro IVA. |
| `l10n_es_vat_book_pos` | Libro IVA filtrado por sesiones POS. |


## 6. Prorrata de IVA

`l10n_es_vat_prorate` — IVA parcialmente deducible (uso mixto personal/profesional).

Config: porcentaje de prorrata en empresa. Wizard para recalcular IVA deducible histórico. Variants: `_mod303_vat_prorate`, `_mod390_vat_prorate`.


## 7. Banking & Payments

| Module | What |
|--------|------|
| `l10n_es_account_banking_sepa_fsdd` | SEPA DD español (norma FSDD) |
| `l10n_es_account_statement_import_n43` | Importar extractos bancarios formato N43 (norma española legacy) |
| `l10n_es_payment_order_confirming_aef` | Confirming formato AEF (Asociación Española de Factoring) |
| `l10n_es_payment_order_confirming_sabadell` | Confirming formato Sabadell |
| `payment_redsys` | Pasarela de pago Redsys (Visa/Mastercard Spain) |
| `account_promissory_note_caixabank` | Pagarés CaixaBank formato A4 |


## 8. Partner & Location

| Module | What |
|--------|------|
| `l10n_es_partner` | NIF validation, IBAN parsing, bank account views |
| `l10n_es_partner_mercantil` | Datos registro mercantil |
| `l10n_es_toponyms` | Municipios/provincias (Geonames import) |
| `l10n_es_location_nuts` | Regiones NUTS (clasificación EU) |
| `l10n_es_cnae` / `l10n_ca_es_cnae` | Códigos CNAE 2009 (actividades económicas) |


## 9. Other

| Module | What |
|--------|------|
| `l10n_es_account_asset` | Amortización adaptada España (depende OCA `account_asset_management`) |
| `l10n_es_mis_report` | Templates MIS Builder para P&L/Balance España |
| `l10n_es_intrastat_report` | Intrastat para comercio EU |
| `l10n_es_pos_oca` | POS adaptado España (tickets legales) |
| `l10n_es_pos_sii` | POS + SII (envío automático) |
| `l10n_es_atc` + `l10n_es_atc_mod420` | Impuestos Canarias (ATC/IPSI) |
| `l10n_es_hr_collective_agreement` | Convenios colectivos laborales |
| `l10n_es_aeat_partner_check` | Validación datos partner para AEAT |
| `delivery_seur_atlas`, `delivery_gls_asm`, `delivery_dhl_parcel` | Carriers españoles |


## 10. Dependency Graph

```
l10n_es_aeat (base)
├── mod111, mod115, mod123, mod130, mod190, mod216, mod296
├── mod303 → mod303_oss, mod303_vat_prorate
├── mod347, mod349, mod369
├── mod390 → mod390_oss, mod390_vat_prorate
├── l10n_es_aeat_sii_oca → sii_match, sii_oss, sii_invoice_summary
├── l10n_es_verifactu_oca
├── l10n_es_vat_book → vat_book_invoice_summary, vat_book_oss, vat_book_pos
├── l10n_es_vat_prorate
├── l10n_es_atc → l10n_es_atc_mod420
└── l10n_es_aeat_partner_check

l10n_es_facturae → facturae_face, facturae_sale_stock
l10n_es_partner → l10n_es_partner_mercantil
l10n_es_pos_oca → l10n_es_pos_sii
```


## 11. What Indaws Already Uses from l10n-spain

| OCA Module | Indaws Status |
|-----------|---------------|
| `l10n_es_account_banking_sepa_fsdd` | ✅ Installed |
| `l10n_es_aeat_mod190` | ✅ Installed |
| `l10n_es_aeat_mod216` | ✅ Installed |
| `l10n_es_aeat_mod390` | ✅ Installed |
| `l10n_es_dua` | ✅ Installed |
| `payment_redsys` | ✅ Installed |

**Indaws custom bridges:** `indaws_l10n_es_edi_sii` (enhances SII), `indaws_l10n_es_edi_verifactu` (enhances Verifactu), `indaws_l10n_es_pos` (POS Spain), `indaws_l10n_es_account_ipc` (IPC index).
