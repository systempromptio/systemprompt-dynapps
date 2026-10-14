# Facturae — B2G Electronic Invoicing (Public Administration)

**Legal scope**: Mandatory format for invoices addressed to Spanish public administrations (central, regional, local). Per Ley 25/2013 and Real Decreto 1619/2012, invoices for amounts >= 5.000€ to public admin MUST be sent in Facturae 3.2.x format via the FACe gateway.

**Format**: XML 3.2.2 (current) / 3.2 (legacy). Signed with XAdES-BES.

## 1. Two paths

### Enterprise — `l10n_es_edi_facturae`

Source: `odoo18/addons/l10n_es_edi_facturae/`
- `models/account_move.py` — 840 lines
- `models/res_partner.py` — 87 lines (DIRe codes)
- `models/uom_uom.py` — 45 lines (UOM mapping to Facturae codes)
- `xml_utils.py` — canonicalization

**Capability**: Generates and signs XML. Does **NOT** submit to FACe. User downloads XML and uploads manually to FACe portal.

### OCA — `l10n_es_facturae` + `l10n_es_facturae_face`

- `l10n_es_facturae` (1226 lines models) — XML generation + signing
- `l10n_es_facturae_face` — submission to FACe via SOAP + status polling cron
- `l10n_es_facturae_sale_stock` — adds delivery note references

**v19 status**: NOT PORTED → companies invoicing public admin and on v19 must use Enterprise (no auto-FACe) OR build custom.

## 2. XML structure

Facturae 3.2.2 root: `<fe:Facturae>`. Top-level blocks:
1. `<FileHeader>` — document metadata (batch ID, modality)
2. `<Parties>` — `<SellerParty>` + `<BuyerParty>` with full tax + admin centre data
3. `<Invoices>` — repeated `<Invoice>` blocks (1 typically, can be many)
4. `<ds:Signature>` — XAdES-BES signature

### Invoice required nodes
```xml
<Invoice>
  <InvoiceHeader>
    <InvoiceNumber>{name}</InvoiceNumber>
    <InvoiceSeriesCode>{prefix}</InvoiceSeriesCode>
    <InvoiceDocumentType>FC|FA|...</InvoiceDocumentType>  <!-- FC=Factura completa -->
    <InvoiceClass>OO|OR|OC|...</InvoiceClass>            <!-- OO=Original -->
  </InvoiceHeader>
  <InvoiceIssueData>
    <IssueDate>{invoice_date}</IssueDate>
    <OperationDate>{delivery_date}</OperationDate>
    <InvoicingPeriod>{period_start} - {period_end}</InvoicingPeriod>
    <PaymentMeans>04</PaymentMeans>                      <!-- 04=transfer -->
  </InvoiceIssueData>
  <TaxesOutputs>...</TaxesOutputs>
  <TaxesWithheld>...</TaxesWithheld>
  <InvoiceTotals>...</InvoiceTotals>
  <Items>{lines}</Items>
  <PaymentDetails>{IBAN}</PaymentDetails>
  <CorrectiveInvoice>...</CorrectiveInvoice>             <!-- if refund -->
</Invoice>
```

## 3. Reason codes (rectificativa)

`l10n_es_edi_facturae/models/account_move.py:54-77`, `SPANISH_CREDIT_REASON_TYPE` — `SPANISH_CREDIT_REASON_TYPE`:

| Code | Reason |
|---|---|
| `01` | Número de la factura |
| `02` | Serie de la factura |
| `03` | Fecha expedición |
| `04` | Razón Social Emisor |
| `05` | Razón Social Receptor |
| `06` | Identificación fiscal Emisor |
| `07` | Identificación fiscal Receptor |
| `08` | Domicilio Emisor |
| `09` | Domicilio Receptor |
| `10` | Detalle Operación (default) |
| `11` | Porcentaje impositivo |
| `12` | Cuota tributaria |
| `13` | Fecha/Periodo |
| `14` | Clase de factura |
| `15` | Literales legales |
| `16` | Base imponible |
| `80` | Cálculo cuotas repercutidas |
| `81` | Cálculo cuotas retenidas |
| `82` | BI modificada por devolución envases |
| `83` | BI modificada por descuentos |
| `84` | BI modificada por resolución firme judicial |
| `85` | BI modificada por concurso (Art. 80.3) |

Field: `move.l10n_es_edi_facturae_reason_code`. Default `'10'`.

## 4. Payment means

`l10n_es_edi_facturae/models/account_move.py:121-141`, `AccountMove.l10n_es_payment_means`:
| Code | Means |
|---|---|
| `01` | In cash |
| `02` | Direct debit |
| `03` | Receipt |
| `04` | Credit transfer (DEFAULT) |
| `05` | Accepted bill of exchange |
| `06` | Documentary credit |
| `07` | Contract award |
| `08` | Bill of exchange |
| `09` | Transferable promissory note |
| `10` | Non transferable promissory note |
| `11` | Cheque |
| `12` | Open account reimbursement |
| `13` | Special payment |
| `14` | Set-off |
| `15` | Postgiro |
| `16` | Certified cheque |
| `17` | Banker's draft |
| `18` | Cash on delivery |
| `19` | Payment by card |

## 5. Country code mapping

`l10n_es/models/account_move.py:21-50`, `AccountMove._compute_l10n_es_is_simplified`: COUNTRY_CODE_MAP — Facturae uses **ISO 3166-1 alpha-3** (3-letter codes), Odoo uses alpha-2. Mapping table covers all countries.

E.g. `ES → ESP`, `FR → FRA`, `US → USA`. There's also `REVERSED_COUNTRY_CODE` for parsing incoming invoices.

## 6. UOM mapping

`uom_uom.py` — maps `uom.uom` records to Facturae UOM codes (CL5 — UN/ECE Recommendation 20):
- units → `01`
- kg → `02` (kilogram)
- liter → `04`
- meter → `05`
- m² → `06`
- m³ → `07`
- ...
- service hour → `30`

If a UOM has no mapping, defaults to `01` (unit).

## 7. Signing flow

### Enterprise (XAdES-BES)
`l10n_es_edi_facturae/xml_utils.py` + `signature_templates.xml` (data) define the XAdES template.
1. Build the unsigned Facturae XML.
2. Compute reference digests (SHA-256) over canonicalized nodes.
3. Build `<ds:SignedInfo>` with the digests.
4. Sign `<ds:SignedInfo>` with private key (RSA-SHA256).
5. Insert `<ds:Signature>` block under root.

The `certificate.certificate` framework provides the signing key (scope='facturae').

### Hash function
SHA-256 (modern). Old Facturae 3.2 uses SHA-1 — Enterprise only emits 3.2.2 with SHA-256.

## 8. DIRe codes (3 admin centre codes)

Spanish public admin entities have a DIRe (Directorio Común de Unidades Orgánicas) hierarchy:
- **Receptor (oficina contable)**: where the invoice is registered (e.g. L01000119)
- **Pagador (órgano gestor)**: who pays (e.g. L01000220)
- **Tramitador (unidad tramitadora)**: who processes (e.g. L01000300)

These are mandatory in the XML for FACe submission. `res_partner.py` adds fields `l10n_es_edi_facturae_dire_*` per partner.

In FACe, you can list/search DIRe codes via the FACe directory web service.

## 9. FACe submission (OCA only)

`l10n_es_facturae_face` flow:
1. Generate signed Facturae XML
2. Wrap in SOAP envelope with WSSE signature
3. POST to FACe SOAP endpoint:
   - Test: `https://se-face-webservice.redsara.es/facturasspp2/...`
   - Prod: `https://webservice.face.gob.es/facturasspp2`
4. Receive registration number (`numeroRegistro`)
5. Cron `cron_data.xml` polls `consultarFactura` periodically to update state:
   - `1200` = registered
   - `1300` = registered + accounting registered
   - `2400` = paid
   - `2500` = rejected by accounting
   - `4500` = rejected (final)

State stored in `edi.exchange.record` (provided by `edi_account_core_oca` dependency).

## 10. PDF embedding

Facturae allows embedding the original PDF as `<AttachedDocument>` (CDATA-encoded base64). Useful for visual reference.

`account_move.py:_l10n_es_edi_facturae_get_default_enable` (line 159):
```python
return not self.invoice_pdf_report_id  # don't enable if PDF already attached
```
Suggests Facturae XML and the PDF are alternatives, not always combined.

## 11. Validation pre-flight

`_l10n_es_edi_facturae_export_data_check()` line 149:
```python
return {
    **self.mapped("company_id")._l10n_es_edi_facturae_export_check(),
    **self.mapped("partner_id")._l10n_es_edi_facturae_export_check(),
}
```
Returns dict of errors for the actionable_errors widget. Checks:
- Company VAT, address, certificate
- Partner VAT, address, country, DIRe codes (if public admin)
- Bank account (IBAN format)

## 12. Known limitations

- **Enterprise has NO FACe submission** — the XML must be uploaded manually.
- Simplified invoices (`l10n_es_is_simplified=True`) are SKIPPED for Facturae (line 163) — public admin invoices cannot be simplified.
- Receipts (`is_invoice(include_receipts=True)`) ARE supported but rare for B2G.
- Multi-currency: Facturae spec allows EUR only realistically — `InvoiceTotals/InvoiceCurrencyCode` should be EUR for AEAT compatibility.
- `l10n_es_edi_facturae_face_base_handler.py` (OCA) shows there's also `l10n_es_facturae_face_update_handler` and `_cancel_handler` — full CRUD over the FACe lifecycle.

## 13. Other facturae variants

- **FACeB2B** (B2B mandatory by 2026 — Crea y Crece law) — different gateway, same XML format. NO Odoo module yet (CE or OCA) — anticipated for 2025/2026.
- **Facturae 3.3** — newer schema with QR code support — NOT YET adopted.
