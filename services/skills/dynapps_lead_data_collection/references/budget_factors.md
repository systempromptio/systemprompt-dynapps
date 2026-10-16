# Implementation Budget Impact Factors

Quick guide to the factors that multiply Odoo implementation hours. Use it to prioritise questions
and gauge complexity during data collection.

> **Internal reference.** Hours and ranges here are for the sales rep only — never quoted to the
> client during a discovery call. Currency and local systems named below are Spanish-market
> examples; substitute the client's own country equivalents.

---

## Global Multipliers

| Factor | Multiplier | Description |
|--------|:----------:|-------------|
| **Multi-company** | x1.5 – x2.0 | Each additional legal entity multiplies configuration (chart of accounts, taxes, documents, inter-company rules) |
| **Multi-currency** | x1.3 | Affects accounting, purchase, sales. Requires exchange-rate configuration |
| **Multi-language** | x1.2 | Interface, document and website translations. Higher impact with eCommerce |
| **Multi-warehouse** | x1.3 – x1.5 | Routes, reordering rules, inter-warehouse transfers |

---

## Additional Hours by Type

### Integrations (+20–40h per integration)

| Integration type | Estimated hours | Notes |
|------------------|:---------------:|-------|
| External eCommerce (Prestashop, WooCommerce) | 40–80h | Two-way sync of products + orders |
| Carriers (SEUR, MRW, GLS, national post) | 20–40h | Connectors exist, but configuration + testing |
| Payment gateways (Redsys, Stripe, PayPal) | 8–16h | Usually available as modules |
| Marketplace (Amazon, eBay) | 40–60h | Complex connectors covering stock + orders |
| Previous ERP (data migration) | 20–80h | Depends on volume and data cleanliness |
| In-house API / internal software | 40–120h | Custom development, complexity-dependent |
| Phone system (VoIP) | 20–40h | Integration + click-to-call configuration |
| Banks (SEPA, local payment files) | 8–16h | Payment batches, reconciliation |

### Data Migration

| Volume | Estimated hours | Notes |
|--------|:---------------:|-------|
| Light (<1,000 master records) | 8–16h | Customers + basic products |
| Medium (1,000–10,000 records) | 16–40h | Masters + categories + price lists |
| Heavy (>10,000 records) | 40–80h | Masters + history + data cleaning |
| With accounting history | +20–40h | Opening entries + balances + reconciliation |

### Reports and PDF Documents

| Type | Estimated hours | Notes |
|------|:---------------:|-------|
| Minor tweak to the standard PDF | 4–8h | Logo, colours, footer |
| Bespoke PDF (quotation/invoice) | 8–16h | Full corporate design |
| Complex PDF (grouped tables, conditionals) | 16–24h | Advanced logic |
| Bespoke management report | 16–40h | Dashboard, metrics, charts |

### Approval Flows

| Type | Estimated hours | Notes |
|------|:---------------:|-------|
| Simple approval (1 level) | 8–16h | Purchases or expenses by amount |
| Multi-level approval | 16–24h | Several approvers by amount/type |
| Complex flow with notifications | 24–40h | Approvals + alerts + escalation |

### Custom Development

| Type | Estimated hours | Notes |
|------|:---------------:|-------|
| Simple module (fields + views) | 16–40h | Custom fields, adjusted views |
| Medium module (business logic) | 40–80h | Flows, automation, reports |
| Complex module (integration) | 80–200h | APIs, complex processes, portal |
| Custom customer portal | 40–80h | Web pages + backend logic |

---

## Quick Table: Impact per Answer

Fast reference for gauging complexity during the call from the client's answers.

| Question | Simple answer (low impact) | Complex answer (high impact) | Extra hours |
|----------|---------------------------|------------------------------|:-----------:|
| Number of sales reps | 1–3 | >5 with teams and territories | +4–8h |
| Product variants | No / basic | Complex with combinations | +8–16h |
| Price lists | 1–2 | >3 with rules | +8–16h |
| Invoicing process | Per order | Mixed (hours + order + deposits) | +8–16h |
| E-invoicing | Not needed | Mandatory local regime | +8–24h |
| Number of warehouses | 1 | >2 with transfers | +16–24h |
| Picking | 1 step | 2–3 steps | +8–16h |
| Traceability | No | Lots / serial numbers | +16–24h |
| Scanners | No | Yes, handheld | +16–32h |
| Purchase approval | No | Yes, tiered | +16–24h |
| Manufacturing BOM | Don't manufacture / 1 level | Multi-level with work centres | +24–40h |
| Subcontracting | No | Yes | +16–24h |
| Payroll | External provider | Internal | +80–200h |
| eCommerce | No / basic | Custom-designed | +24–80h |
| Integrations | None | 1–2 systems | +20–80h |
| Data to migrate | <1,000 records | >10,000 + history | +40–80h |
| Bespoke PDFs | Standard with logo | Full corporate design | +8–24h |
| Multi-company | No | Yes (2+ companies) | x1.5–2.0 |

---

## Indicative Budget Ranges

**NOTE**: this information is INTERNAL to the sales rep. Do NOT share figures with the client
during the discovery call. It exists so the rep understands the magnitude of the project.

| Project type | Hour range | Typical profile |
|--------------|:----------:|-----------------|
| Basic | 80–150h | 1–2 apps, no development, clean data |
| Medium | 150–300h | 3–5 apps, some customisation, migration |
| Complex | 300–500h | 5+ apps, integrations, developments, multi-company |
| Enterprise | 500–1000h+ | Advanced manufacturing, eCommerce, multiple integrations |

---

## How to Use This Guide

1. **During preparation**: identify the factors already known (from the opportunity, attachments,
   website)
2. **During the call**: prioritise the HIGH-impact questions
3. **After the call**: add up the factors to estimate an hour range
4. **Rule of thumb**: 3+ high-impact factors → definitely a complex project
