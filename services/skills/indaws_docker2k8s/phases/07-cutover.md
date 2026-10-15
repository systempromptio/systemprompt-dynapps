# Phase 7 — Production cutover

> **docker2k8s — Phase 7 of 7.** Before starting: Phase 6 finished on the test environments, and stage 1 rehearsed end to end. Everything up to here was reversible; this is not.
> The router, the safety rules and the Prerequisites live in `SKILL.md`.


**STOP — checkpoint before the cutover is planned, not after.**

Everything up to here is reversible. This is not. Before scheduling, run a **fresh review
in a separate agent** against the checklist in
`references/production-checklist.md`, and let it report against the migration record
rather than against memory of how it went.

The cutover plan needs, at minimum: the agreed window, the order of operations, who does
each one, the DNS change and who can make it, the rollback and its point of no return, and
who is watching afterwards and for how long.

> **In the window, every step is closed before the next one starts** (§ CLOSE THE ACTION (`SKILL.md`)).
> This is where the habit pays for itself: the cutover is a chain of enqueued operations
> under time pressure, and the failure mode — queue blocked, everything after it silently
> waiting — looks exactly like "it is taking a while". Budget the reads into the window.

> **The connector swap belongs IN that window, by name, with an owner.** The cutover
> restores production data one last time, which puts `asoo_connector` back — exactly as it
> did on every test environment. The new pair reinstalls itself at the next boot; the
> removal does not. So § 6.3 (`phases/06-move-data.md`) runs again here, **after the final restore and before the DNS
> change**: point the domain at an environment still carrying the old connector and you
> have two things claiming the same authentication hooks. It is minutes of work and the
> easiest thing in the runbook to forget, because it was already done once on the same
> environment.

### The domains — classify them FIRST, at the start of stage 2

**AUTO.** `platform_list_domains(environment_id)` on the Docker environment returns every
`cloud.fqdn` with its `complete_name`. Sort them into two piles by suffix, and act
differently on each:

| the domain ends in | whose it is | what it needs |
|---|---|---|
| `indaws.cloud` or `dyncloud.es` | **ours** | nothing to negotiate. The records get changed by us when stage 2 starts — schedule it, do not ask permission for it |
| **anything else** | **the customer's** | 🔴 **STOP and coordinate.** We do not control that zone. Someone on their side has to change the record, in the agreed window, and be reachable that day |

Prefer moving the environment onto the new `dyncloud.es` names. What happens to existing
`indaws.cloud` names — redirect them, or just repoint the record and keep using them — is
**an open decision, not the skill's to make**: ask, and record the answer per customer.

> Match the suffix properly — the domain either **equals** `indaws.cloud` / `dyncloud.es`
> or **ends with a dot followed by it**. A naive substring test calls
> `shop.indaws.cloud.customer.com` ours, and it is precisely the opposite: a customer
> domain dressed up to look like one of ours.

**Say the red pile out loud, by name, in the plan.** A customer domain that nobody
mentioned does not fail at cutover — it keeps resolving to the old environment, so the
migration looks complete while part of the traffic never moved.

**Do not schedule production while any Phase 1d item is unanswered.** An IP allow-list the
customer never requested does not fail at cutover — it fails the first time that integration
runs, which may be days later, on someone else's shift.

