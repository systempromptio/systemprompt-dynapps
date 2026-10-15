# Phase 2 — Rename the Docker project

> **docker2k8s — Phase 2 of 7.** Before starting: Phase 1 finished and its VERIFY list clean — in particular, the project's action queue is not blocked.
> The router, the safety rules and the Prerequisites live in `SKILL.md`.


**MANUAL, through the DevOps MCP.** The new Kubernetes project takes the customer's real name;
the Docker one moves out of the way by taking a `-docker` suffix. Names are unique, so
this happens before the k8s project is created.

`platform_rename_project` calls the one method the UI also calls. It needs the
**`Project Rename` group** and the `:project_settings:` permission — the tool does not
`sudo()`, so both apply exactly as they would to a human.

**Rename BOTH the display name and the technical name.** The whole point is to free the
slug so the Kubernetes project can take the customer's real name in full, and
`technical_name` is what the uniqueness check looks at.

`_rename` writes `technical_name` explicitly on every call, defaulting to
`slugify(new_name)`. There is no "rename the display name only" — to keep a technical
name you must pass the old one back in. Here you do not: pass `<customer>-docker` for
both.

> **This is safe for the Docker backups, and it was checked rather than assumed**
> (2026-08-21, against the live records):
>
> - The bucket is the **stored** `cloud.storage.name`. `create_storage()` returns on its
>   first line when the project already has one, so the name is never recomputed from
>   `technical_name`.
> - The S3 key prefix comes from the provider's `auto_path` / `manual_path`, and
>   **none of the three live providers uses `{project}`** — Docker's are
>   `BACKUPS/AUTOMATIC/{environment}/` and `BACKUPS/MANUAL/{environment}/`.
>
> In the whole of `cloud_docker` there is exactly **one** reference to
> `project_id.technical_name`, and it is that template interpolation. No container name,
> no path on the host, no repository URL depends on it.

> **The uniqueness check runs on rename, not on create.** `_rename` refuses a technical
> name another project already holds; `_compute_technical_name` does not look. So this
> rename must come BEFORE the Kubernetes project exists — create it first and both
> projects can end up sharing a slug, silently.

**One thing can stop you, and it is not about backups.** `_rename` needs the
**`Project Rename` group**, which `:project_settings:` does not grant on its own, and
**it differs per cluster**: on 2026-08-21 the group had zero members in **production**,
while a rename over MCP on **es-dev-01 went through with no friction**. Do not assume
either way — try it, and if it raises `AccessError`, that is what happened and somebody
has to grant the group. Only `env.su` bypasses it, so a migration script works where a
human or an MCP call does not.

> **Only the Docker project is ever renamed.** The Kubernetes one is CREATED with the
> customer's real name in § Phase 3 (`phases/03-create-deploy.md`) — that is the whole reason this step frees the slug.
> Nothing in this runbook renames a k8s project, so no k8s rename constraint applies
> here.

**VERIFY:** `list_projects` shows the renamed project; `platform_list_backups` still lists
the historical backups under the same bucket and prefix.

