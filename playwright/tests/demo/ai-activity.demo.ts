// Demo chapter: ai-activity.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test, dbRow } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';
import { RequestsPage, RequestDetailPage } from '../support/pages/requests.page';
import { SessionsPage } from '../support/pages/sessions.page';
import { ContextsPage } from '../support/pages/contexts.page';
import { TracesPage } from '../support/pages/traces.page';
import { AnalyticsPage } from '../support/pages/analytics.page';

test('ai-activity chapter', async ({ adminPage }) => {
  const page = new BasePage(adminPage, PATHS.requests);

  await step(
    adminPage,
    'Every model call, one row each',
    {
      what: 'The request log: provider, model, tokens, cost, latency and outcome for each call.',
      does: 'Lets an operator find the one request behind a complaint or a spike.',
      design: 'Dense rows, monospaced numbers, and the outcome carried by a badge rather than by colour alone.',
    },
    async () => {
      await page.goto();
    },
  );
});

test.describe('Requests', () => {
  test('requests chapter', async ({ adminPage }) => {
    const log = new RequestsPage(adminPage);

    await step(
      adminPage,
      'The request log, attributed',
      {
        what: 'Every /v1/messages call in the window, with the project and the group each one is charged to beside the model, the money and the outcome.',
        does: 'Answers "whose spend was this" without a click. Attribution is exclusive, so a person in two groups is counted once and the totals add up.',
        design: 'Six tiles state the window before the eye reaches the table; twelve columns fit because numbers are tabular and right-aligned and the outcome is a badge.',
      },
      async () => {
        await log.goto({ preset: '30d' });
      },
    );

    await step(
      adminPage,
      'Rejected calls keep their row',
      {
        what: 'The Rejected tile applied as a filter: calls the gateway refused before it resolved a route.',
        does: 'Shows what governance actually stopped. These rows have no provider and no model, and are the ones a "why did that fail" question is usually about.',
        design: 'A missing model is an em dash and a REJECTED badge, never a blank cell — the tile that produced the filter stays marked as active.',
      },
      async () => {
        await log.goto({ preset: '30d', status: 'rejected' });
      },
    );

    await step(
      adminPage,
      'Sorted by what it cost',
      {
        what: 'The same window ordered by billed cost, most expensive first.',
        does: 'Puts the calls worth investigating at the top of page one.',
        design: 'Sorting is a link, so the order is in the URL and can be shared; the header states its direction in aria-sort as well as in the arrow.',
      },
      async () => {
        await log.goto({ preset: '30d', sort: 'cost', dir: 'desc' });
      },
    );

    const row = await dbRow<{ id: string }>(
      'SELECT id FROM ai_requests ORDER BY created_at DESC LIMIT 1',
    );
    if (row) {
      const detail = new RequestDetailPage(adminPage, row.id);
      await step(
        adminPage,
        'One request, end to end',
        {
          what: 'The chain of custody for a single call: identity, the policy decisions, the safety scanner findings, the tools it asked to run, and what it cost.',
          does: 'Is the evidence an audit asks for. The scanners are a separate plane from the policy chain, so a finding that was audited but not blocked still appears.',
          design: 'Breadcrumbs back to the log, one section per plane, and a badge that distinguishes a blocked finding from an audited one.',
        },
        async () => {
          await detail.goto();
        },
      );
    }
  });
});

test.describe('Sessions', () => {
  test('sessions chapter', async ({ adminPage }) => {
    const list = new SessionsPage(adminPage);

    await step(
      adminPage,
      'Every interactive run, one row each',
      {
        what: 'The sessions list: who ran it, which group and project it is charged to, how long it lasted, how many requests, contexts, traces and tool calls it produced, and what it cost.',
        does: 'Is the unit an operator recognises. A complaint arrives as "my session", not as a request id, and this is the page that turns one into the other.',
        design: 'One line per session at sixteen columns: the timestamps are nowrap and tabular, the identity and title columns are capped, and the status is a badge so the eye finds it without reading.',
      },
      async () => {
        await list.goto({ preset: '30d' });
      },
    );

    await step(
      adminPage,
      'Attributed to one group and one project',
      {
        what: 'The Group and Project columns, filled for every row — including the explicit "unattributed" bucket for a person no primary container covers.',
        does: 'Answers "whose was this" at a glance. Attribution is exclusive, so a person in two groups is counted once and the group totals partition the instance.',
        design: 'The unattributed bucket is shown, never dropped: a blank cell would read as missing data, and the total would silently stop adding up.',
      },
      async () => {
        await list.goto({ preset: '30d' });
      },
    );

    await step(
      adminPage,
      'Only the sessions that went wrong',
      {
        what: 'The "With errors" tile applied as a filter: sessions holding a failed gateway request or a hook-reported error.',
        does: 'Is the triage path. It narrows a month of traffic to the handful worth reading, without anyone composing a query.',
        design: 'The tile that produced the filter stays marked active, so the number on screen and the rows below it can never disagree.',
      },
      async () => {
        await list.goto({ preset: '30d', error_only: 'true' });
      },
    );

    await step(
      adminPage,
      'Sorted by what it cost',
      {
        what: 'The same window ordered by spend, most expensive session first.',
        does: 'Puts the sessions worth asking about at the top when a bill is queried.',
        design: 'Every sort link carries the window, the scope and the filters, so a sort never silently widens what is being looked at.',
      },
      async () => {
        await list.goto({ preset: '30d', sort: 'cost', dir: 'desc' });
      },
    );

    const session = await dbRow<{ session_id: string }>(
      'SELECT session_id FROM ai_requests WHERE session_id IS NOT NULL ORDER BY created_at DESC LIMIT 1',
    );
    if (session) {
      const detail = new BasePage(adminPage, PATHS.session(session.session_id));

      await step(
        adminPage,
        'One session, end to end',
        {
          what: 'Everything one run produced: its contexts, its traces and its requests, under eight tiles that count them.',
          does: 'Turns "what happened in that session" into one page instead of four searches. Every id on it links to the entity it names.',
          design: 'Breadcrumbs back to the list, one titled and counted section per table, and a rejected request rendered as an em dash with a REJECTED badge rather than as an error.',
        },
        async () => {
          await detail.goto();
        },
      );

      await step(
        adminPage,
        'Whether the run actually worked',
        {
          what: 'The assessed outcome the hooks pipeline wrote when the run ended, and the ratings the people on it left.',
          does: 'Is the only judgement of outcome the console holds. Everything else on the page counts what happened; this says whether it was any good.',
          design: 'Both panels are always answered — a session still in flight gets a notice, an unrated one an empty state — so an absent verdict never reads as a good one.',
        },
        async () => {
          await detail.goto();
        },
      );
    }
  });
});

test.describe('Contexts', () => {
  test('contexts chapter', async ({ adminPage }) => {
    const list = new ContextsPage(adminPage);

    await step(
      adminPage,
      'Every conversation, with its opening turn',
      {
        what: 'The contexts list: the persisted state of each AI conversation, with the first thing the user actually asked shown beside the counts.',
        does: 'Lets someone find the conversation worth reading without opening twenty of them.',
        design: 'The preview column is the opening user turn with the protocol markers stripped — a raw transcript header would fill the column with noise.',
      },
      async () => {
        await list.goto();
      },
    );

    await step(
      adminPage,
      'The same conversations, grouped by who had them',
      {
        what: 'The by-user view: one row per person, with their contexts nested underneath.',
        does: 'Answers "who is using this" rather than "what happened", which is a different question and needs a different shape.',
        design: 'A tab, not a second page: the filters, the scope and the window carry across so the two views always describe the same rows.',
      },
      async () => {
        await list.goto({ view: 'users' });
      },
    );

    const context = await dbRow<{ context_id: string }>(
      "SELECT context_id FROM ai_requests WHERE context_id IS NOT NULL ORDER BY created_at DESC LIMIT 1",
    );
    if (context) {
      const detail = new BasePage(adminPage, PATHS.context(context.context_id));

      await step(
        adminPage,
        'One conversation, in order',
        {
          what: 'The full transcript — every user turn, assistant turn, tool call and tool result interleaved — plus the requests that produced it.',
          does: 'Is the evidence behind a governance question. An admin sees it unredacted, which is the whole reason this page is separate from the owner-facing one.',
          design: 'Breadcrumbs run list, session, context, so the reader can climb back to whichever scope their question actually lives at.',
        },
        async () => {
          await detail.goto();
        },
      );

      await step(
        adminPage,
        'What that session touched',
        {
          what: 'The files, tools and repositories the hooks pipeline recorded against the session this context belongs to, busiest first.',
          does: 'Turns a transcript into an inventory: what an agent actually reached for, and how often.',
          design: 'A bar drawn against the busiest entity rather than the total, so one dominant file reads as a full bar beside an empty one instead of both being flattened.',
        },
        async () => {
          await detail.goto();
        },
      );
    }
  });
});

test.describe('Traces', () => {
  test('traces chapter', async ({ adminPage }) => {
    const list = new TracesPage(adminPage);

    await step(
      adminPage,
      'Every trace in the window',
      {
        what: 'Gateway calls, governance decisions and tool invocations grouped by the session id they share, with span, request, tool and error counts on every row.',
        does: 'Is the triage surface: which traces were denied, which failed, which were slow, and which cost money.',
        design: 'One line per trace. These rows were two-line stacked cells at 51 pixels, which halved how many traces fitted above the fold; the counts are now tabular columns instead.',
      },
      async () => {
        await list.goto({ preset: '30d' });
      },
    );

    await step(
      adminPage,
      'Only what governance refused',
      {
        what: 'The Denied tile applied as a filter: traces containing at least one denied decision.',
        does: 'Shows what the policy chain actually stopped, which is the question an audit opens with.',
        design: 'Deny and error are separate badges and separate filters — a policy refusal is not a fault to chase upstream, and conflating them sends an investigator the wrong way.',
      },
      async () => {
        await list.goto({ preset: '30d', deny_only: 'true' });
      },
    );

    const trace = await dbRow<{ session_id: string }>(
      'SELECT session_id FROM ai_requests WHERE session_id IS NOT NULL ORDER BY created_at DESC LIMIT 1',
    );
    if (trace) {
      const detail = new BasePage(adminPage, PATHS.trace(trace.session_id));

      await step(
        adminPage,
        'One trace, span by span',
        {
          what: 'The waterfall: every span drawn against the trace\'s own first-to-last window, coloured by kind, with the axis labelled in elapsed time.',
          does: 'Shows where the time actually went — which call was slow, and what was waiting on it.',
          design: 'One inline SVG rather than a stack of positioned divs: the axis is drawn once instead of implied per row, and a zero-duration governance decision still gets a visible tick rather than being widened into a lie.',
        },
        async () => {
          await detail.goto();
        },
      );

      await step(
        adminPage,
        'What each span reported',
        {
          what: 'The same spans as a table, carrying the provider, model, latency, duration and cost each one recorded.',
          does: 'Is where the money in the waterfall comes from, and where a refusal names itself.',
          design: 'A governance or tool span has no provider and no cost, so those cells are em dashes — a zero would read as "free" rather than "not applicable", and a rejected request keeps a REJECTED badge instead of an error one.',
        },
        async () => {
          await detail.goto();
        },
      );
    }
  });
});

test.describe('Analytics', () => {
  test('analytics chapter', async ({ adminPage }) => {
    const analytics = new AnalyticsPage(adminPage);

    await step(
      adminPage,
      'The whole instance on one screen',
      {
        what: 'Six tiles, four charts, the top-user leaderboard, the latency split, the anomaly sweep and the code-impact frames — all over one scope and one window.',
        does: 'Answers "how is this doing" before anyone clicks. The five tabs beside it exist so that this page can afford to load everything and they can each load only their own.',
        design: 'Exclusive attribution by default, so the container figures partition the instance and add back up to it. Nothing here is a proxy dressed as a measurement: AI lines and committed lines sit side by side and are never subtracted.',
      },
      async () => {
        await analytics.openTab('overview');
      },
    );

    await step(
      adminPage,
      'Narrowed to one group',
      {
        what: 'The same page scoped to europe-devs, with the scope shown as a chip that can be removed.',
        does: 'Turns an instance-wide number into a team-wide one. The scope carries on to the request log, the sessions list and every other AI-activity page.',
        design: 'The chip is the only thing on screen that says a total is partial. A narrowed figure with nothing to say so is how people misread a slice as the whole.',
      },
      async () => {
        await analytics.openTab('overview', { group: 'europe-devs' });
      },
    );

    await step(
      adminPage,
      'Counted once, or counted everywhere',
      {
        what: 'The attribution toggle switched to the member view, with the warning it puts on screen when it is.',
        does: 'Lets someone ask "who uses what" without the answer being mistaken for a total. A person in two groups is counted in full in both.',
        design: 'Attribution is a claim about the numbers, not a filter on them, so it is offered as two named readings rather than a checkbox — and the overlapping one announces itself.',
      },
      async () => {
        await analytics.openTab('overview', { attr: 'member' });
      },
    );

    await step(
      adminPage,
      'Every model, including the ones that never ran',
      {
        what: 'Requests, tokens split four ways, cost, p50 and p95 latency, error rate and route redirects for each model — plus an "unrouted" row for the calls the gateway refused before choosing one.',
        does: 'Is where a spend question and a latency question are answered with the same table, and where a route quietly rewriting a model becomes visible.',
        design: 'A rejected call has no provider and no model, so it gets a named bucket and a REJECTED badge rather than being dropped. Dropping it is how 48 of 240 requests vanish from a total nobody re-checks.',
      },
      async () => {
        await analytics.openTab('models');
      },
    );

    await step(
      adminPage,
      'Skill cost is measured, or it is blank',
      {
        what: 'Skill adoption: invocations split into slash commands and tool calls, the people and sessions reached, the measured cost, and an explicit count of what carries no cost at all.',
        does: 'Says how much of the picture is actually priced. On this instance every invocation is a slash command, which sends no tool call, so nothing can be priced and the page says so in the tile.',
        design: 'The same-user time-window heuristic that used to fill this column is gone. It produced a number no one could check; a dash beside a stated count is the honest version, and the notice explains why.',
      },
      async () => {
        await analytics.openTab('skills');
      },
    );

    await step(
      adminPage,
      'Which tools work, and which are slow',
      {
        what: 'MCP execution health by server and by tool: executions, success rate over what has settled, p50 and p95 duration, and how many distinct people call each one.',
        does: 'Is the page for "is that integration healthy". Everything on it is measured — one row per real call, with its own status and duration.',
        design: 'Pending calls are counted in their own column rather than folded into the success rate as failures: a slow tool and a broken one are different problems and must not share a number.',
      },
      async () => {
        await analytics.openTab('tools');
      },
    );

    await step(
      adminPage,
      'What a session actually cost its user',
      {
        what: 'Client-reported session cost and context-window pressure, most expensive first, with the ratings people left beside them.',
        does: 'Is the view a bill query starts from, and the only place the console holds a human judgement of whether a run was any good.',
        design: 'Every figure is labelled client-reported because it is the statusline\'s running total, not a gateway measurement — the two are close, they are not the same, and quoting one as the other is how a reconciliation fails.',
      },
      async () => {
        await analytics.openTab('sessions');
      },
    );

    await step(
      adminPage,
      'The supplier bill',
      {
        what: 'Provider cost by day, then by provider and by model, over the same window and scope as every other tab.',
        does: 'Is the operator\'s half of the cost question: what inference cost us, and which model earned it.',
        design: 'One stacked chart rather than one line per provider: the question is what the total was made of, and a stack answers it without the reader summing anything.',
      },
      async () => {
        await analytics.openTab('cost', { audience: 'internal' });
      },
    );

    await step(
      adminPage,
      'The half that can leave the building',
      {
        what: 'The customer view: what each group or project consumed, with no supplier cost anywhere on it.',
        does: 'Is the export that goes to a customer or a finance contact outside the platform team, with the same CSV button beside it.',
        design: 'The query behind this view selects no cost column at all, so a leak is impossible rather than merely unlikely — and the unattributed bucket is a row, not an omission.',
      },
      async () => {
        await analytics.openTab('cost', { audience: 'customer' });
      },
    );
  });
});
