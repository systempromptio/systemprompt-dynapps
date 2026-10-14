# Tax Rate Mapping

How OCR-detected tax rates are mapped to Odoo `account.tax` records.

## Spanish Tax Rates (IVA)

| Rate | Description | Common Uses |
|---|---|---|
| 21% | General (IVA general) | Most goods and services |
| 10% | Reduced (IVA reducido) | Food, transport, hospitality |
| 4% | Super-reduced (IVA superreducido) | Bread, milk, books, medicine |
| 0% | Exempt (Exento) | Medical, education, financial services |

## Search Pattern

The skill searches for tax records using:

```
search_records.sh account.tax '[["amount","=",21],["type_tax_use","=","purchase"]]'
```

### Fields Used

| Field | Description |
|---|---|
| `amount` | Tax percentage (21, 10, 4, 0) |
| `type_tax_use` | `purchase` for vendor invoices, `sale` for customer invoices |

### Tax Use Mapping

| Document Type | `type_tax_use` |
|---|---|
| `in_invoice` (vendor invoice) | `purchase` |
| `in_refund` (vendor credit note) | `purchase` |
| `out_invoice` (customer invoice) | `sale` |
| `out_refund` (customer credit note) | `sale` |

## Odoo Tax Assignment

Taxes are assigned to invoice lines using the Many2many command format:

```json
{
  "tax_ids": [[6, 0, [tax_id]]]
}
```

Where `[6, 0, [ids]]` replaces all existing tax links with the given IDs.

### Multiple Taxes

If a line has multiple taxes (e.g., IVA + IRPF retention), they would be:

```json
{
  "tax_ids": [[6, 0, [iva_tax_id, irpf_tax_id]]]
}
```

## Fallback Behavior

| Scenario | Handling |
|---|---|
| Tax rate found in OCR, matching Odoo tax exists | Assign the tax to the line |
| Tax rate found in OCR, no matching Odoo tax | Skip tax assignment (line created without tax) |
| No tax rate in OCR | Skip tax assignment |
| Tax rate is 0% | Search for exempt tax; skip if not found |
| Non-standard rate (e.g., 15%) | Search as-is; likely no match, line created without tax |

## Common Odoo Tax Names (Spain)

These are typical names in a Spanish Odoo database. The actual names may vary:

| Rate | Typical Name (Purchase) | Typical Name (Sale) |
|---|---|---|
| 21% | IVA 21% (Bienes) | IVA 21% |
| 10% | IVA 10% (Bienes) | IVA 10% |
| 4% | IVA 4% (Bienes) | IVA 4% |
| 0% | Exento de IVA | Exento de IVA |

## IRPF Retention (Spain-specific)

Some invoices from freelancers include IRPF retention (negative tax). Common rates:
- -15% (standard retention)
- -7% (new freelancers, first 3 years)

The OCR may detect these as negative tax amounts. The current implementation does not handle IRPF automatically -- it would require a second tax search with negative amount.

## Configuration

Tax lookup is automatic and requires no configuration. The skill searches based on the percentage detected by OCR. If the Odoo database uses non-standard tax configurations, the tax may not be found and the line will be created without tax assignment.
