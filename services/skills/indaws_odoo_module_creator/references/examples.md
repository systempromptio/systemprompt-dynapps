<!-- verified: v18.0 @ 2026-06 -->
# Usage Examples

## Example 1: Simple Extension Module (Odoo 18.0)

**User:** "Create a module to add a margin field to sale orders for Odoo 18.0"

**Skill Response:**
1. Ask using AskUserQuestion:
   - Odoo version: 18.0
   - Module name: `dynapps_sale_margin`
   - Architecture: User provides clear plan
   - Destination path
2. Propose plan:
   - Module: `dynapps_sale_margin`
   - Target: Odoo 18.0
   - Extend: `sale.order` (from `sale` module)
   - Add field: `margin_percent` (Float, computed)
   - Compute method: `_compute_margin_percent` (using Odoo 18 aggregator syntax)
   - View: Extend form view to show margin
   - Version note: Use `aggregator='avg'` for any grouped fields
3. Wait for confirmation
4. Create module structure with version-specific code
5. Validate and report

## Example 2: New Model Module with Plan Mode (Odoo 19.0)

**User:** "Create a module for equipment maintenance tracking in Odoo 19, but I need help designing it"

**Skill Response:**
1. Ask using AskUserQuestion:
   - Odoo version: 19.0
   - Module name: `dynapps_maintenance_tracking`
   - Architecture: Needs help -> Select Plan Mode
   - Destination path
2. Enter Plan Mode:
   - Collaboratively design models and relationships
   - Discuss fields, workflows, security
   - Design UI/UX together
   - Exit with complete architecture
3. Propose final plan:
   - Module: `dynapps_maintenance_tracking`
   - Target: Odoo 19.0
   - New model: `maintenance.request`
   - Fields: equipment_id, request_date, technician_id, state, etc.
   - Views: Form, list, kanban, calendar (`<list>` tag — v18+, no solo v19)
   - Security: Manager and User groups
   - Version notes: `<list>` en vez de `<tree>` **desde v18** (en v17 sigue siendo `<tree>`), search views sin `<group>`
4. Wait for confirmation
5. Create complete module with Odoo 19 patterns
6. Validate and report

## Example 3: Multi-Version Consideration (Odoo 17.0)

**User:** "Create a report module for sales analysis in Odoo 17"

**Skill Response:**
1. Gather info including version: 17.0
2. Propose plan noting version-specific patterns:
   - Use `aggregator` parameter (introduced 17.2)
   - Use `_read_group()` new signature (since 16.3)
   - Implement `_compute_display_name` instead of `name_get()`
3. Reference `/odoo-version-api-reference` for 17.0-specific patterns
4. Create module with appropriate APIs for version 17.0