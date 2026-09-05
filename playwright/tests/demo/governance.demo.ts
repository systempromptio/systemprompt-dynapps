// Demo chapter: governance.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { GovernancePage } from '../support/pages/governance.page';
import { ApprovalsPage } from '../support/pages/approvals.page';
import { SecretsAuditPage } from '../support/pages/secrets-audit.page';

test('governance chapter', async ({ adminPage }) => {
  const governance = new GovernancePage(adminPage);

  await step(
    adminPage,
    'What policy decided, and why',
    {
      what: 'The governance log: one row per evaluated tool call, with the chain stage that decided it.',
      does: 'Makes an allow or a deny explainable after the fact, to the person it happened to.',
      design:
        'The four chain stages get their own tiles in evaluation order, so reading left to right is reading the chain and you can see which stage is doing the work.',
    },
    async () => {
      await governance.goto({ preset: '30d' });
    },
  );

  await step(
    adminPage,
    'Narrowing to one policy',
    {
      what: 'The same log filtered to the secret scanner, the stage most likely to be over-tuned.',
      does: 'Turns "we deny a lot" into the specific rule and the specific people it is catching.',
      design:
        'Every KPI tile is the filter its number describes, so the number and the rows behind it are one click apart and can never disagree.',
    },
    async () => {
      await governance.filterBy('policy', 'secret_scan');
    },
  );

  await step(
    adminPage,
    'The second plane: safety findings',
    {
      what: 'What the gateway scanners saw in both directions, with blocked and audited kept apart.',
      does: 'Shows what warn mode is absorbing — findings with no blocks under them are calls that ran.',
      design:
        'Scanned and blocked are separate columns and separate tiles because under warn mode they stop being the same fact.',
    },
    async () => {
      await governance.openTab('safety');
    },
  );

  await step(
    adminPage,
    'The plumbing underneath',
    {
      what: 'Hook events as they fired, with the policies and people at the top of the deny rankings.',
      does: 'Answers whether governance is actually running before anyone argues about what it decided.',
      design:
        'The rankings sit under the event log rather than beside it: the log is the evidence, the rankings are the summary, and a summary above its evidence invites trusting it unread.',
    },
    async () => {
      await governance.openTab('hooks');
    },
  );

  const approvals = new ApprovalsPage(adminPage);

  await step(
    adminPage,
    'Calls held for a person',
    {
      what: 'The approval queue: tool calls the require_approval policy parked, oldest wait first.',
      does: 'Lets a named human release or refuse a call that a machine would not decide alone.',
      design:
        'A waiting room, not a log. Age is a first-class column and the oldest wait is a tile, because every pending row is a caller blocked right now.',
    },
    async () => {
      await approvals.goto();
    },
  );

  await step(
    adminPage,
    'Approving one, with the arguments in view',
    {
      what: 'The confirmation an approver answers, naming the tool and the exact arguments.',
      does: 'Binds the decision to that payload; a retry that changes the arguments is held again.',
      design:
        'Destructive and irreversible, so it goes behind the shared confirm dialog rather than a bare button — and the arguments are on the row, because an approver who cannot see them is not approving anything.',
    },
    async () => {
      await approvals.approveButton('e2e-dappr-000').click();
    },
  );

  const secrets = new SecretsAuditPage(adminPage);

  await step(
    adminPage,
    'Who touched which credential',
    {
      what: 'Every recorded action against a stored secret: created, read, rotated, deleted.',
      does: 'Answers the auditor question plaintext storage cannot — who read this, and did they own it.',
      design:
        'Owner and actor are two columns and their disagreement is its own tile, because "an administrator read someone else’s credential" is the one fact you cannot count off the table.',
    },
    async () => {
      await secrets.goto({ preset: '30d' });
    },
  );
});
