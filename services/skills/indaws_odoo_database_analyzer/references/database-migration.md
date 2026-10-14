# Database Migration (Odoo Upgrade)

## Overview

This section covers the database upgrade process, which is **common to all migration strategies** and **optional** (may only need module analysis).

## When to Include Database Migration

Include this section when:
- Full system migration (code + data)
- User explicitly requests database upgrade estimation
- Project scope includes production data migration

Skip this section when:
- User only wants module/code analysis
- Greenfield implementation (no legacy data)
- User explicitly excludes database migration

## Hour Calculation

**Formulas live in the `odoo-effort-estimator` skill**: database hours by volume, iterations by
project size, the version-jump multiplier, risk contingency and the full formula. Single source —
do not duplicate them here.

They are base hours: apply the AI factors after them and **before** contingency.

## Output Table Format

```markdown
| Concept | Process Scope (Iterations) | DB Size | Base Calculation | Total with Risk (H) |
|---------|---------------------------|---------|------------------|---------------------|
| **Upgrade Process** | [N] Iterations.<br>[Test / Rehearsal / Prod] | [GB] GB | [BASE]h | **[FINAL]** |
```

**Note:** In "Total with Risk (H)" column, put only the integer number.

## Common Issues and Adjustments

### Data Quality Issues
If legacy data has known quality problems:
- Duplicate records: +5h
- Inconsistent relationships: +10h
- Missing required fields: +5h

### Custom Module Data
If custom modules have complex data structures:
- Simple relations: included in base
- Complex hierarchies: +5h per module
- Binary/attachment heavy: +8h per module (from matrix)

### Integration Data
External system references that need updating:
- API keys/tokens: +2h
- External IDs mapping: +3h per system

## Checklist Before Migration

Verify with client:
- [ ] Database backup available
- [ ] Current Odoo version confirmed
- [ ] Database size confirmed
- [ ] Number of companies
- [ ] Custom modules list complete
- [ ] Integration endpoints documented
- [ ] Scheduled downtime window identified
