---
name: admin-daily-brief
description: The morning read on the portfolio — a traffic light per project from live Jira evidence and the Confluence record behind it, plus this platform's AI usage.
---

# Daily brief

Use for "what do I need to know this morning", "which projects are in trouble", or any
daily portfolio check. **Every invocation is a fresh brief** — re-read Jira, Confluence
and usage each time. You read Atlassian directly through the session's `atlassian` MCP
server; the thresholds below are yours to apply exactly.

## 1. Scope

1. `getAccessibleAtlassianResources` → `data.resources[]`, each with `cloudId` (not `id`;
   no site `url`).
2. `listJiraProjects` → `data.values[]`; page with `startAt` until `isLast`.
3. Ask which site if more than one. Never invent a `cloudId` or project key.

## 2. Jira evidence

```json
{"cloudId":"<id>", "jql":"project = \"<KEY>\" ORDER BY key ASC",
 "fields":["summary","status","assignee","duedate","updated","resolutiondate"],
 "view":"full", "maxResults":100}
```

**`view: "full"` is mandatory** — compact drops `statusCategory` and the dates, so a busy
project scores green. All-zero counts over a large sample: check the view first.

Page `data.issues[]` with `data.nextPageToken` until `data.isLast`, or stop at 2,500 and
treat counts as floors. `sample` = issues actually returned, counted, never estimated from
key ranges. Null fields are stripped: a missing `duedate` / `resolutiondate` means unset.

## 3. Confluence record

`listConfluenceSpaces` → `data.results[]` (cursor in `data._links.next`). Then
`searchConfluenceUsingCql` with
`space = "<KEY>" AND type = page ORDER BY lastmodified DESC`, `limit` 10, and
`getConfluenceContent` (`detail: "full"`) on status reports, decision logs and meeting
notes. A space with only the welcome homepage has no record — say exactly that.

## 4. Traffic light

Over the issues read:

- `unreadable` = no `fields.status.statusCategory.key`
- `unresolved` = category `new` or `indeterminate`
- `overdue` = unresolved with `duedate` before today
- `stalled` = unresolved with `updated` more than 7 days ago
- `unassigned` = unresolved with null `assignee`

First match wins:

| condition | light |
|---|---|
| `sample == 0` or `unreadable == sample` | **unknown** |
| `overdue > 0` and `overdue * 100 / sample >= 10` | **red** |
| `stalled * 100 / sample >= 25`, or `unassigned * 100 / sample >= 25`, or `overdue > 0` | **amber** |
| otherwise | **green** |

Proportions, never raw counts. Never map status names onto categories. The portfolio light
is the **worst** project light, never an average.

## 5. AI usage

Call `admin_report` `{"report":"costs","days":7}` on every run. Quote the `*_usd` fields,
never convert `*_microdollars`. A huge percentage change on a near-zero baseline is not a
trend.

## Rules

- Never conclude a day, total or trend from one page: page to the end or say "at least N".
- When Confluence status and Jira evidence (or any two sources) disagree, report both and
  do not invent a cause.
- State the window, and any project or source that was denied, empty or truncated. A
  project you cannot read is out of scope for this caller, not missing.
- Show the arithmetic for every red or amber light.
- `[DEMO]` or fixture issues are not delivery evidence; label any light derived from them.
- Read-only: never transition issues, edit pages or change config.

## Result

Lead with the portfolio light and the number of red projects; put sample size and
completeness (`isLast`) beside every light. Then:

1. **Needs attention today** — each red project, the issue keys driving it, their owners.
2. **Drifting** — amber projects and the signal that moved each.
3. **Steady** — green project names only.
4. **Could not be read** — unknown projects and why.
5. **Status vs evidence** — Confluence status disagreeing with Jira.
6. **AI usage** — spend and adoption, with the period.
