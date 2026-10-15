# KPI Definitions - INDAWS CRM Report

Reference document with definitions for all KPIs calculated by the CRM report system.

---

## Data sources after the One Odoo alignment

### Demos are activities, not fields

The two Studio fields that held "demo done" and "demo date" were **deleted from `crm.lead`
with no replacement**. Demo history was migrated to activities:

| | |
|---|---|
| Model | `mail.activity` |
| Domain | `[["activity_type_id","=",<Demo type id>],["res_model","=","crm.lead"]]` |
| Context | `{"active_test": False}` — **mandatory** |
| Fields | `res_id`, `user_id`, `date_done`, `date_deadline`, `active` |

- The **Demo type is resolved by name** at runtime (`mail.activity.type` where
  `name='Demo'` and `res_model='crm.lead'`). It is 38 in production, but the id differs
  between instances and must never be hardcoded.
- A demo that **happened** is a *closed* activity: `active=False`, `state='done'`. Its real
  date is **`date_done`**, not `date_deadline` (which is only when it was scheduled).

> **GOTCHA — the one that bites.** Without `active_test: False` the query returns almost
> nothing, because closing an activity archives it. If demo counts come back at zero or
> absurdly low, this is the first thing to check.

### The 7 stages, in both languages

The ids did not change (1, 26, 3, 29, 27, 4, 28), the names did:

| en_US | New | Qualified | Proposition | Analysis | Negotiation | Won | On Hold |
|---|---|---|---|---|---|---|---|
| **es_ES** | Nuevo | Cualificado | Propuesta | Análisis | Negociación | Ganado | En espera |

Stage classification is keyword-based and must work in **both** languages — the name comes
back in the language of the connected user. See `STAGE_*_KEYWORDS` in `config.py`.

**There is no "lost" stage.** A lost lead is marked on the lead itself, not moved to a
stage, so `STAGE_LOST_KEYWORDS` matches nothing today. It is kept only because
`email_service` and the diagnostics still reference it.

### utm.source: no code change, but the report looks different

The `utm.source` catalogue went from 1,090 values to 51 canonical ones. The skill reads it
as before and needs no change, but **grouping demos by source now looks different**, and
sources that used to be salespeople's names are now **empty**. Flag this in the report when
showing source breakdowns next to historical data.

---

## Executive Summary KPIs

### 1. Pipeline Status

- **Formula**: Based on run rate vs weekly target
- **Values**: ON TRACK (>= target), WARNING (>= 80% target), AT RISK (< 80% target)
- **Source**: `calculate_executive_insights()`
- **Interpretation**: Overall health of lead generation

### 2. Run Rate (4-week average)

- **Formula**: Sum of leads created in last 4 weeks / 4
- **Unit**: Leads per week
- **Target**: 10 leads/week
- **Source**: `weekly_trends` data

### 3. Momentum

- **Formula**: (current week - 4w average) / 4w average
- **Values**: ACCELERATING (> +20%), STABLE (-20% to +20%), DECELERATING (< -20%)
- **Interpretation**: Direction of lead generation trend

---

## Sales Performance KPIs

### 4. Monthly Objective Achievement

- **Formula**: (Pipeline Forecast + Actual Sales) / Monthly Target x 100
- **Unit**: Percentage
- **Thresholds**: Green >= 100%, Yellow >= 80%, Red < 80%
- **Per**: Salesperson + Global total

### 5. Pipeline Forecast (Weighted)

- **Formula**: SUM(Expected Revenue x Probability) for leads with deadline in current month
- **Unit**: Euros
- **Excludes**: Won, Lost, and Frozen stage leads
- **Per**: Salesperson

### 6. Actual Sales

- **Formula**: SUM(amount_untaxed) from confirmed sale.order records in current month
- **Unit**: Euros
- **Source**: `fetch_sales_orders()`

### 7. Gap to Target

- **Formula**: (Pipeline Forecast + Actual Sales) - Monthly Target
- **Unit**: Euros
- **Interpretation**: Positive = on track, Negative = needs attention

---

## Conversion KPIs

### 8. Demo Conversion Rate (90 days)

- **Formula**: Won deals with demo / Total deals with demo (90-day window)
- **Unit**: Percentage
- **Target**: 30% (TARGET_CONVERSION_RATE)
- **Source**: `mail.activity` — leads with at least one **completed** Demo activity whose
  `date_done` falls in the 90-day window
- **Counted per LEAD**: a lead with two demos still counts once
- **Per**: Salesperson + Global

### 9. Proposal Conversion Rate (90 days)

- **Formula**: Won proposals / (Won + Lost proposals) in 90-day window
- **Unit**: Percentage
- **Source**: `calculate_proposal_performance()`

---

## Activity KPIs

### 10. Weekly Activity Count

- **Formula**: Count of leads with `write_date` in last 7 days
- **Unit**: Number of touched leads
- **Per**: Salesperson

### 11. Average Sales Cycle

- **Formula**: AVG(date_closed - create_date) for won deals in last 90 days
- **Unit**: Days
- **Per**: Salesperson + Global

### 12. Demos Realized (Monthly)

- **Formula**: Count of **completed Demo activities** (`mail.activity`) with `date_done` in
  the current month
- **Unit**: Count
- **Target**: 16 per month (DEMOS_MONTHLY_TARGET)
- **Per**: Salesperson + by source

> **Semantics changed with the move to activities.** This now counts **demos, not leads**:
> two demos to the same client in one month count as two. And the salesperson is the
> **activity's `user_id`** — whoever actually ran the demo — not the lead's owner.
>
> The count is more accurate than the old field-based one, but **the figures are not
> comparable with months before the migration**. Say so in the report whenever this KPI
> is shown next to historical data.

---

## Pipeline Health KPIs

### 13. Active Leads Count

- **Formula**: Count of leads not Won, not Lost, and `active=True`
- **Unit**: Count
- **Excludes**: Archived leads

### 14. Concentration Risk

- **Formula**: Top deal weighted value / Total forecast x 100
- **Unit**: Percentage
- **Threshold**: > 50% triggers alert
- **Interpretation**: High value = pipeline depends too much on one deal

### 15. Opportunity Aging

- **Formula**: Days since last update (`write_date`)
- **Buckets**: 0-7d, 8-14d, 15-30d, 31-60d, 60+d
- **Per**: Stage + Salesperson (average)

### 16. Yellow Zone

- **Formula**: Count of leads with 5-6 days inactive
- **Interpretation**: Leads about to become zombies (7+ days)

---

## Alert KPIs

### 17. Zombie Count

- **Formula**: Count of leads with `days_inactive > DAYS_CRITICAL` (7 days)
- **Excludes**: Won, Lost, Frozen, Collaborators, Analysis, On Hold, Academy, archived
- **Priority**: Sorted by days inactive (worst first)

### 18. Proposal Follow-up Alerts

- **Formula**: Leads in proposal stage with `days_inactive > PROPOSAL_FOLLOWUP_DAYS` (3 days)
- **Independent**: from zombie logic (a proposal at 4 days triggers alert even though < 7 days)

### 19. At-Risk Opportunities

- **Signals**:
  - High value (> 15k) with no activity > 3 days
  - Approaching deadline (< 7 days) with probability < 50%
  - High value in early stage with > 14 days inactive
- **Sorted**: By revenue descending

---

## Data Quality KPIs

### 20. Data Quality Issues

- **Formula**: Count of active leads missing: expected_revenue, date_deadline, or probability
- **Excludes**: Early stages only — New/Nuevo, Qualified/Cualificado, 1er Contacto — plus
  Frozen and On Hold/En espera
- **Grouped by**: Salesperson, then Stage

> **Analysis is NOT an early stage.** In the pipeline it sits *after* Proposition and
> *before* Negotiation: it means delivering an analysis the client has already signed. It
> is therefore treated as an **advanced** stage and is **not** exempt from data-quality
> checks — a lead that far along with no revenue or no deadline is a real data problem,
> not an expected gap. It stays excluded from the *active pipeline* metrics (as before,
> with probability forced to 0.95), because the deal is already committed.
>
> Keyword lists live in `config.py`: `STAGE_EARLY_KEYWORDS` (early) vs
> `STAGE_ANALYSIS_KEYWORDS` (advanced). Analysis must never be added to the former.

### 21. Source Ranking

- **Formula**: Count of active leads by source (utm.source)
- **Display**: Top 8 sources
- **Excludes**: Won and Lost leads

---

## Temporal Comparison KPIs

### 22. Month-over-Month Comparison

- **Formula**: Leads created in current month vs previous month
- **Unit**: Count + delta

### 23. Quarter-over-Quarter Comparison

- **Formula**: Leads created in current quarter vs previous quarter
- **Unit**: Count + delta

### 24. Weekly Trend (4-week bars)

- **Formula**: Leads created per week for last 4 weeks
- **Display**: Bar chart with week labels (S-3, S-2, S-1, Esta)

---

## Notes

- All date-based calculations use `datetime.now()` as reference date
- Probability values from Odoo are divided by 100 (Odoo stores 0-100, we use 0-1)
- Revenue is `expected_revenue` field (untaxed)
- "Active" means both pipeline-active (not won/lost) AND Odoo `active=True` (not archived)
- Analysis stage leads have probability overridden to 0.95
