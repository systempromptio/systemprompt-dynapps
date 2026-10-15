# Dyncloud Ops MCP — reference index

The tools of the operations MCP, grouped by what you are doing. Read the file for your
task; the **conventions below apply to every tool**, so the individual pages do not repeat
them.

- **`access-model.md`** — the JIT access engine: categories, stages, grants, who may
  approve what, and the exact refusals. Read it the first time a call comes back denied.
- **`tools-navigation.md`** — find the thing you are going to act on:
  `list_my_companies`, `list_projects`, `list_environments`, `list_databases`,
  `list_target_users`, `list_actions`, `list_release_tags`, and the active context
  (`set_active_context` / `get_active_context` / `clear_active_context`).
- **`tools-read.md`** — look without touching: `platform_env_status`,
  `platform_list_actions`, `platform_job_runs`, `platform_project_health`,
  `platform_list_backups`, `platform_backup_url`, `platform_list_domains`,
  `platform_describe_jobs`, `platform_describe_actions`, `platform_submodules_plan`,
  `platform_submodules_refs`, `platform_deploy_config`.
- **`tools-write.md`** — change something: `platform_deploy`, `platform_lifecycle`,
  `platform_rebuild`, `platform_domains`, `platform_submodules_apply`,
  `platform_backup_create`, `platform_action_control`, `platform_enqueue_action`,
  `platform_rename_project`, `platform_project_deploy_prepare`,
  `platform_project_deploy_commit`, `platform_set_deploy_config`.
- **`tools-destructive.md`** — the ones that cannot be undone:
  `platform_backup_restore`, `platform_destroy_environment`, `docker_ssh_exec`.
- **`workflows/README.md`** — how to **chain** them, one line per recipe so you can choose
  without opening anything.

## Conventions (apply to every tool)

- **Namespaced call.** The real callable is `mcp__<server-name>__list_environments`, and
  the server name is whatever the deployment chose. These pages use short names. Identify
  the operations server by its `platform_*` family; ask the user which to use when more
  than one is connected.
- **Errors are results, not exceptions.** A refusal comes back as
  `{"ok": false, "error_code": "...", "error": "..."}`, often with a `request_hint`
  telling you the fix. Read it and adjust — never retry the identical call.
- **Runs as the calling user.** No superuser anywhere. Odoo record rules and Cloud
  permission codes (`:backup_create:`, `:environment_remove:`, …) apply exactly as they
  would to that person in the web UI. An empty list can mean "you cannot see it", not
  "it does not exist".
- **Two kill-switches make things invisible.** A project or environment with
  `mcp_enabled: False` disappears from `list_projects` / `list_environments` *and* from
  every tool that resolves through them — you get "not found", not "not allowed". The
  cascade is real: an environment is invisible if **either** it or its project is off.
- **Runtime matters.** Every tool page marks docker / k8s / both. A tool that does not
  apply answers `E_NOT_SUPPORTED_FOR_RUNTIME` rather than doing something approximate.
- **A blocked project refuses writes.** `project_blocked: true` in any read means the
  queue is wedged behind a failed action; most writes refuse until
  `platform_action_control` clears it.
- **Enqueued is not done.** Most writes return `action_ids` and end there. The work
  happens asynchronously — poll `platform_list_actions`, and on k8s follow through to
  `platform_job_runs`. Never report a returned `action_ids` as a finished operation.
- **Active context is a convenience for discovery only.** The `list_*` tools fall back to
  it; tools that act on an environment want the id explicitly. With hundreds of live
  environments, a silent fallback on a write is exactly the mistake that must not happen.

## Known limitations, as of this writing

Documented because finding them out by experiment costs an afternoon each.

- **`platform_backup_create` takes no parameters at all.** Only `environment_id` and
  `method`. On k8s you cannot ask for a backup without its filestore
  (`includeFilestore: false`) through this tool.
- **`platform_backup_restore` accepts `params`, and older manager builds drop them.** The
  argument reached the handler, was validated, echoed back as `"mode": "parametrised"` —
  and never travelled to the job, so `jobStorage` and `sourceUrl` were unreachable and the
  sizing silently fell back to its default. Fixed in the build that threads them through
  the dispatcher. If a restore ignores your `jobStorage`, check the manager version before
  debugging the release template.
- **`list_release_tags` returns every tag in one answer**, with no `limit` and no filter
  for obsolete ones. On a mature release source that is a large response for the sake of
  one id.
- **`duration_seconds` in `platform_job_runs` can be negative.** `completed_at` comes from
  the cluster and `started_at` from the manager, so a job that dies while its Job object is
  being created finishes "before" it started. Treat a negative duration as "failed
  immediately", not as data.
