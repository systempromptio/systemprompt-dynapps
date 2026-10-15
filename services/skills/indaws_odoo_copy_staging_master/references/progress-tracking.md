# Progress Tracking, Prerequisites, and Task Management

## Prerequisite Check

Before starting any migration step, verify ALL prerequisites:

```bash
# 1. python3 available
python3 --version

# 2. psql available
psql --version

# 3. Test staging connection via XML-RPC
python3 -c "
import xmlrpc.client
common = xmlrpc.client.ServerProxy('$STAGING_URL/xmlrpc/2/common', allow_none=True)
uid = common.authenticate('$STAGING_DB', '$STAGING_USER', '$STAGING_KEY', {})
print(f'Staging auth: uid={uid}')
assert uid, 'Staging authentication FAILED'
"

# 4. Test production connection (if Odoo running)
python3 -c "
import xmlrpc.client
common = xmlrpc.client.ServerProxy('$PROD_URL/xmlrpc/2/common', allow_none=True)
uid = common.authenticate('$PROD_DB', '$PROD_USER', '$PROD_KEY', {})
print(f'Production auth: uid={uid}')
assert uid, 'Production authentication FAILED'
"

# 5. .cfg file exists and addons_path readable
test -f "$CFG_PATH" && grep "^addons_path" "$CFG_PATH" || echo "ERROR: .cfg not found or no addons_path"

# 6. Check venv / required packages
python3 -c "import xmlrpc.client; print('xmlrpc OK')"
```

**If any check fails:**
- `python3` missing: STOP, tell user
- `psql` missing: WARN, some steps will use XML-RPC instead of SQL
- Staging auth fails: STOP, credentials are wrong
- Production auth fails: WARN if Odoo not started yet (normal for fresh DB)
- .cfg missing: Ask user for the path
- Packages missing: Install with pip

## Indaws Task Creation (if tracking enabled)

### Authenticate with Indaws

```bash
# Resuelto en runtime — ver references/shared/skill-path-resolution.md
ODOO_PILOT_SCRIPTS="${ODOO_PILOT_SCRIPTS:-$(ls -d "$HOME/.claude/plugins/cache"/*/*/*/skills/odoo-pilot/scripts \
    "$HOME/.config/opencode/skills/odoo-pilot/scripts" 2>/dev/null | head -1)}"
bash "$ODOO_PILOT_SCRIPTS/auth.sh" "$INDAWS_URL" "$INDAWS_DB" "$INDAWS_USER" "$INDAWS_KEY"
```

### Create parent GO-LIVE task (or use existing)

```bash
bash "$ODOO_PILOT_SCRIPTS/create_record.sh" \
  "$INDAWS_URL" "$INDAWS_DB" "$INDAWS_USER" "$INDAWS_KEY" \
  "project.task" \
  "{\"name\": \"GO-LIVE $CLIENT_NAME\", \"project_id\": $PROJECT_ID, \"description\": \"Staging to production migration - $CLIENT_NAME\"}"
```

### Create subtasks for each phase

```bash
PHASES=(
  "STEP 0: Discover - Scan repos and modules"
  "STEP 1: Analyze - Compare staging vs production"
  "STEP 2: Prepare - Install modules and import Studio"
  "STEP 3: Generate - Create migration scripts"
  "STEP 4: Execute - Run migration phases"
  "STEP 5: Credentials - Copy real credentials"
  "STEP 6: Extra Data - Migrate taxes, rules, automations"
  "STEP 7: Views - Migrate Studio, custom and website views"
  "STEP 8: Post-Migration - Deactivate automations"
  "STEP 9: Audit - Final audit"
  "STEP 10: Report - Summary and pending items"
)

for phase in "${PHASES[@]}"; do
  bash "$ODOO_PILOT_SCRIPTS/create_record.sh" \
    "$INDAWS_URL" "$INDAWS_DB" "$INDAWS_USER" "$INDAWS_KEY" \
    "project.task" \
    "{\"name\": \"$phase\", \"project_id\": $PROJECT_ID, \"parent_id\": $PARENT_TASK_ID}"
done
```

### Checklist items per phase

| Phase | Checklist items |
|-------|----------------|
| STEP 0 | "Repos scanned", "Modules classified", "Report generated" |
| STEP 1 | "Analysis executed", "Differences documented" |
| STEP 2 | "Modules installed", "Studio imported", "DB prepared" |
| STEP 3 | "Scripts generated", "Reviewed by Claude" |
| STEP 4 | "Phases executed", "Errors fixed", "Data migrated" |
| STEP 5 | "Credentials copied", "0 PLACEHOLDERs" |
| STEP 6 | "Taxes migrated", "Rules migrated", "Automations created inactive", "Sequences migrated", "Mail servers created" |
| STEP 7 | "Studio views migrated", "Custom views migrated", "Website migrated", "Website menus migrated" |
| STEP 8 | "Automations deactivated", "Permissions verified" |
| STEP 9 | "Audit complete", "Differences < 5%" |
| STEP 10 | "Report delivered", "Pending items listed" |

### Create checklist items

```bash
for item in "${CHECKLIST_ITEMS[@]}"; do
  bash "$ODOO_PILOT_SCRIPTS/create_record.sh" \
    "$INDAWS_URL" "$INDAWS_DB" "$INDAWS_USER" "$INDAWS_KEY" \
    "xx.checklist.item" \
    "{\"name\": \"$item\", \"task_id\": $TASK_ID, \"state\": \"pending\"}"
done
```

Store ALL task and checklist IDs in `migration_config.json` under the `tasks` key.

## Execution Rules for ALL Steps

1. **After each step completes**, mark the corresponding Indaws checklist items (if tracking enabled)
2. **After each step**, log the result to `scripts/logs/migration_progress.json`
3. **If a step fails**, log the error and continue to the next step (do NOT stop the entire pipeline)
4. **Before each step**, check if it was already completed (from progress log)

## Progress Logging

```python
import json, os, datetime

def log_step(step_num, step_name, status, details="", error=""):
    os.makedirs("scripts/logs", exist_ok=True)
    log_path = "scripts/logs/migration_progress.json"
    progress = {}
    if os.path.exists(log_path):
        with open(log_path) as f:
            progress = json.load(f)
    progress[f"step_{step_num}"] = {
        "name": step_name,
        "status": status,  # "completed", "failed", "skipped", "in_progress"
        "timestamp": datetime.datetime.now().isoformat(),
        "details": details,
        "error": error
    }
    with open(log_path, "w") as f:
        json.dump(progress, f, indent=2)
```

## Narrative Migration Log

Besides the JSON progress file, maintain a human-readable `scripts/logs/migration_narrative.md`:

```python
def log_narrative(message):
    os.makedirs("scripts/logs", exist_ok=True)
    with open('scripts/logs/migration_narrative.md', 'a') as f:
        f.write(f"\n**[{datetime.now().strftime('%Y-%m-%d %H:%M')}]** {message}\n")
```

On session resume, Claude MUST read `migration_narrative.md` to understand full context, not just the JSON status.

## Checklist Marking Function

```python
import subprocess, json

# Resuelto en runtime — ver references/shared/skill-path-resolution.md
ODOO_PILOT = os.environ.get("ODOO_PILOT_SCRIPTS") or next(
    (d for p in (f"{Path.home()}/.claude/plugins/cache/*/*/*/skills/odoo-pilot/scripts",
                 f"{Path.home()}/.config/opencode/skills/odoo-pilot/scripts")
       for d in sorted(glob.glob(p))), None)

def mark_checklist(config, step_num, item_name):
    if not config.get("indaws", {}).get("enabled"):
        return
    tasks = config.get("tasks", {}).get("steps", {}).get(str(step_num), {})
    checklist_ids = tasks.get("checklist_ids", {})
    check_id = checklist_ids.get(item_name)
    if not check_id:
        return
    indaws = config["indaws"]
    subprocess.run([
        "bash", f"{ODOO_PILOT}/update_record.sh",
        indaws["url"], indaws["db"], indaws["user"], indaws["api_key"],
        "xx.checklist.item", str(check_id),
        json.dumps({"state": "done",
                    "date_done": datetime.now().strftime("%Y-%m-%d")})
    ], capture_output=True)
```

## Parallel Checklist Marking

When Indaws tracking is enabled, mark checklist items after EACH significant action:

```bash
bash "$ODOO_PILOT_SCRIPTS/update_record.sh" \
  "$INDAWS_URL" "$INDAWS_DB" "$INDAWS_USER" "$INDAWS_KEY" \
  "xx.checklist.item" "$CHECK_ID" \
  "{\"state\": \"done\", \"date_done\": \"$(date +%F)\"}"
```

**Important**: If Indaws connection fails at any point, log a warning and continue the migration. Never let tracking failures block the actual migration work.
