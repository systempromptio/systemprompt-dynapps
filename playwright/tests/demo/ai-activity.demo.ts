// Demo chapter: ai-activity.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';

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
