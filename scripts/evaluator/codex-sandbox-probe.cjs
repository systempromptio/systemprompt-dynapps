'use strict';
const fs = require('node:fs');
const { spawnSync } = require('node:child_process');
const { randomBytes } = require('node:crypto');
const { childEnvironment } = require('/opt/systemprompt/codex-runner.cjs');
if (!process.env.CODEX_EVALUATION_TOKEN) throw Error('Native probe requires real parent environment canary');
const parentMarker = 'native-parent-' + randomBytes(24).toString('hex');
process.title = parentMarker;
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
    for (const pid of fs.readdirSync('/proc').filter(name=>/^\\d+$/.test(name))) {
      for (const name of ['environ','cmdline','comm']) {
        let value; try { value=fs.readFileSync('/proc/'+pid+'/'+name,'utf8'); } catch(error) { if(['ENOENT','EACCES','EPERM','ESRCH'].includes(error.code)) continue; throw error; }
        if(name==='environ' && value.includes('CODEX_EVALUATION_TOKEN=')) throw Error('Parent upstream credentials visible in native sandbox');
        if(name==='comm' && value.trim()==='${parentMarker.slice(0,15)}') throw Error('Parent process visible in native sandbox');
        if(name==='cmdline' && value.startsWith('${parentMarker}')) throw Error('Parent process command visible in native sandbox');
      }
    }
    if(fs.readFileSync('/home/tester/.agents/skills/fixture/SKILL.md','utf8')!=='fixture skill')throw Error('skill read failed');
    denied(()=>fs.writeFileSync('/home/tester/.agents/skills/fixture/SKILL.md','tamper'));
    ${access === 'write' ? "fs.writeFileSync('/home/tester/work/probe','allowed');" : "denied(()=>fs.writeFileSync('/home/tester/work/probe','forbidden'));"}
    console.log('native ${access} policy passed');`;
  const result = spawnSync('/opt/systemprompt/codex/bin/codex', ['sandbox', '-P', 'evaluation', '-C', '/home/tester/work',
    ...settings.flatMap(setting => ['-c', setting]), '--', '/usr/local/bin/node', '-e', check],
    { env: childEnvironment(process.env, 'a'.repeat(64)), encoding: 'utf8', timeout: 15000, maxBuffer: 128 * 1024 });
  process.stdout.write(result.stdout || '');
  process.stderr.write(result.stderr || '');
  if (result.error || result.signal || result.status !== 0) throw new Error(`Native Codex ${access} sandbox failed`);
}
