---
name: report-issue
description: File a bug or feedback report about this platform, once per problem, with the call's own
  trace recorded for triage; attach diagnostics when they help, and follow the report's status.
---

# Report an issue

Use when something on this platform is wrong and a person should know: a tool
keeps failing, returns something plainly wrong, refuses where it should not, or
the user asks you to pass on feedback or a bug. The `systemprompt` server's
`report_issue` tool files the report as the signed-in user.

## When to report

- **A real problem, not a single hiccup.** Retry a transient failure once. If it
  fails again the same way, or the result is wrong rather than late, report it.
- **Once per problem.** A repeat with the same fingerprint folds into the open
  report and raises its occurrence count, so never file a fresh report for each
  retry. Pass a stable `fingerprint` (the tool name and error code, for example
  `documents_edit:stale_base`) when the title would vary between attempts.
- **When the user asks.** Feedback and feature requests are `kind: feedback`;
  something broken is `kind: bug`.
- **Not for the user's own mistakes.** A refusal that names what to change (a
  missing argument, a stale revision, no access) is an answer to act on, not a
  bug. Fix the call; report only if the refusal is itself wrong.

## What a good report says

- **Title**: one line naming the failure, for example "documents_edit refuses a
  fresh base revision as stale".
- **Body**: what you were doing and why, what you expected, what happened, and
  whether it repeats. Quote the refusal code and message, not a stack of logs.
- **Severity**: `critical` when work is blocked for everyone, `high` when the
  user's task is blocked, `medium` when there is a workaround, `low` for
  cosmetic issues and suggestions.
- **Client**: a small `client` object (product and version, OS) when you know it.

The tool records the call's trace, context, session, task and MCP execution
itself. Never paste logs, tokens, credentials, customer data or document
contents into the title or body: triage follows the recorded trace instead.

## Attachments

Attach a file only when it adds what words cannot: a zip of diagnostics, a
screenshot, a small sample that reproduces the problem. Use `issue_attachment`:

1. `declare` with the file's name, MIME type, size and SHA-256. The answer
   carries signed upload links valid for fifteen minutes.
2. Send the bytes to those links, part by part, then call the complete URL.
3. `attach` with the `upload_id`.

Strip secrets from anything you attach. A report takes a handful of files at
most, and a resolved report takes none.

## After filing

Tell the user the report id and whether it folded into an existing report.
`issues_list` shows their reports; `issue_status` shows one with its triage
note. A status change arrives in their notifications inbox, so there is no need
to poll.

## Guardrails

- **You report as the user, about the platform.** Never file on someone
  else's behalf, and never report a colleague.
- **A refusal is an answer.** `rate_limited` means stop filing for now;
  `not_active` means the report is closed, so file a new one only if the
  problem is back; `disabled` means this instance takes no reports, so tell the
  user to contact their administrator.
- **No `report_issue` tool means no reach.** The tools sit on the
  admin-scoped `systemprompt` server; when the session has no such tool, tell
  the user to pass the problem to their administrator.
