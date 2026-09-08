// Seed the knowledge bank: sync the fixture categories shipped under
// services/knowledge-bank/fixtures/ into the running instance's pgvector index
// by driving the MCP server exactly as a client would — streamable-HTTP JSON-RPC
// at ${BASE_URL}/api/v1/mcp/knowledge-bank/mcp with a bridge token exchanged from the seeded e2e
// admin — then poll list_knowledge_sources until project_docs reports
// documents. Not part of the default e2e seed: `just knowledge-seed` runs it,
// and it needs `just start` (with the local model provider nothing else).
import { join } from 'node:path';
import { readFileSync } from 'node:fs';
import { McpClient, bridgeJwt, mcpUrl } from './mcp-client';

const BASE_URL = process.env.GATEWAY_URL ?? 'http://localhost:8080';
const MCP_URL = mcpUrl('knowledge-bank', BASE_URL);
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
  return bridgeJwt(secret, BASE_URL);
}

// `list_knowledge_sources` renders one block per category:
//   `<id> — <name> (available, <n> docs readable by you, ...)`
function docCounts(listing: string): Record<string, number> {
  return docCountsOf(listing, WATCH);
}

export function docCountsOf(listing: string, ids: string[]): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const id of ids) {
    const m = listing.match(new RegExp(`^${id} — [^\\n]*?(\\d+) docs readable`, 'm'));
    counts[id] = m ? Number(m[1]) : 0;
  }
  return counts;
}

export async function seedKnowledge(): Promise<Record<string, number>> {
  const client = new McpClient(MCP_URL, await adminToken(), 'astound-knowledge-seed');
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
