# AEAT Tax Models — Spain

The AEAT (Agencia Estatal de Administración Tributaria) accepts tax declarations as fixed-width BOE files (encoded in ISO-8859-1). Each "modelo" (form) has its own legal scope, periodicity and BOE format spec.

## 1. Two implementation paradigms

### Enterprise — tag-based
- Defines `account.report` records (XML data files in `l10n_es_reports/data/mod*.xml`).
- Each report line has expressions with `engine='tax_tags'` or `engine='custom'`.
- Source of truth: the **tag_ids** on `account.tax.repartition.line` (tags like `+10`, `-10` map to casillas).
- Computation: `account.report._compute_expression_totals_for_each_column_group()` aggregates AML balances by tag.
- BOE generation: handlers per report (`l10n_es.tax.report.handler` and subclasses) format strings via `_l10n_es_boe_format_string()` / `_l10n_es_boe_format_number()` — see `l10n_es_reports/models/aeat_tax_reports.py:230-287`, `SpanishTaxReportCustomHandler._convert_period_to_dates`.
- Manual data entered via per-report wizards (e.g. `l10n_es_reports.aeat.boe.mod303.export.wizard`).

### OCA — filter-based
- Each modelo is a `_inherit = "l10n.es.aeat.report.tax.mapping"` model with stored fields per casilla.
- `l10n.es.aeat.map.tax` defines the model→casilla→taxes map (XML/CSV data files).
- Each `l10n.es.aeat.map.tax.line` says: "casilla N = sum (tax T1, T2, T3) of (base|amount|both) on (regular|refund) moves".
- `_get_move_line_domain()` builds the SQL domain per line — see `l10n_es_aeat/models/l10n_es_aeat_report_tax_mapping.py:81-142`, `L10nEsAeatReportTaxMapping._get_move_line_domain`.
- BOE export uses `aeat.model.export.config` (CSV-driven) — record-driven, not code-driven.
- Built-in regularization: `create_regularization_move()` posts a journal entry zeroing tax accounts (4700/4750) → see line 216-225.

## 2. Mod 303 — IVA (VAT) periodic

**Legal**: Liquidación periódica de IVA. Trimestral (1T-4T) or mensual (REDEME, gran empresa).

### Casillas — main blocks
| Casillas | Block |
|---|---|
| [01]–[09] | Régimen general devengado (sales) |
| [10]–[11] | Adquisiciones intracomunitarias (intra-EU) |
| [12]–[13] | Otras operaciones con ISP (inversión sujeto pasivo) |
| [14]–[15] | Modificación bases y cuotas (rectificaciones) |
| [16]–[17], [18]–[19], [20]–[21] | Recargo de equivalencia |
| [22]–[26] | Modificación recargo |
| [27] | **Total cuota devengada** (sum) |
| [28]–[39] | IVA deducible (cuotas soportadas) |
| [40]–[41] | Bienes de inversión |
| [42]–[43] | Compensaciones REAGYP (agricultura) |
| [44]–[45] | Regularización inversiones |
| [46] | **Total a deducir** |
| [64], [65], [66] | Resultado régimen general / % atribuible Estado |
| [67] | Cuotas a compensar (de períodos anteriores) |
| [68] | Regularización anual (4T/12 only) |
| [69], [70], [71] | Resultado total / a deducir / a ingresar |
| [80]–[88], [93]–[99], [125]–[128] | Volumen de operaciones (info anual) |
| [108]–[112] | Rectificativa fields (added 2024-Q4 by Royal Decree 4/2024) |
| [120]–[128] | Specific operations |
| [165]–[170] | Q4-2024 new IVA rates 5% transitorio |

### Result types (`_compute_result_type`, OCA `l10n_es_aeat_mod303/models/mod303.py:474-501`, `L10nEsAeatMod303Report._compute_result_type`)
| Code | Meaning | Trigger |
|---|---|---|
| `I` | To enter | result > 0, no SEPA bank |
| `G` | To enter (CCT — cuenta corriente tributaria) | `use_aeat_account=True` |
| `U` | To enter via SEPA debit | `marca_sepa in ('1','2')` |
| `D` | To return (Spain bank) | result < 0 + last period |
| `V` | To return via CCT | result < 0 + use_aeat_account |
| `X` | To return foreign bank | result < 0 + non-Spain SEPA |
| `C` | To compensate next period | result < 0 + intermediate period |
| `N` | No activity / zero | result == 0 |

### `marca_sepa` (OCA mod303.py:299-314)
- `'1'` Spain
- `'2'` SEPA EU
- `'3'` Rest of world
- `'0'` empty (CCT)

### v18-Q4-2024 update (BOE-A-2024-12944)
Casillas 165-170 added for transitional rates (4%→2%→0% for staple foods, 7.5%→5% intermediate). See `l10n_es_reports/models/aeat_tax_reports.py:719-738`, `SpanishMod303TaxReportCustomHandler._generate_page1` — Enterprise emits these only if `date_from >= '2024-10-01'`.

### Calculation flow (OCA)
```
tax_line_ids (one per casilla) ← _prepare_tax_line_vals(map_line)
                              ← _get_tax_lines() ← domain on AML
total_devengado = SUM(casillas in {3,6,9,11,13,15,18,21,24,26,152,167,158,170,155})
total_deducir   = SUM(casillas in {29,31,33,35,37,39,41,42,43,44})
casilla_46      = total_devengado - total_deducir
atribuible      = casilla_46 * porcentaje_atribuible_estado / 100   # País Vasco/Navarra
casilla_69      = atribuible + [77] - [67] + [68] + [108]
casilla_71      = casilla_69 - previous_result + [109] - [112]      # Resultado liquidación
```

## 3. Mod 347 — Operaciones con terceros

**Legal**: Annual declaration, partners with cumulative annual operations >= **3.005,06 €** (IVA included).

### Three operation types
- **Insurance bought (B)** — `l10n_es_reports_mod347_invoice_type = 'insurance'` (vendor bills)
- **Regular bought (B)** — `move_type in (in_invoice, in_refund, in_receipt)` regular
- **Regular sold (A)** — `move_type in (out_invoice, out_refund, out_receipt)` regular
- **Cash received (C)** — cobros en metálico > 6.000 €

### Threshold engine (Enterprise)
File: `l10n_es_reports/models/aeat_tax_reports.py:71-102`, `MOD_347_CUSTOM_ENGINES_DOMAINS` defines `MOD_347_CUSTOM_ENGINES_DOMAINS` (4 domains). The `_custom_threshold_common()` method (line 942):
1. Builds AML query for the fiscal year (NOT the report period — important).
2. Excludes partners whose annual SUM <= 3.005,06 € via subquery `partners_to_exclude_query`.
3. Returns surviving partners grouped.

### Quarterly breakdown
The partner subline shows quarterly amounts: T1 / T2 / T3 / T4. BOE format requires per-quarter cells.

### Exclusions (auto)
Tags `mod_347_exclude` exclude operations:
- Intra-EU operations (already in mod 349)
- Exports (out of scope)
- Operations subject to retention/IRPF only
- Operations declared in mod 180/190 (IRPF arrendamientos)

### Real estate (`l10n_es_real_estates`)
Adds box "Operaciones de arrendamiento de locales de negocio" — separate section in BOE.

### OCA mod347 (`l10n-spain/l10n_es_aeat_mod347/models/mod347.py:814`, `L10nEsAeatMod347MoveRecord.amount`  lines`)
- Partner-level records with quarterly amounts stored as fields
- `partner.l10n_es_aeat_mod347_active = True` to include
- Partners with `vat_no_validate` skipped from threshold

## 4. Mod 349 — Intra-community operations

**Legal**: Monthly (default) or quarterly (if previous year < 50K€). Reports intra-EU operations.

### Operation keys (Enterprise `l10n_es_reports/models/aeat_tax_reports.py:104`, `MOD_347_CUSTOM_ENGINES_DOMAINS`)
```python
MOD_349_KEYS = ('A', 'E', 'T', 'S', 'I', 'M', 'H', 'R', 'D', 'C')
```
| Key | Meaning |
|---|---|
| `E` | Entregas intracomunitarias de bienes (sales of goods) |
| `A` | Adquisiciones intracomunitarias de bienes (purchases) |
| `T` | Triangulares |
| `S` | Prestaciones de servicios (services sold) |
| `I` | Adquisiciones de servicios (services bought) |
| `M` | Op. asimiladas |
| `H` | Triangulares (H — recipient) |
| `R` | Rectificaciones |
| `D` | Devoluciones |
| `C` | Suministro consignación / call-off stock |

Custom engines are dynamically dispatched via regex `MOD_349_CUSTOM_ENGINES_SPLIT_REGEX = re.compile(r'_report_custom_engine_modelo349_(invoice|refund)_([eatsimhrdc])')` (line 106).

### Periodicity logic
- Default: monthly
- Quarterly possible if total intra-EU < 50.000 € in prior year (and current year)
- One-time switch: from the moment threshold exceeded → must switch to monthly THAT month

## 5. Mod 111 — Retenciones IRPF (workers + professionals)

**Legal**: Quarterly (or monthly for gran empresa). Withholdings on:
- Salaries (Rendimientos del trabajo)
- Professional fees (Profesionales — typical 15%, or 7% for new professionals first 3 years)
- Premios

### Tax tags
Spanish PGCE 2008 chart includes taxes `s_irpf*` (sales side, customer retains us) and `p_irpf*` (purchase side, we retain provider). Always `l10n_es_type='retencion'`.

### v18 vs v19
v18 manifest: "Defines tax reports mod 111, 115 and 303"
v19 manifest: "Defines tax reports mod 111, 115, **130** and 303" — mod 130 wizard absorbed.

## 6. Mod 115 — Retenciones por arrendamientos

**Legal**: Quarterly. Withholdings on rentals of urban property used for business activity. Standard rate 19%.

Same tag structure as mod 111. OCA `l10n_es_aeat_mod115` is a thin wrapper using `l10n.es.aeat.map.tax`.

## 7. Mod 130 — Pago fraccionado IRPF (autonomous)

**Legal**: Quarterly payment-on-account by self-employed in **direct estimation** (estimación directa) regime.

### Calculation
```
Net profit YTD (revenue - deductible expenses)
  × 20%
  - retentions YTD (mod 111/115 received)
  - previous quarter mod 130 paid
= amount payable
```

### Implementation
- v18 CE: `l10n_es_modelo130` (separate addon)
- v18 EE: `l10n_es_reports_modelo130` (separate addon)
- v19: absorbed into `l10n_es` (`data/mod130.xml` in manifest)
- OCA: `l10n_es_aeat_mod130` (NOT ported to v19)

## 8. Mod 390 — Resumen anual IVA

**Legal**: Annual VAT summary. Required UNLESS exonerated by `exonerated_390='1'` (small taxpayers per Orden HAC/171/2021).

### Exoneration conditions (OCA mod303.py:168-178)
- Sujeto en régimen general
- Operations < 6 million € annual
- Submitted all four mod 303 quarters
- Field `has_operation_volume = True`

### Casillas
- [80]+[81]+[83]+[84]+[85]+[86]+[93]+[94]+[95]+[96]+[97]+[98]+[125]+[126]+[127]+[128] — added bases
- [79]+[99] — subtracted (rectifications)
- Casilla 88 = total volumen operaciones — Enterprise XML in `l10n_es_reports/data/mod390.xml`

### Custom 667 (Enterprise)
Casilla 667 added in late update — Enterprise checks for it: `l10n_es_reports/models/aeat_tax_reports.py:150`, `SpanishTaxReportCustomHandler.open_boe_wizard` raises RedirectWarning if `mod_390_casilla_667` not found → forces module update.

## 9. Mod 190 — Resumen anual retenciones (OCA only)

**Legal**: Annual summary of all IRPF withholdings (workers + professionals + premios).

### Implementation status
- **No Enterprise equivalent** — Enterprise `l10n_es_reports` does NOT generate mod 190 BOE (only mod 111).
- OCA `l10n_es_aeat_mod190` is the only path.
- **NOT PORTED to v19** → blocker for v19 migrations needing mod 190.

## 10. Mod 303 OSS extension (`l10n_es_aeat_mod303_oss`)

**Legal**: Sales B2C to other EU member states declared via OSS (One-Stop-Shop) → reported separately and EXCLUDED from regular mod 303.

The OCA module excludes OSS-tagged taxes from regular casillas and adds info casillas for declared OSS volume.

**v19 NOT PORTED** → OSS clients on v19 must rely on Enterprise reports OR stay on v18.

## 11. Mod 303 prorrata (`l10n_es_aeat_mod303_vat_prorate`)

**Legal**: When a company has BOTH taxed and exempt outputs (e.g. medical + admin), input VAT is only deductible in the % of taxed outputs. Adjusted at year-end (regularización anual prorrata).

The OCA module computes prorrata coefficient and applies it to deductible boxes [29]–[44]. Year-end regularization fills casilla [44].

**v19 NOT PORTED** → prorrata clients must build custom or stay on v18.

## 12. BOE format conventions

### String formatting (`_l10n_es_boe_format_string`)
- Encoding: ISO-8859-1
- Strings UPPERCASE, NFKC normalized, accents stripped via NFD decomposition (except Ñ and Ç)
- Padded with spaces (left-aligned by default)
- Truncated to declared length

### Number formatting (`_l10n_es_boe_format_number`)
- Always EUR — currency conversion at `date_to` if company currency != EUR
- Decimal places concatenated (no separator) — e.g. 1234.56 with length=10, decimals=2 → `0000123456`
- Sign: char `'N'` (negative) or `' '` (positive) prepended if `signed=True`
- Always padded with `0` left

### XML wrapping
Each "página" wrapped in `<TXXXNNNNN>` ... `</TXXXNNNNN>` tags (e.g. `<T30301000>` for mod 303 page 1). The actual file is mostly fixed-width BOE inside these XML wrappers — hybrid format.

## 13. Tax mapping cross-reference

| `l10n_es_type` | Used in | Effect |
|---|---|---|
| `sujeto` | mod 303 [01-09], SII Sujeta/NoExenta/S1 | Standard taxed sale |
| `sujeto_isp` | mod 303 [12-13], SII NoExenta/S2 | Inversión sujeto pasivo |
| `sujeto_agricultura` | mod 303 [42-43], SII REAGYP | REAGYP compensation |
| `exento` | mod 303 [60-63 info], SII Exenta + l10n_es_exempt_reason | Exempt sales |
| `no_sujeto` | mod 303 not subject, SII NoSujeta/Articulos7_14_Otros | Out of scope |
| `no_sujeto_loc` | SII NoSujeta/ImporteTAIReglasLocalizacion | Localization rules |
| `no_deducible` | mod 303 (excludes from [28+]) | Non-deductible input VAT |
| `retencion` | mod 111/115/130/180/190, NEVER in SII tax detail | IRPF withholding |
| `recargo` | mod 303 [16-26], SII CuotaRecargoEquivalencia | Recargo de equivalencia |
| `dua` | mod 303 [77], SII TipoFactura=F5 | Customs DUA |
| `ignore` | excluded everywhere | "Don't even count base" |
