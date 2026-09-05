// Demo chapter: governance.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';

test('governance chapter', async ({ adminPage }) => {
  const page = new BasePage(adminPage, PATHS.governance);

  await step(
    adminPage,
    'What policy decided, and why',
    {
      what: 'The governance log: one row per evaluated tool call, with the stage that decided it.',
      does: 'Makes an allow or a deny explainable after the fact, to the person it happened to.',
      design: 'The policy that fired is the primary column, because that is the field an operator tunes.',
    },
    async () => {
      await page.goto();
    },
  );
});
