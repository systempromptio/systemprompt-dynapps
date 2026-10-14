# Match System (Deduplication)

The match system prevents duplicate records on re-import. It requires **TWO things configured**:

## 1. `match=True` on Template Lines

Mark which template lines contain the match field:

```xml
<record id="line_provet_id" model="indaws.import.template.line">
    <field name="template_id" ref="template_species"/>
    <field name="names">provet_id</field>
    <field name="field_id" ref="field_animal_species__provet_id"/>
    <field name="match" eval="True"/>  <!-- Mark as match field -->
    <field name="sequence">10</field>
</record>
```

## 2. `match_line_ids` Records (REQUIRED)

**CRITICAL:** Setting `match=True` on template lines is **NOT ENOUGH**. You MUST also create
`indaws.import.template.match.line` records. The `_get_match` method uses `tmpl.match_line_ids`
to build the search domain, NOT `line_ids.filtered(match=True)`.

**IMPORTANT:** `match_field_ids` is a **COMPUTED field** (read-only). It is automatically
populated from `line_ids` where `match=True`. **NEVER write to `match_field_ids`** in XML or
Python -- it will silently fail or cause errors.

**In XML**, use `match_line_ids` with `Command.create` to define match criteria:

```xml
<record id="template_species" model="indaws.import.template">
    <field name="name">Import Species</field>
    <field name="model_id" ref="model_animal_species"/>
    <!-- Create match_line_ids records explicitly -->
    <field name="match_line_ids" eval="[
        Command.create({
            'sequence': 10,
            'model_id': ref('model_animal_species'),
            'field_id': ref('field_animal_species__provet_id'),
            'domain_operator': '=',
        }),
    ]"/>
</record>
```

**Via API/Python:**

```python
env['indaws.import.template.match.line'].create({
    'template_id': template.id,
    'field_id': provet_id_field.id,
    'domain_operator': '=',
    'sequence': 10,
})
```

**WITHOUT both, records will be duplicated on every import run.**

## How Matching Works Internally

```
_get_match(tmpl, row_vals):
    domain = []
    for match_line in tmpl.match_line_ids:  # <-- Uses match_line_ids, NOT line_ids.match
        field_name = match_line.field_id.name
        operator = match_line.domain_operator
        value = row_vals.get(field_name)
        domain.append((field_name, operator, value))
    return env[tmpl.model].search(domain, limit=1)
```
