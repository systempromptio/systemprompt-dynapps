// One skill-shaped tool round-trip through the gateway, audited.
//
// A skill that "searches the knowledge bank" is, on the wire, a /v1/messages
// call carrying an `mcp__knowledge-bank__search_knowledge` tool that the model
// answers with a tool_use. This spec sends exactly that as the europe
// developer's PAT, against the mock upstream (setup/mock-inference.ts answers
// the `[e2e:kb]` marker with the tool_use), and then asserts the audit the
// gateway wrote: an ai_requests row for this user and model, and its
// ai_request_tool_calls row naming the tool. The gateway captures tool_use
// blocks from every response it relays (core: gateway/audit/complete.rs,
// persist_tool_calls), so the mock route records them exactly as a real
// provider's would.
import { test, expect, dbRow, dbRows } from './support/fixtures';
import { seededPat } from './support/manifest';
import { E2E_SESSIONS } from '../setup/seed';
import { KB_MARKER, KB_QUERY } from '../setup/mock-inference';

const MODEL = 'e2e-mock-sonnet';
const TOOL = 'mcp__knowledge-bank__search_knowledge';
const REQUEST_ID_HEADER = 'x-systemprompt-request-id';

interface ToolUseBlock {
  type: string;
  name?: string;
  input?: { query?: string };
}

test.describe('@REQ-marketplaces a knowledge-bank skill call is audited as a tool call', () => {
  test('the gateway relays the tool_use and records it against the request', async ({
    baseURL,
  }) => {
    const pat = seededPat('europe');
    const res = await fetch(`${baseURL}/v1/messages`, {
      method: 'POST',
      headers: {
        authorization: `Bearer ${pat.secret}`,
        'x-session-id': E2E_SESSIONS[pat.userId],
        'anthropic-version': '2023-06-01',
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        model: MODEL,
        max_tokens: 256,
        tools: [
          {
            name: TOOL,
            description: 'Search the shared project knowledge bank.',
            input_schema: {
              type: 'object',
              properties: { query: { type: 'string' } },
              required: ['query'],
            },
          },
        ],
        messages: [
          {
            role: 'user',
            content: `${KB_MARKER} When does the Larkspur pilot ship? Use the knowledge bank.`,
          },
        ],
      }),
    });
    expect(res.status, await res.clone().text()).toBe(200);
    const requestId = res.headers.get(REQUEST_ID_HEADER);
    expect(requestId, `${REQUEST_ID_HEADER} header`).toBeTruthy();

    const body = (await res.json()) as { stop_reason: string; content: ToolUseBlock[] };
    expect(body.stop_reason).toBe('tool_use');
    const use = body.content.find((b) => b.type === 'tool_use');
    expect(use?.name).toBe(TOOL);
    expect(use?.input?.query).toBe(KB_QUERY);

    const request = await dbRow<{ id: string; user_id: string; model: string }>(
      `SELECT id, user_id, model FROM ai_requests WHERE id = $1`,
      [requestId],
    );
    expect(request?.user_id).toBe(pat.userId);
    expect(request?.model).toBe(MODEL);

    // The audit write is fire-and-forget behind the response, so poll briefly.
    let calls: { tool_name: string; tool_input: unknown }[] = [];
    for (let attempt = 0; attempt < 20 && calls.length === 0; attempt += 1) {
      calls = await dbRows(
        `SELECT tc.tool_name, tc.tool_input
           FROM ai_request_tool_calls tc
           JOIN ai_requests r ON r.id = tc.request_id
          WHERE r.id = $1 AND r.user_id = $2 AND r.model = $3`,
        [requestId, pat.userId, MODEL],
      );
      if (calls.length === 0) await new Promise((r) => setTimeout(r, 250));
    }
    expect(calls.map((c) => c.tool_name)).toEqual([TOOL]);
    expect(JSON.stringify(calls[0].tool_input)).toContain(KB_QUERY);
  });

  test('the second turn, carrying the tool result, is answered from it', async ({ baseURL }) => {
    const pat = seededPat('europe');
    const sentinel = 'The Larkspur pilot ships on the third Tuesday';
    const res = await fetch(`${baseURL}/v1/messages`, {
      method: 'POST',
      headers: {
        authorization: `Bearer ${pat.secret}`,
        'x-session-id': E2E_SESSIONS[pat.userId],
        'anthropic-version': '2023-06-01',
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        model: MODEL,
        max_tokens: 256,
        messages: [
          { role: 'user', content: `${KB_MARKER} When does the Larkspur pilot ship?` },
          {
            role: 'assistant',
            content: [{ type: 'tool_use', id: 'toolu_e2e_1', name: TOOL, input: { query: KB_QUERY } }],
          },
          {
            role: 'user',
            content: [{ type: 'tool_result', tool_use_id: 'toolu_e2e_1', content: sentinel }],
          },
        ],
      }),
    });
    expect(res.status, await res.clone().text()).toBe(200);
    const body = (await res.json()) as { stop_reason: string; content: { type: string; text?: string }[] };
    expect(body.stop_reason).toBe('end_turn');
    expect(body.content[0]?.text).toContain(sentinel.slice(0, 40));
  });
});
