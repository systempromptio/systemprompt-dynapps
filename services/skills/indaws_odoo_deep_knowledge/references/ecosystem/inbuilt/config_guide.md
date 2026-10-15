# inBuilt — Configuration Recipes

## IBCG1: Enable Measurement System
1. Settings > Sales > inBuilt > Enable Measurements
2. Create/edit `indaws.sale.order.type` → check "Use Measurements"
3. On SO with that order type: SOL shows "Measurements" tab
4. Add measurements (units × L × W × H) → `product_uom_qty` auto-computed

## IBCG2: Configure FIEBDC3 Import
1. Settings > Sales > inBuilt > Enable FIEBDC3 Import
2. Configure `fiebdc3.percentage.map` records (Settings > Technical > FIEBDC3 Maps)
3. Map BC3 UOMs: Settings > Technical > UoM, add BC3 unit names
4. On SO: button "Import BC3" → upload file → creates `fiebdc3.job`
5. Wait for daily cron OR manually trigger from job form

## IBCG3: Setup Project Certification
1. Settings > Project > inBuilt > Enable Certifications
2. Company settings: set `certification_product_id` (service product for invoice lines)
3. For origin billing: also set `certification_previous_billing_product_id`
4. On project form: check "Project Certification", select billing type (origin/current)
5. Optionally set retention percentage

## IBCG4: Create First Certification
1. Navigate to project > Certifications tab > Create
2. Click "Add Lines" → wizard shows available SO lines
3. Select lines (parent selection auto-includes children)
4. Set `quantity_certified` per line (or use measurement if enabled)
5. Verify `certified_amount` total
6. Click "Create Invoice" → select presentation mode
7. Review and post invoice

## IBCG5: Configure Retention
1. On project form: set `retention_percent` (e.g., 5%)
2. Company settings: configure retention account
3. When creating invoice from certification: retention move auto-created
4. Final certification: set retention to 0% → creates release invoice

## IBCG6: Project Margin Analysis Setup
1. Install `inbuilt_sale_margin_project` (auto-installs with margin + project)
2. Company settings: set `sale_indirect_purchase_price_percent` (overhead %)
3. SO lines: configure cost categories (labor, materials, machinery, subcontract)
4. Project form shows: budgeted vs real costs/revenues/margins
5. For stock costs: install bridge modules (auto-install)

## IBCG7: FIEBDC3 Percentage Map Configuration
1. Settings > Technical > FIEBDC3 > Percentage Maps
2. Each record: `code_pattern` (e.g., `%GG`), `python_code`, `sequence`
3. Python code returns: `result = yield_value` or `result = {'yield': x, 'base': y}`
4. Available vars: `code`, `bc3`, `siblings`, `base`, `material_base`
5. Lower sequence = higher priority (first match wins)

## IBCG8: Hierarchical Task Creation from SO
1. On SO: click "Create Project"
2. Wizard options:
   - **task_chapter**: one task per section line
   - **task_part**: one task per product line
   - **task_sub_parts**: one task per child line
3. Tasks created with hierarchy matching SO line hierarchy
4. `parent_sol_id` links each task back to its source SOL

## IBCG9: Cost Category Setup for Construction
1. Settings > Sales > Margin > Cost Categories
2. Typical construction categories:
   - Labor (Mano de obra)
   - Materials (Materiales)
   - Machinery (Maquinaria)
   - Subcontract (Subcontratación)
   - Overhead (Gastos generales)
3. Each SOL can have cost breakdown by category
4. FIEBDC3 import auto-maps to these categories

## IBCG10: Multi-Certification Workflow (Monthly)
1. Month 1: Create cert → certify 20% → invoice → retention withheld
2. Month 2: Create cert → certify 50% (cumulative) → invoice adjusts for month 1
3. Month N: Create cert → certify 100% → final invoice → release retention
4. Origin mode: each invoice shows total certified minus previous
5. Current mode: each invoice shows only the delta
