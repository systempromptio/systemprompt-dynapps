# Phase 6 — Move the data

> **docker2k8s — Phase 6 of 7.** Before starting: Phase 5 finished: every migrated environment healthy on its demo database. You also need the `platform_destructive` grant approved by a human before 6.1, and the `ssh_exec` grant before 6.0.
> The router, the safety rules and the Prerequisites live in `SKILL.md`.


### 6.0 AUTO then STOP — will the backup fit on the Docker host?

**Do this before anything else in this phase, and stop if the answer is no.**

The legacy Docker backup **materialises its output on the host's disk before uploading**.
How much it needs depends on which version of `asoo.backup.create.yml` the environment's
repository carries, and the difference is enormous:

| Template | What it writes to disk | Measured peak |
|---|---|---|
| **Legacy** | Three copies: plain `dump.sql`, a full filestore copy, then a zip of both | **41.3 GiB** for a 17 GB backup. One customer would have needed **180 GiB** on a host with 123.7 already used — which is why their filestore backup is switched off |
| **Streaming (slim)** | One `.tgz` streamed to S3; only the database dump touches disk | **1–8 GiB** |

So:

**AUTO — the assistant works this out, it is not a question for the developer.** The
inputs are all reachable: the template comes from the environment's repository, the sizes
from Phase 1a, and free disk from the host over `docker_ssh_exec` (`df -h`). Compute it
and report the three numbers.

1. Read `asoo.backup.create.yml` on the environment's branch and determine **which of the
   two it is**. The marker is unambiguous — look at what the upload line produces:

   | | Streaming (slim) | Legacy |
   |---|---|---|
   | Object | `… \| aws s3 cp - s3://…/{bck_name}.tgz` | a `.zip` |
   | Dump | `pg_dump … -Fc -Z6 > …/dump.pgc` | plain `dump.sql` |
   | Filestore | `docker cp …/. -` piped to `gzip -1` | `docker-compose cp` of the whole tree, then `zip -r` |
   | Present | `set -o pipefail` / `bash -o pipefail -c` | absent |

   The fastest single check: **does the create template mention `.tgz` and `pipefail`?**
   Both present means streaming.

2. Measure free disk on the host (`df -h`) and the sizes from Phase 1a.
3. Compare.

> **Switching an environment to the streaming template does not strand its old backups.**
> The streaming `asoo.backup.restore.yml` opens with `case {object_short_key} in *.zip)`
> — it still restores legacy zip objects alongside the new `.tgz`. So "switch the
> template first" is a real option in the decision below, not a one-way door.

> **The backup is ALWAYS database and filestore together.** Splitting them — dumping the
> database now and moving the filestore some other way — is not on the menu. The two are
> one consistent state, and a migration that carries half of it is a migration whose
> attachments point at files that are not there.

**If the estimate does not fit with room to spare: STOP.** Report the numbers — free
space, expected peak, which template — and let a human decide between the two real
options: **switch that environment's template to the streaming version** (which is not a
one-way door, see above), or **ask for the Docker VPS disk to be extended**. Extending
the disk has lead time; raise it the moment the numbers say so, not on cutover week.

**Do not "try it and see": a Docker host that fills up takes the customer's production
down**, and this has already had to be killed by hand once.

### 6.1 Test environments first — AUTO

For each staging/development environment, in a loop:

1. `platform_backup_create` on the **Docker** environment.
2. **Close the action** (§ CLOSE THE ACTION (`SKILL.md`)). Both layers: `platform_list_actions` for
   `effective_state` and `project_blocked`, and `platform_job_runs` for what the job
   itself did. **Do not trust the action state alone** — an action can report `done` while
   its job failed, and that exact discrepancy hid a broken backup for eight days.
3. `platform_list_backups` — the object exists, with a plausible size.
4. `platform_backup_url` on that object — a presigned URL. **It is a bearer credential
   over customer data and it is short-lived**: use it, do not cache it, do not paste it
   anywhere it will be kept.
5. `platform_backup_restore` on the **Kubernetes** environment, with `sourceUrl` set to
   that URL. Needs an approved `platform_destructive` grant.
   - **`jobStorage`** sizes the restore job's scratch volume. Set it when restoring from a
     URL: Cloud cannot infer a size from an object it does not own, and the fallback
     estimate is derived from the *destination's* volumes, which say nothing about what is
     being poured in.
   - **Close the action** before touching this environment again. A restore that failed
     leaves the queue blocked AND the environment scaled to zero (see below), so the next
     thing you enqueue would wait behind it while the environment looks simply dead.
6. Verify the restored environment: it boots, the data is there, the filestore came with
   it, the modules are the customer's.
7. **Swap the connector — § 6.3, before moving to the next environment.** The database
   you just restored is the customer's, so it carries `asoo_connector` and not the new
   pair. The install half happens on its own at boot; the uninstall is one Proxy call.
   It is part of the loop, not a phase that comes afterwards.

**A restore quiesces the environment** — it scales to zero, runs, and is brought back at
the end of the chain. **If the job fails, the environment stays down**: the un-quiesce only
arrives when the chain completes. Watch it to a terminal state rather than launching and
walking away.

### 6.2 ASK — does production become staging first?

**Ask before doing it. It is a decision, not a step.**

The first restore of production data is normally done onto a **neutralized** database, to
check the migration without the environment being able to reach the outside world.
Neutralization is driven by the environment's stage:

- The stage's `stage_type` becomes `environmentType` in the GitOps values.
- On a **non-production** value, the container's start command runs
  `check_neutralized_or_abort()`: it queries `database.is_neutralized` and, if the database
  is not neutralized, **Odoo refuses to start**. It does not warn — it exits.
- The upgrade command neutralizes as part of its own run.

So moving the environment to `staging` does not "neutralize on restore" — it makes the
environment **demand** a neutralized database, and the platform provides one on the
upgrade path. The effect is what David describes; the mechanism is a hard abort, which is
worth knowing when an environment mysteriously will not boot.

Ask whether to do it, and remember to move it back before the real cutover.

> **Why neutralization is not optional on a copy of production:** a live Odoo database
> carries credentials that point at real things — mail servers that would email real
> customers, API keys, webhooks, DNS records, queued jobs that would retry old work,
> crons that synchronise with live systems. Odoo's own neutralization covers the hooks
> Odoo declares. Anything custom is not covered by it and must be checked separately.

### 6.3 Swap the connector — AUTO over the **Proxy** MCP, after every restore

**This is the change the migration is actually for.** How the customer's users reach and
authenticate against Dyncloud stops being `asoo_connector` and becomes `dyncloud_api` +
`dyncloud_auth`.

**When: after each restore, every time.** Not once for the project — once per restored
database. Every environment in § 6.1 needs it, and production needs it again in § Phase 7 (`phases/07-cutover.md`)
after the cutover restore. A restore replaces the database *and its module list*, so a
swap done before it is simply overwritten.

| Half | How |
|---|---|
| **Install** `dyncloud_auth` + `dyncloud_api` | **Automatic, at boot.** The values overlay emits both in `odoo.requiredModules`, and `install-required-modules.py` in the `upgrade-odoo` initContainer installs whatever is missing on every pod start. Idempotent. Nothing to do by hand |
| **Uninstall** `asoo_connector` | **`admin_uninstall_module` over the Proxy MCP, and only once the install half is VERIFIED** — the channel *is* `dyncloud_api`, so checking the pair is installed is what makes the call possible |

> **The order is not a preference, it is a dependency.** The Proxy MCP reaches a customer's
> Odoo *through* `dyncloud_api`: it calls that module's own REST endpoints, so on a
> database where it is not installed every proxy call 404s, whatever it asks for. The
> module cannot install itself either — see the box below — so until the boot-time install
> has run, there is no channel and no uninstall. Restore, let it boot, then swap.

> **Why `requiredModules` and not `auto_install`.** `dyncloud_api` declares
> `auto_install=True`, and on an existing database that does nothing. The cascade in
> `odoo/modules/db.py` runs inside `initialize()` — fresh bootstrap only — and
> `ir_module.button_install` cascades to an auto-install module only when one of its
> `auto_install_required` dependencies is `'to install'` **in that same run**. On a
> restored environment `dyncloud_auth` is already `installed`, so the condition is never
> met. Without the list entry the module would stay uninstalled forever.

> **The module list is refreshed for you.** `click-odoo-update` runs before the install
> step in the same init script and calls `Registry.new(update_module=True)`, which is
> `Module.update_list()` — so Odoo has already re-read the addons path by the time
> anything is installed. No manual "Update Apps List" is needed on Kubernetes.

> **STOP if the pod does not start after a release upgrade.** `install-required-modules.py`
> does not skip a module it cannot find: it **aborts with exit 1**, and the environment
> never comes up. A module reaches `ir_module_module` only if it is on disk, and an image
> built with `dyncloud-odoo-auth` below 19.0.3.2.0 carries no `dyncloud_api` at all.
>
> **The ordering rule: rebuild first, upgrade second — never the reverse.** An environment
> must be rebuilt onto an image that actually carries `dyncloud_api` BEFORE it adopts the
> values overlay that requires it. Two releases apart in the catalogue is enough for a
> migration to walk into the gap: the release that started *requiring* the module is newer
> than the one that started *shipping* it.
>
> Taking the newest release (§ Prerequisites (`SKILL.md`)) satisfies both at once and this never comes
> up. It only bites when an environment is moved release by release, or when the
> `dyn-build.yaml` pin was left behind (§ 4.2 (`phases/04a-parts.md`) — `dyncloud-odoo-auth`).

> **The image has no `asoo_connector` code, and the uninstall works anyway.** § 4.2 (`phases/04a-parts.md`) drops
> `asoo-tools`, the repository that holds that one module. Odoo does not need the code to
> remove it: STEP 5 of `odoo/modules/loading.py` reads `state='to remove'` straight from
> SQL and calls `module_uninstall()`, which works off `ir.model.data` — tables, models and
> records all go. The graph is consulted only for an `uninstall_hook`, and a module absent
> from it simply contributes none. That hook is the only thing genuinely lost.
>
> **What DID bite, on 2026-08-24, and is now repaired by the platform.** The restored
> database carries `asoo_connector` as `installed`. The first boot upgrades against a
> different image, so `click-odoo-update` includes `base` — and `button_upgrade` writes
> `to upgrade` on **every** installed module without asking whether it still exists on
> disk. The registry then moves back to `installed` only what it could load; the orphan is
> never in the graph, so nothing ever moves it again. And a single module stuck in
> `to upgrade` blocks **every** module operation in that database —
> `_button_immediate_function` refuses to run while any module is `to install`,
> `to upgrade` or `to remove` — including its own uninstall. So
> `install-required-modules.py` exits 1, the initContainer crashloops, and the only way
> back in used to be a shell on the cluster.
>
> `odoo-base` now repairs it at boot: `reset-orphan-module-states.py` resets any module
> left mid-operation whose code is not on the addons path. **The signal that it worked** is
> this line in the `upgrade-odoo` initContainer log:
>
> ```
> Module asoo_connector is to upgrade but has no code on the addons path:
> the operation can never complete. Reset to installed.
> Reset 1 orphan module state(s). Uninstall them to clean up the data they left behind.
> ```
>
> Verified live on 2026-08-24, same backup and same environment twice: crashloop on the
> image before the fix, `3/3 Running` at the first attempt on the image after it, with the
> pair installing itself and the uninstall then going through the Proxy MCP with no
> workaround at all.

> **REMEMBER THE UNINSTALL — it is the half nobody does.** The install half is silent and
> automatic, so the environment looks finished the moment the pod turns green. And the
> reset above deliberately leaves `asoo_connector` as `installed`: it unblocks the
> database, it does not clean it. Its tables and its `ir.model.data` are still there, and
> every later upgrade marks it `to upgrade` again.
>
> **The moment you have VERIFIED that `dyncloud_api` and `dyncloud_auth` are installed,
> uninstall `asoo_connector`.** That order is the whole procedure: the Proxy MCP is the
> tool, `dyncloud_api` is what carries it, so confirming the install half is not a
> courtesy — it is what makes the uninstall reachable at all.

> **A module operation is not a read.** `admin_uninstall_module` drops the module's tables
> and data from a production database. Confirm with the human before the call, name the
> environment and the module in that confirmation, and never batch it across environments
> in one turn.

**VERIFY, per environment:** `dyncloud_api` and `dyncloud_auth` installed,
`asoo_connector` gone from the module list, and a real user logging in through the new
path. A connector that is *installed* is not a connector that *works* — the login is the
check.

**And now the Proxy MCP should reach this environment.** It could not before, and it can
only work if the swap really landed, so a `list_databases` / `orm_search` against it is a
second, independent confirmation that costs one call. It is a check, not the mechanism.

