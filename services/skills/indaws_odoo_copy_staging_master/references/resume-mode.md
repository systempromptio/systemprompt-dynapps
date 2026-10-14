# Resume Mode

## Session Resume Logic

When the skill is invoked, ALWAYS check first:

```python
config_path = "scripts/migration_config.json"
progress_path = "scripts/logs/migration_progress.json"

if os.path.exists(config_path):
    with open(config_path) as f:
        config = json.load(f)
    print(f"Found existing migration config for: {config['client_name']}")

    if os.path.exists(progress_path):
        with open(progress_path) as f:
            progress = json.load(f)
        completed = [k for k, v in progress.items() if v["status"] == "completed"]
        failed = [k for k, v in progress.items() if v["status"] == "failed"]
        in_progress = [k for k, v in progress.items() if v["status"] == "in_progress"]
        all_steps = [f"step_{i}" for i in range(11)]
        next_step = next((s for s in all_steps if s not in completed), None)
```

Then ask: "I found an existing migration for **[Client Name]**. Steps completed: [list]. Continue from **STEP N**?"

## Rules for resume:
- Skip completed steps entirely (do NOT re-run)
- Re-run failed steps (they may have been fixed)
- Re-run in_progress steps (they were interrupted)
- Read credentials from `migration_config.json` (do NOT ask again)
- Read Indaws task IDs from config (do NOT recreate tasks)

## Full context reconstruction on resume

Read ALL files in order:
1. `scripts/migration_config.json` - credentials, scope, mode, tracking
2. `scripts/migration_plan.md` - phases planned, expected counts
3. `scripts/logs/migration_progress.json` - steps completed/failed
4. `scripts/logs/migration_narrative.md` - what happened, why, decisions made
5. `scripts/logs/error_patterns.json` - errors encountered and fixes applied
6. `scripts/logs/pending_decisions.json` - items waiting for human decision
7. `scripts/id_mapping.json` - staging-to-production ID mappings

Present summary:
"I'm resuming the migration for **[Client]**. Here's where we are:
**Completed:** Steps 0-4
**Current:** Step 5 - was in progress when session ended
**Errors fixed:** 3 (Invalid field, product_id, Sendcloud)
**Pending decisions:** 2 (DynamoDB table name, SMB paths)
**Records migrated so far:** 1,583
Continue from Step 5?"

## Context compaction handling

1. **Before compaction**: Save current state to all persistent files
2. **After compaction**: Read persistent files to reconstruct full context
3. **Key rule**: NEVER rely on conversation memory for migration state. ALWAYS read from files.

Claude should proactively save state after EVERY significant action.
