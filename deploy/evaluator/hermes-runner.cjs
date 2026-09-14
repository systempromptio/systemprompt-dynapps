'use strict';
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn } = require('node:child_process');
const { StringDecoder } = require('node:string_decoder');
const { createRelay, childEnvironment } = require('./codex-runner.cjs');
const EXECUTABLE = '/opt/hermes-source/.venv/bin/hermes';

function nativeEnvironment(source, localToken) {
  const env = childEnvironment(source, localToken);
  delete env.CODEX_HOME; delete env.CODEX_LOCAL_PROXY_TOKEN;
  return { ...env, PATH: '/opt/hermes-source/.venv/bin:/usr/local/bin:/usr/bin:/bin',
    HERMES_HOME: '/home/tester/.hermes', HERMES_INFERENCE_PROVIDER: 'custom',
    PYTHONDONTWRITEBYTECODE: '1', HERMES_TELEMETRY_DISABLED: '1' };
}
function nativeConfiguration(configuration, { turns, model, purpose, localToken }) {
  if (!configuration || typeof configuration !== 'object' || Array.isArray(configuration) ||
      !Number.isInteger(turns) || turns < 1 || turns > 100 ||
      !/^[a-zA-Z0-9._/:-]{1,128}$/.test(model) ||
      !['execution', 'review'].includes(purpose) || !/^[a-f0-9]{64}$/.test(localToken)) {
    throw new Error('Invalid Hermes runtime configuration');
  }
  // Hermes0.21.3 host-gates OPENAI_API_KEY to OpenAI endpoints. A private
  // loopback provider instead requires its own explicit model.api_key.
  // Only this per-process token enters the temporary read-only native config;
  // the upstream execution capability stays exclusively in the parent relay.
  return { ...configuration,
    agent: { ...configuration.agent, max_turns: turns },
    auxiliary: { ...configuration.auxiliary, title_generation: { ...configuration.auxiliary?.title_generation, enabled: false } },
    model: { ...configuration.model, default: model, provider: 'custom',
      base_url: 'http://127.0.0.1:8091/v1', api_mode: 'chat_completions', api_key: localToken },
    fallback_providers: [],
    mcp_servers: purpose === 'execution' ? { evaluation_fixture: {
      url: 'http://127.0.0.1:8091/mcp/evaluation_fixture', headers: { Authorization: `Bearer ${localToken}` },
      tools: { include: ['evaluation_fixture'] }, timeout: 10 } } : {},
  };
}
function nativeArguments(prompt, model, purpose) {
  if (typeof prompt !== 'string' || Buffer.byteLength(prompt) > 65536 || prompt.includes('\0') ||
      !/^[a-zA-Z0-9._/:-]{1,128}$/.test(model) || !['execution', 'review'].includes(purpose)) throw new Error('Invalid Hermes native arguments');
  // Oneshot validates the configured MCP server name before discovery creates aliases.
  return [`--oneshot=${prompt}`, '--model', model, '--provider', 'custom', '--toolsets',
    purpose === 'execution' ? 'file,evaluation_fixture' : 'file', '--usage-file', '/tmp/hermes-evidence/usage.json'];
}
function sandboxArguments(config, evidence, skills, purpose, nativeArgs) {
  if (![config, evidence, skills].every(value => path.isAbsolute(value)) || !['execution', 'review'].includes(purpose)) throw new Error('Invalid native namespace mounts');
  return ['--die-with-parent', '--new-session', '--unshare-pid', '--unshare-ipc', '--unshare-uts',
    '--ro-bind', '/', '/', '--proc', '/proc', '--dev', '/dev', '--tmpfs', '/tmp',
    '--tmpfs', '/home/tester/.hermes', '--ro-bind', config, '/home/tester/.hermes/config.yaml',
    '--ro-bind', skills, '/home/tester/.hermes/skills', '--bind', evidence, '/tmp/hermes-evidence',
    purpose === 'execution' ? '--bind' : '--ro-bind', '/home/tester/work', '/home/tester/work',
    '--chdir', '/home/tester/work', '--', EXECUTABLE, ...nativeArgs];
}
function readUsage(file) {
  let fd;
  try {
    fd = fs.openSync(file, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
    const stat = fs.fstatSync(fd);
    if (!stat.isFile() || stat.size > 65536 || stat.nlink !== 1) throw new Error('Invalid native usage file');
    const bytes = fs.readFileSync(fd);
    if (bytes.length > 65536) throw new Error('Native usage file grew beyond its bound');
    const value = JSON.parse(bytes.toString('utf8'));
    if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('Invalid native usage object');
    return { value, sha256: crypto.createHash('sha256').update(bytes).digest('hex') };
  } finally { if (fd !== undefined) fs.closeSync(fd); }
}
function resultEvidence(usage, outcome, model, toolCalls) {
  const coherent = usage && usage.value.model === model && typeof usage.value.session_id === 'string' && usage.value.session_id.length > 0;
  return { type: 'hermes.result', process_exit_code: outcome.code, signal: outcome.signal,
    usage: usage?.value ?? null, usage_sha256: usage?.sha256 ?? null, usage_identity_matches: Boolean(coherent),
    usage_independently_verified: false, output_limit_reached: Boolean(outcome.outputLimit),
    tool_calls: toolCalls, tool_evidence_source: 'native_tool_results_in_provider_requests' };
}
async function main(argv, source = process.env) {
  if (argv.length !== 5 || Buffer.byteLength(argv[4]) > 65536 || argv[4].includes('\0')) throw new Error('Invalid Hermes runner arguments');
  const [turns, tokens, model, purpose, prompt] = argv;
  const localToken = crypto.randomBytes(32).toString('hex');
  const relay = createRelay({ relay: source.HERMES_EVALUATION_RELAY, token: source.HERMES_EVALUATION_TOKEN,
    session: source.HERMES_EVALUATION_SESSION, model, purpose, maxRequests: Number(turns),
    maxOutputTokens: Number(tokens), localToken, wire: 'chat' });
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'hermes-native-'));
  let child;
  let outputLimit = false;
  let written = 0;
  let sequence = 0;
  const event = value => {
    const line = JSON.stringify({ ...value, sequence }) + '\n';
    if (written + Buffer.byteLength(line) > 15 * 1024 * 1024 && value.type !== 'hermes.result') {
      outputLimit = true; terminate(); return;
    }
    sequence++; written += Buffer.byteLength(line); process.stdout.write(line);
  };
  const terminate = () => {
    if (!child?.pid) return;
    try { process.kill(-child.pid, 'SIGTERM'); } catch {}
    const timer = setTimeout(() => { try { process.kill(-child.pid, 'SIGKILL'); } catch {} }, 2000); timer.unref();
  };
  try {
    const configuration = nativeConfiguration(
      JSON.parse(fs.readFileSync('/home/tester/.hermes/evaluator-config.json', 'utf8')),
      { turns: Number(turns), model, purpose, localToken });
    const config = path.join(temporary, 'config.yaml');
    fs.writeFileSync(config, JSON.stringify(configuration), { mode: 0o600 });
    const evidence = path.join(temporary, 'evidence'); fs.mkdirSync(evidence, { mode: 0o700 });
    const emptySkills = path.join(temporary, 'empty-skills'); fs.mkdirSync(emptySkills, { mode: 0o700 });
    const skills = purpose === 'execution' ? '/home/tester/.hermes/skills' : emptySkills;
    fs.mkdirSync(skills, { recursive: true, mode: 0o700 });
    await new Promise((resolve, reject) => { relay.server.once('error', reject); relay.server.listen(8091, '127.0.0.1', resolve); });
    const nativeArgs = nativeArguments(prompt, model, purpose);
    child = spawn('/usr/bin/bwrap', sandboxArguments(config, evidence, skills, purpose, nativeArgs),
      { env: nativeEnvironment(source, localToken), stdio: ['ignore', 'pipe', 'pipe'], detached: true });
    for (const [channel, pipe] of [['stdout', child.stdout], ['stderr', child.stderr]]) {
      const decoder = new StringDecoder('utf8');
      pipe.on('data', bytes => { const text = decoder.write(bytes); if (text) event({ type: `hermes.${channel}`, text }); });
      pipe.once('end', () => { const text = decoder.end(); if (text) event({ type: `hermes.${channel}`, text }); });
    }
    process.once('SIGTERM', terminate); process.once('SIGINT', terminate);
    let outcome;
    try {
      outcome = await new Promise((resolve, reject) => { child.once('error', reject); child.once('close', (code, signal) => resolve({ code, signal, outputLimit })); });
    } finally { process.removeListener('SIGTERM', terminate); process.removeListener('SIGINT', terminate); terminate(); }
    let usage = null;
    try { usage = readUsage(path.join(evidence, 'usage.json')); }
    catch { event({ type: 'hermes.diagnostic', message: 'Native usage file missing, malformed, linked, or oversized' }); }
    event(resultEvidence(usage, outcome, model, relay.toolCalls()));
    process.exitCode = outcome.code === 0 && !outcome.signal && !outputLimit ? 0 : 1;
  } finally {
    terminate(); await relay.close(); fs.rmSync(temporary, { recursive: true, force: true });
  }
}
module.exports = { nativeArguments, nativeConfiguration, nativeEnvironment, sandboxArguments, readUsage, resultEvidence };
if (require.main === module) main(process.argv.slice(2)).catch(() => {
  process.stdout.write(JSON.stringify({ type: 'hermes.launch_error', message: 'Hermes native namespace/configuration startup failed; no verified completion' }) + '\n');
  process.exitCode = 1;
});
