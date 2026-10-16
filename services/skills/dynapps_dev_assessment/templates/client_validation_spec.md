# Functional Specification for Client Validation

Template for producing validation documents the client can understand and approve.

**LANGUAGE:** this template is English. **Render the document in the client's language** —
headings included. If the client's language is unknown, ask before sending.

**WRITING RULES:**
- Non-technical language: the reader is a business owner, not a programmer
- Do NOT use internal Odoo terminology (models, fields, transient, QWeb, etc.)
- DO use the client's own business terminology (invoice, product, warehouse, etc.)
- Short, direct sentences
- Examples with real, concrete numbers
- Describe WHAT the user will see and do, not HOW it is implemented
- No emoji

---

## Document Structure

```markdown
# {descriptive title of the functionality}

**Client**: {client name}
**Date**: {date}
**Version**: 1.0
**Status**: Pending validation

---

## 1. What will be built

{Clear description of the functionality in 3–5 sentences. Answers: "what will the user be able to
do that they cannot do today?" Use the client's everyday language.}

## 2. Why it is needed

{Briefly explain why Odoo does not include it out of the box and why a specific development is
required. 2–3 sentences maximum. No jargon.}

## 3. How it will work, step by step

{Describe the full flow from the user's point of view. Each step is a concrete action the user
performs or sees on screen.}

**Step 1**: {user action}
**Step 2**: {what they see on screen / what they select}
**Step 3**: {result}
…

## 4. What information will be shown

{List the data the user will see, using business names (not field names). If there is a table or
list, describe the columns with an example.}

| {Column 1} | {Column 2} | {Column 3} | … |
|---|---|---|---|
| {example row 1} | … | … | … |
| {example row 2} | … | … | … |

## 5. Worked example

{A concrete case with realistic data. The client should be able to read this example and say
"yes, that is exactly what I need." Include numbers, product names, quantities, amounts.}

**Situation**: {describe the scenario}

**Expected result**: {what the system would show}

## 6. Where it will be in Odoo

{State the exact menu path where the user will find the new functionality. If there are new
buttons or actions, state where they will be.}

- Menu: {menu path}
- {Additional buttons or actions, if any}

## 7. What it does NOT include

{An explicit list of what is out of scope. This is CRITICAL to avoid misunderstandings. Be very
specific.}

- {Exclusion 1}
- {Exclusion 2}
- …

## 8. Acceptance criteria

{Verifiable conditions that determine whether the development is complete and correct. Written as
statements that can be checked.}

1. {Verifiable criterion 1}
2. {Verifiable criterion 2}
3. …

---

**To validate this document, please confirm in writing that the content described matches what you
need. If any point is unclear or needs adjusting, tell us and we will revise it before development
starts.**
```

---

## Section-by-Section Writing Guide

### Section 1: What will be built
- Open with: "A new capability will be added that allows…"
- 5 sentences maximum
- Do NOT mention modules, models, or technologies

### Section 2: Why it is needed
- Open with: "Odoo does not currently include…"
- Explain the GAP in simple language
- Do NOT use "transient model", "QWeb", "inheritance" or similar

### Section 3: How it will work, step by step
- 5–8 steps maximum
- Each step starts with a user action verb: "Open…", "Select…", "Click…"
- If there is a wizard, describe each screen as a step
- Include what happens when the user clicks the final button

### Section 4: What information will be shown
- Use column names the user understands (NOT technical field names)
- Always include an example table with 2–3 rows of fictional data
- If there are totals, show them

### Section 5: Worked example
- Use realistic data from the client's sector
- Include product names, quantities, and concrete amounts
- Describe the "before" (current situation) and the "after" (result of the development)

### Section 6: Where it will be in Odoo
- Menu path with arrows: Menu > Submenu > Option
- If it is a new button on an existing view, state which screen it appears on

### Section 7: What it does NOT include
- List at least 2–3 exclusions
- Be very specific: "Does not include export to format X" rather than "Does not include other
  functionality"
- Anticipate common misunderstandings

### Section 8: Acceptance criteria
- Write as verifiable statements: "The report shows…", "The user can…", "When exporting to PDF…"
- Between 4 and 8 criteria
- Cover: main functionality, exports, permissions, data shown
