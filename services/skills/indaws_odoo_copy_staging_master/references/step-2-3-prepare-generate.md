# STEP 2: PREPARE & STEP 3: GENERATE PHASES

## STEP 2: PREPARE - Install modules, import Studio, fix DB

### 2a. Install missing custom modules

```bash
python3 odoo-bin -c $CFG -i module1,module2,module3 --stop-after-init
```

If bulk install fails, install one by one to identify dependency issues.

### 2b. Import Studio (requires ORM access)

```bash
PYTHONPATH=$ODOO_SOURCE python3 scripts/import_studio.py \
  --cfg $CFG --db $PROD_DB \
  --staging-url $STAGING_URL --staging-db $STAGING_DB --staging-key $STAGING_KEY
```

**After import_studio.py, restart Odoo** so new models/fields are loaded.

### 2b-REPLAY. Studio Data "Replay" Strategy (preferred over raw copy)

**Problem with raw copy**: Reading Studio records from staging and creating them in production with the same field values causes ID mismatches. `inherit_id` references break, action references point to wrong IDs, and m2m relations fail.

**Solution**: REPLAY the Studio creation sequence, resolving every reference by natural key instead of ID.

```
STUDIO REPLAY SEQUENCE:

1. CREATE MODELS (x_*) in dependency order
   - Read x_* models from staging via ir.model
   - Create each model in production (state='manual')
   - Map staging model ID -> production model ID

2. CREATE INVERSE MANY2ONE FIELDS FIRST
   - These are needed BEFORE one2many fields can reference them
   - Create in the target x_* models
   - Map staging field ID -> production field ID

3. CREATE FIELDS (x_studio_*) with ID resolution
   - For many2one fields: resolve 'relation' model by model name (not by ID)
   - For one2many fields: resolve 'relation_field' by field name in target model
   - For selection fields: create selection values
   - Handle delegation inheritance:
     product.product -> product.template
     res.users -> res.partner

4. CREATE ACLs (ir.model.access)
   - Resolve model_id by model name (NOT staging ID)
   - Resolve group_id by xml_id (NOT staging ID)

5. CREATE VIEWS IN DEPENDENCY ORDER
   - First: primary views (mode='primary')
   - Then: extension views (mode='extension'), resolving inherit_id by:
     a. Find parent view in production by KEY field (not by ID)
     b. If no key, find by name + model combination
   - For each view, replace any staging IDs in arch XML:
     - action IDs in <button name="ACTION_ID" type="action"/>
     - Replace with production action ID found by name

6. CREATE WINDOW ACTIONS (ir.actions.act_window)
   - Resolve res_model by model name
   - Resolve view_id by key or name+model
   - Resolve search_view_id similarly

7. CREATE MENUS (ir.ui.menu)
   - Resolve parent_id by finding parent menu by name path
   - Resolve action reference by action name
   - Create in hierarchy order (parents first, then children)

8. LINK EVERYTHING
   - Update views that reference actions with production action IDs
   - Update menus that reference actions with production action IDs
```

**Key principle: NEVER copy a staging ID into production. ALWAYS resolve by natural key (name, code, xml_id, key).**

Example resolution for inherit_id:
```python
# WRONG: copy inherit_id=457 from staging
# RIGHT:
parent_view_staging = stg.read('ir.ui.view', [457], ['key', 'name', 'model'])
if parent_view_staging['key']:
    parent_prod = prod.search('ir.ui.view', [('key', '=', parent_view_staging['key'])])
else:
    parent_prod = prod.search('ir.ui.view', [
        ('name', '=', parent_view_staging['name']),
        ('model', '=', parent_view_staging['model']),
        ('mode', '=', 'primary'),
    ])
# Use parent_prod[0] as the inherit_id in production
```

### 2c. Pre-migration fixes

```bash
# Reset admin password
psql $PROD_DB -c "UPDATE res_users SET password='admin' WHERE id=2;"

# Disable automations that block attribute value creation
psql $PROD_DB -c "UPDATE base_automation SET active=false WHERE model_id IN (SELECT id FROM ir_model WHERE model='product.attribute.value');"

# Clean ID mapping
echo '{}' > scripts/id_mapping.json

# Ensure DB owner matches Odoo db_user
psql postgres -c "ALTER DATABASE $PROD_DB OWNER TO odoo;"
```

Log: `log_step(2, "PREPARE", "completed", details="N modules installed, Studio imported")`
Mark checklist: "Modules installed", "Studio imported", "DB prepared"

---

## STEP 3: GENERATE PHASES - Create migration scripts based on analysis

**This is the most important step.** Claude must plan the exact migration phases based on what STEP 0 and STEP 1 revealed.

### 3a. Build the phase plan

Using the analysis results, classify all models with data into functional groups:

```python
phase_plan = []
analysis = load('analysis_report.json')

# Group 1: ALWAYS - Company and Users
phase_plan.append({
    'number': 0, 'name': 'company_users',
    'models': ['res.company', 'res.users', 'res.partner'],
    'condition': 'ALWAYS',
})

# Group 2: IF account.journal > 0
if analysis.get('account.journal', {}).get('staging', 0) > 0:
    phase_plan.append({
        'number': 1, 'name': 'accounting',
        'models': ['account.journal', 'account.tax', 'account.fiscal.position', ...],
    })
# ... etc for each functional area
```

### 3b. For each phase, generate a script file

Each script must follow this pattern:
```python
#!/usr/bin/env python3
"""Phase NN: Description"""
from utils import OdooRPC, IDMapper, safe_create, safe_search_read, find_by_field

def run(dry_run=True):
    stg = OdooRPC(*get_connection('staging'), dry_run=False)
    prod = OdooRPC(*get_connection('production'), dry_run=dry_run)
    mapper = IDMapper()
    # 1. Read from staging (with safe_search_read)
    # 2. For each record: search in production by natural key
    # 3. If exists: skip (or update in incremental mode)
    # 4. If not exists: create with safe_create
    # 5. Save mapping
    mapper.save()
    return 0
```

### 3c. Apply known particularities per model

Read `templates/PHASE_GENERATION.md` and `references/lessons-learned.md` for:
- Carriers need product_id fallback
- ftp.instance matches by host
- Sendcloud carriers forced to fixed
- base.automation always inactive
- etc.

### 3d-3f. Orchestrator, plan presentation, persistence

- Generate `run_migration.py` orchestrator (dependency checking, error handling per phase, summary)
- Present phase plan to user for approval via AskUserQuestion
- Save the complete plan to `scripts/migration_plan.md` (single source of truth)

### 3g. User approval gate (MANDATORY)

```
Question: "Migration plan ready. Review and approve?"
Options:
  - "Approve and execute (Recommended)"
  - "Approve with dry-run first"
  - "Edit plan"
  - "Save plan and stop"
```

**NEVER execute real writes without explicit user approval of the plan.**

Log: `log_step(3, "GENERATE", "completed", details="N phase scripts generated")`
Mark checklist: "Scripts generated", "Reviewed by Claude"
