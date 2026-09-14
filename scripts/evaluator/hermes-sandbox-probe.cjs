'use strict';
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const { sandboxArguments, nativeEnvironment } = require('/opt/systemprompt/hermes-runner.cjs');
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'hermes-native-probe-'));
try {
  const config = path.join(directory, 'config'); const skills = path.join(directory, 'skills'); const evidence = path.join(directory, 'evidence');
  fs.writeFileSync(config, '{}'); fs.mkdirSync(skills); fs.mkdirSync(evidence); fs.writeFileSync(path.join(skills, 'SKILL.md'), 'fixture');
  for (const purpose of ['execution', 'review']) {
    const args = sandboxArguments(config, evidence, skills, purpose, []);
    args[args.indexOf('/opt/hermes-source/.venv/bin/hermes')] = '/usr/bin/python3';
    args.push('-c', `import os,pathlib
assert 'HERMES_EVALUATION_TOKEN' not in os.environ
assert not pathlib.Path('/proc/${process.pid}/environ').exists()
def denied(path):
 try:
  pathlib.Path(path).write_text('tamper')
 except OSError:
  return
 raise AssertionError('forbidden write succeeded: '+path)
assert pathlib.Path('/home/tester/.hermes/skills/SKILL.md').read_text()=='fixture'
denied('/home/tester/.hermes/skills/SKILL.md')
denied('/home/tester/.hermes/config.yaml')
${purpose === 'execution' ? "pathlib.Path('/home/tester/work/canary').write_text('allowed')" : "denied('/home/tester/work/canary')"}
print('native ${purpose} namespace passed')`);
    const result = spawnSync('/usr/bin/bwrap', args, { env: nativeEnvironment({ HERMES_EVALUATION_TOKEN: 'parent-only', TZ: 'Asia/Tokyo' }, 'local-canary'),
      encoding: 'utf8', timeout: 15000, maxBuffer: 128 * 1024 });
    process.stdout.write(result.stdout || ''); process.stderr.write(result.stderr || '');
    if (result.error || result.signal || result.status !== 0) throw new Error(`Hermes ${purpose} namespace unavailable or isolation failed`);
  }
} finally { fs.rmSync(directory, { recursive: true, force: true }); }
