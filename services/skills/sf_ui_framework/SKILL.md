---
name: sf-ui-framework
description: Use when implementing or reviewing UI Framework (Domain Knowledge) in
  Salesforce.
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

# Skill: UI Framework (Domain Knowledge)

Use this when building or reviewing Lightning UI.

## Expertise areas

- **LWC:** Lightning Web Components, wire adapters, Lightning Data Service (LDS), lifecycle hooks.
- **Aura:** Legacy support when required (components, events).
- **Lightning Messaging Service (LMS):** Cross-tab and cross-component communication.
- **SLDS:** Salesforce Lightning Design System — markup, tokens, accessibility.
- **Dynamic Forms / FlexiPages:** Field sections, header regions, visibility filters, `uiBehavior`.
- **Quick Actions:** Object-specific, screen-flow-launching, header actions.

## Key patterns

- Prefer LDS (`getRecord`, `updateRecord`, etc.) over custom Apex wire when CRUD is the only need.
- Use SLDS classes and structure; avoid one-off CSS that diverges from the design system.
- For LWC tests: Jest with standard LWC test utilities and wire mocks.
- Quick actions in `platformActionList` use the qualified `Object.ActionName` form.
- Screen Flows: the first screen must have `allowBack=true` or `allowFinish=true` (Salesforce requires it).
- `enablePageActionConfig=true` on a FlexiPage requires Dynamic Actions for the object to be enabled in Setup; do not flip it without confirming the org allows it.

## Common LWC scaffolding (per component)

| File | Purpose |
|---|---|
| `<name>.html` | SLDS markup; no inline styles |
| `<name>.js` | Logic; wire adapters; LDS imports |
| `<name>.css` | Component-scoped styles only |
| `<name>.js-meta.xml` | `<isExposed>`, target page types, design properties |
| `__tests__/<name>.test.js` | Jest tests with `@salesforce/sfdx-lwc-jest` |

## Accessibility checklist

- [ ] Every interactive element has a label (visible or `aria-label`).
- [ ] Keyboard navigation works (Tab order, Enter / Space activation).
- [ ] Color contrast meets WCAG AA via SLDS tokens.
- [ ] No text-only state (use icons + text together).
