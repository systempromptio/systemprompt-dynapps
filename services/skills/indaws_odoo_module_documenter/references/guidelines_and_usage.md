# Guidelines, Audience, and Usage Examples

## Target Audience

### Technical Documentation

The technical documentation serves:
- **Developers** - Need detailed technical specifications and code structure
- **Technical Consultants** - Need implementation details and technical context
- **Junior Developers** - Need a functional and technical map before diving into the code

### End-User Documentation

The end-user documentation serves:
- **Business Users** - Need to understand what they can do with the module
- **Administrators** - Need configuration and setup instructions
- **Functional Consultants** - Need business-oriented feature descriptions

## How to Provide the Module Code

### Option 1: Provide Module Path Directly

The user can specify the path when invoking the skill:
- "Document the Odoo module at `/path/to/my_custom_module`"
- "Generate documentation for the module in `~/projects/odoo/addons/purchase_discount`"

### Option 2: Current Working Directory

If the user is already in the module directory, they can simply ask:
- "Document this Odoo module"
- "Generate functional documentation for this module"

In this case, use the current working directory as the module path.

### Option 3: Ask for Path

If no path is provided and we're not in a module directory, ask the user:
"Please provide the path to the Odoo module you want to document (the directory containing `__manifest__.py`)."

### Validation

Before proceeding, always verify that the provided path contains a valid Odoo module:
- Check for `__manifest__.py` or `__openerp__.py` file
- If not found, inform the user and ask for the correct path

## Key Guidelines

### CRITICAL: View Inheritance and Translations

**Always Follow Inheritance Chains (MANDATORY):**
- When views use `inherit_id`, trace the chain to determine the REAL location in the UI
- Example: If `res_config_settings.xml` has `<field name="inherit_id" ref="project.res_config_settings_view_form"/>`, those settings are under **Proyectos > Configuracion > Ajustes** -- NOT under Inventario or any other app
- NEVER assume or guess where a view appears -- always verify by reading the parent view definition
- For settings views (`res.config.settings`), the `inherit_id` reference tells you which app's settings page hosts the configuration
- If the parent module is accessible, read its views to confirm the menu path

**Always Use Translated Names (MANDATORY):**
- Before writing ANY field name, menu name, button label, or section title in the documentation, check the `i18n/{lang}.po` file
- Use `msgstr` (translated) values, not `msgid` (English source) values
- This applies to: field labels, menu items, view titles, group names, button text, setting labels, tab names
- Also check dependency module translation files for inherited fields
- Example: If `es.po` has `msgid "Transfers"` / `msgstr "Transferencias"`, use "Transferencias" in Spanish documentation

### Common to Both Types

**Use User-Centric Language:**
- `res.partner` -> "Contact" or "Partner"
- `sale.order` -> "Sales Order" or "Quotation"
- `Many2one` -> "Link to [Model]" or "Reference"
- `Char` -> "Text field"

**Focus on Business Value:**
- Bad: "Adds a `discount` field"
- Good: "Adds a **Discount (%)** field to purchase order lines, allowing buyers to record negotiated price reductions"

### Technical Documentation Specific

**Include Technical Context:**
- Mention model names, field types, and computed fields
- Reference technical implementations when relevant ("This is a computed field based on...")
- Include all 6 sections with detailed technical breakdowns
- Explain constraints and validations with technical terminology

**Infer Logic from Code:**
- If a field has `compute=` -> Explain it's automatically calculated and reference the computation logic
- If there's `@api.onchange` -> Describe the automatic behavior with technical context
- If there's a constraint -> Explain the business rule with technical implementation details

### End-User Documentation Specific

**Simplify Technical Concepts:**
- Avoid: "computed field", "inherited model", "constraint", "Many2one relation"
- Use: "automatically calculated", "extended functionality", "validation rule", "link to another record"
- Focus on WHAT users see and DO, not HOW the system does it internally
- Generate only sections 1-4 (skip technical breakdown sections 5-6)

**Action-Oriented Language:**
- "You can now..." instead of "This module adds a field that allows..."
- "The system automatically..." instead of "A computed field calculates..."
- "Users will see..." instead of "The view has been modified to include..."

## Usage Examples

### Example 1: Document a Module with Explicit Path

**User:** "Document the Odoo module at `~/odoo/custom_addons/purchase_discount`"

**Workflow:**
1. Validate path has `__manifest__.py`
2. Analyze module structure
3. Generate documentation -> `purchase_discount_documentation.md`

### Example 2: Document Current Directory Module

**User is in:** `~/odoo/custom_addons/sale_workflow/`
**User:** "Generate documentation for this Odoo module"

**Workflow:**
1. Use current working directory
2. Validate it's an Odoo module
3. Generate documentation -> `sale_workflow_documentation.md`

### Example 3: User Doesn't Provide Path

**User:** "I need to document an Odoo custom module"
**Assistant:** "Please provide the path to the module you want to document (the directory containing `__manifest__.py`)."

### Example 4: Relative Path

**User is in:** `~/projects/`
**User:** "Document the module at `./odoo-addons/hr_attendance_biometric`"

**Workflow:**
1. Resolve relative path to absolute
2. Validate module
3. Generate documentation

## Notes

- This skill works with local module code only (no Odoo instance connection required)
- Supports Odoo versions 14.0 through 19.0
- Can handle both new modules and inherited/extended modules
- Works best with well-structured modules following Odoo conventions
- Accepts absolute paths, relative paths, and tilde-expanded paths (~)
- Always validates that the path contains a valid Odoo module before proceeding
