# Night audit end-to-end

Proves the nightly accounting audit on the agent farm against a live instance
(default the staging tenant, `https://dynapps.systemprompt.io`): the
`dynapps-night-audit` farm runs `accounting_night_audit` as Ed, reads Odoo only
through the hosted `dynapps-odoo` MCP, writes the dashboard and the 11-section
report, publishes a summary to DynBrain, and a morning Claude Code session
through the bridge picks it up and proposes one fix.

```bash
export ED_PAT=...                       # Ed's PAT on the target; never written to disk
systemprompt admin session switch production && systemprompt admin session login
./demo/agent-farm/run-all.sh            # 00 → 40, then 90 (always)
APPROVE_FIX=1 ./demo/agent-farm/run-all.sh   # also apply the proposed fix
FROM=30 ./demo/agent-farm/run-all.sh    # re-assert the last run
SKIP="40" ./demo/agent-farm/run-all.sh  # no bridge-configured claude here
```

| Step | What it proves |
|------|----------------|
| `00-preflight.sh` | target up; Ed exists; his `dynapps-odoo` and `dynbrain` grants are on file and not revoked; he is entitled to the farm marketplace, both servers and `dynapps-farm`; his PAT lists the agent |
| `10-run.sh` | the `farm_scheduled_run` job, run remotely on the tenant, queues the run as Ed with `triggered_via = scheduler` (`RUN_VIA=api` launches over the API with Ed's PAT instead, recorded as `api`; set `TRIGGER=api` for 30) |
| `20-wait.sh` | the run reaches `succeeded`; run, report and events are saved under `.state/` |
| `30-assert.sh` | outputs hold `out/dashboard.md`, `out/DELIVERY.md` and `out/informe_auditoria_Dynapps_YYYY_MM.md`; the eval passed; every `dynapps-odoo` call was a read and governance saw no write attempt; `ai_requests` and MCP trace rows exist; a `dynbrain` `docs_create` succeeded |
| `40-followup.sh` | `claude -p` as Ed fetches the DynBrain article and proposes ONE fix with no write tool; with `APPROVE_FIX=1` the session is resumed with `orm_write` only, and the write is found in `mcp_tool_executions` and `governance_decisions` |
| `90-report.sh` | `evals/night-audit/<date>/metrics.json` and `report.md`: duration, turns, tokens, cost, MCP read/write per server, governance warnings, artifacts, delivery, eval score, pass/fail per assertion |

Environment: `TARGET` (or `BASE_URL`), `PROFILE` (default `production`),
`ED_EMAIL`, `ED_PAT`, `FARM_ID`, `FARM_AGENT`, `CASE`, `TRIGGER`,
`RUN_VIA` (`scheduler` or `api`), `RUN_TIMEOUT` (2400 s), `DYNBRAIN_PARENT_ID` (where the summary is published),
`APPROVE_FIX`, `FOLLOWUP_MODEL`, `ODOO_TOOL_PREFIX`, `BRAIN_TOOL_PREFIX`,
`OUT_DIR`, `LOG_DIR`, `FROM`, `SKIP`.

Database checks go through `systemprompt infra db query` with the profile's
admin session, so the production profile needs `database.external_db_access:
true` (see CLAUDE.md, "Operating on production"). The farm itself, its agent
and eval live in `systempromptio/systemprompt-agent-farm` (`farm/`).
