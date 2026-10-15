# Phase 4 — Make the two deployments equivalent

> **docker2k8s — Phase 4 of 7.** Before starting: Phase 3 finished: the Kubernetes project exists and its first environment came up on the generated demo database.
> The router, the safety rules and the Prerequisites live in `SKILL.md`.


This is the phase that decides whether the migrated environment behaves like the original.
Everything here is about **parity**: the same modules at the same commits, and the same
Python dependencies at the same versions.

### 4.0 Which environments migrate — MANUAL, ask the developer

Ask which Docker environments become Kubernetes environments. Not all of them always do:
a stale `test` branch nobody has opened in a year is worth retiring, not migrating.

> **Ask this BEFORE § Phase 3 (`phases/03-create-deploy.md`), not here.** The project wizard's second step is where the
> first environments are declared, so by the time you reach this section the answer has
> already been used. It stays numbered 4.0 because this is where the consequences live —
> branches, parts, dependencies — but the question belongs to the phase before.

Re-state the non-Odoo environments from 1c here as an explicit exclusion, so the answer is
given with them in view.

### 4.1 Branches of the code repo — MANUAL

**One branch per environment, but only where they diverge.** Two Docker environments
running the same parts at the same commits need one branch, not two. Create a second
branch when the manifests actually differ.

> **WARN — a shared branch deploys to every environment that follows it.**
>
> A deploy target is `(code repo, branch, image_key)`, and **several environments can point
> at the same one**. Bumping parts on a branch is therefore *not* an operation on one
> environment: every environment whose deploy config names that target with
> `autodeploy` picks the change up on its own, without anybody choosing it environment by
> environment.
>
> That is the whole point of sharing a branch, and it is also how a change meant for one
> environment reaches three. **Before touching a shared branch, list the environments that
> follow it** and say out loud which ones are about to be deployed. If the change is meant
> for one of them only, that is the moment the branches diverge — split it first.

### 4.2 The parts: `.gitmodules` / `repositories.yml` → `dyn-build.yaml` — LOCAL

**→ `phases/04a-parts.md`.** It is the longest section of the runbook and it is only
needed while the manifest is actually being converted, so it lives in its own file: the
three pin rules and the table of the four combinations, where the commits come from, the
parts that do not travel, the `indawsodoo` → `dynapps-es/mirrors` rewrite, the weekly date
tag for Odoo core / Enterprise / themes, and the `dyncloud-odoo-auth` part that must be
pinned by tag and kept current.

**Do not do this section from memory.** Read that file.

### 4.3 The customer's own modules: the custom repo moves INTO the code repo — LOCAL

The Docker fleet keeps the customer's own Odoo modules in a **separate repository** —
conventionally `addons-<customer>`, cloned on the host next to the code repo and mounted
into the container. Kubernetes has no such second repository: a k8s customer code repo
comes from the skeleton with `repo_type=customer`, which puts modules in
**`parts/custom/`** and already declares the slot in `dyn-build.yaml`:

```yaml
- repo: parts/custom
  source: local
  test_enabled: true
```

So this step is not a conversion — **the content moves in.** Find the repository first:
the mount is in the compose file § 1a (`phases/01-survey.md`) already read, and the real name is the customer's,
not a convention. Then clone it **at the commit the environment runs, not the branch
head** — the drift rule of § 4.2 (`phases/04a-parts.md`) applies here too: the host checkout can be ahead of
anything a manifest pins. Where the pin lives decides where you read it:

| The custom repo appears… | The running commit comes from… |
|---|---|
| pinned in `.gitmodules` / `repositories.yml` | the manifest pin — and then **exclude it from § 4.2 (`phases/04a-parts.md`)'s part list** (see the trap below) |
| only as a clone on the host | `git -C <mount-path> rev-parse HEAD` over `docker_ssh_exec` |

Copy the modules into `parts/custom/` and commit on the code repo's working branch — at
this point in the procedure the branches of § 4.1 have not diverged yet. From that commit
on, **the old `addons-<customer>` repository is no longer the source of truth**: changes
to the customer's modules become ordinary MRs in the code repo. The old repository is not
deleted here — it retires with the rest of the Docker project — but nobody pushes to it
again.

> **GOTCHA — do not let the modules in twice.** If the custom repo is pinned in the
> Docker manifest, an unthinking § 4.2 (`phases/04a-parts.md`) conversion would emit it as a `git` part pointing
> at the old repository — and the same modules would exist twice: as `parts/custom/`
> content *and* as a bumpable remote part that can drift away from the copy. A repository
> whose content moved into `parts/custom/` must not also appear as a part in
> `dyn-build.yaml`.

> **`local` means not bumpable, and that is by design.** `platform_submodules_plan` will
> not list `parts/custom` and the submodules dialog cannot update it: a `local` part has
> no remote to bump. Say this to the customer's developers — until now they pushed to
> `addons-<…>`, and after this step that push lands nowhere that deploys.

### 4.4 The Python dependencies — AUTO over `docker_ssh_exec`, then MANUAL

**Only the requirements the Docker entrypoint generates, not the whole environment.** A
`pip freeze` of the container is the wrong list: it carries the base image's own
dependencies, and pinning those into the k8s code repo would fight the platform image.

**This one genuinely needs the host** — `docker_ssh_exec`, because the answer only
exists inside a running container. What you are after is the extra libraries the
customer's developments pull in, so they end up in the **k8s code repo's
`requirements.txt`**.

```bash
docker exec <odoo-container> sh -c 'ls -la /opt/odoo/requirements*.txt' 2>/dev/null
```

Then diff it against the k8s code repo's `requirements.txt` and reconcile version by
version.

> **UNVERIFIED — one specific thing is missing, and here is exactly what.**
>
> The **input** side is confirmed. `docker-compose.yml` mounts
> `./src/requirements:/home/odoo/requirements`, and that directory carries
> `force_requirements.txt` — the slot for pinning a version by hand. Those are the
> requirements the addon repositories *bring*, not the consolidated list.
>
> What is **not** confirmed is where the entrypoint leaves the file it generates from
> them. That entrypoint lives inside the image — `indaws/odoo-slim:18.0` — and not in
> `deploy-templates`, so it could not be read here. Find it on a running container:
>
> ```bash
> docker exec <odoo-container> sh -c 'ls -la /home/odoo/requirements/'
> docker exec <odoo-container> sh -c 'find / -name "requirements*.txt" -newer /etc/hostname 2>/dev/null'
> ```
>
> **Record the real path in this file on the first migration**, and delete this box.
>
> Do not substitute `pip freeze`: it carries the base image's own dependencies, and
> pinning those into the k8s code repo fights the platform image. The k8s base
> (`odoo-base`) has its own separate mechanism — `requirements-infra.txt` for the
> infrastructure venv — which is not the same list and not the one to copy.

### 4.5 What else has to match — MANUAL

Parity is more than parts and pip. Walk the Docker environment's own configuration and
carry each item across, or decide deliberately that it does not travel:

- **Odoo configuration** — everything in the Docker `asoo.yml` / compose environment that
  changes Odoo's behaviour: workers, limits, `server_wide_modules`, extra config sections.
- **Environment variables** the customer's modules read.
- **Domains** — every FQDN, its redirects, its IP allow-lists (`platform_list_domains` on
  the Docker side lists them).
- **Scheduled work** — Odoo crons come with the database; anything outside it does not.
- **The connector — it is swapped, not carried.** `asoo_connector` goes, `dyncloud_api`
  and `dyncloud_auth` come. That is a step of its own with an ordering trap in it: **§
  6.3**. The mechanism also changes underneath: a Kubernetes environment does **not** get
  the manager's public keys the way Docker does. Docker *pulls* them from the JWKS
  endpoint on a cron; Kubernetes is *given* them as files by an ExternalSecret, runs in
  disk mode, and disables that cron. The Docker variables (`BACKEND_URL`, `ENV_UUID`,
  `LINEAGE_NAME`) stop being the mechanism — harmless if left behind, but no longer what
  makes it work.
  → `dyncloud-migration` § *references/modes/k8s.md* owns this in detail.

### 4.6 Deploy the environments and wire their deploy targets — AUTO/MANUAL

Deploy as many k8s environments as 4.0 chose, and give each one its deploy config.

**Deploy them ONE AT A TIME, closing each action before starting the next**
(§ CLOSE THE ACTION (`SKILL.md`)). They share a project, so they share a queue: fire three deploys in a row and a
failure in the first leaves the other two `enqueued` behind it — reporting success at the
moment you asked, doing nothing afterwards. Two environments that "did not deploy" and one
that failed is the same single failure, read three times.

The wiring below (`platform_set_deploy_config`) is a plain ORM write and enqueues nothing —
it returns `changed`, not `action_ids`, so it needs no closing.

| Stage | `deploy_policy` |
|---|---|
| development / staging | `autodeploy` |
| production | `manual` |

The default is `manual`, so staging and development are the ones that need changing.

**AUTO**, on any manager recent enough to expose these two tools:

- `platform_deploy_config(environment_id=…)` — or `project_id=…` to see every environment
  in one call — reads the wiring: target, policy and current image per `image_key`. An
  environment with no rows answers `rows: []` with a note saying so, which is not the same
  as failing to read it.
- `platform_set_deploy_config(environment_id=…, image_key="", deploy_policy=…,
  deploy_target_id=…)` writes it.

Two things it deliberately will not do. It **never creates** a row — a missing one answers
`E_DEPLOY_CONFIG_NOT_FOUND` and lists the `image_key`s that exist, because creating the
wiring belongs to the deploy wizard that owns the invariants tying a target to a
repository. And a write that moved nothing answers `changed: {}` rather than a success
indistinguishable from a real change.

> **Read this BEFORE pushing to any shared branch** — the warning in § 4.1 is this
> field. If `deploy_policy` is `autodeploy`, the push IS the deploy. `autodeploy_count`
> comes back at the top level so you can branch on one number instead of walking the
> list.
>
> On an older manager the tools are simply absent: do it in the UI, and say so rather
> than reporting a step as automated that is not.

