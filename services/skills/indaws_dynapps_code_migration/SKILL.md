---
name: indaws-dynapps-code-migration
description: >
  Migrates a Dynapps customer buildout repository between Odoo versions (source code track).
  Use when the user asks to migrate a customer repo, port modules to a new Odoo version,
  set up a new-version buildout branch, or perform §8 of the Dynapps migration process.
  Covers git history integration, rebase conflict resolution, marking modules non-installable,
  per-module porting loop, stub modules for uninstallation, and migration scripts in xx_base/xx_all.
version: 1.1.2
model: opus
effort: high
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion]
---

# Dynapps Code Migration

Covers **§8 of the Dynapps migration process**: migrating the customer source-code repository
from one Odoo version to another. Runs in parallel with the DB track (`odoo-database-upgrader`
+ `indaws-migration-kit`).

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Rebase + conflict resolution | **opus** | Merge decisions require context |
| Per-module port | **opus** → delegates to `odoo-module-upgrader` | Code transformation |
| Git operations | sonnet → delegates to `git-manager` | Mechanical |
| Topological order | sonnet → runs `indaws-migrate modules order` | CLI call |

## Workflow

Full step-by-step procedure (STEP 0 → STEP 7), STOP points, and the GitLab
confirmation protocol are in **`references/workflow.md`**. Summary:

| Step | What |
|------|------|
| STEP 0 | Gather context (customer, source/target version, local `repo_path`) |
| STEP 0.5 | Verify prerequisites (history integrated, GitLab access, code-standards gate) |
| STEP 1 | Integrate git history from old repo (rebase) |
| STEP 2 | `[MIG] Initial Commit` (`--no-verify` permitted here only) |
| STEP 3 | Mark modules non-installable + pre-commit exclude |
| STEP 3.5 | Set up Docker iterate environment |
| STEP 4 | Determine migration order (`indaws-migrate modules order`) |
| STEP 5 | Per-module migration loop (delegate port to `odoo-module-upgrader`) |
| STEP 6 | Stub modules for uninstallation |
| STEP 7 | Migration scripts (`end-migrate.py`, `end-reactivate_views.py`) |

## GOTCHAS

1. **GitLab writes always need explicit confirmation** — every `git push` / `glab mr create|update`
   must be shown and confirmed before running, even if the user earlier said "go ahead". Prepare →
   show → wait → execute. Never autonomous.
2. **`develop` is protected (Maintainers only, no force-push)** — the foundational rebase rewrites
   develop, so a regular dev can't push it. Push to a side branch (`develop-rebased`) and ask a
   Maintainer to reset develop to it. An MR can't replace history.
3. **External-flagged user → CI builds 0 jobs** — the pipeline "fails" with no code error because it
   can't read `infra/gitlab-ci-components`. Check `glab api user` → `external:false`. This is an
   infra/admin request, not a code fix.
4. **CI_JOB_TOKEN inherits the triggering user's access** — the build clones every `dynapps-apps/*`
   repo in `dyn-build.yaml` (incl. subgroups like `dyn-edi`); it fails (exit 128) on the first one
   the dev can't read. Request read access to ALL of them together.
5. **Local green is the gate, not CI** — CI may be blocked (external flag / dyn-edi). Never push a
   module unless `pre-commit run -a` AND its own test suite pass locally on a clean fresh DB.
6. **Never start per-module MRs without integrated history** — if STEP 1-2 were skipped, the MR sits
   on a base that gets rewritten later, forcing a full redo. Verify STEP 0.5-A first.
7. **Module `depends` lists ONLY `xx_base` + other custom modules** — standard/OCA deps live in
   `xx_base`; each module MR adds the deps it introduces to `xx_base`, never to its own manifest.

## RULES

### NEVER
- Never push or create/update an MR without showing it and getting explicit confirmation first.
- Never fetch files from the GitLab remote to read them — always work from the local clone.
- Never push a module that isn't green locally (`pre-commit run -a` + own tests on a fresh DB).
- Never start per-module work on a repo whose old-version history isn't integrated (STOP Point 5).
- Never put standard/OCA dependencies in a custom module's manifest — they go in `xx_base`.
- Never add `Co-Authored-By` / AI attribution to commits in customer repos.

### ALWAYS
- Always run STEP 0.5 prerequisite checks every time, even when joining an in-progress migration.
- Always resolve rebase conflicts using `references/rebase-ruleset.md`; STOP if unclear.
- Always migrate in topological order (`indaws-migrate modules order`).
- Always test install + run tests on a clean freshly-installed DB before creating a module MR.

## References

| Situation | Read |
|---|---|
| Full step-by-step workflow + STOP points + GitLab confirmation | `references/workflow.md` |
| Prerequisite/access checks (history, branch protection, external flag, dyn-edi) | `references/gitlab-prerequisites.md` |
| Rebase conflict resolution rules | `references/rebase-ruleset.md` |
| Per-module loop exact steps | `references/per-module-loop.md` |
| Buildout structure, stubs, migration scripts | `references/buildout-structure.md` |
| Migration order criteria | `references/migration-order.md` |
| Docker container for iterate loop | `references/docker-iterate-env.md` |
| Cross-suite shared conventions | `common:references/shared/` |

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| `odoo-module-upgrader` | developer | Port source code per module (Step 5) |
| `git-manager` | developer | Commits, MRs, pipeline diagnosis |
| `indaws-migration-kit` CLI | infra | Topological module order (Step 4) |

## Consumed by

| Skill | Suite | Context |
|-------|-------|---------|
| `dynapps-migrator` | developer | Orchestrated in BLOQUE 3 (code track) |
