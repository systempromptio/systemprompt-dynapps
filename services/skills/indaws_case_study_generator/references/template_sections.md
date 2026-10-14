# Case Study Section Templates

Prose templates for each section. Replace `[PLACEHOLDER]` with actual data. Wrap each placeholder paragraph in a Word comment.

---

## Section 0 — Verification Banner (auto-generated, no comment needed)

```
Customer: [NAME] ([LEGAL NAME]) | Partner ID: [ID] | Tier: Hero | Date: [DATE]
Data sources: Odoo partner [ID], [N] sale orders, [N] invoices, [N] tasks, [N] projects, [N] helpdesk tickets.
Verified: [N] claims | Inferred from Odoo: [N] | Gaps needing interview: [N]
```

---

## Section 1 — Identifier Band

```
Sector: [SECTOR] · [SUBSECTOR]
```
*Comment: FILLED from Odoo or GAP depending on category_id tags.*

```
Footprint: [COUNTRY] — [CITY], [ADDRESS]. [EXPORT/MULTI-SITE NOTE]
```
*Comment: FILLED from Odoo partner address.*

```
Scale: [N] sale orders with Indaws ([YEAR]–[YEAR]). [N] posted invoices. ~EUR [TOTAL] total investment over [N]+ years.
```
*Comment: FILLED from Odoo sale.order + account.move counts.*

```
Dynapps partner: Indaws (Dynapps Group, Spain). [RELATIONSHIP SUMMARY WITH KEY DATES].
```
*Comment: FILLED from Odoo — list key sale orders with dates.*

```
Backing: [OWNERSHIP STRUCTURE]. (Source: transcript / [CONFIRM])
```

---

## Section 2 — How It Started

```
By [YEAR], [COMPANY] was already a [AGE]-year-old [TYPE] business. [OWNER NAME], [TITLE], ran the company from [LOCATION], [BUSINESS DESCRIPTION] (Source: transcript). The product side held together. The software did not.
```

```
[COMPANY] was running on [PREVIOUS SYSTEM] — [DESCRIPTION OF LIMITATIONS].
```
*Comment: GAP — Pre-Odoo system name. Ask: "What software were you using before Odoo — what was it called and what could it do?"*

```
When [REFERRAL SOURCE] suggested Odoo, [OWNER] faced a choice: [STAY vs. CHANGE framing].
```
*Comment: GAP — Decision trigger. Ask: "Who first recommended Odoo to you, and when was that conversation?"*

---

## Section 3 — The Challenge

```
The company's real problem was [CORE PROBLEM]. With every [PRODUCT TYPE] personalised — [SPECIFICS] — [COMPANY]'s [TEAM] needed [WHAT THEY NEEDED]. They did not have it.
```
*Comment: GAP — Pain vignette. Ask: "Give me one specific [order/customer/week] where something went wrong because the information wasn't centralised."*

```
A second voice inside the company shared the same frustration (Source: transcript, [SECOND SPEAKER]).
```
*Comment: GAP — Second speaker identity. FROM ODOO: [LIST CONTACTS]. Ask: "Who was the second person in the video — name and role?"*

---

## Section 4 — The Decision

```
[OWNER] [description of selection process]. [INDAWS] — the [LOCATION] Odoo partner — was the implementation choice.
```
*Comment: GAP — Options considered. Ask: "What other options did you look at — other ERPs, other Odoo partners, or staying on the old tool?"*

```
The starting point was [INITIAL SCOPE]: [DESCRIPTION]. (Source: Odoo records)
```
*Comment: FILLED from Odoo — first subscription/project order.*

---

## Section 5 — What We Built

```
The initial implementation covered the core [INDUSTRY] workflow: [MODULE LIST].
```
*Comment: FILLED from Odoo tasks — list tasks marked Hecho with task IDs.*

```
Custom developments added over time include: [CUSTOM DEV LIST].
```
*Comment: FILLED from Odoo helpdesk tickets and task descriptions — cite ticket dates and task IDs.*

```
[IF MIGRATION] In [YEAR], [COMPANY] migrated from Odoo v[N] to v[N], with Indaws handling the full database migration and module upgrade. [DETAILS]
```
*Comment: FILLED from Odoo migration tasks — cite task IDs.*

```
[IF AI PRODUCTS] In [YEAR], [COMPANY] adopted [AI PRODUCT LIST] from Indaws. [DETAILS]
```
*Comment: FILLED from Odoo sale order lines — cite order name, date, EUR amount.*

```
Additional integrations: [INTEGRATION LIST].
```
*Comment: FILLED from Odoo tasks/tickets — cite signal sources.*

---

## Section 6 — The Rollout

```
Go-live was [DESCRIPTION] — not because the software failed, but because [REASON] (Source: transcript).
```
*Comment: NOTE — go-live proxy from Odoo: first helpdesk ticket date = [DATE]. Confirm exact date with [PM NAME].*

```
The account was handled by [ACCOUNT OWNER] (Indaws account owner) and [PM NAME] (Indaws project manager), with the [COMPANY] side led by [CUSTOMER LEAD].
```
*Comment: FILLED from Odoo — user_id on partner + project PM.*

```
The biggest friction came not from [X] but from [REAL CAUSE] (Source: transcript).
```
*Comment: GAP — Named friction moment. Ask [OWNER] AND [PM]: "What was the hardest specific moment — a week when the team nearly gave up, a process that took multiple attempts?"*

```
One thing Indaws learned from the [COMPANY] project was [LEARNING] (Source: inferred).
```
*Comment: GAP — Indaws learning. Ask [PM]: "Is there a practice you now apply to every similar rollout that originated from what you learned here?"*

---

## Section 7 — The Outcome

```
The clearest result [OWNER] names is [OUTCOME] (Source: transcript).
```
*Comment: GAP — Metric. Ask: "How long did [X] take before Odoo? How long does it take now?"*

```
Operationally, the scale is visible in the data: [SCALE FIGURES] (Source: Odoo, [SOURCE REFERENCE]).
```
*Comment: FILLED from Odoo — cite source (migration task data, invoice count, etc.). Add: "Confirm with [OWNER] whether these numbers can be cited publicly."*

```
The loyalty to Indaws over [N]+ years — through [UPGRADE LIST] — is the most durable metric of the relationship (Source: Odoo records).
```
*Comment: FILLED from Odoo — list key upgrade milestones with dates.*

---

## Section 8 — In Their Words

```
"[QUOTE 1]" — [NAME], [TITLE], [COMPANY] (Source: transcript)
```

```
"[QUOTE 2]" — [SECOND SPEAKER NAME], [TITLE], [COMPANY] (Source: transcript)
```
*Comment: GAP — Second speaker quote. Identify speaker first (see Section 3 gap).*

---

## Section 9 — What Comes Next

```
[COMPANY] is now [DESCRIPTION OF NEXT PHASE]. The next chapter of their Odoo journey is [FRAMING].
```
*Comment: FILLED from Odoo if recent orders suggest next phase; otherwise GAP — ask: "What do you want Odoo to do in the next two years that it can't do today?"*

---

## Section 10 — Related Cases

```
Three related Indaws case studies: [CASE 1] · [CASE 2] · [CASE 3]
```
*Comment: GAP — Related case slugs. Ask Indaws content/marketing team for three matches based on sector, size, and migration journey.*
