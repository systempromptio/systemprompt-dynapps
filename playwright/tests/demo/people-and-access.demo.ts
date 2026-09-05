// Demo chapter: people-and-access.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';

test('people-and-access chapter', async ({ adminPage }) => {
  const page = new BasePage(adminPage, PATHS.users);

  await step(
    adminPage,
    'Everyone with an account',
    {
      what: 'The people roster: every account, the groups it belongs to, and what it costs.',
      does: 'Turns "who is using this" into a list an administrator can act on.',
      design: 'One table, one row per person, sortable on every column that a question starts from.',
    },
    async () => {
      await page.goto();
    },
  );
});
