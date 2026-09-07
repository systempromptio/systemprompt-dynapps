---
name: sf-salesforce-project-context
description: Use when implementing or reviewing Salesforce project context in Salesforce.
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

# Skill: Salesforce project context

> **This file is a TEMPLATE.** Fill in the placeholders for the current project. The agent reads this first on any task to ground itself in this project's conventions and existing automations.

Use when implementing or reviewing features that use this project's custom objects and fields, or when adding new Salesforce metadata.

## Purpose

Improve accuracy by grounding all implementation in the project's **canonical schema** and **conventions**, so the agent and automation use correct API names and patterns.

## Where to look

| Need | File |
|------|------|
| Custom object / field API names, picklist values, relationships | `docs/PROJECT_SCHEMA.md` |
| Standing rules (Flow vs Apex, triggers, security) | `the project standing rules (bundled reference in sf-salesforce-project-context)` |
| Story intake and agent routing | `the project story-intake rules (bundled reference in sf-salesforce-project-context)` |
| Apex / SOQL / order of execution | `the sf-core-logic skill` |

---

## 1. Project identity

- **Project name:** _<fill in>_
- **Primary Salesforce edition / clouds in scope:** _<e.g. Sales Cloud, Service Cloud, CPQ, Industries, Marketing Cloud, Data Cloud>_
- **Default package directory:** `force-app/`
- **`sourceApiVersion`:** _<from `sfdx-project.json`>_
- **Branching model:** `feature/<STORY-ID>` -> `develop` -> `main` (or describe variation).

## 2. Custom objects and fields

Maintain the canonical list in **`docs/PROJECT_SCHEMA.md`** and reference it from here. Below is a quick lookup of the **most-used** objects and the conventions that apply.

| Object (API) | Purpose | Key custom fields | Notes |
|---|---|---|---|
| _<Object__c>_ | _<one-line purpose>_ | _<Field_API__c (type), Field_API__c (type)>_ | _<recordTypes, sharing notes>_ |

> Always re-read `docs/PROJECT_SCHEMA.md` for the authoritative list before writing code.

## 3. Existing automations (catalog — fill in per project)

Document the **existing** triggers, handlers, flows, and CMT-driven rules so the agent knows what already lives in the org and avoids duplicating logic.

| Component | Type | What it does | Files |
|---|---|---|---|
| _<TriggerName>_ | Apex Trigger | _<one-line behavior>_ | `force-app/main/default/triggers/<name>.trigger` |
| _<HandlerName>_ | Apex Handler | _<bulkified business logic for the trigger>_ | `force-app/main/default/classes/<name>.cls` |
| _<FlowApiName>_ | Record-Triggered Flow (before-save / after-save) | _<one-line behavior>_ | `force-app/main/default/flows/<api-name>.flow-meta.xml` |
| _<ConfigCMT>_ | Custom Metadata Type | _<rule / mapping it drives>_ | `force-app/main/default/objects/<Cmt__mdt>/` |

## 4. Naming conventions (this project)

| Component | Convention |
|---|---|
| Triggers | `ObjectNameTrigger` (e.g. `OpportunityTrigger`) |
| Handlers | `ObjectNameTriggerHandler` or `ObjectNameHandler` |
| Validation rules | `Field_Required_When_Stage` / `Format_<Field>_<Reason>` |
| Flows | `Object_When_What` (e.g. `Opportunity_BeforeSave_Auto_Close`) |
| LWCs | `camelCase` (e.g. `accountHealthBanner`) |
| Custom objects / fields | `PascalCase__c` |
| Picklist values | Title Case |
| Permission sets | `<Persona>_Object_Access` for object permissions; `<Persona>_System_Access` for user / tab |
| Permission set groups | `<Persona>` (groups bundle multiple permission sets) |
| Custom permissions | `<Object>_<Action>_Bypass` for bypass logic |

## 5. Project-wide rules (this project)

1. **Object / field names**
   Use only API names from `docs/PROJECT_SCHEMA.md`. Do not guess or introduce new custom fields / objects without adding them to the schema in the same change.

2. **Adding new objects / fields**
   - Add the object / field to `force-app/` metadata.
   - Document it in `docs/PROJECT_SCHEMA.md` (API name, type, description, picklist values if any).
   - Follow one-trigger-per-object and handler pattern if adding trigger logic.
   - Add to the relevant page layout(s) in the same change so users see the field after deploy.
   - Update FLS on the relevant profile / permission set in the same change.

3. **Bypass / integration users**
   - Use a **Custom Permission** (assignable via permission set) for bypass logic. Do not hardcode profile name checks in Apex unless the bypass is for the System Administrator.
   - Integration identities are assigned the relevant `<Object>_<Action>_Bypass` permission set.

## 6. Pre-implementation checklist (for accuracy)

- [ ] Confirmed which objects / fields are used from `docs/PROJECT_SCHEMA.md`.
- [ ] Any new custom object / field is added to the schema doc in the same PR.
- [ ] Flow / Validation / CMT vs Apex decision is explicit (per `always-active.md`: prefer declarative when it satisfies the requirement).
- [ ] Trigger / handler naming matches project conventions.
- [ ] Page layout, FLS, and (if needed) record type visibility are updated in the same change.
- [ ] Story folder (`stories/<story-id>/`) exists and all four files are present and current.

## Bundled project templates

Read references/rules/always-active.md and references/rules/story-intake.md. The bundled references/docs files are upstream templates, not verified facts about the current project. Resolve project identity from the current repository and authenticated Salesforce metadata before filling any placeholders. Never assume the example schema is the target org.
