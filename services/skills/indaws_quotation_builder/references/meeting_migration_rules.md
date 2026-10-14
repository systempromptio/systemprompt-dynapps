# Meeting, Training and Migration Rules

Formulas and criteria for estimating client meeting hours, per-application training, and data
migration.

> The weekly-meeting formula depends on the project week count. Under the governing model
> (`hour_estimation.md`) the weeks come from the two-track calendar; under the legacy model they
> come from `timeline_rules.md`. Use whichever matches the quotation at hand (SKILL.md
> `## Critical` 4).

---

## Meetings and Training

### General Formula

```
Total_Meetings = Kickoff_Closing + Weekly_Meetings + App_Training
```

### Kick-off and Closing

```
Kickoff_Closing = 4h (fixed)
  - Kick-off: 2h (project presentation, team, methodology, planning)
  - Closing:  2h (final validation, documentation, handover)
```

Always applies, whatever the project's size.

### Weekly Meetings

```
Weekly_Meetings = 2h × project_weeks
```

Each weekly meeting covers:
- 1.5h meeting with the client (follow-up, demos, decisions)
- 0.5h preparation and minutes

**Note**: the project's weeks come from the timeline step. For an initial estimate, use the
typical ranges:
- Simple: ~8 weeks → 16h
- Medium: ~12 weeks → 24h
- Complex: ~20 weeks → 40h

These get corrected to the exact figure once the timeline is calculated.

### Training per Application

Client training hours per application implemented:

| Tier | Hours | Applications |
|---|---|---|
| **Tier 1** | 1h | Sign, Fleet, Planning, Email Marketing, Mobile app |
| **Tier 2** | 2h | CRM, Sales, Purchase, Invoicing, Project, Helpdesk, Field Service, Quality, eLearning |
| **Tier 3** | 3h | Inventory, Accounting, HR, Point of Sale, Website, eCommerce |
| **Tier 4** | 4h | Manufacturing |

```
App_Training = sum of the tier hours of every app in the project
```

**Example** (a Medium project with CRM, Sales, Purchase, Inventory, Invoicing, Accounting):
```
Training = 2 + 2 + 2 + 3 + 2 + 3 = 14h
```

### Manual Adjustment

After applying the formula, present the result and let the consultant adjust it by **±30%**,
justifying the change.

Typical reasons to adjust:
- Client already experienced with Odoo → reduce training
- Many distinct user groups to train → increase training
- Client wants extra administrator training → increase
- Project runs fortnightly rather than weekly meetings → reduce meetings

---

## Data Migration

### Migration Levels

| Level | Criterion | Hours |
|---|---|---|
| **No migration** | New company, no prior data, starting from scratch | 0 |
| **Basic** | Contacts and/or products from Excel/CSV. Few records, simple structure | 8 |
| **Medium** | Several models (contacts, products, price lists, order history). Needs mapping and cleaning | 20 |
| **High** | A previous ERP (SAP, Navision, A3, …), several models + complex transformations + cross-validation | 40 |
| **Complex** | Full migration with accounting and/or operational history. Several years of data, reconciliation required | 60–80 |

### Indicators in the Collected Data

Look for these signals (in the source language) to set the level:

**No migration**: new company, starting from scratch, no data, startup

**Basic**: "import contacts", "we have an Excel with…", customer list, catalogue in Excel

**Medium**: several files, products with prices, recent order history, last year's data

**High**: "we're migrating from [ERP name]", previous system, SAP, Navision, A3, Sage,
"transform the data", "different structure"

**Complex**: accounting history, "X years of journal entries", full migration, all the data,
operational + accounting data, "reconcile against accounting"

### Ambiguity Rule

If the collected data **does not mention migration**, do NOT assume a level. Ask the user:

```
The collected data does not mention data migration.
Does the client have data to migrate? If so, which system does it come from and what
needs migrating?
```

### Volume Adjustment

If the collected data points to large volumes within a level, adjust upwards:

- Basic with >10,000 contacts → consider Medium (20h)
- Medium with >5 distinct models → consider High (40h)
- High with >3 years of accounting history → consider Complex (60–80h)
