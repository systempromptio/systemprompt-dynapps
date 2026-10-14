# Migration Report Template

This is the mandatory output template. Render all content in `LANGUAGE`.
Replace all `[PLACEHOLDERS]` with actual values.

---

```markdown
### Migration Plan - [CLIENT_NAME] - Odoo [TARGET_VERSION] - [PROJECT_SIZE] Risk

**Client/Project:** [CLIENT_NAME]
**Date:** [CURRENT_DATE]
**Source Version:** [SOURCE_VERSION] → **Target Version:** [TARGET_VERSION]
**Migration Type:** [MIGRATION_TYPE]
**Project Size:** [PROJECT_SIZE] (+[XX]% safety margin)

---

#### 1. Code Refactoring (Local Analysis)

**TABLE FORMAT RULES:**

- **Solution Column — MANDATORY EMOJI PREFIX (never omit):**
  - `🟢 Standard` - Native Odoo functionality
  - `🟡 Studio` - No-code configuration (NOT for 1:1 or No_Studio)
  - `🟣 Core` - Indaws Core module (NOT for 1:1)
  - `🔴 Custom` - Code development required

  ⚠️ Every row MUST include the emoji. If the emoji is missing, the report is INVALID.

- **Strategy Column:** Contains with `<br>` line breaks:
  - **Description**: Brief strategy explanation
  - **Core modules**: (ONLY if Solution is `🟣 Core`)
  - **Hour breakdown**: Concept: Xh - separated by " - "
    - Setup: only if >0 AND Solution is Core or Standard
    - Dev: only if >0 AND Solution is Studio or Custom
    - QA: only if >0

- **Total (H) Column:** Integer only, always rounded UP

| Source Module | Functionality | Solution | Strategy | Total (H) |
|---|---|---|---|---|
| `module_name` | Description | 🟢/🟡/🟣/🔴 Solution | Strategy details | Integer |

*(List ALL modules found. Total includes [XX]% safety margin)*

#### 2. Database Migration (Odoo Upgrade)

*Include only if INCLUDE_DB_MIGRATION = Yes*

| Concept | Process Scope (Iterations) | DB Size | Base Calculation | Total with Risk (H) |
|---|---|---|---|---|
| **Upgrade Process** | [N] Iterations.<br>[Test / Rehearsal / Prod] | [GB] GB | [BASE]h | **[TOTAL]** |

#### 3. Functional Testing (Business Validation)

*Include only if INCLUDE_FUNCTIONAL_TESTING = Yes*

| Module | Complexity | Validation Scope | Total (H) |
|---|---|---|---|
| `module_name` | Low/Medium/High | Business flows to validate | Integer |

*(Total includes [XX]% safety margin)*

#### 4. Unit Test Development (Custom Modules Only)

*Include only if INCLUDE_UNIT_TESTING = Yes*

**🤖 AI-Assisted Test Generation**
Unit tests will be generated using AI-assisted tools (odoo-test-generator skill in Claude Code).
A developer reviews, validates, and adjusts the generated tests. Hours reflect this AI-assisted workflow.

**Coverage estimation method:**
```
coverage_estimate = LOC_tests / (LOC_code × 0.6)
```
*Factor 0.6: not all code requires equal test density — boilerplate, imports, and simple accessors are excluded from the effective coverage calculation.*

**Hour calculation (LOC-proportional with AI factor):**
```
Coverage Gap:      gap_LOC = LOC_code × (gap% / 100)
                   manual_hours = gap_LOC × RATE_PER_LOC
Final:             total = ceil(manual_hours × AI_factor × (1 + risk%))

RATE_PER_LOC:  Low = 0.05 h/LOC | Medium = 0.04 h/LOC | High = 0.03 h/LOC
MINIMUM:       Low = 1h         | Medium = 2h          | High = 4h
AI_factor:     `odoo-effort-estimator` skill (single source; do not reproduce here)
```

| Module | Complexity | LOC Code | LOC Tests | Current Coverage | Target | Gap | Test Migration (H) | New Tests (H) | Total (H) |
|---|---|---|---|---|---|---|---|---|---|
| `module_name` | Low/Med/High | [N] | [N] | [XX]% | 90% | [XX]% | [N] | [N] | Integer |

**Coverage Legend:**
- ✅ **Sufficient (≥80%)**: Only test migration/adaptation to new version API needed
- ⚠️ **Insufficient (<80%)**: Test migration + new tests to reach 90% coverage
- ❌ **No tests (0%)**: Full test development from scratch

*(Hours include AI assistance factors. Total includes [XX]% safety margin)*

---

### 📋 EXECUTIVE SUMMARY (PROTECTED BUDGET)

**Client/Project:** [CLIENT_NAME]

* **1. Total Code Refactoring:** [TABLE 1 SUM] h
* **2. Total DB Migration:** [TABLE 2 SUM or N/A] h
* **3. Total Functional Testing:** [TABLE 3 SUM or N/A] h
* **4. Total Unit Testing (AI-assisted):** [TABLE 4 SUM or N/A] h
* **5. TOTAL PROJECT:** [SUM 1 + 2 + 3 + 4] h
* **Global Contingency Buffer ([XX]%):** [RISK HOURS] h
  * *This buffer is included in all sections above, protecting margin against uncertainty.*
* **Coste/urgencia de no migrar (si aplica):**
  * *(Hosting Odoo.sh: fecha en que [SOURCE_VERSION] se retira de Odoo.sh — deadline duro; tras esa fecha la BD queda bloqueada permanentemente)*
  * *(Enterprise en versión fuera de las 3 más recientes: posible recargo +25% en renovación — confirmar fecha de renovación con comercial)*
  * *(Política y calendario: `references/shared/odoo-lifecycle-licensing.md` — no hardcodear fechas ni la regla)*
* **Critical Points:**
  * *(Specific technical risks detected)*
  * *(Savings achieved by using Standard/Core if applicable)*
  * *(Code quality findings if deep analysis enabled)*
  * *(Unit Testing: X modules with no existing tests (0% coverage))*
  * *(Unit Testing: Y modules below 80% coverage — require gap filling to reach 90%)*
  * *(Unit Testing: Z modules with sufficient coverage (≥80%) — only test migration needed)*
  * *(Unit Testing: Hours estimated with AI assistance — tests generated by odoo-test-generator, reviewed by developer)*
```
