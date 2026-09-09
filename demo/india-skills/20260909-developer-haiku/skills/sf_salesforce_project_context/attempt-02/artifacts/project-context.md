# India Skills Fixture — Project Context

> **Fixture Status:** Synthetic test data only. No live Salesforce org or GitHub branch configured.

## 1. Project Identity

- **Project name:** India Skills Fixture
- **Primary Salesforce edition / clouds in scope:** Sales Cloud
- **Default package directory:** `force-app/`
- **`sourceApiVersion`:** 64.0 (verified from `sfdx-project.json`)
- **Branching model:** Feature branches not active in fixture; deployment validation only

---

## 2. Standard Objects and Fields (Verified)

### Lead (Standard Object)

| Field (API) | Type | Notes |
|---|---|---|
| Id | Standard | Record identifier |
| FirstName | Text | Standard field |
| LastName | Text | Standard field |
| Company | Text | Standard field |
| Email | Email | Standard field |
| Country | Text | Standard field; used in handler updates |
| Status | Picklist | **Known values:** Open - Not Contacted, Qualified |
| OwnerId | Lookup | Standard owner reference |

**Custom Fields:** None verified in fixture.

---

## 3. Existing Automations (Fixture Metadata)

| Component | Type | What it does | Files | Status |
|---|---|---|---|---|
| LeadHandler | Apex Handler | Updates Lead.Country on insert/update; has bypass flag | `fixtures/LeadHandler.cls` | Fixture artifact |
| Demo | Custom Application | Sales Cloud app with utility bar reference | `fixtures/broken-app.xml` | Incomplete (missing FlexiPage) |

---

## 4. Naming Conventions

**From bundled template (`always-active.md`)**

| Component | Convention |
|---|---|
| Triggers | `ObjectNameTrigger` (e.g. `LeadTrigger`) |
| Handlers | `ObjectNameTriggerHandler` or `ObjectNameHandler` |
| Flows | `Object_When_What` (e.g. `Lead_BeforeSave_Defaults`) |
| Custom objects / fields | `PascalCase__c` |
| Picklist values | Title Case |

**Fixture-specific rules:** Roundoff on checkout values using half-to-even rounding; no email send.

---

## 5. Unknown Org Details (Fixture Gaps)

- Org identity and sandboxes
- Queue definitions and ownership routing
- User list and role hierarchy
- Lead record types and page layouts
- Lead conversion mappings and settings
- Dynamic Actions or Custom Buttons
- Validation rules or Record-Triggered Flows
- Permission sets and FLS configuration
- Custom metadata types or CMT records

---

## 6. Bundled Templates vs. Verified Fixture Facts

| Source | Classification | Trust Level |
|---|---|---|
| `sfdx-project.json` | Verified fixture config | **High** |
| `docs/PROJECT_SCHEMA.md` (fixture) | Synthetic schema declaration | **High** |
| `fixtures/LeadHandler.cls` | Fixture artifact | **High** |
| `fixtures/broken-app.xml` | Fixture artifact (incomplete) | **Medium** (app missing utility FlexiPage) |
| `references/docs/PROJECT_SCHEMA.md` | Bundled template (example only) | **Low** — not this project's schema |
| `references/rules/always-active.md` | Bundled standing rules | **Medium** — project conventions apply; verify overrides in project schema |

---

## 7. Key Implementation Notes

1. **Lead updates:** LeadHandler runs on Lead records and updates the Country field. Bypass flag is static Boolean initialized to false.
2. **App deployment:** The Demo app references a utility bar (Demo_UtilityBar) that must exist as a FlexiPage before app deploy succeeds.
3. **No custom fields verified:** The fixture schema explicitly states no custom fields are verified. Any new fields must be added to `docs/PROJECT_SCHEMA.md` in the same change.
4. **Testing scope:** Fixture omits email sends and uses half-to-even rounding for numeric operations.

