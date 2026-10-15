# Materialise the MCP

Three ways to turn an exposure plan into real records. Pick by deliverable type.

| Path | When | Reproducible? |
|------|------|---------------|
| **Data module** | Client deliverable, staging→prod, version control | ✅ yes |
| **GOD server** | Done live in the client's instance, by the AI itself | partial (records, no file) |
| **odoo-pilot** | Quick prototype / your own testing | ❌ no (live records) |

All three create the *same* records: a catalog (`mcp.model` + `mcp.model.field`
[+ `mcp.method`]), a server (`mcp.server.config`), and exposures
(`mcp.server.model`). The difference is only *how*.

---

## Path A — Installable data module (recommended)

A tiny module depending on `dynapps_mcp_server`, carrying the server as data.
Reproducible, reviewable, deployable. Use the shipped idempotent helpers:

- `mcp.model._ensure_catalog(model_name, field_names=None, method_names=None)`
  → upserts the catalog entry + fields/methods. Returns the `mcp.model`.
- `mcp.server.model._ensure_god_exposure(server_xml_id, model_name, field_names=None, method_names=None)`
  → upserts a **read-only** exposure of a catalogued model on a server. Despite
  the name it works for any server resolved by xml-id (read the source before
  relying on it across versions). For write/method exposures, create the
  `mcp.server.model` record explicitly (below) so you control the `allow_*` flags.

**Layout**:
```
client_mcp_sales/
├── __manifest__.py            # depends: ['dynapps_mcp_server']; data: the 3 xml below
└── data/
    ├── 10_catalog.xml         # mcp.model + fields  (load first)
    ├── 20_server.xml          # mcp.server.config   (load second)
    └── 30_exposures.xml       # mcp.server.model    (load last)
```

**10_catalog.xml** — register models + the field positive-list:
```xml
<odoo noupdate="1">
    <function model="mcp.model" name="_ensure_catalog">
        <value>sale.order</value>
        <value eval="['name','partner_id','date_order','state','amount_total',
                      'amount_untaxed','amount_tax','currency_id','user_id',
                      'team_id','order_line','validity_date','client_order_ref']"/>
    </function>
    <function model="mcp.model" name="_ensure_catalog">
        <value>sale.order.line</value>
        <value eval="['order_id','product_id','name','product_uom_qty',
                      'price_unit','discount','tax_id','price_subtotal']"/>
    </function>
    <function model="mcp.model" name="_ensure_catalog">
        <value>res.partner</value>
        <value eval="['name','display_name','email','phone','vat',
                      'customer_rank','country_id']"/>
    </function>
    <function model="mcp.model" name="_ensure_catalog">
        <value>product.product</value>
        <value eval="['name','display_name','default_code','list_price','uom_id']"/>
    </function>
    <!-- expose a workflow method only if the plan is transactional -->
    <function model="mcp.model" name="_ensure_catalog">
        <value>sale.order</value>
        <value eval="[]"/>
        <value eval="['action_confirm']"/>
    </function>
</odoo>
```

**20_server.xml** — the server with safe defaults + instructions:
```xml
<odoo noupdate="1">
    <record id="mcp_server_sales" model="mcp.server.config">
        <field name="name">Client Sales MCP</field>
        <field name="active" eval="True"/>
        <field name="enable_tools" eval="True"/>
        <field name="core_tools" eval="True"/>
        <field name="enable_prompts" eval="False"/>
        <field name="allow_sudo_tools" eval="False"/>
        <field name="company_id" ref="base.main_company"/>
        <field name="group_ids" eval="[(6,0,[ref('sales_team.group_sale_salesman')])]"/>
        <field name="instructions"><![CDATA[You manage the Sales area of this Odoo.
Quotations and orders are sale.order with lines in sale.order.line. Resolve
customers (res.partner) and products (product.product) by name before filtering
or creating. Use aggregate for counts/totals by state, salesperson or period.
Amounts are in the order's currency_id. You may read everything in scope; create
and confirm quotations only when asked.]]></field>
    </record>
</odoo>
```

**30_exposures.xml** — wire catalog → server. Read-only via the helper; write/method
exposures as explicit records:
```xml
<odoo noupdate="1">
    <!-- read-only models -->
    <function model="mcp.server.model" name="_ensure_god_exposure">
        <value>client_mcp_sales.mcp_server_sales</value>
        <value>res.partner</value>
        <value eval="['name','display_name','email','phone','vat','customer_rank','country_id']"/>
    </function>
    <function model="mcp.server.model" name="_ensure_god_exposure">
        <value>client_mcp_sales.mcp_server_sales</value>
        <value>product.product</value>
        <value eval="['name','display_name','default_code','list_price','uom_id']"/>
    </function>

    <!-- transactional model: explicit record so we control allow_* + methods -->
    <record id="exp_sale_order" model="mcp.server.model">
        <field name="server_id" ref="mcp_server_sales"/>
        <field name="model_id" ref="..."/>   <!-- the mcp.model for sale.order -->
        <field name="allow_read" eval="True"/>
        <field name="allow_create" eval="True"/>
        <field name="allow_write" eval="True"/>
        <field name="allow_unlink" eval="False"/>
        <!-- field_ids / method_ids: link the catalogued mcp.model.field /
             mcp.method records (search by field_name/method_name) -->
    </record>
</odoo>
```
Resolving `model_id`/`field_ids` to the catalogued records is fiddly in pure XML;
the pragmatic pattern is a tiny post-load `<function>` that looks them up by name
(mirror `_ensure_god_exposure`, but set `allow_write=True` and attach the
method). Keep `noupdate="1"` so admins can tune flags in the UI without the next
upgrade reverting them.

Install/upgrade with the odoo-pilot scripts (`install_module.sh` /
`upgrade_module.sh`), then go to **Validate**.

---

## Path B — In-product GOD server

When the client wants the AI to build it live, drive the GOD server's tools in
order — each step returns a token the next consumes:

1. `list_available_models` / `inspect_model` — discover, check
   `list_sensitive_fields` and `list_blacklisted_methods`.
2. `add_to_catalog` — register model + field/method subset.
3. `propose_server_config` — draft the server + exposures + CRUD flags → fingerprint.
4. `validate_proposal` — returns a `confirm_token` (sha256[:16] of the proposal;
   any edit invalidates it).
5. `apply_proposal` — persist with the token.

GOD enforces the same invariants (never `sudo_execution`/`allow_sudo_tools` in a
proposal; custom tools land `active=False` for human review). Your job is to feed
it the **domain playbook** so the proposal is the *right* narrow exposure, not
everything.

---

## Path C — Live via odoo-pilot (prototype only)

Per global rules, all live Odoo API work goes through the `odoo-pilot` scripts —
never ad-hoc rpc. Authenticate first (`eval $(./scripts/auth.sh)`), then
`create_record.sh` against `mcp.server.config`, `mcp.model`, `mcp.model.field`,
`mcp.server.model`. Good for a throwaway prototype on staging; **not** a
deliverable — there's no version-controlled artefact and it won't reproduce on
prod. Promote a validated prototype to Path A before handoff.
