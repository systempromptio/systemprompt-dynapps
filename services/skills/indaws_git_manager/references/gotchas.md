# Gotchas

Failure patterns and critical "watch out for" items. Built from real incidents and testing.
Read this before any risky operation (cherry-pick, tag, hotfix, release, force operations).

---

## 1. Sequence gotchas

- **Hotfix: production FIRST, then integration.**
  If you sync to integration before merging to production, the fix may reach staging before production, giving a false sense of resolution.
  The order is ALWAYS: merge to production -> tag -> deploy -> sync to integration.

- **Release MR (develop -> master): NEVER `--remove-source-branch`.**
  If the source is a permanent branch (develop), deleting it removes the integration branch entirely.
  Only use `--remove-source-branch` on temporary branches (feature/, hotfix/).

---

## 2. Merge method consequences

- **Squash: 1 feature = 1 commit.**
  Cherry-pick is clean -- one SHA, one feature. But if you need to cherry-pick from a branch that used squash merge, the squash commit SHA is on the TARGET branch, not the source.
  Find it with `git log {target} --oneline`.

- **Fast-forward: commits preserved.**
  Cherry-pick requires identifying all commits of a feature. Use `git cherry-pick A^..B` for a range.
  If you cherry-pick just one commit from a multi-commit feature, you get a partial feature in production.

- **Merge-commit: merge commit on target.**
  NEVER blindly cherry-pick HEAD after a merge -- it may be the merge commit, not the actual change.
  Check with `git log --oneline -2`: the merge commit is first, the real commit is second.
  Or use `git cherry-pick -m 1 <merge-SHA>` to pick the merge itself.

---

## 3. Cherry-pick gotchas

- **Cherry-pick after squash can conflict unexpectedly.**
  When integration and production have diverged significantly, a squash commit may not apply cleanly even though the original branch merged fine.
  Always preview: `git cherry-pick --no-commit <SHA>` first, then `git cherry-pick --continue` if clean.

- **Hotfix cherry-pick to integration may silently succeed with wrong content.**
  If integration has divergent changes in the same files, the cherry-pick may apply without conflict but produce logically incorrect code.
  ALWAYS review after cherry-pick: `git diff HEAD~1`.

---

## 4. Tag gotchas

- **Always use annotated tags (`git tag -a`).**
  Some CI pipelines only trigger on annotated tags. Lightweight tags (`git tag v1.0` without `-a`) may not trigger deployments.
  Always use `-a` with a message.

- **Tags are effectively immutable once pushed.**
  Deleting and recreating a tag that was already pushed may cause issues -- other developers may have fetched the old tag, CI may have already triggered.
  Coordinate with the team before deleting a pushed tag.

---

## 5. Platform CLI gotchas

- **macOS grep lacks `-P` (Perl regex).**
  BSD grep on macOS doesn't support `-P`. Always use `grep -oE` instead of `grep -oP` for regex extraction.
  Example: `grep -oE 'P[0-9]+-[0-9]+'` instead of `grep -oP 'P\d+-\d+'`.

- **`glab ci view` requires TTY.**
  In non-interactive environments (Claude Code, scripts), use `glab ci status` instead.

- **`glab mr create` does not support `--hostname`.**
  The hostname is determined by the git remote of the current repo.
  Ensure the remote points to the correct instance.

- **`glab ci trace` needs the numeric job ID, not the job name.**
  Get the ID from `glab ci status` output or the API.

- **`glab mr create` defaults to the repo's default branch.**
  Always pass `--target-branch` explicitly, or the MR may target the wrong branch.

- **`gh run rerun --failed`** retries only failed jobs (more efficient than full rerun).

---

## 6. Pre-commit gotchas

- **Prettier reformats YAML files.**
  Inline YAML lists like `[a, b, c]` get expanded to multiline.
  Generate YAML in the format prettier expects, or it will fail in CI.

- **Prettier wraps Markdown prose (`proseWrap`).**
  Long lines in `.md` fail the prettier hook in CI ("files were modified by this hook").
  Run `pre-commit run prettier --files <f.md>` locally to auto-wrap before committing, then re-add.

- **`mixed-line-ending` / `trailing-whitespace` corrupt fixed-width & CRLF files.**
  Committing EDI/flat-file samples (e.g. an A3 `suenlace.dat`: fixed 510+CRLF, space padding) into a
  repo with the standard Dynapps pre-commit: `mixed-line-ending` strips the CR (510+CRLF → 510+LF, breaks
  the record length) and `trailing-whitespace` can eat the padding. The file is silently mangled.
  Fix: add the folder to the top-level `exclude:` regex in `.pre-commit-config.yaml` (e.g. `/docs/examples/|`),
  OR don't commit the raw sample — embed the example inline in a `.md` instead. Verify length survived:
  `awk 'NR==1{print length}' file.dat` (should stay 511 = 510 content + CR).

- **Never bypass with `--no-verify`.**
  If pre-commit hooks fail, fix the issue. Bypassing means the CI pipeline will fail anyway, wasting more time.
  If a hook is misconfigured, fix it or report it.

---

## 7. Branch gotchas

- **Detached HEAD after checking out a tag or commit.**
  Use `git checkout <branch-name>` to return to a branch.
  If you made changes in detached HEAD, create a branch first: `git checkout -b recovery-branch`.

- **Stale local branches accumulate.**
  After MRs are merged with `--remove-source-branch`, the remote branch is deleted but the local one remains.
  Clean up periodically with:
  ```bash
  git fetch --prune
  git branch -vv | grep ': gone]' | awk '{print $1}' | xargs git branch -d
  ```
