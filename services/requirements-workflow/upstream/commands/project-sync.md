---
name: /project-sync
id: project-sync
category: Setup
description: Preflight sync — pull the latest tooling on the project branch; if the pull brought new commits, install root Node deps and refresh the OpenSpec schema/config from the library; then always refresh and reindex project-context
---

Preflight sync. Brings the local checkout up to date before you start work:

1. Pull the latest tooling library on the current project branch.
2. **Only if the pull brought new commits**, install/update the shared Node dependencies at the `.cursor`
   root (`package.json` / `package-lock.json`) so skill scripts pick up any newly added packages.
3. **Only if the pull brought new commits**, force-refresh the tooling-owned OpenSpec IP
   (`openspec/config.yaml` + `openspec/schemas/`) from the library templates, so any schema/template
   change we ship lands in your local (gitignored) copy.
4. Always refresh project-context sources and reindex so retrieval reflects the latest project state
   (Confluence/Jira live outside git, so this runs even when the pull was a no-op).

This keeps the whole team on the same schema and prompts: when we change a schema or artifact template
in the library, a `/project-sync` re-materializes it for everyone. It never rewrites client deliverables
(`openspec/specs/`, `openspec/changes/`).

---

## Operating Rules

- This is an automatic preflight step, not a discovery, design, or artifact-generation step.
- Do only these actions: pull the project branch; **if (and only if) the pull brought new commits**,
  install `.cursor` root Node deps and refresh the OpenSpec schema/config; then always sync + reindex
  project-context. The install + schema refresh are git-gated because they only ever change when new
  tooling commits land; project-context is NOT git-gated (Confluence/Jira and local specs change
  independently of the `.cursor` branch), so it always runs.
- Do NOT run `openspec status`, `openspec instructions`, or any artifact-generation logic.
- Do NOT touch `openspec/specs/` or `openspec/changes/` — the schema refresh only overwrites
  `openspec/config.yaml` and the `openspec/schemas/` tree (both gitignored, tooling-owned).
- Never read, print, stage, or commit `.cursor/.project/.env`.
- Do not commit or push anything. The only git action is a `pull` on the current project branch.
- Install Node deps only at the `.cursor` root. Do not run `npm install` in
  `.cursor/.project/project-context`, `.cursor/.project/eval`, or other nested engines — those keep
  their own isolated manifests and are outside this command's scope.

---

## Step 1 — Preflight

1. Confirm `.cursor` is its own git repo: `git -C .cursor rev-parse --git-dir`. If it fails, report that
   this project isn't using the branch-per-project tooling model and stop.
2. Get the current branch: `git -C .cursor branch --show-current`.
   - If it's the shared base branch itself (`master` or `main`), stop and tell the user this command is
     meant to be run from a project branch — check out the project branch first.

---

## Step 2 — Pull the latest tooling on the project branch

Fast-forward the current project branch to pick up the latest tooling (rules, skills, commands,
schema/template changes) that were merged into it. Capture the HEAD commit before and after so the next
steps know whether anything actually changed.

```bash
BEFORE=$(git -C .cursor rev-parse HEAD)
git -C .cursor pull --ff-only
AFTER=$(git -C .cursor rev-parse HEAD)
```

- If the working tree is dirty and the pull is refused, stop and report — do not stash, reset, or force.
  Tell the user to commit or stash their `.cursor` changes first, then re-run.
- If the pull fails for another reason (no upstream, network, merge conflict), report the exact failing
  command and error and stop. Do not attempt a non-fast-forward merge or a rebase.
- **If `BEFORE` equals `AFTER` (nothing was pulled), the tooling is unchanged: SKIP Step 3 and Step 4
  entirely** and go straight to Step 5. Note "tooling already up to date — skipped deps + schema
  refresh". Only when `AFTER` differs from `BEFORE` do you run Steps 3 and 4.

---

## Step 3 — Install `.cursor` root Node dependencies (only if the pull brought new commits)

**Skip this step entirely when `BEFORE` equals `AFTER` from Step 2** — with no new commits the lockfile
cannot have changed, so there is nothing to install. When the pull did bring new commits, install so any
newly added packages from the tooling library are available:

```bash
npm --prefix .cursor install
```

- This targets only the shared root manifest (`.cursor/package.json` + `.cursor/package-lock.json`),
  used by skill scripts (Jira/Confluence CLI, FSD/ISD templating, markdown round-trip, diagrams, etc.).
- If `npm install` fails, stop and report the exact failing command and error. Do not continue to
  OpenSpec refresh or project-context sync with a broken dependency tree.

---

## Step 4 — Refresh the OpenSpec schema and config from the library (only if the pull brought new commits)

**Skip this step entirely when `BEFORE` equals `AFTER` from Step 2** — the templates are shipped in the
tooling repo, so with no new commits the materialized schema/config is already current. When the pull did
bring new commits, re-materialize the tooling-owned OpenSpec IP into the code repo's local (gitignored)
`openspec/`:

```bash
node .cursor/.project/scripts/init.mjs --sync-openspec
```

This force-overwrites `config.yaml`, `.gitignore`, and the entire `schemas/` tree under the project's `openspec/` from
`.cursor/.project/templates/openspec/`. The target location is resolved from `PROJECT_CODE_PATH` in
`.env` (so a monorepo `openspec/` under a subfolder is handled automatically). `specs/` and `changes/`
are left untouched.

- If it reports `openspec/ not found`, the code repo isn't initialized — tell the user to run
  `/project-init` first, then stop.

---

## Step 5 — Refresh project-context and reindex (always runs)

This step is NOT git-gated: Confluence/Jira and local `openspec/specs` change independently of the
`.cursor` branch, so run it even when Steps 3 and 4 were skipped. It is already incremental — Atlassian
sync fetches only records changed since the last sync, and the indexer re-embeds only files whose content
hash changed — so a no-op run is cheap. Run the sync/index commands from
`.cursor/.project/project-context`.

```bash
cd .cursor/.project/project-context
npm run sync
```

- `npm run sync` already runs the incremental indexer as its final stage, so no separate `npm run index`
  is needed here.
- If `npm run sync` is unavailable, skip it and run `npm run index`; report that source sync was skipped.
- If credentials are missing or Atlassian sync fails, stop and report the exact failing command. Do not
  start OpenSpec discovery against a stale or partially synced index.
- If indexing fails, stop and report the exact failing command and error summary.

---

## Step 6 — Report concise result

```
## Sync Complete

**Branch:** project/<name>
**Tooling:** pulled N commit(s) (or already up to date)
**Dependencies:** npm install at .cursor root (ok) — or "skipped (no new commits)"
**OpenSpec schema/config:** refreshed from library (specs/changes untouched) — or "skipped (no new commits)"
**Project context:** synced and indexed
```

If the pull brought no new commits, report "Tooling already up to date — skipped deps + schema refresh;
project-context reindexed."
