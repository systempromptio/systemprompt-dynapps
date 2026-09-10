---
name: admin-daily-brief
description: The morning read on the portfolio — a traffic light per project from live Jira evidence,
  what changed since yesterday, and the AI usage strip.
---

# Daily brief

Use for "what do I need to know this morning", "which projects are in trouble",
or any standing daily check on the portfolio. It answers in one pass and leaves
behind a dashboard the reader can keep open.

## Workflow

1. **Resolve the scope.** Call `admin_project_catalog` with no arguments to list
   the Atlassian sites this caller can reach, then again with the chosen
   `cloud_id` to list its projects. Ask which site if there is more than one.
   Never guess a `cloud_id` or a project key — an invented key comes back as an
   empty project, which reads exactly like a healthy one.
2. **Read the brief.** Call `admin_report` with
   `{"report":"brief","cloud_id":"<id>","project_keys":["A","B"],"days":7}`.
   It returns one row per project with its light and the counts behind it, the
   portfolio's worst light under `highlights.Portfolio`, and an interactive
   dashboard. The tool is limited to 20 projects per call; for a larger estate,
   brief the ones under active delivery rather than paging through all of them.
3. **Read the usage strip.** Call `admin_report` with
   `{"report":"costs","days":7}` for spend, models, trends and sessions.
4. **Go one level deeper on anything red.** For each red project call
   `admin_report` with `{"report":"projects","cloud_id":"<id>","project_key":"<key>","days":7}`
   and name the specific issues driving the light — the overdue ones by key,
   and the stalled ones by how long they have sat. A red light with no named
   issue behind it is not actionable.

## What the lights mean

The server derives them from the issues, not from a RAG field someone typed:

- **red** — 10% or more of the sample is unresolved and past its due date.
- **amber** — a quarter or more is stalled (untouched for seven days) or
  unassigned, or anything at all is overdue below the red threshold.
- **green** — none of the above.
- **unknown** — no readable issues, or the project could not be read.

`unknown` is a real answer and must be reported as one. If a project could not
be read, say so and say why the source coverage gives — never fold it into green.

## Result

Lead with the portfolio light and the count of red projects. Then, in order:

1. **Needs attention today** — every red project, one line each, naming the
   issues behind the light and who owns them.
2. **Drifting** — the amber projects and the single signal that moved them there.
3. **Steady** — the green ones, as a list of names. Do not narrate them.
4. **Could not be read** — the unknown ones, with the reason from source coverage.
5. **AI usage** — spend and adoption for the period, with the period stated.
6. **Dashboard** — the artifact the report returned.

## Guardrails

- **Read-only, always.** This skill reports; it never transitions an issue,
  edits a page, or changes configuration.
- **You see what the caller sees.** Atlassian runs as their own linked account,
  so a project that is not visible is out of scope, not missing.
- **Report partial data as partial.** If `complete` is false, say so before the
  numbers, not after them — a total from an incomplete sample is a wrong total,
  not a rough one.
- **Never state a light you did not read.** Do not re-derive the thresholds
  yourself or override the server's answer; two implementations of the same rule
  is two different briefs.
- **State the period on every figure.** Both halves take `days`, and a reader
  comparing yesterday's brief to today's needs to know they cover the same window.

## If the dashboard is missing

The paired artifact is `admin-daily-brief`. If the reader does not have it in
their Cowork Artifacts library, run the `systemprompt_setup_admin` skill, which
installs and reconciles it. Do not attempt to create it by hand.
