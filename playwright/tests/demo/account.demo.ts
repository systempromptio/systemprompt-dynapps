// Demo chapter: account.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';

test('account chapter', async ({ developerPage }) => {
  const page = new BasePage(developerPage, PATHS.profile);

  await step(
    developerPage,
    'A person’s own account',
    {
      what: 'The profile page as an ordinary member of staff sees it: their groups, devices and keys.',
      does: 'Gives everyone a self-service view of what they hold, without an admin round trip.',
      design: 'The same components as the admin pages, scoped to one person; nothing here is a second design.',
    },
    async () => {
      await page.goto();
    },
  );
});
