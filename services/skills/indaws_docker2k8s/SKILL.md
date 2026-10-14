---
name: indaws-docker2k8s
description: >
  Migrates an Odoo customer from a legacy Docker deployment to Kubernetes on Dyncloud:
  survey and warnings, renaming the Docker project aside, building the k8s code
  repo so both deployments carry the SAME parts, custom modules and Python dependencies,
  deploying and health-checking the new environments, moving the data through S3, and the
  production cutover, including the connector swap that follows every restore. Covers what is
  automatic through the two Dyncloud MCP surfaces — DevOps for the platform, Proxy for the
  customer's database — what a developer does by hand, what only the customer can answer, and
  the hard stops where the migration must pause.
  Load the `dyncloud-ops-mcp` skill alongside it: every automated step here is a call to
  that MCP.
  Use it when someone says: "migrate this customer to kubernetes", "move X from docker to
  k8s", "docker2k8s", "pasar un cliente a kubernetes", "migrar de docker a k8s", or asks
  what a Docker-to-Kubernetes migration involves on Dyncloud.
version: 0.7.2
# NO allowed-tools ON PURPOSE. It was declared here and listed only
# [Read, Bash, Grep, Glob, Edit, Write] — which forbade AskUserQuestion (the very
# first instruction below is to ask one) and, worse, every mcp__* tool, in a runbook
# whose AUTO steps ARE MCP calls. It could not have worked, and it failed on the
# opening question the first time it ran (2026-08-21).
# An allowlist cannot fix this either: the MCP client prefix is chosen by whoever
# configures it (§ Prerequisites), so there is no name to allow.
effort: high
model: opus
---

# Docker → Kubernetes migration

Moves one customer from the legacy Docker fleet to Kubernetes. The customer keeps their
data, their modules and their URLs; everything underneath changes at once — build,
runtime, storage, ingress and the connector.

> **This procedure has never been run end to end as written.** It is assembled from the
> code that implements each step and from the incidents that shaped it. Treat every
> **VERIFY** as mandatory and every **STOP** as a real stop. When something does not match
> what is written here, believe the platform and fix the page.

## Who does what

Every step is tagged:

| Tag | Meaning |
|---|---|
| **AUTO** | Driven through a Dyncloud MCP by the assistant — the **DevOps** server, except § 6.3 (`phases/06-move-data.md`) which is the **Proxy** (§ Prerequisites identifies both by capability) |
| **LOCAL** | The assistant does it off-platform — a git clone, reading files, writing YAML. No MCP, no host |
| **MANUAL** | A developer does it — a commit, a file, a decision |
| **CUSTOMER** | Only the customer can answer or act; ask early, it has lead time |
| **STOP** | Do not continue. Report and wait for a human decision |
| **WARN** | Proceed, but say it out loud and write it down |

**AUTO does not mean unattended.** Two of the categories it uses are gated on a human who
is not the assistant:

- **`docker_ssh_exec`** (§ 1a (`phases/01-survey.md`), 4.3, 4.4, 6.0) needs a JIT grant on `ssh_exec`, `by_request` in
  every stage. The requester cannot approve it from the same channel they asked with — an
  MCP call carries `mcp_call` and degrades to the strict rule, so the signature happens in
  a browser. It also requires `:docker_terminal:` on the environment and refuses outright
  while `cloud.ssh_close_connections` is disabled.
- **`platform_backup_restore` and `platform_destroy_environment`** (§ 6.1 (`phases/06-move-data.md`)) sit in
  `platform_destructive`, same shape.

**Ask for those grants at the START of the phase that needs them**, not at the line that
uses them. An assistant that discovers it needs a signature halfway through a data move
has already stopped.

## CLOSE THE ACTION — the rule that governs every AUTO step

**A tool that answers `action_ids` has QUEUED work, not done it.** `ok: true` means
*accepted*, never *finished*. Between that answer and the next call there is a queue, and
this platform's queue has a hard rule: **a project whose LAST FINISHED action failed runs
nothing new**. Everything enqueued after it sits `enqueued` forever.

That is how it fails in practice, and it is worth reading twice because the shape is
deceptive: a commit deploy fails, a backup is asked for immediately after, the backup
answers `ok: true` — and never runs. Nothing reports an error at the moment you made the
mistake. It surfaces later, somewhere else, as "this is not progressing" (test-bck-docker,
es-dev-01, 2026-08-24).

**So, after every call that returns `action_ids`, and BEFORE enqueueing anything else on
that project:**

1. `platform_list_actions(environment_id=…)` — read **`effective_state`**, never the raw
   `state`. The raw one is vacuumed away and lies.
2. Read **`project_blocked`** in the same answer. `true` = the queue is wedged and nothing
   new will run.
3. **Still running? Wait and read again. Do NOT queue on top of it.**
4. **Failed?** `detail=true` for the payload, plus `exc_info` / `terminal_exc_name` /
   `terminal_exc_message` for the reason. Note `exc_info_available: false` means *the
   trace existed and is lost*, NOT *there was no error*.
5. **`done` is not enough when a k8s job is involved.** The chain's last step is "launch
   the job", and launching is that step's whole job — so `done` says it started, not that
   it worked. `platform_job_runs` is the layer that knows. That exact gap hid a broken
   backup for eight days.

> **STOP on `project_blocked: true`.** Do not enqueue a backup, a deploy, a restore or a
> destroy "to keep moving". Resolve first: `platform_action_control` requeues the chain
> (re-rendered from the environment's CURRENT configuration, so a fixed cause is picked
> up) or cancels it so the queue moves. Requeue of a `k8s_destroy_*` action is a known
> platform defect and returns a typed error — that one escalates to a human.

Every step below that enqueues anything ends with **"close the action"**. It means this,
and it is not optional.

## Prerequisites

- **LOAD THE `dyncloud-ops-mcp` SKILL FIRST — before the first `platform_*` call.**

  Every **AUTO** step in this runbook is a call to the Dyncloud operations MCP, and that
  skill is where that server is documented: the full tool reference, the JIT grant model
  (which categories are gated on which stage, who may approve what, and the exact
  refusals), the three-layer state story, and how a blocked project queue behaves. **This
  page does not repeat any of it** — it says *which* call to make at each step and assumes
  you know how the server works.

  So load it at the start, not at the first refusal. The migration touches `platform_ops`,
  `platform_destructive` and `ssh_exec`, and two of those need a signature from a human
  who is not you — which is a wait to schedule, not an error to hit halfway through a data
  move.

  > If that skill is unavailable, say so and continue with care: the calls in here still
  > work, but the failure modes it documents — `E_CATEGORY_NEEDS_GRANT` on staging as well
  > as production, a job that reports `done` while it failed, `cloud.image` ids that are
  > not `cloud.build` ids — are exactly the ones this runbook will hit.

- **BOTH Dyncloud MCP servers — each identified by what it exposes, never by its name.**

  Two Dyncloud MCP surfaces exist and this runbook needs **both**, at different moments.
  **DevOps** carries the `platform_*` tools that drive the platform from outside.
  **Proxy** reaches INTO a customer's Odoo over ORM and is what uninstalls
  `asoo_connector` after each restore (§ 6.3 (`phases/06-move-data.md`)). It travels *through* `dyncloud_api`, so it
  only reaches an environment once that module is installed — which the platform now does
  at boot. Each person names their client connection whatever they like, and the same
  server has different names on different machines — so **there is no name to look for.**
  Identify each by capability:

  > **DevOps** is the server whose tool list contains **`platform_env_status`**. Its
  > prefix is the one for every AUTO step that acts on the platform.
  >
  > **Proxy** is the server whose tool list contains **`admin_uninstall_module`** (and the
  > `orm_*` / `odoo_*` family). Its prefix is the one for § 6.3 (`phases/06-move-data.md`).

  Asking one to do the other's job wastes a turn discovering it cannot: DevOps has no ORM,
  and Proxy knows nothing about deployments — and it reaches nothing at all on an
  environment that has not yet booted with `dyncloud_api` installed.

  **If no connected server exposes `platform_env_status`**, the DevOps server is not
  connected — or your user is not published on it, which fails the same way. Stop and ask;
  do not fall back to the proxy — it cannot do any of this. **If none exposes
  `admin_uninstall_module`**, say so at the START rather than at § 6.3 (`phases/06-move-data.md`): the connector swap
  then needs the Odoo UI or a shell, and that is a different plan for the cutover window.

  **If MORE THAN ONE does, stop and ask which.** That is not a redundancy, it is two
  different clusters — one probably production. Nothing in the tool list says which is
  which. `list_projects` will tell you (the customers differ), but the choice is a human's
  and it belongs at the start, not at the first write.

- **The target is es-prod-01 unless somebody says otherwise.** This runbook migrates real
  customers, and real customers live on the production manager:

  | | Manager | Releases repo |
  |---|---|---|
  | **default** | `dyncloud.infra.dynapps.es` (es-prod-01) | `infra-es/dyncloud/releases` |
  | only if asked | `dyncloud.dev01.infra.dynapps.es` (es-dev-01) | `infra-es/dyncloud/releases-dev` |

  Do not read a manager version number off this page and compare it with what you find —
  any version written in a runbook is stale by the time it is read. Take the platform as
  it answers. When a tool this runbook needs is simply absent, that manager is older than
  the feature; say so plainly and use the UI for that step rather than reporting an
  automated step that was not.

  > **SAY WHICH MCPs YOU ARE ABOUT TO USE, BEFORE THE FIRST CALL.** Name the DevOps and
  > Proxy connections you resolved above **and the manager host each one points at**, and
  > state that you will work on es-prod-01. Developers commonly have the es-dev-01 servers
  > configured too, sometimes *only* those — and a migration that quietly runs against
  > dev01 looks like it is working right up to the moment somebody goes looking for the
  > customer on the wrong cluster. One line at the start, before anything is created:
  >
  > *"I will use `<devops-prefix>` (DevOps → `<host>`) and `<proxy-prefix>` (Proxy →
  > `<host>`), working on es-prod-01. Say so now if that is not what you want."*
  >
  > If the resolved host is **not** the production manager and nobody asked for that,
  > **STOP and confirm** rather than continuing on the cluster that happened to be
  > connected.
- A JIT grant for `platform_ops` on the project.
  Destructive steps need `platform_destructive`, which **a human must approve in the UI** —
  the assistant cannot self-approve it. Plan for that wait.
- **Git access to the customer's repositories from this machine** — § 4.2 (`phases/04a-parts.md`) clones the code
  repo to read the submodule pins out of the tree, and § 4.3 (`phases/04-equivalence.md`) clones the custom-addons
  repo to move its content into `parts/custom/`. Confirm it works (`git ls-remote <url>`)
  before reaching those phases; a missing SSH key surfaces there as a failed clone that
  reads like a wrong URL. **Ask the developer for the repository URLs** rather than
  guessing them from the project name: the namespace pattern is per-cluster
  configuration, not a convention.
- `kubectl` **READ-ONLY**, and only if you have it at all — most of this runbook does not
  need it. The only write allowed is the ArgoCD `refresh=hard` annotation. Everything
  that reaches the cluster goes through GitOps.
- **Every `docker` / `docker-compose` / `df` command in this runbook runs ON THE DOCKER
  HOST, through `docker_ssh_exec`.** None of them is a local command, and the container
  names are the customer's, not a convention — read them from `docker ps` (§ 1a (`phases/01-survey.md`)) instead
  of assuming.
- **The NEWEST release available, for the Odoo series the customer is already on.** Not a
  number written on this page — **look it up**, every time, and take the highest one.

  The series comes from the Docker project being migrated: a customer on 18.0 migrates
  onto the newest `odoo-18.0-r*`, one on 19.0 onto the newest `odoo-19.0-r*`. A migration
  does not upgrade Odoo; that is a separate decision with its own testing.

  **Never write a release number into this runbook.** Any number here is stale within
  weeks, and choosing an old release is not a cosmetic mistake — the catalogue is what
  carries the per-stage sizing defaults, the connection budget and whether the connection
  pooler is on. An environment created on a release from a few weeks ago comes up
  oversized, with pgbouncer running, and nothing reports a problem. It just costs money
  and diverges from the rest of the fleet.

  Where to look it up:

  - `list_release_tags(environment_id=…)` — for an **existing** environment. Newest first,
    with `is_current`. It answers `E_NO_TARGET_APP` when the environment has no app, so it
    cannot tell you which release to pick for a project that has no environments yet.
  - `platform_project_deploy_prepare` (§ Phase 3 (`phases/03-create-deploy.md`)) — for a **new** project. That is the
    wizard's own domain and it is where the choice actually gets made.

  **Check the environment's current release before promising a URL restore** (§ Phase 6 (`phases/06-move-data.md`)):
  restoring from an S3 URL needs the build where `s3Key` stopped being required, and the
  backup itself needs the fix for the deadlock that meant **no k8s backup ever completed**
  — `package_tgz` waited on a pipe nobody drained, silently, while the action recorded
  `done`. On a release older than those fixes the restore is refused before the job even
  renders. Taking the newest release makes this a non-issue; inheriting an old one does
  not.

  Two clusters, two **different repositories**, so the numbering is independent and the
  tags are not comparable across them: `infra-es/dyncloud/releases` for es-prod-01,
  `infra-es/dyncloud/releases-dev` for es-dev-01.

---

## First, the question that shapes everything: one run or two?

**Ask before anything else, and write the answer into the migration record.** The seven
phases below are the work; how they are grouped in TIME is this decision.

**Two stages — the default, and what any customer already live needs:**

- **Stage 1 — build it and rehearse.** Phases 1 to 6.1: the whole structure created, and
  a *production* database restored into it **in test mode, neutralised**. Nothing about
  production changes. The point is to prove the migration works and to learn what it
  costs, with the old environment still serving.
- **Stage 2 — the cutover.** Opens with a **fresh backup of the Docker environment**, on a
  date agreed with the customer *because stage 1 already showed it works*. Then the data
  move and Phase 7.

**One run — only when the customer is not using production yet.** A project still in
implantation has nobody to coordinate with and no window to protect, so the whole thing
goes in a single pass. Ask explicitly; do not infer it from the environment looking quiet.

> **The final backup belongs to stage 2, not to the rename.** The rename does not touch the backup
> layout, and a backup taken early is stale by cutover: take it the day of the move.

> **"Neutralised" is not a switch you flip by moving the environment to staging.** Moving
> it there does not neutralise the database — what it does is make
> `check_neutralized_or_abort()` **refuse to start Odoo** while the database is not
> neutralised. Same practical outcome, different mechanism, and it is the usual
> explanation for an environment that "does not come up" and says nothing useful.

---
---

## The seven phases — READ THE PHASE FILE BEFORE STARTING IT

**This page is the router. It is not the procedure.** Each phase lives in its own file
under `phases/`, and every one of them carries VERIFY lists, STOP conditions and gotchas
that are not repeated here. **Open the file for a phase before doing anything in that
phase**, and keep it open while you work through it — a phase done from this index alone
is a phase done from memory.

| Phase | File | What it does |
|---|---|---|
| **1** | `phases/01-survey.md` | Survey and warnings. Changes nothing. Finds the facts that shape the migration, and the customer-side items with weeks of lead time |
| **2** | `phases/02-rename.md` | Renames the Docker project to `<customer>-docker`, freeing the customer's real name |
| **3** | `phases/03-create-deploy.md` | Creates the Kubernetes project through the deploy wizard and brings up its first environment. **Not reversible** |
| **4** | `phases/04-equivalence.md` | Makes the two deployments equivalent: branches, custom modules, Python dependencies, config parity, deploy targets |
| **4.2** | `phases/04a-parts.md` | The parts conversion — manifest → `dyn-build.yaml`. Split out because it is the longest section and only needed while converting |
| **5** | `phases/05-prove-runtime.md` | Proves the runtime healthy on the demo database, before any customer data goes near it |
| **6** | `phases/06-move-data.md` | Moves the data through S3: disk check, restores, neutralization, and the connector swap that follows every restore |
| **7** | `phases/07-cutover.md` | The production cutover: the checklist review, the window, the domains, the final connector swap |

**The order is the order.** The only phase that moves is 4.0's question — which
environments migrate — which has to be answered before Phase 3, because the wizard is
where they get declared.

**Two of them repeat.** § 6.3 (`phases/06-move-data.md`) runs once per restored database,
so it is inside 6.1's loop *and* again in Phase 7 after the cutover restore. And
`phases/04a-parts.md` is re-read whenever a manifest is converted, not once per migration.

## Delegates to

| Skill | Purpose |
|-------|----------|
| `dyncloud-ops-mcp` | **Load it first.** The operations MCP itself — tools, the JIT grant model, reading an environment's real state, a blocked queue. Every AUTO step here is one of its calls |
| `common-skills:dyncloud-mcp` | the OTHER surface: the ORM/admin proxy that reaches inside the customer's Odoo — § 6.3 (`phases/06-move-data.md`) |
| `dyncloud-migration` | the connector itself (`dyncloud_auth` / `dyncloud_api`) |
| `developer-skills:git-manager` | branch / commit / MR on the code and gitops repos |
| `gitops-change` | anything that reaches the cluster |
| `cluster-inspector` | looking at a cluster without touching it |
| `incident-runbooks` | when the cutover goes wrong |

## References

- `phases/` — the procedure itself, one file per phase. **This page routes; those pages
  instruct.** The index above says which is which
- `references/production-checklist.md` — the pre-production review, item by item
- `references/parts-conversion.md` — manifest → `dyn-build.yaml`, formats and edge cases
- `dyncloud-migration` — the connector itself (`dyncloud_auth` / `dyncloud_api`)
- `common-skills:dyncloud-mcp` — driving the MCP
- `gitops-change` — anything that reaches the cluster
- `cluster-inspector` — looking at a cluster without touching it
- `incident-runbooks` — when the cutover goes wrong
