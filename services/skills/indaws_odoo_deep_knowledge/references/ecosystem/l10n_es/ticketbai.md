# TicketBAI — País Vasco (Basque Country)

**Legal scope**: Real-time invoice submission to the Basque provincial governments (Diputaciones Forales). Mandatory for all businesses in:
- Araba/Álava (Diputación Foral de Álava)
- Bizkaia (Diputación Foral de Bizkaia, system: **Batuz** since 2024)
- Gipuzkoa (Diputación Foral de Gipuzkoa)

**Periodicity**: Real-time (each invoice sent immediately after posting).

**Implementation**: ONLY Enterprise (`l10n_es_edi_tbai`) — there is **NO OCA TicketBAI module** (see `ESG29`). Module sits in `odoo18/addons/` despite being Spanish-only.

## 1. Module structure

Source: `odoo18/addons/l10n_es_edi_tbai/`

```
models/
├── account_move.py                    (390 lines) — invoice integration + send/cancel
├── account_move_line.py               (10 lines)
├── l10n_es_edi_tbai_document.py       (872 lines) — XML generation, signing, web service
├── l10n_es_edi_tbai_agencies.py       (76 lines) — agency endpoint URLs
├── certificate.py                     (27 lines)
├── res_company.py                     (167 lines) — chain sequence, company config
├── xml_utils.py                       (88 lines) — canonicalization, references
└── account_move_send.py               (65 lines)
```

## 2. Three regional differences

### 2.a Endpoints — `l10n_es_edi_tbai_agencies.py`

#### Araba (Álava)
- post_url_test: `https://pruebas-ticketbai.araba.eus/TicketBAI/v1/facturas/`
- post_url_prod: `https://ticketbai.araba.eus/TicketBAI/v1/facturas/`
- xsd: `ticketBaiV1-2.xsd`
- sigpolicy_url: `https://ticketbai.araba.eus/tbai/sinadura/`
- Single zip with all XSDs

#### Bizkaia (Batuz LROE)
- post_url_test: `https://pruesarrerak.bizkaia.eus/N3B4000M/aurkezpena`
- post_url_prod: `https://sarrerak.bizkaia.eus/N3B4000M/aurkezpena`
- xsd: `ticketBaiV1-2-1.xsd` + `Anula_ticketBaiV1-2-1.xsd` (separate)
- sigpolicy: PDF document `https://www.batuz.eus/.../sinadura_elektronikoaren_zehaztapenak...`
- **WRAPS TicketBAI XML inside an LROE envelope** — different protocol from the other two
- Supports vendor bills (purchase) — only Bizkaia does this

#### Gipuzkoa
- post_url_test: `https://tbai-prep.egoitza.gipuzkoa.eus/WAS/HACI/HTBRecepcionFacturasWEB/rest/recepcionFacturas/alta`
- post_url_prod: `https://tbai-z.egoitza.gipuzkoa.eus/sarrerak/alta`
- xsd: `ticketBaiV1-2-1.xsd` (in zip)
- sigpolicy_url: `https://www.gipuzkoa.eus/TicketBAI/signature`

### 2.b Submission semantics

- **Araba/Gipuzkoa**: send raw signed XML via HTTP POST. Endpoint returns `<Estado>0</Estado>` on success, with `<ResultadosValidacion><Codigo>NNN</Codigo>...` for warnings.
- **Bizkaia**: gzipped LROE envelope (mode 240 corp / 140 freelancer) → POST with extra headers `eus-bizkaia-n3-version`, `eus-bizkaia-n3-content-type`, `eus-bizkaia-n3-data` (JSON metadata). Response in `eus-bizkaia-n3-tipo-respuesta` header.

### 2.c Bizkaia Batuz freelancer (autónomo)

`l10n_es_edi_tbai/models/account_move.py:108`, `AccountMove._compute_l10n_es_tbai_is_required`: For Bizkaia, vendor bills (purchases) ALSO go to TicketBAI — `move.is_purchase_document() and move.company_id.l10n_es_tbai_tax_agency == 'bizkaia'`.

For autónomos in Bizkaia, an additional config parameter is required:
```
l10n_es_edi_tbai.epigrafe = (epigrafe code from xlsx Anexo Bizkaia)
```
Otherwise `_check_can_post()` returns an error (line 115-120).

## 3. Hash chain (cadena de huella)

Each TicketBAI document references the PREVIOUS document's signature. This forms a tamper-evident chain.

### chain_index
`l10n_es_edi_tbai.document.chain_index` (Integer, no_gap sequence per company).
- Created by `company._get_l10n_es_tbai_next_chain_index()` — uses `ir.sequence`.
- Assigned ONLY when document is about to be sent (post-validate, pre-send).
- If sending fails, `chain_index = 0` and the spot is "wasted" (line 194).

### Chain integrity check
`l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:127-129`, `L10nEsEdiTbaiDocument._check_can_post`:
```python
chain_head_doc = self.company_id._get_l10n_es_tbai_last_chained_document()
if chain_head_doc and chain_head_doc != self and chain_head_doc.state != 'accepted':
    return _("TicketBAI: Cannot post invoice while chain head (%s) has not been posted", chain_head_doc.name)
```
**Cannot post invoice N+1 if invoice N is not 'accepted'**. If invoice N is rejected and you want to skip it, you must mark it 'accepted' manually OR cancel the chain entirely.

### Reset to draft blocked
`l10n_es_edi_tbai/models/account_move.py:122-130`, `AccountMove.button_draft`:
```python
if move.l10n_es_tbai_chain_index and move.l10n_es_tbai_state != 'cancelled':
    raise UserError("You cannot reset to draft an entry that has been posted to TicketBAI's chain")
```

### Unlink blocked
`l10n_es_edi_tbai/models/account_move.py:132-136`, `AccountMove._l10n_es_tbai_unlink_except_in_chain`: invoices with chain_index cannot be deleted (unless `force_delete` context is set — escape hatch for sysadmins).

## 4. XML signing (XAdES-EPES)

Source: `xml_utils.py` (88 lines) + `l10n_es_edi_tbai_document.py:_generate_xml`.

- Signature standard: XAdES-EPES (with explicit signature policy)
- Algorithm: SHA-256 / RSA-SHA256
- Canonicalization: C14N exclusive (`http://www.w3.org/2001/10/xml-exc-c14n#`)
- Reference digests: 2 references (signed properties + the document)
- Embeds the **signature policy** identifier (sigpolicy_url + sigpolicy_digest) per agency

Custom helpers:
- `calculate_references_digests()` — calculates `<ds:DigestValue>` for each `<ds:Reference>`
- `canonicalize_node()` — C14N normalization
- `cleanup_xml_signature()` — removes whitespace artifacts

## 5. Refund (rectificativa) — Refund Reasons

`l10n_es/models/account_move.py:12-18`, `class AccountMove`:

| Code | Description |
|---|---|
| `R1` | Art. 80.1, 80.2, 80.6 + rights-founded error |
| `R2` | Art. 80.3 (concurso) |
| `R3` | Art. 80.4 (créditos incobrables) |
| `R4` | Art. 80 — other |
| `R5` | Factura rectificativa simplificada |

Validation (`l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:152-157`, `L10nEsEdiTbaiDocument._check_can_post`):
- Refund reason **MUST be set** for credit notes
- Simplified invoices → reason MUST be `R5`
- Non-simplified → reason MUST NOT be `R5`

`l10n_es_tbai_reversed_ids` (M2M to original bills) — required for vendor refunds with multiple sources.

## 6. Cancellation flow

Different from refund: a "cancellation" tells the agency the invoice should be removed from the chain.

- `_l10n_es_tbai_cancel_document_id` is a separate `l10n_es_edi_tbai.document` record with `is_cancel=True`.
- Original invoice's `state='posted'` does NOT change in Odoo; only TBAI status moves to `'cancelled'`.
- Cancellation XML uses `Anula_ticketBai...xsd` schema (different from emission).
- For Bizkaia: same LROE wrapper but `apa='2.1'` (freelancer) or `apa='2'` (corp).

## 7. QR code

Each TBAI invoice gets a QR code embedded in printed/emailed PDFs:
```
{qr_url_agency}?id={tbai_id}&s={serial}&nf={invoice_number}&i={amount}
```

Where:
- `tbai_id` is the TBAI identifier built from NIF + date + chain_index
- The QR includes a CRC8 checksum at the end (table at `l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:27-44`, `CRC8_TABLE`)

URL pattern verified at `_l10n_es_tbai_get_qr_url()` (different per agency).

## 8. Tax mapping

TicketBAI reuses the same `l10n_es_type` selection as SII. Differences:
- TBAI separates S1 (sujeto) and S2 (sujeto_isp) into different XML sections (where SII merges them with TipoNoExenta).
- See `l10n_es_edi_sii/models/account_edi_format.py:198-199`, `AccountEdiFormat._l10n_es_edi_get_invoices_tax_details_info`: helper returns `'S1_list'` and `'S2_list'` separately for TBAI consumption.
- TBAI does NOT support DUA (`F5`). Use SII for DUA.

## 9. Foreign customer + tax_scope check

`l10n_es_edi_tbai/models/l10n_es_edi_tbai_document.py:131-136`, `L10nEsEdiTbaiDocument._check_can_post`:
```python
if values['partner']._l10n_es_is_foreign() and values['taxes'].filtered(lambda t: not t.tax_scope):
    return _("In case of a foreign customer, you need to configure the tax scope on taxes...")
```
Foreign customer → all taxes MUST have `tax_scope` (`consu`/`service`). Required to fill the right XML section (Entrega vs PrestacionServicios).

## 10. Locking on submission

`l10n_es_edi_tbai/models/account_move.py:177-185`, `AccountMove._l10n_es_tbai_lock_move`:
```python
self.env.cr.execute('SELECT * FROM account_move WHERE id = %s FOR UPDATE NOWAIT', [self.id])
```
On TBAI send, an exclusive row lock is taken. If another process is already sending the same invoice, raises "Cannot send this entry as it is already being processed."

This prevents concurrent sends but does NOT prevent gap in chain if the Odoo worker dies between chain_index assignment and send completion.

## 11. Test environment

- `company.l10n_es_tbai_test_env` (Boolean)
- All endpoints have separate `_test` and `_prod` URLs
- AEAT does NOT validate the test certificate against real KO-FE (Bizkaia Batuz pre-prod)

## 12. Common errors

| Error code | Agency | Meaning |
|---|---|---|
| `005` / `019` | Araba/Gipuzkoa | XML already received with this sequence (treated as success — `_process_post_response_xml_ar_gi:254`) |
| `B4_2000003` | Bizkaia | Already received (success — line 330) |
| Various 4xxx | Araba | Bad XML / validation |
| Schema validation errors | All | Usually means missing/wrong field per region's XSD |

## 13. Performance notes

- Each invoice is sent one-by-one (no batching for Araba/Gipuzkoa).
- Bizkaia LROE supports multiple TBAI XMLs inside one envelope but Odoo sends one per invoice (line 310: single-element list `[base64.b64encode(self.xml_attachment_id.raw).decode()]`).
- For high-volume customers, this is a constraint — chain serialization makes parallelization impossible (must wait for previous to be 'accepted').
