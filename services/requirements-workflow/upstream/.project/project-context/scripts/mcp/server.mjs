// MCP stdio server. Exposes three tools to Cursor: search, list_sources,
// index_stats. Run via: npm run mcp  (configured in .cursor/mcp.json).

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';

import { loadEnv } from '../core/env.mjs';
import { loadConfig, resolveHome } from '../core/config.mjs';
import { preflight } from '../core/preflight.mjs';
import { search, listSources, indexStats } from '../retrieval/service.mjs';
import { error } from '../core/log.mjs';

// Load config best-effort: it lets us derive the source_type filter from the
// project's own sources and honor a custom env_path. When no config exists yet
// (e.g. a fresh checkout or the smoke test), fall back to a free-form filter.
let config = null;
try {
  config = loadConfig();
} catch {
  config = null;
}

loadEnv(config?.envPath ?? resolveHome());

try {
  preflight();
} catch (err) {
  error(err.message);
  process.exit(1);
}

const sourceNames = config ? Object.keys(config.sources) : [];
const defaultTopK = config?.retrieval?.topK ?? 5;
// "Live" categories exclude the historical archive(s). Archive sources are only
// searched when the caller lists them explicitly (the archive opt-in), so a broad
// search means passing all live categories, never omitting the filter.
const liveNames = sourceNames.filter((n) => !/archive/i.test(n));
const archiveNames = sourceNames.filter((n) => /archive/i.test(n));
const sourceFilter =
  sourceNames.length > 0
    ? z
        .array(z.enum(sourceNames))
        .nonempty()
        .describe(
          'REQUIRED. One or more source categories to search, as an array. Pick the smallest set ' +
            'that could hold the answer; to search broadly, pass all live categories explicitly: ' +
            `[${liveNames.join(', ')}]. Include archive sources (${
              archiveNames.join(', ') || 'none'
            }) ONLY for historical rationale — never for current scope, status, or approvals. ` +
            `Allowed values: ${sourceNames.join(', ')}.`,
        )
    : z
        .array(z.string())
        .nonempty()
        .describe('REQUIRED. One or more source categories to search, as an array.');

const server = new McpServer({ name: 'project-context', version: '0.1.0' });

function asJson(value) {
  return { content: [{ type: 'text', text: JSON.stringify(value, null, 2) }] };
}

async function guard(fn) {
  try {
    return asJson(await fn());
  } catch (err) {
    error(err?.stack ?? String(err));
    return asJson({ error: String(err?.message ?? err) });
  }
}

server.registerTool(
  'search',
  {
    title: 'Search project context',
    description:
      'Find prior project knowledge needed to answer, plan, or explain work: decisions, ' +
      'requirements, Jira/Confluence context, meeting outcomes, ADRs, and specs. Returns ' +
      'excerpts with source file and line pointers for follow-up reading. You MUST scope the ' +
      'search with `source_type` (an array of categories); to search broadly, list all live ' +
      'categories explicitly rather than omitting the filter.',
    inputSchema: {
      query: z
        .string()
        .describe(
          'Natural-language query for the project context you need; prefer English and project terms.',
        ),
      source_type: sourceFilter,
      top_k: z
        .number()
        .int()
        .min(1)
        .max(20)
        .optional()
        .describe(
          `Number of excerpts to return (default ${defaultTopK}, max 20). Use more for synthesis.`,
        ),
    },
  },
  async ({ query, source_type, top_k }) =>
    guard(() => search(query, { sourceType: source_type, topK: top_k ?? null })),
);

server.registerTool(
  'list_sources',
  {
    title: 'List indexed sources',
    description:
      'Show which source types are indexed, with document counts and last sync time. ' +
      'Use before filtering or when search results look empty.',
    inputSchema: {},
  },
  async () => guard(() => listSources()),
);

server.registerTool(
  'index_stats',
  {
    title: 'Index statistics',
    description:
      'Check index health: total chunks, per-source document counts, and last sync time.',
    inputSchema: {},
  },
  async () => guard(() => indexStats()),
);

const transport = new StdioServerTransport();
await server.connect(transport);
error('project-context MCP server ready (stdio).');
