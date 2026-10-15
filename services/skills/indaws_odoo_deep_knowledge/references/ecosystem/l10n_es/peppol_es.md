# l10n_es — Peppol, Crea y Crece, VeriFactu/SII/TBAI/Facturae

How Spain's e-invoicing patchwork maps onto Odoo. Six concurrent regulatory regimes coexist: **FACE** (B2G), **SII** (real-time AEAT), **VeriFactu** (anti-fraud), **TicketBAI** (Basque Country), **Crea y Crece** (B2B mandatory, pending RD), **Peppol** (B2B channel once Crea y Crece kicks in).

This file is the navigation hub. Engine details live in [[../../functional/edi/peppol]], [[../../functional/edi/ubl_cii]], [[../../enterprise/intrastat/core]], and the existing [[overview]], [[facturae]], [[sii]], [[ticketbai]], [[aeat_models]].

---

## 1. Regulatory landscape (May 2026 snapshot)

| Regime | Scope | Format | Channel | Odoo module | Status |
|--------|-------|--------|---------|-------------|--------|
| **FACE** | B2G (public admin) | Facturae 3.2.2 | FACE web service | `l10n_es_edi_facturae` (CE) | Mandatory since 2015 |
| **FACeB2B** | B2B optional | Facturae 3.2.2 | Same as FACE | Same module | Optional |
| **SII** | Companies > €6M | JSON-SII XML | AEAT SOAP | `l10n_es_edi_sii` (CE) | Mandatory > €6M, 4-day window |
| **VeriFactu** | Anti-fraud, hash + QR | JSON + AEAT submission | AEAT SOAP WSDL | `l10n_es_edi_verifactu` (CE) | 2025: > €600M; 2026: SMEs |
| **TicketBAI** | Basque Country (PV) | XML + QR + chain | Diputaciones Forales SOAP | `l10n_es_edi_tbai` (CE) | Mandatory PV since 2020 |
| **Crea y Crece** | B2B Spain | UBL BIS3 via Peppol | Peppol AccessPoint | `account_peppol` + `account_edi_ubl_cii` | RD pending; expected 2026/2027 |
| **Intrastat** | Intra-EU trade stats | XML/CSV (AEAT specific) | AEAT INTRASTAT-WEB | `l10n_es_intrastat` (Enterprise v19) | Monthly if > €400 000 / year |
| **Modelo 349** | Intra-EU recap VAT | XML | AEAT | `l10n_es_reports` (Enterprise) | Quarterly/annual, B2B EU |

**Key fact:** these are **not alternatives**. A typical mid-sized Spanish company invoicing both public admin, mainland B2B, Basque customers and EU partners may need to operate **all of them simultaneously**.

---

## 2. Peppol on Spain

### EAS code
- **`9920`** = Spain VAT (NIF). Validation regex: `r'[^0-9A-Z]'` (digits + uppercase letters).
- Standard format: 8 digits + 1 letter (e.g. `B12345678` for sociedades, `12345678X` for autónomos).
- Endpoint format: `9920:ES12345678X`.

Spain is in `PEPPOL_DEFAULT_COUNTRIES` (`odoo19/addons/account/models/company.py`). `account_peppol` auto-installs when Spain is the company country and `account_edi_ubl_cii` is present.

### SMP authority
Spain joined Peppol in 2024. Authorised access-points (FNMT, Camerfirma, etc.) handle SMP registration. The Odoo IAP layer (`peppol.api.odoo.com`) wraps registration; you do not interact with the AccessPoint directly.

### Crea y Crece timeline (Ley 18/2022, Art. 12)
- **2026-07-01**: companies with turnover > €8 M (estimated, pending Real Decreto).
- **2027-07-01**: all companies (estimated).
- Format expected: UBL BIS3 via Peppol.
- Public admin (B2G) remains Facturae 3.2.2 via FACE — **Peppol does not replace Facturae for B2G**.

### Spanish-flavour gotchas in `account_edi_ubl_cii`
- **IGIC (Canary Islands)** → `ubl_cii_tax_category_code='L'`. Auto-detected when partner's postal code starts with 35 or 38.
- **IGIC (Ceuta/Melilla)** → `'M'`. Postal codes 51-52.
- **Intra-community zero-tax** → `'K'` (EEA+EEA, zero tax).
- **Reverse charge** → `'AE'` (auto-detected via `tax.has_negative_factor`).
- v19 lets you override the auto-detection on each `account.tax` via `ubl_cii_tax_category_code` and `ubl_cii_tax_exemption_reason_code` — **strongly recommended** for Spanish taxes.

---

## 3. VeriFactu vs SII (the coexistence trap)

| Feature | SII | VeriFactu |
|---------|-----|-----------|
| Threshold | > €6 M turnover | All (gradual 2025-2026) |
| Submission timing | Real-time, 4 business days | Batch + polling |
| Format | JSON-SII XML | JSON + hash chain |
| Required attestation | AEAT-recognised certificate | Same |
| QR code | No | Yes |
| Hash chain | No | **Yes — sequential** |
| Cancellation | Aceptado/rechazado/anulado states | New submission with rejection reason |

**Trap:** the two systems submit the same invoice data but with different timing and different schemas. They are **not redundant** — both are mandatory if your turnover crosses the SII threshold. You cannot use VeriFactu to replace SII.

**`l10n_es_edi_verifactu` model:**
- `l10n_es_edi_verifactu.document` with `chain_index` (hash chain order) and `state` (rejected / registered_with_errors / accepted).
- `account.move.l10n_es_edi_verifactu_state`, `l10n_es_edi_verifactu_qr_code`.
- 5 refund reasons R1-R5 mapped to Art 80 VAT Law (`l10n_es_edi_facturae/models/account_move.py:69-79`, `SPANISH_CREDIT_REASON_TYPE`).

**Hash chain lock-in:** once a document enters the chain (`chain_index` set), it cannot be deleted. Corrections require new submissions; cancellations create separate documents referencing the original.

---

## 4. TicketBAI (Basque Country)

- Mandatory since 2020 in Araba, Bizkaia, Gipuzkoa.
- XML format with QR and hash chain (similar to VeriFactu, predates it).
- Submission to three regional Diputaciones (not AEAT directly).
- Module: `l10n_es_edi_tbai` (CE).
- Status field: `account.move.l10n_es_edi_tbai_status`.

**Multi-region scenario (company with PV + national activity):**
- National invoices: VeriFactu + (SII if > €6 M) + (Crea y Crece Peppol once in force).
- PV invoices: TicketBAI (instead of VeriFactu).
- PV customer's address must carry the exact Diputación code at issue time.
- POS variants exist: `l10n_es_edi_tbai_pos` aggregates POS sales by session.

---

## 5. FACE (B2G)

- Facturae 3.2.2 XML — **completely different schema from UBL/Peppol BIS3**.
- Submission via FACE web service (SOAP).
- Required for invoices to Spanish public administrations.
- Module `l10n_es_edi_facturae` (CE) auto-installs with `l10n_es`.
- DIR3 administrative-centre codes managed by `l10n_es_edi_facturae_adm_centers`.
- Invoice period extension via `l10n_es_edi_facturae_invoice_period`.

**Critical:** do NOT route public-admin invoices through Peppol. Partner type must drive format selection (`invoice_edi_format='facturae'` for FACE partners, `'ubl_bis3'` for everyone else).

---

## 6. Intrastat for Spain

See [[../../enterprise/intrastat/core]] for the engine. Spanish specifics:

- **Threshold**: €400 000 annual on sales OR purchases (`l10n_es_intrastat` v19 only).
- **Forms**: Modelo N-I (introducciones / arrivals), Modelo N-E (expediciones / dispatches).
- **Submission**: AEAT INTRASTAT-WEB (XML or CSV).
- **Deadline**: monthly, +12 days after month-end.
- **Mandatory fields**: NIF, código provincia.
- v19 `l10n_es_intrastat` auto-creates two `account.return.type` records and links them to MOD 349 (`casilla_10`, `casilla_59`).

**Modelo 349 vs Intrastat:** they are **separate declarations**, both required when applicable.
- Modelo 349 = recapitulative VAT statement (quarterly or annual).
- Intrastat = goods/services statistics (monthly).
- A single intra-EU sale appears in both, with different field sets.

---

## 7. Compliance roadmap per company profile

### A — Spanish SME, no Basque exposure, no public admin (May 2026)
- Install: `l10n_es`, `l10n_es_edi_verifactu`.
- 2025: VeriFactu mandatory once threshold reaches you (RD 1007/2023 timeline).
- 2026/2027: enable Peppol (`account_peppol` + `account_edi_ubl_cii`) once Crea y Crece kicks in.
- Optional: `l10n_es_edi_sii` if your turnover crosses €6 M.

### B — Same as A + sells to public administrations
- Add: `l10n_es_edi_facturae` for FACE.
- Configure partner `invoice_edi_format='facturae'` for each public-admin customer.
- Stagger format selection — never Peppol-route a B2G invoice.

### C — Basque Country resident (PV office)
- Add: `l10n_es_edi_tbai` (instead of `l10n_es_edi_verifactu`).
- Optional: `l10n_es_edi_tbai_pos` for POS.
- PV-resident invoices follow TicketBAI; cross-region invoices follow VeriFactu.

### D — Large company > €6 M turnover
- All of the above + `l10n_es_edi_sii`.
- Plan for AEAT certificate renewal (1-2 weeks lead time before expiry).

### E — Cross-border B2B EU
- Add Intrastat (`l10n_es_intrastat`) if > €400 K threshold.
- Modelo 349 already in `l10n_es_reports` (Enterprise).
- Peppol UBL BIS3 once Crea y Crece is in force (for ES↔EU B2B).

---

## 8. OCA `l10n-spain` alternatives

The OCA Community modules pre-date the Odoo native ones and remain in active use, especially in CE-only setups.

| OCA module | Native equivalent | Notes |
|------------|-------------------|-------|
| `l10n_es_aeat` | (base for) | Generic AEAT framework. |
| `l10n_es_aeat_mod303` | `l10n_es_reports` (mod303) | IVA quarterly. |
| `l10n_es_aeat_mod349` | `l10n_es_reports` (mod349) | Intra-EU recap. |
| `l10n_es_aeat_mod347` | none in native | Customer/supplier yearly. |
| `l10n_es_aeat_sii_oca` | `l10n_es_edi_sii` | Different approach (XML files vs SOAP). |
| `l10n_es_verifactu_oca` | `l10n_es_edi_verifactu` | OCA flavour. |
| `l10n_es_facturae` | `l10n_es_edi_facturae` | Pre-dates native. |
| `l10n_es_vat_book` | none in native | Libro registro IVA. |

**OCA v19 status**: per memory snapshot, ~73-80 % of `l10n-spain` modules are NOT YET PORTED to v19. **Blocking issue** for CE customers planning v18→v19 upgrade. Native Enterprise modules are the safer bet for v19.

### Indaws adapters
- `indaws_l10n_es_edi_sii`: extends native SII.
- `indaws_l10n_es_edi_verifactu`: extends native VeriFactu.

---

## 9. Gotchas summary (Spain-specific)

- **Peppol ≠ FACE**: separate channels, separate formats. Don't route B2G through Peppol.
- **VeriFactu hash chain is permanent**: once a document is in the chain, no deletion possible.
- **TicketBAI region code must match partner province exactly** — Diputación SOAP rejects mismatches.
- **NIF format on Peppol**: accept alphanumeric (digits + uppercase letter), no separators.
- **IGIC = `L` or `M`**: Odoo auto-detects by postal code (35/38 = Canarias = L; 51/52 = Ceuta/Melilla = M). Set explicitly in v19 via tax extension fields.
- **Modelo 349 vs Intrastat**: separate, both required. Different scopes and field sets.
- **Spanish Peppol endpoint**: NIF without country prefix on the endpoint (`B12345678`, not `ESB12345678`).
- **Certificate renewal**: SII, VeriFactu, TBAI all use PKCS#12 cert auth. Stale cert → silent submission failures (cron errors logged but no UI alert).

See [[../../functional/edi/gotchas]] for the broader EDI gotcha list.

---

## 10. Cross-references

- Peppol channel + IAP → [[../../functional/edi/peppol]].
- UBL/CII generation + import → [[../../functional/edi/ubl_cii]].
- Bridge modules → [[../../functional/edi/bridges]].
- v18 → v19 EDI changes → [[../../functional/edi/v19_changes]].
- Intrastat core (engine) → [[../../enterprise/intrastat/core]].
- l10n_es overview → [[overview]].
- AEAT models → [[aeat_models]].
- Facturae details → [[facturae]].
- SII details → [[sii]].
- TicketBAI details → [[ticketbai]].
- l10n_es v19 diff (existing) → [[v19_diff]].
- Spain-wide gotchas → [[gotchas]].
