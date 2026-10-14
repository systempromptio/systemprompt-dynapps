---
name: indaws-git-pr
description: |
  Quick shortcut to push current branch and create a Merge Request (GitLab)
  or Pull Request (GitHub). Auto-generates description from diff. Runs
  pre-commit before push. Equivalent to git-manager action 'push' with
  streamlined input.

  TRIGGER when user says (English or Spanish): "sube los cambios",
  "sube esto", "sube el codigo", "sube el código", "haz push",
  "crea el PR", "crea la PR", "abre el PR", "abre la PR", "abre la MR",
  "haz el PR", "ya esta, sube", "ya está, súbelo", "subelo", "súbelo",
  "subir y crear PR", "push and PR", "open PR", "open MR", "create pull request",
  "create merge request".
version: 1.0.2
model: sonnet
effort: low
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# /git-pr — Create MR/PR from current branch

> **Canonical flow**: see `git-manager/SKILL.md` action `push`. This shortcut is a thin wrapper; **do not duplicate logic here**. If the flow changes, edit the parent action and leave this file as a discoverable entry point.

Shortcut for `git-manager` action `push`. Pushes and creates a Merge/Pull Request with an auto-generated description.

## Usage

```
/git-pr
/git-pr --draft             (creates as draft/WIP)
```

## Step-by-step flow

### 1. Pre-flight checks
- Verify there are commits to push (ahead of remote)
- Check for uncommitted changes — warn if any, suggest committing first
- Detect provider (GitLab/GitHub) from remote URL
- Verify provider CLI installed and authenticated

### 2. Pre-commit gate (mandatory if `.pre-commit-config.yaml` exists)
Run `pre-commit run --all-files` before pushing:
- All pass → proceed
- Auto-fixable failures (formatting) → apply fixes, stage, amend commit, proceed
- Non-fixable failures (lint errors) → stop, show what to fix
- pre-commit not installed → offer to install

### 3. Determine target branch
From `.gitworkflow.yml` → `branches.integration` (usually `develop`). If no config, ask once.

### 4. Generate MR/PR title
Extract from branch name and commit messages:
- Branch `feature/P42-12345-add-invoice-validation` → title: `feat(invoice): add invoice validation [P42-12345]`
- Use conventional commits format from commit-standards.md

### 5. Generate MR/PR description
Read the diff (`git diff {target}...HEAD`) and auto-generate using mr-template.md:
- Summary (1-3 lines, why not what)
- Task ID (extracted from branch name)
- Changes checklist (auto-detected: Models, Views, Security, Tests, etc.)
- How to validate locally (suggest based on changed files)
- Show the generated description and ask for confirmation before creating

### 6. Push and create
```bash
git push -u origin {current_branch}
# Then create MR/PR via provider CLI
```

### 7. Report
Show the MR/PR URL and next steps: "MR creado: {url}. Siguiente paso: pedir review a un compañero."

## Gotchas

- **Sin commits para subir** — si tu rama ya está en remoto y sin cambios nuevos, `/git-pr` no hace nada. No es un bug; es la forma correcta. Comprueba con `/git-status`.
- **Pre-commit obligatorio, sin `--no-verify`** — si `.pre-commit-config.yaml` existe y falla algún hook no auto-fixable, el skill se detiene y te muestra qué arreglar. Nunca se salta la validación.
- **Título auto-generado es propuesta, no dogma** — el skill infiere el título del PR a partir del nombre de rama + commits. Lo muestra antes de crearlo y te pide confirmación. Puedes editarlo.
- **`--remove-source-branch` solo en ramas temporales** — nunca lo aplica a ramas protegidas. Tampoco cuando la rama base no está configurada como temporal en `.gitworkflow.yml`.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| (ninguna — atajo de usuario directo) | — | — |

## Delegates to

- [../git-manager/references/mr-template.md](../git-manager/references/mr-template.md) — MR description template
- [../git-manager/references/commit-standards.md](../git-manager/references/commit-standards.md) — title format
- [../git-manager/references/providers.yml](../git-manager/references/providers.yml) — provider commands
- [../git-manager/references/gotchas.md](../git-manager/references/gotchas.md) — never `--remove-source-branch` on permanent branches

See common:references/shared/ for shared ecosystem patterns.
