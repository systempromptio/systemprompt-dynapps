# Workflows (index)

A single call rarely finishes a real task; the skill is knowing how to chain them. Each
recipe is a concrete sequence with the actual calls and the reasoning behind the order.
Tool details are in `../tools-*.md`; shared conventions in `../README.md`.

Golden thread: **orient → read → act → verify one layer deeper than you think you need.**
The reads are cheap and several of them will change your plan. The verification is not
optional: most writes return `action_ids` and stop, and on k8s the action goes `done` the
moment a job is *launched*.

**This page exists so you can choose a recipe without opening anything.**

## `diagnose.md` — something is wrong, or nothing is happening

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | An environment is not responding | The user says "it's down" and you have no idea why |
| 2 | A job "succeeded" but nothing happened | The chain says `done` and the result disagrees |
| 3 | The queue is stuck | Actions sit `enqueued`, `project_blocked: true` |
| 4 | Backups look fine and are not being taken | The kanban is green and the bucket is empty |

## `backup-restore.md` — data in and out

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Take a backup and prove it landed | Before any risky change, or on request |
| 2 | Hand a backup to someone | They want the file, not a description of it |
| 3 | Restore into an environment | Rolling back, or seeding a test environment |
| 4 | Restore from a URL, or size the job by hand | The source is outside the platform |

## `change.md` — moving an environment forward

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Upgrade an environment to a new release | A release tag is out and an environment should have it |
| 2 | Pin submodules | Custom code needs to move to a new commit |
| 3 | Apply a configuration change on docker | You changed configuration and a restart did nothing |
| 4 | Add or move a domain | A new hostname, or DNS that has not landed |

## `teardown.md` — destroying an environment

| § | Workflow | Open it when |
| --- | --- | --- |
| 1 | Destroy an environment, end to end | A test environment has outlived its purpose |

## Composition cheatsheet

| Goal | Chain |
| --- | --- |
| Find an environment | `list_my_companies` → `list_projects` → `list_environments` |
| Know which runtime you are on | `list_environments` → `project_type` |
| Is this environment healthy? | `platform_env_status` → `health_issues` + `project_blocked` |
| Why did it fail? | `platform_list_actions` → `exc_info` → **k8s: `platform_job_runs`** |
| Did the job actually work? | `platform_job_runs` → `job_state` + `error_message` + `artifacts` |
| Unblock a queue | `platform_action_control { op: "requeue" \| "cancel" }` |
| Elevated call refused | read `request_hint` → `request_mcp_access` → retry → `revoke_mcp_access` |
| Grant came back `pending` | tell the user its `name`, **stop**, poll `get_my_mcp_access` |
| Take a backup | `platform_backup_create` → `platform_job_runs` → `platform_list_backups` |
| Give someone a backup | `platform_list_backups` → `platform_backup_url` (short-lived, bearer) |
| Restore | grant → `platform_backup_restore` → `platform_job_runs` → env back up |
| Upgrade a release | `list_release_tags` (`is_current`!) → `platform_deploy { upgrade_release }` |
| Pin submodules | `platform_submodules_plan` → (`platform_submodules_refs`) → `platform_submodules_apply` |
| Apply docker config | `platform_rebuild` — **not** `platform_lifecycle { restart }` |
| Change DNS | `platform_list_domains` → `platform_domains { create/update }` → `{ apply }` |
| Destroy | grant → `platform_destroy_environment` → `platform_list_actions` |
| Run a shell command | grant (**always pending**) → `docker_ssh_exec` — docker only, no sudo, no session |
