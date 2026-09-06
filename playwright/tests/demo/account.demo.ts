// Demo chapter: account.
//
// The four pages a person meets as themselves rather than as an administrator,
// in the order they meet them: the setup guide that tells them what to do, the
// profile that hands them the credential to do it, the conversations that
// result, and the settings that shape how they read.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { ProfilePage } from '../support/pages/profile.page';
import { HistoryPage } from '../support/pages/history.page';
import { SettingsPage } from '../support/pages/settings.page';
import { SetupPage } from '../support/pages/setup.page';

test('account chapter', async ({ developerPage }) => {
  const setup = new SetupPage(developerPage);
  const profile = new ProfilePage(developerPage);
  const history = new HistoryPage(developerPage);
  const settings = new SettingsPage(developerPage);

  await step(
    developerPage,
    'The setup guide',
    {
      what: 'Four phases from a fresh account to a governed skill library, with the plugin and skill counts that decide which phase you are on.',
      does: 'Answers "what do I do first" without a person having to ask, and keeps answering it as they progress.',
      design: 'Completion is derived from what actually reaches the account, so the status column cannot say Complete about something that has not happened. It is a table, not a stepper graphic, because four rows of real status beat four decorated circles.',
    },
    async () => {
      await setup.goto();
    },
  );

  await step(
    developerPage,
    'A person’s own account',
    {
      what: 'The profile page as an ordinary member of staff sees it: spend across three windows, the models they use, the marketplaces and agents that reach them, and the identity facts a support call asks for.',
      does: 'Gives everyone a self-service view of what they hold and what they cost, without an admin round trip.',
      design: 'The same page header, KPI row and tables as every admin page, scoped to one person. There is no second design for the account section.',
    },
    async () => {
      await profile.goto();
    },
  );

  await step(
    developerPage,
    'Minting a connect code',
    {
      what: 'The connect code and the one-line installer it goes into, revealed only after the button is pressed.',
      does: 'Signs a machine in as this person and syncs their organization’s skills, plugins and MCP servers.',
      design: 'Redeeming a code yields a durable token signing in as its owner, so it is a bearer credential. It is minted on request and never on page load — this page gets left open, reloaded and screen-shared.',
    },
    async () => {
      await profile.issueConnectCode();
    },
  );

  await step(
    developerPage,
    'Your conversations',
    {
      what: 'Every Claude Code session and gateway conversation this person owns, with model, turns, tokens and cost per row.',
      does: 'Lets anyone audit their own AI usage, and lets a manager audit their members’ — the same page, a wider scope.',
      design: 'The header states whose conversations these are, because the same page shows a different set to an admin. Snippets pass the transcript redactor before rendering.',
    },
    async () => {
      await history.goto();
    },
  );

  await step(
    developerPage,
    'Searching the transcript',
    {
      what: 'Full-text search across the redacted transcripts, narrowing the same table.',
      does: 'Finds the conversation where something was said, months later, without exporting anything.',
      design: 'Search is a filter on the listing rather than a separate results page, so the columns, sorting and pagination a reader has already learned still apply.',
    },
    async () => {
      await history.search('governance');
    },
  );

  await step(
    developerPage,
    'Account settings',
    {
      what: 'Display name, avatar and timezone, with username and email shown but locked.',
      does: 'Lets a person change what is theirs to change, and says plainly who owns the rest.',
      design: 'A locked field is rendered disabled and labelled with why, rather than hidden. Hiding it invites the support ticket asking where it went. The delete-account row sits in its own section behind the shared confirm dialog.',
    },
    async () => {
      await settings.goto();
    },
  );
});
