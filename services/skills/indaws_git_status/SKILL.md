---
name: indaws-git-status
description: >-
  Snapshot del estado del repo: rama actual, cambios sin commit, commits
  ahead/behind, MRs/PRs abiertas, estado del pipeline y sugerencia de
  siguiente paso. Equivalente a git-manager action 'status'.
  Triggers: "donde estoy", "que tengo pendiente", "estado del repo",
  "como esta mi PR", "que tengo en marcha", "siguiente paso", "que hago ahora",
  "where am I", "what's next", "status", "repo status".
version: 1.0.2
model: sonnet
effort: low
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# /git-status — Full repo snapshot

> **Canonical flow**: see `git-manager/SKILL.md` action `status`. This shortcut is a thin wrapper; **do not duplicate logic here**. If the flow changes, edit the parent action and leave this file as a discoverable entry point.

Shortcut for `git-manager` action `status`. Shows everything you need to know about where you are and what to do next.

## Usage

```
/git-status
```

## Step-by-step flow

### 1. Branch info
```bash
git branch --show-current
git log --oneline -1          # last commit
git rev-list --left-right --count origin/{base}...HEAD  # ahead/behind
```

Display: current branch, last commit, how many commits ahead/behind the base branch.

### 2. Working tree
```bash
git status --short
```

Display: staged, modified, untracked files. If clean, say "Directorio limpio, sin cambios pendientes."

### 3. Open MRs/PRs
Query provider CLI for open MRs/PRs authored by the current user:
- GitLab: `glab mr list --author=@me`
- GitHub: `gh pr list --author=@me`

Show: title, target branch, pipeline status, approvals.

### 4. Pipeline status (current branch)
If there's a CI pipeline for the current branch, show its status (passed/failed/running).

### 5. Workflow context
Detect which flow the user is in by matching branch name against config patterns:
- `feature/*` → "Estás trabajando en una feature"
- `hotfix/*` → "Estás en un hotfix — recuerda: producción primero"
- `release/*` → "Estás en un release — no añadas features nuevas"
- Integration/production branch → "Estás en una rama protegida — no hagas commits directos"

### 6. Next-step suggestion
Based on the state, suggest the logical next action:
- Uncommitted changes → "Haz commit con `/git-manager commit`"
- Commits not pushed → "Pusha con `/git-pr` para crear MR"
- MR open, pipeline green → "Tu MR está lista para review"
- MR approved → "Tu MR está aprobada, pide merge al maintainer"
- Branch behind base → "Sincroniza con `/git-sync`"

## Gotchas

- **MR/PR no tuya no se lista** — el skill filtra por `@me`. Si trabajas con PRs ajenas (p. ej. revisando), usa directamente `gh pr list` / `glab mr list`.
- **Pipeline en curso aparece como "running"** — no está roto, está esperando. El skill te sugiere volver a ejecutar `/git-status` en unos minutos si el pipeline es largo.
- **Rama sin tracking remoto** — si la rama local nunca se ha pusheado, `ahead/behind` no aplica. El skill lo detecta y sugiere el primer push con `/git-pr`.
- **No modifica nada** — `/git-status` es puramente lectura. Nunca hace fetch, pull, ni altera el working tree. Si necesitas traer cambios, usa `/git-sync`.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| (ninguna — atajo de usuario directo) | — | — |

## Delegates to

- [../git-manager/references/providers.yml](../git-manager/references/providers.yml) — provider commands

See common:references/shared/ for shared ecosystem patterns.
