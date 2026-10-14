# Code Migration — Full Workflow (STEP 0 → STEP 7)

Covers §8 of the Dynapps migration process: migrating the customer source-code
repository from one Odoo version to another. Runs in parallel with the DB track
(`odoo-database-upgrader` + `indaws-migration-kit`).

## STEP 0: Gather Context

Required inputs:
- Customer slug (e.g. `acme`)
- Source version (e.g. `16`) and target version (e.g. `18`)
- `repo_path` — local path to the customer repo: `$ODOO_SRC/buildouts/<customer><version>/parts/custom`
  - Auto-detect: `ls $ODOO_SRC/buildouts/${CUSTOMER}${VERSION_DESTINO}/parts/custom`
  - **Never fetch from GitLab remote to read files** — always work from the local clone on disk.
  - The GitLab remote URL is only needed for `git remote add` (STEP 1) and is read from the local repo: `git -C ${REPO_PATH} remote get-url origin`

Read `references/buildout-structure.md` to understand directory layout.

## STEP 0.5: Verify prerequisites — ALWAYS, before any migration work

Run these EVERY time, even (especially) when joining a migration someone else started. A repo can
be scaffolded + have partial work (e.g. EDI) pushed WITHOUT the history rebase ever done.
Full commands + remediation in `references/gitlab-prerequisites.md`.

**A. Git history integrated?** The foundational rebase (STEP 1) + `[MIG] Initial Commit` (STEP 2)
may have been skipped. Verify before doing anything per-module:
```bash
git -C <repo> remote add <oldver> git@gitlab.dynapps.be:customers/<ver>/<customer>.git 2>/dev/null
git -C <repo> fetch <oldver>
git -C <repo> merge-base develop <oldver>/develop >/dev/null && echo "OK: history integrated" || echo "MISSING"
git -C <repo> log --oneline --all | grep -iq "MIG.*Initial Commit" && echo "OK: Initial Commit" || echo "MISSING"
```
If MISSING → **STOP Point 5**. Do STEP 1-2 first (see protected-branch note in STEP 2).

**B. GitLab access & branch protection** (gitlab.dynapps.be) — these block CI even with perfect code:
- `develop`/`master` are protected: **push = Maintainers only**, `allow_force_push: false`. A regular
  dev CANNOT push to develop. The foundational rebase rewrites develop → needs a **Maintainer** to
  force-push (push the rebased history to a side branch like `develop-rebased` and have a Maintainer
  reset develop to it; a normal MR can't do it — it's a history rewrite, not a merge).
  Check: `glab api projects/:id/protected_branches`.
- The dev's user must **not be flagged External** — else CI builds **0 jobs** and the pipeline
  "fails" (can't read the internal `infra/gitlab-ci-components`). Check: `glab api user` → `external:false`.
- The dev's user must have **read access to ALL `dynapps-apps/*` repos in `dyn-build.yaml`** (direct
  projects AND subgroups like `dynapps-apps/dyn-edi`) — the `CI_JOB_TOKEN` inherits the triggering
  user's access, so the build fails cloning the first one it can't read (exit 128). The dev usually
  lacks a subset; check ALL and request them together (loop in `references/gitlab-prerequisites.md` §D).
- Any failure here = **infra/admin request** (a Dynapps maintainer), not a code fix. Surface it.

**C. Code-standards gate** (applies to STEP 2 and every per-module MR):
- `pre-commit run -a` green on the active modules.
- The migrated module's tests pass **locally on a clean, freshly-installed DB** — NOT the migrated
  "browse" DB (a different addons config throws env-mismatch errors unrelated to the module):
  `createdb t_<m>; odoo-bin -d t_<m> -i <module> --test-enable --test-tags /<module> --stop-after-init`
  → `0 failed, 0 error`. Don't rely on CI; it may be blocked by B.
- Commit messages: **no `Co-Authored-By` / AI attribution** (customer repos, human authorship only).

## STEP 1: Integrate git history from old repo

```bash
git remote add <oldver> git@gitlab.dynapps.be:customers/<ver>/<customer>.git
git fetch <oldver>
git checkout develop
git rebase <oldver>/develop
```

Resolve conflicts using `references/rebase-ruleset.md`.
After rebase, delete files listed in that reference (tools/, deploy scripts, legacy cfg, etc.).

## STEP 2: Initial commit

```bash
git add --all
pre-commit run -a   # expected to fail — modules not yet migrated
git add --all
git commit --no-verify -m "[MIG] Initial Commit"
git push
```

`--no-verify` is **permitted here only** (pre-commit fails intentionally at this stage).

**Pushing the Initial Commit / rebased develop** (per STEP 0.5-B): `develop` is protected
(Maintainers only, force-push disabled). If develop already has commits pushed (scaffold/EDI), the
rebase rewrites its history → a plain push is rejected (non-fast-forward) and force-push is blocked.
A regular dev resolves this by pushing the rebased history to a **side branch** (`develop-rebased`)
and asking a **Maintainer** to reset `develop` to it (and, if needed, to temporarily lift force-push
protection). This is NOT an MR — a merge can't replace history, only a force-update can.

## STEP 3: Mark modules as non-installable

For every module in `parts/custom/` except `xx_base`, `xx_all`, `dyn_test_suite_common`:
1. Set `"installable": False` in `__manifest__.py`.
2. Add module path to the `exclude` list in `.pre-commit-config.yaml`.

See `references/rebase-ruleset.md` → "Pre-commit exclude block" for format.

## STEP 3.5: Set up Docker iterate environment

Before the module loop starts, the iterate loop needs a clean Docker PostgreSQL
container so the kit can do restore → apply → `-u all` cycles without touching
the local development DB.

```bash
cd migration_kit_<cliente>
docker compose up -d db
docker compose exec db createdb -U odoo <cliente>_migrate
```

Read `references/docker-iterate-env.md` for full docker-compose.yml and
migration.yaml context configuration.

## STEP 4: Determine migration order

```bash
indaws-migrate modules order
```

Order: Odoo standard patches → OCA → Dynapps verticals → custom modules.
Read `references/migration-order.md` for full criteria.

## STEP 5: Per-module migration loop

For each custom module in order — read `references/per-module-loop.md` for exact steps.
Delegate code port to **`odoo-module-upgrader`**.
After port: uncomment module in `xx_all/__manifest__.py`, test install, create MR.

**Dependency pattern (do NOT get this wrong):** a custom module's `depends` lists ONLY `xx_base`
plus the other CUSTOM modules it needs — **never standard Odoo / OCA modules directly**. The
standard/OCA dependencies live in `xx_base`, and each module's MR **adds the ones it introduces to
`xx_base`'s `depends`** (so `xx_base` grows lean→full as modules are migrated). E.g. when migrating a
module that uses `sale`/`hr`/`account_followup`, put those in `xx_base`, not in the module manifest.
Confirm against the old version's manifests (`git show <oldver>/develop:parts/custom/<m>/__manifest__.py`).

## STEP 6: Stub modules for uninstallation

For every module that must be removed from the new version:
create a stub in `parts/modules_to_remove/<module_name>/`.
Read `references/buildout-structure.md` → "Stub modules" section.

## STEP 7: Migration scripts

- `xx_base/migrations/<version>/end-migrate.py` — uninstalls stub modules at upgrade end.
- `xx_all/migrations/<version>/end-reactivate_views.py` — reactivates inactive views.

Full code in `references/buildout-structure.md` → "Migration scripts" section.

## STOP Points

1. **Rebase conflict without clear ruleset** — show diff, ask user before resolving.
2. **`# TODO-PORT-vXX` left by `odoo-module-upgrader`** — present list, ask developer.
3. **Pipeline fails outside known patterns** — diagnose, then AskUserQuestion.
4. **Modules not in topological order output** — ask user for manual placement.
5. **Prerequisites not met** (STEP 0.5) — if the old-version git history is NOT integrated into
   `develop` (no shared ancestor / no `[MIG] Initial Commit`), STOP: the foundation (STEP 1-2) must
   be done first. Never start per-module MRs on a repo without the integrated history — they'd sit
   on a base that has to be rewritten later, forcing a redo.
6. **A module MR/push is not green locally** — never push a module that doesn't pass `pre-commit run -a`
   AND its own test suite locally. CI may be blocked (external flag / dyn-edi access, see STEP 0.5),
   so local green is the gate, not CI.

## GitLab actions — confirmation required (no exceptions)

**Every action that touches GitLab must be confirmed by the user before executing**, regardless
of permission mode or bypass settings. This includes:

| Action | Must confirm before |
|---|---|
| `git push` (any branch) | Showing exact branch + commit summary |
| `glab mr create` | Showing full MR title, target branch, and diff stats |
| `glab mr update` | Showing what changes |
| Any `glab` write command | Any remote write operation |

Never push or create/update MRs autonomously even if the user previously said "go ahead" —
each push/MR is a separate confirmation. The rule is: **prepare → show → wait → execute**.
