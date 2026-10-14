# DOCX Variables Reference (docx.variable)

Variables are reusable Python code blocks that inject functions, computed values, or helpers
into the template rendering context. They are a **core mechanism** of `indaws_report_docx`
for encapsulating complex logic outside the template itself.

## How Variables Work

### Execution pipeline

```
1. Template rendering starts
2. docx.template.get_variables(eval_ctx) is called
3. For EACH docx.variable linked to the template:
   a. The python_code is executed via safe_eval(code, eval_ctx, mode="exec")
   b. The variable named `value` is extracted from the eval context
   c. `value` is stored in the rendering context under the variable's `name`
   d. `value` is removed from eval_ctx to avoid pollution
4. All collected values are merged into the template context
5. Template renders with access to all variable names
```

### Scope rules

- **Global variables** (`model_id = False`): Available to ALL templates regardless of model
- **Model-specific variables** (`model_id = sale.order`): Only available to templates targeting that model
- A template automatically gets ALL global variables + ALL variables for its model
- This is computed in `docx.template._compute_variables_ids`:
  ```python
  Var.search(['|', ('model_id', '=', False), ('model_id', '=', tmpl.model_id.id)])
  ```

### Execution context

Variables execute with the **full rendering context** available:

```python
# Available in python_code:
doc          # Current record being rendered
docs         # All records in the batch
env          # Odoo environment (env['model'].search(...))
ctx          # Request context dict
model        # Model class
UserError    # Exception class
time         # Python time module
datetime     # Python datetime module
dateutil     # dateutil module (relativedelta, parser, etc.)
pytz         # Timezone handling
log          # Logging function: log('message', level='info')
format_lang  # Number formatter: format_lang(value, digits=2)
format_date  # Date formatter: format_date(value, date_format='dd/MM/yyyy')
len          # Python len()
enumerate    # Python enumerate()
bs4          # BeautifulSoup (safe subset)
```

### The `value` contract

Every variable's `python_code` MUST set a variable named `value`. This is what gets
injected into the template context. `value` can be:

- **A function** (most common) — callable from the template
- **A static value** — string, number, dict, list
- **A class** — instantiable in the template
- **A builtin** — like `str`, `int`, `list`

## Built-in Variables (shipped with module)

These are created in `data/docx_variable.xml` and are **global** (no model_id):

### `pct` — Percentage formatting
```python
# python_code:
def format_pct(val):
    return "{} %".format(format_lang(val))
value = format_pct

# Template usage:
# {{ pct(doc.discount) }}        → "21 %"
# {{ pct(line.discount) }}       → "10 %"
```

### `format_float` — Locale-aware float formatting
```python
# python_code:
def format_float(val, digits=None):
    return format_lang(val, digits=digits)
value = format_float

# Template usage:
# {{ format_float(doc.weight, digits=3) }}     → "12,500"
# {{ format_float(doc.quantity) }}              → "100,00"
```

### `monetary` — Currency formatting with symbol
```python
# python_code:
def format_monetary(val, currency=None, digits=None):
    currency = currency or doc.currency_id
    return format_lang(val, digits=digits, currency_obj=currency)
value = format_monetary

# Template usage:
# {{ monetary(doc.amount_total) }}                    → "1.250,00 €"
# {{ monetary(doc.amount_total, doc.currency_id) }}   → "1.250,00 €"
# {{ monetary(line.price_subtotal) }}                 → "500,00 €"
```

### `str` — Type conversion
```python
# python_code:
value = str

# Template usage:
# {{ str(doc.sequence) }}     → "5"
# {{ str(doc.id) }}           → "42"
```

### `name_get` — Display name retrieval
```python
# python_code:
def name_get(record):
    return record.name_get()[0][1] if record else ''
value = name_get

# Template usage:
# {{ name_get(doc.partner_id) }}    → "Azure Interior (14)"
```

### `format_lang` — Number formatting (passthrough)
```python
# python_code:
value = format_lang

# Template usage:
# {{ format_lang(doc.amount, digits=2) }}           → "1.250,50"
# {{ format_lang(doc.quantity, digits=0) }}          → "100"
# {{ format_lang(doc.weight, digits=3, grouping=True) }}
```

### `format_date` — Date formatting (passthrough)
```python
# python_code:
value = format_date

# Template usage:
# {{ format_date(doc.date_order) }}                          → "15/01/2025"
# {{ format_date(doc.date_order, date_format='dd MMMM yyyy') }}  → "15 enero 2025"
```

## Creating Custom Variables

### Pattern 1: Simple function

The most common pattern — define a function and assign it to `value`:

```python
# Variable name: "tax_summary"
# Model: account.move
def tax_summary(move):
    """Return formatted tax breakdown string."""
    lines = []
    for group in move.amount_by_group:
        tax_name = group[0]
        tax_amount = group[1]
        base_amount = group[2]
        lines.append("{}: {} (base: {})".format(
            tax_name,
            format_lang(tax_amount, digits=2, currency_obj=move.currency_id),
            format_lang(base_amount, digits=2, currency_obj=move.currency_id)
        ))
    return "\n".join(lines)
value = tax_summary

# Template usage:
# {{ tax_summary(doc) }}
```

### Pattern 2: Computed value (depends on doc)

When the variable needs the current record at evaluation time:

```python
# Variable name: "is_overdue"
# Model: account.move
from datetime import date
value = doc.invoice_date_due and doc.invoice_date_due < date.today() and doc.payment_state != 'paid'

# Template usage:
# {% if is_overdue %}VENCIDA{% endif %}
```

**WARNING**: This pattern evaluates ONCE at variable load time. If rendering multiple records
in a single batch, `doc` refers to the LAST record. For per-record logic, use a function instead:

```python
# CORRECT: Function pattern for per-record logic
def check_overdue(record):
    from datetime import date
    return record.invoice_date_due and record.invoice_date_due < date.today()
value = check_overdue

# Template: {% if check_overdue(doc) %}VENCIDA{% endif %}
```

### Pattern 3: Lookup / dictionary helper

```python
# Variable name: "state_label"
# Model: sale.order
LABELS = {
    'draft': 'Borrador',
    'sent': 'Enviado',
    'sale': 'Confirmado',
    'done': 'Bloqueado',
    'cancel': 'Cancelado',
}
def state_label(state):
    return LABELS.get(state, state)
value = state_label

# Template usage:
# {{ state_label(doc.state) }}    → "Confirmado"
```

### Pattern 4: Aggregation / computation

```python
# Variable name: "line_stats"
# Model: sale.order
def line_stats(order):
    """Compute summary statistics for order lines."""
    lines = order.order_line.filtered(lambda l: not l.display_type)
    return {
        'count': len(lines),
        'total_qty': sum(lines.mapped('product_uom_qty')),
        'avg_price': sum(lines.mapped('price_unit')) / max(len(lines), 1),
        'has_discount': any(lines.mapped('discount')),
    }
value = line_stats

# Template usage:
# {% set stats = line_stats(doc) %}
# Total lineas: {{ stats.count }}
# Cantidad total: {{ format_lang(stats.total_qty, digits=0) }}
# {% if stats.has_discount %}Incluye descuentos{% endif %}
```

### Pattern 5: Environment access / cross-model queries

```python
# Variable name: "pending_deliveries"
# Model: sale.order
def pending_deliveries(order):
    """Get pending delivery orders for this sale."""
    pickings = env['stock.picking'].search([
        ('origin', '=', order.name),
        ('state', 'not in', ['done', 'cancel']),
    ])
    return pickings
value = pending_deliveries

# Template usage:
# {% set deliveries = pending_deliveries(doc) %}
# {% if deliveries %}
# Entregas pendientes:
# {%p for pick in deliveries %}
#   {{ pick.name }} - {{ format_date(pick.scheduled_date) }}
# {%p endfor %}
# {% endif %}
```

### Pattern 6: Formatting wrapper with defaults

```python
# Variable name: "fmt"
# Model: (global, no model_id)
class Formatter:
    """Utility formatting class."""
    @staticmethod
    def phone(val):
        """Format phone number."""
        if not val:
            return ''
        # Remove spaces and format
        clean = ''.join(c for c in val if c.isdigit() or c == '+')
        if len(clean) == 9:
            return f'{clean[:3]} {clean[3:6]} {clean[6:]}'
        return clean

    @staticmethod
    def nif(val):
        """Format NIF/CIF with separator."""
        if not val:
            return ''
        val = val.strip().upper()
        if val.startswith('ES'):
            val = val[2:]
        return val

    @staticmethod
    def address(partner, separator='\n'):
        """Format full address from partner record."""
        parts = filter(None, [
            partner.street,
            partner.street2,
            ' '.join(filter(None, [partner.zip, partner.city])),
            partner.state_id.name if partner.state_id else None,
            partner.country_id.name if partner.country_id else None,
        ])
        return separator.join(parts)

value = Formatter()

# Template usage:
# Tel: {{ fmt.phone(doc.partner_id.phone) }}
# NIF: {{ fmt.nif(doc.partner_id.vat) }}
# {{ fmt.address(doc.partner_id) }}
```

### Pattern 7: Conditional image helper

```python
# Variable name: "safe_image"
# Model: (global, no model_id)
def safe_image(record, field='image_128', width=15, height=None):
    """Return Image() call only if the record has the image field populated."""
    img_data = getattr(record, field, None)
    if not img_data:
        return ''
    # Return the raw base64 data - must be used with Image() in template
    return img_data
value = safe_image

# Template usage:
# {{ Image(safe_image(line.product_id, 'image_128'), width=15) }}
```

### Pattern 8: Table of contents / section grouping

```python
# Variable name: "grouped_lines"
# Model: sale.order
def grouped_lines(order):
    """Group order lines by sections."""
    groups = []
    current_section = {'name': 'General', 'lines': []}
    for line in order.order_line:
        if line.display_type == 'line_section':
            if current_section['lines']:
                groups.append(current_section)
            current_section = {'name': line.name, 'lines': []}
        elif not line.display_type:
            current_section['lines'].append(line)
    if current_section['lines']:
        groups.append(current_section)
    return groups
value = grouped_lines

# Template usage:
# {%p for group in grouped_lines(doc) %}
#   SECCION: {{ group.name }}
#   {%p for line in group.lines %}
#     {{ line.product_id.name }} - {{ monetary(line.price_subtotal) }}
#   {%p endfor %}
# {%p endfor %}
```

## XML Data File Format

To ship variables with an Odoo module:

```xml
<odoo>
    <!-- Global variable (available to all models) -->
    <record id="docx_variable_my_global_func" model="docx.variable">
        <field name="name">my_func</field>
        <field name="description">FUNCTION: Description of what it does</field>
        <field name="python_code">
def my_func(val):
    return str(val)
value = my_func
        </field>
    </record>

    <!-- Model-specific variable -->
    <record id="docx_variable_sale_custom" model="docx.variable">
        <field name="name">custom_helper</field>
        <field name="model_id" ref="sale.model_sale_order"/>
        <field name="description">FUNCTION: Sale order specific helper</field>
        <field name="python_code">
def custom_helper(order):
    # Complex logic here
    return result
value = custom_helper
        </field>
    </record>
</odoo>
```

## Best Practices

1. **Prefer functions over static values** — functions work correctly across multi-record rendering
2. **Name variables clearly** — they become template-level identifiers (e.g., `monetary`, `pct`, `grouped_lines`)
3. **Use description with prefix** — `FUNCTION:` for callables, `VALUE:` for static, `CLASS:` for classes
4. **Keep python_code minimal** — move complex logic to helper methods on the model if possible
5. **Handle None/empty gracefully** — always check for falsy values before operations
6. **Use `format_lang` and `format_date`** inside variables — they're available in eval_ctx
7. **Avoid side effects** — variables should be pure functions, not modify records
8. **Unique constraint**: `(name, model_id)` — one variable name per model, null model = global
9. **Global vs model-specific**: Use global for generic utilities (formatting, addresses), model-specific for business logic
10. **Variables can access other variables** — they share the eval_ctx, so order matters (execution order = search order)
