# Activation Output

**When this skill activates, display this message to the user:**

```
🔍 ODOO MIGRATION ANALYZER ACTIVATED

I'll help you analyze your Odoo codebase for migration planning.
I need to gather some information to generate an accurate estimate.

Please provide the following:

🏢 CLIENT / PROJECT
   • Client or project name: [name]

📁 MODULE PATHS
   Path(s) to client's custom/legacy modules
   Example: ~/odoo/addons-client, ~/custom_modules

📁 INDAWS CORE PATHS (optional for 1:1 migration)
   Path(s) to Indaws Core source code
   Example: /opt/indaws/core

📌 VERSION INFORMATION
   • Current Odoo version: [14/15/16/17/18/19]
   • Target Odoo version: [15/16/17/18/19]

📊 PROJECT SCOPE
   • Project size: [Small/Medium/Large]
     - Small: <10 modules, simple customizations
     - Medium: 10-30 modules, moderate complexity
     - Large: >30 modules, complex integrations

🔄 MIGRATION TYPE
   1. Direct 1:1 - Migrate each module as-is (no Core/Studio)
   2. Refactoring - Optimize with Core/Studio/Standard (recommended)
   3. Refactoring without Studio - Optimize without Studio license

💾 DATABASE MIGRATION (optional)
   • Include database upgrade estimation? [Yes/No]
   • If yes, database size in GB: [number]

🧪 FUNCTIONAL TESTING (optional)
   • Include functional testing estimation? [Yes/No]
     (Business consultant validates migrated flows end-to-end)

🔬 UNIT TESTING (optional)
   • Include unit test development estimation? [Yes/No]
     (AI-assisted test generation for Custom modules, estimate effort to reach 90%+)

📄 OUTPUT
   • Report output path: [full path with filename.md]
   • Report language: [English/Spanish/Catalan]

🔬 ANALYSIS DEPTH
   • Enable deep code analysis? [Yes/No] (recommended: Yes)
```
