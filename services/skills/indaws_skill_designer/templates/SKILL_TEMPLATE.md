---
name: {{SKILL_NAME}}
version: 1.0.0
model: {{sonnet|opus}}
effort: {{low|medium|high}}
description: "{{TRIGGER_DESCRIPTION}}"
---

# {{SKILL_NAME}} v1.0

{{ONE_LINE_CONTEXT}}

## ROL
{{ONE_LINE_ROLE}}

## WORKFLOW

### Step 0 — Input [STOP]
{{WHAT_TO_ASK_USER}}

### Step 1 — {{FIRST_ACTION}}
{{INSTRUCTIONS}}

### Step 2 — {{SECOND_ACTION}}
{{INSTRUCTIONS}}

### Step N — Output
{{WHAT_TO_DELIVER}}

## KEY RULES

### NEVER
1. {{CRITICAL_DONT_1}}
2. {{CRITICAL_DONT_2}}

### ALWAYS
1. {{CRITICAL_DO_1}}
2. {{CRITICAL_DO_2}}

## GOTCHAS
1. **{{GOTCHA_1}}** — {{EXPLANATION}}
2. **{{GOTCHA_2}}** — {{EXPLANATION}}
3. **{{GOTCHA_3}}** — {{EXPLANATION}}

## Delegates to
- {{OTHER_SKILL_OR_None}}

## Consumed by
- {{WHO_INVOKES_THIS_OR_None}}

## REFERENCES
- `common:references/shared/{{SHARED}}.md` — {{connection/batch/fields si aplica; si no, quita esta línea}}
- `references/{{REF_1}}.md` — {{DESCRIPTION}}
- `scripts/{{SCRIPT_1}}.py` — {{DESCRIPTION}}
