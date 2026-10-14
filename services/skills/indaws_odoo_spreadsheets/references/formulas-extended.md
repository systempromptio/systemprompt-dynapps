# Formulas — Extended Patterns (Standard Functions + Advanced Combos)

## Standard Spreadsheet Functions (used in 33 Odoo dashboards)

| Function | Uses | Pattern |
|----------|------|---------|
| `VLOOKUP(key, range, col, exact)` | 1444 | Dynamic data lookup across sheets |
| `SUM(range)` | 18 | Subtotals in Data sheets |
| `IF(cond, true, false)` | 15 | Conditional display |
| `IFERROR(expr, fallback)` | 267 | Safe division, missing data |
| `CONCATENATE(a, b, ...)` | 26 | Build strings (dates, labels) |
| `EDATE(date, months)` | 50 | Date offset calculations |
| `AND(a, b, ...)` | 11 | Multi-condition checks |
| `ROUND(value, decimals)` | 15 | Rounding for display |
| `LEFT(text, n)` / `RIGHT(text, n)` | 12 | Extract date parts from filter values |
| `LEN(text)` | 12 | String length checks |
| `TEXT(value, format)` | 2 | Format values as text |
| `YEAR(date)` / `MONTH(date)` | 13 | Extract date components |
| `TODAY()` | 4 | Current date reference |
| `ISBLANK(cell)` | 3 | Empty cell check |
| `FILTER(range, cond)` | 8 | Dynamic filtering (Stock dashboard) |
| `UNIQUE(range)` | 4 | Remove duplicates |
| `SORT(range, col, asc)` | 1 | Sort arrays |
| `INDEX(range, row, col)` | — | Array lookup (with MATCH) |
| `MATCH(key, range, type)` | — | Position lookup (with INDEX) |

## VLOOKUP Pattern (most used — 1444 occurrences)

```
=VLOOKUP(A5, Data!A:D, 3, FALSE)
=IFERROR(VLOOKUP(K2, A:D, 3, 0), "")
```

## IF/IFERROR Wrapping Patterns

```
// Check filter before showing data
=IF(ODOO.FILTER.VALUE("Period")<>"", PIVOT.VALUE(14, "cycle_time"), "N/A")

// Double protection (filter + division)
=IF(ODOO.FILTER.VALUE("Period")<>"", IFERROR(C5/C4, "NA"), "N/A")

// Multi-condition with AND
=IFERROR(IF(AND(K2<>"", ODOO.FILTER.VALUE("Period")<>""), VLOOKUP(K2, A:D, 3, 0), ""), "")

// Division ratios with numeric fallback
=IFERROR((Q70-Q40)/(E40-Q40), 0)
=IFERROR((X40+J71+N71+R71)/X40, 1)
```

## INDEX/MATCH for Dynamic Pivot Selection (Stock dashboard)

```
=INDEX(I6:J7, MATCH(H14, H6:H7, 0), MATCH(H15, I5:J5, 0))
```

H14 = "Sort graph by" filter, H15 = "Top/Bottom" filter. Selects which pivot/measure to display.

## EDATE for Date Offset Calculations (Accounting Benchmark)

```
F1: =ODOO.FILTER.VALUE("Month")      // "03/2026"
F2: =LEFT(F1, 2)                      // "03"
F3: =RIGHT(F1, 4)                     // "2026"
F4: =CONCATENATE(F2, "/01/", F3)      // "03/01/2026"
F5: =EDATE(F4, -1)                    // 1 month ago
F9: =EDATE(F4, -4)                    // 4 months ago
```

## Running Cumulative Totals (Retention dashboard)

```
E11: =D9-SUM(D12:D36)      // Base value
E12: =E11+D12               // Each row adds to previous
E13: =E12+D13               // ...continues
```

## Cross-Sheet References

```
=Data!D4                                            // Simple (no spaces)
='By product overview fill rate - On time'!B1       // Spaces → single quotes
='WH Operations'!K:K                                // Entire column for VLOOKUP
```

## String Concatenation with Units

```
=ROUND(PIVOT.VALUE(2, "delay"), 0) & " days"
```

## Display filter value as section header

```
B24: =ODOO.FILTER.VALUE("Year")
```

Used in Accounting dashboard to show selected year in section titles.
