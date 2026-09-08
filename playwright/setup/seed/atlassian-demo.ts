// Seed a clearly labelled demo workspace in Atlassian THROUGH the hosted
// Atlassian MCP, as the signed-in user's connected account — never with an
// Atlassian credential of our own. Everything lands inside the EAICOMM space
// and project the knowledge bank already indexes, so no config edit and no
// restart: a parent page `[KB DEMO] …` with six child pages, and eight
// `[KB DEMO]` issues labelled `kb-demo`. Each run carries a nonce
// (`KBDEMO-yyyymmdd-hhmm`) in the parent title and in every body so a search
// assertion is unambiguous; a rerun finds the parent by title and reuses it.
//
// The hosted server exposes no delete tools, so the workspace stays; it is
// labelled for exactly that reason. Requires the connector's write scopes
// (write:confluence:agent-interface, write:jira:agent-interface) on the
// caller's grant — a read-only grant lists no create tools and this seed says
// so rather than inventing content elsewhere.
//
// Usage: KB_E2E_PAT=<pat of a user with Atlassian connected> npx tsx setup/seed/atlassian-demo.ts
import { GATEWAY_URL, McpClient, type ToolSpec, bridgeJwt, mcpUrl } from './mcp-client';

export const DEMO_TITLE = '[KB DEMO] Knowledge Bank Demo Workspace';
export const DEMO_LABEL = 'kb-demo';
export const SPACE_KEY = process.env.KB_DEMO_SPACE ?? 'EAICOMM';
export const PROJECT_KEY = process.env.KB_DEMO_PROJECT ?? 'EAICOMM';

export interface DemoWorkspace {
  nonce: string;
  cloudId: string;
  spaceId: string;
  parentPageId: string;
  pageIds: string[];
  issueKeys: string[];
  created: boolean;
}

// Why: the demo's facts are invented for one fictional engagement so a search
// hit can be traced to exactly one page or issue, and so the BA skills have
// realistic material (brief, ADR, runbook, notes, glossary, release notes).
function pages(nonce: string): { title: string; body: string }[] {
  const stamp = `\n\n---\n_Demo content, nonce ${nonce}. Safe to ignore; created by the knowledge-bank e2e seed._\n`;
  return [
    {
      title: '[KB DEMO] Project brief — Larkspur Outdoors storefront',
      body: `# Larkspur Outdoors storefront\n\n**Client:** Larkspur Outdoors (fictional). **Platform:** Salesforce Commerce Cloud, Storefront Next.\n\n## Goal\n\nReplace the legacy SiteGenesis checkout with a Storefront Next checkout by the end of Q4, keeping the Larkspur loyalty programme ("Trailhead Points") working on day one.\n\n## Scope\n\n- Checkout, cart and order confirmation pages\n- Trailhead Points redemption at checkout\n- Payment via Adyen drop-in (cards, iDEAL, Apple Pay)\n\n## Out of scope\n\n- Returns portal (separate SOW)\n- Store locator\n\n## Stakeholders\n\nProduct owner: Mara Quist. Engineering lead: Tomas Ekwall. Astound BA: Priya Nandakumar.${stamp}`,
    },
    {
      title: '[KB DEMO] ADR-001 — Cart totals round half-to-even',
      body: `# ADR-001: Cart totals round half-to-even\n\n**Status:** accepted\n\n## Context\n\nLarkspur's ERP ("Osprey") rounds line totals half-to-even, while Storefront Next defaults to half-up. Reconciliation showed a one-cent drift on 3.2% of orders in the pilot week.\n\n## Decision\n\nCart, tax and shipping totals in the storefront round **half-to-even** (banker's rounding) to match Osprey. The rounding mode is set once in the pricing service, never per template.\n\n## Consequences\n\n- Reconciliation drift eliminated in the second pilot week.\n- Promotions that split a discount across lines must use the same helper.${stamp}`,
    },
    {
      title: '[KB DEMO] Runbook — Deploying the Nightjar cache layer',
      body: `# Deploying the Nightjar cache layer\n\nNightjar is the edge cache in front of the Larkspur product listing pages.\n\n## Steps\n\n1. Run \`just deploy-next\` from the release branch.\n2. Purge Nightjar with \`nightjar purge --site larkspur --scope plp\`.\n3. Watch the hit ratio dashboard for ten minutes; a ratio under 85% means the purge fanned out too wide.\n4. Roll back with \`just deploy\` on main if p95 latency stays above 400 ms.\n\n## Gotchas\n\nNightjar keys on the full query string, so a marketing UTM parameter is a cache miss. Strip UTM parameters at the edge rule before purging.${stamp}`,
    },
    {
      title: '[KB DEMO] Meeting notes — Checkout sync 2026-09-02',
      body: `# Checkout sync — 2026-09-02\n\n**Attendees:** Mara Quist, Tomas Ekwall, Priya Nandakumar\n\n## Decisions\n\n- Trailhead Points redemption ships behind a feature flag \`trailhead_redeem\` and is enabled for 10% of sessions first.\n- Apple Pay is deferred to the second release; iDEAL stays in the first.\n\n## Actions\n\n- Tomas: wire the feature flag by Friday.\n- Priya: update the acceptance criteria on the redemption story.${stamp}`,
    },
    {
      title: '[KB DEMO] Glossary — Larkspur engagement terms',
      body: `# Glossary\n\n- **Osprey** — Larkspur's ERP; source of truth for prices and stock.\n- **Nightjar** — the edge cache in front of product listing pages.\n- **Trailhead Points** — Larkspur's loyalty currency, redeemable at checkout.\n- **Kestrel** — the internal name of the Adyen payment adapter.\n- **Half-to-even** — the rounding mode every total uses (see ADR-001).${stamp}`,
    },
    {
      title: '[KB DEMO] Release notes — Storefront Next 1.4.0',
      body: `# Storefront Next 1.4.0 — release notes\n\n## Added\n\n- Trailhead Points balance shown on the cart page.\n- iDEAL via the Kestrel adapter.\n\n## Changed\n\n- Cart totals round half-to-even (ADR-001).\n\n## Fixed\n\n- Nightjar purge no longer misses UTM-tagged listing URLs.${stamp}`,
    },
  ];
}

function issues(nonce: string): { type: string; summary: string; description: string }[] {
  const stamp = `\n\n---\nDemo issue, nonce ${nonce}. Created by the knowledge-bank e2e seed.`;
  return [
    { type: 'Epic', summary: '[KB DEMO] Storefront Next checkout for Larkspur', description: `Replace the SiteGenesis checkout with Storefront Next, including Trailhead Points redemption and Kestrel payments.${stamp}` },
    { type: 'Story', summary: '[KB DEMO] Redeem Trailhead Points at checkout', description: `As a Larkspur shopper I can apply Trailhead Points to my order total so that loyalty balances are usable on day one. Behind the trailhead_redeem flag at 10% of sessions.${stamp}` },
    { type: 'Story', summary: '[KB DEMO] Pay with iDEAL through Kestrel', description: `Add iDEAL to the Kestrel (Adyen) adapter. Apple Pay is deferred to release two.${stamp}` },
    { type: 'Story', summary: '[KB DEMO] Show Trailhead Points balance on the cart page', description: `The cart page shows the shopper's current Trailhead Points balance from Osprey.${stamp}` },
    { type: 'Bug', summary: '[KB DEMO] One-cent drift between cart total and Osprey invoice', description: `3.2% of pilot orders differ by one cent from the Osprey invoice. Root cause: half-up rounding in the storefront. Fix per ADR-001: round half-to-even.${stamp}` },
    { type: 'Bug', summary: '[KB DEMO] Nightjar purge misses UTM-tagged listing URLs', description: `Nightjar keys on the full query string; listing pages reached with utm_source stay stale after a purge. Strip UTM at the edge rule.${stamp}` },
    { type: 'Task', summary: '[KB DEMO] Wire the trailhead_redeem feature flag', description: `Create the flag in the pricing service and default it to 10% of sessions.${stamp}` },
    { type: 'Spike', summary: '[KB DEMO] Measure Nightjar hit ratio after a scoped purge', description: `Confirm a plp-scoped purge keeps the hit ratio above 85% for ten minutes.${stamp}` },
  ];
}

export function newNonce(): string {
  const now = new Date();
  const p = (n: number) => String(n).padStart(2, '0');
  return `KBDEMO-${now.getUTCFullYear()}${p(now.getUTCMonth() + 1)}${p(now.getUTCDate())}-${p(now.getUTCHours())}${p(now.getUTCMinutes())}`;
}

function requireTool(tools: ToolSpec[], names: string[]): ToolSpec {
  const found = tools.find((t) => names.includes(t.name));
  if (!found) {
    throw new Error(
      `the Atlassian MCP lists none of ${names.join('/')} for this account — the connected grant is read-only. ` +
        'Reconnect Atlassian on /admin/profile#connected-accounts after the write scopes were added.',
    );
  }
  return found;
}

export async function seedAtlassianDemo(bearer: string, baseUrl: string = GATEWAY_URL): Promise<DemoWorkspace> {
  const client = new McpClient(mcpUrl('atlassian', baseUrl), bearer, 'astound-kb-demo-seed');
  await client.initialize();
  const tools = await client.listTools();
  const resources = await client.callData('getAccessibleAtlassianResources', {});
  const cloudId: string = resources.resources?.[0]?.cloudId ?? resources.resources?.[0]?.id;
  if (!cloudId) throw new Error('the connected Atlassian account reaches no site');

  const spaces = await client.callData('listConfluenceSpaces', { cloudId, keys: [SPACE_KEY] });
  const spaceId: string | undefined = spaces.results?.[0]?.id;
  if (!spaceId) throw new Error(`Confluence space ${SPACE_KEY} is not readable by this account`);

  // Reuse an existing workspace: the parent page carries the nonce in its title.
  // Why: the children share the `[KB DEMO]` prefix, so the search names the
  // parent's full title and the match is on that exact prefix.
  const existing = await client.callData('searchConfluence', {
    cloudId,
    cql: `space = "${SPACE_KEY}" AND type = page AND title ~ "KB DEMO"`,
    limit: 50,
  });
  const parent = (existing.results ?? []).find((r: any) => String(r.title ?? r.content?.title ?? '').startsWith(`${DEMO_TITLE} (`));
  if (parent) {
    const title: string = parent.title ?? parent.content?.title;
    const nonce = /KBDEMO-\d{8}-\d{4}/.exec(title)?.[0] ?? 'KBDEMO-unknown';
    const parentPageId: string = parent.id ?? parent.content?.id;
    const children = await client.callData('getConfluenceContentDescendants', { cloudId, contentId: parentPageId, depth: 1, limit: 50 });
    const pageIds = (children.results ?? []).map((c: any) => String(c.id));
    const found = await client.callData('searchJiraIssuesUsingJql', {
      cloudId,
      jql: `project = ${PROJECT_KEY} AND labels = ${DEMO_LABEL} ORDER BY created ASC`,
      maxResults: 50,
      fields: ['summary'],
    });
    const issueKeys = (found.issues ?? []).map((i: any) => String(i.key));
    console.log(`demo workspace exists (${nonce}); reusing ${pageIds.length} pages and ${issueKeys.length} issues`);
    return { nonce, cloudId, spaceId, parentPageId, pageIds, issueKeys, created: false };
  }

  const createPage = requireTool(tools, ['createConfluenceContent', 'createConfluencePage']);
  const createIssue = requireTool(tools, ['createJiraIssue']);
  const nonce = newNonce();

  // Pinned 2026-09-08 from the live catalogue: createConfluenceContent takes
  // {cloudId, contentType, title, parent: {spaceId, parentContentId?},
  // body: {format, value}}; createJiraIssue takes {cloudId, projectKey,
  // summary, issueType, description, labels}.
  const parentBody = `# Knowledge Bank Demo Workspace\n\nA fictional Astound Digital engagement (Larkspur Outdoors, Storefront Next) used to prove the knowledge bank end to end: these pages and the \`${DEMO_LABEL}\` Jira issues are read through the connected Atlassian account, indexed, and searched.\n\nNonce: ${nonce}\n\n_Safe to ignore. Created by the knowledge-bank e2e seed._\n`;
  const created = await client.callData(createPage.name, {
    cloudId,
    contentType: 'page',
    title: `${DEMO_TITLE} (${nonce})`,
    parent: { spaceId },
    body: { format: 'markdown', value: parentBody },
  });
  const parentPageId = String(created.id ?? created.page?.id ?? created.content?.id);
  if (!parentPageId || parentPageId === 'undefined') throw new Error(`${createPage.name} returned no id: ${JSON.stringify(created).slice(0, 300)}`);
  console.log(`created parent page ${parentPageId}`);

  const pageIds: string[] = [];
  for (const page of pages(nonce)) {
    const made = await client.callData(createPage.name, {
      cloudId,
      contentType: 'page',
      title: `${page.title} (${nonce})`,
      parent: { spaceId, parentContentId: parentPageId },
      body: { format: 'markdown', value: page.body },
    });
    const id = String(made.id ?? made.page?.id ?? made.content?.id);
    pageIds.push(id);
    console.log(`created page ${id}: ${page.title}`);
  }

  const issueKeys: string[] = [];
  for (const issue of issues(nonce)) {
    const made = await client.callData(createIssue.name, {
      cloudId,
      projectKey: PROJECT_KEY,
      issueType: issue.type,
      summary: issue.summary,
      description: issue.description,
      labels: [DEMO_LABEL],
    });
    const key = String(made.key ?? made.issue?.key ?? made.issueKey);
    if (!key || key === 'undefined') throw new Error(`${createIssue.name} returned no key: ${JSON.stringify(made).slice(0, 300)}`);
    issueKeys.push(key);
    console.log(`created issue ${key}: ${issue.summary}`);
  }
  return { nonce, cloudId, spaceId, parentPageId, pageIds, issueKeys, created: true };
}

if (require.main === module) {
  const pat = process.env.KB_E2E_PAT;
  if (!pat) {
    console.error('KB_E2E_PAT is not set: a PAT for a user whose Atlassian account is connected on /admin/profile');
    process.exit(2);
  }
  bridgeJwt(pat)
    .then((jwt) => seedAtlassianDemo(jwt))
    .then((ws) => console.log(JSON.stringify(ws, null, 2)))
    .catch((e) => {
      console.error(e instanceof Error ? e.message : e);
      process.exit(1);
    });
}
