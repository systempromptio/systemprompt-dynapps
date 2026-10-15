# Merge/Pull Request Template

This template is provider-agnostic — the skill uses the detected provider's CLI command from `providers.yml` to create the MR/PR.

## Template

```markdown
## Summary
<!-- What was done and why. Focus on the "why" — the diff shows the "what". -->

## Task
[{task_id}](task-url)

## Changes
<!-- Check what applies. Remove unchecked items. -->
- [ ] Models (Python) — new/modified models or fields
- [ ] Views (XML) — form, list, kanban, search views
- [ ] Security (ir.model.access / record rules)
- [ ] Data (XML/CSV) — initial data, configuration
- [ ] Tests — unit tests, tour tests
- [ ] Assets (JS/CSS/SCSS) — frontend changes
- [ ] Reports (QWeb PDF/HTML)
- [ ] Controllers / API endpoints
- [ ] Migrations (pre/post/end)

## How to validate locally
<!-- Numbered steps so the reviewer can reproduce and verify. -->
1. ...
2. ...
3. Expected result: ...

## Screenshots
<!-- Required for UI changes. Remove this section if no UI impact. -->

| Before | After |
|--------|-------|
|        |       |

## Post-merge considerations
<!-- Remove this section if not applicable. -->
<!-- For Core modules: does this affect downstream client repos? -->
<!-- Does any dependency, skeleton reference, or documentation need updating? -->
```

## How to fill each section

### Summary

Write what the MR/PR does and **why**. If it is a fix, explain what was broken. If it is a feature, explain what the user can now do. The reviewer should understand the motivation without reading the diff.

Good: "Adds a margin percentage field to sale orders, computed on confirmation. Requested by ACME's sales manager to track profitability without exporting to Excel."

Bad: "Added field and view changes."

### Task

The task ID is extracted from the branch name using the config's task pattern. Build the full tracker link when the URL pattern is known.

### Changes

Check only the categories that apply. This gives the reviewer a quick map of what areas to focus on. The skill can auto-detect applicable items by analyzing the diff — file paths and extensions reveal which categories are touched.

### How to validate locally

This is the **most valuable section** for the reviewer. Write numbered steps they can follow to test the change:

- **Setup** — any configuration, data, or preconditions needed
- **Action** — what to do (navigate, click, API call, shell command)
- **Expected result** — what should happen after the action

If there are edge cases you considered or areas of uncertainty, mention them so the reviewer knows where to focus.

### Screenshots

For any UI change, use the Before/After table. Side-by-side comparison makes it obvious what changed. For non-UI changes, remove the section entirely.

### Post-merge considerations

Use this section when the change has consequences beyond the MR/PR itself:

- **Core modules** — does this change a field, model, or method that client repos inherit? List which modules may need updating
- **Dependencies** — does `dyn-build.yaml` or the skeleton need a version bump?
- **Breaking changes** — do downstream teams need to adapt? (also mark with `!` in the MR/PR title)
- **Documentation** — does any wiki, README, or knowledge article need updating?
- **Monitoring** — should someone watch logs or dashboards after deploy?

For simple feature MR/PRs in client repos, remove this section.

## Suggesting reviewers

If the repo has a `CODEOWNERS` file (root or `.github/CODEOWNERS`), read it to identify who should review based on the changed file paths. The format maps glob patterns to usernames or groups:

```
# Example CODEOWNERS
parts/custom/   @team-lead
*.py            @senior-dev
```

Use CODEOWNERS to suggest a reviewer when creating the MR/PR, but always let the user confirm the assignment.

Always ask the user for the summary, validation steps, and post-merge notes before creating the MR/PR. The changes checklist can be auto-detected from the diff.
