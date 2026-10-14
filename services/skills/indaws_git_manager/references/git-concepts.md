# Git Concepts Reference

Operational git knowledge for branch-based development workflows.

## Branch models

A branch model defines how many long-lived branches exist and what role each plays.

**1-branch (trunk-based):** A single branch serves as both integration and production. Features merge directly into it. Releases are marked with tags. Works well for small teams with strong CI.

**2-branch:** Integration branch (features land here) + production branch (released code). Most common setup for project teams.

**3-branch (with validation):** Adds a validation/testing branch between integration and production. Features are validated individually in a testing environment before being grouped into a release. Useful when business stakeholders approve features one by one.

**4-branch (with acceptance):** Adds an acceptance/pre-production branch after validation, mirroring production data. Common in enterprise or regulated environments requiring final sign-off.

### Branch roles

- **Integration** -- where features merge. CI runs here. Deploys to staging.
- **Validation** -- QA/testing. Individual feature validation before release.
- **Acceptance** -- pre-production. Final validation with production-like data.
- **Production** -- live code. Only receives tested, approved changes.

The actual branch name is irrelevant (`develop`, `dev`, `main` -- whatever). What matters is the role.

## Merge methods and their consequences

The merge method determines how commits appear on the target branch and directly affects cherry-pick and selective deploys.

### Squash

All commits from a feature branch become a single commit on the target.

- **History:** 1 feature = 1 commit. Clean, linear.
- **Cherry-pick:** One SHA per feature -- trivial: `git cherry-pick abc1234`
- **Trade-off:** Intermediate commit detail lost on target (still visible in the MR).

### Fast-forward

All commits preserved and replayed on top of the target. No merge commit created.

- **History:** All original commits preserved. Linear if rebased before merge.
- **Cherry-pick:** Requires range: `git cherry-pick A^..B` or individual picks.
- **Trade-off:** More detail, but more complex for selective deploys.

### Merge commit

Creates a merge commit + preserves all original commits. Non-linear history.

- **History:** Merge commits mark where features were integrated.
- **Cherry-pick:** `git cherry-pick -m 1 <merge-SHA>` (parent 1 = target branch side).
- **Trade-off:** Full branching structure visible, harder to read linearly.

## Protected branches

A protected branch enforces: no direct push (MR/PR required), force push blocked, CI pipeline must pass, review approval needed. This ensures code on critical branches has been reviewed and tested, and history cannot be rewritten.

Typically integration and production are protected. Validation branches may or may not be.

## Release strategies

### Develop-based

All features in integration go to production together. Simpler, fewer cherry-picks, less coordination. Risk: one broken feature blocks the release. Best for pre-go-live or high-velocity teams.

### Release-based

Features validated individually, selectively included in releases via cherry-pick. Full control over what ships. More coordination and potential cherry-pick conflicts. Best for production-critical projects.

### Transitioning

Teams typically start develop-based during initial phases and transition to release-based once in production. This means:
1. Adding a validation branch for individual feature testing.
2. Switching from "merge integration to production" to "cherry-pick approved features into a release branch."
3. Optionally adding an acceptance branch for final sign-off.

## Cherry-pick operations

Cherry-pick applies changes from a specific commit onto the current branch, creating a new commit with a different SHA.

**When:** moving validated features to a release branch, syncing hotfixes from production to integration, selective releases.

**How merge method affects cherry-pick:**

| Merge method | What to pick | Command |
|---|---|---|
| Squash | Single squashed commit | `git cherry-pick <SHA>` |
| Fast-forward | All feature commits | `git cherry-pick A^..B` |
| Merge commit | The merge commit | `git cherry-pick -m 1 <SHA>` |

With squash, "1 feature = 1 commit" makes selective deploys clean. This is a major reason teams choose squash for release-based workflows.

**Conflict handling:**
```bash
# Resolve conflicts, then:
git add <resolved-files>
git cherry-pick --continue
# Or abort:
git cherry-pick --abort
```

## Sync strategies

Keeping a feature branch up to date with its base branch.

### Rebase

Replays your commits on top of the updated base. Commits get new SHAs.
```bash
git fetch origin
git rebase origin/develop
```
Use when the branch will be squashed (history doesn't matter). Avoid on shared branches -- rebase rewrites history, forcing collaborators to force-pull.

### Merge

Creates a merge commit bringing base branch changes into your feature branch.
```bash
git fetch origin
git merge origin/develop
```
Use when commits will be preserved (ff merge) or the branch is shared.

### Auto (derived from merge method)

- **Squash merge** implies rebase sync -- branch will be squashed anyway.
- **Fast-forward merge** implies merge sync -- commits are preserved, don't rewrite them.

Always check for uncommitted changes before syncing. Stash or commit first.

## Fork model

Developers push to a personal fork instead of the main repository. Used for shared/core repos with many contributors or inner-source models.

### Setup
```bash
git clone git@host:your-user/repo.git
git remote add upstream git@host:main-org/repo.git
# origin    → your fork (push here)
# upstream  → main repo (pull from here, MR/PR target)
```

### Feature flow
```bash
git fetch upstream
git checkout upstream/develop -b feature/TASK-123
# Work, commit...
git push -u origin feature/TASK-123
# Create MR/PR: origin/feature/TASK-123 → upstream/develop
```

### Keeping fork in sync
```bash
git fetch upstream && git checkout develop
git merge upstream/develop && git push origin develop
```

## Tags and deployments

Tags mark release points in history.

**Annotated vs lightweight:** Annotated tags (`git tag -a`) store tagger, date, and message. Some CI only triggers on annotated. Lightweight tags are just pointers. Always use annotated for releases:
```bash
git tag -a v19.0.4 -m "Release v19.0.4"
git push origin v19.0.4
```

**Naming:** Semantic versioning (`vMAJOR.MINOR.PATCH`). For Odoo projects: major.minor matches the Odoo version, patch increments per release (`v19.0.1`, `v19.0.2`). Hotfixes add a fourth segment: `v19.0.2.1`.

**Immutability:** Tags are effectively immutable once pushed. Deleting a pushed tag is dangerous -- it may have triggered a deploy or been referenced elsewhere. Only delete mistaken tags, and always confirm first.

## Why not git-flow CLI

The `git-flow` CLI (upstream archived, Homebrew disabling Dec 2026) is incompatible with MR/PR-based workflows:

- `git flow feature finish` merges locally, bypassing MR/PR, pipeline, squash, and branch protection.
- `git flow release finish` does the same.
- Only `start` and `publish` are safe, but they're just aliases for `git checkout -b` and `git push -u`.

Use `git` directly with the platform's MR/PR mechanism to ensure all merges go through review and CI.
