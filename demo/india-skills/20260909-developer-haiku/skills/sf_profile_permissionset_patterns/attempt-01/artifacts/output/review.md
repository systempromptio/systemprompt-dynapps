# Permission Set Design Review

## Scenario
Design permission sets for a Lead Reviewer persona with read/edit access to Country, Email, and Status fields, excluding delete and Modify All privileges.

## Artifacts Generated

### 1. Lead_Reviewer_Object_Access.permissionset-meta.xml
- **Purpose**: Object and field-level permissions
- **Lead Object**: Read=true, Edit=true, Create=false, Delete=false, ModifyAll=false, ViewAll=false
- **Field Permissions**: Country, Email, Status all set to readable=true, editable=true
- **Rationale**: Restricts access to review-only operations, no destructive actions

### 2. Lead_Reviewer_System_Access.permissionset-meta.xml
- **Purpose**: Application launcher visibility
- **App Visibility**: Demo app set to visible=true with default=false
- **Rationale**: Ensures Demo app appears in App Launcher without leaking visibility to unintended profiles

### 3. Lead_Reviewer.permissionsetgroup-meta.xml
- **Purpose**: Aggregates both permission sets
- **Members**: Lead_Reviewer_Object_Access, Lead_Reviewer_System_Access
- **Rationale**: Follows naming convention `<Persona>` and deployment hierarchy

## Compliance with Requirements
✓ No delete permissions on Lead object  
✓ No Modify All/View All privileges  
✓ Field permissions limited to Country, Email, Status  
✓ App visibility included (Demo app)  
✓ Full profiles preserved (none modified)  
✓ No extra privileges granted  
✓ Follows Salesforce naming and deployment patterns  

## Deployment Order
1. Permission Sets (both .permissionset-meta.xml files)
2. Permission Set Group (Lead_Reviewer.permissionsetgroup-meta.xml)
3. Assign group to users via Setup > Users > Permission Set Groups
