// Deterministic mock inference upstream for the e2e suite.
//
// Speaks the Anthropic wire on POST /v1/messages and echoes the last user
// message back, so a full gateway round-trip (auth -> governance -> dispatch
// -> audit -> cost) runs against real traffic without a live provider. One
// scripted exception: a request whose tools[] carries a `*search_knowledge`
// tool and whose last user text contains `[e2e:kb]` is answered with a
// tool_use for it, and a request whose last message carries a tool_result is
// answered by echoing that result — enough for a spec to drive one real
// skill-shaped tool round-trip through the gateway's tool-call audit. The
// gateway reaches it via the `e2e-mock` provider in the local profile
// (endpoint http://127.0.0.1:18091/v1, surface backend, model
// e2e-mock-sonnet).
//
// Run standalone (`npx tsx setup/mock-inference.ts`) or let global-setup
// spawn it; either way it is idempotent — if the port is already held by a
// live mock (GET /health answers), the second start exits quietly.
import { createServer } from 'node:http';

export const MOCK_PORT = 18091;

type WireMessage = { role: string; content: unknown };
type WireTool = { name: string };
type ContentBlock = { type: string; text?: string; content?: unknown; [k: string]: unknown };

export const KB_MARKER = '[e2e:kb]';
export const KB_QUERY = 'Larkspur pilot';
const KB_TOOL_SUFFIX = 'search_knowledge';
const ECHO_CHARS = 80;

function blocksOf(message: WireMessage | undefined): ContentBlock[] {
  if (!message || !Array.isArray(message.content)) return [];
  return message.content as ContentBlock[];
}

function toolResultText(block: ContentBlock): string {
  if (typeof block.content === 'string') return block.content;
  if (Array.isArray(block.content)) {
    return (block.content as ContentBlock[])
      .filter((b) => b.type === 'text' && typeof b.text === 'string')
      .map((b) => b.text)
      .join('\n');
  }
  return JSON.stringify(block.content ?? '');
}

type Scripted = { content: ContentBlock[]; stop_reason: 'tool_use' | 'end_turn' };

// The knowledge-bank script, or null when the request is ordinary traffic.
function scriptedReply(parsed: { messages?: WireMessage[]; tools?: WireTool[] }): Scripted | null {
  const messages = parsed.messages ?? [];
  const last = messages[messages.length - 1];
  const result = blocksOf(last).find((b) => b.type === 'tool_result');
  if (result) {
    const echoed = toolResultText(result).slice(0, ECHO_CHARS);
    return { content: [{ type: 'text', text: `Mock tool echo: ${echoed}` }], stop_reason: 'end_turn' };
  }
  const tool = (parsed.tools ?? []).find((t) => t.name.endsWith(KB_TOOL_SUFFIX));
  if (tool && lastUserText(messages).includes(KB_MARKER)) {
    return {
      content: [
        {
          type: 'tool_use',
          id: `toolu_mock_${Date.now()}`,
          name: tool.name,
          input: { query: KB_QUERY },
        },
      ],
      stop_reason: 'tool_use',
    };
  }
  return null;
}

function lastUserText(messages: WireMessage[]): string {
  for (let i = messages.length - 1; i >= 0; i -= 1) {
    const m = messages[i];
    if (m.role === 'user') {
      if (typeof m.content === 'string') return m.content;
      if (Array.isArray(m.content)) {
        const texts = m.content
          .filter((b): b is { type: string; text: string } => b?.type === 'text')
          .map((b) => b.text);
        if (texts.length > 0) return texts.join('\n');
      }
    }
  }
  return '';
}

export function startMockInference(): Promise<void> {
  const server = createServer((req, res) => {
    if (req.method === 'GET' && req.url === '/health') {
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end('{"mock":"inference"}');
      return;
    }
    if (req.method !== 'POST' || !req.url?.endsWith('/messages')) {
      res.writeHead(404, { 'content-type': 'application/json' });
      res.end('{"error":"mock-inference: unknown route"}');
      return;
    }
    let body = '';
    req.on('data', (c) => {
      body += c;
    });
    req.on('end', () => {
      let parsed: { model?: string; messages?: WireMessage[]; tools?: WireTool[] };
      try {
        parsed = JSON.parse(body);
      } catch {
        res.writeHead(400, { 'content-type': 'application/json' });
        res.end('{"error":"invalid json"}');
        return;
      }
      const prompt = lastUserText(parsed.messages ?? []);
      const text = `Mock reply: ${prompt}`.slice(0, 4000);
      const reply: Scripted = scriptedReply(parsed) ?? {
        content: [{ type: 'text', text }],
        stop_reason: 'end_turn',
      };
      const inputTokens = Math.max(1, Math.ceil(body.length / 4));
      const outputTokens = Math.max(1, Math.ceil(JSON.stringify(reply.content).length / 4));
      res.writeHead(200, { 'content-type': 'application/json' });
      res.end(
        JSON.stringify({
          id: `msg_mock_${Date.now()}`,
          type: 'message',
          role: 'assistant',
          model: parsed.model ?? 'e2e-mock-sonnet',
          content: reply.content,
          stop_reason: reply.stop_reason,
          stop_sequence: null,
          usage: { input_tokens: inputTokens, output_tokens: outputTokens },
        }),
      );
    });
  });

  return new Promise((resolve, reject) => {
    server.once('error', (err: NodeJS.ErrnoException) => {
      if (err.code === 'EADDRINUSE') {
        resolve();
      } else {
        reject(err);
      }
    });
    server.listen(MOCK_PORT, '127.0.0.1', () => {
      console.log(`mock inference listening on 127.0.0.1:${MOCK_PORT}`);
      resolve();
    });
  });
}

if (require.main === module) {
  startMockInference().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
