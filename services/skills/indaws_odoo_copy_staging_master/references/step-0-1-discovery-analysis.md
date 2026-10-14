# STEP 0: DISCOVER & STEP 1: ANALYZE

## STEP 0: DISCOVER - Scan repos, addons_path, source code

**Purpose**: Understand what exists locally and in staging BEFORE touching any database.

### 0a. Find and parse addons_path

```bash
grep "^addons_path" "$CFG_PATH"
```

### 0b. Detect Odoo version

```bash
grep "version_info" "$ODOO_SOURCE/odoo/release.py"
# Or via XML-RPC:
python3 -c "import xmlrpc.client; print(xmlrpc.client.ServerProxy('$STAGING_URL/xmlrpc/2/common').version())"
```

Version-specific differences relevant to migration:
- **v17+**: `ir.actions.server` has no `active` field
- **v17+**: `base.automation` has no `sequence` field
- **v17+**: Translation fields stored as JSONB
- **v18+**: JSON2 protocol (`/json/2/`)

### 0c. Scan repos and classify modules

**Delegate to `odoo-module-scanner`** to analyze each custom module found in addons_path directories. The scanner produces a structured inventory of models, fields, views, security rules, dependencies, and controllers per module.

For the migration context, classify each addons_path directory by origin:
- **Odoo base** (odoo/addons, odoo/odoo/addons)
- **Enterprise** (enterprise/)
- **OCA** (OCA/*)
- **Indaws Core** (indaws-odoo/, indaws-odoo-enterprise/)
- **Custom client** (addons-client/, ind_*, specific modules)
- **Third party** (third_party/*, custom/*)
- **Verticals** (vertical-odoo-inbuilt/)

### 0d. Compare with staging installed modules

```python
installed = stg.search_read('ir.module.module', [('state','=','installed')], ['name','shortdesc'])
```

Cross-reference scanner output against staging installed list:
- **Installed in staging AND available locally** -> OK
- **Installed in staging BUT NOT available locally** -> BLOCKER, need repo/code
- **Available locally BUT NOT in staging** -> Ignore

### 0f. Detect hardcoded IDs in custom code

```bash
grep -rn "\.browse(\d" /path/to/custom/modules/ --include="*.py"
grep -rn "'id'.*=.*\d\d" /path/to/custom/modules/ --include="*.py"
```

### 0h. Detect data XML files with noupdate issues

Scan all custom module data/*.xml files for records that will be OVERWRITTEN on `-u all`:

```bash
# Find all data XML without noupdate="1"
grep -rn 'noupdate="0"\|<odoo>' /path/to/custom/modules/*/data/*.xml | grep -v 'noupdate="1"'
```

**Report format:**
```
DATA XML OVERWRITE RISK
=======================
WARNING: These files will OVERWRITE user-modified data on -u all:

  module_a/data/pricing_data.xml (noupdate="0")
    -> Records: automatic.pricing (7 rules with Python code)
    -> RISK: User-modified code, thresholds, discount percentages will be lost

  module_b/data/ir_cron_data.xml (no noupdate attribute)
    -> Records: ir.cron (3 scheduled actions)
    -> RISK: Custom intervals and next_call will be reset to XML defaults

RECOMMENDATION: Change noupdate="0" to noupdate="1" in all files above BEFORE go-live.
If code access available, fix automatically:
  sed -i 's|<odoo>|<odoo noupdate="1">|g' file.xml
  sed -i 's|noupdate="0"|noupdate="1"|g' file.xml
```

### 0g. Generate discovery report

```
DISCOVERY REPORT
================
Client: [name]
Odoo version: [version]
Modules in staging: N installed
Modules available locally: N
Custom modules: N (M installed + K pending)
OCA modules: N
Modules without local repo: N

CUSTOM MODULES TO INSTALL (K):
  - module_name (depends: base, other)
  ...

STUDIO CUSTOMIZATIONS:
  - N models x_*
  - N fields x_studio_*
  - N views, N actions, N menus, N ACLs

HARDCODED IDS DETECTED: N files
  - path/file.py: browse(5) at line 142
  ...
```

**Decision tree:**
- Missing local repos -> STOP, ask user to provide them
- Hardcoded IDs -> Document for post-migration correction
- All OK -> Continue to STEP 1

Log: `log_step(0, "DISCOVER", "completed", details=report_summary)`
Mark checklist: "Repos scanned", "Modules classified", "Report generated"

---

## STEP 1: ANALYZE - Staging vs Production DB comparison

```bash
python3 scripts/analyze.py \
  --staging-url $STAGING_URL --staging-db $STAGING_DB --staging-key $STAGING_KEY \
  --prod-url $PROD_URL --prod-db $PROD_DB --prod-key $PROD_KEY
```

**What to look for:**
- Models with `-N` = missing N records in production
- `ERR` = model doesn't exist (needs module install or Studio import)
- Missing modules = install BEFORE proceeding
- Studio customizations count = need import_studio.py first

**Decision tree:**
- Missing modules -> Install them (STEP 2), then re-analyze
- Studio models missing -> Run import_studio.py (STEP 2)
- Counts reasonable -> Proceed to STEP 3

Log: `log_step(1, "ANALYZE", "completed", details="N models analyzed, M with differences")`
Mark checklist: "Analysis executed", "Differences documented"
