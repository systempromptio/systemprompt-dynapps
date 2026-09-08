// Configuration loader for the eval module.
//
// Reads config.yaml into a plain object with camelCase keys and resolves paths
// relative to the module home (the directory that contains config.yaml).

// Credentials are loaded separately via the shared ../lib/load-env.mjs loader,
// so this module does not deal with .env resolution. Eval model placement is
// owned by config.yaml; .env only carries credentials shared with the rest of the
// tooling.

import { readFileSync, existsSync } from 'node:fs';
import { resolve, dirname, isAbsolute, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parse } from 'yaml';

/** Module home = the directory that contains this lib/ folder's parent. */
export function resolveHome() {
  const here = dirname(fileURLToPath(import.meta.url));
  return resolve(here, '..');
}

function resolvePath(base, p, fallback) {
  const value = p ?? fallback;
  return isAbsolute(value) ? value : resolve(base, value);
}

/**
 * Load and normalize config.yaml.
 * @param {string} [configPath]
 * @returns {object} config
 */
export function loadConfig(configPath) {
  const home = resolveHome();
  const path = configPath ? resolve(configPath) : join(home, 'config.yaml');
  if (!existsSync(path)) {
    throw new Error(`config.yaml not found at ${path}`);
  }

  const raw = parse(readFileSync(path, 'utf8')) ?? {};
  const t = raw.temperatures ?? {};
  const e = raw.eval ?? {};
  const s = raw.scoring ?? {};
  const p = raw.paths ?? {};
  const m = raw.model ?? {};
  const region = m.region ?? 'us-east-1';
  const maxTokens = Number.parseInt(m.max_tokens ?? '8192', 10);
  if (!Number.isInteger(maxTokens) || maxTokens < 1) {
    throw new Error(`model.max_tokens must be a positive integer in config.yaml (got ${m.max_tokens}).`);
  }
  const genModelId = m.generate?.id ?? 'openai.gpt-oss-120b-1:0';
  const judgeModelId = m.judge?.id ?? 'mistral.mistral-large-3-675b-instruct';

  const rawModes = raw.modes ?? {};
  const modes = {};
  for (const [name, v] of Object.entries(rawModes)) {
    modes[name] = {
      generate: v?.generate ?? null,
      judge: v?.judge ?? null,
      runs: v?.runs != null ? Number.parseInt(v.runs, 10) : null,
      judgeSamples: v?.judge_samples != null ? Math.max(1, Number.parseInt(v.judge_samples, 10) || 1) : null,
    };
  }
  const defaultMode = raw.default_mode ?? null;

  const rawFlows = raw.flows ?? {};
  const flows = {
    order: Array.isArray(rawFlows.order) ? rawFlows.order.map(String) : [],
    contextOverlay: rawFlows.context_overlay ?? '{context}.{flow}.md',
    golden: rawFlows.golden ?? '{artifact}.{flow}.md',
  };

  const rawStates = raw.states ?? {};
  const states = {
    draft: rawStates.draft ?? 'initial',
    settled: rawStates.settled ?? 'final',
    reviewSession: rawStates.review_session ?? '.context/review-session.{artifact}.{flow}.md',
  };

  return {
    modes,
    defaultMode,
    flows,
    states,
    home,
    genModel: {
      id: genModelId,
      region,
      maxTokens,
    },
    judgeModel: {
      id: judgeModelId,
      region,
      maxTokens,
    },
    temperatures: {
      generate: t.generate ?? 0.7,
      judge: t.judge ?? 0,
    },
    eval: {
      runs: e.runs ?? 10,
      concurrency: e.concurrency ?? 20,
      contextWarnTokens: e.context_warn_tokens ?? 100000,
      judgeSamples: Math.max(1, Number.parseInt(e.judge_samples ?? 1, 10) || 1),
    },
    scoring: {
      passThreshold: s.pass_threshold ?? 0.8,
    },
    paths: {
      suite: resolvePath(home, p.suite, 'evalsuite'),
      results: resolvePath(home, p.results, '.results'),
      // OPENSPEC_ROOT env wins over config; both fall back to the templates
      // scaffold. Loaded by the shared ../lib/load-env.mjs before loadConfig runs.
      openspecRoot: resolvePath(home, process.env.OPENSPEC_ROOT ?? p.openspec_root, '../templates/openspec'),
      skillsRoot: resolvePath(home, p.skills_root, '../../skills'),
    },
  };
}
