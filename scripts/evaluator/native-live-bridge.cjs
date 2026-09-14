'use strict';
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const http = require('node:http');
const { spawn } = require('node:child_process');
async function start(executable, planFile, evidence) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'native-gateway-'));
  const env = { PATH: process.env.PATH, HOME: process.env.HOME, ANTHROPIC_API_KEY: 'sk-native-live-fixture' };
  for (const key of ['DATABASE_URL', 'TEST_DATABASE_URL']) if (process.env[key]) env[key] = process.env[key];
  const errorFd = fs.openSync(path.join(evidence, 'live-gateway.stderr'), 'w');
  const child = spawn(executable, [planFile], { env, stdio: ['pipe', 'pipe', errorFd] });
  fs.closeSync(errorFd);
  let buffer = '', pending, exited = false;
  const messages = [];
  const exit = new Promise(resolve => child.once('close', (code, signal) => { exited = true; if (pending) pending.reject(Error(`Live gateway exited ${code}/${signal}`)); resolve({ code, signal }); }));
  child.on('error', error => { if (pending) pending.reject(error); });
  child.stdout.on('data', chunk => {
    buffer += chunk.toString('utf8');
    if (buffer.length > 32 * 1024 * 1024) { child.kill('SIGTERM'); if (pending) pending.reject(Error('Live gateway output exceeded bound')); return; }
    for (let newline; (newline = buffer.indexOf('\n')) >= 0;) {
      const line = buffer.slice(0, newline); buffer = buffer.slice(newline + 1);
      try { const value = JSON.parse(line); if (pending) { const next = pending; pending = undefined; next.resolve(value); } else { if (messages.length >= 64) throw Error('Live gateway unsolicited output exceeded bound'); messages.push(value); } }
      catch { if (pending) { const next = pending; pending = undefined; next.reject(Error('Malformed live gateway output')); } child.stdin.end(); }
    }
  });
  const receive = () => {
    if (messages.length) return Promise.resolve(messages.shift());
    if (exited) return Promise.reject(Error('Live gateway exited before response'));
    return new Promise((resolve, reject) => { const timer = setTimeout(() => { pending = undefined; child.kill('SIGTERM'); reject(Error('Live gateway response deadline')); }, 35000); pending = { resolve: value => { clearTimeout(timer); resolve(value); }, reject: error => { clearTimeout(timer); reject(error); } }; });
  };
  let server;
  async function cleanup() {
    if (server) { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
    if (!exited) child.stdin.end();
    let timer;
    const finished = await Promise.race([exit, new Promise(resolve => { timer = setTimeout(() => resolve(null), 5000); })]).finally(() => clearTimeout(timer));
    if (!finished) {
      child.kill('SIGKILL');
      const killed = await Promise.race([exit, new Promise(resolve => { timer = setTimeout(() => resolve(null), 2000); })]).finally(() => clearTimeout(timer));
      if (!killed) throw Error('Live gateway process survived bounded cleanup');
    }
    fs.rmSync(directory, { recursive: true, force: true });
  }
  try {
  const ready = await receive();
  if (ready.kind !== 'ready' || !String(ready.execution_token).startsWith('spexec_') || !ready.session_id) throw Error('Live gateway did not issue an execution capability');
  let serial = Promise.resolve();
  const rpc = message => {
    const operation = serial.then(async () => { const answer = receive(); child.stdin.write(JSON.stringify(message) + '\n'); return answer; });
    serial = operation.catch(() => {}); return operation;
  };
  let firstDispatch;
  server = http.createServer(async (req, res) => {
    try {
      let size = 0; const chunks = [];
      for await (const chunk of req) { size += chunk.length; if (size > 20 * 1024 * 1024) throw Error('Native gateway RPC exceeds bound'); chunks.push(chunk); }
      const message = JSON.parse(Buffer.concat(chunks));
      if (message.kind !== 'dispatch' && message.kind !== 'authenticate') throw Error('Unsupported native fixture RPC');
      if (message.kind === 'dispatch' && !firstDispatch) firstDispatch = structuredClone(message);
      const result = await rpc(message);
      fs.appendFileSync(path.join(evidence, 'live-gateway.jsonl'), JSON.stringify({ ordinal: message.ordinal, ...result }) + '\n');
      res.setHeader('Content-Type', 'application/json'); res.end(JSON.stringify(result));
    } catch (error) { res.writeHead(502).end(JSON.stringify({ kind: 'gateway_fixture_error', diagnostic: error.message })); }
  });
  server.maxConnections = 4; server.requestTimeout = 35000; server.headersTimeout = 10000;
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(path.join(directory, 'gateway.sock'), resolve); });
  fs.chmodSync(path.join(directory, 'gateway.sock'), 0o600);
  let closed = false;
  return {
    directory, ready,
    interrupt() { child.kill('SIGTERM'); server.closeAllConnections(); server.close(); },
    async close() {
      if (closed) return; closed = true;
      try {
        const boundaryProofs = [];
        if (firstDispatch) {
          for (const boundary of ['forged-token', 'forged-session', 'output-bound', 'unknown-pricing', 'cost-bound']) {
            const probe = structuredClone(firstDispatch);
            if (['unknown-pricing', 'cost-bound'].includes(boundary)) probe.proof_probe = boundary;
            if (boundary === 'forged-token') probe.credential = 'spexec_forged';
            if (boundary === 'forged-session') probe.session_id = 'forged-native-session';
            if (boundary === 'output-bound') {
              delete probe.body.max_tokens; delete probe.body.max_output_tokens; delete probe.body.max_completion_tokens;
              probe.body[probe.path === '/v1/responses' ? 'max_output_tokens' : 'max_tokens'] = 32769;
            }
            const rejected = await rpc(probe);
            if (rejected.kind !== 'rejected' || JSON.stringify(rejected.before) !== JSON.stringify(rejected.after)) throw Error(`Live gateway ${boundary} changed reserved/settled accounting`);
            const reason = String(rejected.diagnostic || '');
            if (boundary === 'unknown-pricing' && !reason.includes('No configured pricing')) throw Error('Missing-pricing probe failed for an unrelated reason');
            if (boundary === 'cost-bound' && !reason.toLowerCase().includes('budget')) throw Error('Cost-bound probe failed for an unrelated reason');
            if (boundary === 'output-bound' && !reason.includes('text/output envelope')) throw Error('Output-bound probe failed for an unrelated reason');
            boundaryProofs.push({ boundary, rejected_without_reservation: true, reason });
          }
        }
        const finalized = await rpc({ kind: 'finalize' });
        finalized.boundaries = boundaryProofs;
        if (finalized.kind !== 'finalized') throw Error('Native execution finalization failed');
        fs.writeFileSync(path.join(evidence, 'live-gateway-final.json'), JSON.stringify(finalized, null, 2));
        child.stdin.end();
        let cleanupTimer;
        const finished = await Promise.race([exit, new Promise((_, reject) => { cleanupTimer = setTimeout(() => { child.kill('SIGKILL'); reject(Error('Live gateway process cleanup timeout')); }, 5000); })]).finally(() => clearTimeout(cleanupTimer));
        if (finished.code !== 0) throw Error('Live gateway failed during cleanup');
        return finalized;
      } finally { await cleanup(); }
    },
  };
  } catch (error) {
    try { await cleanup(); } catch (cleanupError) { throw Error(`${error.message}; ${cleanupError.message}`); }
    throw error;
  }
}
module.exports = { start };
