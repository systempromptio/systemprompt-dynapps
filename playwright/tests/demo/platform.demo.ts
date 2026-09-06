// Demo chapter: platform.
//
// Page agents append their steps to this file, in the order a person meets the
// pages. Read tests/demo/_template.demo.ts for the contract before adding one.
import { test } from '../support/fixtures';
import { step } from '../support/demo';
import { PATHS } from '../support/paths';
import { BasePage } from '../support/pages/base.page';
import { GatewayPage } from '../support/pages/gateway.page';
import { McpDetailPage, McpListPage } from '../support/pages/mcp.page';

const EUROPE = 'astound-europe-dev';
const SERVER = 'systemprompt';

test('platform chapter', async ({ platformAdminPage }) => {
  const page = new BasePage(platformAdminPage, PATHS.marketplaces);

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

  await step(
    platformAdminPage,
    'Declared audience against resolved audience',
    {
      what: 'One marketplace: the groups that hold it, and what the resolver actually decides for each.',
      does: 'Catches the case where a marketplace is granted to a group that a deny rule elsewhere still closes.',
      design: 'The two answers sit in adjacent columns of the same row, because a difference between them is the finding.',
    },
    async () => {
      await platformAdminPage.goto(PATHS.marketplace(EUROPE));
    },
  );

  await step(
    platformAdminPage,
    'Granting a marketplace to a group',
    {
      what: 'Assigning a group to this marketplace, and the row updating to say it is granted.',
      does: 'Entitles everyone in that group to everything the marketplace ships, in one action.',
      design: 'The action is on the row it changes, and the confirmation for the reverse is explicit about who loses access.',
    },
    async () => {
      const row = platformAdminPage.locator('tr[data-group-id="business-users"]');
      await row.locator('[data-action="marketplace-assign"]').click();
      await platformAdminPage.waitForLoadState('networkidle');
    },
  );

  await step(
    platformAdminPage,
    'Taking the grant back',
    {
      what: 'The same row after unassigning, with the confirmation answered.',
      does: 'Removes the entitlement, leaving the marketplace exactly as the chapter found it.',
      design: 'Destructive actions are confirmed and say what is lost, not merely that the action is permanent.',
    },
    async () => {
      await platformAdminPage
        .locator('tr[data-group-id="business-users"] [data-action="marketplace-unassign"]')
        .click();
      await page.confirm();
      await platformAdminPage.waitForLoadState('networkidle');
    },
  );

  await step(
    platformAdminPage,
    'The plugin catalog',
    {
      what: 'Every plugin, what it carries, whether it is public, and how many rules name it.',
      does: 'Answers "what is installable here" without opening eight YAML files.',
      design: 'A dense table rather than cards: the reader is comparing entries, not browsing them.',
    },
    async () => {
      await platformAdminPage.goto(PATHS.plugins);
    },
  );

  await step(
    platformAdminPage,
    'The skills people can run',
    {
      what: 'The skill catalog, with the plugins that carry each skill and its visibility.',
      does: 'Separates the definition of a skill from the question of who is running it, which is one click away.',
      design: 'Skills with no plugin behind them are counted on a tile, because a skill nothing ships is invisible in use.',
    },
    async () => {
      await platformAdminPage.goto(PATHS.skills);
    },
  );

  const mcp = new McpListPage(platformAdminPage);
  await step(
    platformAdminPage,
    'Which tool servers are alive',
    {
      what: 'Every MCP server: whether a client is attached, how much it served, and how often it failed.',
      does: 'Turns "is the tool layer healthy" into one screen instead of a log search.',
      design: 'Liveness is derived from the last heartbeat rather than stored, so a server that stopped answering shows as stale on its own.',
    },
    async () => {
      await mcp.goto();
    },
  );

  await step(
    platformAdminPage,
    'A server nothing declares',
    {
      what: 'The list sorted so the undeclared server, serving traffic under a name no YAML file names, is visible.',
      does: 'Surfaces a binary answering under an unexpected name, which every other screen is silent about.',
      design: 'Rows are never dropped for being unrecognised; the row appears and the status column says why it is odd.',
    },
    async () => {
      await platformAdminPage.goto(`${PATHS.mcp}?sort=status&dir=asc`);
    },
  );

  const detail = new McpDetailPage(platformAdminPage, SERVER);
  await step(
    platformAdminPage,
    'One server, end to end',
    {
      what: 'The tools a server has served, its recent calls, the sessions on it, and the rules that let people reach it.',
      does: 'Takes an operator from "this server looks wrong" to the specific call and the specific user in one page.',
      design: 'Ordered as the questions are asked: what it serves, what it did, who is on it, who may reach it, and only then the file to edit.',
    },
    async () => {
      await detail.goto();
    },
  );

  const gateway = new GatewayPage(platformAdminPage);
  await step(
    platformAdminPage,
    'Where model traffic goes',
    {
      what: 'The gateway routing table in dispatch order, with the provider and rewrite each route applies.',
      does: 'Shows which provider will serve a given model request, and lets the order be changed without editing YAML by hand.',
      design: 'The ordinal is a column because order is the policy here: the first pattern that matches wins.',
    },
    async () => {
      await gateway.goto();
    },
  );

  await step(
    platformAdminPage,
    'Adding a route, and being refused',
    {
      what: 'The route form rejecting a route with no model pattern, with the reason from the API shown inline.',
      does: 'Keeps an invalid route out of the file rather than writing it and failing at dispatch time.',
      design: 'The error is the API\'s own words next to the field, not a generic toast that leaves the operator guessing.',
    },
    async () => {
      await gateway.openNewRoute();
      await gateway.submitRoute({ model_pattern: '', provider: 'anthropic' });
    },
  );

  await step(
    platformAdminPage,
    'What one person is offered',
    {
      what: 'The routes the gateway would advertise to a chosen account, resolved through the live rules.',
      does: 'Answers "why can this person not reach that model" without reproducing their session.',
      design: 'The probe reads the same resolver the dispatcher uses, so the page cannot disagree with what will actually happen.',
    },
    async () => {
      await platformAdminPage.locator('[data-action="route-cancel"]').click();
      await gateway.probe('e2e-admin');
    },
  );
});
