// Demo chapter: people-and-access.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';
import { GroupsPage, GroupDetailPage } from '../support/pages/groups.page';
import { ProjectsPage, ProjectDetailPage } from '../support/pages/projects.page';
import { DevicesPage } from '../support/pages/devices.page';
import { RolesPage } from '../support/pages/roles.page';
import { AccessControlPage } from '../support/pages/access-control.page';
import { UsersPage, UserDetailPage } from '../support/pages/users.page';

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

test('devices chapter', async ({ adminPage }) => {
  const devices = new DevicesPage(adminPage);

  await step(
    adminPage,
    'Every credential the platform has handed out',
    {
      what: 'The device fleet: running bridges, access tokens, device certificates, and connect codes nobody has used yet.',
      does: 'Answers the question an audit starts from — what could still reach this platform, and is anything using it.',
      design:
        'Six fleet tiles above four tabs, then a version histogram drawn from the same counts printed beside it. The tiles do not change with the tab, because the summary is about the estate and not the table below it.',
    },
    async () => {
      await devices.goto();
    },
  );

  await step(
    adminPage,
    'The bridges that have gone quiet',
    {
      what: 'The same table filtered to bridges with no heartbeat in over seven days.',
      does: 'Isolates the machines still holding a valid credential that nobody is watching.',
      design:
        'The KPI tile is the filter. The number an operator reacts to is the control that acts on it, so there is no separate filter to find.',
    },
    async () => {
      await adminPage.goto(`${PATHS.devices}?stale=1`);
    },
  );

  await step(
    adminPage,
    'Personal access tokens, estate-wide',
    {
      what: 'Every token ever issued: its name, the prefix that identifies it, who holds it, when it was last used, and whether it is still live.',
      does: 'Lets an administrator retire a credential for somebody else without asking them to do it.',
      design:
        'The prefix is shown and the secret is not — enough to recognise a token, never enough to use one.',
    },
    async () => {
      await devices.gotoTab('pats');
    },
  );

  await step(
    adminPage,
    'Device certificates and pending links',
    {
      what: 'The certificate a machine presents to enrol, and the connect codes issued but not yet claimed.',
      does: 'Explains a bridge that will not connect: either its certificate was revoked, or its code ran out.',
      design:
        'Expired codes stay listed rather than disappearing, because a code that lapsed is the commonest answer to "why did enrolment fail".',
    },
    async () => {
      await devices.gotoTab('links');
    },
  );
});

test.describe('Projects', () => {
  test('projects chapter', async ({ adminPage }) => {
    const list = new ProjectsPage(adminPage);

    await step(
      adminPage,
      'Where the work is attributed',
      {
        what: 'Every project, its people, the groups feeding it, and what it spent in thirty days.',
        does: 'Answers "which piece of work is costing us, and is the tooling behind it working" in one screen.',
        design:
          'Exclusive attribution, so the rows partition the instance and the tile marked Unattributed carries whatever no project claims — the numbers add up rather than nearly adding up.',
      },
      async () => {
        await list.goto();
      },
    );

    await step(
      adminPage,
      'Sorted by what it costs',
      {
        what: 'The same listing ordered by thirty-day cost, most expensive first.',
        does: 'Turns the page into the answer to a budget question without a filter dialog.',
        design:
          'Sorting is a link, not script: the order is in the URL, so it can be sent to someone else and it survives a reload.',
      },
      async () => {
        await list.goto({ sort: 'cost', dir: 'desc' });
      },
    );

    const detail = new ProjectDetailPage(adminPage, 'storefront');

    await step(
      adminPage,
      'One project: who is on it',
      {
        what: 'Storefront’s membership, each person with their own requests, tokens, cost and last activity.',
        does: 'Shows who is actually doing the work, and which memberships an operator may change.',
        design:
          'This is the one member-attributed table in the console, and the line above it says so — a person on two projects appears under both, and the header tiles above still count them once.',
      },
      async () => {
        await detail.gotoTab('members');
      },
    );

    await step(
      adminPage,
      'One project: what it did',
      {
        what: 'Requests per day, the model mix, the skills used with their ratings, MCP tool health, sessions and the commits that came out of them.',
        does: 'Connects spend to output: the same window, the same people, from the gateway call through to the commit.',
        design:
          'Tool rows carry an error rate rather than a raw failure count, because the count alone does not say whether a server is broken or merely busy.',
      },
      async () => {
        await detail.gotoTab('usage');
      },
    );

    await step(
      adminPage,
      'One project: how it is configured',
      {
        what: 'The project’s name and description, the directory groups that write membership into it, and the entities gated to it.',
        does: 'Puts the two ways membership arrives — by hand and from the directory — on the same page as the name everything is attributed under.',
        design:
          'A directory-written row has no Remove button anywhere in the console: the next sign-in would write it straight back, so the page does not offer an action that would undo itself.',
      },
      async () => {
        await detail.gotoTab('settings');
      },
    );
  });
});

test.describe('Roles', () => {
  test('roles chapter', async ({ adminPage }) => {
    const roles = new RolesPage(adminPage);

    await step(
      adminPage,
      'Roles and permissions',
      {
        what: 'Six role cards with the number of people holding each, five totals across the estate, and every assignment as one row per person per role.',
        does: 'Answers the question a role name alone cannot: who holds it, whether a person or the directory granted it, and what it costs to hold.',
        design:
          'The role cards are the filter for the table under them, so the page has one list rather than six collapsible ones and a person with three roles is three sortable rows.',
      },
      async () => {
        await roles.goto();
      },
    );

    await step(
      adminPage,
      'One role: who holds it',
      {
        what: 'The assignment table filtered to a single role by clicking its card.',
        does: 'Turns "who is an admin here" into a page an auditor can read and export rather than a query someone has to write.',
        design:
          'Selecting a card is a link, not a script: the filtered view has its own URL, so it can be sent to someone.',
      },
      async () => {
        await roles.selectRole('Admin');
      },
    );

    await step(
      adminPage,
      'A directory grant offers no revoke',
      {
        what: 'The same table narrowed to the grants the directory holds.',
        does: 'Shows which grants this console can actually change, and which have to be changed in the directory.',
        design:
          'A directory row carries the word Directory where the Revoke button would be. The console does not offer an action the next sign-in would undo.',
      },
      async () => {
        await roles.goto();
        await roles.filterBy('source', 'directory');
      },
    );

    await step(
      adminPage,
      'What a role opens',
      {
        what: 'Every access-control rule written at the role band: the entity kind, the entity, whether it allows or denies, and whether the entity is open by default.',
        does: 'Connects a role to the marketplaces, MCP servers and gateway routes it actually reaches, instead of leaving that inference to the reader.',
        design:
          'The entity default sits beside the access column because an allow on an already-open entity grants nothing new, and the two columns only mean something together.',
      },
      async () => {
        await roles.goto();
      },
    );
  });
});

test.describe('Access control', () => {
  test('access-control chapter', async ({ adminPage }) => {
    const access = new AccessControlPage(adminPage);

    await step(
      adminPage,
      'Every rule on this instance',
      {
        what: 'The rule ledger: one row per access-control rule, with the subject band it is written at and whether the YAML in the source repository declares it.',
        does: 'Makes configuration drift visible. A rule only this database holds disappears the next time the estate is rebuilt from source.',
        design:
          'The source column is derived by rebuilding the declarative set from the YAML files rather than stored on the row, because the loaders upsert and the last writer of a row is not its only author.',
      },
      async () => {
        await access.goto();
      },
    );

    await step(
      adminPage,
      'One subject band at a time',
      {
        what: 'The ledger filtered to the role band alone.',
        does: 'Separates the four bands — role, group, project and link — that decide access in precedence order, so a reader can audit one without the others in the way.',
        design:
          'Subject kind and entity kind are separate filters because the question is usually about one of them, and combining them into one control would hide half the ledger behind the wrong menu.',
      },
      async () => {
        await access.filterBy('subject_kind', 'role');
      },
    );

    await step(
      adminPage,
      'Granting access to a whole group',
      {
        what: 'The band editor for one group: every entity of every kind, each set to inherit, allow or deny.',
        does: 'Writes one rule that covers everyone in the group, rather than the same rule once per member.',
        design:
          'The editor sits below the ledger, not instead of it: an operator reads what exists before changing it, and the two panes are the same page so the change lands in the list they just read.',
      },
      async () => {
        await access.goto();
        await access.selectGroup('Unassigned');
      },
    );
  });
});

test.describe('Groups', () => {
  test('groups chapter', async ({ adminPage }) => {
    const groups = new GroupsPage(adminPage);
    const detail = new GroupDetailPage(adminPage, 'europe-devs');

    await step(
      adminPage,
      'Every group, and the spend each one accounts for',
      {
        what: 'The group roster: membership, the marketplaces its members inherit, its busiest model, and what it spent in the window.',
        does: 'Answers "where did the money go" at the level a budget is actually held, without opening a single group.',
        design:
          'Spend is attributed exclusively, so each person counts once. The rows and the Unattributed row at the foot therefore add up to what the instance spent — a group listing whose numbers do not reconcile cannot settle an argument.',
      },
      async () => {
        await groups.goto();
      },
    );

    await step(
      adminPage,
      'The traffic no group accounts for',
      {
        what: 'The remainder row: requests with no user, a non-user actor such as a service token, or a person holding no primary group. Here it reads zero, which is the healthy answer — every account is attributed and the group rows already account for the whole estate.',
        does: 'Makes the gap between the group totals and the estate total a number an operator can act on, rather than a discrepancy they have to notice. When it is not zero, the KPI above turns amber past a quarter and the header offers the recompute that closes it.',
        design:
          'It is a row of the same table rather than a footnote, and it is drawn even when it is empty. A remainder that appears only when it is non-zero teaches a reader that the rows always add up, which is exactly the assumption that makes a silent attribution failure invisible.',
      },
      async () => {
        await groups.goto();
        await groups.remainderRow().scrollIntoViewIfNeeded();
      },
    );

    await step(
      adminPage,
      'Ordering the roster by cost',
      {
        what: 'The same table sorted on the Cost column, descending.',
        does: 'Puts the group to ask about first at the top.',
        design:
          'Every column a question starts from is a link the server answers, so the order is in the URL and can be sent to someone. Unassigned sorts to the end whatever the column: it is a derived bucket, not a team, and letting it win a spend sort would be misleading.',
      },
      async () => {
        await groups.goto({ sort: 'cost', dir: 'desc' });
      },
    );

    await step(
      adminPage,
      'One group: usage',
      {
        what: "A single group's usage tab — exclusive totals in the tiles, then the member-level breakdowns: daily volume, spend by member, model mix, top skills and top MCP tools.",
        does: 'Turns a group from a membership list into an account you can read: what it costs, who inside it drives that, and which models and tools it leans on.',
        design:
          'Two attributions live on one tab and the page says so. The tiles are exclusive and partition the estate; the breakdowns are the member view and overlap, because a person in two groups appears in full under both. Each breakdown carries the label rather than relying on a note nobody reads.',
      },
      async () => {
        await detail.gotoTab('usage');
      },
    );

    await step(
      adminPage,
      'One group: members and where they came from',
      {
        what: "The group's membership, each row badged with the source that put them there and carrying their own spend.",
        does: 'Shows who is in the group and lets an administrator add or remove the people they may.',
        design:
          'A directory-sourced membership offers no Remove button. The next sign-in replaces the whole directory set and would write the row straight back, so the control is absent rather than present and failing — and the badge says why.',
      },
      async () => {
        await detail.gotoTab('members');
      },
    );

    await step(
      adminPage,
      'One group: marketplace entitlement',
      {
        what: 'The whole marketplace catalog, ticked where this group reaches it.',
        does: "Grants or withdraws a group's access to a marketplace in one place.",
        design:
          'Entitlement is not a table of its own: it is an ordinary access-control rule keyed by the group, resolved by the same resolver the enforcement point calls. The unticked rows are what make this an editor rather than a receipt, and saving reconciles the whole set so an untick really is a withdrawal.',
      },
      async () => {
        await detail.gotoTab('marketplaces');
      },
    );

    await step(
      adminPage,
      'One group: the directory groups that fill it',
      {
        what: 'The AD groups mapped into this group, with the source of each mapping.',
        does: 'Controls who the directory places here at their next sign-in.',
        design:
          'Only a platform admin sees the controls. A mapping decides what the directory grants, which is the same weight as granting the platform-admin role itself, so it sits at the same tier.',
      },
      async () => {
        await detail.gotoTab('mappings');
      },
    );
  });
});

test.describe('Users', () => {
  test('users chapter', async ({ adminPage }) => {
    const roster = new UsersPage(adminPage);
    const detail = new UserDetailPage(adminPage, 'e2e-member-1');

    await step(
      adminPage,
      'The roster, and what each seat costs',
      {
        what: 'Every account with its roles, groups, projects, last activity and thirty-day gateway spend, fifty to a page.',
        does: 'Answers the two questions an administrator opens this page with — who has access, and is that access being used — without opening a single person.',
        design:
          'One flat table rather than the group-bucketed list this used to be. Bucketing hid the number the page exists to show, and group membership is a column and a filter instead.',
      },
      async () => {
        await roster.goto();
      },
    );

    await step(
      adminPage,
      'The tiles are the filters',
      {
        what: 'Seven tiles: accounts, active, unassigned, no role, idle for thirty days, and the estate’s spend and volume with a change against the previous thirty.',
        does: 'Turns each number an operator reacts to into the click that isolates the people behind it.',
        design:
          'The number and the control are the same element. A separate filter menu would make the reader find twice what they can already see once.',
      },
      async () => {
        await roster.goto();
        await roster.kpi('Unassigned').locator.scrollIntoViewIfNeeded();
      },
    );

    await step(
      adminPage,
      'The people no group accounts for',
      {
        what: 'The roster narrowed to accounts the directory placed in no group.',
        does: 'Isolates the seats whose spend lands in the unattributed bucket on every group and project page.',
        design:
          'Unassigned stopped being a page and became a filter, because it was never a team — it is the absence of one, and a page of its own implied otherwise. The old address redirects here.',
      },
      async () => {
        await adminPage.goto(PATHS.unassigned);
      },
    );

    await step(
      adminPage,
      'Ordered by what a seat costs',
      {
        what: 'The same roster sorted on thirty-day cost, most expensive first.',
        does: 'Puts the account to ask about at the top, with the filter still applied.',
        design:
          'Every column sorts, and sorting is a link the server answers: the order lives in the URL, so it survives a reload and can be sent to someone else.',
      },
      async () => {
        await roster.goto({ sort: 'cost', dir: 'desc' });
      },
    );

    await step(
      adminPage,
      'One person: who they are',
      {
        what: 'The Identity tab: profile, the flat role set, the Active Directory groups the last sign-in asserted, and every linked identity.',
        does: 'Gathers everything that decides what this account may do onto the page that names it.',
        design:
          'Directory groups are shown but not editable. They are the input to group mapping, not a grant, and a control here would be undone by the next sign-in.',
      },
      async () => {
        await detail.openTab('identity');
      },
    );

    await step(
      adminPage,
      'One person: where their spend is counted',
      {
        what: 'The Membership tab: group and project checkboxes, and above them the primary group and project this person is attributed to.',
        does: 'Moves a person between teams, and decides which group and project page their cost lands on.',
        design:
          'Only a container they belong to can be their primary one. Every cost total in the console counts a person exactly once, under these two ids, so this is the single control that can move that money — and saving it pins the choice against the automatic recomputation.',
      },
      async () => {
        await detail.openTab('membership');
      },
    );

    await step(
      adminPage,
      'One person: what can sign in as them',
      {
        what: 'The Devices tab: bridge sessions, device certificates and access tokens in one list, each with when it was last seen.',
        does: 'Answers the question an offboarding starts from — what still holds this person’s credentials — and revokes it here.',
        design:
          'Three tables were three revoke buttons in three places. A bridge session is telemetry rather than a credential, so its row carries no action and says so instead of offering one that would do nothing.',
      },
      async () => {
        await detail.openTab('devices');
      },
    );

    await step(
      adminPage,
      'One person: their sign-in history',
      {
        what: 'Every recorded session, live and revoked, with its source, address, request count and last activity.',
        does: 'Ends one session, or all of them, without a shell.',
        design:
          'A revoked session stays listed. Revoking is an update, not a delete: the row carries that session’s cost and traffic, and ending a session must not rewrite history.',
      },
      async () => {
        await detail.openTab('sessions');
      },
    );

    await step(
      adminPage,
      'One person: what they spent it on',
      {
        what: 'The Usage tab: gateway totals, the models they used, their recent conversations and the commits that came out of them.',
        does: 'Connects a seat to its output — from the request through to the code it produced.',
        design:
          'This was a separate page under Analytics. It was the same account seen from a different sidebar entry, so it is a tab now and the old address redirects to it.',
      },
      async () => {
        await detail.openTab('usage');
      },
    );
  });
});
