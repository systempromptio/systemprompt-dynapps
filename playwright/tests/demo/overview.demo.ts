// Demo chapter: overview.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';

test('overview chapter', async ({ adminPage }) => {
  const page = new BasePage(adminPage, PATHS.overview);

  await step(
    adminPage,
    'The console landing page',
    {
      what: 'The overview: spend, request volume and policy activity across the last 30 days.',
      does: 'Answers "is anything on fire" before anyone has to choose a page.',
      design: 'Four KPI tiles above the fold, then the trend. Numbers first, detail on demand.',
    },
    async () => {
      await page.goto();
    },
  );
});
