// Seed the knowledge bank: sync the fixture categories shipped under
// services/knowledge-bank/fixtures/ into the running instance's pgvector index
// by driving the MCP server exactly as a client would — streamable-HTTP JSON-RPC
// at ${BASE_URL}/api/v1/mcp/knowledge-bank/mcp with a bridge token exchanged from the seeded e2e
// admin — then poll list_knowledge_sources until project_docs reports
// documents. Not part of the default e2e seed: `just knowledge-seed` runs it,
// and it needs `just start` (with the local model provider nothing else).
import { join } from 'node:path';
import { readFileSync } from 'node:fs';

const BASE_URL = process.env.GATEWAY_URL ?? 'http://localhost:8080';
const MCP_URL = `${BASE_URL}/api/v1/mcp/knowledge-bank/mcp`;
const TIME_BOX_MS = 120_000;
const POLL_MS = 3_000;
const WATCH = ['project_docs', 'project_tickets', 'client_accounts'];

async function adminToken(): Promise<string> {
  // Why: the gateway's MCP route accepts the bridge JWT a client obtains by
  // exchanging its PAT at /v1/auth/bridge/pat, not an admin-plane web token.
  // Using the seeded admin PAT here exercises the same path a bridge takes.
  const pats = JSON.parse(readFileSync(join(__dirname, '..', '..', '.auth', 'pats.json'), 'utf8')) as Record<
    string,
    { secret: string }
  >;
  const secret = pats.admin?.secret;
  if (!secret) throw new Error('playwright/.auth/pats.json has no admin PAT — run `just e2e-seed` first');
  const res = await fetch(`${BASE_URL}/v1/auth/bridge/pat`, {
    method: 'POST',
    headers: { authorization: `Bearer ${secret}`, 'content-type': 'application/json' },
    body: '{}',
  });
  if (!res.ok) throw new Error(`/v1/auth/bridge/pat -> HTTP ${res.status}: ${(await res.text()).slice(0, 200)}`);
  const { token } = (await res.json()) as { token: string };
  return token;
}

class McpClient {
  private session: string | undefined;
  private nextId = 1;

  constructor(private readonly token: string) {}

  private async post(body: unknown): Promise<Response> {
    const headers: Record<string, string> = {
      authorization: `Bearer ${this.token}`,
      'content-type': 'application/json',
      accept: 'application/json, text/event-stream',
    };
    if (this.session) headers['mcp-session-id'] = this.session;
    const res = await fetch(MCP_URL, { method: 'POST', headers, body: JSON.stringify(body) });
    const sid = res.headers.get('mcp-session-id');
    if (sid) this.session = sid;
    return res;
  }

  private static async payload(res: Response): Promise<any> {
    const text = await res.text();
    if (!res.ok) throw new Error(`${MCP_URL} -> HTTP ${res.status}: ${text.slice(0, 300)}`);
    const data = res.headers.get('content-type')?.includes('text/event-stream')
      ? text
          .split('\n')
          .filter((l) => l.startsWith('data: '))
          .map((l) => l.slice(6))
          .pop()
      : text;
    if (!data) throw new Error(`empty MCP response from ${MCP_URL}`);
    const parsed = JSON.parse(data);
    if (parsed.error) throw new Error(`MCP error: ${JSON.stringify(parsed.error)}`);
    return parsed.result;
  }

  async initialize(): Promise<void> {
    const res = await this.post({
      jsonrpc: '2.0',
      id: this.nextId++,
      method: 'initialize',
      params: {
        protocolVersion: '2025-06-18',
        capabilities: {},
        clientInfo: { name: 'astound-knowledge-seed', version: '1' },
      },
    });
    await McpClient.payload(res);
    if (!this.session) throw new Error('the MCP server returned no mcp-session-id');
    await this.post({ jsonrpc: '2.0', method: 'notifications/initialized' });
  }

  async callTool(name: string, args: Record<string, unknown>): Promise<string> {
    const res = await this.post({
      jsonrpc: '2.0',
      id: this.nextId++,
      method: 'tools/call',
      params: { name, arguments: args },
    });
    const result = await McpClient.payload(res);
    if (result.isError) throw new Error(`${name} failed: ${JSON.stringify(result.content)}`);
    return (result.content ?? [])
      .filter((c: { type: string }) => c.type === 'text')
      .map((c: { text: string }) => c.text)
      .join('\n');
  }
}

// `list_knowledge_sources` renders one block per category:
//   `<id> — <name> (available, <n> docs readable by you, ...)`
function docCounts(listing: string): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const id of WATCH) {
    const m = listing.match(new RegExp(`^${id} — [^\\n]*?(\\d+) docs readable`, 'm'));
    counts[id] = m ? Number(m[1]) : 0;
  }
  return counts;
}

export async function seedKnowledge(): Promise<Record<string, number>> {
  const client = new McpClient(await adminToken());
  await client.initialize();
  const started = await client.callTool('sync_knowledge', {});
  console.log(`sync_knowledge: ${started.split('\n')[0]}`);

  const deadline = Date.now() + TIME_BOX_MS;
  let counts = docCounts(await client.callTool('list_knowledge_sources', {}));
  while (counts.project_docs === 0 && Date.now() < deadline) {
    await new Promise((r) => setTimeout(r, POLL_MS));
    counts = docCounts(await client.callTool('list_knowledge_sources', {}));
  }
  for (const id of WATCH) console.log(`${id}: ${counts[id]} docs`);
  if (counts.project_docs === 0) {
    throw new Error(
      `project_docs has no documents after ${TIME_BOX_MS / 1000}s; check the knowledge-bank ` +
        `child's log (systemprompt plugins mcp logs knowledge-bank)`,
    );
  }
  return counts;
}

if (require.main === module) {
  seedKnowledge().catch((e) => {
    console.error(e instanceof Error ? e.message : e);
    process.exit(1);
  });
}
