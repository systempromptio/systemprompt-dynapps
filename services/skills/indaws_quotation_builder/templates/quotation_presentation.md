# Template: Quotation Presentation

The presentation format shown to the user at Step 9, before confirming the write to Odoo.

> **Language.** Present this to the user in **their** language. The skeleton below is English.
> **Amounts stay in EUR.**
>
> **Estimation model.** The Timeline block below belongs to the **legacy** model
> (`timeline_rules.md`). Under the governing model (`hour_estimation.md`), replace it with the
> two-track calendar from Step 3.5. Show one model, never both (SKILL.md `## Critical` 4).

---

## Presentation Format

```markdown
## Implementation Quotation: {opportunity_name}

### General Information
- **Client**: {client_name}
- **Opportunity**: {opportunity_name} (ID: {id})
- **Classification**: {Simple / Medium / Complex}   <!-- legacy model only -->
- **Estimated users**: {N}
- **Hosting**: {Indaws Cloud / Odoo Online / Odoo.sh / On Premise}
- **Rescue Pack**: {Yes / No}
- **Analysis type**: {None / Online / On-site ({N} days)}

---

### Applications ({N} apps)

| Application | Hours | Level | Adjustment |
|---|---|---|---|
| {App 1} | {X}h | {Simple/Medium/Complex} | {justification if adjusted} |
| {App 2} | {X}h | {Simple/Medium/Complex} | {—} |
| ... | ... | ... | ... |

---

### Phase 0: Rescue Pack (if applicable)
- Rescue Pack: €995 (product ID 330)
- Duration: 1 week
- Prerequisite confirmed: source code access AND database access

### Phase 1: Functional Analysis — {X}h
- Functional analysis ({type}): {X}h
- Hourly rate: €80/h
- Subtotal: €{X}

{If on-site:}
- Travel costs: €{320–650}

### Phase 2: Implementation — {X}h

| Type | Concept | Hours | Price |
|---|---|---|---|
| Funcional | Technical configuration | 10 | €{X} |
| Funcional | Internal project management | {X} | €{X} |
| Funcional | Data migration | {X} | €{X} |
| Funcional | Module implementation | {X} | €{X} |
| Funcional | Meetings and training | {X} | €{X} |
| Desarrollo | {development name 1} | {X} | €{X} |
| Funcional | Go-live support | {X} | €{X} |
| | **Phase 2 subtotal** | **{X}h** | **€{X}** |

- Hourly rate: €75/h

> No "print formats" line: standard formats sit inside module implementation, custom formats are
> a Desarrollo concept (SKILL.md rule 10).

---

### Pain Points Identified

| # | Challenge | Odoo solution |
|---|---|---|
| 1 | {challenge 1} | {solution 1} |
| 2 | {challenge 2} | {solution 2} |
| 3 | {challenge 3} | {solution 3} |

---

### Indaws Core Packages

| Package | Price |
|---|---|
| {package 1} | €{price} |
| {package 2} | €{price} |
| **Total licences** | **€{total}** |

---

### Expert Team

| Expert | Area |
|---|---|
| {name 1} | {area 1} |
| {name 2} | {area 2} |

### Success Cases Linked

| Case | Sector |
|---|---|
| {case 1} | {sector 1} |
| {case 2} | {sector 2} |

---

### Timeline

**Governing model** — show the approved two-track calendar from Step 3.5 (functional 12h/week ∥
development 25h/week), with the total weeks and the project-management hours.

**Legacy model only** — the three pace profiles:

| Item | Accelerated | Normal | Extended |
|---|---|---|---|
| Weekly capacity | 60h/wk | 50h/wk | 40h/wk |
| Rescue Pack (if applicable) | 1 wk | 1 wk | 1 wk |
| Phase 1: Analysis | {X} wk | {X} wk | {X} wk |
| Phase 2: Implementation | {X} wk | {X} wk | {X} wk |
| Go-Live | 4 wk | 4 wk | 4 wk |
| **Total** | **{X} wk** | **{X} wk** | **{X} wk** |
| Project management | {X}h | {X}h | {X}h |

Selected profile: **{profile}**

---

### Recurring Services

| Item | Amount |
|---|---|
| Monthly maintenance | €{250/350/460}/month |
| Server ({type}) | €{amount}/year |
| Odoo Enterprise licence | €27.90/month (reference, Custom yearly, list price, Spain) |
| Estimated future migration | €{range} |

### Cost of staying on Odoo.sh (only if the current hosting is Odoo.sh)

Source of truth for dates and policy: `references/shared/odoo-lifecycle-licensing.md` —
never hardcode the numbers. Quantify the cost of NOT moving, against coming to Indaws/Dynapps
servers.

| Item | Impact |
|---|---|
| Their version's withdrawal from Odoo.sh | {date per version, from the SSOT} — a hard deadline that forces the migration |
| +25% surcharge on Enterprise renewal | {applies / possible / no} — confirm the client's renewal date |
| Annual overspend from the +25% | annual_licence_cost × 0.25 = €{X}/year |

### Demo
- URL: {demo_url or "Not available"}

---

### Financial Summary

| Item | Hours | Amount |
|---|---|---|
| Rescue Pack | — | €{995} |
| Analysis phase | {X}h | €{X} |
| Implementation phase | {X}h | €{X} |
| Core packages | — | €{X} |
| **TOTAL** | **{X}h** | **€{X}** |
```

---

## Presentation Rules

1. **Everything** in the quotation must be shown before asking for confirmation
2. Pain points are presented in summary form (challenge + solution)
3. Custom developments >8h appear as separate lines in Phase 2
4. Show **one** timeline model: the approved two-track calendar (governing) or the three pace
   profiles (legacy). Never both
5. Maintenance is tiered by project total by default; the scalable formula only appears when the
   consultant asked for the override
6. Rescue Pack appears only if the client already runs Odoo, and only with both access
   prerequisites confirmed
7. Travel costs appear only for an on-site analysis
8. **No separate "print formats" line** — standard formats are inside module implementation,
   custom formats are a Desarrollo concept
9. The expert team and success cases are shown as proposals (the user can adjust them)
10. This presentation is what the user sees before deciding whether it goes into Odoo
11. The "Cost of staying on Odoo.sh" block appears ONLY if the client's current hosting is
    Odoo.sh; the +25% needs the client's renewal date confirmed before an amount can be given
    (official rule in the SSOT)
