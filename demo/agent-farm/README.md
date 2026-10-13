# Agent Farm end-to-end

Proves the whole Agent Farm path on a local instance: the kit is signed and
published to GHCR, pinned and imported, a user with a PAT starts a run over the
public API, the run's gateway, hook and governance trail lands in Postgres, and
every `/admin/farm*` page renders it.

```bash
export BUNDLE_SIGNING_SEED=...        # or BUNDLE_SIGNING_SEED_FILE=path
./demo/agent-farm/run-all.sh          # all steps, stops at the first failure
FROM=50 ./demo/agent-farm/run-all.sh  # resume (kit already published/imported)
SKIP="10" ./demo/agent-farm/run-all.sh
```

| Step | What it proves |
|------|----------------|
| `00-keys.sh` | the signing seed derives the key recorded in `deploy/kit/known-kits.json` |
| `10-publish-kit.sh` | kit imports `--strict`, validates, packs signed, publishes `:vX.Y.Z` and `:stable` (`SKIP_PUBLISH=1` stops after packing) |
| `20-pin.sh` | the local profile pins `dynapps-agent-farm@stable` (idempotent) |
| `30-start-and-import.sh` | server up; `POST /api/v1/admin/services/refresh` imports the bundle |
| `40-verify-catalog.sh` | `/admin/marketplaces/dynapps-agent-farm` and `/admin/sync` show the kit |
| `50-mint-user.sh` | farm user exists, is in project `odoo-delivery`, holds a PAT |
| `60-farm-sync.sh` | farm sync succeeds; the user lists the four agents |
| `70-run.sh` | a PAT-started run reaches `succeeded` with no failed assertions |
| `80-assert-db.sh` | `ai_requests` with `api_key_id`, `plugin_usage_events` for the plugin, governance decisions |
| `90-dashboard.sh` | every farm page answers 200 and shows the run, agent and user |

Environment: `KIT_DIR` (default `../systemprompt-agent-farm`), `VERSION`,
`GHCR_USER`, `CASE` (default: first case in `farm/evals/odoo_module_reviewer.yaml`),
`FARM_AGENT`, `FARM_MODEL` (override the agent's model), `FARM_USER_EMAIL`, `ADMIN_EMAIL`, `RUN_TIMEOUT` (900 s),
`FARM_ADMIN_AUTH` (an admin PAT instead of the CLI session JWT), `ROTATE=1`
(new PAT), `PROFILE`, `BASE_URL`.

State (tokens, cookie jar, digest, run id) is written beside the scripts and is
gitignored by `.gitignore` here. The dashboard is fetched with a cookie from
`systemprompt-dev-login`, so it only works on a development profile.
Publishing needs `gh auth login` with `write:packages`.
