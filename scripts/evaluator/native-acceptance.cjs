'use strict';
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');
const { spawn, spawnSync } = require('node:child_process');
const { createHash, randomBytes } = require('node:crypto');
const { chooseSubnet } = require('./native-subnet.cjs');
const [exporter, image = 'systemprompt-evaluator:local', selection = 'all', scenarioSelection = 'all'] = process.argv.slice(2);
const scenarios = ['candidate', 'baseline', 'mcp', 'credentials', 'forbidden', 'network', 'failure', 'cancel', 'attempt-bound', 'judge', 'suggestion'];
const selectedScenarios = scenarioSelection === 'all' ? scenarios : scenarioSelection.split(',');
if (selectedScenarios.length > scenarios.length || selectedScenarios.some(scenario => !scenarios.includes(scenario)) || new Set(selectedScenarios).size !== selectedScenarios.length) throw Error('Invalid bounded scenario selection');
const clients = ['claude-code', 'open-code', 'codex', 'hermes'];
if (!exporter || !path.isAbsolute(exporter) || !fs.existsSync(exporter) || !['all', ...clients].includes(selection) || image.startsWith('-') || image.length > 255) throw Error('Expected absolute fixture exporter, image and client|all');
const root = path.resolve(__dirname, '../..');
const report = path.join(root, '.build/native-acceptance', `${Date.now()}-${process.pid}`);
fs.mkdirSync(report, { recursive: true });
const privateHome = fs.mkdtempSync(path.join(os.tmpdir(), 'native-docker-'));
const docker = process.env.EVAL_DOCKER_BIN || '/usr/bin/docker';
if (!path.isAbsolute(docker)) throw Error('Docker executable must be absolute');
const dockerEnv = { HOME: privateHome, DOCKER_CONFIG: privateHome, PATH: process.env.PATH, ...(process.env.DOCKER_HOST ? { DOCKER_HOST: process.env.DOCKER_HOST } : {}) };
const runNonce = randomBytes(6).toString('hex');
let activeChild;
const ownedContainers = new Set(), ownedNetworks = new Set();
const result = { paid_inference: false, automated_target_enabled: false, production_gateway_metering_verified: false, cases: [] };
const invoke = (binary, args, options = {}) => {
  const child = spawnSync(binary, args, { encoding: 'utf8', timeout: 20000, maxBuffer: 1024 * 1024, ...options });
  if (child.error || child.signal || child.status !== 0) throw Error(`${path.basename(binary)} failed: ${(child.stderr || child.error?.message || '').slice(0, 2048)}`);
  return child.stdout.trim();
};
const d = (...args) => invoke(docker, args, { env: dockerEnv });
const cleanupDocker = (...args) => invoke(docker, args, { env: dockerEnv, timeout: 5000 });
for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => {
  result.status = 'interrupted'; result.signal = signal;
  let clean = true;
  const boundedDocker = (...args) => invoke(docker, args, { env: dockerEnv, timeout: 5000 });
  for (const name of ownedContainers) { try { boundedDocker('rm', '--force', name); } catch {} }
  for (const name of ownedContainers) { try { if (boundedDocker('ps', '-aq', '--filter', `name=^/${name}$`)) clean = false; } catch { clean = false; } }
  for (const name of ownedNetworks) { try { boundedDocker('network', 'rm', name); } catch { clean = false; } }
  if (activeChild) activeChild.kill('SIGTERM');
  result.interruption_cleanup_confirmed = clean;
  fs.writeFileSync(path.join(report, 'report.json'), JSON.stringify(result, null, 2));
  fs.rmSync(privateHome, { recursive: true, force: true });
  process.exit(signal === 'SIGINT' ? 130 : 143);
});
function allocateNetwork(name) {
  const ids = d('network', 'ls', '-q').split('\n').filter(Boolean);
  if (ids.length > 4096) throw Error('Docker network inventory exceeds bounded inspection');
  const occupied = [];
  for (let index = 0; index < ids.length; index += 32) {
    const networks = JSON.parse(d('network', 'inspect', ...ids.slice(index, index + 32)));
    for (const network of networks) for (const config of network.IPAM?.Config || []) if (config.Subnet && !config.Subnet.includes(':')) occupied.push(config.Subnet);
  }
  const ip = process.env.EVAL_IP_BIN || '/usr/sbin/ip';
  if (!path.isAbsolute(ip)) throw Error('IP route executable must be absolute');
  const routes = JSON.parse(invoke(ip, ['-j', '-4', 'route', 'show', 'table', 'all'], { env: dockerEnv }));
  for (const route of routes) if (route.dst) occupied.push(route.dst);
  for (let attempt = 0; attempt < 4; attempt++) {
    const subnet = chooseSubnet(occupied, process.pid + attempt);
    try { d('network', 'create', '--internal', '--subnet', subnet, '--label', 'systemprompt.native-acceptance=true', name); return subnet; }
    catch (error) { if (!/overlap|Pool overlaps/i.test(error.message)) throw error; occupied.push(subnet); }
  }
  throw Error('Concurrent network allocation prevented bounded private subnet selection');
}
const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
const audit = directory => {
  const filename = path.join(directory, 'provider.jsonl');
  return fs.existsSync(filename) ? fs.readFileSync(filename, 'utf8').trim().split('\n').filter(Boolean).map(line => JSON.parse(line)) : [];
};
function materializedEnvironment(home, skillPath) {
  const excluded = path.dirname(skillPath.replace('/home/tester/', ''));
  const entries = []; let bytes = 0;
  function walk(relative) {
    for (const name of fs.readdirSync(path.join(home, relative)).sort()) {
      const child = path.join(relative, name);
      if (child === excluded || child.startsWith(excluded + path.sep)) continue;
      const filename = path.join(home, child), stat = fs.lstatSync(filename);
      if ((!stat.isFile() && !stat.isDirectory()) || entries.length >= 256 || bytes + stat.size > 16 * 1024 * 1024) throw Error('Invalid or oversized materialized environment');
      entries.push({ path: child, mode: stat.mode & 0o777, uid: stat.uid, gid: stat.gid, ...(stat.isFile() ? { sha256: createHash('sha256').update(fs.readFileSync(filename)).digest('hex') } : {}) });
      if (stat.isFile()) bytes += stat.size;
      else walk(child);
    }
  }
  walk(''); return createHash('sha256').update(JSON.stringify(entries)).digest('hex');
}
function toolResults(value, collected = []) {
  if (Array.isArray(value)) for (const item of value) toolResults(item, collected);
  else if (value && typeof value === 'object') {
    if ((value.type === 'tool_result' && value.tool_use_id === 'call_native_fixture') || (value.role === 'tool' && value.tool_call_id === 'call_native_fixture') || (value.type === 'function_call_output' && value.call_id === 'call_native_fixture')) collected.push(value.content ?? value.output);
    for (const nested of Object.values(value)) toolResults(nested, collected);
  }
  return collected;
}
async function runCase(client, scenario) {
  const directory = path.join(report, `${client}-${scenario}`);
  invoke(exporter, ['prepare', client, directory, scenario]);
  const plan = JSON.parse(fs.readFileSync(path.join(directory, 'plan.json')));
  const evidence = path.join(directory, 'evidence'); fs.mkdirSync(evidence);
  const suffix = `${process.pid}-${runNonce}-${result.cases.length}`;
  const network = `eval-native-${suffix}`, provider = `eval-provider-${suffix}`, native = `eval-client-${suffix}`, versionContainer = `eval-version-${suffix}`;
  const entry = { client, scenario, image_id: result.image_id, status: 'failed', cleanup_confirmed: false, environment_digest: createHash('sha256').update(JSON.stringify({ argv: plan.argv, frozen: plan.frozen, configuration_digest: plan.configuration_digest })).digest('hex') };
  result.cases.push(entry);
  let outputFd, errorFd;
  try {
    entry.materialized_environment_digest = materializedEnvironment(path.join(directory, 'home'), plan.skill_path);
    const versionHome = path.join(directory, 'version-home');
    fs.cpSync(path.join(directory, 'home'), versionHome, { recursive: true, errorOnExist: true });
    entry.internal_subnet = allocateNetwork(network);
    ownedNetworks.add(network);
    for (const name of [native, provider, versionContainer]) ownedContainers.add(name);
    d('run', '-d', '--pull', 'never', '--name', provider, '--network', network, '--network-alias', 'eval-relay-native', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--pids-limit', '32', '--memory', '128m', '--cpus', '0.5', '--user', `${process.getuid()}:${process.getgid()}`, '--mount', `type=bind,source=${directory},target=/fixture,readonly`, '--mount', `type=bind,source=${evidence},target=/evidence`, '--mount', `type=bind,source=${path.join(__dirname, 'native-provider-fixture.cjs')},target=/provider.cjs,readonly`, result.image_id, '/usr/local/bin/node', '/provider.cjs');
    for (let count = 0; count < 50 && !audit(evidence).some(event => event.kind === 'ready'); count++) await delay(100);
    if (!audit(evidence).some(event => event.kind === 'ready')) throw Error('Provider fixture did not become ready');
    entry.executable_sha256 = d('exec', provider, '/usr/bin/sha256sum', plan.executable).split(' ')[0];
    if (!/^[a-f0-9]{64}$/.test(entry.executable_sha256)) throw Error('Invalid native executable digest');
    const rawVersion = d('run', '--rm', '--name', versionContainer, '--pull', 'never', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', `${process.getuid()}:${process.getgid()}`, '--tmpfs', '/tmp:rw,nosuid,nodev,size=67108864', '--mount', `type=bind,source=${versionHome},target=/home/tester`, '--env-file', path.join(directory, 'client.env'), result.image_id, plan.executable, ...plan.version_arguments);
    fs.writeFileSync(path.join(directory, 'version.stdout'), rawVersion + '\n');
    invoke(exporter, ['version', client, path.join(directory, 'version.stdout'), path.join(directory, 'version.txt')]);
    entry.client_version = fs.readFileSync(path.join(directory, 'version.txt'), 'utf8');
    if (entry.materialized_environment_digest !== materializedEnvironment(path.join(directory, 'home'), plan.skill_path)) throw Error('Version verification changed execution environment');
    outputFd = fs.openSync(path.join(directory, 'stdout.jsonl'), 'w'); errorFd = fs.openSync(path.join(directory, 'stderr.log'), 'w');
    const child = spawn(docker, ['run', '--rm', '--pull', 'never', '--name', native, '--network', network, '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--pids-limit', '128', '--memory', '2g', '--cpus', '1', '--user', `${process.getuid()}:${process.getgid()}`, '--tmpfs', '/tmp:rw,nosuid,nodev,size=268435456', '--workdir', '/home/tester/work', '--mount', `type=bind,source=${path.join(directory, 'home')},target=/home/tester`, '--env-file', path.join(directory, 'client.env'), result.image_id, ...plan.argv], { env: dockerEnv, stdio: ['ignore', outputFd, errorFd] });
    activeChild = child;
    const complete = new Promise(resolve => { child.once('error', error => resolve({ error: error.message })); child.once('exit', (code, signal) => resolve({ code, signal })); });
    let timedOut = false, cancelled = false;
    const timer = setTimeout(() => { timedOut = true; try { cleanupDocker('rm', '--force', native); } catch {} child.kill('SIGKILL'); }, 45000);
    const bound = setInterval(() => {
      if (fs.fstatSync(outputFd).size + fs.fstatSync(errorFd).size > 16 * 1024 * 1024) { timedOut = true; try { cleanupDocker('rm', '--force', native); } catch {} child.kill('SIGKILL'); }
      if (scenario === 'cancel' && audit(evidence).some(event => event.kind === 'provider')) { cancelled = true; try { cleanupDocker('rm', '--force', native); } catch {} }
    }, 200);
    const exit = await complete; activeChild = undefined; clearTimeout(timer); clearInterval(bound);
    fs.closeSync(outputFd); outputFd = undefined; fs.closeSync(errorFd); errorFd = undefined;
    invoke(exporter, ['normalize', client, path.join(directory, 'stdout.jsonl'), path.join(directory, 'normalized.json')]);
    const normalized = JSON.parse(fs.readFileSync(path.join(directory, 'normalized.json')));
    const events = audit(evidence), requests = events.filter(event => event.kind === 'provider');
    entry.exit = exit; entry.normalized = normalized; entry.provider_attempts = requests.length; entry.cancelled = cancelled; entry.timed_out = timedOut;
    entry.route_rejections = events.filter(event => ['unauthorized', 'forbidden_route'].includes(event.kind)).slice(0, 8);
    if (!requests.length) {
      const reason = String(normalized.text || normalized.diagnostic || fs.readFileSync(path.join(directory, 'stderr.log'), 'utf8')).slice(0, 1024);
      throw Error(`No accepted provider request; exit=${JSON.stringify(exit)}; routes=${JSON.stringify(entry.route_rejections).slice(0, 512)}; native=${reason}`);
    }
    if (events.some(event => event.kind === 'unauthorized' || event.kind === 'forbidden_route')) throw Error('Native request escaped authenticated route contract');
    if (timedOut) throw Error('Native execution exceeded timeout or artifact bound');
    if (scenario === 'cancel') { if (!cancelled || normalized.completion === 'completed') throw Error('Cancelled native execution produced success'); }
    else if (scenario === 'failure') { if (normalized.completion === 'completed') throw Error('Failed provider produced completed evidence'); }
    else if (scenario === 'attempt-bound') { if (requests.length > 1) throw Error('Native attempt bound exceeded'); }
    else {
      if (normalized.completion !== 'completed') throw Error(`Native completion missing: ${normalized.diagnostic || normalized.completion}`);
      if (['forbidden', 'network', 'judge', 'suggestion'].includes(scenario) && fs.existsSync(path.join(directory, 'home/work/forbidden-marker'))) throw Error('Forbidden native tool modified workspace');
      const returned = toolResults(requests.slice(1).map(event => event.body));
      entry.native_tool_results = returned.length;
      if (!returned.length) throw Error('No native result for the requested boundary tool was returned');
      const subsequent = JSON.stringify(returned);
      if (scenario === 'candidate' && !subsequent.includes('NATIVE_SKILL_SENTINEL')) throw Error('Native skill bytes were not read back through tool evidence');
      if (scenario === 'mcp' && !events.some(event => event.kind === 'mcp' && event.method === 'tools/call')) throw Error('Native MCP tool never executed');
      if (scenario === 'baseline' && subsequent.includes('NATIVE_SKILL_SENTINEL')) throw Error('Baseline unexpectedly read candidate skill');
      if (scenario === 'credentials' && subsequent.includes('spexec_native_fixture')) throw Error('Native tool leaked scoped upstream credential');
    }
    entry.status = 'passed';
  } catch (error) { entry.error = error.message; }
  finally {
    if (outputFd !== undefined) fs.closeSync(outputFd); if (errorFd !== undefined) fs.closeSync(errorFd);
    for (const container of [native, provider, versionContainer]) { try { cleanupDocker('rm', '--force', container); } catch {} }
    try { const remaining = [native, provider, versionContainer].flatMap(name => cleanupDocker('ps', '-aq', '--filter', `name=^/${name}$`).split('\n').filter(Boolean)); if (remaining.length) throw Error('Container cleanup incomplete'); cleanupDocker('network', 'rm', network); ownedNetworks.delete(network); for (const name of [native, provider, versionContainer]) ownedContainers.delete(name); entry.cleanup_confirmed = true; } catch (error) { entry.cleanup_error = error.message; entry.status = 'failed'; }
    fs.writeFileSync(path.join(report, 'report.json'), JSON.stringify(result, null, 2));
    process.stdout.write(`${client} ${scenario}: ${entry.status}${entry.error ? ` (${entry.error})` : ''}\n`);
  }
}
(async () => {
  try {
    result.image_id = d('image', 'inspect', '--format', '{{.Id}}', image);
    if (!/^sha256:[a-f0-9]{64}$/.test(result.image_id)) throw Error('Invalid immutable image identity');
    for (const client of selection === 'all' ? clients : [selection]) {
      for (const scenario of selectedScenarios) { await runCase(client, scenario); if (!result.cases.at(-1).cleanup_confirmed) throw Error('Native cleanup failed; further cases stopped'); }
      const pair = result.cases.filter(entry => entry.client === client && ['candidate', 'baseline'].includes(entry.scenario));
      if (selectedScenarios.includes('candidate') && selectedScenarios.includes('baseline') && (pair.length !== 2 || pair[0].environment_digest !== pair[1].environment_digest || pair[0].materialized_environment_digest !== pair[1].materialized_environment_digest || pair[0].image_id !== pair[1].image_id || pair[0].executable_sha256 !== pair[1].executable_sha256)) { result.environment_parity_error = client; }
    }
    result.status = result.cases.every(entry => entry.status === 'passed') && !result.environment_parity_error ? 'passed' : 'failed';
    fs.writeFileSync(path.join(report, 'report.json'), JSON.stringify(result, null, 2));
    process.exitCode = result.status === 'passed' ? 0 : 1;
  } finally { fs.rmSync(privateHome, { recursive: true, force: true }); process.stdout.write(`Native fixture report: ${report}/report.json\n`); }
})().catch(error => { process.stderr.write(error.message + '\n'); process.exitCode = 1; });
