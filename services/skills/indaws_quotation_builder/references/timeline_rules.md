# Timeline Calculation Rules (LEGACY MODEL)

Algorithm for calculating project timescales using three pace profiles, plus the formulas for
distributing time.

> **Legacy model.** These 40/50/60h-per-week pace profiles contradict the governing model in
> `hour_estimation.md` (functional 12h/week ∥ development 25h/week). Use this file only when
> editing a quotation already built on the legacy model, and never mix the two within one
> quotation (SKILL.md `## Critical` 4). `meeting_migration_rules.md` depends on the week counts
> produced here.

---

## Pace Profiles

| Profile | Weekly capacity | Description | Typical use |
|---|---|---|---|
| **Accelerated** | 60h/week | Dedicated team, full availability | Urgent project, external deadline |
| **Normal** | 50h/week | Standard, balanced pace | Most projects |
| **Extended** | 40h/week | More slack, less pressure | Shared team, client with little availability |

Weekly capacity covers the hours of the whole assigned team (consultants + developers).

---

## Calculation Algorithm

### Step 1: Analysis weeks

Fixed duration by project classification:

| Classification | Analysis hours | Analysis weeks |
|---|---|---|
| Simple (25h) | 25 | 1 week |
| Medium (40h) | 40 | 1 week |
| Complex (60h) | 60 | 2 weeks |

### Step 2: Separate the implementation hours

```
consulting_impl_hours = total_phase2 - custom_development_hours
development_hours     = sum of every development line (>8h)
```

They are separated because consulting and development advance in parallel with different
capacities.

### Step 3: Implementation weeks

```
consulting_weeks  = ceil(consulting_impl_hours / profile_weekly_capacity)
development_weeks = ceil(development_hours / 40)
```

- Consulting uses the capacity of the chosen profile (40, 50 or 60 h/week)
- Development is always ~40h/week (a development team with fixed capacity)

```
impl_weeks = max(consulting_weeks, development_weeks)
```

Development runs in parallel with implementation; the phase lasts as long as the slower track.

### Step 4: Go-live weeks

```
golive_weeks = 4 (ALWAYS 1 month)
```

Covers production support, incident fixing, fine-tuning, reinforcement training.

### Step 5: Total

```
TOTAL_WEEKS = analysis_weeks + impl_weeks + golive_weeks
```

### Step 6: Project management

```
project_management = 1h × TOTAL_WEEKS
```

This line is added to the project's total hours. It generally does not change the week count (the
impact is marginal), but if the recalculation adds a week, iterate once.

---

## Worked Example

**Medium project**: 40h analysis + 220h implementation (of which 40h is development)

**Normal profile (50h/wk)**:
```
analysis_weeks    = 1
consulting_hours  = 220 - 40 = 180h
consulting_weeks  = ceil(180 / 50) = 4
development_weeks = ceil(40 / 40) = 1
impl_weeks        = max(4, 1) = 4
golive_weeks      = 4
TOTAL             = 1 + 4 + 4 = 9 weeks
project_mgmt      = 9h
```

**Accelerated profile (60h/wk)**:
```
consulting_weeks = ceil(180 / 60) = 3
impl_weeks       = max(3, 1) = 3
TOTAL            = 1 + 3 + 4 = 8 weeks
project_mgmt     = 8h
```

**Extended profile (40h/wk)**:
```
consulting_weeks = ceil(180 / 40) = 5
impl_weeks       = max(5, 1) = 5
TOTAL            = 1 + 5 + 4 = 10 weeks
project_mgmt     = 10h
```

---

## Typical Range Table

To sanity-check the calculated timeline:

| Project type | Analysis hours | Impl hours (range) | Total weeks (Normal) |
|---|---|---|---|
| Simple | 25 | 60–120 | 7–9 weeks |
| Medium | 40 | 150–300 | 10–14 weeks |
| Complex | 60 | 300–600 | 16–24 weeks |

If the calculation falls outside these ranges, revisit the estimated hours.

---

## Presentation Format

Present the three profiles in a comparison table, in the user's language:

```markdown
| Item | Accelerated | Normal | Extended |
|---|---|---|---|
| Weekly capacity | 60h/wk | 50h/wk | 40h/wk |
| Phase 1: Analysis | X wk | X wk | X wk |
| Phase 2: Implementation | X wk | X wk | X wk |
| Phase 3: Go-live | 4 wk | 4 wk | 4 wk |
| **Total** | **X wk** | **X wk** | **X wk** |
| Project management | Xh | Xh | Xh |
```

**Default recommendation**: the Normal profile, unless the user flags urgency (Accelerated) or
team constraints (Extended).
