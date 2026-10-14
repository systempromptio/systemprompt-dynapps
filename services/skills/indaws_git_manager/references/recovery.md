# Recovery Playbooks

When something goes wrong with Git, use the appropriate playbook below. Each one explains the situation, the fix, and how to verify you're back on track.

**Rule of thumb:** If the mistake is local (not pushed), fixing is easy and safe. If it's been pushed, coordinate with the team before rewriting history.

---

## 1. Working on the wrong branch

**Situation:** You've been making changes but you're on the wrong branch (e.g., on `develop` instead of your feature branch).

### If changes are NOT committed yet

```bash
# Save your work
git stash

# Switch to the correct branch (create if needed)
git checkout -b feature/{task_id}
# or if it exists:
git checkout feature/{task_id}

# Restore your work
git stash pop
```

### If changes ARE committed (not pushed)

```bash
# Note the SHA(s) of your commits
git log --oneline -3

# Switch to the correct branch
git checkout -b feature/{task_id}

# Bring your commits over
git cherry-pick <SHA1> <SHA2> ...

# Go back and remove from wrong branch
git checkout wrong-branch
git reset --hard HEAD~{number_of_commits}
```

**Verify:**
```bash
git log --oneline -5  # commits should be on the right branch
git diff HEAD~1       # your changes are intact
```

---

## 2. Committed to the wrong branch (and pushed)

**Situation:** You committed AND pushed to a branch you shouldn't have (e.g., directly to `develop`).

**Warning:** This involves rewriting shared history. Coordinate with the team.

```bash
# 1. Note the SHA of your commit
git log --oneline -3

# 2. Create your feature branch with the commit
git checkout -b feature/{task_id}
git push -u origin feature/{task_id}

# 3. Remove from the wrong branch (COORDINATE WITH TEAM FIRST)
git checkout develop
git revert <SHA>      # Safe: creates a new commit that undoes the change
git push origin develop
```

**Why `revert` instead of `reset`?** `revert` creates a new commit (safe for shared branches). `reset` rewrites history (dangerous on shared branches).

---

## 3. Need to undo the last commit

### If NOT pushed

```bash
# Undo commit but keep changes staged
git reset --soft HEAD~1

# Undo commit and unstage changes (keep files)
git reset HEAD~1

# Undo commit and discard changes entirely (DESTRUCTIVE)
git reset --hard HEAD~1
```

### If already pushed

```bash
# Create a new commit that reverses the changes (safe for shared branches)
git revert HEAD
git push
```

---

## 4. Cherry-pick conflict

**Situation:** `git cherry-pick <SHA>` resulted in conflicts.

### Option A: Resolve the conflict

```bash
# See which files conflict
git status

# Edit conflicting files (look for <<<<<<< markers)
# After resolving all conflicts:
git add <resolved-files>
git cherry-pick --continue
```

### Option B: Abort and try a different approach

```bash
git cherry-pick --abort
```

**Alternative approaches when cherry-pick consistently conflicts:**
1. **Manual apply:** Look at the commit's diff (`git show <SHA>`) and apply changes manually
2. **Targeted cherry-pick:** Cherry-pick with `--no-commit`, then selectively stage only what you need
3. **Merge instead:** If many commits conflict, consider merging the full integration branch to production

**Post-cherry-pick verification** (always do this):
```bash
git diff HEAD~1   # Review what actually changed
# Verify the result makes sense — a clean cherry-pick can still produce wrong code
```

---

## 5. Pushed a sensitive file

**Situation:** A file with credentials, API keys, or secrets was committed and pushed.

**URGENT:** The secret is compromised the moment it's pushed. Even if you remove it, it's in git history and may have been pulled by others.

### Immediate actions

1. **Rotate the secret NOW** — change the password/API key in the service it belongs to
2. **Remove from the repo:**

```bash
# Remove the file and commit
git rm --cached .env
echo ".env" >> .gitignore
git add .gitignore
git commit -m "chore: remove sensitive file, add to gitignore"
git push
```

3. **Clean history** (if the secret was in previous commits):

```bash
# Using git-filter-repo (recommended, install: pip install git-filter-repo)
git filter-repo --path .env --invert-paths

# Force push (COORDINATE WITH TEAM)
git push --force-with-lease
```

**Important:** `--force-with-lease` is safer than `--force` — it fails if someone else has pushed in the meantime.

4. **Notify the team** — they need to re-clone or reset their local repos after history rewrite

---

## 6. Merge conflicts when pulling

**Situation:** `git pull` on your feature branch results in conflicts because the base branch has changed.

```bash
# Option A: Rebase (cleaner history, preferred for feature branches)
git fetch origin
git rebase origin/{integration}

# Resolve any conflicts, then:
git add <resolved-files>
git rebase --continue

# If it gets messy:
git rebase --abort

# Option B: Merge (simpler, creates a merge commit)
git merge origin/{integration}
# Resolve conflicts, then:
git add <resolved-files>
git commit
```

**Which to choose?**
- **Rebase** for feature branches (cleaner history, will be squashed anyway)
- **Merge** if the branch has been shared with others (rebase rewrites history)

---

## 7. Accidentally deleted a branch

### Local branch

```bash
# Find the last commit on the deleted branch
git reflog | grep "checkout: moving from deleted-branch"

# Recreate it
git checkout -b deleted-branch <SHA>
```

### Remote branch

```bash
# If you still have it locally:
git push -u origin deleted-branch

# If not, check if there was an MR/PR — the commits may still be accessible
# GitLab: glab mr view <id> to find the source branch commits
# GitHub: gh pr view <id> to find the head branch commits
```

---

## 8. Wrong tag created

**Situation:** A tag was created on the wrong commit or with the wrong name.

```bash
# Delete locally
git tag -d v19.0.4

# Delete from remote (CONFIRM FIRST — may have triggered deploy)
git push origin :refs/tags/v19.0.4

# Create correct tag
git tag -a v19.0.4 <correct-SHA> -m "Release v19.0.4"
git push origin v19.0.4
```

**Warning:** If the tag triggered a deployment pipeline, coordinate with the team before deleting. The deployment may need to be rolled back separately.

---

## 9. Pre-commit hooks failing and blocking commit

**Situation:** `git commit` fails because pre-commit hooks detect issues.

```bash
# See what failed
pre-commit run --all-files

# Auto-fix what can be auto-fixed
pre-commit run --all-files  # many hooks auto-fix on second run

# Stage the auto-fixes
git add -u
git commit -m "style: fix linting issues"
```

**Never bypass with `--no-verify`** — if the hook fails, fix the issue. Bypassing means the CI pipeline will fail anyway, wasting more time.

If a specific hook is misconfigured or broken:
1. Read `.pre-commit-config.yaml` to understand what's configured
2. Check if the hook version matches the project's Python/Node version
3. Report the issue to the team if it's a systemic problem

---

## 10. Detached HEAD state

**Situation:** You checked out a specific commit or tag and now you're in "detached HEAD" state.

```bash
# If you just want to go back to your branch
git checkout {your-branch}

# If you made changes in detached HEAD and want to keep them
git checkout -b recovery-branch
# Then cherry-pick or merge to the right branch
```

---

## Decision tree: "Something went wrong, now what?"

```
Did you push it?
├── No (local only)
│   ├── Wrong branch? → Playbook #1
│   ├── Bad commit? → Playbook #3 (soft reset)
│   ├── Sensitive file? → Playbook #5 (still rotate the secret!)
│   └── Conflicts? → Playbook #6
│
└── Yes (pushed to remote)
    ├── Wrong branch? → Playbook #2 (revert)
    ├── Bad commit? → Playbook #3 (revert)
    ├── Sensitive file? → Playbook #5 (URGENT: rotate + clean history)
    ├── Wrong tag? → Playbook #8
    └── Cherry-pick conflict? → Playbook #4
```
