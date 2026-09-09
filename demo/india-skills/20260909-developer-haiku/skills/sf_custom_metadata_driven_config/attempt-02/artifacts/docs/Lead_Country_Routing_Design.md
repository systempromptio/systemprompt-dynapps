# Lead Country Routing via Custom Metadata

## Overview
Admin-configurable routing rule mapping Lead Country to a target queue/owner using Custom Metadata Type. Supports priority-based evaluation, activation control, operator validation, and deterministic tie-breaking.

## Schema: Lead_Country_Routing__mdt

| Field | Type | Purpose |
|---|---|---|
| `DeveloperName` | Text (built-in) | Stable identifier (e.g., `ROUTE_US`, `ROUTE_EMEA`) |
| `Country__c` | Text | Country value to match (source key) |
| `Operator__c` | Picklist | Comparison: `==`, `!=`, `contains`, `startsWith`, `endsWith` |
| `Target_Queue_Id__c` | Text | Queue or User Id to route to |
| `Priority__c` | Number | Sort order (ascending); lower number = higher priority |
| `Is_Active__c` | Checkbox | Enable/disable rule without deletion |
| `Description__c` | Text | Admin notes |

## Evaluation Rules

1. **Filter:** Include only records where `Is_Active__c = true`
2. **Sort:** By `Priority__c` (ascending), then `DeveloperName` (ascending) for tie-breaking
3. **Match:** Return first rule where `Operator__c` applied to Lead.Country equals true
4. **No Match:** Explicitly return null (no default; caller handles fallback)
5. **Invalid Config:** Reject unsupported operators; log and skip malformed records

## Supported Operators

- `==` – exact match
- `!=` – not equal
- `contains` – substring match
- `startsWith` – prefix match
- `endsWith` – suffix match

## Tie-Breaking
Equal priority → sort by `DeveloperName` (ascending). First match wins; no overwrite.

## Missing Config Behavior
If no active rule matches, return `null`. Caller must define fallback (default queue, error, escalate).

## Deployment
1. Deploy CMT object definition (Lead_Country_Routing__mdt)
2. Deploy seed records as `.md-meta.xml` files
3. Document that production records may diverge after admin edits
