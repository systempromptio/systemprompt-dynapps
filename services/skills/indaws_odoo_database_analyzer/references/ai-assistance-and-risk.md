# AI Assistance Factor & Risk Contingency

## AI Assistance Factor

**Delegate to the `odoo-effort-estimator` skill**: it is the single source of truth for AI
factors and now covers migration too (code porting, database, data migration, analysis). Do not
duplicate the table here.

Factors apply per component, after the risk factors and **before** contingency.

**Full formula for Coverage Gap Filling:**
```
gap_LOC = LOC_code × (gap_percentage / 100)
manual_hours = gap_LOC × RATE_PER_LOC
ai_hours = max(manual_hours × 0.35, MINIMUM_HOURS)
final_hours = ceil(ai_hours × (1 + risk_percentage))
```

**RATE_PER_LOC by complexity** (from CSV):

| Complexity | Rate | Minimum | Rationale |
|-----------|------|---------|-----------|
| Low (<500 LOC) | 0.05 h/LOC | 1h | Simple CRUD, ~3 min per line manually |
| Medium (500-2000 LOC) | 0.04 h/LOC | 2h | Economies of scale, moderate logic |
| High (>2000 LOC) | 0.03 h/LOC | 4h | Large modules, repetitive patterns |

---

## Risk Contingency Matrix

| Project Size | Contingency | Typical Scope |
|--------------|-------------|---------------|
| Small | +10% | <10 modules, small team |
| Medium | +18% | 10-30 modules, medium team |
| Large | +25% | >30 modules, large team |

**Formula:**
```
Final_Hours = Ceiling(Base_Hours * (1 + Risk_Percentage))
```
