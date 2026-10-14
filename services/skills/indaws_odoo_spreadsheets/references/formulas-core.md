# Odoo Spreadsheet Formulas — Core (v18/v19)

## All 16 Odoo Formulas

### Pivot Formulas

| Formula | Description |
|---------|-------------|
| `PIVOT(pivot_id, [row_count], [include_total], [include_headers])` | Auto-expand full pivot table |
| `PIVOT.VALUE(pivot_id, measure, ...domain_pairs)` | Single aggregated value |
| `PIVOT.HEADER(pivot_id, ...domain_pairs)` | Header label for a dimension value |

### List Formulas

| Formula | Description |
|---------|-------------|
| `ODOO.LIST(list_id, index, field_name)` | Value from list at row index |
| `ODOO.LIST.HEADER(list_id, field_name)` | Column header label |

### Accounting Formulas

| Formula | Description |
|---------|-------------|
| `ODOO.BALANCE(codes, date_range, [offset], [company_id], [unposted])` | Account balance |
| `ODOO.CREDIT(codes, date_range, [offset], [company_id], [unposted])` | Credit total |
| `ODOO.DEBIT(codes, date_range, [offset], [company_id], [unposted])` | Debit total |
| `ODOO.RESIDUAL(codes, date_range, [offset], [company_id], [unposted])` | Residual amount (v18+) |
| `ODOO.PARTNER.BALANCE(partner_ids, [codes], [date_range], [offset], [company_id], [unposted])` | Balance per partner (v18+) |
| `ODOO.ACCOUNT.GROUP(account_type)` | Account codes by type |

### Utility Formulas

| Formula | Description |
|---------|-------------|
| `ODOO.CURRENCY.RATE(from, to, [date], [company_id])` | Currency exchange rate |
| `ODOO.FISCALYEAR.START(date, [company_id])` | Fiscal year start date |
| `ODOO.FISCALYEAR.END(date, [company_id])` | Fiscal year end date |
| `ODOO.FILTER.VALUE(filter_name)` | Current value of a global filter |
| `FORMAT.LARGE.NUMBER(value)` | Humanize numbers (1.2k, 3.5M) |
| `_t(value)` | Translation for internationalization |

## Detailed Usage

### PIVOT()

```
=PIVOT(1)                    // Full pivot, all rows, with totals and headers
=PIVOT(1, 10)                // Limit to 10 data rows
=PIVOT(1, 10, TRUE)          // 10 rows with totals
=PIVOT(1, 10, TRUE, TRUE)    // 10 rows with totals and headers
```

- Auto-expands: one cell generates the entire pivot table
- Uses `formulaId`, NOT the UUID key

### PIVOT.VALUE()

```
=PIVOT.VALUE(1, "amount:sum")
=PIVOT.VALUE(1, "amount:sum", "user_id", 9)
=PIVOT.VALUE(1, "amount:sum", "user_id", 9, "date:month", "01/2026")
```

- Returns a single number. Dimension pairs: `"field_name", value` alternating.

### PIVOT.HEADER() with positional index (`#field`)

For Top N ranking tables, use `#` prefix with positional index:

```
=PIVOT.HEADER(3, "#user_id", 1)    // Name of 1st user in ranking
=PIVOT.VALUE(3, "__count", "#user_id", 1)  // Count for 1st user
```

The `#` prefix means "position N in the group", not a specific record ID.

### ODOO.LIST()

```
=ODOO.LIST(1, 1, "name")           // First record, field "name"
=ODOO.LIST(1, 5, "amount_total")   // Fifth record, amount
```

- `list_id`: matches `formulaId`. `index`: 1-based row number.

### ODOO.FILTER.VALUE()

```
=ODOO.FILTER.VALUE("Año")           // Current year filter value
```

- References filter by `label` (display name), not by ID
- Returns `""` when no filter value selected

**v19 BREAKING**: Returns structured data in v19 (not plain text). During v18→v19 migration, calls are renamed to `ODOO.FILTER.VALUE.V18()` which preserves text behavior.

### FORMAT.LARGE.NUMBER()

```
=FORMAT.LARGE.NUMBER(1234567)     // → "1.2M"
=FORMAT.LARGE.NUMBER(56300)       // → "56.3k"
```

### ODOO.BALANCE + ODOO.ACCOUNT.GROUP (Combined Pattern)

```
=ODOO.BALANCE(ODOO.ACCOUNT.GROUP("income"), B$30)
=-ODOO.BALANCE(ODOO.ACCOUNT.GROUP("expense"), B$30)
```

Account type constants: `asset_receivable`, `asset_cash`, `asset_current`, `asset_prepayments`, `asset_non_current`, `asset_fixed`, `liability_payable`, `liability_credit_card`, `liability_current`, `liability_non_current`, `income`, `income_other`, `expense`, `expense_depreciation`, `expense_direct_cost`, `off_balance`

### _t() Translation Function

```
=_t("Opportunity")     // → "Oportunidad" (in es_ES)
```

Only works in cell content (not figure titles or filter labels).

## Locale Rule

**CRITICAL**: Formulas in JSON always use **commas** as argument separators, regardless of locale. The `formulaArgSeparator` only affects UI display.

```
// CORRECT (in JSON, always commas)
=PIVOT.VALUE(1, "amount:sum", "user_id", 9)

// WRONG (semicolons cause #BAD_EXPR)
=PIVOT.VALUE(1; "amount:sum"; "user_id"; 9)
```

## Essential Combos

### Conditional display based on filter

```
=IF(ODOO.FILTER.VALUE("Order date")<>"", PIVOT.VALUE(2, "__count"), "N/A")
```

### Safe division

```
=IFERROR(B3/B2)
=IFERROR(B5/B4, "NA")
```

### Data Sheet KPI pattern

```
B2: =PIVOT.VALUE(1,"__count")              // Raw: 1234
D2: =FORMAT.LARGE.NUMBER(B2)              // Formatted: "1.2k"
Scorecard keyValue: "Data!D2"             // Shows "1.2k"
```
