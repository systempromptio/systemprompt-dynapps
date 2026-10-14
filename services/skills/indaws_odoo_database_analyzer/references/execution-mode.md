# Execution Mode

**Default: Agent Teams (parallel execution).** If agent teams are available in the Claude Code installation, split work across parallel agents for faster analysis. If not available, fallback to sequential execution.

## Parallel Agent Distribution

```
Main Agent:          STEP 0 (Data Gathering) → coordinates all agents
                     STEP 4 (Hour Calculation) → needs all classification results
                     STEP 8 (Generate Report) → needs all results

Agent 1 (Scanner):   STEP 1 (Module Scanning)
                     Starts immediately after STEP 0

Agent 2 (Classifier): STEP 2 + STEP 3 (Classification + Version Analysis)
                      Starts after Agent 1 completes

Agent 3 (DB Analyzer): STEP 5 (Database Migration)
                       Starts immediately after STEP 0 if INCLUDE_DB_MIGRATION = Yes
                       Independent from Agents 1-2

Agent 4 (Test Analyzer): STEP 6 + STEP 7 (Functional + Unit Testing)
                         Starts after Agent 2 completes
                         Needs module classifications and LOC data
```

## Sequential Fallback

If agent teams are not available, execute all steps sequentially: STEP 0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8.
