# Lead_Review Screen Flow – Deployment Runbook

## Deployment Sequence

### Phase 1: Flow Deployment (Inactive)
**Step 1.1**: Deploy `Lead_Review` flow as **inactive** (API: `Lead_Review`, v1.0)
- **Validation**: First screen must have `allowBack="true"` or `allowFinish="true"` 
- **Action**: Verify XML in metadata shows `<screens><screen allowBack="true">` on initial screen
- **Status**: Blocks step 2 if missing

**Step 1.2**: Confirm prerequisite components exist
- Permission Set: validate `prerequisite` dependency resolved
- Custom Application `Demo`: confirm present and deployable

### Phase 2: Supporting Metadata
**Step 2.1**: Deploy `Demo_UtilityBar` component to `CustomApplication Demo`
- Resolves: "CustomApplication Demo: Demo_UtilityBar missing"

**Step 2.2**: Deploy or update PermissionSet with prerequisites
- Resolves: "PermissionSet: permission requires prerequisite"
- Use permission set instead of profile for entry conditions (avoid direct profile refs)

### Phase 3: Quick Action & Layout
**Step 3.1**: Deploy `Lead.Review` Quick Action
- References: `Lead_Review` flow (must be deployed first, else deployment fails)
- Scope: Lead object, button visibility via permission set

**Step 3.2**: Deploy Layout with Quick Action button
- Include `Lead.Review` in Lead page layout

### Phase 4: Visibility & Activation
**Step 4.1**: Fix profile/visibility model
- **Issue**: "Profile: applicationVisibilities visible=false"
- **Resolution**: Use PermissionSet or custom permission instead of profile-based visibility
- **Action**: Assign PermissionSet to profiles/users needing flow access

**Step 4.2**: Activate Lead_Review flow
- **Setup path**: Setup → Flows → Find "Lead_Review" → Click "Activate"
- **Verify**: Status shows "Active" and no deployment errors in org logs

## Validation Checklist
- [ ] Lead_Review flow deployed and first screen has navigation enabled
- [ ] Demo_UtilityBar present in CustomApplication Demo
- [ ] PermissionSet prerequisites satisfied before deployment
- [ ] Lead.Review Quick Action created and linked to flow
- [ ] Flow activation succeeds post-deploy
- [ ] Test: Create/open Lead record, verify Quick Action button visible and flow launches

## Rollback
If activation fails: Deactivate flow, investigate org logs, re-deploy with fixes to first screen, re-activate.
