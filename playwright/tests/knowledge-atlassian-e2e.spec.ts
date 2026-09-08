// The knowledge bank end to end, over the CONNECTED Atlassian account.
//
// Nothing Atlassian is configured on the client: the only credential a user
// holds is their systemprompt PAT, exchanged for a bridge JWT, and the
// Atlassian account they linked once on /admin/profile#connected-accounts.
// This spec drives the same two MCP routes a Claude Code session would —
// `/api/v1/mcp/atlassian` and `/api/v1/mcp/knowledge-bank` — and proves the
// flow the way a user experiences it: the Atlassian MCP answers as that
// account; a labelled demo workspace is seeded through it (never with an
// Atlassian token of ours); `sync_knowledge` indexes it; `search_knowledge`
// finds it with citations into the site; the grants are the syncing user's
// alone; a user with no connected account degrades to the files categories
// and is told why; and every tool call left an audit row.
//
// Needs a live `just start` and KB_E2E_PAT — a PAT for a user with Atlassian
// connected — and skips with that reason otherwise. Write-tool steps need the
// connector's write scopes on the grant.
import { test, expect, dbRows } from './support/fixtures';
import { seededPat } from './support/manifest';
import { McpClient, bridgeJwt, mcpUrl } from '../setup/seed/mcp-client';
import { docCountsOf } from '../setup/seed/knowledge';
import { DEMO_LABEL, SPACE_KEY, seedAtlassianDemo, type DemoWorkspace } from '../setup/seed/atlassian-demo';

const PAT = process.env.KB_E2E_PAT;
const SYNC_BOX_MS = 5 * 60_000;
const POLL_MS = 5_000;
const SITE = process.env.KB_DEMO_SITE ?? 'https://astounddigital.atlassian.net';

interface GrantRow {
  doc_id: string;
  subject_kind: string;
  subject_value: string;
}

interface AuditRow {
  server_name: string;
  tool_name: string;
  status: string;
  n: string;
}

async function kbClient(baseURL: string, jwt: string): Promise<McpClient> {
  const client = new McpClient(mcpUrl('knowledge-bank', baseURL), jwt, 'astound-kb-e2e');
  await client.initialize();
  return client;
}

async function pollSync(client: McpClient, ids: string[], atLeast: number): Promise<Record<string, number>> {
  const deadline = Date.now() + SYNC_BOX_MS;
  let counts = docCountsOf(await client.callTool('list_knowledge_sources', {}), ids);
  while (ids.some((id) => counts[id] < atLeast) && Date.now() < deadline) {
    await new Promise((r) => setTimeout(r, POLL_MS));
    counts = docCountsOf(await client.callTool('list_knowledge_sources', {}), ids);
  }
  return counts;
}

test.describe('@REQ-047 @REQ-052 the knowledge bank reads Atlassian through the connected account', () => {
  // Why: the steps are one story — the bearer, the seeded workspace and the
  // nonce flow from one test to the next — so they run in one worker, in
  // order, and a failed step fails the rest rather than leaving them blind.
  test.describe.configure({ mode: 'serial' });
  test.skip(!PAT, 'KB_E2E_PAT is not set: needs a PAT for a user whose Atlassian account is connected');
  test.setTimeout(SYNC_BOX_MS + 10 * 60_000);

  let jwt = '';
  let userId = '';
  let workspace: DemoWorkspace;
  const started = new Date();

  test('the Atlassian MCP answers as the connected account, with no Atlassian credential on the client', async ({ baseURL }) => {
    jwt = await bridgeJwt(PAT!, baseURL!);
    const claims = JSON.parse(Buffer.from(jwt.split('.')[1], 'base64url').toString('utf8'));
    userId = String(claims.sub ?? claims.user_id);
    expect(userId).toBeTruthy();

    const atlassian = new McpClient(mcpUrl('atlassian', baseURL!), jwt, 'astound-kb-e2e');
    await atlassian.initialize();
    const resources = await atlassian.callData('getAccessibleAtlassianResources', {});
    expect(resources.resources?.length, 'the connected account reaches a site').toBeGreaterThan(0);
    const spaces = await atlassian.callData('listConfluenceSpaces', { cloudId: resources.resources[0].cloudId, keys: [SPACE_KEY] });
    expect(spaces.results?.[0]?.key).toBe(SPACE_KEY);
  });

  test('a labelled demo workspace is seeded through the Atlassian MCP', async ({ baseURL }) => {
    workspace = await seedAtlassianDemo(jwt, baseURL!);
    expect(workspace.pageIds.length).toBeGreaterThanOrEqual(6);
    expect(workspace.issueKeys.length).toBeGreaterThanOrEqual(8);
    console.log(`demo nonce ${workspace.nonce}; parent page ${workspace.parentPageId}`);
  });

  test('sync_knowledge indexes the workspace as the caller', async ({ baseURL }) => {
    const kb = await kbClient(baseURL!, jwt);
    const startedAt = Date.now();
    const reply = await kb.callTool('sync_knowledge', { source_types: ['confluence', 'jira'] });
    expect(reply).toContain('through your connected Atlassian account');
    const counts = await pollSync(kb, ['confluence', 'jira'], 1);
    console.log(`sync: confluence ${counts.confluence} docs, jira ${counts.jira} docs, ${Math.round((Date.now() - startedAt) / 1000)}s`);
    expect(counts.confluence, 'the demo pages were indexed').toBeGreaterThanOrEqual(workspace.pageIds.length);
    expect(counts.jira, 'the demo issues were indexed').toBeGreaterThanOrEqual(workspace.issueKeys.length);
  });

  test('search_knowledge finds the demo content with citations into the site', async ({ baseURL }) => {
    const kb = await kbClient(baseURL!, jwt);
    const pages = await kb.callTool('search_knowledge', {
      query: 'why do cart totals round half-to-even for Larkspur',
      source_types: ['confluence'],
      top_k: 5,
    });
    expect(pages).toContain('ADR-001');
    expect(pages).toContain(`${SITE}/wiki/spaces/${SPACE_KEY}/pages/`);

    const issues = await kb.callTool('search_knowledge', {
      query: 'Nightjar purge misses UTM-tagged listing URLs',
      source_types: ['jira'],
      top_k: 5,
    });
    expect(issues).toContain('Nightjar');
    expect(issues).toContain(`${SITE}/browse/`);
  });

  test('the demo documents are granted to the syncing user alone', async () => {
    const docIds = [
      ...workspace.pageIds.map((id) => `confluence:${id}`),
      ...workspace.issueKeys.map((key) => `jira:${key}`),
    ];
    const rows = await dbRows<GrantRow>(
      'SELECT doc_id, subject_kind, subject_value FROM knowledge_document_grants WHERE doc_id = ANY($1)',
      [docIds],
    );
    expect(rows.length, 'every demo document carries a grant').toBeGreaterThanOrEqual(docIds.length);
    for (const row of rows) {
      expect(row.subject_kind).toBe('user');
      expect(row.subject_value).toBe(userId);
    }
  });

  test('a user with no connected account degrades to the files categories and is told why', async ({ baseURL }) => {
    const other = seededPat('knowledgeWorker');
    const otherJwt = await bridgeJwt(other.secret, baseURL!);
    const kb = await kbClient(baseURL!, otherJwt);

    // Why: the scoped call is refused before any run starts, so it goes
    // first; the unscoped one spawns a files-only run that would otherwise
    // answer "already in progress" to whatever follows it.
    await expect(kb.callTool('sync_knowledge', { source_types: ['confluence'] })).rejects.toThrow(/connected-accounts/);

    const unscoped = await kb.callTool('sync_knowledge', {});
    expect(unscoped).toContain('Atlassian was skipped');
    expect(unscoped).toContain('connected-accounts');

    const listing = await kb.callTool('list_knowledge_sources', {});
    expect(listing).toMatch(/confluence — [^\n]*last sync failed/);

    const hits = await kb.callTool('search_knowledge', {
      query: 'why do cart totals round half-to-even for Larkspur',
      source_types: ['confluence', 'jira'],
    });
    expect(hits, 'REQ-052: no grant, no hit').not.toContain(workspace.nonce);
    expect(hits).not.toContain('ADR-001');
  });

  test('every knowledge-bank call left an audit row for the caller', async () => {
    const rows = await dbRows<AuditRow>(
      `SELECT server_name, tool_name, status, COUNT(*)::text AS n FROM mcp_tool_executions
       WHERE user_id = $1 AND started_at >= $2 GROUP BY 1, 2, 3 ORDER BY 1, 2, 3`,
      [userId, started],
    );
    const tools = new Set(rows.filter((r) => r.server_name === 'knowledge-bank').map((r) => r.tool_name));
    for (const tool of ['sync_knowledge', 'list_knowledge_sources', 'search_knowledge']) {
      expect(tools, `${tool} was audited`).toContain(tool);
    }
    console.log(rows.map((r) => `${r.server_name}/${r.tool_name} ${r.status} x${r.n}`).join('\n'));
  });

  test('the demo stays labelled for reuse', async () => {
    expect(workspace.issueKeys.length).toBeGreaterThan(0);
    console.log(`Confluence: ${SITE}/wiki/spaces/${SPACE_KEY}/pages/${workspace.parentPageId} — Jira: ${SITE}/issues/?jql=labels%3D${DEMO_LABEL}`);
  });
});
