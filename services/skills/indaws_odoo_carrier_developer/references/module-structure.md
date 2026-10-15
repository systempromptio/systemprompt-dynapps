# Module Structure Reference

## Directory Structure Template

```
delivery_<provider>/
├── __init__.py
├── __manifest__.py
├── models/
│   ├── __init__.py
│   ├── delivery_<provider>.py      # delivery.carrier _inherit + 4 methods + extended features
│   ├── <provider>_request.py       # Pure Python service class (HTTP calls)
│   ├── <provider>_master_data.py   # Optional: constants (if >50 tracking/error codes)
│   ├── stock_package_type.py       # Optional: package_carrier_type extension
│   └── stock_picking.py            # Optional: extra fields + buttons (label/tracking/POD)
├── data/
│   ├── delivery_<provider>.xml     # Default product + package types
│   └── ir_cron.xml                 # Optional: tracking update scheduler
├── security/
│   └── ir.model.access.csv         # Only if new models created
└── views/
    ├── delivery_<provider>.xml     # Form view with provider config fields
    └── stock_picking_views.xml     # Optional: picking buttons + extra fields
```

---

## Master Data File Rule

- If the carrier has **>50 tracking codes**, **>50 error codes**, or **>15 service codes**: create a separate `<provider>_master_data.py` with all constants (e.g., SEUR with 443 event codes, Enlaza with ~220 error codes across 8 endpoints).
- If **<=50 codes total**: put constants directly in `<provider>_request.py`.
- Carriers with per-endpoint error code dictionaries (e.g., separate dicts for shipment, label, tracking, cancel) -> always use master_data file.

---

## Cron for Tracking Updates (Odoo 18+)

```xml
<record id="ir_cron_<provider>_tracking" model="ir.cron">
    <field name="name"><Provider>: Update Tracking States</field>
    <field name="model_id" ref="stock_delivery.model_stock_picking"/>
    <field name="state">code</field>
    <field name="code">model._cron_<provider>_update_tracking()</field>
    <field name="interval_number">4</field>
    <field name="interval_type">hours</field>
    <field name="active">True</field>
</record>
```

**Note:** Do NOT use `numbercall` field -- it was removed in Odoo 18 (0 ocurrencias en CE18/CE19).
Para «una sola pasada» el sustituto es `_notify_progress(..., deactivate=True)`, y ojo con la
auto-desactivación por fallos que v18 añade: ver `common-pitfalls.md` §18.
