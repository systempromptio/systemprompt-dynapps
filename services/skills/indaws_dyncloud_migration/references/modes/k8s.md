# Mode B — Docker → Kubernetes

> **NOT IMPLEMENTED.** This file is the scaffold: what is already known, what is deliberately
> absent, and where the seams are. **Do not follow it as a procedure** — it has never been run end
> to end. If a user asks for this migration today, say it is not implemented, offer Mode A for
> connecting a Docker environment, and hand the migration itself to whoever owns the platform.

## This skill is not enough on its own — route through the suite

Mode A is self-contained: two files in one repo. Mode B is not. Moving an environment to
Kubernetes touches the cluster, GitOps, secrets, DNS/TLS, observability and the database, and
**this skill only understands the connector part** — the smallest piece of it.

Start at **`devops-assistant`**, the suite's router: it carries the safety rules, the access model
and the kubeconfig/mesh setup, and it points at the skill that owns each piece —
`gitops-change` for anything that reaches the cluster, `cluster-inspector` to look without
touching, `secrets-openbao` for the signing keys arriving through an ExternalSecret,
`observability-guide` to confirm the migrated environment is actually serving, and
`incident-runbooks` when the cutover goes wrong. Whoever implements this mode should read those
before writing the procedure, not after.

## Why it is a different problem, not a variant

Mode A connects an environment that stays where it is. Mode B **moves** it: data, filestore, DNS,
TLS, and the connector all change substrate at once. The connector part — the only part this skill
understands well — is the smallest piece of that.

The connector mechanism is genuinely different on Kubernetes, in three ways.

## 1. How the module gets into the image

Docker reads the environment's `repositories.yml`. Kubernetes builds from **`dyn-build.yaml`**, and
the pin there follows a different rule.

**Pin by commit only — never `branch` together with `commit`.** With both set, the build performs a
final `git checkout <branch>` on top of a `depth=1` clone made by commit, where no local branch
exists, and it fails. The convention in the generated repositories is commit-only, matching the
shape used for the Odoo core parts.

That is the exact opposite of Mode A, where the neighbouring entries carry both keys and it is
fine. Carrying the Docker habit over is a real trap.

The pin also has to be maintained: the skeleton that generates new client repos carries its own
pin, and a stale one produces environments born with old module code.

## 2. How the public keys arrive

Docker environments **pull** them: the module calls the manager's JWKS endpoint on a cron and
caches them in its own table. Kubernetes environments are **given** them: an ExternalSecret
materialises the public keys as files under a mounted directory, and the module detects that
directory at install time, runs in **disk mode**, and **disables the JWKS cron** — it does not need
it.

Consequences for a migration:

- The environment variables that matter on Docker (`BACKEND_URL`, `ENV_UUID`, `LINEAGE_NAME`)
  stop being the mechanism. Leaving them behind is not harmful, but they are no longer what makes
  it work.
- The signing lineage differs between runtimes. A migrated environment verifies against the
  Kubernetes lineage, not the Docker one.
- The mount must be **read-only**; the verifier warns at boot if the directory is writable.

## 3. How the modules get installed

Docker: manual, by a developer (Mode A, step 5).

Kubernetes: **`odoo.requiredModules`** in the environment's GitOps values, applied by an init
container at pod start. It is idempotent — it skips modules already installed — and it is the
supported answer to the same `must_install()` problem that forces the manual step on Docker.

### `requiredModules` and `dyncloud_api` — a decision, not an oversight

`requiredModules` lists **only `dyncloud_auth`** on the Kubernetes environments checked, and
**that is deliberate for now**: new environments are built on fresh databases, where
`initialize()` walks the auto-install set recursively and pulls `dyncloud_api` in on its own.
Adding it to the list would be redundant there.

It stops being redundant in two cases, and whoever hits one of them should revisit this:

- **A restored database** (`pg_restore` from odoo.sh or another host). `initialize()` never runs
  again, so nothing is auto-installed. Installing `dyncloud_auth` through `requiredModules` does
  pull `dyncloud_api` in behind it — the auth module genuinely enters `'to install'` — so the
  single-entry list still works here.
- **An environment that already had `dyncloud_auth`** before `dyncloud_api` existed. The auth
  module only ever *upgrades* (`'to upgrade'`), never enters `'to install'`, so it drags
  nothing. Neither `auto_install` nor the current list covers this one: `dyncloud_api` has to be
  named explicitly, or installed by hand.

The second case is what produces a 404 on an environment whose image demonstrably contains the
module. Diagnose it before assuming the deployment failed.

## Loose ends for whoever implements this

- The verification pair at the end of Mode A applies unchanged to a migrated environment: it tests
  the connector, not the runtime.
- `troubleshooting.md` § 1 already distinguishes the Docker cause (manual install) from the
  Kubernetes one (`requiredModules`); extend there rather than starting a new troubleshooting page.
- Infra changes go the GitOps way → delegate to `gitops-change`; never a manual `kubectl apply`.
