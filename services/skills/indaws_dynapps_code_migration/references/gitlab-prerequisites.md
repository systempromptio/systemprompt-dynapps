# GitLab prerequisites & access checks (Dynapps Belgium)

Run these at **STEP 0.5**, every time, before any migration work. They catch the failures that
look like "your code is broken" but are really missing-foundation or access problems. `<id>` is the
numeric project id (`glab api projects/:id` → `.id`); the repo is `customers/<v>/<customer>/odoo`.

## A. Is the old-version git history integrated?

A repo is often scaffolded with copier + has partial work (EDI, etc.) pushed to `develop` WITHOUT
the rebase ever being done. If you start per-module MRs on top of that, the base later has to be
rewritten and every MR rebased — wasted work. So verify first:

```bash
git -C <repo> remote add <oldver> git@gitlab.dynapps.be:customers/<ver>/<customer>.git 2>/dev/null
git -C <repo> fetch <oldver>
# 1) shared ancestry?
git -C <repo> merge-base develop <oldver>/develop >/dev/null 2>&1 && echo "OK ancestry" || echo "MISSING ancestry"
# 2) the foundational commit?
git -C <repo> log --oneline --all | grep -iq "MIG.*Initial Commit" && echo "OK initial-commit" || echo "MISSING initial-commit"
```
MISSING on either → the foundation (SKILL STEP 1-2) was skipped. Do it first. This is STOP Point 5.

## B. Branch protection (why a Maintainer is needed for the foundation)

```bash
glab api "projects/<id>/protected_branches" | python3 -c "import sys,json;[print(b['name'],b.get('allow_force_push'),[a.get('access_level_description') for a in b.get('push_access_levels',[])]) for b in json.load(sys.stdin)]"
```
Typical: `develop` and `master` → push = **Maintainers**, `allow_force_push = false`.

Consequence: the rebase rewrites `develop`'s history. A regular dev cannot push to develop at all,
and even a maintainer can't force-push while `allow_force_push=false`. Flow that works:
1. Dev pushes the rebased history to a **side branch**: `git push -u origin develop:develop-rebased`.
2. A **Maintainer** resets develop to it (force-update). They may need to temporarily flip
   `allow_force_push` (or push from a maintainer account). **A merge/MR cannot do this** — replacing
   history is a force-update, not a merge.
Removing the force-push protection alone does NOT let a non-maintainer push to develop (separate rule).

## C. External-user flag (CI builds 0 jobs)

If the dev's GitLab user is flagged **External**, pushes still work but the pipeline ends **failed
with 0 jobs** — because the repo's CI config (`ci_config_path = .gitlab-ci-odoo-19.yml@infra/odoo-gitlabci`)
pulls **CI components from `infra/gitlab-ci-components`** (an Internal project an External user can't read),
so the pipeline can't even be compiled.

```bash
glab api user | python3 -c "import sys,json;print('external:',json.load(sys.stdin).get('external'))"
# confirm the cause if a pipeline has 0 jobs:
glab api "projects/<id>/ci/lint?dry_run=true&ref=<branch>"   # -> "... project is Internal, cannot be accessed by an External User"
```
Fix = **admin removes the External flag** (Admin → Users) or grants Reporter on `infra/gitlab-ci-components`
and `infra/odoo-gitlabci`. Not a code fix.

## D. Access to ALL buildout repos (`dynapps-apps/*`) — CI build clone fails

The buildout (`dyn-build.yaml`) pulls private repos from `dynapps-apps/*` (direct projects AND
subgroups like `dynapps-apps/dyn-edi`). The `CI_JOB_TOKEN` inherits the **triggering user's** access,
so the `build-docker-image` job fails (exit 128, GitPython `NoSuchPathError`) on the **first** repo
the dev can't read — while the repos they can read clone fine. A pipeline triggered by a member
(e.g. a maintainer) passes. The dev usually lacks a **subset**, so check ALL of them, not one — and
request the whole list at once (granting one and re-running just surfaces the next, e.g. ZNU needed
both the `dyn-edi` subgroup and `dyn_queue_followers_group`):

```bash
for r in $(grep -E "repo: dynapps-apps" dyn-build.yaml | sed 's/.*repo: //'); do
  enc=$(echo "$r" | sed 's#/#%2F#g')
  glab api "projects/$enc" >/dev/null 2>&1 || echo "NO access: $r"
done
```
Fix = **admin grants the dev read on every listed repo**. Until then the dev's own pipelines fail at
build even with correct code; validate locally instead (section E).

## E. Code-standards gate (every push / MR)

- `pre-commit run -a` green. The dynapps config auto-excludes `installable: False` modules and
  `parts/{oca_pull,modules_to_remove}`; populate the `# NOT INSTALLABLE ADDONS` block (the
  `oca-update-pre-commit-excluded-addons` hook may not populate it — do it manually like the old repo).
- The migrated module's tests pass locally on a **clean, freshly-installed DB** — NOT the migrated
  "browse" DB. A DB built with a different addons config (e.g. the upgrade-iterate DB) carries
  automations/modules that throw env-mismatch errors unrelated to the module (`'res.partner' has no
  attribute 'recompute_assortments'`, `No module named xx_base.migration_utils`). Use a throwaway DB:
  `createdb t_<m>; odoo-bin -d t_<m> -i <m> --test-enable --test-tags /<m> --stop-after-init` → `0 failed, 0 error`.
  Watch for v16→v19 API removals in tests (e.g. `account_followup.followup.line` dropped `send_letter`).
- Commit messages: **no `Co-Authored-By` / AI attribution** — customer repos, human authorship only.

## Dependency pattern (recurring mistake)

Custom module `depends` = `xx_base` + other CUSTOM modules only; **standard/OCA deps go in `xx_base`**,
added to `xx_base` in the migrating module's MR. Verify against `<oldver>/develop:.../__manifest__.py`.
`xx_base` is lean at the Initial Commit and grows as modules are migrated.
