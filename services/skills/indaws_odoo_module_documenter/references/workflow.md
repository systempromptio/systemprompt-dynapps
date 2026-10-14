# Documentation Workflow — Detailed Steps

## Step 1: Identify the Module and Language

### Part A: Determine the module path

Use one of these approaches:

1. **If user provided a path** in their request -> Use that path
2. **If user said "this module" or similar** -> Use current working directory
3. **If no path provided** -> Ask user for the module path

**Validate the path:**
- Check if `__manifest__.py` exists (or `__openerp__.py` for older versions)
- If not found, inform the user: "The path `[path]` doesn't appear to be a valid Odoo module (missing `__manifest__.py`). Please provide the correct path."

### Part B: Ask for output language (MANDATORY)

After validating the module path, **ALWAYS ask the user** for the documentation language:

**Prompt example:**
"In which language should I generate the documentation?"
- Spanish (Espanol)
- English
- Other (specify)

**Important:**
- This question is **MANDATORY** - never skip it
- Default to the language the user is communicating in if they don't specify
- Remember the language choice for all sections of the documentation

### Part C: Ask about documentation type (MANDATORY)

After asking for the language, **ALWAYS ask the user** what type of documentation they want:

**Prompt example:**
"What type of documentation do you want to generate?"
- **Technical** (for developers and technical consultants - includes all 6 sections with technical details)
- **End-User** (for business users and administrators - simplified language, only sections 1-4)

**Important:**
- This question is **MANDATORY** - never skip it
- Technical documentation includes all 6 sections with model names, field types, and technical analysis
- End-User documentation generates only sections 1-4 with simplified, business-oriented language

### Part D: Ask about screenshot placeholders (MANDATORY)

After asking for the documentation type, **ALWAYS ask the user** if they want to include screenshot placeholders:

**Prompt example:**
"Do you want to include placeholders for screenshots in the documentation? If yes, you can later use the `odoo-screenshot-generator` skill to automatically generate the screenshots and embed them in the document."
- Yes (include placeholders)
- No (documentation without images)

**Important:**
- This question is **MANDATORY** - never skip it
- Placeholders will be simple markers like `[SCREENSHOT:form_view]` that can be replaced later with actual images
- If user says YES, set `include_screenshots` = true and add placeholders in appropriate sections

### Part E: Ask about output format (MANDATORY)

After asking about screenshots, **ALWAYS ask the user** for the output format:

**Prompt example:**
"What output format do you want?"
- **(a) Markdown** — `.md` file (default, editable)
- **(b) PDF with Indaws branding** — Professional branded PDF
- **(c) HTML with Indaws branding** — Standalone branded HTML document
- **(d) Presentation** — Slide deck with Indaws branding (ideal for training/delivery)

**Important:**
- This question is **MANDATORY** — never skip it
- If the user already specified the format in their initial request (e.g., "genera un PDF del módulo X"), use that choice and skip this question
- Default is Markdown if user says nothing specific
- For options (b), (c), (d): the Markdown is ALWAYS generated first as source of truth, then converted

### Early Detection of Format in User Request

**CRITICAL:** Before asking the 4 questions, scan the user's initial request for format indicators. If found, pre-set `output_format` and skip Part E.

| User says | Set `output_format` to |
|-----------|----------------------|
| "genera un PDF", "quiero PDF", "en PDF" | `pdf` |
| "genera un HTML", "documento HTML" | `html` |
| "presentación", "slides", "formación" | `presentation` |
| "documenta el módulo X" (no format) | Ask in Part E |

### Variables to set for subsequent steps:
- `module_path`: The validated module path
- `output_language`: The chosen language (e.g., "Spanish", "English")
- `documentation_type`: The chosen type ("technical" or "end_user")
- `include_screenshots`: Boolean indicating if placeholders should be included
- `output_format`: The chosen format ("markdown", "pdf", "html", "presentation")

## Step 2: Analyze the Module Structure

Use the Glob and Read tools to explore the module:

1. **Read the manifest** (`__manifest__.py`):
   - Extract module name, version, category
   - Identify dependencies
   - Get module description and summary

2. **Scan module directories**:
   ```
   module_name/
   ├── __manifest__.py
   ├── models/          # Business logic and data models
   ├── views/           # User interface definitions
   ├── security/        # Access rights and groups
   ├── wizard/          # Transient models (wizards)
   ├── report/          # Reports and templates
   ├── data/            # Initial data and configurations
   ├── i18n/            # Translation files (.po)
   └── static/          # Assets (optional)
   ```

3. **Identify key files** to analyze:
   - Python models (`.py` files in `models/` and `wizard/`)
   - XML views (`.xml` files in `views/`)
   - Security definitions (`ir.model.access.csv`, `security.xml`)
   - Reports (`.xml` files in `report/`)
   - **Translation files** (`i18n/{lang}.po`) - MANDATORY for field/menu/label names

   **Component Discovery Accelerator:** The **odoo-module-scanner** skill can automatically scan and classify modules' models, fields, views, controllers, and security components. Use its output to speed up file identification in this step.

4. **Read dependency modules when accessible** (MANDATORY):
   - Check `depends` in `__manifest__.py` for all dependencies
   - If dependency modules are accessible on the filesystem (e.g., in parent directory, nearby addons paths), **READ their views and translations too**
   - This is critical for modules that inherit views from dependencies -- you must trace where settings, menus, and views actually appear in the UI
   - If a dependency module is NOT accessible, explicitly inform the user and ask for the path

## Step 3: Extract Technical Information

For each relevant file:

**From Python Models:**
- Model names (technical and inherited)
- Field definitions (name, type, string, help, required, readonly, etc.)
- Computed fields and their dependencies
- Constraints and validations
- Business methods (especially those called from UI like buttons)
- Domain filters and default values

**From XML Views:**
- Modified/new form views, tree views, kanban views
- New fields in existing views
- Buttons and actions
- Menu items
- User-facing labels (use these instead of technical field names)
- **CRITICAL -- Follow `inherit_id` chains:** When a view uses `<field name="inherit_id" ref="module.view_id"/>`, you MUST trace where that parent view lives to determine the correct menu path. For example, if a settings view inherits from `project.res_config_settings_view_form`, the settings appear under **Proyectos > Configuracion > Ajustes**, NOT under Inventario or any other app. NEVER assume the menu location -- always verify by following the inheritance chain to the original module.

**From Translation Files (`i18n/`):**
- **MANDATORY:** Before writing ANY field name, menu name, label, section title, or button text in the documentation, consult the `i18n/{lang}.po` file matching the documentation language
- The `.po` file contains `msgid` (English source) and `msgstr` (translated string) pairs
- Use the translated `msgstr` values for all user-facing names in the documentation
- Also check translation files of dependency modules for inherited field labels
- If no translation file exists for the chosen language, use the English `string` attribute from the Python field definition or XML view

**From Security:**
- New security groups
- Access rights per model
- Record rules

## Step 4: Apply Documentation Guidelines

**IMPORTANT:** Read the complete guidelines from `references/documentation_guidelines.md` before generating the documentation.

**Filtering Guidelines Based on Documentation Type:**

The guidelines file contains conditional sections marked with:
- `[BOTH]` - Applies to both technical and end-user documentation
- `[TECHNICAL]` - Only for technical documentation
- `[END_USER]` - Only for end-user documentation
- `[TECHNICAL_ONLY]` - Entire section only for technical documentation
- `[END_USER_ONLY]` - Entire section only for end-user documentation (if any)

**Apply this filtering logic:**

1. **If `documentation_type == "technical"`:**
   - Process all content marked with `[BOTH]`, `[TECHNICAL]`, and `[TECHNICAL_ONLY]`
   - IGNORE content marked with `[END_USER]` and `[END_USER_ONLY]`
   - Generate all 6 sections

2. **If `documentation_type == "end_user"`:**
   - Process all content marked with `[BOTH]`, `[END_USER]`, and `[END_USER_ONLY]`
   - IGNORE content marked with `[TECHNICAL]` and `[TECHNICAL_ONLY]`
   - Generate ONLY sections 1-4 (skip sections 5 and 6 entirely)

## Step 5: Generate the Documentation

**IMPORTANT:** Generate ALL documentation content in the language chosen by the user in Step 1.

- If Spanish: Use Spanish for all headers, descriptions, field labels, and explanations
- If English: Use English for all content
- Maintain technical terms (model names, field names) in their original form

**Documentation Structure:**

- **For Technical Documentation:** Generate all 6 sections
- **For End-User Documentation:** Generate only sections 1-4

Create a Markdown file with the appropriate structure:

```markdown
# Functional Documentation: [Module Name]

### **1. Module Purpose & Business Value**

* **Problem:** [Business problem in standard Odoo]
* **Solution:** [How this module solves it]

### **2. Key Features Overview**

* [Feature 1 with user-facing description]
* [Feature 2 with user-facing description]
* ...

### **3. Key Workflow & User Journey**

[Step-by-step scenario showing how users interact with the module]

1. [Step 1 with role and action]
2. [Step 2 with role and action]
...

### **4. Configuration and Setup Guide**

* **Dependencies:** [Required modules or settings]
* **Security Permissions:** [Which groups need access]
* **Configuration Steps:**
  * [Step-by-step setup instructions]

### **5. Business Objects & Field Breakdown**

#### **5.1. Object: [User-Friendly Name] (`technical.model.name`)**

* **Purpose:** [Why this object was modified/created]

* **New/Modified Fields:**

| Field Label | Technical Name | Field Type | Functional Description |
|:------------|:---------------|:-----------|:-----------------------|
| [Label]     | `field_name`   | [Type]     | [Business purpose]     |

[Repeat for each model]

### **6. Changes to Reports and User Interface**

* **[View/Report Name]:** [Description of changes]
* [Continue for all UI changes]
```

## Step 6: Generate Output in Chosen Format

The `output_format` was determined in Step 1 (Part E or early detection). For all branded formats (PDF, HTML, Presentation), the Markdown is ALWAYS saved first as source of truth.

---

### Option A: Markdown (default)

Write the documentation to `[module_name]_documentation.md` in the current working directory (or user-specified location).

---

### Option B: PDF with Indaws branding

1. **Save Markdown first** — Write `[module_name]_documentation.md`
2. **Delegate to dynapps-brand + pdf-renderer:**

```
Agent({
  description: "Generate branded PDF documentation",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Use common-skills:dynapps-brand to generate a PDF report from the Markdown documentation at [path_to_md]. Document type: technical report. Read dynapps-brand references/pdf-reports.md for layout rules. Then use common-skills:pdf-renderer to generate the final PDF. Output: [module_name]_documentation.pdf in the same directory."
})
```

The PDF will include:
- Indaws logo and brand colors (Montserrat font, brand palette)
- Professional cover page with module name, version, and date
- Table of contents
- Page headers/footers with Indaws branding
- Proper page breaks between sections

---

### Option C: HTML with Indaws branding

1. **Save Markdown first** — Write `[module_name]_documentation.md`
2. **Delegate to dynapps-brand:**

```
Agent({
  description: "Generate branded HTML documentation",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Use common-skills:dynapps-brand to generate a standalone HTML document from the Markdown documentation at [path_to_md]. Document type: HTML document. Read dynapps-brand references/html-documents.md for layout rules. Load brand tokens from dynapps-brand assets/brand-tokens.json. Output: [module_name]_documentation.html in the same directory. The HTML must be self-contained (inline CSS, no external dependencies) and viewable in any browser."
})
```

The HTML will include:
- Indaws brand colors and Montserrat typography (inline CSS)
- Responsive layout
- Navigation sidebar or table of contents
- Self-contained (no external dependencies)

---

### Option D: Presentation with Indaws branding

1. **Save Markdown first** — Write `[module_name]_documentation.md`
2. **Delegate to dynapps-brand:**

```
Agent({
  description: "Generate branded presentation",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Use common-skills:dynapps-brand to generate a presentation from the Markdown documentation at [path_to_md]. Document type: presentation. Read dynapps-brand references/presentations.md for slide structure and layout rules. Structure: title slide, agenda, one slide per major section, summary slide. Output: [module_name]_presentation.html in the same directory."
})
```

The presentation will include:
- Indaws branded title slide with module name
- One slide per documentation section (condensed for visual impact)
- Key features and workflows as bullet points or diagrams
- Ideal for client delivery meetings or team training sessions

---

### After saving (all options)

Inform the user:
1. Where the file(s) were saved (always mention the .md source + the branded output if applicable)
2. Offer to:
   - Review or expand specific sections
   - Regenerate in a different format (e.g., "also want it as PDF?")
