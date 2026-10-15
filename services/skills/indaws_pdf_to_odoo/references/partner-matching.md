# Partner Matching Algorithm

3-tier algorithm to find or create the correct `res.partner` in Odoo from OCR-extracted vendor data.

## Algorithm Overview

```
OCR data (vendor_name, vendor_vat, vendor_address)
    │
    ├─ Tier 1: Exact VAT Match
    │   └─ search res.partner where vat = "ESB12345678"
    │       ├─ Found → return partner_id ✓
    │       └─ Not found → try without country prefix
    │           ├─ Found → return partner_id ✓
    │           └─ Not found → Tier 2
    │
    ├─ Tier 2: Fuzzy Name Match
    │   └─ fetch all companies → difflib.get_close_matches()
    │       ├─ Match ≥ threshold → return partner_id ✓
    │       └─ No match → Tier 3
    │
    └─ Tier 3: Auto-Create
        └─ PARTNER_AUTO_CREATE=true?
            ├─ Yes → create_record.sh res.partner → return new ID ✓
            └─ No → return None (manual review needed)
```

## Tier 1: Exact VAT Match

### VAT Normalization

Before searching, the VAT number is normalized:
1. Convert to uppercase
2. Strip spaces, dots, and hyphens

Examples:
- `es b-1234.5678` → `ESB12345678`
- `ES B12345678` → `ESB12345678`
- `B12345678` → `B12345678`

### Search Strategy

1. **Exact match**: `[["vat", "=", "ESB12345678"]]`
2. **Without country prefix**: if VAT starts with 2 letters, try `[["vat", "like", "B12345678"]]`

This handles cases where the same VAT is stored with or without the country prefix.

## Tier 2: Fuzzy Name Match

### Name Normalization

Company names are normalized before comparison:
1. Convert to lowercase
2. Remove common legal suffixes: S.L., S.A., SL, SA, SLU, S.L.U.
3. Strip whitespace

Examples:
- `Acme Corporation S.L.` → `acme corporation`
- `ACME CORPORATION, S.A.` → `acme corporation,`

### Matching

Uses Python's `difflib.get_close_matches()`:
- Fetches all company-type partners (`is_company = True`)
- Compares normalized OCR name against all normalized partner names
- Returns best match if similarity score >= `PARTNER_MATCH_THRESHOLD` (default 0.7)

### Threshold Guidelines

| Threshold | Behavior |
|---|---|
| 0.5 | Very permissive (may cause false positives) |
| 0.7 | Balanced (recommended default) |
| 0.85 | Conservative (fewer matches, more manual review) |
| 1.0 | Exact match only |

## Tier 3: Auto-Create

When `PARTNER_AUTO_CREATE=true` and no match was found, a new partner is created:

```json
{
  "name": "Vendor Name from OCR",
  "is_company": true,
  "supplier_rank": 1,
  "vat": "ESB12345678",
  "street": "Address from OCR"
}
```

The new partner is created with `supplier_rank: 1` to mark it as a vendor.

## Edge Cases

| Scenario | Handling |
|---|---|
| No VAT in OCR | Skip Tier 1, go directly to Tier 2 |
| No name in OCR | Skip Tier 2, go to Tier 3 (or return None) |
| Neither VAT nor name | Return None (cannot match) |
| Multiple VAT matches | Use first result (sorted by ID) |
| Fuzzy match returns wrong partner | Increase threshold or disable auto-create |
| Partner exists as contact, not company | Tier 2 only searches `is_company=True`; contacts under companies are covered by the company match |

## Configuration

```env
# Minimum similarity score for fuzzy matching (0.0-1.0)
PARTNER_MATCH_THRESHOLD="0.7"

# Auto-create partner if no match found
PARTNER_AUTO_CREATE="true"
```
