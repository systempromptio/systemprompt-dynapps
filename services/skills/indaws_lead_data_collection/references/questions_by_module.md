# Key Questions per Odoo Module

Implementation questionnaires condensed down to **the questions that move the budget**, organised
across 12 modules. Each module has:

- **Scoping questions** (3–5): decide whether the module is simple or complex
- **Complexity indicators**: answers that add hours
- **Red flags**: answers that imply custom development

**Total**: ~55 key questions for a ~1 hour call.

> **Language.** The questions below are English source text. Render them in the output language
> when writing the form (see SKILL.md `## Critical`). Localise fiscal references to the client's
> country — the Spanish examples (SII, TicketBAI, VeriFactu, SEPA files 19/34/58, Redsys, SEUR,
> MRW, GLS) are illustrative, not universal.

---

## 1. CRM

### Scoping questions

1. **How many sales reps will use the system?**
   - _Context_: drives licences and sales-team configuration
2. **Do you use a CRM today? Which one?**
   - _Context_: migrating from another CRM means data to import
3. **Do you have a defined sales process with stages?**
   - _Context_: if not, it has to be designed; if yes, it has to be replicated

### Complexity indicators
- More than 5 reps → teams and territories configuration
- Existing CRM with history → opportunity migration
- Several distinct pipelines → advanced configuration

### Red flags
- "We want LinkedIn / social media integration" → custom development likely
- "We need automatic lead scoring" → advanced configuration + possible development

---

## 2. Sales

### Scoping questions

4. **What type of product do you sell?** (physical, services, subscriptions, mixed…)
   - _Context_: changes product and invoicing configuration entirely
5. **Do you handle product variants (sizes, colours, …)?**
   - _Context_: complex variants multiply configuration hours
6. **Do you have several rate cards or price lists?**
   - _Context_: multi-pricing adds significant complexity
7. **Roughly how many orders/quotations do you handle per month?**
   - _Context_: high volume may require automation
8. **What is the sales process?** (quotation → confirmation → delivery → invoice)
   - _Context_: non-standard flows trigger customisation

### Complexity indicators
- Variants with combined attributes → +8–16h configuration
- More than 3 price lists → advanced pricing configuration
- Process with approvals → custom flow
- Selling kits/packs → kit BOM configuration

### Red flags
- "We need an online product configurator" → development
- "Price depends on complex formulas" → development
- "We sell by volume with complex tiered discounts" → advanced configuration

---

## 3. Invoicing

### Scoping questions

9. **How do you invoice today?** (per order, per delivery, per hour, deposits)
   - _Context_: sets the invoicing policy and its complexity
10. **How many invoices do you issue per month?**
    - _Context_: high volume may require batch invoicing
11. **Do you need mandatory e-invoicing?** (local regime — e.g. TicketBAI, VeriFactu, SII in
    Spain; Peppol, KSeF, FatturaPA elsewhere)
    - _Context_: each fiscal regime adds configuration
12. **Do you have customised invoice/quotation templates?**
    - _Context_: bespoke PDFs cost development hours

### Complexity indicators
- Mixed invoicing (per order + per hour) → advanced configuration
- More than 100 invoices/month → batch invoicing automation
- Mandatory e-invoicing → fiscal module + regime configuration

### Red flags
- Consolidation across several companies → multi-company + advanced accounting
- A heavily customised chart of accounts → extra configuration hours

---

## 4. Accounting

### Scoping questions

13. **Do you keep the books internally or with an external accountant?**
    - _Context_: an external accountant shrinks the accounting scope considerably
14. **Do you need a real-time tax reporting connection?** (e.g. SII in Spain)
    - _Context_: mandatory for certain companies — an additional module
15. **Do you work in multiple currencies?**
    - _Context_: multi-currency adds complexity across the board
16. **Do you need bank payment files?** (SEPA, local formats such as Spanish 19/34/58)
    - _Context_: specific banking configuration

### Complexity indicators
- Full internal accounting → chart of accounts + tax configuration
- Mandatory real-time tax reporting → fiscal module + certificate + testing
- Multi-currency → x1.3 on accounting hours
- Analytic accounting by department/project → advanced configuration

### Red flags
- "We need consolidation across several companies" → multi-company + advanced accounting
- "Our chart of accounts is heavily customised" → extra configuration hours

---

## 5. Inventory

### Scoping questions

17. **How many warehouses or locations do you have?**
    - _Context_: multi-warehouse multiplies configuration
18. **How do you pick orders?**
    - _Context_: 1 step vs 2 steps vs 3 steps changes the flow entirely
19. **Do you need lot or serial number traceability?**
    - _Context_: traceability adds significant complexity
20. **Do you use, or want to use, barcode scanners?**
    - _Context_: hardware requires integration and testing
21. **How many people work in the warehouse?**
    - _Context_: drives licences and training

### Complexity indicators
- More than 1 warehouse → routes, reordering rules
- Picking in 2–3 steps → complex flow
- Lot/serial traceability → +16–24h
- Scanners → +16–32h hardware integration

### Red flags
- "We need cross-docking" → advanced configuration
- "We have temperature-zoned warehouses" → development likely
- "We need carrier integration" (e.g. MRW, SEUR, DHL, UPS) → connector module

---

## 6. Purchase

### Scoping questions

22. **How many main suppliers do you manage?**
    - _Context_: supplier volume affects data migration
23. **Do you have purchase approval flows?**
    - _Context_: custom approvals require configuration or development
24. **Do you do dropshipping (supplier ships directly to the customer)?**
    - _Context_: dropshipping needs specific route configuration

### Complexity indicators
- More than 50 suppliers → data migration + purchase agreements
- Approval by amount → approval flow
- Dropshipping → special routes + configuration

### Red flags
- "We need a supplier portal for uploading invoices" → development
- "We want tenders and automatic comparisons" → advanced configuration

---

## 7. Manufacturing

### Scoping questions

25. **Do you manufacture your own products? What kind of manufacturing?**
    - _Context_: discrete vs continuous vs project-based changes the approach
26. **How many levels do your bills of materials have?**
    - _Context_: multi-level BOMs add significant complexity
27. **Do you have distinct work centres?**
    - _Context_: planning per work centre is advanced configuration
28. **Do you subcontract part of the manufacturing?**
    - _Context_: subcontracting has its own flow in Odoo

### Complexity indicators
- BOMs with 3+ levels → complex manufacturing route configuration
- Work centres with capacities → advanced planning
- Subcontracting → specific flow + quality control
- Quality control at receipt/production → additional module

### Red flags
- "We manufacture to order with configurable variants" → MTO development + configurator
- "We need MES/SCADA" → complex industrial integration
- "We have by-products and scrap to control" → advanced configuration

---

## 8. HR

### Scoping questions

29. **How many employees does the company have?**
    - _Context_: drives licences, migration and complexity
30. **Do you need attendance/time-clock tracking?**
    - _Context_: may require hardware (terminals) and integration
31. **Do you run payroll internally or through a provider?**
    - _Context_: payroll in Odoo is a project in its own right
32. **Do you need the employee portal?** (time off, expenses, timesheets)
    - _Context_: standard, but requires training

### Complexity indicators
- More than 50 employees → data migration + advanced configuration
- Time clocks with hardware → terminals + integration
- Internal payroll → +80–200h (a separate project is recommended)
- Multiple collective agreements / contract types → complex configuration

### Red flags
- "We need integration with our existing payroll system" → API development
- "We have complex rotating shifts" → shift planning development

---

## 9. eCommerce / Website

### Scoping questions

33. **Do you have an online shop today? On which platform?**
    - _Context_: catalogue, SEO and customer migration
34. **How many products will the online catalogue hold?**
    - _Context_: volume affects performance and configuration
35. **Which payment providers do you need?** (e.g. Stripe, PayPal, Adyen, Redsys)
    - _Context_: each provider needs configuration and testing
36. **Do you need carrier integration for shipping costs?**
    - _Context_: shipping connectors may be standard or custom
37. **Do you need a corporate website as well as the shop?**
    - _Context_: a corporate site with custom design adds hours

### Complexity indicators
- More than 500 products → bulk import + categorisation
- Custom web design → +24–80h
- Several payment providers → configuration + testing for each
- B2B + B2C with different prices → advanced configuration

### Red flags
- "We want a design identical to our current site" → custom theme development
- "We need a marketplace with multiple sellers" → complex development
- "Amazon/eBay integration" → specific connectors

---

## 10. Project

### Scoping questions

38. **Do you invoice time & materials or fixed price?**
    - _Context_: sets the project → invoicing flow
39. **Do employees need to record timesheets?**
    - _Context_: standard, but requires training
40. **How many simultaneous projects do you normally run?**
    - _Context_: volume drives the need for views and reports

### Complexity indicators
- Mixed invoicing (hours + fixed) → advanced contract configuration
- More than 20 simultaneous projects → dashboards and reports
- Projects with phases and dependencies → advanced configuration

### Red flags
- "We need an interactive Gantt with dependencies" → Odoo Enterprise + configuration
- "We want Jira / dev-tool integration" → API development

---

## 11. Helpdesk

### Scoping questions

41. **Through which channels do support requests arrive?** (email, phone, web, chat)
    - _Context_: each channel needs specific configuration
42. **Do you need SLAs?**
    - _Context_: custom SLAs require advanced configuration
43. **How many people are on the support team?**
    - _Context_: multiple teams with specialities → advanced configuration
44. **Do you need a knowledge base / FAQ for customers?**
    - _Context_: a self-service portal needs content + configuration

### Complexity indicators
- Multiple teams with assignment rules → advanced configuration
- SLAs differentiated by customer type → configuration + automation
- Customer portal with ticket tracking → module + configuration

### Red flags
- "We need phone-system integration" → VoIP development
- "We want an automatic chatbot" → AI/chatbot development
- "SLAs with contractual penalties and reporting" → advanced configuration + custom reports

---

## 12. Point of Sale

### Scoping questions

45. **How many stores / points of sale do you have?**
    - _Context_: multi-store requires per-location configuration
46. **How many terminals per store?**
    - _Context_: hardware + licences + synchronisation
47. **What hardware do you have or need?** (receipt printer, cash drawer, scanner, scale)
    - _Context_: each device requires integration
48. **Do you need a loyalty programme or gift cards?**
    - _Context_: loyalty is an additional module with configuration

### Complexity indicators
- More than 3 stores → multi-point configuration + synchronisation
- Specialised hardware (scale, label printer) → integration
- Simplified + full invoices → fiscal configuration
- Cash-closing reconciliation → training + configuration

### Red flags
- "We need weighing-system integration" → hardware development
- "We want self-checkout / kiosks" → interface development
- "Sales with reservations and deposits" → non-standard flow

---

## Cross-cutting Questions (apply to every project)

These always go in the "Customisations and Developments" and "Data Migration" sections of the
form, NOT in the individual modules:

49. **Do you need integrations with other systems?** (previous ERP, eCommerce, carriers, banks…)
50. **Do you have custom reports/PDFs?** (quotation, invoice, delivery note with corporate design)
51. **How much historical data do you need to migrate?** (customers, products, invoices, stock)
52. **Can you export your current data to Excel/CSV?**
53. **Is it multi-company?** (several legal entities)
54. **Do you work multi-language?** (interface, documents, website)
55. **Do you have custom approval needs?** (purchase, sales, expenses)

---

## Summary: Questions by Budget Impact

| Impact | Questions |
|---------|-----------|
| **HIGH** (>20h extra) | Multi-company, integrations, internal payroll, custom eCommerce, multi-level manufacturing, scanners, traceability |
| **MEDIUM** (8–20h extra) | Product variants, multi-warehouse, custom SLAs, bespoke PDFs, e-invoicing, bank payment files, approvals |
| **LOW** (<8h extra) | Price lists, sales teams, timesheets, knowledge base, POS loyalty |
