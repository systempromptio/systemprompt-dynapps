// A minimal MCP streamable-HTTP client for the seeds and the e2e specs.
//
// Speaks JSON-RPC to `${GATEWAY}/api/v1/mcp/<server>/mcp` exactly as a bridge
// would: the caller's bridge JWT as bearer, `mcp-session-id` captured from the
// first response and echoed on every later one, SSE-framed responses unwrapped.
// Two servers matter here — `knowledge-bank` (this repo's RAG engine) and
// `atlassian` (the hosted Atlassian MCP, reached through the gateway as the
// signed-in user's connected account). Nothing Atlassian is ever sent from
// here; the gateway swaps the bearer for the banked token upstream.

export const GATEWAY_URL = process.env.GATEWAY_URL ?? 'http://localhost:8080';

export function mcpUrl(server: string, baseUrl: string = GATEWAY_URL): string {
  return `${baseUrl.replace(/\/$/, '')}/api/v1/mcp/${server}/mcp`;
}

/** Exchange a PAT for the bridge JWT the MCP routes accept. */
export async function bridgeJwt(patSecret: string, baseUrl: string = GATEWAY_URL): Promise<string> {
  const res = await fetch(`${baseUrl.replace(/\/$/, '')}/v1/auth/bridge/pat`, {
    method: 'POST',
    headers: { authorization: `Bearer ${patSecret}`, 'content-type': 'application/json' },
    body: '{}',
  });
  if (!res.ok) throw new Error(`/v1/auth/bridge/pat -> HTTP ${res.status}: ${(await res.text()).slice(0, 200)}`);
  const { token } = (await res.json()) as { token: string };
  return token;
}

export interface ToolResult {
  isError?: boolean;
  content?: { type: string; text?: string }[];
  structuredContent?: unknown;
}

export interface ToolSpec {
  name: string;
  inputSchema?: { properties?: Record<string, unknown>; required?: string[] };
}

export class McpClient {
  private session: string | undefined;
  private nextId = 1;

  constructor(
    private readonly url: string,
    private readonly token: string,
    private readonly clientName = 'astound-e2e',
  ) {}

  private async post(body: unknown): Promise<Response> {
    const headers: Record<string, string> = {
      authorization: `Bearer ${this.token}`,
      'content-type': 'application/json',
      accept: 'application/json, text/event-stream',
    };
    if (this.session) headers['mcp-session-id'] = this.session;
    const res = await fetch(this.url, { method: 'POST', headers, body: JSON.stringify(body) });
    const sid = res.headers.get('mcp-session-id');
    if (sid) this.session = sid;
    return res;
  }

  private async payload(res: Response): Promise<any> {
    const text = await res.text();
    if (!res.ok) throw new Error(`${this.url} -> HTTP ${res.status}: ${text.slice(0, 300)}`);
    const data = res.headers.get('content-type')?.includes('text/event-stream')
      ? text
          .split('\n')
          .filter((l) => l.startsWith('data: '))
          .map((l) => l.slice(6))
          .pop()
      : text;
    if (!data) throw new Error(`empty MCP response from ${this.url}`);
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
        clientInfo: { name: this.clientName, version: '1' },
      },
    });
    await this.payload(res);
    if (!this.session) throw new Error('the MCP server returned no mcp-session-id');
    await this.post({ jsonrpc: '2.0', method: 'notifications/initialized' });
  }

  /** Every tool the server lists, following pagination. */
  async listTools(): Promise<ToolSpec[]> {
    const tools: ToolSpec[] = [];
    let cursor: string | undefined;
    for (;;) {
      const res = await this.post({
        jsonrpc: '2.0',
        id: this.nextId++,
        method: 'tools/list',
        params: cursor ? { cursor } : {},
      });
      const result = await this.payload(res);
      tools.push(...(result.tools ?? []));
      cursor = result.nextCursor;
      if (!cursor) return tools;
    }
  }

  async callRaw(name: string, args: Record<string, unknown>): Promise<ToolResult> {
    const res = await this.post({
      jsonrpc: '2.0',
      id: this.nextId++,
      method: 'tools/call',
      params: { name, arguments: args },
    });
    return (await this.payload(res)) as ToolResult;
  }

  /** The joined text content of a successful call. */
  async callTool(name: string, args: Record<string, unknown>): Promise<string> {
    const result = await this.callRaw(name, args);
    if (result.isError) throw new Error(`${name} failed: ${JSON.stringify(result.content)}`);
    return (result.content ?? [])
      .filter((c) => c.type === 'text')
      .map((c) => c.text ?? '')
      .join('\n');
  }

  /** The `data` payload the hosted Atlassian MCP wraps every answer in. */
  async callData(name: string, args: Record<string, unknown>): Promise<any> {
    const result = await this.callRaw(name, args);
    if (result.isError) throw new Error(`${name} failed: ${JSON.stringify(result.content)}`);
    if (result.structuredContent !== undefined) return unwrap(result.structuredContent);
    for (const block of result.content ?? []) {
      if (block.type !== 'text' || !block.text) continue;
      try {
        return unwrap(JSON.parse(block.text));
      } catch {
        // not JSON; keep looking
      }
    }
    throw new Error(`${name} returned no JSON payload`);
  }
}

function unwrap(value: any): any {
  return value && typeof value === 'object' && 'data' in value ? value.data : value;
}
