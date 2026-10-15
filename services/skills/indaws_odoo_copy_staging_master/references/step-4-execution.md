# STEP 4: EXECUTE MIGRATION

## Pre-flight checklist (before executing)

```python
preflight_checks = []

# 1. Odoo is running and responding
try:
    prod.search('res.company', [], limit=1)
    preflight_checks.append(('Odoo responding', True))
except:
    preflight_checks.append(('Odoo responding', False))

# 2. Admin can authenticate
uid = authenticate(prod_url, prod_db, prod_user, prod_key)
preflight_checks.append(('Admin auth', bool(uid)))

# 3. id_mapping.json is clean
mapping = json.load(open('scripts/id_mapping.json'))
preflight_checks.append(('ID mapping clean', mapping == {}))

# 4. Automations disabled on product.attribute.value
auto_pav = prod.search('base.automation', [
    ('model_id.model', '=', 'product.attribute.value'), ('active', '=', True)
])
preflight_checks.append(('PAV automations disabled', len(auto_pav) == 0))

# 5. All custom modules installed
missing = analyze_missing_modules(staging, production)
preflight_checks.append(('Custom modules installed', len(missing) == 0))

# 6. Studio imported
x_models_stg = stg.search_count('ir.model', [('model', 'like', 'x_%')])
x_models_prod = prod.search_count('ir.model', [('model', 'like', 'x_%')])
preflight_checks.append(('Studio models imported', x_models_prod >= x_models_stg))

all_ok = all(v for _, v in preflight_checks)
if not all_ok:
    # ASK USER: "Pre-flight checks failed: [list]. Fix now and re-check, or proceed anyway?"
```

## Dry-run preview (optional but recommended)

```python
for phase in phases:
    result = run_phase(phase, dry_run=True)
    print(f"Phase {phase.number} ({phase.name}):")
    print(f"  Would create: {result.would_create} records")
    print(f"  Would skip: {result.would_skip} records")
    print(f"  Would fail: {result.would_fail} records")
```

If dry-run shows >10% failure rate on any phase, WARN the user.

## Execute phases

```bash
echo "YES" | python3 run_migration.py --live
# Re-run single phase:
echo "YES" | python3 run_migration.py --live --phase N
```

## Per-phase prerequisite check

```python
PHASE_PREREQUISITES = {
    'accounting': {
        'requires_phases': ['company_users'],
        'requires_models': ['res.company'],
        'verify': lambda: prod.search_count('res.company', []) > 0,
    },
    'warehouses': {
        'requires_phases': ['company_users', 'accounting'],
        'verify': lambda: prod.search_count('account.journal', []) > 0,
    },
    'carriers': {
        'requires_phases': ['company_users', 'warehouses'],
        'verify': lambda: (
            prod.search_count('stock.warehouse', []) > 0 and
            prod.search_count('product.product', [('type','=','service')]) > 0
        ),
    },
    'pos': {
        'requires_phases': ['company_users', 'accounting', 'warehouses'],
    },
}
```

## Per-phase validation

After EACH phase, compare what was PLANNED vs what ACTUALLY happened:

```python
def validate_phase_result(phase, plan, actual):
    for model in phase.models:
        expected = plan.get(model, {}).get('staging_count', 0)
        created = actual.get(model, {}).get('created', 0)
        skipped = actual.get(model, {}).get('skipped', 0)
        total = created + skipped
        if total < expected * 0.95:  # Less than 95% coverage
            severity = 'CRITICAL' if total < expected * 0.5 else 'WARNING'
            log_narrative(f"  {severity}: {model} expected {expected}, got {total}")
```

**Decision after each phase:**
- All models match or exceed staging -> Continue
- Some models have <5% difference -> Log warning, continue
- Any model has >10% missing -> STOP, investigate, fix, re-run phase
- Phase crashed -> Check TROUBLESHOOTING, apply fix, re-run

## Auto-correction patterns

| Error pattern | Auto-fix | Max retries |
|--------------|----------|-------------|
| `Invalid field 'X' on model 'Y'` | Remove field X from vals, retry create | 3 |
| `product_id is required` | Create fallback "Delivery" product, retry | 1 |
| `Sendcloud key validation` | Force delivery_type='fixed', retry | 1 |
| `Access denied on x_*` | Create ir.model.access with group_id=NULL, retry | 1 |
| `Transaction aborted` | Use savepoint pattern, retry individual record | Per record |
| `Singleton error on read` | Switch to batch read (10 at a time), retry | 1 |
| `duplicate key value` | Skip record (already exists), continue | 0 |
| `field X does not exist on model` | Filter with fields_get, retry | 1 |
| `not-null constraint` | Add required field with default value, retry | 1 |

## Autonomous decision framework

### AUTO-CONTINUE (no user input needed):
- Record creation fails for <5% of records -> Log warning, continue
- Field doesn't exist -> Filter it out, continue
- Duplicate record -> Skip, continue
- Connection timeout -> Retry up to 3 times
- M2O reference not resolved -> Skip field, log it, continue
- Minor count mismatch (<5%) -> Continue

### AUTO-FIX (apply fix and retry without asking):
- product_id missing on carrier -> Create fallback "Delivery" product
- Sendcloud validation -> Force delivery_type='fixed'
- Access denied on x_* model -> Create ir.model.access with group_id=NULL
- Transaction aborted -> Use savepoint, retry individual record
- Invalid field -> Remove from vals via fields_get, retry
- Singleton error on staging read -> Switch to batch mode (10 at a time)

### ASK USER (stop and present options):
- Module not available in addons_path
- >10% records failing in a phase
- Staging has model that doesn't exist in production at all
- Credentials test fails
- Pre-flight check fails

### LOG FOR LATER (track for final report):
- Hardcoded IDs that couldn't be remapped
- Server action code that references staging-specific data
- Records with staging URLs
- M2M relationships that couldn't be resolved
- noupdate=0 files that need changing

## Error learning within session

Claude maintains `scripts/logs/error_patterns.json`:

```python
def learn_error(error_str, fix_applied, model):
    """Record an error pattern and its fix for future reference."""
    # ...saves to error_patterns.json

def check_known_error(error_str):
    """Check if this error was already encountered and fixed."""
    # ...reads from error_patterns.json, returns last successful fix
```

Log: `log_step(4, "EXECUTE", "completed", details="N/M phases passed")`
Mark checklist: "Phases executed", "Errors fixed", "Data migrated"
