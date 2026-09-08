#!/usr/bin/env node
// opsx-eval: LLM-as-a-judge for OpenSpec artifacts.
//
// One judge core, two subcommands:
//   eval  — generate N candidates in memory from a generator manifest + case context,
//           judge each, and print an aggregate variability report.
//   judge — judge one already-produced file (thin CLI over the same core).
//
//   node run.mjs eval  --artifact fsd --dir fixtures/store-locator [--flow ba|sa|ambiguous] [--mode dev|release] [--runs 10] [--judge-samples 3] [--save] [--prompt-only]
//   node run.mjs judge --artifact fsd --file <path> [--context <dir>] [--flow ba|sa|ambiguous] [--mode dev|release] [--judge-samples 3] [--save] [--prompt-only]
//
// `judge --artifact discovery` is intentionally rejected (blocked on live
// primary/initial-context sourcing); use `eval --artifact discovery` instead.

import { existsSync, mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { join, resolve, basename } from 'node:path';

// Shared credentials loader (.project/.env). Side-effect import: keeps this
// module in sync with the rest of the tooling instead of duplicating a loader.
// Portable: swap this one line for a local loader if moving the module.
import '../lib/load-env.mjs';

import { loadConfig } from './lib/config.mjs';
import { resolveArtifact, resolveCaseDir, resolveContext, resolveGolden, resolveFlows, scopeBaselinesToDiscovery } from './lib/resolve.mjs';
import { createClient } from './lib/bedrock.mjs';
import {
  generateArtifact,
  judgeArtifact,
  previewGenerationPrompt,
  previewJudgePrompt,
} from './lib/judge.mjs';
import { formatReport, formatAggregate, formatPromptProfile } from './lib/report.mjs';

const JUDGE_CLI_UNSUPPORTED = new Set(['discovery']);

function parseArgs(argv) {
  const args = { flags: new Set(), _: [] };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--prompt-only' || a === '--save') {
      args.flags.add(a);
    } else if (a.startsWith('--')) {
      const key = a.slice(2);
      const val = argv[i + 1];
      args[key] = val;
      i++;
    } else {
      args._.push(a);
    }
  }
  return args;
}

function fail(msg) {
  process.stderr.write(`\nError: ${msg}\n`);
  process.exit(1);
}

/**
 * Resolve the active run mode into a config clone: a mode overrides the
 * generator, judge, run count, and judge self-consistency samples in one switch.
 * `--mode` selects it (default: config.default_mode). Explicit `--judge-samples`
 * still wins over the mode; `--runs` is applied later by each subcommand.
 * @returns {object} config clone with genModel/judgeModel/eval overridden + activeMode
 */
function applyMode(config, args) {
  const name = args.mode ?? config.defaultMode ?? null;
  let cfg = config;
  if (name) {
    const m = config.modes?.[name];
    if (!m) {
      fail(`Unknown --mode "${name}". Available: ${Object.keys(config.modes ?? {}).join(', ') || '(none defined)'}`);
    }
    cfg = {
      ...config,
      genModel: { ...config.genModel, id: m.generate ?? config.genModel.id },
      judgeModel: { ...config.judgeModel, id: m.judge ?? config.judgeModel.id },
      eval: {
        ...config.eval,
        runs: m.runs ?? config.eval.runs,
        judgeSamples: m.judgeSamples ?? config.eval.judgeSamples,
      },
    };
  }
  if (args['judge-samples'] != null) {
    const s = Math.max(1, Number.parseInt(args['judge-samples'], 10) || 1);
    cfg = { ...cfg, eval: { ...cfg.eval, judgeSamples: s } };
  }
  if (args['gen-model']) cfg = { ...cfg, genModel: { ...cfg.genModel, id: args['gen-model'] } };
  if (args['judge-model']) cfg = { ...cfg, judgeModel: { ...cfg.judgeModel, id: args['judge-model'] } };
  return { ...cfg, activeMode: name ?? '(config defaults)' };
}

function warnMissingContext(contextBlocks) {
  for (const c of contextBlocks) {
    if (!c.found) process.stderr.write(`Warning: context "${c.id}" not found in case dir — grounding for it will be skipped.\n`);
  }
}

/** Rough token estimate (~4 chars/token) — good enough for a budget guardrail. */
function estimateTokens(text) {
  return Math.round(String(text ?? '').length / 4);
}

/**
 * Warn when an assembled prompt is oversized. Not a hard limit (the model window
 * is far larger) — a big prompt is a design smell that the eval is loading too
 * much. See the opsx-eval skill, "Context budget".
 */
function warnLargeContext(label, system, user, limitTokens) {
  const tokens = estimateTokens(system) + estimateTokens(user);
  if (tokens > limitTokens) {
    process.stderr.write(
      `\nWARNING: assembled ${label} prompt is ~${tokens} tokens (> ${limitTokens}). ` +
        `The eval context is heavily overloaded — trim skills/command/sources or scope the schema step. ` +
        `See the opsx-eval skill, "Context budget".\n\n`,
    );
  }
  return tokens;
}

/** Summarize which live references were neutralized when assembling the prompt. */
function writeSubstitutionReport(resolved) {
  process.stderr.write(`\n----- MANIFEST SOURCING -----\n`);
  process.stderr.write(`  command: ${resolved.command ? `${resolved.command.ref}${resolved.command.exists ? '' : ' (NOT FOUND)'}` : '(none)'}\n`);
  const skills = resolved.skills ?? [];
  process.stderr.write(`  skills:  ${skills.length ? skills.map((s) => `${s.ref}${s.exists ? '' : ' (NOT FOUND)'}`).join(', ') : '(none)'}\n`);
  const rules = resolved.rules ?? [];
  process.stderr.write(`  rules:   ${rules.length ? rules.map((r) => `${r.ref.split('/').pop()}${r.exists ? '' : ' (NOT FOUND)'}`).join(', ') : '(none)'}\n`);
  if (resolved.schemaArtifact) {
    const sa = resolved.schemaArtifact;
    process.stderr.write(`  schema:  ${sa.ref}${sa.exists ? '' : ` (NOT RESOLVED: ${sa.error ?? 'unknown'})`}\n`);
  } else {
    process.stderr.write(`  schema:  (none)\n`);
  }
  const subs = resolved.substitutions ?? [];
  process.stderr.write(`----- LIVE-REFERENCE SUBSTITUTIONS (${subs.length}) -----\n`);
  for (const s of subs) {
    process.stderr.write(`  [${s.id}] ${s.kind}${s.redirectContext ? ` -> CONTEXT:${s.redirectContext}` : ''}\n`);
  }
  process.stderr.write(`\n`);
}

function timestamp() {
  return new Date().toISOString().replace(/[:.]/g, '-');
}

/** Run `total` tasks through `worker(i)` with at most `limit` in flight. */
async function runPool(total, limit, worker) {
  const results = new Array(total);
  let next = 0;
  const lane = async () => {
    for (;;) {
      const i = next++;
      if (i >= total) return;
      results[i] = await worker(i);
    }
  };
  await Promise.all(Array.from({ length: Math.max(1, Math.min(limit, total)) }, lane));
  return results;
}

async function runEval(args, config) {
  config = applyMode(config, args);
  const schema = args.schema ?? 'openspec-feature';
  const { artifact, dir } = args;
  const runs = args.runs ? Number.parseInt(args.runs, 10) : config.eval.runs;
  const concurrency = args.concurrency ? Number.parseInt(args.concurrency, 10) : config.eval.concurrency;
  const promptOnly = args.flags.has('--prompt-only');
  const save = args.flags.has('--save');

  if (!artifact || !dir) {
    fail('Usage: run.mjs eval --artifact <artifact> --dir <case-dir> [--schema openspec-feature] [--flow ba|sa|ambiguous] [--mode dev|release] [--runs N] [--concurrency N] [--judge-samples N] [--save] [--prompt-only]');
  }
  if (!Number.isInteger(runs) || runs < 1) fail(`--runs must be a positive integer (got ${args.runs}).`);
  if (!Number.isInteger(concurrency) || concurrency < 1) fail(`--concurrency must be a positive integer (got ${args.concurrency}).`);

  let resolved;
  let caseDir;
  try {
    // Use only the case dir's last segment as the openspec change name: the CLI
    // rejects a name with path separators, so passing the raw --dir (e.g.
    // `evalsuite/.../store-locator`) silently drops the whole OPENSPEC
    // INSTRUCTIONS block. Sanitize to a plain slug.
    const changeName = basename(dir).replace(/[^a-zA-Z0-9._-]/g, '-') || `${artifact}-eval`;
    resolved = resolveArtifact(config, schema, artifact, { changeName });
    caseDir = resolveCaseDir(config, schema, dir);
  } catch (err) {
    fail(err.message);
  }
  if (!resolved.scenario) {
    fail(`No generator manifest at ${resolved.scenarioPath}. eval needs a generator manifest for "${artifact}".`);
  }

  const fixtures = { flowsConfig: config.flows, states: config.states };
  // The candidate is generated against the generator's own context when it
  // declares one (a role-specific projection), else the judge's context.
  const genIds = resolved.generatorContextIds ?? resolved.contextIds;
  const fixtureSpecs = [resolved.goldenSpec, ...resolved.contextIds, ...genIds].filter((s) => s?.from);
  let flows;
  let previewCtx;
  try {
    flows = args.flow ? [args.flow] : resolveFlows(config, caseDir, artifact, { specs: fixtureSpecs });
    previewCtx = scopeBaselinesToDiscovery(resolveContext(caseDir, genIds, { flow: flows[0], ...fixtures }));
  } catch (err) {
    fail(err.message);
  }
  const flowed = flows.length > 1 || flows[0] != null;

  {
    const { system, user } = previewGenerationPrompt({ resolved, contextBlocks: previewCtx });
    if (promptOnly) {
      process.stdout.write(`----- GENERATION SYSTEM -----\n${system}\n\n----- GENERATION USER -----\n${user}\n`);
      const tokens = warnLargeContext('generation', system, user, config.eval.contextWarnTokens);
      writeSubstitutionReport(resolved);
      process.stderr.write(formatPromptProfile(system, user));
      process.stderr.write(`Estimated generation prompt size: ~${tokens} tokens.\n`);
      return 0;
    }
    warnLargeContext('generation', system, user, config.eval.contextWarnTokens);
  }

  const client = createClient(config);
  const lanes = Math.min(concurrency, runs);
  process.stderr.write(
    `eval ${schema}/${artifact} [mode: ${config.activeMode}]: ${flowed ? `${flows.length} flows (${flows.join(', ')}) x ` : ''}${runs} runs${flowed ? '/flow' : ''} (${lanes} in parallel) — gen ${client.genModel} / judge ${client.judgeModel} (case: ${caseDir})\n`,
  );

  const judgeSamples = config.eval.judgeSamples ?? 1;
  if (judgeSamples > 1) {
    process.stderr.write(`judge self-consistency: ${judgeSamples} samples/candidate (median score + majority-vote criticals)\n`);
  }

  const flowRuns = [];
  for (const flow of flows) {
    const contextBlocks = scopeBaselinesToDiscovery(resolveContext(caseDir, resolved.contextIds, { flow, ...fixtures }));
    // Generator sees its own role-specific projection when declared; otherwise it
    // shares the judge's context (identity check keeps the common case a no-op).
    const genContextBlocks =
      genIds === resolved.contextIds
        ? contextBlocks
        : scopeBaselinesToDiscovery(resolveContext(caseDir, genIds, { flow, ...fixtures }));
    warnMissingContext(contextBlocks);
    const golden = resolveGolden(caseDir, artifact, { flow, ...fixtures, spec: resolved.goldenSpec });
    if (flowed) process.stderr.write(`\n----- flow: ${flow} -----\n`);

    let done = 0;
    let errored = 0;
    // A single run must never abort the batch: any error (retries exhausted, a
    // non-retryable API status, a judge that never returns JSON) is caught and
    // recorded as a scored-0 FAIL so the pool always completes and everything is
    // evaluated. runPool's Promise.all therefore never rejects.
    const results = await runPool(runs, concurrency, async (i) => {
      let generationRaw = '';
      let candidate = '';
      try {
        ({ raw: generationRaw, candidate } = await generateArtifact({ client, resolved, contextBlocks: genContextBlocks }));
        const { verdict, raw, raws } = await judgeArtifact({ client, resolved, contextBlocks, golden, candidate, samples: judgeSamples });
        done++;
        const spread =
          judgeSamples > 1 && Array.isArray(verdict.judge_samples)
            ? ` [samples ${verdict.judge_samples.map((s) => (typeof s.score === 'number' ? s.score.toFixed(2) : '?')).join('/')}]`
            : '';
        process.stderr.write(
          `  ${flowed ? `[${flow}] ` : ''}run ${i + 1}/${runs} (${done} done): ${verdict.pass ? 'PASS' : 'FAIL'} (${typeof verdict.score === 'number' ? verdict.score.toFixed(2) : '?'})${spread}\n`,
        );
        return { generationRaw, candidate, verdict, raw, raws };
      } catch (err) {
        errored++;
        const message = err?.message ?? String(err);
        process.stderr.write(
          `  ${flowed ? `[${flow}] ` : ''}run ${i + 1}/${runs} ERRORED — counted as FAIL, batch continues: ${message}\n`,
        );
        const verdict = {
          pass: false,
          score: 0,
          rubric_scores: {},
          critical_findings: [
            {
              issue: 'Run errored before a verdict could be produced (recorded as a fail; the batch was not aborted).',
              evidence: message.slice(0, 300),
            },
          ],
          warnings: [],
          missing_evidence: [],
          judge_samples: [],
          errored: true,
        };
        return { generationRaw, candidate, verdict, raw: `RUN ERROR: ${message}`, raws: [] };
      }
    });
    if (errored) process.stderr.write(`  [${flow ?? 'flow'}] ${errored}/${runs} run(s) errored and were scored 0 (batch completed).\n`);
    flowRuns.push({ flow, results });
    // Persist this flow immediately so an external interruption can never lose an
    // already-completed flow.
    if (save) saveFlowResults({ config, schema, artifact, caseDir, client, resolved, flow, results, runs });
  }

  for (const { flow, results } of flowRuns) {
    const meta = {
      schema,
      artifact,
      genModel: client.genModel,
      model: client.judgeModel,
      passThreshold: resolved.passThreshold,
      caseDir: flow ? `${caseDir} (flow: ${flow})` : caseDir,
      runs,
    };
    process.stdout.write(formatAggregate(results.map((r) => r.verdict), meta));
  }

  const allResults = flowRuns.flatMap((f) => f.results);
  const totalRuns = allResults.length;
  const totalPass = allResults.filter((r) => r.verdict.pass).length;
  if (flowed) {
    process.stdout.write(`\n## Combined across ${flows.length} flows: ${totalPass}/${totalRuns} pass\n`);
    for (const { flow, results } of flowRuns) {
      const p = results.filter((r) => r.verdict.pass).length;
      process.stdout.write(`  - ${flow}: ${p}/${results.length} pass\n`);
    }
    process.stdout.write('\n');
  }

  return totalPass === totalRuns ? 0 : 2;
}

/** Persist one flow's full run trail + aggregate. Called incrementally per flow. */
function saveFlowResults({ config, schema, artifact, caseDir, client, resolved, flow, results, runs }) {
  const flowTag = flow ? `-${flow}` : '';
  const outDir = join(config.paths.results, `${schema}-${artifact}-${basename(caseDir)}${flowTag}-${timestamp()}`);
  mkdirSync(outDir, { recursive: true });
  results.forEach((r, i) => {
    const n = String(i + 1).padStart(2, '0');
    // Full trail per run so a recurring finding (traced by run number in the
    // aggregate) can be inspected end to end: raw generation (with the plan),
    // the stripped candidate the judge saw, the raw judge output, and verdict.
    writeFileSync(join(outDir, `run-${n}-generation-raw.md`), r.generationRaw, 'utf8');
    writeFileSync(join(outDir, `run-${n}-candidate.md`), r.candidate, 'utf8');
    writeFileSync(join(outDir, `run-${n}-judge-raw.txt`), r.raw, 'utf8');
    // When self-consistency is on, keep every judge sample so the folded
    // verdict's spread (the whole point of the fix) stays inspectable.
    if (Array.isArray(r.raws) && r.raws.length > 1) {
      r.raws.forEach((raw, k) => {
        writeFileSync(join(outDir, `run-${n}-judge-sample-${k + 1}.txt`), raw, 'utf8');
      });
    }
    writeFileSync(join(outDir, `run-${n}-verdict.json`), JSON.stringify(r.verdict, null, 2), 'utf8');
  });
  const meta = { schema, artifact, flow, genModel: client.genModel, model: client.judgeModel, passThreshold: resolved.passThreshold, caseDir, runs };
  writeFileSync(join(outDir, 'aggregate.txt'), formatAggregate(results.map((r) => r.verdict), meta), 'utf8');
  writeFileSync(join(outDir, 'meta.json'), JSON.stringify(meta, null, 2), 'utf8');
  process.stderr.write(`Saved ${results.length} runs + aggregate: ${resolve(outDir)}\n`);
}

async function runJudge(args, config) {
  config = applyMode(config, args);
  const schema = args.schema ?? 'openspec-feature';
  const { artifact, file, context } = args;
  const promptOnly = args.flags.has('--prompt-only');
  const save = args.flags.has('--save');

  if (!artifact || !file) {
    fail('Usage: run.mjs judge --artifact <artifact> --file <path> [--schema openspec-feature] [--context <dir>] [--flow ba|sa|ambiguous] [--mode dev|release] [--judge-samples N] [--save] [--prompt-only]');
  }
  if (JUDGE_CLI_UNSUPPORTED.has(artifact) && !context) {
    fail(
      `judge for "${artifact}" needs its primary sources: pass --context <case> so the fixtures ` +
        `(initial-context + baseline specs) supply them, or use \`eval --artifact ${artifact} --dir <case>\`. ` +
        `With --context you can score the golden itself as a ceiling check.`,
    );
  }
  if (!existsSync(file)) fail(`Candidate file not found: ${file}`);

  let resolved;
  try {
    resolved = resolveArtifact(config, schema, artifact, { withInstructions: false });
  } catch (err) {
    fail(err.message);
  }

  const flow = args.flow ?? null;
  let contextBlocks = [];
  let golden = { found: false, content: '' };
  if (context) {
    let caseDir;
    try {
      caseDir = resolveCaseDir(config, schema, context);
    } catch (err) {
      fail(err.message);
    }
    contextBlocks = scopeBaselinesToDiscovery(
      resolveContext(caseDir, resolved.contextIds, { flow, flowsConfig: config.flows, states: config.states }),
    );
    warnMissingContext(contextBlocks);
    golden = resolveGolden(caseDir, artifact, {
      flow,
      flowsConfig: config.flows,
      states: config.states,
      spec: resolved.goldenSpec,
    });
  } else if (resolved.contextIds.length) {
    process.stderr.write(
      `Note: no --context given; grounding dimensions (${resolved.contextIds.map((c) => c.id).join(', ')}) will be judged as unverifiable.\n`,
    );
  }

  const candidate = readFileSync(file, 'utf8');

  {
    const { system, user } = previewJudgePrompt({ resolved, contextBlocks, golden, candidate });
    if (promptOnly) {
      process.stdout.write(`----- JUDGE SYSTEM -----\n${system}\n\n----- JUDGE USER -----\n${user}\n`);
      const tokens = warnLargeContext('judge', system, user, config.eval.contextWarnTokens);
      process.stderr.write(`Estimated judge prompt size: ~${tokens} tokens.\n`);
      return 0;
    }
    warnLargeContext('judge', system, user, config.eval.contextWarnTokens);
  }

  const client = createClient(config);
  const judgeSamples = config.eval.judgeSamples ?? 1;
  process.stderr.write(
    `judge ${schema}/${artifact} [mode: ${config.activeMode}] with ${client.judgeModel} (candidate: ${file})` +
      (judgeSamples > 1 ? ` — ${judgeSamples} samples (median + majority-vote criticals)` : '') +
      '\n',
  );

  const { verdict, raw } = await judgeArtifact({ client, resolved, contextBlocks, golden, candidate, samples: judgeSamples });
  const meta = { schema, artifact, model: client.judgeModel, candidatePath: file, passThreshold: resolved.passThreshold };
  process.stdout.write(formatReport(verdict, meta));

  if (save) {
    mkdirSync(config.paths.results, { recursive: true });
    const name = `${schema}-${artifact}-${basename(file).replace(/\.md$/, '')}-${timestamp()}.json`;
    const outPath = join(config.paths.results, name);
    writeFileSync(outPath, JSON.stringify({ meta, verdict, raw }, null, 2), 'utf8');
    process.stderr.write(`Saved verdict: ${resolve(outPath)}\n`);
  }

  return verdict.pass ? 0 : 2;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const command = args._[0];
  const config = loadConfig();

  let code;
  if (command === 'eval') code = await runEval(args, config);
  else if (command === 'judge') code = await runJudge(args, config);
  else {
    fail(
      `Unknown command "${command ?? ''}". Use:\n` +
        `  run.mjs eval  --artifact <a> --dir <case-dir> [--flow ba|sa|ambiguous] [--mode dev|release] [--runs N] [--judge-samples N] [--save] [--prompt-only]\n` +
        `  run.mjs judge --artifact <a> --file <path> [--context <dir>] [--flow ba|sa|ambiguous] [--mode dev|release] [--judge-samples N] [--save] [--prompt-only]`,
    );
  }
  process.exit(code);
}

main().catch((err) => {
  fail(err?.stack || String(err));
});
