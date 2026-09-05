// Demo chapter: platform.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';

test('platform chapter', async ({ platformAdminPage }) => {
  const page = new BasePage(platformAdminPage, PATHS.marketplace);

  await step(
    platformAdminPage,
    'What people are allowed to install',
    {
      what: 'The marketplace: plugins and skills, and the groups entitled to each.',
      does: 'Puts the "who may install what" decision in one place instead of in configuration files.',
      design: 'Entitlement is shown as a set of groups, so the answer reads the way the policy is written.',
    },
    async () => {
      await page.goto();
    },
  );
});
