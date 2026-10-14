# SII — Suministro Inmediato de Información

**Legal scope** (Procedimiento G417): Real-time submission of issued/received invoice books to AEAT (or Bizkaia/Gipuzkoa). Mandatory for:
- Annual turnover > 6M€
- Inscripción en REDEME (Régimen de Devolución Mensual)
- Grupos de IVA
- Voluntary opt-in

**Submission deadline**: 4 calendar days from invoice issue/registration (or before next VAT period closes, whichever is earlier).

## 1. Three implementations

### 1.a Enterprise (`l10n_es_edi_sii`) — file:line refs

Source: `odoo18/addons/l10n_es_edi_sii/models/account_edi_format.py` (684 lines)

- **Format**: JSON in DB (`jsondump.json` attached to invoice) → SOAP envelope built by zeep.
- **State**: Tracked via `account.edi.document.state` (account_edi framework).
- **Trigger**: `_get_move_applicability()` line 575 returns dict with `post`/`cancel` callbacks if `move.l10n_es_edi_is_required`.
- **Batching**: `post_batching: lambda invoice: (invoice.move_type, invoice.l10n_es_edi_csv)` — invoices share a SOAP batch only if same move_type and same CSV (cluster of related invoices).

### 1.b OCA (`l10n_es_aeat_sii_oca`)

Source: `OCA/v18/l10n-spain/l10n_es_aeat_sii_oca/models/sii_mixin.py` (888 lines), `account_move.py` (864 lines)

- **Format**: dict serialized to JSON in `aeat_content_sent` field on `aeat.mixin`.
- **State**: own `aeat_state` selection: `not_sent / sent / sent_w_errors / cancelled / sent_modified / cancelled_modified`.
- **Trigger**: button on invoice `confirm_one_document()` → `_send_document_to_sii()` (line 795).
- **Async**: Uses a NEW cursor (`Registry(self.env.cr.dbname).cursor()` line 871) on exception so partial state is preserved → invoice gets marked `aeat_send_failed=True` even on rollback.

### 1.c Indaws bridge (per memory)
Bridge layer adds match logic + extended exempt reasons. See SII analysis memory note for paridad-OCA roadmap (50-150h estimated).

## 2. Tax agency endpoints

### Enterprise — `l10n_es_edi_sii/models/account_edi_format.py:376-410`, `AccountEdiFormat._l10n_es_edi_web_service_aeat_vals`

```python
# AEAT (mainland Spain)
url:      https://www2.agenciatributaria.gob.es/static_files/common/internet/dep/aplicaciones/es/aeat/ssii_1_1/fact/ws/SuministroFactEmitidas.wsdl
test_url: https://prewww1.aeat.es/wlpl/SSII-FACT/ws/fe/SiiFactFEV1SOAP

# Bizkaia (separate provincial system)
url:      https://www.bizkaia.eus/ogasuna/sii/documentos/SuministroFactEmitidas.wsdl
test_url: https://pruapps.bizkaia.eus/SSII-FACT/ws/fe/SiiFactFEV1SOAP

# Gipuzkoa
url:      https://egoitza.gipuzkoa.eus/ogasuna/sii/ficheros/v1.1/SuministroFactEmitidas.wsdl
test_url: https://sii-prep.egoitza.gipuzkoa.eus/JBS/HACI/SSII-FACT/ws/fe/SiiFactFEV1SOAP
```

Mirror endpoints exist for `SuministroFactRecibidas` (vendor bills).

**Test mode quirk**: `l10n_es_edi_sii/models/account_edi_format.py:468-474`, `AccountEdiFormat._l10n_es_edi_call_web_service_sign_common` — if `l10n_es_sii_test_env=True` AND `connection_vals` has no `test_url`, the service name is suffixed with `'Pruebas'` (e.g. `SuministroFactEmitidasPruebas`). Otherwise `_binding_options['address']` is overwritten.

### Cipher patch
```python
EUSKADI_CIPHERS = "DEFAULT:!DH"   # account_edi_format.py:15
session.mount('https://', CertificateAdapter(ciphers=EUSKADI_CIPHERS))
```
Required because Bizkaia/Gipuzkoa servers use small DH keys → modern OpenSSL refuses unless we drop DH ciphers. Used for ALL agencies (including AEAT) for safety.

## 3. Authentication — certificate flow

### Enterprise (`certificate.certificate` model)
- `company.l10n_es_sii_certificate_id` is a `certificate.certificate` record (PEM/PKCS12).
- `CertificateAdapter` from `odoo.addons.certificate.tools` converts the certificate to a temp PEM pair for `requests.Session.cert`.
- The certificate is `scope='sii'` (filtered).

### OCA (`l10n_es_aeat`)
- `aeat.certificate` model (older, predates `certificate.certificate`).
- Password-encrypted PKCS12 in DB.
- File: `l10n_es_aeat/models/aeat_certificate.py` (82 lines).

## 4. Tax mapping (the source-of-truth conflict)

### Enterprise — l10n_es_type-driven

`l10n_es_edi_sii/models/account_edi_format.py:25-54`, `AccountEdiFormat._l10n_es_edi_get_invoices_tax_details_info`:
```python
def grouping_key_generator(base_line, tax_data):
    tax = tax_data['tax']
    return {
        'applied_tax_amount': tax.amount,
        'l10n_es_type': tax.l10n_es_type,
        'l10n_es_exempt_reason': tax.l10n_es_exempt_reason if tax.l10n_es_type == 'exento' else False,
        'l10n_es_bien_inversion': tax.l10n_es_bien_inversion,
    }
```

The grouping_key drives the SII XML structure. Each `l10n_es_type` maps to a node:

| `l10n_es_type` | SII XML node | Field |
|---|---|---|
| `sujeto` | `Sujeta/NoExenta/DesgloseIVA/DetalleIVA` (TipoNoExenta=S1) | TipoImpositivo, BaseImponible, CuotaRepercutida |
| `sujeto_isp` | `Sujeta/NoExenta/DesgloseIVA/DetalleIVA` (TipoNoExenta=S2) | Same; reverse charge |
| `sujeto_agricultura` | `DetalleIVA` (vendor bills only) | PorcentCompensacionREAGYP, ImporteCompensacionREAGYP |
| `exento` | `Sujeta/Exenta/DetalleExenta` | BaseImponible, CausaExencion (E1-E6) |
| `no_sujeto` | `NoSujeta/ImportePorArticulos7_14_Otros` | base only |
| `no_sujeto_loc` | `NoSujeta/ImporteTAIReglasLocalizacion` | base only |
| `recargo` | nested `CuotaRecargoEquivalencia` + `TipoRecargoEquivalencia` | inside parent sujeto |
| `retencion` | EXCLUDED from tax detail; subtracted from `ImporteTotal` | — |
| `dua` | `TipoFactura='F5'` | base & tax in DetalleIVA |
| `ignore` | filtered out completely | — |

### OCA — Tax XMLID + map

`l10n_es_aeat_sii_oca/models/aeat_sii_map.py:70`, `class AeatSiiMapLines` defines `aeat.sii.map.line` records keyed by:
- `code` (e.g. 'sfess_e' = "service exempt sales")
- `tax_xml_id_ids` (M2M to taxes by XMLID)
- `date_from` / `date_to` (versioning)

The mixin `_get_aeat_taxes_map(codes, date)` (sii_mixin.py:185) resolves codes → tax recordset for the given period. Then for each invoice line, matches its taxes against map categories.

**Difference**: OCA can have the SAME tax classified differently in SII vs mod 303 (because they're separate map records). Enterprise uses the SAME `l10n_es_type` for both.

## 5. Special regimes (ClaveRegimenEspecialOTrascendencia)

`l10n_es/models/account_tax.py:42-55`, `AccountTax._l10n_es_get_regime_code` — `_l10n_es_get_regime_code()`:

| Code | Regime | Trigger |
|---|---|---|
| `01` | Operación normal | default |
| `02` | Exportación | `l10n_es_exempt_reason == 'E2'` (Art. 21) |
| `09` | Adquisición intracomunitaria | tag `mod_303_casilla_10/11_balance` matched |
| `17` | OSS | tag `l10n_eu_oss.tag_oss` present |

Vendor bills (`l10n_es_edi_sii/models/account_edi_format.py:268-277`, `AccountEdiFormat._l10n_es_edi_get_invoices_info`):
```
intracom (tags 10/11 in mod_303) → 09
reagyp tax present              → 02
else                            → 01
```

There are 11 more regime codes (NOT implemented in Enterprise): see comment line 44-45 referencing Gipuzkoa Anexo I.

## 6. Invoice type (TipoFactura)

`l10n_es_edi_sii/models/account_edi_format.py:279-294`, `AccountEdiFormat._l10n_es_edi_get_invoices_info`:
| Move type | Simplified? | Special | TipoFactura |
|---|---|---|---|
| `out_invoice` | yes | — | `F2` |
| `out_invoice` | no | — | `F1` |
| `out_refund` | yes | — | `R5` |
| `out_refund` | no | — | `R1` |
| `in_invoice` | — | REAGYP | `F6` |
| `in_invoice` | — | DUA | `F5` |
| `in_invoice` | — | (regular) | `F1` |
| `in_refund` | — | — | `R4` |

For refunds: `TipoRectificativa='I'` (always, "by differences" — substitution `'S'` not supported).

## 7. Validation rules (`_check_move_configuration`)

`l10n_es_edi_sii/models/account_edi_format.py:593-630`, `AccountEdiFormat._get_move_applicability`:
- Company VAT must be set
- Per line, only ONE of each: retención, recargo, sujeto, no_sujeto, no_sujeto_loc
- Per line, only ONE main tax (sujeto + no_sujeto + no_sujeto_loc total ≤ 1)
- Foreign customer: at least one tax must have `tax_scope` (consu/service)
- `in_invoice/in_refund` must have `move.ref` (vendor's invoice number)

## 8. Error handling & retry

### Response state codes
| Code | Meaning |
|---|---|
| `Correcto` | All accepted, CSV returned |
| `ParcialmenteCorrecto` | Some accepted, some errored |
| `AceptadoConErrores` | Accepted with warnings |
| `Incorrecto` | Rejected |

### Specific error codes (`l10n_es_edi_sii/models/account_edi_format.py:540-565`, `AccountEdiFormat._l10n_es_edi_call_web_service_sign_common`)
- `RegistroDuplicado` with `EstadoRegistro='Correcta'` → treated as success (we already sent it)
- Cancel + `CodigoErrorRegistro=3001` → already cancelled → success
- Error code `1117` (Datos inválidos en el sujeto) → recursive retry with `error_1117=True` context (line 556-557) — uses `IDOtro` instead of `NIF`

### Network errors
```python
except requests.exceptions.SSLError → "SSL certificate could not be validated"
except (zeep.exceptions.Error, requests.exceptions.ConnectionError) → "Networking error"
except Exception → str(error)
```
All are returned with `blocking_level='warning'` (retryable) — invoice stays in 'to_send'.

## 9. Vendor bill matching (the hard case)

`l10n_es_edi_sii/models/account_edi_format.py:512-538`, `AccountEdiFormat._l10n_es_edi_call_web_service_sign_common`: When AEAT response refers to a vendor bill by `NumSerieFacturaEmisor` (= `move.ref`), and **multiple bills** in our DB share the same ref but different partners:

1. Match by `partner_info['NIF'] == respl_partner_info.NIF`
2. Else match by `partner_info['IDOtro']` keys (CodigoPais + IDType + ID)
3. Else fall back to `candidates[0]` and post a warning to chatter

This is a known-imperfect heuristic — comment line 533: "this case shouldn't happen and means there is something wrong in this code".

## 10. CSV (Código Seguro de Verificación)

After successful submission, AEAT returns a CSV (Código Seguro de Verificación) string. Stored in `move.l10n_es_edi_csv`.

### Communication type
- `TipoComunicacion='A0'` if `csv_number is empty` (first send)
- `TipoComunicacion='A1'` if CSV exists (modification of previously sent — "Comunicacion de modificacion")

`l10n_es_edi_sii/models/account_edi_format.py:438`, `AccountEdiFormat._l10n_es_edi_call_web_service_sign_common`: All invoices in a batch share the SAME csv_number (post_batching grouping ensures this) — important because A0 + A1 cannot coexist in one envelope.

## 11. OSS / SII OSS bridge

`l10n_es_aeat_sii_oss` (OCA, NOT v19) extends OCA SII so OSS-tagged invoices use code `17` and are reported under the OSS section instead of normal sales.

Enterprise: detects OSS via tag `l10n_eu_oss.tag_oss` in `_l10n_es_get_regime_code()` automatically.

## 12. Bizkaia/Gipuzkoa specific

Bizkaia and Gipuzkoa share the SAME SII protocol structure (same SOAP messages) but DIFFERENT endpoints. Configuration: `company.l10n_es_sii_tax_agency` selection: `aeat / bizkaia / gipuzkoa`.

The function dispatch via `getattr(self, f'_l10n_es_edi_web_service_{l10n_es_sii_tax_agency}_vals')` (line 430) — convention-based.

## 13. SII match (`l10n_es_aeat_sii_match`)

OCA-only. After invoices are sent, can query AEAT for "ConsultaLR..." and compare what AEAT has vs what we have using `deepdiff<8`. Surfaces discrepancies in a separate report. **NOT PORTED to v19**.

## 14. Migration / conflict notes

- **Don't install both** Enterprise + OCA SII simultaneously — they fight over fields like `aeat_state` and the cron triggers.
- **Migration path Enterprise → OCA SII**: not officially supported. Need custom script to translate `account.edi.document` records to `aeat.mixin` state.
- **Source-of-truth conflict**: tax tags on repartition lines feed BOTH SII (Enterprise) and mod 303 (Enterprise). If the consultant sets a wrong tag, BOTH break. Symptom: SII says invoice is "intracom" but mod 303 reports it under regular VAT (or vice versa). See ESG7.
