# Access model — what the user's level allows (shared pattern)

**Azure AD is the single IdP** for the whole infrastructure (ArgoCD, OpenBao, Grafana,
NetBird, cluster access via Pinniped, the documentation itself). The user's Azure AD **group**
decides what they may do. Every skill in the plugin reasons with this model and **adapts what
it proposes to the user's level**.

## The operational rule (this is what lives here)

- **When in doubt, assume the most restrictive level and say so.** Offer the safe path
  (read-only, or "this needs someone with the higher level") instead of proposing a command
  the user cannot run — or worse, one they can run and shouldn't.
- **Don't invent the user's group.** If an action depends on the level and you don't know it,
  ask; until then, reason as read-only.
- **The levels are three** — `devops`, `developer`, `readonly` (auditor). Use the names as
  vocabulary when you talk to the user; a `consultant` level has been discussed but is not
  something to assume exists.

## Where the detail comes from

**The current group ids and exactly what each level can do are documentation, not skill
content** — they change without any skill changing. Ask the documentation MCP (identity and
access, under the GitOps/infra section) following the contract in
[`docs-access.md`](docs-access.md), and delegate the "how to ask" to `common-skills:dynbrain`.

On this Dynapps instance that MCP is the portal's `dynbrain` server, and Odoo records come
from the portal's `dynapps-odoo` server. Both run as the user's own portal account, so what a
lookup returns is already bounded by that account's access.

With **MCP unavailable**: say you can't confirm the level's exact scope, apply the
most-restrictive rule above, and don't quote group ids from memory.

## Nuance with the "`kubectl` READ-ONLY" rule

The golden rule "`kubectl` read-only except the ArgoCD refresh" (see
[`danger-gates.md`](danger-gates.md)) is about **not mutating the declarative state** of the
cluster (no `apply`/`edit`/`delete`/`scale`/`patch` outside GitOps). Apart from that:

- **`kubectl exec` on Odoo pods** (e.g. `odoo-bin shell`, reading logs, inspecting) does NOT
  mutate cluster state and is a **legitimate capability of the `developer` (and `devops`)
  level** — so it is allowed for them. It is used mostly by `dyncloud-development`.
- **Beware the blast-radius of `exec`**: an `odoo-bin shell` against a **production**
  environment can write business data. Treat it like any dangerous write: **warn about the
  scope and confirm** before running anything that modifies data.
- Any mutation of cluster **resources** stays forbidden outside GitOps (except the ArgoCD
  `annotate … refresh=hard`).
