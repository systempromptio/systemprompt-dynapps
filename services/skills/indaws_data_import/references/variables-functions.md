# Variables as Reusable Functions

Variables (`indaws.import.variable`) can return **functions**, not just values. This is powerful
for creating reusable helpers across multiple collectors.

## Example: URL Resolver Function

```xml
<record id="variable_url_resolver" model="indaws.import.variable">
    <field name="name">url_resolver</field>
    <field name="python_code">
def _resolve(url, model_name=None):
    if not url:
        return False
    try:
        pid = int(str(url).rstrip('/').split('/')[-1])
    except (ValueError, IndexError):
        return False
    if not model_name:
        return pid
    record = env[model_name].search([('provet_id', '=', pid)], limit=1)
    return record.id if record else False

value = _resolve
    </field>
</record>
```

**Why this works:** `safe_eval` with `nocopy=True` preserves the function's closure, so `env`
remains accessible when the function is called later from the collector context.

## Using in Collectors

Link the variable to the collector via `variable_ids`:

```xml
<record id="collector_breeds" model="indaws.import.data.collector">
    <field name="variable_ids" eval="[(4, ref('module.variable_url_resolver'))]"/>
    <field name="python_code">
# url_resolver is now a callable function in this context
species_odoo_id = url_resolver(r.get('species', ''), 'animal.species')
    </field>
</record>
```
