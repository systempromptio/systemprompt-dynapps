# docxtpl Syntax Reference for indaws_report_docx

Complete reference for the Jinja2-based template syntax used by `docxtpl` within the `indaws_report_docx` module.

## Context Variables

Every template has access to these variables automatically:

| Variable | Type | Description |
|---|---|---|
| `doc` | Record | The current Odoo record being rendered |
| `docs` | RecordSet | All records being rendered |
| `env` | Environment | Odoo environment (`env['model.name'].search(...)`) |
| `ctx` | dict | Request context |
| `model` | Model | The model class (e.g., `env['sale.order']`) |
| `time` | module | Python `time` module |
| `datetime` | module | Python `datetime` module |
| `dateutil` | module | `dateutil` module (relativedelta, etc.) |
| `pytz` | module | Timezone handling |
| `len` | function | Python `len()` |
| `enumerate` | function | Python `enumerate()` |

## Variable Insertion

### Simple variables
```
{{ doc.name }}                    → Plain text insertion
{{ doc.partner_id.name }}         → Dotted access to related fields
{{ doc.amount_total }}            → Number (raw, no formatting)
```

### RichText variables (prefix `r`)
```
{{ r Html2Docx(doc.note) }}       → Insert formatted HTML as rich text
{{ r my_richtext_var }}           → Insert any RichText object
```

**CRITICAL**: The `r` prefix tells docxtpl to insert a `RichText` object. Without it, RichText objects render as their Python repr string. Always use `{{ r }}` with `Html2Docx()`.

### Paragraph-level variables (prefix `p`)
```
{{ p doc.long_description }}      → Replace entire paragraph with content
```

## Control Structures

### Conditionals
```
{% if doc.note %}
  {{ doc.note }}
{% endif %}

{% if doc.state == 'sale' %}
  Confirmed Order
{% elif doc.state == 'draft' %}
  Quotation
{% else %}
  Other
{% endif %}
```

### Loops — Paragraph level (`{%p %}`)
Use inside regular paragraphs (NOT in tables):
```
{%p for line in doc.order_line %}
  {{ line.product_id.name }} - {{ monetary(line.price_subtotal) }}
{%p endfor %}
```

### Loops — Table row level (`{%tr %}`)
Use to repeat entire table rows:
```
┌────────────────────────────────────────────────────┐
│ Product       │ Qty    │ Price    │ Subtotal       │  ← Header row (static)
├────────────────────────────────────────────────────┤
│{%tr for line in doc.order_line %}                  │
│{{ line.product_id.name }}│{{ line.product_uom_qty }}│{{ format_lang(line.price_unit, digits=2) }}│{{ monetary(line.price_subtotal) }}│
│{%tr endfor %}                                      │
└────────────────────────────────────────────────────┘
```

**IMPORTANT**: `{%tr %}` and `{%tr endfor %}` must be placed in the **first cell** of the row. The entire row gets duplicated per iteration.

### Loops — Table column level (`{%tc %}`)
Use to repeat entire table columns (rare):
```
{%tc for col in columns %}
{{ col.header }}
{%tc endfor %}
```

### Loop Variables
Inside loops, standard Jinja2 loop variables are available:
```
{{ loop.index }}      → 1-based iteration counter
{{ loop.index0 }}     → 0-based iteration counter
{{ loop.first }}      → True on first iteration
{{ loop.last }}       → True on last iteration
{{ loop.length }}     → Total number of items
```

## Built-in Functions

### Image Insertion
```
{{ Image(doc.company_id.logo, width=30) }}
{{ Image(doc.company_id.logo, width=30, height=20) }}
{{ Image(doc.product_id.image_128, width=15, height=15) }}
```
- `image`: Base64-encoded binary field from Odoo
- `width`: Width in **millimeters**
- `height`: Height in **millimeters** (optional, maintains aspect ratio if omitted)
- `rotate`: Degrees to rotate (optional)
- Returns empty string if image is falsy (safe to use without `{% if %}`)

### HTML to Rich Text
```
{{ r Html2Docx(doc.note) }}
{{ r Html2Docx(doc.description_sale) }}
```
- Converts Odoo HTML fields to Word-compatible formatted text
- Supports: bold, italic, underline, strikethrough, color, font size, lists
- **Must use `{{ r }}` prefix** — RichText insertion
- Returns empty string if HTML is empty

### HTML to Plain Text
```
{{ Html2Text(doc.note) }}
```
- Strips all HTML formatting, returns plain text
- Useful when rich formatting is not needed

### Barcode Generation
```
{{ Barcode(doc.name, 'Code128', width=50, height=15) }}
{{ Barcode(doc.name, 'QR', width=25, height=25) }}
{{ Barcode(doc.ean13, 'EAN13', width=40, height=20) }}
```
- `value`: String to encode
- `barcode_type`: 'Code128', 'QR', 'EAN13', 'EAN8', 'UPC-A', 'DataMatrix', etc.
- `width`/`height`: Dimensions in **millimeters**
- `dpi`: Resolution (default 300)
- `rotate`: Degrees to rotate (optional)

### Image Rotation
```
{{ Rotate(some_image_bytes, 90) }}
```
- Rotates a PIL-compatible image by degrees
- Returns BytesIO stream

## Formatting Functions (from docx.variable)

### Currency Formatting
```
{{ monetary(doc.amount_total) }}
{{ monetary(doc.amount_total, doc.currency_id) }}
{{ monetary(doc.amount_total, doc.currency_id, digits=2) }}
```
- Uses `doc.currency_id` by default if not specified
- Adds currency symbol with non-breaking space
- Locale-aware (respects language setting)

### Number Formatting
```
{{ format_lang(doc.amount_total, digits=2) }}
{{ format_lang(doc.quantity, digits=0) }}
{{ format_float(doc.weight, digits=3) }}
```
- `digits`: Number of decimal places
- Locale-aware thousand separators and decimal marks

### Date Formatting
```
{{ format_date(doc.date_order) }}
{{ format_date(doc.date_order, date_format='dd/MM/yyyy') }}
{{ format_date(doc.date_order, lang_code='es_ES') }}
```
- Default format follows user's locale
- Custom format uses ICU pattern (dd/MM/yyyy, not Python's %d/%m/%Y)

### Percentage
```
{{ pct(doc.discount) }}
```
- Adds ` %` suffix with locale-formatted number

### String Conversion
```
{{ str(doc.sequence) }}
```
- Converts any value to string

## Escaping and Edge Cases

### Handling None values
```
{{ doc.phone or '' }}                    → Empty string fallback
{% if doc.phone %}{{ doc.phone }}{% endif %}  → Conditional display

{{ doc.partner_id.name or 'N/A' }}       → Default value
```

### Multi-line text
```
{{ doc.description|replace('\n', '\a') }}
```
The `\a` character creates a line break within a paragraph in docxtpl.

### Preserving whitespace
```
{{ doc.code }}                           → Normal (trimmed)
```

### Using filters
Standard Jinja2 filters work:
```
{{ doc.name|upper }}                     → UPPERCASE
{{ doc.name|lower }}                     → lowercase
{{ doc.name|title }}                     → Title Case
{{ doc.name|truncate(50) }}              → Truncate to 50 chars
{{ doc.items|join(', ') }}               → Join list
{{ doc.amount|round(2) }}               → Round number
{{ doc.name|default('N/A') }}           → Default if None/undefined
```

## Table Patterns

### ⚠ Two hard rules for tables (docxtpl 0.16.7 AND 0.20, verified)

1. **Every cell MUST contain a `<p>` (paragraph).** A "bare" cell (text typed directly into the cell
   without a paragraph) makes the export flatten or crash. When you build the table by hand or via
   python-docx, ensure each cell has at least one paragraph.
2. **`{%tr ... %}` in the SAME row as the data BREAKS.** Putting the `{%tr for %}` and `{%tr endfor %}`
   on the same physical row as the content raises errors (`unknown tag 'tr'`/`endfor`). Use **for/endfor
   on their OWN separate rows** (tag-row above + data-row + tag-row below) — docxtpl deletes the tag-rows
   and repeats the data-row — **or** use `{%p ... %}` (paragraph-level loop) instead of `{%tr%}`. This
   also applies when a browser/editor (e.g. SuperDoc) produces the DOCX: same rule.

### Basic table with header + loop rows
```
┌──────────────────────────────────────────────────────┐
│ # │ Product         │ Qty │ Unit Price │ Subtotal    │
├──────────────────────────────────────────────────────┤
│{%tr for line in doc.order_line %}                    │
│{{ loop.index }}│{{ line.product_id.name }}│{{ format_lang(line.product_uom_qty, digits=0) }}│{{ format_lang(line.price_unit, digits=2) }}│{{ monetary(line.price_subtotal) }}│
│{%tr endfor %}                                        │
└──────────────────────────────────────────────────────┘
```

### Table with section breaks
```
{%tr for section in doc.section_line_ids %}
  {% if section.display_type == 'line_section' %}
    {{ section.name }}  (merged across all columns, bold)
  {% elif section.display_type == 'line_note' %}
    {{ section.name }}  (merged across all columns, italic)
  {% else %}
    {{ section.product_id.name }} │ {{ section.product_uom_qty }} │ ...
  {% endif %}
{%tr endfor %}
```

### Nested data (e.g., grouped lines)
```
{%tr for group in grouped_lines %}
  Group: {{ group.name }}
  {%tr for line in group.lines %}
    {{ line.product_id.name }} │ {{ monetary(line.price_subtotal) }}
  {%tr endfor %}
{%tr endfor %}
```

## Advanced Patterns

### Accessing environment
```
{% set company = env.company %}
{{ company.name }}
{{ company.street }}
```

### Custom computations in template
```
{% set total_qty = doc.order_line|sum(attribute='product_uom_qty') %}
Total quantity: {{ format_lang(total_qty, digits=0) }}
```

### Multiple currencies
```
{% if doc.currency_id.name != 'EUR' %}
  {{ monetary(doc.amount_total, doc.currency_id) }}
  (≈ {{ monetary(doc.amount_total_in_eur, env.ref('base.EUR')) }})
{% endif %}
```
