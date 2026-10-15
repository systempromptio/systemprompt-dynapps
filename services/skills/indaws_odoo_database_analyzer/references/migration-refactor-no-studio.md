# Migration Strategy: Refactoring Without Studio

> **Reglas de clasificacion:** Este documento aplica las reglas de clasificacion (Arbol 4: Migracion sin Studio). Para la fuente de verdad sobre clasificaciones, Golden Rules y umbrales, delega en la skill `odoo-classification-guide`.

## Overview

This strategy is identical to the full refactoring approach but **excludes Odoo Studio** as a solution option. Use this when:
- Client doesn't have Studio license
- Client prefers code-based solutions for maintainability
- Environment restrictions prevent Studio usage

## Core Philosophy

### Criterion: governed code (Studio excluded here)
The goal is **not** to avoid code but to keep logic as **governed** code (reviewed, tested, migratable). Ladder for this strategy (Studio excluded): `Standard > Core > Custom`. The classification criterion lives in `odoo-classification-guide` (single source of truth) — do not re-derive it here.

**Note:** Studio is NOT an option in this strategy (by client/environment constraint).

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

### LEVEL 2 - CORE

Search in Indaws Core repositories:
- Does a similar module exist that covers this functionality?
- → **Core** (Cost: installation + setup)

### LEVEL 3 - CUSTOM (Absorbs Studio cases)

**Important:** What would be Studio in full refactoring becomes Custom here.

Is it any customization requiring development?
- → **Custom** (Cost: code development)

**Custom includes (that would be Studio):**
- Adding custom fields → Custom module with fields
- Simple computed fields → Custom module
- View modifications → Custom XML views
- Automated actions → Custom server actions
- Basic approval workflows → Custom workflow module

## FORBIDDEN Classifications

**DO NOT USE in this strategy:**
- ❌ `🟡 Studio` - Never recommend Odoo Studio

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

### Custom Modules (Including former Studio cases)

**Simple customizations (former Studio):**
| Type | Dev Hours | QA Hours |
|------|-----------|----------|
| Field additions (1-5 fields) | 2h | 1h |
| Simple computed fields | 2h | 1h |
| View modifications | 2-3h | 1h |
| Automated actions | 3-4h | 2h |

**Complex customizations:**
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
3. **Everything else** → Custom (no Studio option)

### Step 4: Code Analysis (if enabled)
1. Scan for deprecated APIs
2. Check for security issues
3. Identify performance problems
4. Adjust hours based on findings

### Step 5: Calculate Hours
1. Apply base hours from matrix
2. Apply code quality adjustments
3. Apply risk contingency

## Output Table Format

| Source Module | Functionality | Solution | Strategy | Total (H) |
|---------------|---------------|----------|----------|-----------|
| `module_name` | Description | `🟢 Standard` / `🟣 Core` / `🔴 Custom` | Details | Integer |

**Note:** Only 3 solution types available (no 🟡 Studio).

### Strategy Column Content

**For Standard:**
```
Description: Now native in vXX<br>
Hour breakdown: QA: Xh
```

**For Core:**
```
Description: [Strategy]<br>
Core modules: [module_names]<br>
Hour breakdown: Setup: Xh - Dev: Xh - QA: Xh
```

**For Custom:**
```
Description: [Development approach]<br>
Hour breakdown: Dev: Xh - QA: Xh
```

## Example Classifications

| Module | Full Refactor Result | No-Studio Result | Hours Diff |
|--------|---------------------|------------------|------------|
| `custom_product_fields` | `🟡 Studio` (3h) | `🔴 Custom` (4h) | +1h |
| `custom_approval` | `🟡 Studio` (4h) | `🔴 Custom` (6h) | +2h |
| `custom_import_excel` | `🟣 Core` (7h) | `🟣 Core` (7h) | 0h |
| `custom_dashboard` | `🟢 Standard` (6h) | `🟢 Standard` (6h) | 0h |

## Budget Impact

Inform the client that without Studio:
- Trivial customizations Studio would cover no-code may cost somewhat more (a small governed module instead of a Studio field); with AI-assisted development this gap is much smaller than it used to be.
- Everything ships as governed customizations: code-based, testable and migratable.
- Changes go through development (no visual self-service configuration).

Calculate and report the difference in the Executive Summary.
