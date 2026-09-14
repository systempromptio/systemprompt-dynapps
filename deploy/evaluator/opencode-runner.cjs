'use strict';
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn } = require('node:child_process');
const { createRelay, childEnvironment } = require('./codex-runner.cjs');
const EXECUTABLE = '/usr/local/bin/opencode';

function nativeEnvironment(source, inline) {
  const environment = childEnvironment(source, '');
  delete environment.CODEX_HOME; delete environment.CODEX_LOCAL_PROXY_TOKEN;
  return { ...environment, PATH: '/usr/local/bin:/usr/bin:/bin',
    OPENCODE_CONFIG: '/home/tester/.config/opencode/opencode.json', OPENCODE_CONFIG_CONTENT: inline,
    OPENCODE_DISABLE_PROJECT_CONFIG: '1', OPENCODE_DISABLE_AUTOUPDATE: '1', OPENCODE_DISABLE_MODELS_FETCH: '1',
    OPENCODE_DISABLE_AUTOCOMPACT: '1', OPENCODE_DISABLE_PRUNE: '1', OPENCODE_DISABLE_FFF: '1',
    OPENCODE_EXPERIMENTAL_DISABLE_FILEWATCHER: '1', DO_NOT_TRACK: '1' };
}
function nativeConfiguration(configuration, localToken, purpose) {
  if (!configuration || typeof configuration !== 'object' || Array.isArray(configuration) ||
      !/^[a-f0-9]{64}$/.test(localToken) || !['execution', 'review'].includes(purpose) ||
      JSON.stringify(configuration.enabled_providers) !== '["systemprompt"]') throw Error('Invalid OpenCode native configuration');
  return { ...configuration, provider: { systemprompt: { npm: '@ai-sdk/openai-compatible', name: 'Evaluation gateway',
    options: { baseURL: 'http://127.0.0.1:8091/v1', apiKey: localToken, headers: { 'x-inference-protocol': 'openai-chat' } } } },
    mcp: purpose === 'execution' ? { evaluation_fixture: { type: 'remote', url: 'http://127.0.0.1:8091/mcp/evaluation_fixture',
      oauth: false, timeout: 5000, headers: { Authorization: `Bearer ${localToken}` } } } : {} };
}
function sandboxArguments(config, skills, purpose, nativeArgs) {
  if (![config, skills].every(value => path.isAbsolute(value)) || !['execution', 'review'].includes(purpose)) throw Error('Invalid OpenCode namespace mounts');
  return ['--die-with-parent', '--new-session', '--unshare-pid', '--unshare-ipc', '--unshare-uts',
    '--ro-bind', '/', '/', '--proc', '/proc', '--dev', '/dev', '--tmpfs', '/tmp',
    '--tmpfs', '/home/tester/.config/opencode', '--ro-bind', config, '/home/tester/.config/opencode/opencode.json',
    '--ro-bind', skills, '/home/tester/.config/opencode/skills',
    ...['.cache/opencode', '.local/share/opencode', '.local/state/opencode'].flatMap(directory => ['--bind', `/home/tester/${directory}`, `/home/tester/${directory}`]),
    purpose === 'execution' ? '--bind' : '--ro-bind', '/home/tester/work', '/home/tester/work',
    '--chdir', '/home/tester/work', '--', EXECUTABLE, ...nativeArgs];
}
function invocation(argv) {
  if (argv.length !== 15 || argv[5] !== '--pure' || argv[6] !== 'run' || argv[7] !== '--format' || argv[8] !== 'json' ||
      argv[9] !== '--model' || argv[10] !== `systemprompt/${argv[2]}` || argv[11] !== '--agent' || argv[12] !== 'evaluation' ||
      argv[13] !== '--' || Buffer.byteLength(argv[14]) > 65536 || argv[14].includes('\0') || Buffer.byteLength(argv[4]) > 65536) throw Error('Invalid bounded OpenCode runner arguments');
  const config = JSON.parse(argv[4]);
  if (config.agent?.evaluation?.steps !== Number(argv[0]) || config.provider?.systemprompt?.models?.[argv[2]]?.limit?.output !== Number(argv[1])) throw Error('OpenCode advisory settings differ from hard relay bounds');
  return { turns: Number(argv[0]), tokens: Number(argv[1]), model: argv[2], purpose: argv[3], inline: argv[4], nativeArgs: argv.slice(5) };
}
function groupMembers(group) {
  const members = [];
  for (const entry of fs.readdirSync('/proc')) {
    if (!/^[0-9]+$/.test(entry)) continue;
    let stat;
    try { stat = fs.readFileSync(`/proc/${entry}/stat`, 'utf8'); }
    catch (error) { if (['ENOENT', 'ESRCH'].includes(error.code)) continue; throw error; }
    const fields = stat.slice(stat.lastIndexOf(')') + 2).split(' ');
    if (Number(fields[2]) === group && Number(fields[3]) === group) members.push({ pid: Number(entry), start: fields[19], state: fields[0] });
  }
  return members;
}
function terminateProcessGroup(child) {
  if (!Number.isInteger(child?.pid) || child.pid <= 0) throw Error('Missing owned native process group');
  // A detached launch owns both this process group and session. Never resume
  // signalling after the group disappears, even if its numeric ID is reused.
  const initial = groupMembers(child.pid);
  if (!initial.length) return Promise.resolve();
  const anchors = new Map(initial.map(member => [member.pid, member.start]));
  return new Promise((resolve, reject) => {
    const started = Date.now();
    let killed = false;
    const tick = () => {
      try {
        const members = groupMembers(child.pid);
        if (!members.length || members.every(member => member.state === 'Z')) { resolve(); return; }
        if (!members.some(member => anchors.get(member.pid) === member.start)) throw Error('Owned native group identity cannot be established');
        for (const member of members) anchors.set(member.pid, member.start);
        if (Date.now() - started >= 4000) throw Error('Owned native process group cleanup exceeded bound');
        if (Date.now() - started >= 2000 && !killed) { killed = true; process.kill(-child.pid, 'SIGKILL'); }
        setTimeout(tick, 25);
      } catch (error) { if (error.code === 'ESRCH') resolve(); else reject(error); }
    };
    // Deliver TERM once, while the captured session members establish ownership.
    try { process.kill(-child.pid, 'SIGTERM'); } catch (error) { if (error.code === 'ESRCH') { resolve(); return; } reject(error); return; }
    tick();
  });
}
async function main(argv, source = process.env) {
  const args = invocation(argv), localToken = crypto.randomBytes(32).toString('hex');
  const original = '/home/tester/.config/opencode/opencode.json';
  if (fs.statSync(original).size > 65536) throw Error('OpenCode configuration exceeds bound');
  const configuration = JSON.parse(fs.readFileSync(original, 'utf8'));
  const upstream = configuration.provider?.systemprompt?.options;
  if (typeof upstream?.baseURL !== 'string' || !upstream.baseURL.endsWith('/v1')) throw Error('Missing OpenCode gateway');
  const relay = createRelay({ relay: upstream.baseURL.slice(0, -3), token: upstream.apiKey, session: upstream.headers?.['x-session-id'],
    model: args.model, purpose: args.purpose, maxRequests: args.turns, maxOutputTokens: args.tokens, localToken, wire: 'chat', chatClient: 'opencode' });
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'opencode-native-'));
  let child, termination, rejectCleanup, outputLimit = false, written = 0;
  const cleanupFailure = new Promise((_, reject) => { rejectCleanup = reject; });
  cleanupFailure.catch(() => {});
  const terminate = () => { if (child?.pid && !termination) { termination = terminateProcessGroup(child); termination.catch(rejectCleanup); } };
  relay.server.once('native-attempt-limit', terminate);
  try {
    const config = path.join(temporary, 'opencode.json');
    fs.writeFileSync(config, JSON.stringify(nativeConfiguration(configuration, localToken, args.purpose)), { mode: 0o600 });
    const skills = args.purpose === 'execution' ? '/home/tester/.config/opencode/skills' : path.join(temporary, 'empty-skills');
    fs.mkdirSync(skills, { recursive: true, mode: 0o700 });
    for (const directory of ['.cache/opencode', '.local/share/opencode', '.local/state/opencode']) fs.mkdirSync(`/home/tester/${directory}`, { recursive: true, mode: 0o700 });
    await new Promise((resolve, reject) => { relay.server.once('error', reject); relay.server.listen(8091, '127.0.0.1', resolve); });
    child = spawn('/usr/bin/bwrap', sandboxArguments(config, skills, args.purpose, args.nativeArgs),
      { env: nativeEnvironment(source, args.inline), stdio: ['ignore', 'pipe', 'pipe'], detached: true });
    for (const [destination, pipe] of [[process.stdout, child.stdout], [process.stderr, child.stderr]]) pipe.on('data', bytes => {
      written += bytes.length; if (written > 15 * 1024 * 1024) { outputLimit = true; terminate(); return; } destination.write(bytes);
    });
    process.once('SIGTERM', terminate); process.once('SIGINT', terminate);
    let outcome;
    try { outcome = await Promise.race([cleanupFailure, new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', terminate); child.once('close', (code, signal) => resolve({ code, signal })); })]); }
    finally { process.removeListener('SIGTERM', terminate); process.removeListener('SIGINT', terminate); terminate(); await termination; }
    process.stdout.write('\n' + JSON.stringify({ type: 'opencode.runner_result', process_exit_code: outcome.code, signal: outcome.signal,
      provider_attempts: relay.attempts(), max_requests: args.turns, output_limit_reached: outputLimit }) + '\n');
    process.exitCode = outcome.code === 0 && !outcome.signal && !outputLimit && relay.attempts() <= args.turns ? 0 : 1;
  } finally { terminate(); relay.server.removeListener('native-attempt-limit', terminate); await relay.close(); fs.rmSync(temporary, { recursive: true, force: true }); await termination; }
}
module.exports = { nativeEnvironment, nativeConfiguration, sandboxArguments, invocation, terminateProcessGroup };
if (require.main === module) main(process.argv.slice(2)).catch(() => {
  process.stdout.write('\n' + JSON.stringify({ type: 'opencode.runner_error', diagnostic: 'Native OpenCode namespace/configuration failed; no verified completion' }) + '\n');
  process.exitCode = 1;
});
