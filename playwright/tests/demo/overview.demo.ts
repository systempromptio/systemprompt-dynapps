// Demo chapter: overview.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { OverviewPage } from '../support/pages/overview.page';

test('overview chapter', async ({ adminPage }) => {
  const page = new OverviewPage(adminPage);

  await step(
    adminPage,
    'The console landing page',
    {
      what: 'Six figures for the last 24 hours — requests, spend, active people, p95 latency, error rate and denials — each against the same window a day earlier.',
      does: 'Answers "is anything on fire" before anyone has had to choose a page, and every tile is the door to the rows behind it.',
      design: 'Numbers first, at the top, with the change beside each one. A number with no comparison is a number nobody can act on.',
    },
    async () => {
      await page.goto();
    },
  );

  await step(
    adminPage,
    'Widening the window',
    {
      what: 'The same six figures over 30 days, with every tile and both leaderboards recomputed.',
      does: 'Separates a bad hour from a bad month without leaving the page.',
      design: 'Three windows, not six. The overview is a question about now; a fifteen-minute slice of it is noise.',
    },
    async () => {
      await page.rangeLink('30d').click();
    },
  );

  await step(
    adminPage,
    'Where the money went',
    {
      what: 'The top five projects and the top five groups by spend over the window, side by side, with an explicit unattributed row.',
      does: 'Turns one instance-wide cost figure into the two breakdowns a budget conversation actually needs.',
      design: 'Exclusive attribution: each person counts in one container, so the rows sum back to the spend tile above. Traffic that belongs to no container is named rather than dropped — the row says "no primary project", because dropping it is how a leaderboard quietly stops adding up.',
    },
    async () => {
      await page.section('Top projects by spend').scrollIntoViewIfNeeded();
    },
  );

  await step(
    adminPage,
    'Which MCP servers are answering',
    {
      what: 'Every server declared in services/mcp, with the age of its most recent session heartbeat.',
      does: 'Tells an operator that a configured server has gone quiet — including one nothing has ever reached, which has no row of its own to be found by.',
      design: 'Alive within two heartbeats, stale after that, and "no sessions" for silence. A server nobody used today is not a server that fell over, and the page does not claim it is.',
    },
    async () => {
      await page.section('MCP servers').scrollIntoViewIfNeeded();
    },
  );

  await step(
    adminPage,
    'What is waiting on a person',
    {
      what: 'Calls parked by the require_approval policy, and the usage anomalies the detector raised in the last day.',
      does: 'Surfaces the two queues that only move when a human moves them.',
      design: 'Counted here, resolved on the governance pages. The landing page says something is waiting and where to go; it is not a second place to do the work.',
    },
    async () => {
      await page.section('Waiting on a person').scrollIntoViewIfNeeded();
    },
  );

});
