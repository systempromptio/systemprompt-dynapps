# Memory Recall: Rounding Strategy

**Source**: Session persisted at `.astound/ba/sessions/rounding-review.md` and correction record at `.astound/ba/sessions/corrections.jsonl`

**Stored Fact**: "Half-to-even (banker's rounding) is the required rounding method for checkout values, not half-up rounding."

## Rounding Test Values to Two Decimals

Using half-to-even (banker's) rounding per the stored correction:

| Value | Rounding Rule | Result |
|-------|---------------|--------|
| 1.005 | 5 rounds to nearest even (0 is even) | **1.00** |
| 1.015 | 5 rounds to nearest even (2 is even) | **1.02** |

**Derivation**: Half-to-even rounding rounds a .5 value to the nearest even digit. For 1.005, the hundredths digit rounds down to 0 (even). For 1.015, it rounds up to 2 (even).

## Session Reference
- Session file: `.astound/ba/sessions/rounding-review.md`
- Corrections file: `.astound/ba/sessions/corrections.jsonl` (row 1)
- Correction type: factual_error
- Timestamp: 2026-09-09
