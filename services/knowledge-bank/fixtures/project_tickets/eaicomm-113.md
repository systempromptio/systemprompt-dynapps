---
key: EAICOMM-113
title: "Data | PDP | Validate configurator templates on upload"
type: Story
status: To do
sprint: "Sprint 15 (2026-08-03 to 2026-08-16)"
assignee: Tomasz Wieczorek
project: EAICOMM
date: 2026-08-04
---

# EAICOMM-113 Data | PDP | Validate configurator templates on upload

## Context

Templates are authored by the product data team as JSON content assets under configurator/templates. An invalid template must be refused at upload with a readable error rather than discovered by a customer seeing the fallback selector. This story adds the validation step to the content import job using the same schema as the storefront.

## Acceptance criteria

1. A template that fails the schema is refused at import and the import log names the template and the first violation.
2. A template that references an option ID that does not exist on the master product is refused.
3. A valid template imports and the product renders the configurator on staging.

## Out of scope

Anything not named in the acceptance criteria above, and in particular any change to the OMS, ERP or PSP integrations beyond what the criteria state; those need their own ticket and, for production, a change ticket from Marcus Feld. The story is accepted against the criteria as written, not against the specification page it links, which may move on.

## Technical notes

(left blank by the BA; the developer who picks up the story fills this in)

## Comments

- **Decision** (Ana Lucia Ferreira, 2026-08-05): The schema is owned by the storefront repository and published as a package the import job depends on, so the two cannot drift.

Status: To do. Sprint: Sprint 15 (2026-08-03 to 2026-08-16). Assignee: Tomasz Wieczorek.
