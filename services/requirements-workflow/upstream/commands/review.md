# Review Branch

Perform a comprehensive code review of the current branch compared to `develop`.

## Instructions

Launch the `code-review` subagent with this task:

```
Review all changes in the current branch compared to develop.

1. Run: git diff develop...HEAD --name-only (to get list of changed files)
2. Run: git diff develop...HEAD (to get the full diff)
3. Read each changed file for full context
4. Apply the code-review skill from .cursor/skills/test/code-review/SKILL.md

Review priorities: Security > Performance > Correctness > Clean Code > Tech Debt

Project-specific checks:
- Auto-imports: flag unnecessary imports of React hooks, React Router, React Intl, custom hooks from app/hooks/
- TypeScript strict: no any, no as casting, no @ts-ignore
- Clean Architecture: domain must NOT import from infrastructure or application layers
- Comments: file has a 2-sentence header + one-line intent on public API; inline only for non-obvious *why*; hacks marked `TODO`; no change-narrative, code-restating, or commented-out code (see code-comments)

Output format:
1. Strengths — what's done well
2. Executive Summary — 3-6 key points
3. Blocking Issues — must fix before merge (file:line, why, fix)
4. Important Improvements — should fix soon
5. Nice-to-Have — lower priority
6. Quick Wins — fast fixes with high impact
7. Verdict — Ready to merge / Ready with fixes / Not ready

Scope discipline: Only review code within the diff. Do not flag pre-existing issues outside the change scope.
```
