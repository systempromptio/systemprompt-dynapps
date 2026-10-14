# Phase 1 — Survey and warnings

> **docker2k8s — Phase 1 of 7.** Before starting: Nothing before it — this is where the migration starts. Its answers feed every later phase, and two of its items (1d, the customer's external dependencies) have weeks of lead time.
> The router, the safety rules and the Prerequisites live in `SKILL.md`.


**Nothing is changed in this phase.** It exists to find, before anything moves, the facts
that decide the shape of the migration and the ones with external lead time.

### 1a. What the platform can tell you — AUTO

Through the **DevOps** MCP (§ Prerequisites (`SKILL.md`) — the one carrying `platform_env_status`):

- `list_projects` / `list_environments` — the project, its environments, their stages.
  **Note every environment, not only the Odoo ones** (see 1c).
- `platform_env_status`, `platform_list_actions` — is the project healthy, is its queue
  blocked (`project_blocked`), is anything half-finished. **A migration does not start on
  a blocked queue**: whatever failed there predates you, and everything you enqueue would
  queue behind it. Resolve or report it before Phase 2.
- `platform_list_backups` — do backups even run today, and how big are they. A project
  with no recent successful backup is a project whose backup path is broken; find out why
  **now**, not in Phase 6.
- `platform_list_domains` — every FQDN, its TLS, its IP allow-list. The allow-lists matter:
  see 1d.
- `docker_ssh_exec` — the host itself. Database size, filestore size, free disk:

  ```bash
  df -h                                    # free space, the Phase 6 input
  docker ps --format '{{.Names}}\t{{.Image}}'
  du -sh <home>/<env>/                     # workspace on disk
  ```

  For the database, ask Postgres rather than guessing from the volume:

  ```sql
  SELECT pg_size_pretty(pg_database_size(current_database()));
  ```

**WARN — a very large database changes the plan.** It changes the backup window, the
restore job's scratch volume, and whether the cutover fits in the agreed downtime. Decide
the sizing before Phase 6 rather than discovering it there.

### 1b. Which parts manifest — AUTO, with a trap

A Docker checkout keeps its submodules in **`.gitmodules`** (plain git) **or** in
**`repositories.yml`** (the Cloud manifest) — **never both**. Which one decides how Phase
4 converts them, and they are different code paths.

`platform_submodules_plan` on the environment reports it.

> **GOTCHA — "no submodules" may mean "could not read".** `cloud.git.repo.get_file`
> returns `None` for *every* failure: unresolvable repo, missing branch, expired
> credentials, host down, secret store not answering. A consumer cannot tell that from a
> file that is simply absent. This has cost real hours: one environment served a 503 for
> six hours because an empty read became an empty compose file. The submodule tool now
> refuses to assert the interpretation, and `platform_submodules_plan` returns a
> `warning` distinguishing "no submodules" from "could not read". **Read that field.** If
> it says the read failed, stop and fix the access before drawing any conclusion.

### 1c. Environments that are not Odoo — WARN

A Docker project may hold environments that are not Odoo at all — a `cors-everywhere`
proxy, a static helper, an internal tool. **They do not migrate through this procedure.**
List them explicitly as out of scope, say what each one is, and agree separately how it
gets deployed. Do not let one ride along silently inside "migrate the project".

Detect them by the image and the compose file, not by the name:

```bash
docker ps --format '{{.Names}}\t{{.Image}}'
```

### 1d. What only the customer knows — CUSTOMER, ask on day one

**These have external lead time. Ask before any technical work starts.** The public IP
changes with the runtime, and everything keyed to the old one breaks silently at cutover.

Ask the customer, item by item:

- **Outbound IP allow-lists.** Any provider that restricts access by source IP — banks,
  tax agencies, EDI, logistics, payment gateways, a supplier's API, a VPN endpoint. **The
  customer must ask each provider to allow the new egress IP**, and some take weeks.
- **Direct Postgres access.** If anyone connects to the database from outside — a BI tool,
  an external report, the customer's own analysts — production needs an ingress
  **domain routing to the Postgres port**, which is a deliberate configuration, not a
  default. Find out who connects and from where.
- **Inbound webhooks and callbacks.** Anything a third party calls: payment
  notifications, courier status, e-invoicing, marketplace callbacks, CI hooks. If any is
  registered against an IP rather than a hostname, it must be re-registered.
- **Outbound integrations and SMTP.** Which mail server sends, whether it authenticates by
  IP, whether SPF records name the current host.
- **External cron and scheduled jobs.** Anything outside Odoo that reaches into it — a
  script on another server, a customer-side scheduler, a partner's nightly job.
- **DNS not managed by us.** Records the customer controls, and who can change them on
  cutover day, at the hour agreed.
- **Fixed maintenance windows.** When the business can afford downtime, and when it
  absolutely cannot (payroll, closing, campaigns).

Then ask the open question, because this list is never complete:

> *"Beyond these, is there anything that knows this system by its IP address, or anything
> that would notice it moving?"*

**Write every answer into the migration record.** An unanswered item is a hard stop before
production, not before staging.

### VERIFY before leaving Phase 1

- [ ] Manifest format identified, and the read confirmed to have **succeeded**
- [ ] Every environment classified: migrates / does not migrate / not Odoo
- [ ] Database and filestore sizes measured, free disk on the host recorded
- [ ] The customer list answered, or the open items named with an owner and a date
- [ ] The project's action queue is not blocked

