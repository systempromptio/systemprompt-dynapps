# Permission Set Review: Lead Reviewer

## Design Summary

Three metadata files created for Sales Cloud Lead reviewer persona:

### Files Written
- `/home/tester/work/output/Lead_Reviewer_Object_Access.permissionset-meta.xml`
- `/home/tester/work/output/Lead_Reviewer_System_Access.permissionset-meta.xml`
- `/home/tester/work/output/Lead_Reviewer.permissionsetgroup-meta.xml`

## Permissions Defined

**Lead Object Access (Lead_Reviewer_Object_Access)**
- Create: ✓ true
- Read: ✓ true
- Edit: ✓ true
- Delete: ✗ false
- Modify All Records: ✗ false
- View All Records: ✗ false

**Field-Level Permissions (FLS)**
- Country: Read ✓, Edit ✓
- Email: Read ✓, Edit ✓
- Status: Read ✓, Edit ✓

**System Access (Lead_Reviewer_System_Access)**
- Salesforce app visibility: true
- AllowViewKnowledge: true

## Permission Set Group Structure

`Lead_Reviewer` combines:
1. `Lead_Reviewer_Object_Access` (object and FLS)
2. `Lead_Reviewer_System_Access` (system and app visibility)

## Test Cases (Unexecuted)

| Scenario | Expected Result |
|----------|-----------------|
| Assigned user opens Lead record | Can view, can edit Country/Email/Status |
| Assigned user deletes Lead | Permission denied; Delete = false |
| Assigned user modifies all Leads | Permission denied; Modify All Records = false |
| Unassigned user accesses Lead | No additional permissions gained |

## Notes

- No profile modification proposed; profiles remain unchanged
- Standard naming conventions applied per skill guidance
- Deployment order: 1) Permission Sets, 2) Permission Set Groups
- User assignment via Setup → Users → Permission Set Groups (separate step)
