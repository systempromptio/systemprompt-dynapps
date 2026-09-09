> Astound marketplace adaptation: Historical upstream project template, not Claude Code installation instructions or facts about your org. Cursor paths and agent rosters describe the source kit and are not installed by this marketplace. Use the installed sf-salesforce-project-context skill and the current repository instructions for this client.

# Implementation Brief — `<STORY-ID>`

> Lightweight pre-implementation brief that the agent (or a human Feature Lead) writes before any code is committed. Lives at the start of the work; superseded by the story's `design-note.md` once implementation begins.

---

## 1. Story summary

- **Story / ticket:** `<STORY-ID>`
- **One-line description:** _<what we are building>_
- **Acceptance criteria:** _<list>_

## 2. Schema impact

| Object | Field / Component | Change | Source of truth |
|---|---|---|---|
| `<Object>` | `<Field__c>` | new / changed / unchanged | `docs/PROJECT_SCHEMA.md` |

If this story introduces new custom objects / fields, they will be added to `docs/PROJECT_SCHEMA.md` in the same change.

## 3. Components to add or change

| Type | Name | Purpose |
|---|---|---|
| Trigger | `<ObjectTrigger>` | _<one-line>_ |
| Handler | `<ObjectTriggerHandler>` | _<one-line>_ |
| Flow | `<flow_api_name>` | _<one-line>_ |
| Validation Rule | `<rule_api_name>` | _<one-line>_ |
| LWC | `<componentName>` | _<one-line>_ |
| Permission Set | `<persona_access>` | _<one-line>_ |

## 4. Agents assigned (and rationale)

| Agent | Rationale |
|---|---|
| `Feature_Lead` | Initial design |
| `<Apex_Architect / Declarative_Lead / UI_UX_Specialist>` | _<why>_ |
| `QA_Engineer` | Always — test class + manual plan + video |

## 5. Architectural choice (declarative vs Apex)

State explicitly which option was considered and why the chosen path is correct (per `always-active.md` Apex priority rule).

- **Considered:** _<Flow / Validation / CMT / Apex>_
- **Chose:** _<...>_
- **Why:** _<one-line>_

## 6. Deployment ordering

1. _<dependency 1>_
2. _<dependency 2>_
3. _<change itself>_
4. _<post-deploy: layout assignment, perm set assignment, Flow activation, etc.>_

## 7. Rollback strategy

_<reverse order, destructive package, or manual revert steps>_

## 8. Test strategy

- Apex test class(es): _<list>_
- Manual test cases: `MT-NN` style, in `qa-test-request.md`
- Video capture of manual UI testing: required

## 9. Open questions

- _<question to clarify with PO / admin / business>_
