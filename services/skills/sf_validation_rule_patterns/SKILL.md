---
name: sf-validation-rule-patterns
description: Use when implementing or reviewing Validation Rule Patterns in Salesforce.
---

## Systemprompt execution contract

The marketplace provisions authenticated `atlassian`, `salesforce`, and `github`
MCP connectors for the signed-in user. Use their advertised tools and input
schemas. Never request, print, or copy credentials, and never bypass Systemprompt
with direct provider HTTP requests. Missing tools or rejected authentication are
failures: name the connector and stop the affected operation. Do not substitute
mock data or report an unexecuted action as successful.

Use the current repository as project context. Confirm unknown project identifiers
from authenticated reads; never invent an org, Jira key, schema, or repository.
Respect the user's existing authorization and the review requirements below.
For provider writes, use the format accepted by the advertised input schema.

# Skill: Validation Rule Patterns

Reusable validation-rule formulas and conventions distilled from real Sales / Service Cloud projects. Use this skill whenever a story asks for "must be filled when ...", "must follow format ...", or "cannot change after ...".

## Authoring rules

1. **One purpose per rule.** Combine multiple acceptance criteria in one rule **only** when they share the same activation context and the same error message would make sense.
2. **Name** the rule for what it enforces, not the AC number: `Field_Required_When_Stage`, `Format_Email_Lead`, `Block_Edit_Closed_Opportunity`.
3. **`errorDisplayField`:** when the rule targets a single field, attach the error to that field (e.g. `Email`); use top-of-page only for cross-field rules.
4. Always paraphrase the error message so a user knows **what to fix** (not just "invalid input").
5. Add a manual test case (`MT-NN`) to the story's `qa-test-request.md` for every new validation rule.

## Pattern 1 — Required field when status / stage transitions

```
AND(
    ISPICKVAL(StageName, "<TargetStageOrStatus>"),
    ISBLANK(<RequiredField__c>)
)
```

For text values inside a picklist field, wrap with `TEXT(...)`:

```
AND(
    ISPICKVAL(StageName, "<TargetStage>"),
    ISBLANK(TEXT(<PicklistField__c>))
)
```

When more than one field is required, prefer narrow rules per field (or one rule with `OR(ISBLANK(...), ISBLANK(...))` and an error message that lists every missing field).

## Pattern 2 — At least one of N must be filled

```
AND(
    ISBLANK(<FieldA>),
    ISBLANK(<FieldB>),
    ISBLANK(<FieldC>)
)
```

Error message: "At least one of A, B, or C is required."

## Pattern 3 — Cross-field "if A then B"

Marketing-rejection style: the field A is allowed to satisfy a downstream rule when its sibling B is blank.

```
AND(
    ISPICKVAL(<Status>, "Cancelled"),
    ISBLANK(TEXT(<Closed_Reason__c>)),
    ISBLANK(<Rejection_Reason__c>)     /* allow either reason field to satisfy the rule */
)
```

## Pattern 4 — Email format

```
AND(
    NOT(ISBLANK(Email)),
    OR(
        CONTAINS(Email, " "),
        NOT(REGEX(Email, ".+@.+\\..+"))
    )
)
```

REGEX `.+@.+\\..+` requires `user@domain.tld` minimum. Document the intent in the design note.

## Pattern 5 — Phone / mobile format hardening

Catches unmatched parentheses, leading / trailing hyphens, double spaces, consecutive hyphens, hyphen-adjacent spaces, and "all the same digit" numbers (e.g. `9999999999`).

```
AND(NOT(ISBLANK(Phone)),
    OR(
        /* unmatched parentheses: count '(' vs ')' */
        (LEN(Phone) - LEN(SUBSTITUTE(Phone,"(","")))
            != (LEN(Phone) - LEN(SUBSTITUTE(Phone,")",""))),
        LEFT(TRIM(Phone), 1) = "-",
        RIGHT(TRIM(Phone), 1) = "-",
        CONTAINS(Phone, "  "),
        CONTAINS(Phone, "--"),
        CONTAINS(Phone, " -"),
        CONTAINS(Phone, "- "),
        REGEX(
            SUBSTITUTE(SUBSTITUTE(SUBSTITUTE(
            SUBSTITUTE(SUBSTITUTE(SUBSTITUTE(
                Phone," ",""),"-",""),"(",""),")",""),"+",""),".",""),
            "(\\d)\\1{9,}"
        )
    )
)
```

`(\\d)\\1{9,}` after stripping formatting catches 10+ consecutive identical digits.

## Pattern 6 — Name format (FirstName / LastName)

`FirstName` is part of a standard compound name and **cannot be set required via metadata** — enforce required + format via a validation rule:

```
OR(
    ISBLANK(FirstName),
    AND(NOT(ISBLANK(FirstName)),
        OR(
            LEN(TRIM(FirstName)) = 1,
            REGEX(FirstName, ".*[0-9].*"),         /* contains digits */
            NOT(REGEX(FirstName, ".*[a-zA-Z].*")), /* only special chars */
            CONTAINS(FirstName, "..")
        )
    )
)
```

## Pattern 7 — Block edits after a status / stage is reached

When a user must not change fields once the record is "closed" (or any terminal status), do this in **Apex** rather than a validation rule (you need to compare each populated field against its prior value). The Apex pattern lives in `.cursor/skills/Bypass_Patterns.md` and the `Apex_Architect` agent.

A pure-formula version that blocks **any** change while a status is set:

```
AND(
    PRIORVALUE(IsClosed) = true,
    NOT(ISCHANGED(IsClosed))   /* still closed; not just being closed/reopened */
)
```

This tells you the record was already closed and stays closed; pair with a check on a specific field if you want to allow some fields to remain editable. For the full "edit any updateable field" enforcement, prefer the Apex trigger handler pattern.

## Pattern 8 — Standard field that must keep a value (read-only via FLS)

For fields like `Probability` that must not be edited by certain personas, prefer **profile / permission set FLS** (`editable=false`, `readable=true`) over a validation rule. Keep the rule for cross-field invariants only.
