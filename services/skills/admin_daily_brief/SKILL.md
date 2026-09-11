---
name: admin-daily-brief
description: The morning read on the portfolio — a traffic light per project from live Jira evidence and the Confluence record behind it, plus this platform's AI usage.
---

# Daily brief

Use for "what do I need to know this morning", "which projects are in trouble",
or any standing daily check on the portfolio.

**Every invocation is a fresh brief.** Re-read Jira, Confluence and the usage
report each time, even if you ran this earlier in the conversation — the reader
asked again because they want the current state, not a pointer to the last one.

You read Jira and Confluence **directly**, through the `atlassian` MCP server
this session already holds. There is no server-side report for delivery data:
the platform deliberately does not proxy Atlassian, so the thresholds below are
yours to apply, exactly as written.

## Step 1 — Resolve the scope

1. `getAccessibleAtlassianResources` → the sites this account can reach. Every
   payload from this server is wrapped in a `data` object, so the sites are at
   `data.resources[]`, and each carries `cloudId` (**not** `id`). There is no
   site `url` — do not try to build browse links from one.
2. `listJiraProjects` with that `cloudId` → `data.values[]`, paginated with
   `startAt` / `isLast` / `total`. Page until `isLast` is true; an administrator
   routinely has more projects than one page holds, and briefing page one while
   calling it the portfolio is the failure this step exists to prevent.
3. Ask which site if there is more than one. Never invent a `cloudId` or a
   project key — an invented key returns an empty project, which reads exactly
   like a healthy one.

## Step 2 — Read the Jira evidence

For each project, `searchJiraIssuesUsingJql`:

```json
{"cloudId":"<id>", "jql":"project = \"<KEY>\" ORDER BY key ASC",
 "fields":["summary","status","assignee","duedate","updated","resolutiondate"],
 "view":"full", "maxResults":100}
```

**`view: "full"` is mandatory.** The default is `"compact"`, which returns
`status` as `{"name":"Open"}` with no `statusCategory` and drops `duedate`,
`updated` and `resolutiondate` entirely — whatever the `fields` list asks for.
Under the default every count below reads zero and a busy project scores green.
If you find yourself with 0 overdue, 0 stalled and 0 done across a large sample,
check that you sent `view: "full"` before you believe it.

Issues are at `data.issues[]`; page with `data.nextPageToken` until
`data.isLast` is true, or until you have read 2,500 issues — then say the sample
is truncated and treat every count as a floor.

**`sample` is the length of `data.issues[]` summed over the pages you read.
Count it; never estimate it from key ranges** — keys have gaps, and a brief
that says 94 for a project with 76 issues is wrong in a way the reader can
check in one click.

**The server strips null fields.** Under `view: "full"` an issue with no due
date has no `duedate` key at all, and one that is unresolved has no
`resolutiondate`. An absent key means the field is unset — count it that way
and say so ("no due dates set on any of the 76 issues"). Do not read the
absence as a view or permission problem: the view problem shows up as a missing
`statusCategory`, never as a missing date.

## Step 3 — Read the Confluence record

Jira says what is happening; Confluence says what the team believes is
happening, and the gap between them is usually the finding.

- `listConfluenceSpaces` → `data.results[]`, paged by the cursor in
  `data._links.next`.
- For the project's space, find the record with `searchConfluenceUsingCql`:

  ```json
  {"cloudId":"<id>",
   "cql":"space = \"<KEY>\" AND type = page ORDER BY lastmodified DESC",
   "limit":10}
  ```

  Then `getConfluenceContent` with `detail: "full"` on the pages whose titles
  read as status reports, decision logs or meeting notes. The space homepage is
  not the record — a space whose only page is the default welcome text has no
  Confluence record, and the brief must say exactly that rather than "no status
  reports exist" after reading one page.

Where a status page claims a colour that the issues contradict — a page saying
green over a project with a quarter of its work overdue — report the
**discrepancy** as the finding. Do not silently prefer one source. Say what each
said and let the reader judge.

## Step 4 — Derive the traffic light

Compute, over the issues you actually read:

- `sample` = issues in the sample
- `unreadable` = issues whose `fields.status.statusCategory.key` is absent
- `unresolved` = issues whose category is `new` or `indeterminate`
- `overdue` = unresolved with `duedate` earlier than today
- `stalled` = unresolved whose `updated` is more than 7 days ago
- `unassigned` = unresolved with a null `assignee`

Then, in this order — the first match wins:

| condition | light |
|---|---|
| `sample == 0`, or `unreadable == sample` | **unknown** |
| `overdue > 0` and `overdue * 100 / sample >= 10` | **red** |
| `stalled * 100 / sample >= 25`, or `unassigned * 100 / sample >= 25`, or `overdue > 0` | **amber** |
| otherwise | **green** |

Proportions, never raw counts: a hundred-issue project with three overdue items
is healthier than a five-issue project with three, and a count threshold ranks
them the wrong way round.

**`unknown` is a real answer and green is never the default.** A project whose
statuses you cannot classify, or that you could not read at all, is unknown. Do
not map status *names* onto categories to fill the gap — a site renames statuses
freely, so "Open" means different things in different projects and a guess moves
a light.

The **portfolio** light is the WORST light across every project, never an
average. An average reports green while one project burns.

## Step 5 — AI usage

Call `admin_report` with `{"report":"costs","days":7}` — on every run, never a
result reused from an earlier turn. This is platform data from our own audit
tables — no third party — so it is exact. Money arrives twice: `*_microdollars`
is the raw store and its `*_usd` sibling is the figure to quote; never convert
the raw value yourself. State the period alongside every figure, and treat a
very large percentage change as a near-zero-baseline artefact rather than a
real trend unless the absolute numbers support it.

## Result

Lead with the portfolio light and the count of red projects. Put the sample
size and whether it was complete (`isLast`) next to every project's light. Then:

1. **Needs attention today** — every red project, one line each, naming the
   issues by key that drive the light and who owns them. A red light with no
   named issue behind it is not actionable.
2. **Drifting** — the amber projects and the single signal that moved them.
3. **Steady** — the green ones as a list of names. Do not narrate them.
4. **Could not be read** — the unknown ones, and why.
5. **Status vs evidence** — any project whose Confluence status disagrees with
   its Jira issues.
6. **AI usage** — spend and adoption for the period, with the period stated.

## Guardrails

- **Read-only, both systems.** Never transition an issue, edit a page, or change
  configuration. If asked for a change, show it and wait for a clear yes.
- **You see what the caller sees.** Both connectors run as their own linked
  account, so a project you cannot read is out of scope, not missing.
- **Show your arithmetic when a light is red or amber.** The reader must be able
  to check the number against the table above.
- **Truncated is not complete.** Say "at least N" whenever you stopped paging.
- **Demo and fixture data is not delivery evidence.** Issues titled `[DEMO]` or
  drawn from a seeded fixture project must be called out as such; do not present
  a light derived from them as a real assessment.
