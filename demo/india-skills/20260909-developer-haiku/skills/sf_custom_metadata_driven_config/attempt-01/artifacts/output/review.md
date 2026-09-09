# Lead Routing by Country — Custom Metadata Design

## Overview
Admin-configurable Lead routing rule mapping countries to regional queues with priority-based tie resolution and validation.

## Custom Metadata Types

### 1. Lead_Country_Routing__mdt
Maps country to routing queue with priority, activation, and validation.

| Field | Type | Purpose |
|---|---|---|
| `DeveloperName` | Text (built-in) | Unique key: `ROUTE_<COUNTRY_CODE>` |
| `Country_Code__c` | Text | ISO 3166-1 Alpha-2 (e.g., `US`, `GB`) |
| `Queue_DeveloperName__c` | Text | Target queue API name (e.g., `APAC_Queue`) |
| `Priority__c` | Number | Route order (1=highest); resolves ties deterministically |
| `IsActive__c` | Checkbox | Enable/disable without deletion |
| `Description__c` | Text | Admin-facing reason (e.g., "North America region") |

## Validation & Behavior

**Operator Validation (Apex trigger)**
- Country_Code__c: must match regex `^[A-Z]{2}$`
- Queue_DeveloperName__c: must reference existing queue (SOQL check)
- Priority__c: must be > 0

**Tie Handling**
- Sort by Priority__c ascending; lowest wins
- Secondary sort by DeveloperName alphabetically

**Missing Configuration**
- If country not found in active routing rules, assign to `Default_Sales_Queue`
- Log to custom object `Lead_Routing_Exception__c` for audit

## Sample Metadata Records

```
DeveloperName: ROUTE_US
Country_Code__c: US
Queue_DeveloperName__c: NA_Sales_Queue
Priority__c: 1
IsActive__c: true
Description__c: United States leads to NA queue

---
DeveloperName: ROUTE_GB
Country_Code__c: GB
Queue_DeveloperName__c: EMEA_Sales_Queue
Priority__c: 2
IsActive__c: true
Description__c: United Kingdom leads to EMEA queue

---
DeveloperName: ROUTE_DEFAULT
Country_Code__c: ZZ
Queue_DeveloperName__c: Default_Sales_Queue
Priority__c: 999
IsActive__c: true
Description__c: Fallback for unmapped countries
```

## Apex Evaluator Pattern

```apex
@InvocableMethod(label='Route Lead by Country')
public static List<String> routeLeadByCountry(List<Lead> leads) {
    Map<String, Lead_Country_Routing__mdt> routingMap = 
        new Map<String, Lead_Country_Routing__mdt>();
    
    for (Lead_Country_Routing__mdt route : 
         [SELECT Country_Code__c, Queue_DeveloperName__c, Priority__c 
          FROM Lead_Country_Routing__mdt 
          WHERE IsActive__c = true 
          ORDER BY Priority__c, DeveloperName]) {
        routingMap.put(route.Country_Code__c, route);
    }
    
    List<String> results = new List<String>();
    for (Lead lead : leads) {
        Lead_Country_Routing__mdt route = routingMap.get(lead.Country);
        results.add(route != null ? route.Queue_DeveloperName__c : 'Default_Sales_Queue');
    }
    return results;
}
```

## Deployment
1. Deploy `Lead_Country_Routing__mdt` object definition first
2. Deploy individual metadata records as `.md-meta.xml` files
3. Document in runbook: production records managed by admin; source records are seeds

## Status: Design Complete
✓ Priority-based routing with deterministic tie resolution
✓ Activation control for admin flexibility
✓ Operator validation constraints defined
✓ Missing-config fallback (Default_Sales_Queue)
✓ Audit logging strategy (custom exception object)
