'use strict';
const fs = require('node:fs');
const { spawnSync } = require('node:child_process');
fs.mkdirSync('/home/tester/.codex', { recursive: true });
fs.mkdirSync('/home/tester/.agents/skills/fixture', { recursive: true });
fs.writeFileSync('/home/tester/.codex/canary', 'private-credential-canary');
fs.writeFileSync('/home/tester/.agents/skills/fixture/SKILL.md', 'fixture skill');
for (const access of ['write', 'read']) {
  const settings = [
    `permissions.evaluation.filesystem={":minimal"="read","/proc"="deny","/opt/systemprompt/codex"="read","/home/tester/.codex"="deny","/home/tester/.agents/skills"="read","/home/tester/work"="${access}"}`,
    'permissions.evaluation.network.enabled=false',
  ];
  const check = `const fs=require('fs');
    function denied(operation) { try { operation(); } catch (error) { if (['EACCES','EPERM','ENOENT','EROFS'].includes(error.code)) return; throw error; } throw Error('Forbidden access succeeded'); }
    denied(()=>fs.readFileSync('/home/tester/.codex/canary'));
    denied(()=>fs.readFileSync('/proc/self/environ'));
    if(fs.readFileSync('/home/tester/.agents/skills/fixture/SKILL.md','utf8')!=='fixture skill')throw Error('skill read failed');
    denied(()=>fs.writeFileSync('/home/tester/.agents/skills/fixture/SKILL.md','tamper'));
    ${access === 'write' ? "fs.writeFileSync('/home/tester/work/probe','allowed');" : "denied(()=>fs.writeFileSync('/home/tester/work/probe','forbidden'));"}
    console.log('native ${access} policy passed');`;
  const result = spawnSync('/opt/systemprompt/codex/bin/codex', ['sandbox', '-P', 'evaluation', '-C', '/home/tester/work',
    ...settings.flatMap(setting => ['-c', setting]), '--', '/usr/local/bin/node', '-e', check],
    { encoding: 'utf8', timeout: 15000, maxBuffer: 128 * 1024 });
  process.stdout.write(result.stdout || '');
  process.stderr.write(result.stderr || '');
  if (result.error || result.signal || result.status !== 0) throw new Error(`Native Codex ${access} sandbox failed`);
}
