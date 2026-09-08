---
name: /project-context:sync
id: project-context-sync
category: Knowledge
description: Refresh local project memory — pull committed docs from .cursor, sync Confluence + Jira from Atlassian, then rebuild the project-context index
---

You are refreshing the **local project memory** served to Cursor by the `project-context` MCP
server. Do this in three stages, in order:

1. **Pull** the latest committed docs from the `.cursor` repo — meeting notes and ADRs are authored
   by hand and committed to the project branch, so a teammate may have added new ones.
2. **Sync** Confluence + Jira from Atlassian into `inbox/`.
3. **Reindex** so both the pulled docs *and* the freshly synced Atlassian content become searchable.

Two kinds of source live side by side and must both be refreshed:

- **Committed, hand-authored** — `inbox/meeting_notes/` and `inbox/adrs/` are tracked in the `.cursor`
  git repo. They arrive via `git pull` (Stage 1), not from Atlassian.
- **Synced from Atlassian** — `inbox/confluence/` and `inbox/jira/` are pulled live (Stage 2).

The indexer (Stage 3) walks the *entire* inbox, so it is what makes the git-pulled notes searchable —
skipping it means new meeting notes are on disk but invisible to `search`.

---

## Operating Rules

- **Never** read, print, stage, or commit `.cursor/.project/.env` — it holds the Bedrock and Atlassian
  credentials.
- Run the stages **in order** and **stop on the first failure**. Report the error. If it is a
  configuration/credentials error, point the user at `.cursor/.project/project-context/INSTALL.md`.
- This command only refreshes **local** state (`inbox/` + `.index/`). Do **not** commit or push anything.
- Assume you are at the **code repository root** (the parent directory of `.cursor`). The engine lives
  at `.cursor/.project/project-context`.

---

## Stage 1 — Pull committed docs from `.cursor`

1. Confirm `.cursor` is its own git repo:

```bash
git -C .cursor rev-parse --git-dir
```

   If this fails, this project isn't using the branch-per-project tooling model — note that there is
   nothing to pull and skip straight to Stage 2.

2. Check the branch and that the working tree is clean:

```bash
git -C .cursor branch --show-current
git -C .cursor status --porcelain
```

   If the tree is dirty in a way that would block a pull (e.g. local edits to inbox files), list the
   changed files and stop — ask the user to commit or stash first.

3. Pull the current branch (fast-forward only — never force, never rebase a shared branch):

```bash
git -C .cursor pull --ff-only
```

   - If it isn't fast-forwardable or a conflict occurs, stop and report — let the user reconcile.
   - Report which new files/commits arrived, especially under `inbox/meeting_notes/` and `inbox/adrs/`.

---

## Stage 2 — Sync Confluence + Jira from Atlassian

Work from the engine directory. First make sure dependencies are installed (once per machine):

```bash
cd .cursor/.project/project-context
[ -d node_modules ] || npm install
```

Fetch fresh Confluence pages and Jira issues into `inbox/` **without** indexing yet — Stage 3 does the
single reindex pass over everything:

```bash
npm run sync -- --no-index
```

Options:
- `npm run sync -- --no-index --source jira` — Jira only (skip Confluence).
- `npm run sync -- --no-index --source confluence` — Confluence only.

Report the per-source add/update/unchanged/delete counts from the sync log.

---

## Stage 3 — Rebuild the index

From the same engine directory (`.cursor/.project/project-context`), run the indexer over the whole
inbox. This picks up the meeting notes / ADRs pulled in Stage 1 **and** the Atlassian content synced in
Stage 2:

```bash
npm run index
```

- Incremental by default — unchanged files (same md5) are skipped, removed files are pruned.
- For a clean, full rebuild from scratch (e.g. after changing chunking/embedding config):

```bash
npm run index -- --rebuild
```

> Shortcut: `npm run sync` (no flags) performs Stage 2 **and** Stage 3 in one command. The split above
> is the explicit form; use the shortcut when you don't need per-stage control.

---

## Stage 4 — Verify

Confirm the index is populated by calling the `project-context` MCP tool `index_stats` (or
`list_sources`). Check the total chunk count, per-source document counts, and `last_sync`. The MCP
server re-reads the index version on each call, so no restart is needed.

---

## Report

```
## Project Context Refreshed

**Pulled (.cursor):** N new commit(s) — X meeting note(s), Y ADR(s)   [or "already up to date"]
**Atlassian sync:** confluence +a ~u =n -d · jira +a ~u =n -d
**Index:** total chunks: T
- meeting_notes: … · adrs: … · confluence: … · jira: … · specs: … · openspec_archive: …

**last_sync:** <timestamp>
```

If a stage was skipped (e.g. `.cursor` isn't a git repo, or `--source` filtered one out), say so
explicitly rather than implying it ran.
