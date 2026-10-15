---
name: indaws-git-sync
description: >-
  Sincroniza la rama feature actual con la base (develop/main): elige rebase o
  merge según .gitworkflow.yml. Shortcut de git-manager action 'sync'.
  Triggers: "actualiza mi rama", "sincroniza con develop", "rebase con develop",
  "mi rama esta desactualizada", "trae los cambios de develop", "ponme al dia",
  "sync branch", "rebase my branch", "update my branch".
version: 1.0.2
model: sonnet
effort: low
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# /git-sync — Sync branch with base

> **Canonical flow**: see `git-manager/SKILL.md` action `sync`. This shortcut is a thin wrapper; **do not duplicate logic here**. If the flow changes, edit the parent action and leave this file as a discoverable entry point.

Shortcut for `git-manager` action `sync`. Brings your feature branch up to date with the integration branch.

## Usage

```
/git-sync
```

## Step-by-step flow

### 1. Check preconditions
- Verify no uncommitted changes (`git status`). If dirty, ask the user to commit or stash first.
- Identify current branch — must be a feature/hotfix branch, NOT a protected branch.

### 2. Determine sync strategy
From `.gitworkflow.yml` → `merge_requests.method`:
- **squash** → use `git rebase` (clean history, since commits will be squashed anyway)
- **ff** (fast-forward) → use `git merge` (preserve individual commits)
- **No config** → ask once, suggest rebase as default

### 3. Fetch latest
```bash
git fetch origin
```

### 4. Sync
**Rebase strategy:**
```bash
git rebase origin/{base_branch}
```

**Merge strategy:**
```bash
git merge origin/{base_branch}
```

### 5. Handle conflicts
If conflicts arise:
- Show which files conflict
- Guide through resolution (open, fix, `git add`, continue)
- Link to recovery playbook #6 in git-manager for complex cases

### 6. Report
"Rama sincronizada con `{base_branch}`. {n} commits nuevos integrados." or "Sin cambios — tu rama ya estaba al día."

## RULES

- **NEVER** sync a protected branch (integration/production) — `/git-sync` only operates on feature/hotfix branches.
- **NEVER** use `git push --force`; if a rebase rewrites pushed history, use `--force-with-lease` and warn first.
- **ALWAYS** verify the working tree is clean (commit or stash) before rebasing — rebase with uncommitted changes aborts.

## Gotchas

- **Trabajo sin commitear** — `/git-sync` hace rebase, y rebase con cambios sin commitear aborta. El skill verifica primero y te pide commitear o hacer stash antes de continuar.
- **Conflictos durante el rebase** — si hay conflictos, el skill NO los resuelve automáticamente. Te los muestra y te indica cómo continuar (editar, `git add`, `git rebase --continue`) o cómo abortar (`git rebase --abort`).
- **Rama ya pusheada** — si la feature branch ya está en remoto y otros la están viendo, un rebase reescribe el historial y obliga a `push --force-with-lease`. El skill te avisa antes de hacerlo; no usa `push --force` a secas.
- **Estrategia según merge method** — el skill lee `merge_requests.method` de `.gitworkflow.yml`: `squash` → rebase, `ff` → merge. Si fuerzas la estrategia equivocada el historial puede quedar inconsistente con lo que espera la MR.
- **Rama protegida como objetivo** — si invocas `/git-sync` desde develop/main, el skill se detiene: sincronizar una rama de integración no tiene sentido y podría reescribir historial compartido.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| (ninguna — atajo de usuario directo) | — | — |

## Delegates to

- [../git-manager/references/git-concepts.md](../git-manager/references/git-concepts.md) — sync strategies, merge method implications
- [../git-manager/references/recovery.md](../git-manager/references/recovery.md) — conflict resolution (playbook #6)

See common:references/shared/ for shared ecosystem patterns.
