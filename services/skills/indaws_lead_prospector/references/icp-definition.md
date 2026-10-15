# Indaws Ideal Customer Profile (ICP)

This document defines the Ideal Customer Profile for Indaws lead prospecting.
All lead generation and qualification activities should reference these criteria.

---

## Company Profile

### Size

| Dimension | Range | Notes |
|-----------|-------|-------|
| **Employees** | 10-250 | Sweet spot: 20-100 employees |
| **Annual revenue** | 1M-50M EUR | Sweet spot: 2M-20M EUR |

Companies below 10 employees typically lack the complexity to justify full ERP implementation.
Companies above 250 employees often require enterprise-grade partners or direct Odoo implementation.

### Industries (Priority Order)

| Industry | Why good fit | Odoo modules relevant |
|----------|-------------|----------------------|
| **Manufacturing** | Complex operations, inventory, MRP needs | Manufacturing, Inventory, Purchase, Quality |
| **Distribution / Logistics** | Warehouse management, multi-location | Inventory, Purchase, Sales, Fleet |
| **Professional Services** | Project management, timesheets, billing | Project, Timesheet, Invoicing, HR |
| **Retail / Commerce** | POS, e-commerce, inventory | Point of Sale, eCommerce, Inventory |
| **Food & Beverage** | Traceability, expiration management, production | Manufacturing, Inventory, Quality, Traceability |
| **Construction** | Project management, subcontractor management | Project, Purchase, Fleet, Field Service |
| **Healthcare** | Patient management, regulatory compliance | Custom modules, Appointments, Invoicing |

### Geography

| Priority | Region | Details |
|----------|--------|---------|
| **Primary** | Comunidad Valenciana | Valencia, Alicante, Castellon provinces |
| **Secondary** | All Spain | National reach supported by Dynapps infrastructure |
| **Tertiary** | Spanish-speaking markets | Through Dynapps international network (future) |

Geographic proximity matters for presencial meetings, implementation support, and trust building.
Comunidad Valenciana is the home market where Indaws has the strongest brand presence and referral network.

---

## Technology Profile

### Current ERP Situation (Best to Worst Fit)

| Current situation | Fit level | Conversion approach |
|-------------------|-----------|---------------------|
| **No ERP** (Excel, manual processes) | Excellent | Greenfield implementation, show transformation potential |
| **Legacy ERP** (SAP Business One, Dynamics NAV) | Very good | Migration story, total cost reduction argument |
| **Sage / A3 / ContaPlus** | Very good | Outgrown current tool, modern platform upgrade |
| **Custom / in-house software** | Good | Replace technical debt, standardize on proven platform |
| **Basic cloud tools** (spreadsheets, standalone apps) | Good | Integration and automation benefits |
| **Competitor ERP** (Holded, Factorial, etc.) | Moderate | Feature gap analysis, scalability argument |
| **Already using Odoo** (another partner) | Low | Only if migration opportunity (unhappy with partner) |

### Technology Buying Signals

| Signal | Strength | Where to find |
|--------|----------|---------------|
| Job posting for ERP specialist/admin | Strong | LinkedIn Jobs, InfoJobs, Indeed |
| Job posting for IT manager or digital transformation role | Strong | Job boards |
| Recent leadership change (new CEO, CFO, CTO) | Strong | LinkedIn, company news |
| Significant funding round or investment | Strong | Press releases, startup databases |
| Rapid headcount growth (>20% YoY) | Moderate | LinkedIn, Glassdoor |
| Digital transformation mentions on website/LinkedIn | Moderate | Company content |
| Expansion to new locations or markets | Moderate | Company news |
| Compliance requirements (SII, TicketBAI) | Moderate | Industry-specific |
| Acquiring or merging with another company | Strong | Press releases |
| Upgrading website or digital presence | Weak | Observable changes |

---

## Decision Maker Profiles

### Primary Decision Makers

| Role (Spanish) | Role (English) | Typical concerns | Messaging angle |
|----------------|---------------|-------------------|-----------------|
| **CEO / Director General** | CEO | Business growth, efficiency, competitive advantage | Strategic transformation, ROI |
| **Gerente** | General Manager | Operations, costs, team productivity | Operational efficiency, visibility |
| **CFO / Director Financiero** | CFO | Cost control, reporting, compliance | Cost reduction, financial visibility, audit readiness |
| **Director de Finanzas** | Finance Director | Accounting, budgets, cash flow | Automated accounting, real-time reporting |

### Secondary Decision Makers (Influencers)

| Role (Spanish) | Role (English) | Typical concerns | Messaging angle |
|----------------|---------------|-------------------|-----------------|
| **COO / Director de Operaciones** | COO | Process efficiency, supply chain, quality | Process automation, operational excellence |
| **Director de Produccion** | Production Director | Manufacturing efficiency, planning, quality | MRP, scheduling, quality control |
| **CTO / Director de IT** | CTO | Technology stack, integration, security | Modern architecture, open source, API ecosystem |
| **Director de Sistemas** | IT Director | Implementation, maintenance, support | Odoo technical capabilities, support model |
| **Director de RRHH** | HR Director | HR management, payroll, employee experience | HR module, payroll, employee portal |
| **Director Comercial** | Sales Director | Sales pipeline, CRM, customer management | Odoo CRM, sales automation |

### Clay MCP Contact Filter Mapping

Use these `job_title_keywords` when searching for decision makers:

**C-level (all industries):**
```
["CEO", "Director General", "Gerente", "Consejero Delegado",
 "CFO", "Director Financiero", "Director de Finanzas",
 "COO", "Director de Operaciones",
 "CTO", "Director de IT", "Director de Sistemas", "Director de Tecnologia"]
```

**Extended (include influencers):**
```
["CEO", "Director General", "Gerente", "Consejero Delegado",
 "CFO", "Director Financiero", "Director de Finanzas",
 "COO", "Director de Operaciones", "Director de Produccion",
 "CTO", "Director de IT", "Director de Sistemas", "Director de Tecnologia",
 "Director de RRHH", "Director de Recursos Humanos",
 "Director Comercial", "Director de Ventas",
 "Responsable de IT", "Responsable Financiero", "Responsable de RRHH"]
```

---

## Disqualifiers

Do NOT pursue leads that match any of these criteria:

| Disqualifier | Reason |
|-------------|--------|
| **Already using Odoo** (and satisfied with current partner) | Low conversion probability, ethical concerns |
| **Fewer than 5 employees** | Odoo overkill, low deal value, high support cost ratio |
| **Public sector / government** | Complex procurement (licitaciones), long sales cycles, low margins |
| **Non-profit / associations** (unless >50 staff) | Budget constraints, complex decision-making |
| **Startups <2 years old with <10 employees** | Unstable, may not survive implementation period |
| **Companies in financial distress** | Payment risk, project cancellation risk |
| **Multinational HQ outside Spain** (ERP decisions made centrally) | Local entity cannot decide on ERP |

---

## Lead Scoring Alignment

The ICP criteria feed into the Quality Gate scoring system (see `quality-gate.md`):
- ICP industry match = +10 points
- Spanish location = +15 points
- Spanish TLD (.es) = +15 points
- Appropriate company size indicators = considered qualitatively

Leads that match multiple ICP criteria should be prioritized for outreach regardless of
the Quality Gate score alone. The Quality Gate focuses on data quality; ICP match focuses
on business fit.
