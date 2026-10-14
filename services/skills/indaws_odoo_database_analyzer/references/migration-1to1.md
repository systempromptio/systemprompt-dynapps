# Migration Strategy: 1:1 (Direct Migration)

> **Reglas de clasificacion:** Este documento aplica las reglas de clasificacion (Arbol 3: Migracion 1:1). Para la fuente de verdad sobre clasificaciones, Golden Rules y umbrales, delega en la skill `odoo-classification-guide`.

## Overview

This strategy migrates each existing module directly to the target version without refactoring or replacing functionality. The goal is minimal changes while ensuring compatibility with the new Odoo version.

## Core Principles

### Preserve Existing Structure
- Migrate each module as-is, maintaining current architecture
- Focus on API compatibility updates only
- Minimal intervention approach

### Solution Classification (Priority Order)

#### 1. STANDARD (Only if absolutely clear)
Use Standard ONLY when:
- The custom module **exactly** replicates functionality that is **100% native** in target version
- No business logic modifications exist
- Simple field additions or views that are now standard
- **Criteria**: Less than 10 lines of custom Python logic

#### 2. CUSTOM (Default)
Use Custom for:
- All modules with business logic
- Any module with Python code beyond simple computed fields
- Modules with API integrations (migrate the integration, don't replace with Core)
- Modules with complex views or reports
- **This is the DEFAULT classification for 1:1 migration**

## FORBIDDEN Classifications

**DO NOT USE in 1:1 Migration:**
- ❌ `🟣 Core` - Never recommend Indaws Core modules
- ❌ `🟡 Studio` - Never recommend Odoo Studio

## Hour Calculation Rules

### Standard Modules
| Complexity | Config Hours | QA Hours |
|------------|--------------|----------|
| Simple native replacement | 1h | 1h |
| Native with minor config | 2h | 2h |

### Custom Modules (Code Migration)

Consult `odoo-effort-estimator` skill for base hours, then apply:

Base hours come from the component matrix (LOC tiers per component) or, without an inventory,
from the coarse per-module tier table. Both live in `odoo-effort-estimator`.

### Code Analysis Adjustments

Apply these multipliers to Custom modules:

Code quality adjustments (deprecated APIs, security, performance, OWL, poor quality) live in the
**`odoo-effort-estimator` skill**. Single source — do not duplicate the percentages here.

## Analysis Workflow

### Step 1: Module Inventory
1. Scan all `__manifest__.py` files
2. Count lines of code per module
3. Identify Python, XML, JS/OWL components

### Step 2: Classify Each Module
For each module:
1. Check if functionality is **exactly** native in target version → Standard
2. Otherwise → Custom (default)

### Step 3: Calculate Hours
1. Apply base hours from estimation matrix
2. Run code analysis for deprecated patterns
3. Apply adjustments based on findings
4. Apply risk contingency

## Output Table Format

| Source Module | Functionality | Solution | Strategy | Total (H) |
|---------------|---------------|----------|----------|-----------|
| `module_name` | Description | `🟢 Standard` or `🔴 Custom` | Migration approach | Integer |

### Strategy Column Content

**For Standard:**
```
Description: [Brief explanation]<br>
Hour breakdown: Setup: Xh - QA: Xh
```

**For Custom:**
```
Description: Direct code migration to vXX<br>
API changes: [List deprecated APIs found]<br>
Hour breakdown: Dev: Xh - QA: Xh
```

## Example Classifications

### Correct 1:1 Classification
| Module | Analysis | Result |
|--------|----------|--------|
| `custom_invoice_report` | QWeb report with custom layout | `🔴 Custom` |
| `custom_product_fields` | 3 computed fields, 1 view | `🔴 Custom` |
| `simple_activity_log` | Reimplements mail.thread (native v15+) | `🟢 Standard` |
| `api_connector_erp` | REST API integration | `🔴 Custom` |

### WRONG 1:1 Classification (Never do this)
| Module | Wrong Analysis | Correct |
|--------|----------------|---------|
| `custom_import` | "Use indaws_data_import" | `🔴 Custom` |
| `custom_fields` | "Simple, use Studio" | `🔴 Custom` |

## Special Cases

### API Integrations
- **Always** migrate as Custom
- DO NOT recommend Core modules (indaws_data_import/export)
- Estimate based on CODE_MIGRATION + DATA_IMPORT_EXPORT hours

### Dashboards/KPIs
- **Always** migrate as Custom
- DO NOT recommend Spreadsheets
- Estimate based on original complexity

### JavaScript/OWL Components
- High risk for v16+ migrations (OWL framework change). Measured: `@odoo/owl` imports in
  `web/static/src` go from **0 in v15 to 261 in v16**. OWL itself already existed in v15 (OWL 1,
  137 files mention it) — what breaks at v16 is **OWL 1 → OWL 2 plus the import path**, not the
  arrival of OWL.
- **El acantilado es solo v15→v16, y conviene decirlo al cotizar.** Re-medido 2026-08-17, misma
  cuenta, los cinco majors: **v15 0 · v16 261 · v17 287 · v18 309 · v19 341**. De v16 en adelante la
  curva es crecimiento vegetativo, no ruptura: **no apliques el +50% de OWL a un v17→v18 o v18→v19**
  por el mero hecho de que el módulo lleve JS. El salto que lo justifica ya ocurrió.
- Apply the OWL/JS adjustment from the `odoo-effort-estimator` skill
- Consider separate line item for JS migration effort
