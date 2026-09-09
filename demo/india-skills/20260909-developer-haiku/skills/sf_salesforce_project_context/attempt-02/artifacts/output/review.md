# Salesforce Project Context — Fixture Review

**Date:** 2026-09-09  
**Scenario:** sf_salesforce_project_context (synthetic fixture)

---

## Summary

Generated `project-context.md` identifying the **India Skills Fixture** as a Sales Cloud (API 64.0) project with synthetic test data. Verified metadata from local fixtures; distinguished bundled template examples from fixture facts.

---

## Artifacts Saved

| Path | Type | Content |
|---|---|---|
| `/home/tester/work/project-context.md` | Markdown | Project identity, verified Lead schema, existing automations (LeadHandler, Demo app), naming conventions, unknown org details, template vs. fixture classification matrix |

---

## Verified Facts (Evidence)

| Fact | Source | Confidence |
|---|---|---|
| Project name: "India Skills Fixture" | `fixtures/project-brief.md:1–2` | **High** |
| Edition: Sales Cloud | `fixtures/project-brief.md:3` | **High** |
| sourceApiVersion: 64.0 | `sfdx-project.json:1` | **High** |
| Lead standard fields (8 listed) | `docs/PROJECT_SCHEMA.md:3` | **High** |
| Status picklist: "Open - Not Contacted", "Qualified" | `docs/PROJECT_SCHEMA.md:4` | **High** |
| LeadHandler.cls exists, uses static bypass flag | `fixtures/LeadHandler.cls` | **High** |
| Demo app metadata incomplete (missing FlexiPage) | `fixtures/broken-app.xml:2 comment` | **High** |

---

## Unknown Org Details (Fixture Gaps)

Per `docs/PROJECT_SCHEMA.md:5–6`, the following are explicitly unknown:

- Org identity, queues, users, record types
- Lead conversion mappings, Dynamic Actions
- Validation rules, Flows, custom metadata
- Permission sets, FLS, sharing model

---

## Template vs. Fixture Classification

- **Bundled references** (`references/rules/`, `references/docs/`): template examples; do not assume they apply to this fixture
- **Fixture files** (`fixtures/`, `docs/PROJECT_SCHEMA.md`, `sfdx-project.json`): verified facts for synthetic project context

---

## Unsupported Operations

None. All requested deliverables completed using workspace file tools only (Read, Write, Glob).

