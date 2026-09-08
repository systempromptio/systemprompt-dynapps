// Startup checks. Called before the MCP server or sync CLI does any real work.
// On failure, throws an Error with a human- and LLM-readable message that
// references the install guide so the agent can open it and follow the steps.

import { existsSync } from 'node:fs';
import { join, resolve } from 'node:path';

import { resolveHome } from './config.mjs';

/** Path to the install guide — lives alongside the engine in project-context/. */
function guideRef(home) {
  const abs = resolve(home, 'INSTALL.md');
  return existsSync(abs) ? abs : '.cursor/.project/project-context/INSTALL.md';
}

/**
 * Run preflight checks.
 *
 * @param {{ requireAtlassian?: boolean }} options
 *   requireAtlassian — set true for the sync CLI (Atlassian keys are mandatory)
 * @throws {Error} if any check fails
 */
export function preflight({ requireAtlassian = false } = {}) {
  const home = resolveHome();
  const issues = [];

  // .env presence — warn but do not block (env may come from shell / CI)
  const envPath = join(resolve(home, '..'), '.env');
  if (!existsSync(envPath)) {
    issues.push(
      `No .env file found at ${envPath}\n` +
        `  → Copy .cursor/.project/project-context/.env.example to .cursor/.project/.env and fill in credentials.`,
    );
  }

  // Bedrock key — required by both MCP and sync (embeddings)
  if (!process.env.AWS_BEARER_TOKEN_BEDROCK?.trim()) {
    issues.push(
      'AWS_BEARER_TOKEN_BEDROCK is not set.\n' +
        '  → See Step 3 in the install guide (Bedrock API key from the AWS console).',
    );
  }

  // Atlassian keys — only required for sync
  if (requireAtlassian) {
    if (!process.env.ATLASSIAN_BASE_URL?.trim() && !process.env.ATLASSIAN_SITE?.trim()) {
      issues.push(
        'ATLASSIAN_BASE_URL is not set.\n' +
          '  → Set it to your Atlassian base URL, e.g. https://acme.atlassian.net',
      );
    }
    if (!process.env.ATLASSIAN_EMAIL?.trim()) {
      issues.push('ATLASSIAN_EMAIL is not set.');
    }
    if (!process.env.ATLASSIAN_API_TOKEN?.trim()) {
      issues.push(
        'ATLASSIAN_API_TOKEN is not set.\n' +
          '  → Get a token at https://id.atlassian.com/manage-profile/security/api-tokens',
      );
    }
  }

  if (issues.length === 0) return;

  const guide = guideRef(home);
  const list = issues.map((s, i) => `  ${i + 1}. ${s}`).join('\n');
  throw new Error(
    `Project Context is not configured correctly.\n` +
      `See the install guide: ${guide}\n\n` +
      `${list}`,
  );
}
