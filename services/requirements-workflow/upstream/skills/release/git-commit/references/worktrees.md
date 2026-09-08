# Working on a git worktree

When a feature lives in a sibling worktree (e.g. `<repo>-worktrees/PROJ-198-openspec/`) but the Cursor
session opened the main repo (`<repo>/`) as workspace root, **Auto-Attached rules silently stop firing**
for files in the worktree. Their `globs` (e.g. `"**/*.{ts,tsx,js,jsx}"`) resolve relative to workspace
root — sibling paths outside that root never match, so rules like `coding-standards.mdc` are not pinned
into the agent context while editing worktree files.

## Rule

First action on any worktree-based task: move the agent's workspace root onto the worktree.

```
cursor-app-control.move_agent_to_root  path=<absolute worktree path>
```

After the call, globs resolve against the worktree and Auto-Attach works as intended. Skip this step and
you will quietly write code that violates rules the agent never saw.

## When to run

- Resuming or starting any feature where the branch is checked out in a worktree (not in the main repo).
- After creating a new worktree with `git worktree add`.
- Any time the initial `<rules>` block lists a rule as `agent_requestable` when its frontmatter declares
  `alwaysApply: false` + `globs: "…"` — that's the tell that Auto-Attach didn't apply.

## Alternative

Open the parent directory containing both the main repo and every worktree as workspace root. Globs then
match both the main repo and every worktree. Trade-off: noisier file trees.
