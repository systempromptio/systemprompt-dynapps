// The contract every demo chapter follows. This file is never run (the demo
// project ignores `_*.demo.ts`); copy it when adding a chapter.
//
// A demo spec is NOT a test. It asserts nothing, it never creates or deletes
// anything, and it must run green against the seeded dataset on any machine.
// Its only job is to walk one section of the console in the order a person
// would meet it, and to explain each stop well enough that a reader who has
// never seen the product understands what they are looking at.
//
// Rules:
//   * One file per section, named exactly `<section>.demo.ts` where <section>
//     is one of SECTIONS in tests/support/demo.ts. The recorder derives the
//     chapter from the filename and refuses any other name.
//   * Every stop goes through `step(page, title, meta, fn)`. `fn` performs the
//     navigation or interaction; the helper screenshots and records afterwards.
//   * `meta` has three fields and all three are prose a reader sees:
//       what   — what is on the screen, in one sentence
//       does   — the job this page does for the person using it
//       design — why it looks and behaves this way
//   * Steps run in file order and appear in the book in that order.
//   * Use PATHS, never a literal URL. Use the page objects, never raw selectors.
//   * The recorder throws if a step lands on the login page, so a chapter that
//     silently lost its session fails loudly instead of shipping figures of a
//     sign-in form.
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
      what: 'The overview: spend, request volume and policy activity for the last 30 days.',
      does: 'Answers "is anything on fire" before anyone has to choose a page.',
      design: 'Four KPI tiles above the fold, then the trend; numbers first, detail on demand.',
    },
    async () => {
      await page.goto();
    },
  );
});
