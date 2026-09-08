// Configuration loader.
//
// Reads config.yaml into a plain object with camelCase keys. Chunking strategy
// is fixed per source type. Each source may override its origin `path` and
// filter which files are taken with include/exclude globs.

import { readFileSync, existsSync } from 'node:fs';
import { resolve, dirname, isAbsolute, join } from 'node:path';
import { parse } from 'yaml';

/** Resolve the project home directory (contains config.yaml, inbox/, .index/). */
export function resolveHome() {
  return process.env.PROJECT_CONTEXT_HOME
    ? resolve(process.env.PROJECT_CONTEXT_HOME)
    : process.cwd();
}

/** Resolve the config.yaml path from an explicit path, env, or home. */
export function resolveConfigPath(explicit) {
  if (explicit) return resolve(explicit);
  return join(resolveHome(), 'config.yaml');
}

function resolvePath(base, p, fallback) {
  const value = p ?? fallback;
  return isAbsolute(value) ? value : resolve(base, value);
}

/** Normalize an include/exclude value into a string[] of globs, or null. */
function toGlobList(value) {
  if (value == null) return null;
  const list = (Array.isArray(value) ? value : [value]).map((g) => String(g).trim()).filter(Boolean);
  return list.length ? list : null;
}

/**
 * Load and normalize config.yaml.
 * @param {string} [configPath]
 * @returns {object} config
 */
export function loadConfig(configPath) {
  const path = resolveConfigPath(configPath);
  if (!existsSync(path)) {
    throw new Error(
      `config.yaml not found at ${path}. Set PROJECT_CONTEXT_HOME or run from the project root.`,
    );
  }

  const raw = parse(readFileSync(path, 'utf8')) ?? {};
  const home = dirname(path);
  const inboxPath = resolvePath(home, raw.inbox_path, 'inbox');
  const b = raw.bedrock ?? {};
  const c = raw.chunking ?? {};
  const r = raw.retrieval ?? {};
  const sources = {};
  for (const [name, s] of Object.entries(raw.sources ?? {})) {
    // `path` is the origin: where originals are read from. It defaults to
    // inbox/<name>. An override is resolved relative to the config home (the
    // project root sits a few levels up from the deploy home). When overridden
    // to an external directory, that directory is never written to - files are
    // materialized into inbox/<name> instead.
    const origin = s?.path
      ? (isAbsolute(s.path) ? s.path : resolve(home, s.path))
      : join(inboxPath, name);
    sources[name] = {
      strategy: s?.strategy ?? 'script',
      description: s?.description ?? '',
      path: origin,
      include: toGlobList(s?.include),
      exclude: toGlobList(s?.exclude),
    };
  }

  return {
    home,
    inboxPath,
    indexPath: resolvePath(home, raw.index_path, '.index'),
    envPath: resolvePath(home, raw.env_path, '.env'),
    bedrock: {
      region: process.env.PROJECT_CONTEXT_BEDROCK_REGION ?? 'eu-central-1',
      embedModel: b.embed_model ?? 'amazon.titan-embed-text-v2:0',
      embedDim: b.embed_dim ?? 1024,
      chunkModel: b.chunk_model ?? 'openai.gpt-oss-120b-1:0',
      rerankModel: b.rerank_model ?? 'cohere.rerank-v3-5:0',
      embedConcurrency: b.embed_concurrency ?? 16,
      maxConcurrency: b.max_concurrency ?? 8,
    },
    chunking: {
      targetChars: c.target_chars ?? 1600,
      maxChars: c.max_chars ?? 3000,
      minChars: c.min_chars ?? 300,
      splitTolerance: c.split_tolerance ?? 100,
      enrich: c.enrich ?? true,
    },
    dates: {
      extractViaLLM: raw.dates?.extract_via_llm ?? true,
    },
    retrieval: {
      topK: r.top_k ?? 5,
      candidateMultiplier: r.candidate_multiplier ?? 4,
      rerankEnabled: r.rerank_enabled ?? true,
    },
    indexing: {
      docConcurrency: raw.indexing?.doc_concurrency ?? 6,
    },
    sources,
  };
}

/** The fixed chunking strategy for a source type (default: script). */
export function strategyFor(config, sourceType) {
  return config.sources[sourceType]?.strategy ?? 'script';
}
