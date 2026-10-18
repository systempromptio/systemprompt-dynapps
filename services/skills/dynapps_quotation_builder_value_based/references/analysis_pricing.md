# Functional Analysis and Custom Developments — Pricing Calculation

## Principle

The client sees a **closed price**, NEVER hours. All hour-based calculations are internal.

---

## Functional analysis

Ask the sales rep: **Online or on-site analysis?**

### Online

```
analysis_hours = (n_apps × 1.5h × 2) + (1h × n_developments) + 5h_preparation + 4h_defence
precio_analisis = analysis_hours × 80€/h        # precio_analisis IS a pack string key
```

### On-site

```
analysis_hours = (n_days × 8h × 2) + (1h × n_developments) + 5h_preparation + 4h_defence
precio_analisis = analysis_hours × 80€/h        # precio_analisis IS a pack string key
```

### No analysis

Allowed for simple projects. Omit this block.

### Presentation to the sales rep

```
FUNCTIONAL ANALYSIS:
  Type: Online
  Internal calculation: XX hours × €80/h (NOT visible to the client)
  Closed price: €X,XXX
  Includes: analysis meetings with each area's owners,
            functional analysis document, proposal defense

Validate the price, or adjust it?
```

The sales rep can round it (€1,840 → €1,800 or €2,000).

---

## Custom developments

Identify them from the "Integrations and Developments" section of the intake data and from the conversation.

### Internal calculation

```
development_price = estimated_hours × 75€/h
```

> **AI factor.** `estimated_hours` must be the real effort, with the AI factor already applied —
> delegate to the `odoo-effort-estimator` skill. If the hours come from a technical estimate, they
> already carry it: do not apply it twice. The functional analysis hours above are meetings and
> document work: they carry **no** AI factor.

### Reference hours table

| Type | Hour range | Example |
|------|-------------|---------|
| Simple API connector (1-2 endpoints) | 15-25h | Shopify billing connector |
| Complex API connector (3+ endpoints) | 25-50h | Manufacturing app integration |
| Carrier integration (per carrier) | 20-40h | SEUR, MRW, GLS |
| Shipping-rate comparator | 20-30h | Multi-carrier with optimal selection |
| Custom PDF | 5-10h | Custom invoice/delivery note |
| Custom report/dashboard | 10-20h | Custom reporting |
| Simple functional module | 15-25h | Fields + views + basic logic |
| Complex functional module | 40-80h | Complete new workflow |

### Presentation to the sales rep

```
CUSTOM DEVELOPMENT (additional features):
┌──────────────────────────────────┬────────────┬──────────────────────────────┐
│ Feature                          │ Price      │ Description                  │
├──────────────────────────────────┼────────────┼──────────────────────────────┤
│ Manufacturing app integration     │  €X,XXX    │ 3-endpoint API (stock/MO)    │
│ Shopify connector                │  €X,XXX    │ Online sales invoicing       │
│ Carrier integration               │  €X,XXX    │ SEUR + MRW + GLS + pallets   │
└──────────────────────────────────┴────────────┴──────────────────────────────┘

Validate, adjust prices, add or remove developments?
```

### Linking to apps

Each development can be:
- **Linked to an app** → appears on that app's sheet (e.g. the Spanish key "Conector Shopify" under Sales)
- **General** → separate "Additional Features" sheet (e.g.: carriers affecting several apps)

Ask the sales rep which developments are linked to which app.

---

## Price per app (implementation)

```
app_price = (base + Σ(addon.precio × addon.cantidad)) × (1 + adjustment_pct/100)
# `precio` / `cantidad` are real workbook/JSON keys - see addon_details.md
```

Round to whole numbers.

### Adjustment rules

- **±15%** free for the sales rep to set
- **>15%** requires a management-approval flag (highlight in orange)
- The adjustment is applied to `base + add-ons`, not just to the base
