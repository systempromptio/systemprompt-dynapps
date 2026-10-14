---
name: indaws-git-start
description: >-
  Crea una rama de feature con el naming correcto leyendo .gitworkflow.yml (o
  pregunta interactivamente). Shortcut de git-manager action 'branch'.
  Triggers: "empezar tarea", "voy a empezar con P42-XXX", "nueva tarea",
  "crea rama para P42-XXX", "abre rama para esta tarea", "tengo una tarea nueva",
  "start task", "new task P42-XXX", "create feature branch", "start working on".
version: 1.0.2
model: sonnet
effort: low
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# /git-start — Start working on a task

> **Canonical flow**: see `git-manager/SKILL.md` action `branch`. This shortcut is a thin wrapper; **do not duplicate logic here**. If the flow changes, edit the parent action and leave this file as a discoverable entry point.

Shortcut for `git-manager` action `branch`. Creates a feature branch with the correct naming convention.

## Usage

```
/git-start P42-12345
/git-start                  (asks for task ID interactively)
```

## Step-by-step flow

### 1. Read workflow config
Look for `.gitworkflow.yml` in repo root.
- **Found**: read `naming.task_pattern` to know the expected format (e.g., `P{project_id}-{task_id}`)
- **Not found**: ask the user: "¿Qué formato de tarea usáis? (ej: P42-12345, JIRA-110)" — then offer to generate the config file via `git-manager init`

### 2. Validate task ID
If the user passed a task ID as argument:
- Validate against the pattern from config
- If invalid format: show the expected format and ask again

If no argument:
- Ask: "¿Cuál es el ID de la tarea?"

### 3. Determine base branch
From config `branches.integration` (usually `develop` or `main`). Ensure it's up to date:
```bash
git fetch origin
git checkout {base_branch}
git pull origin {base_branch}
```

### 4. Create feature branch
```bash
git checkout -b feature/{task_id}-{slug} {base_branch}
```

Ask for a short slug (2-4 words) describing the feature. Suggest one from the task ID context if possible.

### 5. Confirm
Report: "Rama `feature/{task_id}-{slug}` creada desde `{base_branch}`. Puedes empezar a trabajar."

## Gotchas

- **Ya estás en una feature branch** — si invocas `/git-start` desde una rama que no es la de integración (develop / 18.0-develop), el skill te avisará antes de crear otra. Es fácil olvidar volver a `develop` entre tareas.
- **Base desactualizada sin darte cuenta** — el skill siempre hace `git pull` de la rama de integración antes de ramificar. Si hay conflictos locales sin commitear en `develop`, se detiene y te pide que los resuelvas primero.
- **Slug con caracteres prohibidos** — el slug descriptivo de la rama (`feature/P42-55001-add-special-discount`) usa solo `[a-z0-9-]`. Si escribes mayúsculas, acentos o espacios, el skill los normaliza automáticamente y te muestra el nombre final antes de crearla.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| (ninguna — atajo de usuario directo) | — | — |

## Delegates to

All git knowledge, provider detection, and guardrails come from the `git-manager` skill. Read its references when needed:
- [../git-manager/references/providers.yml](../git-manager/references/providers.yml) — provider detection
- [../git-manager/references/gotchas.md](../git-manager/references/gotchas.md) — branch gotchas

See common:references/shared/ for shared ecosystem patterns.
