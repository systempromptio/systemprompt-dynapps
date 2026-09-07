# Agent Guide

How to use this Salesforce Core agent kit on a project.

---

## What is in this kit

```
raw/
    .cursor/
        INDEX.md                          # navigable index of agents / rules / skills
        rules/
            always-active.md              # always-on standards (Apex, LWC, Flow, security, deploy, story)
            story-intake.md               # workflow that triggers on any pasted story
        agents/                           # personas (worker / lead / lane)
        skills/                           # domain knowledge + pattern playbooks
    docs/
        PROJECT_SCHEMA.md                 # template — fill in for the current project
        IMPLEMENTATION_BRIEF_TEMPLATE.md  # pre-implementation brief
        BRANCH_AND_DEPLOY_PRACTICE.md     # full branch / deploy / promotion practice
        AGENT_GUIDE.md                    # this file
        SETUP_GUIDE.md                    # step-by-step Cursor + sf CLI + Node + git setup
    stories/
        README.md                         # one-folder-per-story pattern + templates
    LEARNINGS.md                          # generic patterns / observations distilled from prior projects
    README.md
    .gitignore
```

## Two ways to apply it to a new project

### Option A — Copy the `.cursor/`, `docs/`, and `stories/` folders into the project

The simplest path. Copy:

- `raw/.cursor/` -> `<project-root>/.cursor/`
- `raw/docs/PROJECT_SCHEMA.md` -> `<project-root>/docs/PROJECT_SCHEMA.md`
- `raw/docs/IMPLEMENTATION_BRIEF_TEMPLATE.md` -> `<project-root>/docs/`
- `raw/docs/BRANCH_AND_DEPLOY_PRACTICE.md` -> `<project-root>/docs/`
- `raw/docs/SETUP_GUIDE.md` -> `<project-root>/docs/`
- `raw/stories/README.md` -> `<project-root>/stories/README.md`

Then fill in:

1. `.cursor/skills/Salesforce_Project_Context.md` — project name, custom objects, existing automations, conventions.
2. `docs/PROJECT_SCHEMA.md` — every custom object and field the project owns.

### Option B — Reference this kit as a sibling repo / submodule

Keep this kit as its own repo and pull updates from it into projects via a sync script. Useful for organisations that want one source of truth for the standards.

## Day-one workflow on a new project

1. Apply the kit (Option A above).
2. Open the project in Cursor. Cursor automatically loads `.cursor/rules/always-active.md` and exposes the agents / skills.
3. Read `.cursor/INDEX.md` to see what is available.
4. Fill in `.cursor/skills/Salesforce_Project_Context.md` (project identity, naming, existing automations).
5. Fill in `docs/PROJECT_SCHEMA.md` (objects and fields).
6. Create the first story: `stories/<STORY-ID>/` with the four files (use `stories/README.md`).
7. Create the feature branch (`feature/<STORY-ID>`), implement, deploy, test, PR.

## How the agent uses these files

- **`always-active.md`** is treated as standing instructions on every prompt.
- **`story-intake.md`** triggers automatically when the user pastes a story / requirement.
- **Agents** are persona descriptions; the agent picks one based on the work type (and explicitly states which persona it is assuming).
- **Skills** are read on demand — when the work signals a relevant area (e.g. "validation rule for stage transition" -> `Validation_Rule_Patterns`).
- **`Salesforce_Project_Context`** and **`PROJECT_SCHEMA`** are read first on every task so the agent uses the exact API names and conventions that exist in the project.

## When to update this kit (vs the project)

| Change | Where it goes |
|---|---|
| A pattern that applies to **any** Salesforce project | This kit (`raw/.cursor/skills/`) |
| A convention that applies to **only** the current project | The project's `.cursor/skills/Salesforce_Project_Context.md` |
| A generic improvement to a rule | This kit (`raw/.cursor/rules/`) |
| A project-specific override of a rule | The project's own `.cursor/rules/` (with a comment explaining the override) |

Keep project-specific names (orgs, sandboxes, employees, customer brands, story IDs) **out** of this kit. The kit must remain reusable across projects.
