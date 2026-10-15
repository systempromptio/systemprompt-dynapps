# Migration Strategy: Refactoring (Full Optimization)

> **Reglas de clasificacion:** Este documento aplica las reglas de clasificacion (Arbol 2: Migracion con refactoring). Para la fuente de verdad sobre clasificaciones, Golden Rules (Data→Core, BI→Standard) y Core Supremacy, delega en la skill `odoo-classification-guide`.

## Overview

This strategy reduces technical debt by moving each functionality to its most appropriate — and most **governed** — option: Standard Odoo, Indaws Core, Studio (only for the trivial) or a governed custom module. The classification criterion is owned by `odoo-classification-guide`; this document only applies it to the refactoring flow.

## Core Philosophy

### Criterion: governed code, not "no code"
The goal is **not** to avoid code but to keep non-trivial logic as **governed** code (reviewed, tested, migratable) rather than ungovernable Studio. The ladder `Standard > Core > Studio > Custom` and the Studio-vs-development threshold live in `odoo-classification-guide` (single source of truth) — do not re-derive them here.

### Core Supremacy
If an Indaws Core module solves the problem, it's **mandatory** to use it.

## Solution Classification (Priority Order)

### GOLDEN RULE 1 - DATA & INTEGRATIONS (CRITICAL)

**If functionality is Import/Export** (Excel, CSV, connectors, API integrations):
- → **MANDATORY INDAWS CORE**
- Assign: `indaws_data_import` OR `indaws_data_export`
- Consult `odoo-effort-estimator` skill → Category `DATA_IMPORT_EXPORT`

### GOLDEN RULE 2 - DASHBOARDS & KPIs (CRITICAL)

**If functionality is Dashboards/KPIs/Business Intelligence:**
- → **MANDATORY STANDARD WITH SPREADSHEETS**
- Use Odoo's native Spreadsheets functionality
- Consult `odoo-effort-estimator` skill → Category `SPREADSHEETS`

### LEVEL 1 - STANDARD

Check `odoo-version-api-reference` skill for version diffs:
- Is this functionality **now native** in target version?
- → **Standard** (Cost: minimal configuration)

**Examples:**
- Custom audit trails → mail.thread (native)
- Custom sequences → ir.sequence (native)
- Custom email sending → mail.template (native)

### LEVEL 2 - CORE

Search in Indaws Core repositories:
- Does a similar module exist that covers this functionality?
- → **Core** (Cost: installation + setup)

**Common Core Replacements:**
- API integrations → `indaws_data_import` / `indaws_data_export`
- Multi-company sync → `indaws_intercompany`
- Advanced reporting → `indaws_reports`

### LEVEL 3 / 4 - STUDIO vs CUSTOM

**Do NOT decide this by line count.** Whether an item stays as Studio or becomes a governed custom module is owned by `odoo-classification-guide` (single source of truth):
- **Studio** only for the trivial and low-risk (a field, a view, a one-step automation) that does not collide with existing governed code.
- **Custom (governed module)** for non-trivial logic (business rules, multi-step flows, validations, integrations) or when it collides with an existing development — extend the existing piece rather than stack Studio on top.

Classify each item via `odoo-classification-guide`, then estimate its hours below.

## Hour Calculation Rules

### Standard Modules
| Type | Config Hours | QA Hours |
|------|--------------|----------|
| Direct native replacement | 0h | 1h |
| Native with configuration | 1-2h | 1-2h |

### Core Modules
Consult `odoo-effort-estimator` skill:

| Category | Setup | Dev | QA |
|----------|-------|-----|-----|
| DATA_IMPORT_EXPORT | 2h | 3h | 2h |
| SPREADSHEETS | 2h | 2h | 2h |

### Studio Modules
| Complexity | Config Hours | QA Hours |
|------------|--------------|----------|
| Simple (<20 elements) | 1-2h | 1h |
| Medium (20-50 elements) | 2-4h | 2h |

### Custom Modules
From `odoo-effort-estimator` skill:

| Complexity | Base Hours |
|------------|------------|
Per-module hours: **`odoo-effort-estimator` skill** (component matrix with LOC tiers, or the
coarse tier table). Single source — the old CUSTOM_LIGHT/MEDIUM/HEAVY scale gave 2-4x less than
the component matrix in this same skill, so it is retired.

## Code Quality Adjustments

Apply these multipliers based on code analysis:

Code quality adjustments (deprecated APIs, security, performance, OWL, poor quality) live in the
**`odoo-effort-estimator` skill**. Single source — do not duplicate the percentages here.

Lo que sí se decide aquí son las reclasificaciones, que no son ajustes de horas:

| Finding Type | Impact | Action |
|---|---|---|
| Reimplemented standard | Positive | Reclassify to Standard |
| Complex custom logic | High | Studio → Custom reclassification |

## Analysis Workflow

### Step 1: Module Inventory
1. Scan all `__manifest__.py` files
2. Categorize by functionality type
3. Identify integration points

### Step 2: Apply Golden Rules First
1. Check for import/export functionality → Core
2. Check for dashboards/KPIs → Spreadsheets

### Step 3: Classify Remaining Modules
For each module:
1. Check release notes for native functionality → Standard
2. Search Indaws Core for alternatives → Core
3. Check if <50 lines of logic → Studio
4. Otherwise → Custom

### Step 4: Code Analysis (if enabled)
1. Scan for deprecated APIs
2. Check for security issues
3. Identify performance problems
4. Adjust classifications based on findings

### Step 5: Calculate Hours
1. Apply base hours from matrix
2. Apply code quality adjustments
3. Apply risk contingency

## Output Table Format

| Source Module | Functionality | Solution | Strategy | Total (H) |
|---------------|---------------|----------|----------|-----------|
| `module_name` | Description | `🟢 Standard` / `🟡 Studio` / `🟣 Core` / `🔴 Custom` | Details | Integer |

### Strategy Column Content

**For Standard:**
```
Description: Now native in vXX<br>
Hour breakdown: QA: Xh
```

**For Studio:**
```
Description: [Configuration approach]<br>
Hour breakdown: Dev: Xh - QA: Xh
```

**For Core:**
```
Description: [Strategy]<br>
Core modules: [module_names]<br>
Hour breakdown: Setup: Xh - Dev: Xh - QA: Xh
```

**For Custom:**
```
Description: [Migration approach]<br>
Hour breakdown: Dev: Xh - QA: Xh
```

## Example Classifications

| Module | Analysis | Result | Hours |
|--------|----------|--------|-------|
| `custom_import_excel` | Excel import functionality | `🟣 Core` (indaws_data_import) | 7h |
| `custom_dashboard` | Sales KPIs dashboard | `🟢 Standard` (Spreadsheets) | 6h |
| `custom_product_fields` | 5 simple fields | `🟡 Studio` | 3h |
| `custom_api_logic` | Complex external API with business rules | `🔴 Custom` | 12h |
| `custom_audit` | Activity logging | `🟢 Standard` (mail.thread native) | 2h |

## Savings Identification

Always highlight savings achieved by **reusing** Standard/Core instead of rebuilding:
- Calculate theoretical from-scratch Custom hours
- Compare with actual Standard/Core (or governed-module) hours
- Report savings in Executive Summary
