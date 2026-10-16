# Template: Discovery Data Collection Form

## Usage

This template is the base structure of the form the sales rep uses during the discovery call.
The LLM must:

1. **Pre-fill** fields with extracted information (opportunity, attachments, website, chatter)
2. **Mark the source** of pre-filled data: `[PREFILLED - website]`, `[PREFILLED - attachment]`,
   `[PREFILLED - chatter]`, `[PREFILLED - opportunity]`
3. **Mark pending** fields with `[ASK]`
4. **Organise modules** into Part A (detected) and Part B (the rest)

> **Language.** This template is English. **Render the form in the output language** (default: the
> language of the CRM record) — headings, labels and questions all together. The `[PREFILLED …]`
> and `[ASK]` markers stay English in the working draft; the visible pending label is localised
> only in the final HTML. See SKILL.md `## Critical`.

**Data placeholders** (from the extraction JSON):

- `{opportunity_name}` — crm.lead name
- `{partner_name}` — res.partner name
- `{date}` — generation date
- `{sales_rep_name}` — name of the lead's user_id
- `{industry}` — partner industry
- `{employee_count}` — partner employee count
- `{city}` — partner city
- `{website}` — partner website

**Pre-fill rules:**

- NEVER invent, assume or infer data — only information verified 100% in the sources
- A field without confirmed information is always marked `[ASK]` with a specific question
- CORRECT: `[ASK: how many users will need access?]`
- WRONG: `Roughly 20 users (estimated from the sector)` — that is an assumption; do not do it
- State the source in parentheses: (website), (attachment: name.pdf), (chatter: date), (opportunity)
- At most ~50 active `[ASK]` questions in Part A (a 1-hour call does not fit more)

**Section 3 rules (Functional Scope):**

- **Part A**: modules the analysis found relevant. Include a short explanation of why each was
  detected.
- **Part B**: ALL remaining modules with their questions. The rep can delete what does not apply.
- Each module's questions come from `references/questions_by_module.md`

---

## Form Template

```markdown
# DATA COLLECTION - {opportunity_name}

**Client**: {partner_name} | **Date**: {date} | **Sales rep**: {sales_rep_name}
**Sources analysed**: CRM opportunity | Attachments ({n_attachments}) | Client website ({yes/no}) | Chatter ({n_messages} messages)

---

## 1. GENERAL PROFILE

- **Project type**: [New implementation / Migration from {software}]
- **Current software**: {current_software}
- **Sector / activity**: {industry}
- **Business description**: {business_description}
- **Total employees**: {employee_count}
- **Expected Odoo users**: [ASK]
- **Locations / sites**: {city}
- **Multi-company**: [Yes (number of companies: ___) / No]
- **Multi-language**: [Yes (languages: ___) / No]
- **Website**: {website}

---

## 2. OBJECTIVES AND MOTIVATION

- **Why do they want to change system?**: {reason_for_change}
- **Main pain point**: {pain_point}
- **Project objective**: {objective}
- **Desired go-live window**: [ASK]
- **Is there a deadline or urgency?**: [ASK]
- **What do they expect the implementation to achieve?**: [ASK]

---

## 3. FUNCTIONAL SCOPE

### --- PART A: modules detected as relevant ---

_Modules the available information says this project needs. A short note explains why each was
detected._

#### 3.A.{n} {Module Name}

> **Why detected**: {short explanation of the triggering signal}

- **Question 1**: {pre-filled value or [ASK]}
- **Question 2**: {pre-filled value or [ASK]}
- **Question 3**: {pre-filled value or [ASK]}
- _(etc., per this module's questions in questions_by_module.md)_
- **Additional comments**: _(space for the rep's notes during the call)_

<!-- Repeat the block for each detected module -->

---

### --- PART B: other modules (include if applicable) ---

_Every remaining Odoo module with its questions. The rep can delete whatever clearly does not
apply. If one of these comes up during the call, the questions are already prepared._

#### 3.B.{n} {Module Name}

- **Question 1**: [ASK]
- **Question 2**: [ASK]
- **Question 3**: [ASK]
- _(etc., per this module's questions in questions_by_module.md)_
- **Additional comments**: _(space for the rep's notes during the call)_

<!-- Repeat the block for each undetected module -->

---

## 4. CUSTOMISATIONS AND DEVELOPMENTS

### Integrations

- **Do they need integration with other systems?**: {integrations_detected or [ASK]}
  - System 1: ___ → data type, direction (Odoo↔external), frequency
  - System 2: ___ → data type, direction, frequency

### Custom flows

- **Do they have special, non-standard processes?**: {flows_detected or [ASK]}
- **Do they need approval flows?**: [ASK] (purchases, expenses, sales…)

### Reports and documents

- **Do they need bespoke PDF reports?**: [ASK] (quotation, invoice, delivery note with corporate design)
- **Do they have current templates they want replicated?**: [ASK]

### Fields and functionality

- **Do they need special fields or functionality?**: [ASK]
- **Is there a critical process we have not covered?**: [ASK]

---

## 5. DATA MIGRATION

- **Master data to migrate**: [Customers / Products / Suppliers / Chart of accounts / Employees]
- **Approximate volume**:
  - Customers: [ASK]
  - Products: [ASK]
  - Suppliers: [ASK]
  - Historical invoices: [ASK]
- **Can they export to Excel/CSV from their current system?**: [ASK]
- **Historical data**: [How many years need migrating?]
- **Is there data in other formats (paper, Access, …)?**: [ASK]
- **Additional comments**: _(space for the rep's migration notes)_

---

## 6. COMMERCIAL INFORMATION

- **Client's indicative budget**: [ASK - tactfully, do not push]
- **Decision process**: [ASK] (Who decides? Is there a committee? What timeline?)
- **Are they evaluating other ERPs/solutions?**: [ASK]
- **Key decision-maker(s)**: [ASK] (name, role, part in the decision)
- **Next steps agreed**: ___
- **Overall impression**: ___

---

## 7. ADDITIONAL NOTES

_(Free space for the rep's observations during or after the call)_

-
-
-

```

---

## Module Detection Rules (Part A)

Criteria for classifying a module as "detected" into Part A:

| Module | Detected if… |
|--------|--------------|
| CRM | Sales process, sales team, pipeline, lead generation mentioned |
| Sales | Quotations, orders, sales process, price lists mentioned |
| Invoicing | Invoicing, invoices, collections mentioned |
| Accounting | Accounting, tax reporting, taxes, external accountant mentioned |
| Inventory | They sell physical products, warehouse, stock |
| Purchase | Suppliers, purchasing, procurement mentioned |
| Manufacturing | They manufacture products, production, BOM |
| HR | Employees, payroll, time off, time clocks mentioned |
| eCommerce/Website | They have an online shop, website, internet sales |
| Project | They sell services, projects, timesheets |
| Helpdesk | Support, tickets, incidents, SLA mentioned |
| Point of Sale | They have physical stores, point of sale, retail |

**Rule**: EVERY module is detected from signals in the analysed information. No module goes into
Part A by default — without evidence of need, it goes to Part B.
