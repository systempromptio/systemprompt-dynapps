---
name: indaws-odoo-module-documenter
version: 1.0.3
effort: medium
model: sonnet
description: Generate comprehensive functional documentation for Odoo custom modules by analyzing source code. Supports two documentation types - Technical (all 6 sections with technical details for developers/consultants) and End-User (simplified 4-section format for business users). Use this skill when users need to document Odoo modules, create technical documentation for custom developments, generate user guides from code, or produce functional analysis of Odoo customizations. Supports Odoo versions 14.0-19.0. Output is a structured Markdown file.
---

# Odoo Module Documenter

Transforms Odoo module source code into comprehensive, audience-appropriate documentation.

- **Technical Documentation** (6 sections) - For developers and technical consultants: model structures, field definitions, code analysis.
- **End-User Documentation** (4 sections) - For business users and administrators: features and usage without technical jargon.

See `references/shared/field-mapping-syntax.md` for field types and command tuples.

## Workflow Summary

1. **Identify module** - Validate path has `__manifest__.py` (from user input, cwd, or ask)
2. **Ask the 4 questions** in Critical Rules (language, type, screenshots, output format), skipping any already answered
3. **Analyze module** - Structural inventory + translations, then read prose-level detail.
   **Inventory [MOTOR determinista, ~0 tokens LLM]:** run `../odoo-module-scanner/scripts/scan_module.py <module_path>`
   → JSON (models+types, fields, views+inherit_id, security, controllers, JS, i18n). **Fallback:** si el script
   falla o el entorno no lo ejecuta, delega en `odoo-module-scanner` o lee los ficheros a mano.
   Luego el LLM lee lo que el motor NO da y sí necesita la doc: **`i18n/{lang}.po`** (labels traducidos = `msgstr`),
   **help text / `_description`** de los `.py`, y traza las cadenas `inherit_id` para la ruta de menú REAL
   (+ módulos de `depends` accesibles). El motor da la estructura; el LLM la prosa.
4. **Apply guidelines** - Read `references/documentation_guidelines.md`, filter by doc type (`[TECHNICAL]`/`[END_USER]`/`[BOTH]` markers)
5. **Generate documentation** - All content in chosen language; technical terms kept as-is
6. **Output format** - Produce the chosen format (asked in Step 1 or defaulting to Markdown)
   **Branded output [SONNET SUBAGENT]:** For PDF/HTML/Presentation, save .md first, then launch Agent(model: sonnet, description: "brand {doc_type} documentation as {format}"):
   - **prompt:** "Convert the Markdown documentation at {md_file_path} into {format} (PDF/HTML/Presentation) using the Indaws brand guidelines. Document title: '{module_name} — {doc_type} Documentation'. Language: {language}. For PDF: use the pdf-renderer skill after applying dynapps-brand styling. For HTML: produce a standalone HTML file with embedded CSS. For Presentation: produce an HTML slide deck. Preserve all section headers, tables, and screenshot placeholders from the source Markdown."

See `references/workflow.md` for detailed step-by-step instructions.

## Critical Rules

### MANDATORY: 4 Questions Before Generating

Always ask these before starting — never skip any. **If the user already specified any answer in their initial request, use it and skip that question.**
1. **Language** - "In which language should I generate the documentation?"
2. **Type** - "Technical or End-User?"
3. **Screenshots** - "Include screenshot placeholders? (usable with odoo-screenshot-generator later)"
4. **Output format** - "What output format?" → Markdown / PDF / HTML / Presentation (see Step 6)

### MANDATORY: View Inheritance Chains

When views use `inherit_id`, trace the chain to determine the REAL UI location. Example: if a settings view inherits `project.res_config_settings_view_form`, it appears under **Proyectos > Configuracion > Ajustes** — NOT Inventario. NEVER guess menu paths.

### MANDATORY: Use Translated Names

Before writing ANY field name, menu, button label, or title in the documentation, check `i18n/{lang}.po`. Use `msgstr` values, not `msgid`. Also check dependency module translations for inherited fields.

### MANDATORY: Read Dependency Modules

Check `depends` in manifest. If dependency modules are accessible on the filesystem, READ their views and translations. Critical for inherited views. If not accessible, ask the user for the path.

## Gotchas

- End-User docs generate ONLY sections 1-4 (skip 5-6 entirely)
- Technical docs include ALL 6 sections with model names, field types, computed field logic
- Use user-centric language: `res.partner` -> "Contact", `Many2one` -> "Link to [Model]", `Char` -> "Text field"
- Focus on business value, not code changes: "Adds a **Discount (%)** field allowing buyers to record negotiated price reductions" not "Adds a `discount` field"
- End-user language: "You can now..." / "The system automatically..." (never "computed field" or "inherited model")
- The **odoo-module-scanner** skill can accelerate Step 3 by auto-scanning module components
- Works with local code only — no Odoo instance connection required
- Supports Odoo 14.0-19.0, absolute/relative/tilde paths

## Delegates to
| Skill | Suite | When | Agent()? |
|-------|-------|------|----------|
| odoo-module-scanner | common | Component inventory before documenting | Yes — Pattern A |
| dynapps-brand | common | PDF output with Indaws branding | Yes — Pattern C |
| pdf-renderer | common | PDF generation (via dynapps-brand) | Indirect |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| indaws-core-documenter | consultant | Core module documentation |
| Project documentation | consultant | Custom module docs for clients |

## References

| File | Contents |
|:-----|:---------|
| `references/workflow.md` | Detailed 6-step workflow: module identification, analysis, extraction, guideline filtering, generation, saving |
| `references/guidelines_and_usage.md` | Target audience details, module path options, key guidelines per doc type, usage examples |
| `references/documentation_guidelines.md` | Complete documentation standards, role definitions, section templates, field type translations, examples |
