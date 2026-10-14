# Production cutover checklist

Run this **as a fresh review in a separate agent**, against the migration record rather
than against anyone's memory of how the migration went. The point of a separate agent is
that it has not spent the last three days believing the migration is fine.

Every unchecked item is a **STOP**, not a note. Report them; do not weigh them.

## A. External dependencies — the ones with lead time

- [ ] Every provider that restricts by source IP has been asked, **and has confirmed**,
      that the new egress IP is allowed. Asked is not confirmed.
- [ ] Inbound webhooks and callbacks registered against an IP have been re-registered.
- [ ] SMTP works from the new runtime, and SPF/DKIM/DMARC still pass. Send a real message
      and read its headers; do not infer it from configuration.
- [ ] External schedulers that reach into Odoo have been repointed.
- [ ] Certificates that are not platform-issued are installed and valid.
- [ ] Direct Postgres consumers: the ingress domain to the Postgres port exists, is
      reachable, and the consumer has been told the new endpoint.

## B. Parity between the two deployments

- [ ] Every module present on Docker is present on Kubernetes, at the intended version.
      Compare the Apps lists, not the manifests. **Two deliberate exceptions**:
      `asoo_connector` must be GONE, and `dyncloud_api` + `dyncloud_auth` must be there.
- [ ] `project_blocked` is **false** right now, and no action in the recent history is
      sitting `enqueued` behind a failure. A blocked queue turns every step of the cutover
      into a no-op that reports success.
- [ ] The connector swap is **in the cutover plan, with an owner and a slot** — after the
      final restore, before the DNS change. It was done once on this environment already;
      the final restore undoes it, and that is exactly why it gets forgotten.
- [ ] **Every `dyn-build.yaml` part was read one by one** against the pin table in
      SKILL.md § 4.2 — not spot-checked, not trusted to the conversion script. Each part
      matches exactly one of the two valid shapes: `commit:` with a `# branch:` comment
      and no `branch:` key, or `branch:` with `# tag` and no `commit:` key.
- [ ] No part carries `commit:` and a `branch:` key together — that build fails, in both
      builders: the commit clone is shallow and detached, and the `git checkout <branch>`
      that follows finds no such branch.
- [ ] No part carries a **tag name inside `commit:`**. Clone-by-commit resolves shas only.
- [ ] `dyncloud-odoo-auth` is pinned **by tag, with no `commit:`**, and the tag is the
      newest one for the customer's series — looked up with `git ls-remote --tags`, not
      inherited from whatever the scaffold happened to carry. A stale pin here is how the
      environment fails to boot with `Missing module(s): dyncloud_api`.
- [ ] `requirements.txt` matches the Docker entrypoint's generated file, version by
      version.
- [ ] Odoo configuration carried across: workers, limits, `server_wide_modules`, extra
      config.
- [ ] Every FQDN exists on the new environment with its redirects and IP allow-lists.
- [ ] The connector answers: `/dyncloud/api/v1/*` does not 404, and a real user has
      logged in through it. Installed is not the same as working.
- [ ] No `dyn-build.yaml` part still points at `github.com/indawsodoo/*` — those are
      `dynapps-es/mirrors/…` with `source: dynapps`. And `asoo-tools` is not among them.

## C. The data path, proved not assumed

- [ ] A full backup → restore cycle has completed on a **non-production** environment with
      real production-sized data, and the result was verified by a human opening it.
- [ ] The restore job's scratch volume was sized correctly, and `jobStorage` is known for
      the production run.
- [ ] Backup on the Docker host fits in free disk **on the day**, re-measured. Disk moves.
- [ ] The filestore arrived: count files, do not trust the absence of an error.
- [ ] The production restore duration is measured, and it fits the agreed window with
      margin.

## D. The environment itself

- [ ] **Sizing matches the fleet standard** (SKILL.md § Phase 5): Odoo pod requests
      165m/902Mi on production or 65m/646Mi on staging, `pooler.enabled: false`,
      `dbMaxConn: 10`, and `max_connections` **declared** as the quoted string `"100"`.
      A mismatch means the environment was built on an old release, not that somebody
      tuned it — unless a deliberate, written tuning decision says otherwise.
- [ ] Production stage is back to `production` if it was moved to `staging` for testing.
      An environment left on `staging` **refuses to start** on a non-neutralized database.
- [ ] `deploy_policy` on production is `manual`, not `autodeploy`.
- [ ] Health is green and the queue is not blocked.
- [ ] No half-finished action from earlier phases is sitting in the chain.
- [ ] Backups are scheduled on the new environment, and one has actually succeeded —
      checked in `platform_job_runs`, not in the action state.

## E. The plan itself

- [ ] The window is agreed with the business, in writing.
- [ ] The order of operations is written down, with an owner per step.
- [ ] The DNS change has a named person who can make it, available in the window.
- [ ] The rollback is written down, **with its point of no return stated explicitly** —
      the moment after which going back means losing data written to the new system.
- [ ] Someone is watching after the cutover, and it is agreed for how long.
- [ ] The customer knows what to expect and who to call.

## F. What is deliberately not migrating

- [ ] Non-Odoo environments are listed, and each has an agreed destiny — migrated another
      way, retired, or left on Docker.
- [ ] The Docker project keeps its historical backups, and the retention that governs them
      is understood. Do not decommission the old host while it is the only copy.
