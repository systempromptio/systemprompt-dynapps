# Odoo Carriers — Canonical Skeleton & v18→v19 Migration

Goal: **reuse Odoo's own skeleton, don't reinvent it.** This file maps the exact
structure Odoo (CE + Enterprise) and OCA use for delivery carriers, the reference
providers to copy from per scenario, and the concrete v18→v19 migration aspects.
All facts verified against `addons/delivery`, `addons/stock_delivery`, the
enterprise `delivery_*` modules and `$OCA_SRC/v19/delivery-carrier` in 19.0.

---

## 0. How NOT to break it in the first place

Everything below is **repairs**. The rule that makes them unnecessary is one:

> **Do not type the name of a core model if a field already takes you to it.**

A module written that way is the *same file* on 18 and on 19. `stock.quant.package` →
`stock.package` only bites you if you **name** the model; read
`picking.move_line_ids.result_package_id` and call `_get_weight(picking.id)` — same signature,
same `{record: weight}` return in both versions — and portability is free. Same with
`models.Constraint` (don't declare SQL constraints) and `env._()` (use `_()`, valid in both).

Evidence: the v19→v18 backport of `indaws_delivery_cbl` took **4 lines and zero code changes**,
and not by luck — of the eight 18→19 deltas listed in §4 / pitfalls 19–26, **not one applied**.

Lock it in with a test that reads the module's own source and fails if the renamed model name
appears in it. Example: `indaws_delivery_cbl/tests/test_cbl_wire.py::TestCBLVersionPortability`.

---

## 1. The canonical skeleton (what every Odoo provider does)

> This describes the **shape** of an Odoo provider module (for reading reference
> providers). At generation time the frame files — `__manifest__.py`,
> `security/ir.model.access.csv` + ir.rules, `tests/` — are produced by
> **`odoo-module-creator`** (the base). This skill writes the **domain** files:
> `delivery_carrier.py`, `<provider>_request.py`, `stock_picking.py`, `data/`, `views/`.

A provider module is a thin extension of `delivery.carrier` plus a request/service
class. The dispatch is dynamic: `delivery.carrier.rate_shipment()` calls
`getattr(self, '%s_rate_shipment' % self.delivery_type)(order)`.

```
delivery_<provider>/
├── __manifest__.py                # depends: ['stock_delivery']  (or 'delivery' if no stock)
├── models/
│   ├── delivery_carrier.py        # _inherit delivery.carrier  → the 4 methods
│   ├── <provider>_request.py      # API client (a plain Python class, NOT a model)
│   └── stock_picking.py           # optional: provider-specific picking fields
├── data/<provider>_data.xml       # delivery.carrier + product master data
├── views/delivery_carrier_views.xml
├── security/ir.model.access.csv   # if new models (service/courier/channel)
└── tests/
```

### The 4 mandatory methods (naming is the contract — NO leading underscore)

Verified identical across shiprocket, easypost, fedex_rest, sendcloud (EE) and
correos_express (OCA) in 19.0:

```python
class DeliveryCarrier(models.Model):
    _inherit = 'delivery.carrier'

    delivery_type = fields.Selection(
        selection_add=[('<provider>', 'Label')],
        ondelete={'<provider>': lambda recs: recs.write(
            {'delivery_type': 'fixed', 'fixed_price': 0})},
    )

    def <provider>_rate_shipment(self, order):
        # → {'success': bool, 'price': float,
        #    'error_message': str|False, 'warning_message': str|False}

    def <provider>_send_shipping(self, pickings):
        # → list[dict] (one per picking) with at least
        #   {'exact_price': float, 'tracking_number': str}
        # labels: picking.message_post(body=..., attachments=[(name, bytes)])

    def <provider>_get_tracking_link(self, picking):
        # → str (single URL) or json.dumps(list[(ref, url)])

    def <provider>_cancel_shipment(self, picking):   # singular or plural per provider
        # cancel at carrier; clear carrier_tracking_ref
```

Optional hooks Odoo also exposes: `_get_delivery_type()` (when the dynamic prefix
differs from the stored `delivery_type`, e.g. easypost), `<provider>_get_default_custom_package_code()`,
`_<provider>_convert_weight()`.

---

## 2. Reference providers — copy the closest one, don't start blank

| Scenario | Copy from | Where |
|---|---|---|
| Modern REST, token auth, multi-package, labels+manifest | `delivery_shiprocket` | EE 19 |
| REST with service/rate catalog + insurance | `delivery_easypost` | EE 19 |
| Big REST (FedEx/UPS/DHL) + payment integration | `delivery_fedex_rest`, `delivery_ups_rest`, `delivery_dhl_rest` | EE 19 |
| Wizard-driven shipping + JS service widget | `delivery_starshipit`, `delivery_sendcloud` | EE 19 |
| **Spanish carrier, plain `requests`, request class** | **`delivery_correos_express`** | **OCA 19** |
| Fixed price / price rules (no API) | `delivery` base (`fixed`, `base_on_rule`) | CE 19 |

The **request class** pattern (`<provider>_request.py` / `_service.py`) is a normal
Python class instantiated as `Provider(self, self.log_xml)` — it holds the HTTP
calls, auth retry and payload building. It is **not** an Odoo model.

---

## 3. Reuse OCA instead of rebuilding (OCA `delivery-carrier`, $OCA_SRC/v19)

Available in 19.0 — prefer extending these to writing custom equivalents:

| Need | OCA module |
|---|---|
| **Tracking states + POD + history** (delivery_state, tracking_json, tracking_state_history, pod_file, date_shipped/delivered, delivered-email) | `delivery_state` |
| Number of packages on picking | `delivery_package_number` |
| Force fixed/rule price on methods | `delivery_price_method` |
| Manual shipping cost on SO | `delivery_carrier_manual_price` |
| Multiple destinations per method | `delivery_multi_destination` |
| Carrier ↔ partner / agency / options | `delivery_carrier_partner`, `delivery_carrier_agency`, `delivery_carrier_option` |
| Validate picking matches carrier reqs / dangerous goods | `delivery_carrier_picking_valid(_dangerous_goods)` |
| Reference Spanish API provider | `delivery_correos_express` |

> `delivery_state` is the OCA standard for carrier tracking states — use it (or
> align field names to it) instead of a bespoke state-tracking layer.
> **Note:** the classic `base_delivery_carrier_label` framework is **not yet
> ported to 19.0**; Odoo's native `message_post(attachments=...)` is the label path.

---

## 4. v18 → v19 migration checklist (carriers)

**Key insight:** provider code is mostly stable across 18→19 (e.g. `delivery_easypost`
has no method-signature changes, only line shifts). The breaks are at the
**framework level** in `stock_delivery` / `delivery`. Fix the call sites:

### Blocking (will raise / silently break)
1. **`stock.quant.package` → `stock.package`.** Model renamed in core stock 19.0
   (`stock/models/stock_package.py`). Replace every `stock.quant.package` reference,
   `env['stock.quant.package']`, `comodel_name=`, and `_inherit`. Packages of a
   picking: `picking.move_line_ids.result_package_id` (now `stock.package`). New
   `stock.package.history` exists.
2. **`_pre_put_in_pack_hook` moved** from `stock.picking` to `stock.package`, new
   signature `(package_id=False, package_type_id=False, package_name=False, from_package_wizard=False)`.
   Move any override there and call `super()` with the new args. `_set_delivery_package_type`
   is gone from `stock.picking` (package-type assignment now on the package).
   **Careful: in 19.0 the hook exists with TWO different signatures**, and picking the wrong model
   gives a silent argument mismatch:
   - on `stock.package` — `(package_id=False, package_type_id=False, package_name=False, from_package_wizard=False)`
     (`stock/models/stock_package.py:332`, `StockPackage._pre_put_in_pack_hook`, `stock_delivery/models/stock_package.py:35`)
   - on `stock.move` / `stock.move.line` — same kwargs but with `all_lines=False` **first**
     (`stock/models/stock_move_line.py:1040`, `StockMoveLine._pre_put_in_pack_hook`, v19; `stock_delivery/models/stock_move.py:103`)

   In 18.0 there was one signature, `(move_line_ids)`, on `stock.picking`
   (`stock/models/stock_picking.py:1775`, `Picking._pre_put_in_pack_hook`, `stock_delivery/models/stock_picking.py:117`).
3. **`_apply_margins(self, price)` → `_apply_margins(self, price, order=False)`.**
   Update overrides and call sites (e.g. starshipit overrode/called it in 18.0;
   core `send_to_shipper` now calls `_apply_margins(res['exact_price'], self.sale_id)`).

### Behavioral / signatures
4. **`available_carriers`/`_match*` take `source`, not `order`** — generalized to
   picking-or-order. Rename the param in overrides (`available_carriers(partner, source)`,
   `_match(partner, source)`, `_match_weight/volume/…(source)`).
5. **New `stock.picking._compute_allowed_carrier_ids`** restricts selectable
   carriers — account for it if a carrier "doesn't appear".
6. **Delivery now ships payment integration**: `delivery/models/payment_provider.py`,
   `payment_transaction.py`, `ir_http.py` (pay-on-delivery / onsite). New surface;
   don't assume the module matches 18.0. Several EE providers add their own
   `payment_provider.py` (ups_rest, shiprocket).

### Views & ORM (whole module, not carrier-specific but they bite)
7. `<list>` not `<tree>`; **no `<group>` inside `<search>`**.
8. `_compute_display_name` not `name_get()`; `aggregator` not `group_operator`.
9. Cron XML: no `numbercall` (removed in 18.0 — **0 ocurrencias** en CE18/CE19). **Pero NO
   "repite indefinidamente por defecto"**: v18 introduce auto-desactivación tras **5 fallos** y
   **7 días** (`odoo/addons/base/models/ir_cron.py:24-25`, `MIN_FAILURE_COUNT_BEFORE_DEACTIVATION`, lógica en `:474-488`, con aviso al admin) y el sustituto de
   `numbercall=1` es `_notify_progress(..., deactivate=True)` (`:734-749`, aplicado en `:444-445`).
   Detalle en `common-pitfalls.md` §18.

### Migration procedure
1. `grep -rn "stock.quant.package\|_pre_put_in_pack_hook\|_set_delivery_package_type\|_apply_margins\|name_get\|<tree>\|group_operator" <module>`.
2. Fix per §1–9 above. Re-point package model + hook; add `order`/`source` args.
3. Bump manifest to `19.0.x.y.z`; check `depends` (`stock_delivery`).
4. Confirm the 4 dispatch methods still match the contract in §1 (they rarely change).
5. Run the provider's own tests (`tests/test_delivery_<provider>.py`) against 19.0.
6. **Diff the master-data key sets, do not eyeball them.** A port silently drops entries from the
   error/state dictionaries and nothing ever fails — the code just answers `Unknown error (code N)`
   the day a customer hits one (11 of 46 shipment codes and 5 of 34 pickup codes went missing in a
   real TRANSLD port). For each dict in `<provider>_master_data.py`:

   ```bash
   python3 - "$OLD/<provider>_master_data.py" "$NEW/<provider>_master_data.py" <<'EOF'
   import ast, re, sys
   def keys(path, name):
       s = open(path, encoding="utf-8").read()
       blk = s[s.index(name + " = {"):]
       return set(re.findall(r'^\s{4}(-?\d+|"[^"]+"):', blk[:blk.index("\n}")], re.M))
   for name in ("SHIPMENT_ERROR_CODES", "PICKUP_ERROR_CODES"):  # adapt to the provider
       a, b = keys(sys.argv[1], name), keys(sys.argv[2], name)
       print(name, "perdidas:", sorted(a - b), "| nuevas:", sorted(b - a))
   EOF
   ```

   The old version is the source of truth here **only** where no provider document covers it (pickup
   codes usually), and the provider's sheet wins wherever it exists (see common-pitfalls #44, #47).
</content>
