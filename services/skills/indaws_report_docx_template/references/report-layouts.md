# Report Layout Patterns for python-docx

Patterns and code snippets for generating `.docx` templates with `python-docx` that are compatible with `docxtpl` rendering in `indaws_report_docx`.

## Color Handling (CRITICAL)

**NEVER assume or invent colors.** Always run `scripts/extract_colors.py` on the mockup image first.

### Default palette (when no colors detected)

Most business documents are black & white with borders. Use this as default:

```python
# Default palette — no colored backgrounds
BLACK_RGB = (0, 0, 0)
GRAY_RGB = (100, 100, 100)       # Secondary text
BORDER_GRAY = 'C0C0C0'           # Table borders
LIGHT_GRAY = 'F2F2F2'            # Subtle alternating rows (only if seen in original)
```

### When to add color constants

Only add background color constants when `extract_colors.py` detects color bands:

```python
# ONLY if extract_colors.py reports color bands:
ACCENT = '008BD6'                 # Paste exact hex from script output
ACCENT_RGB = (0, 139, 214)       # Same
```

### Common mistake: logo color ≠ table header color

A document with a blue logo does NOT mean table headers have blue backgrounds. The extraction script distinguishes between:
- **Color bands** (horizontal fills spanning the page) → actual backgrounds
- **Chromatic pixels** (scattered colors) → usually logo/brand elements, NOT backgrounds

If only chromatic pixels are found and no bands: **use white backgrounds everywhere**.

## Page Setup

### A4 Portrait (most common)
```python
from docx import Document
from docx.shared import Mm, Pt, Cm, Inches, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.enum.table import WD_TABLE_ALIGNMENT
from docx.oxml.ns import qn

doc = Document()
section = doc.sections[0]
section.page_width = Mm(210)
section.page_height = Mm(297)
section.left_margin = Mm(20)
section.right_margin = Mm(20)
section.top_margin = Mm(15)
section.bottom_margin = Mm(15)
```

### A4 Landscape
```python
section.page_width = Mm(297)
section.page_height = Mm(210)
section.orientation = 1  # WD_ORIENT.LANDSCAPE
```

### Custom Label (e.g., 100x150mm)
```python
section.page_width = Mm(100)
section.page_height = Mm(150)
section.left_margin = Mm(5)
section.right_margin = Mm(5)
section.top_margin = Mm(5)
section.bottom_margin = Mm(5)
```

## Typography Helpers

```python
def set_run_font(run, name='Arial', size=10, bold=False, italic=False, color=None):
    """Configure a text run's font properties."""
    run.font.name = name
    run.font.size = Pt(size)
    run.bold = bold
    run.italic = italic
    if color:
        run.font.color.rgb = RGBColor(*color)

def add_styled_paragraph(doc, text, font_name='Arial', font_size=10,
                         bold=False, italic=False, color=None,
                         alignment=WD_ALIGN_PARAGRAPH.LEFT,
                         space_before=0, space_after=0):
    """Add a paragraph with full styling."""
    p = doc.add_paragraph()
    p.alignment = alignment
    p.paragraph_format.space_before = Pt(space_before)
    p.paragraph_format.space_after = Pt(space_after)
    run = p.add_run(text)
    set_run_font(run, font_name, font_size, bold, italic, color)
    return p
```

## Common Layout: Invoice / Sales Order

### Structure Overview
```
┌─────────────────────────────────────────────────────┐
│  [LOGO]              COMPANY NAME                    │ ← Header area
│                      Company Address                 │
│                      Phone / Email / VAT             │
├─────────────────────────────────────────────────────┤
│  DOCUMENT TITLE (e.g., "Presupuesto S00042")        │ ← Title
├─────────────────────────────────────────────────────┤
│  Customer:           │  Document info:               │ ← Info block
│  Name                │  Date: xx/xx/xxxx             │   (2-column table)
│  Address             │  Ref: xxx                     │
│  VAT: xxx            │  Payment: xxx                 │
├─────────────────────────────────────────────────────┤
│  # │ Description │ Qty │ Unit │ Price │ Subtotal    │ ← Lines table
│  1 │ Product A   │ 10  │ Uds  │ 50.00 │ 500.00     │
│  2 │ Product B   │  5  │ Uds  │ 30.00 │ 150.00     │
├─────────────────────────────────────────────────────┤
│                              Subtotal:  650.00 €    │ ← Totals
│                              IVA 21%:   136.50 €    │
│                              TOTAL:     786.50 €    │
├─────────────────────────────────────────────────────┤
│  Notes / Terms and conditions                        │ ← Footer area
│  Payment instructions                                │
└─────────────────────────────────────────────────────┘
```

### Header Area (Logo + Company Info)

```python
def create_header(doc):
    """Create document header with logo and company info."""
    table = doc.add_table(rows=1, cols=2)
    table.autofit = True
    table.alignment = WD_TABLE_ALIGNMENT.CENTER

    # Logo cell (left)
    cell_logo = table.cell(0, 0)
    cell_logo.width = Mm(50)
    p_logo = cell_logo.paragraphs[0]
    run = p_logo.add_run('{{ Image(doc.company_id.logo, width=40) }}')
    set_run_font(run, size=10)

    # Company info cell (right)
    cell_info = table.cell(0, 1)
    cell_info.width = Mm(120)
    p = cell_info.paragraphs[0]
    p.alignment = WD_ALIGN_PARAGRAPH.RIGHT

    # Company name
    run = p.add_run('{{ doc.company_id.name }}')
    set_run_font(run, size=14, bold=True)
    p.add_run('\n')

    # Address
    for field in [
        '{{ doc.company_id.street }}',
        '{% if doc.company_id.street2 %}{{ doc.company_id.street2 }}{% endif %}',
        '{{ doc.company_id.zip }} {{ doc.company_id.city }}',
        '{{ doc.company_id.country_id.name }}',
    ]:
        run = p.add_run(field + '\n')
        set_run_font(run, size=9, color=(100, 100, 100))

    # Contact
    run = p.add_run('{{ doc.company_id.phone or "" }}  {{ doc.company_id.email or "" }}')
    set_run_font(run, size=9, color=(100, 100, 100))

    # Remove table borders (invisible layout table)
    remove_table_borders(table)
    return table
```

### Document Title

```python
def create_title(doc, title_template='{{ doc.name }}', subtitle=''):
    """Create document title section."""
    p = doc.add_paragraph()
    p.alignment = WD_ALIGN_PARAGRAPH.LEFT
    p.paragraph_format.space_before = Pt(20)
    p.paragraph_format.space_after = Pt(10)

    run = p.add_run(title_template)
    set_run_font(run, size=18, bold=True, color=(44, 62, 80))

    if subtitle:
        p.add_run('\n')
        run = p.add_run(subtitle)
        set_run_font(run, size=11, color=(127, 140, 141))
```

### Info Block (Customer + Document Details)

```python
def create_info_block(doc):
    """Create 2-column info block with customer and document data."""
    table = doc.add_table(rows=1, cols=2)
    table.autofit = True
    remove_table_borders(table)

    # Left: Customer info
    cell_left = table.cell(0, 0)
    p = cell_left.paragraphs[0]
    run = p.add_run('Cliente\n')
    set_run_font(run, size=9, bold=True, color=(127, 140, 141))
    for field in [
        '{{ doc.partner_id.name }}',
        '{% if doc.partner_id.street %}{{ doc.partner_id.street }}{% endif %}',
        '{{ doc.partner_id.zip or "" }} {{ doc.partner_id.city or "" }}',
        '{% if doc.partner_id.vat %}NIF: {{ doc.partner_id.vat }}{% endif %}',
    ]:
        run = p.add_run(field + '\n')
        set_run_font(run, size=10)

    # Right: Document info
    cell_right = table.cell(0, 1)
    p = cell_right.paragraphs[0]
    p.alignment = WD_ALIGN_PARAGRAPH.RIGHT

    info_fields = [
        ('Fecha:', '{{ format_date(doc.date_order) }}'),
        ('Referencia:', '{{ doc.name }}'),
        ('Vendedor:', '{{ doc.user_id.name or "" }}'),
        ('Pago:', '{{ doc.payment_term_id.name or "" }}'),
    ]
    for label, value in info_fields:
        run = p.add_run(f'{label} ')
        set_run_font(run, size=9, bold=True, color=(127, 140, 141))
        run = p.add_run(f'{value}\n')
        set_run_font(run, size=10)
```

### Lines Table

```python
def create_lines_table(doc):
    """Create product lines table with docxtpl loop."""
    # Define columns: (header_text, width_mm, alignment)
    columns = [
        ('#', 10, WD_ALIGN_PARAGRAPH.CENTER),
        ('Descripcion', 70, WD_ALIGN_PARAGRAPH.LEFT),
        ('Cantidad', 20, WD_ALIGN_PARAGRAPH.RIGHT),
        ('Precio Ud.', 25, WD_ALIGN_PARAGRAPH.RIGHT),
        ('Dto.', 15, WD_ALIGN_PARAGRAPH.RIGHT),
        ('Subtotal', 30, WD_ALIGN_PARAGRAPH.RIGHT),
    ]

    table = doc.add_table(rows=2, cols=len(columns))
    table.alignment = WD_TABLE_ALIGNMENT.CENTER
    table.autofit = False

    # Header row
    header_row = table.rows[0]
    for i, (text, width, align) in enumerate(columns):
        cell = header_row.cells[i]
        cell.width = Mm(width)
        p = cell.paragraphs[0]
        p.alignment = align
        run = p.add_run(text)
        set_run_font(run, size=9, bold=True, color=(255, 255, 255))
        # Header background color
        set_cell_shading(cell, '2C3E50')

    # Data row (with docxtpl loop)
    data_row = table.rows[1]
    loop_data = [
        ('{%tr for line in doc.order_line %}{{ loop.index }}', WD_ALIGN_PARAGRAPH.CENTER),
        ('{{ line.product_id.name }}', WD_ALIGN_PARAGRAPH.LEFT),
        ('{{ format_lang(line.product_uom_qty, digits=0) }} {{ line.product_uom.name }}', WD_ALIGN_PARAGRAPH.RIGHT),
        ('{{ format_lang(line.price_unit, digits=2) }}', WD_ALIGN_PARAGRAPH.RIGHT),
        ('{% if line.discount %}{{ format_lang(line.discount, digits=0) }}%{% endif %}', WD_ALIGN_PARAGRAPH.RIGHT),
        ('{{ monetary(line.price_subtotal) }}{%tr endfor %}', WD_ALIGN_PARAGRAPH.RIGHT),
    ]

    for i, (text, align) in enumerate(loop_data):
        cell = data_row.cells[i]
        p = cell.paragraphs[0]
        p.alignment = align
        run = p.add_run(text)
        set_run_font(run, size=9)

    style_table_borders(table)
    return table
```

### Totals Section

```python
def create_totals(doc):
    """Create totals section aligned to the right."""
    table = doc.add_table(rows=3, cols=2)
    table.alignment = WD_TABLE_ALIGNMENT.RIGHT
    table.autofit = False

    totals = [
        ('Subtotal:', '{{ monetary(doc.amount_untaxed) }}'),
        ('Impuestos:', '{{ monetary(doc.amount_tax) }}'),
        ('TOTAL:', '{{ monetary(doc.amount_total) }}'),
    ]

    for i, (label, value) in enumerate(totals):
        # Label cell
        cell_label = table.cell(i, 0)
        cell_label.width = Mm(30)
        p = cell_label.paragraphs[0]
        p.alignment = WD_ALIGN_PARAGRAPH.RIGHT
        run = p.add_run(label)
        is_total = (i == len(totals) - 1)
        set_run_font(run, size=10 if not is_total else 12,
                     bold=True, color=(44, 62, 80))

        # Value cell
        cell_value = table.cell(i, 1)
        cell_value.width = Mm(35)
        p = cell_value.paragraphs[0]
        p.alignment = WD_ALIGN_PARAGRAPH.RIGHT
        run = p.add_run(value)
        set_run_font(run, size=10 if not is_total else 12, bold=is_total)

    remove_table_borders(table)
    return table
```

### Notes/Footer Area

```python
def create_notes_section(doc):
    """Create notes section with HTML content support."""
    # Separator line
    p = doc.add_paragraph()
    p.paragraph_format.space_before = Pt(20)

    # Notes (conditional)
    p = doc.add_paragraph()
    run = p.add_run('{% if doc.note %}')
    set_run_font(run, size=1, color=(255, 255, 255))  # Hidden tag

    p = doc.add_paragraph()
    run = p.add_run('Notas:')
    set_run_font(run, size=9, bold=True, color=(127, 140, 141))

    p = doc.add_paragraph()
    run = p.add_run('{{ r Html2Docx(doc.note) }}')
    set_run_font(run, size=9)

    p = doc.add_paragraph()
    run = p.add_run('{% endif %}')
    set_run_font(run, size=1, color=(255, 255, 255))  # Hidden tag
```

## Utility Functions

### Remove Table Borders (Layout Table)
```python
def remove_table_borders(table):
    """Remove all borders from a table (invisible layout table)."""
    tbl = table._tbl
    tbl_pr = tbl.tblPr if tbl.tblPr is not None else tbl._add_tblPr()
    borders = '<w:tblBorders xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
    for border in ['top', 'left', 'bottom', 'right', 'insideH', 'insideV']:
        borders += f'<w:{border} w:val="none" w:sz="0" w:space="0" w:color="auto"/>'
    borders += '</w:tblBorders>'
    from docx.oxml import parse_xml
    tbl_pr.append(parse_xml(borders))
```

### Style Table Borders
```python
def style_table_borders(table, color='D5D8DC', size=4):
    """Apply consistent borders to a data table."""
    tbl = table._tbl
    tbl_pr = tbl.tblPr if tbl.tblPr is not None else tbl._add_tblPr()
    borders = f'<w:tblBorders xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
    for border in ['top', 'left', 'bottom', 'right', 'insideH', 'insideV']:
        borders += f'<w:{border} w:val="single" w:sz="{size}" w:space="0" w:color="{color}"/>'
    borders += '</w:tblBorders>'
    from docx.oxml import parse_xml
    tbl_pr.append(parse_xml(borders))
```

### Cell Shading (Background Color)
```python
def set_cell_shading(cell, color_hex):
    """Set cell background color."""
    from docx.oxml import parse_xml
    shading = parse_xml(
        f'<w:shd xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" '
        f'w:fill="{color_hex}" w:val="clear"/>'
    )
    cell._tc.get_or_add_tcPr().append(shading)
```

### Cell Vertical Alignment
```python
def set_cell_vertical_alignment(cell, align='center'):
    """Set cell vertical alignment (top, center, bottom)."""
    tc = cell._tc
    tc_pr = tc.get_or_add_tcPr()
    from docx.oxml import parse_xml
    val_align = parse_xml(
        f'<w:vAlign xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" '
        f'w:val="{align}"/>'
    )
    tc_pr.append(val_align)
```

### Cell Padding
```python
def set_cell_padding(cell, top=0, bottom=0, left=0, right=0):
    """Set cell padding in twips (1 point = 20 twips)."""
    tc = cell._tc
    tc_pr = tc.get_or_add_tcPr()
    from docx.oxml import parse_xml
    margins = (
        '<w:tcMar xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">'
        f'<w:top w:w="{top}" w:type="dxa"/>'
        f'<w:left w:w="{left}" w:type="dxa"/>'
        f'<w:bottom w:w="{bottom}" w:type="dxa"/>'
        f'<w:right w:w="{right}" w:type="dxa"/>'
        '</w:tcMar>'
    )
    tc_pr.append(parse_xml(margins))
```

### Set Table Grid (required for LibreOffice)

LibreOffice ignores `cell.width` and `autofit` — always call `set_table_grid()` with explicit widths:

```python
def set_table_grid(table, widths_mm):
    """Fix column widths via tblGrid so LibreOffice respects them."""
    from docx.oxml import OxmlElement
    from docx.oxml.ns import qn
    from docx.shared import Mm
    tbl = table._tbl
    grid = tbl.find(qn("w:tblGrid"))
    if grid is None:
        grid = OxmlElement("w:tblGrid")
        tbl.insert(0, grid)
    for child in list(grid):
        grid.remove(child)
    for w_mm in widths_mm:
        gc = OxmlElement("w:gridCol")
        gc.set(qn("w:w"), str(int(Mm(w_mm).emu / 635)))  # exact EMU→twip
        grid.append(gc)
    for row in table.rows:
        for c, w_mm in zip(row.cells, widths_mm):
            c.width = Mm(w_mm)
```

Pattern for any table with fixed columns:

```python
table = doc.add_table(rows=1, cols=3)
table.autofit = False                    # MUST be False
set_table_grid(table, [50, 100, 50])    # widths must sum to usable width
```

### Set Table Indent (position table from page margin)

Use `tblInd` to indent a table from the left margin without affecting the first column width. Useful when the logo must stay flush-left but body/footer content should have a left margin:

```python
def set_table_indent(table, indent_dxa):
    """Indent table from left margin via tblInd (does not eat column width)."""
    from docx.oxml import parse_xml
    from docx.oxml.ns import qn
    tbl_pr = table._tbl.tblPr if table._tbl.tblPr is not None else table._tbl._add_tblPr()
    for old in tbl_pr.findall(qn("w:tblInd")):
        tbl_pr.remove(old)
    tbl_pr.append(parse_xml(
        f'<w:tblInd xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"'
        f' w:w="{indent_dxa}" w:type="dxa"/>'
    ))

# Usage: indent body/footer tables 4mm from the left margin
INDENT_MM = 4
INDENT_DXA = int(Mm(INDENT_MM).emu / 635)  # exact EMU→twip (see GOTCHA #18)
set_table_indent(body_table, INDENT_DXA)
```

### Paragraph spacing — tight vs spaced

`tight()` (space 0 / line 1.0) is correct inside table cells. For standalone paragraphs between sections, use `spaced()` and `add_spacer()` to create visual breathing room:

```python
def tight(p):
    pf = p.paragraph_format
    pf.space_before = Pt(0)
    pf.space_after = Pt(0)
    pf.line_spacing = 1.0
    return p

def spaced(p, before=0, after=4, line=1.15):
    pf = p.paragraph_format
    pf.space_before = Pt(before)
    pf.space_after = Pt(after)
    pf.line_spacing = line
    return p

def add_spacer(doc, pt=6):
    """Insert an invisible paragraph of the given height to separate sections."""
    p = doc.add_paragraph()
    pf = p.paragraph_format
    pf.space_before = Pt(0)
    pf.space_after = Pt(0)
    pf.line_spacing = 1.0
    r = p.add_run("")
    r.font.size = Pt(pt)
    return p
```

Recommended gaps for label documents (100×100mm):
- Header → body: `add_spacer(doc, pt=8)`
- Body → barcode: `add_spacer(doc, pt=10)`
- Barcode → footer: `add_spacer(doc, pt=6)`

### Label/value 2-col table (aligned columns)

For shipping/picking labels where rows must align vertically (label in fixed col, value in fixed col):

```python
def body_row(table, row_idx, label, value_jinja, label_bold=True):
    """Fill one row of a 2-col label/value table."""
    from docx.oxml import parse_xml

    c = table.cell(row_idx, 0)
    set_cell_padding(c, top=0, bottom=40, left=0, right=40)
    p = tight(c.paragraphs[0])
    if label:
        r = p.add_run(label)
        r.font.size = Pt(9)
        r.bold = label_bold

    c = table.cell(row_idx, 1)
    set_cell_padding(c, top=0, bottom=40, left=0, right=0)
    p = tight(c.paragraphs[0])
    r = p.add_run(value_jinja)
    r.font.size = Pt(9)

# Setup (100mm page, 4mm margins → 92mm usable, body indented 4mm → 88mm):
body = doc.add_table(rows=6, cols=2)
body.autofit = False
remove_table_borders(body)
set_table_grid(body, [28, 60])        # labels 28mm, values 60mm
set_table_indent(body, INDENT_DXA)   # 4mm left indent

body_row(body, 0, "Entregar a:", "{{ doc.partner_id.name or '' }}")
body_row(body, 1, "Dirección:", "{{ doc.partner_id.street or '' }}")
body_row(body, 2, "", "{{ doc.partner_id.zip or '' }} {{ doc.partner_id.city or '' }}")
```

## Ship `.docx` inside an Odoo module

To auto-load the template binary when the module is installed/upgraded, put both `<data>` blocks in the **same** XML file (`data/docx_templates_data.xml`): structural records under `noupdate="1"`, the binary under `noupdate="0"` so it is re-uploaded on every module upgrade.

```xml
<odoo>
  <!-- Structural records: imported only on first install -->
  <data noupdate="1">
    <record id="template_my_report" model="docx.template">
      <field name="name">My Report</field>
      <field name="report_name">module_name.report_my_report</field>
    </record>
    <record id="action_report_my_report" model="ir.actions.report">
      <field name="name">My Report</field>
      <field name="model">my.model</field>
      <field name="report_type">docx</field>
      <field name="report_name">module_name.report_my_report</field>
      <field name="docx_output_filetype">pdf</field>
      <field name="template_docx_id" ref="template_my_report"/>
    </record>
  </data>

  <!-- Binary template: re-uploaded on every `odoo -u module_name` -->
  <data noupdate="0">
    <record id="file_template_my_report" model="docx.file.template">
      <field name="template_id" ref="template_my_report"/>
      <field name="position">body</field>
      <field name="filename">my_report.docx</field>
      <field name="file" type="base64" file="module_name/data/templates/my_report.docx"/>
    </record>
  </data>
</odoo>
```

Add the `.docx` file to `data/templates/` and declare it in the manifest `data` list. The `noupdate="0"` block re-loads the binary every `odoo -u module_name`, which is the workflow for iterating on template design.

## Delivery Slip Layout

Simpler than invoice — focuses on products and quantities:

```
┌─────────────────────────────────────────────────────┐
│  [LOGO]              DELIVERY SLIP                   │
│                      WH/OUT/00001                    │
├─────────────────────────────────────────────────────┤
│  Deliver to:         │  From:                        │
│  Customer Name       │  Warehouse Name               │
│  Address             │  Date: xx/xx/xxxx             │
│                      │  Origin: S00042               │
├─────────────────────────────────────────────────────┤
│  # │ Product    │ SKU      │ Ordered │ Delivered     │
│  1 │ Product A  │ PROD-001 │ 10      │ 10            │
│  2 │ Product B  │ PROD-002 │ 5       │ 5             │
├─────────────────────────────────────────────────────┤
│  Carrier: Carrier Name                               │
│  Tracking: TRACK123456                               │
│                                                      │
│  Signature: ____________________                     │
└─────────────────────────────────────────────────────┘
```

## Label Layout (ZPL-oriented)

Small format for product/shipping labels:

```python
doc = Document()
section = doc.sections[0]
section.page_width = Mm(100)
section.page_height = Mm(60)
section.left_margin = Mm(3)
section.right_margin = Mm(3)
section.top_margin = Mm(3)
section.bottom_margin = Mm(3)

# Barcode
p = doc.add_paragraph()
p.alignment = WD_ALIGN_PARAGRAPH.CENTER
run = p.add_run("{{ Barcode(doc.name, 'Code128', width=80, height=15) }}")

# Product name
p = doc.add_paragraph()
p.alignment = WD_ALIGN_PARAGRAPH.CENTER
run = p.add_run('{{ doc.product_id.name }}')
set_run_font(run, size=14, bold=True)

# Reference
p = doc.add_paragraph()
p.alignment = WD_ALIGN_PARAGRAPH.CENTER
run = p.add_run('{{ doc.product_id.default_code or "" }}')
set_run_font(run, size=10)
```

## Complete Generator Script Template

```python
#!/usr/bin/env python3
"""Generate DOCX template for [MODEL_NAME] report.

Usage:
    python generate_template.py

Output:
    template_[model].docx — Ready to upload to docx.template in Odoo

Requirements:
    pip install python-docx
"""
from docx import Document
from docx.shared import Mm, Pt, RGBColor
from docx.enum.text import WD_ALIGN_PARAGRAPH
from docx.enum.table import WD_TABLE_ALIGNMENT
from docx.oxml import parse_xml


# === Utility functions (copy from above) ===

def set_run_font(run, name='Arial', size=10, bold=False, italic=False, color=None):
    run.font.name = name
    run.font.size = Pt(size)
    run.bold = bold
    run.italic = italic
    if color:
        run.font.color.rgb = RGBColor(*color)

# ... (include all utility functions needed)


# === Document generation ===

def generate():
    doc = Document()

    # Page setup
    section = doc.sections[0]
    section.page_width = Mm(210)
    section.page_height = Mm(297)
    section.left_margin = Mm(20)
    section.right_margin = Mm(20)
    section.top_margin = Mm(15)
    section.bottom_margin = Mm(15)

    # Build sections
    # ... (header, title, info, lines, totals, notes)

    # Save
    output_path = 'template_output.docx'
    doc.save(output_path)
    print(f'Template saved to: {output_path}')
    return output_path


if __name__ == '__main__':
    generate()
```
