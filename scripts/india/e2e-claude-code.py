#!/usr/bin/env python3
"""Clean Docker India skill evidence. Authentication never falls back to local/staging."""
import argparse
import base64
from datetime import datetime, timezone
from decimal import Decimal
import fcntl
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import tempfile
import time
import urllib.request
import uuid

from claude_cases import FIXTURES, cases, prompt, write_fixtures
from claude_evidence import Budget, artifact_checks, installation, redact, sha, summarize

ROOT = Path(__file__).resolve().parents[2]
GATEWAY = 'https://sp-dev.astound.digital'
MODEL = 'claude-haiku-4-5-20251001'
IMAGE = 'astound-india-clean:2.1.266'
PRIVATE = ROOT/'.build/india-private'


def command(argv, input=None, timeout=60, check=True):
    result = subprocess.run(argv, input=input, text=True, capture_output=True, timeout=timeout)
    if check and result.returncode:
        # Do not include argv/output: installer arguments and errors can carry credentials.
        raise RuntimeError(f'{Path(argv[0]).name} exited {result.returncode}')
    return result


def request(path, token=None, data=None):
    headers = {'Content-Type': 'application/json'}
    if token:
        headers['Authorization'] = 'Bearer '+token
    req = urllib.request.Request(GATEWAY+path, headers=headers,
                                 data=None if data is None else json.dumps(data).encode())
    with urllib.request.urlopen(req, timeout=30) as response:
        return json.load(response)


def save(path, value, secrets=()):
    path.parent.mkdir(parents=True, exist_ok=True)
    body = value if isinstance(value, str) else json.dumps(value, indent=2, ensure_ascii=False)+'\n'
    path.write_text(redact(body, secrets))


def cx(container, argv, input=None, timeout=60, check=True):
    return command(['docker', 'exec', '-i', '--user', 'tester', container, *argv], input, timeout, check)


def py(container, script):
    return json.loads(cx(container, ['python3', '-c', script]).stdout)


def create_client(name, run_id, image=IMAGE):
    command(['docker', 'run', '-d', '--name', name, '--label', 'india.skills.run='+run_id,
             '--cap-drop', 'NET_RAW', '--memory', '2g', '--cpus', '2',
             '-e', 'ASTOUND_BRIDGE_GATEWAY_URL='+GATEWAY, image])
    inspected = json.loads(command(['docker', 'inspect', name]).stdout)[0]
    if inspected['Mounts']:
        raise RuntimeError('Clean client unexpectedly has mounts')
    empty = py(name, "import json,pathlib; h=pathlib.Path.home(); print(json.dumps({p: (h/p).exists() for p in ['.claude','.claude.json','.config/astound']}))")
    if any(empty.values()):
        raise RuntimeError('Clean client has inherited auth/configuration')
    return {'image_id': inspected['Image'], 'mounts': inspected['Mounts'], 'initial_config_present': empty,
            'claude_version': cx(name, ['claude', '--version']).stdout.strip()}


def remove_client(name, run_id):
    got = command(['docker', 'inspect', name], check=False)
    if got.returncode:
        return
    if json.loads(got.stdout)[0]['Config']['Labels'].get('india.skills.run') != run_id:
        raise RuntimeError('Refusing to remove a container from another run')
    command(['docker', 'rm', '--force', name])


def enroll(container, pat, installer=None):
    if installer:
        command(['docker','cp',str(installer.resolve()),container+':/tmp/india-install.sh'])
    script = '''import os,pathlib,subprocess,sys
h=pathlib.Path.home(); pat=sys.stdin.read().strip()
url='https://sp-dev.astound.digital'
if os.path.exists('/tmp/india-install.sh'):
 (h/'install.sh').write_bytes(pathlib.Path('/tmp/india-install.sh').read_bytes())
else:
 subprocess.run(['curl','-fsS','--max-time','30',url+'/files/downloads/install.sh','-o',str(h/'install.sh')],check=True)
with (h/'enrollment.log').open('w') as log:
 r=subprocess.run(['sh',str(h/'install.sh'),'--download-base',url+'/files/downloads','--gateway',url,'--pat',pat,'--host','claude-code'],stdout=log,stderr=subprocess.STDOUT)
sys.exit(r.returncode)
'''
    cx(container, ['python3', '-c', script], input=pat, timeout=240)


DISK_SCRIPT = r'''import pathlib,json,hashlib
h=pathlib.Path.home(); root=h/'.claude/plugins'; market=root/'marketplaces'
def read(p): return json.loads(p.read_text()) if p.exists() else {}
registry=read(root/'installed_plugins.json').get('plugins',{})
settings=read(h/'.claude/settings.json')
bodies=[]; files={}; servers=set()
for p in market.rglob('*'):
 if p.is_file():
  files[str(p.relative_to(market))]=hashlib.sha256(p.read_bytes()).hexdigest()
  if p.name=='SKILL.md': bodies.append({'name':p.parent.name,'path':str(p.relative_to(market)),'body':p.read_text()})
  if p.name=='.mcp.json': servers.update(read(p).get('mcpServers',{}))
cache={}
for key, entries in registry.items():
 for entry in entries:
  path=pathlib.Path(entry['installPath'])
  cache[key]= {str(p.relative_to(path)):hashlib.sha256(p.read_bytes()).hexdigest() for p in path.rglob('*') if p.is_file()}
print(json.dumps({'marketplaces':sorted(p.name for p in market.iterdir() if p.is_dir()) if market.exists() else [],'registry':list(registry),'enabled':[k for k,v in settings.get('enabledPlugins',{}).items() if v],'skills':[b['name'] for b in bodies],'bodies':bodies,'files':files,'mcp_names':sorted(servers),'active_cache':cache}))
'''


def inspect_client(container, manifest, inventory):
    disk = None
    for _ in range(20):
        disk = py(container, DISK_SCRIPT)
        if disk['registry']:
            break
        time.sleep(1)
    result = installation(manifest, disk, inventory)
    result['bridge_version'] = cx(container, ['bash', '-lc', 'astound-bridge --version']).stdout.strip()
    result['bridge_sha256'] = cx(container, ['bash', '-lc', 'sha256sum "$HOME/.local/bin/astound-bridge"']).stdout.split()[0]
    # The active cache is what Claude loads, not just the marketplace mirror.
    mismatched = []
    for key, files in disk['active_cache'].items():
        plugin, market = key.split('@', 1)
        prefix = f'{market}/plugins/{plugin}/'
        for path, digest in disk['files'].items():
            if path.startswith(prefix) and files.get(path[len(prefix):]) != digest:
                mismatched.append(key+':'+path[len(prefix):])
    result['checks']['active_cache_matches_mirror'] = {'pass': not mismatched, 'mismatches': mismatched}
    if mismatched:
        result['status'] = 'FAIL'
    return result, disk


def copy_fixtures(container):
    with tempfile.TemporaryDirectory(prefix='india-fixtures-') as temp:
        write_fixtures(temp)
        command(['docker', 'cp', temp+'/.', container+':/home/tester/work/'])
    command(['docker', 'exec', '--user', 'root', container, 'chown', '-R', 'tester:tester', '/home/tester/work'])


def collect_artifacts(container):
    return py(container, '''import json,pathlib
root=pathlib.Path('/home/tester/work'); out={}
for folder in ['output','.astound/ba','force-app','stories','docs']:
 for p in (root/folder).rglob('*'):
  if p.is_file() and not p.is_symlink() and p.stat().st_size<300000:
   try: out[str(p.relative_to(root))]=p.read_text()
   except UnicodeError: pass
print(json.dumps(out))''')


def bridge_session(container):
    command(['docker','cp',str(ROOT/'scripts/india/mcp_probe.py'),container+':/tmp/india-probe.py'])
    result = cx(container,['python3','/tmp/india-probe.py','systemprompt','systemprompt'],
                input=json.dumps({'command':'--version'}))
    wire = json.loads(result.stdout)
    meta = wire.get('result',{}).get('_meta',{}).get('io.systemprompt/execution',{})
    if not meta.get('session_id') or not meta.get('user_id'):
        raise RuntimeError('Cannot identify the authenticated proxy session for audit correlation')
    return {'session_id':meta['session_id'],'user_id':meta['user_id'],'probe_execution_id':meta['mcp_execution_id']}


def audit(uid, session, since, bridge_sid=None):
    db = os.environ.get('INDIA_AUDIT_DATABASE_URL')
    audit_container = os.environ.get('INDIA_AUDIT_CONTAINER')
    if not db and not audit_container:
        return {'status': 'UNAVAILABLE', 'reason': 'No explicit live audit DB configured; no localhost/staging fallback'}
    sql = '''WITH requests AS (
      SELECT * FROM ai_requests WHERE user_id=:'uid' AND
        (client_session_id=:'sid' OR session_id=:'sid' OR session_id=:'bridge_sid')
      AND created_at >= :'since'::timestamptz
    ), executions AS (
      SELECT DISTINCT e.mcp_execution_id,e.tool_name,e.server_name,e.status,e.trace_id,e.execution_time_ms
      FROM requests r JOIN ai_request_tool_calls c ON c.request_id=r.id
      JOIN mcp_tool_executions e ON e.mcp_execution_id=c.mcp_execution_id
        OR (e.ai_tool_call_id IS NOT NULL AND e.ai_tool_call_id=c.ai_tool_call_id)
      WHERE e.user_id=:'uid'
    ) SELECT json_build_object('requests',(SELECT COALESCE(json_agg(json_build_object(
      'id',id,'request_id',request_id,'trace_id',trace_id,'session_id',session_id,
      'client_session_id',client_session_id,'model',model,'requested_model',requested_model,
      'cost_microdollars',cost_microdollars,'input_tokens',input_tokens,'output_tokens',output_tokens,
      'cache_read_tokens',cache_read_tokens,'cache_creation_tokens',cache_creation_tokens,
      'latency_ms',latency_ms,'status',status)), '[]'::json) FROM requests),
      'mcp_executions',(SELECT COALESCE(json_agg(row_to_json(e)), '[]'::json) FROM executions e)) AS evidence;'''
    if audit_container:
        info = json.loads(command(['docker','inspect',audit_container]).stdout)[0]
        if 'ASTOUND_BRIDGE_GATEWAY_URL='+GATEWAY not in info['Config']['Env']:
            raise RuntimeError('Audit container is not scoped to the live Astound endpoint')
        command(['docker','cp',str(ROOT/'scripts/india/mcp_probe.py'),audit_container+':/tmp/india-audit.py'])
        for name,value in [('uid',uid),('sid',session),('since',since),('bridge_sid',bridge_sid or session)]:
            sql = sql.replace(":'"+name+"'", "'"+value.replace("'","''")+"'")
        response = cx(audit_container,['python3','/tmp/india-audit.py','systemprompt','systemprompt'],
                      input=json.dumps({'command':'infra db query '+shlex.quote(sql)+' --json'}))
        wire = json.loads(response.stdout)
        payload = wire.get('result',{})
        if payload.get('isError'):
            return {'status':'UNAVAILABLE','reason':'Governed admin audit query failed'}
        data = payload['structuredContent']['items'][0]['evidence']
        if isinstance(data,str):
            data = json.loads(data)
        data['status'] = 'CORRELATED' if data['requests'] else 'NO CORRELATED ROWS'
        data['audit_transport'] = 'governed systemprompt MCP on live endpoint'
        data['audit_execution'] = payload.get('_meta',{}).get('io.systemprompt/execution',{})
        return data
    env = dict(os.environ, PGDATABASE=db, PGOPTIONS='-c default_transaction_read_only=on')
    result = subprocess.run(['psql','-X','-tA','-v','ON_ERROR_STOP=1','-v','uid='+uid,'-v','sid='+session,'-v','since='+since,'-v','bridge_sid='+(bridge_sid or session)],
                            env=env, input=sql, capture_output=True, text=True, timeout=20)
    if result.returncode:
        return {'status': 'UNAVAILABLE', 'reason': 'Live audit query failed; see connection/schema configuration'}
    data = json.loads(result.stdout)
    data['status'] = 'CORRELATED' if data['requests'] else 'NO CORRELATED ROWS'
    return data


def prepare(out, run_id):
    inventory = json.loads((ROOT/'docs/india-skills-inventory.json').read_text())['skills']
    expected = {s['id'] for s in inventory}
    if {c['id'] for c in cases()} != expected:
        raise RuntimeError('Scenario inventory differs from the 34 imported skills')
    reviews = []
    for skill in inventory:
        body = (ROOT/skill['destination']).read_text()
        refs = re.findall(r'\.cursor/[^`\s)]+', body)
        reviews.append({'id': skill['id'], 'sha256': sha(body.encode()),
                        'inventory_hash_matches': sha(body.encode()) == skill['destination_sha256'],
                        'obsolete_cursor_references': refs,
                        'unbundled_agent_reference': 'Apex_Architect' in body})
    save(out/'source-review.json', reviews)
    write_fixtures(out/'fixtures')
    rows = []
    for case in cases():
        save(out/'skills'/case['id']/'starting-prompt.md', prompt(case, run_id))
        save(out/'skills'/case['id']/'acceptance.json', case)
        rows.append({'id': case['id'], 'mode': case['mode'], 'status': 'NOT RUN', 'attempts': []})
    return {'run_id': run_id, 'gateway': GATEWAY, 'model': MODEL, 'budget_usd': '5', 'spent_usd': '0',
            'unknown_cost': False, 'rows': rows, 'ordinary_developer_access': 'UNVERIFIED: admin only',
            'source_commit': command(['git','rev-parse','HEAD']).stdout.strip()}


def public_probe(out, run_id):
    started = time.monotonic()
    data = {'models': request('/v1/models'), 'gateway': GATEWAY}
    with urllib.request.urlopen(GATEWAY+'/files/downloads/install.sh', timeout=30) as r:
        installer = r.read()
    data['installer_sha256'] = sha(installer)
    match = re.search(r'^DEFAULT_DOWNLOAD_BASE="([^"]+)"', installer.decode(), re.M)
    data['installer_default_download_base'] = match[1] if match else None
    data['default_targets_requested_endpoint'] = data['installer_default_download_base'] == GATEWAY+'/files/downloads'
    name = 'india-public-'+uuid.uuid4().hex[:10]
    try:
        data['container'] = create_client(name, run_id)
        # Execute the real published installer up to its authentication gate.
        # No token, no fabricated identity and no model calls are involved.
        cx(name, ['curl','-fsS','--max-time','30',GATEWAY+'/files/downloads/install.sh','-o','/home/tester/install.sh'])
        result = cx(name, ['sh','/home/tester/install.sh','--download-base',GATEWAY+'/files/downloads',
                           '--gateway',GATEWAY,'--host','claude-code'], timeout=240, check=False)
        save(out/'onboarding-public.txt', result.stdout+result.stderr)
        data['installer_exit_without_auth'] = result.returncode
        version = cx(name, ['bash','-lc','astound-bridge --version'], check=False)
        data['downloaded_bridge_version'] = version.stdout.strip()
        digest = cx(name, ['sha256sum','/home/tester/.local/bin/astound-bridge'], check=False)
        data['downloaded_bridge_sha256'] = digest.stdout.split()[0] if digest.stdout else None
        data['claude_version_after_installer'] = cx(name, ['claude','--version']).stdout.strip()
        data['auth_status'] = 'BLOCKED: live admin login required'
    finally:
        remove_client(name, run_id)
        data['wall_seconds'] = round(time.monotonic()-started,3)
        save(out/'onboarding-public.json', data)
    return data


def report(out, state):
    save(out/'results.json', state)
    lines = ['# India skills test matrix', '', f'Endpoint: {GATEWAY}. Model: {MODEL}.', '',
             '| Skill | Coverage | Outcome | Cost USD | Seconds |', '|---|---|---|---:|---:|']
    for row in state['rows']:
        attempts = row['attempts']
        unknown = any(a.get('cost_usd') is None for a in attempts)
        cost = 'unknown' if unknown else (str(sum(Decimal(str(a['cost_usd'])) for a in attempts)) if attempts else '—')
        duration = str(round(sum(a.get('wall_seconds',0) for a in attempts),2)) if attempts else '—'
        lines.append(f'| [{row["id"]}](skills/{row["id"]}/starting-prompt.md) | {row["mode"]} | {row["status"]} | {cost} | {duration} |')
    lines += ['', f'Known model spend: US${state["spent_usd"]}; cap: US$5. Unknown accounting: {state["unknown_cost"]}.',
              '', 'Admin-only runs do not prove ordinary India-developer assignment or isolation. Salesforce/GitHub live execution is excluded.',
              'REVIEW REQUIRED means execution evidence exists but the semantic rubric has not been signed off. It is not a pass.']
    save(out/'matrix.md', '\n'.join(lines)+'\n')


def apply_review(out, state, path):
    """Require per-criterion evidence and server proof before publishing a pass."""
    reviews = json.loads(path.read_text())
    catalog = {c['id']:c for c in cases()}
    for skill, review in reviews.items():
        row = next(r for r in state['rows'] if r['id'] == skill)
        number = review['attempt']
        if number != len(row['attempts']) or number < 1:
            raise RuntimeError('Review must identify the latest preserved attempt')
        if row['status'] != 'REVIEW REQUIRED':
            raise RuntimeError('Cannot turn an execution failure or unrun case into a pass')
        folder = out/'skills'/skill/f'attempt-{number:02}'
        assertions = review['checks']
        if [a['criterion'] for a in assertions] != catalog[skill]['rubric']:
            raise RuntimeError('Review must cover the exact complete scenario rubric in order')
        for item in assertions:
            if not item.get('evidence'):
                raise RuntimeError('Each reviewed assertion needs an evidence path')
            for evidence in item['evidence']:
                target = (folder/evidence).resolve()
                if not target.is_relative_to(folder.resolve()) or not target.is_file():
                    raise RuntimeError('Review evidence must be a file in the reviewed attempt')
        if not all(a.get('passed') is True for a in assertions):
            row['status'] = 'FAIL—RUBRIC'
        elif catalog[skill]['mode'].startswith('atlassian'):
            proof = json.loads((folder/'audit.json').read_text())
            if proof.get('status') != 'CORRELATED' or not any(e['status']=='success' and e['server_name']=='atlassian' for e in proof.get('mcp_executions',[])):
                raise RuntimeError('Live E2E pass requires correlated server MCP execution evidence')
            row['status'] = 'LIVE E2E PASS'
        else:
            row['status'] = 'LOCAL FUNCTIONAL PASS'
        if row['attempts'][-1]['content_origin'] == 'local-patch':
            row['status'] += ' (LOCAL PATCH)'
        save(folder/'review.json',review)


def model_turn(container, case, text, session, allowance, resume=False, writes=False):
    denied = 'Agent,Task,Bash,WebSearch,WebFetch,mcp__github*,mcp__salesforce*'
    allowed = 'Read,Write,Edit,Glob,Grep,Skill'
    if case['mode'] == 'local':
        denied += ',mcp__*'
    else:
        allowed += ',mcp__*atlassian*__discover,mcp__*atlassian*__executeRead'
        if writes:
            allowed += ',mcp__*atlassian*__executeWrite'
        else:
            denied += ',mcp__*__executeWrite'
    args = ['claude','-p','--model',MODEL,'--output-format','stream-json','--verbose',
            '--permission-mode','default',
            '--max-budget-usd',str(allowance),'--max-turns','12',
            '--tools','Read,Write,Edit,Glob,Grep,Skill',
            '--allowedTools',allowed, '--disallowedTools',denied,
            '--resume' if resume else '--session-id',session]
    shell = ('export CLAUDE_CODE_MAX_OUTPUT_TOKENS=1536 ANTHROPIC_DEFAULT_HAIKU_MODEL='+MODEL+
             ' ANTHROPIC_DEFAULT_SONNET_MODEL='+MODEL+' ANTHROPIC_DEFAULT_OPUS_MODEL='+MODEL+
             '; cd "$HOME/work"; exec '+shlex.join(args))
    return cx(container, ['bash','-lc',shell], input=text, timeout=300, check=False)


def local_overlay(container, skill, disk):
    """Overlay only after preserving live installation proof, never as live evidence."""
    source = ROOT/'services/skills'/skill
    target_name = skill.replace('_','-')
    destinations = []
    for body in disk['bodies']:
        if body['name'] == target_name:
            destinations.append('/home/tester/.claude/plugins/marketplaces/'+str(Path(body['path']).parent))
    for key in disk['active_cache']:
        # Read the registered path inside the container rather than assuming a
        # cache version layout. Keys came from the installer, not from a prompt.
        paths = py(container, 'import json,pathlib; r=json.loads((pathlib.Path.home()/".claude/plugins/installed_plugins.json").read_text()); print(json.dumps([e["installPath"] for e in r["plugins"]['+repr(key)+']]))')
        for path in paths:
            if not path.startswith('/home/tester/.claude/plugins/'):
                raise RuntimeError('Unexpected active plugin cache path')
            if any(p.endswith('/'+target_name+'/SKILL.md') for p in disk['active_cache'][key]):
                destinations.append(path+'/skills/'+target_name)
    if not destinations:
        raise RuntimeError('Cannot overlay a skill that did not install from the live endpoint')
    for dest in destinations:
        command(['docker','cp',str(source/'SKILL.md'),container+':'+dest+'/SKILL.md'])
        if (source/'references').exists():
            command(['docker','cp',str(source/'references'),container+':'+dest+'/'])
    return {'origin':'LOCAL PATCH; NOT LIVE DEPLOYMENT', 'skill_sha256':sha((source/'SKILL.md').read_bytes()),
            'destinations':destinations}


def run(args, out, state):
    inventory = json.loads((ROOT/'docs/india-skills-inventory.json').read_text())['skills']
    if not args.pat_file and not getattr(args, 'code_file', None):
        for row in state['rows']:
            if not row['attempts']:
                row['status'] = 'BLOCKED—ADMIN LOGIN'
        state['blocking_reason'] = 'Provide a private --pat-file for your live admin account; localhost fixtures are not accepted'
        return
    if getattr(args, 'code_file', None):
        code = args.code_file.read_text().strip()
        issued = request('/v1/auth/bridge/session-pat', data={'code':code,'device_name':'India skills '+state['run_id']})
        pat = issued['pat']
        # Keep the exchanged credential private for subsequent scenario turns.
        credential_path = PRIVATE/(state['run_id']+'.pat')
        credential_path.write_text(pat+'\n')
        credential_path.chmod(0o600)
    else:
        pat = args.pat_file.read_text().strip()
    exchanged = request('/v1/auth/bridge/pat', pat, {})
    token = exchanged['token']
    claims = json.loads(base64.urlsafe_b64decode(token.split('.')[1]+'==='))
    wire = request('/v1/bridge/manifest', token)
    manifest = wire['payload']
    if isinstance(manifest,str):
        manifest = json.loads(manifest)
    identity = manifest.get('user', {})
    if identity.get('id') != claims['sub'] or 'admin' not in identity.get('roles', []):
        raise RuntimeError('Server manifest does not identify this credential as an admin')
    if str(identity.get('email','')).endswith(('@e2e.local','@example.invalid')):
        raise RuntimeError('Fixture identity is not the requested real admin')
    if state.get('identity') and state['identity'].get('id') != identity['id']:
        raise RuntimeError('Cannot mix identities in one run ledger')
    state['identity'] = identity
    state.pop('blocking_reason',None)
    missing = {s['id'] for s in inventory} - {s['id'] for s in manifest['skills']}
    save(out/'current-access.json',{'identity':identity,'marketplaces':manifest['marketplaces'],
                                 'skill_ids':[s['id'] for s in manifest['skills']],
                                 'missing_india_skills':sorted(missing)})
    if missing:
        for row in state['rows']:
            if row['id'] in missing:
                row['status'] = 'BLOCKED—INDIA ACCESS'
        state['blocking_reason'] = 'Live admin login works; '+str(len(missing))+' India skills are not granted to this account'
        return
    if not state.get('image_id'):
        state['image_id'] = command(['docker','image','inspect','--format','{{.Id}}',IMAGE]).stdout.strip()
    save(PRIVATE/(state['run_id']+'-manifest.json'), manifest, [pat,token])
    budget = Budget(state['budget_usd'], state['spent_usd'], state['unknown_cost'])
    catalog = {c['id']: c for c in cases()}
    for row in state['rows']:
        if args.only and row['id'] not in args.only.split(','):
            continue
        if row['attempts'] and not args.retry and not args.followup_file:
            continue
        allowance = budget.allowance('0.20' if row['mode'].startswith('atlassian') else '0.10')
        if allowance < Decimal('0.02'):
            row['status'] = 'NOT RUN—ACCOUNTING' if budget.unknown else 'NOT RUN—BUDGET'
            continue
        case = catalog[row['id']]
        if args.local_skill_files and not row['attempts']:
            raise RuntimeError('Run the live-installed scenario first; local retests must preserve that attempt')
        attempt_id = len(row['attempts'])+1
        folder = out/'skills'/row['id']/f'attempt-{attempt_id:02}'
        resume = bool(args.followup_file)
        prior = row['attempts'][-1] if row['attempts'] else None
        if resume and (not prior or not prior.get('client_retained')):
            raise RuntimeError('Follow-up needs a retained client from the same scenario')
        name = prior['container'] if resume else 'india-'+uuid.uuid4().hex[:12]
        attempt = {'session_id': prior['session_id'] if resume else str(uuid.uuid4()),
                   'container': name, 'cost_usd': None, 'phase': 'followup' if resume else 'initial',
                   'content_origin': 'local-patch' if args.local_skill_files else 'live-installer'}
        attempt['bootstrap_origin'] = 'local-installer-fix' if args.installer_file else 'live-installer'
        dispatched = False
        print(row['id']+': enrolling fresh Docker client', flush=True)
        try:
            if resume:
                inspected = json.loads(command(['docker','inspect',name]).stdout)[0]
                if inspected['Mounts'] or inspected['Config']['Labels'].get('india.skills.run') != state['run_id']:
                    raise RuntimeError('Retained client provenance does not match this run')
                attempt['container_provenance'] = prior['container_provenance']
            else:
                attempt['container_provenance'] = create_client(name, state['run_id'], state['image_id'])
                enroll(name, pat, args.installer_file)
            installed, disk = inspect_client(name, manifest, inventory)
            save(folder/'installation.json', installed)
            save(folder/'installed-files.json', {k:v for k,v in disk.items() if k != 'bodies'}, [pat,token])
            for body in disk['bodies']:
                if body['name'] == row['id'].replace('_','-'):
                    save(folder/'installed-SKILL.md', body['body'])
            if installed['status'] != 'PASS':
                row['status'] = 'FAIL—INSTALLATION'
                attempt['cost_usd'] = 0
                continue
            if args.local_skill_files:
                save(folder/'local-overlay.json', local_overlay(name,row['id'],disk))
            if not resume:
                copy_fixtures(name)
            text = args.followup_file.read_text() if resume else prompt(case,state['run_id'])
            if resume:
                text += ('\nContinue the same installed skill and scenario. Preserve the run marker '+state['run_id']+
                         '. Only approved run-labelled demo items may be changed. Read back all changes through MCP and '
                         'save the actual item URLs and cleanup status to output/remote-items.json. '
                         'Never send notifications or mutate unrelated records, project settings, sprints or release versions.')
            save(folder/'prompt.md',text)
            attempt['bridge_session'] = bridge_session(name)
            if attempt['bridge_session']['user_id'] != identity['id']:
                raise RuntimeError('Proxy identity does not match the authenticated manifest')
            since = datetime.now(timezone.utc).isoformat()
            started = time.monotonic()
            # Persist before dispatch: a killed host must not forget paid work.
            state['inflight'] = {'skill':row['id'],'session_id':attempt['session_id'],'since':since}
            state['unknown_cost'] = True
            report(out,state)
            dispatched = True
            result = model_turn(name,case,text,attempt['session_id'],allowance,resume,args.enable_reviewed_writes)
            attempt['wall_seconds'] = round(time.monotonic()-started,3)
            attempt['exit_code'] = result.returncode
            raw = result.stdout+result.stderr
            save(PRIVATE/(state['run_id']+'-'+row['id']+f'-{attempt_id}.jsonl'),raw,[pat,token])
            save(folder/'transcript.jsonl',raw,[pat,token])
            summary = summarize(raw,row['id'])
            end = summary['result']
            attempt.update(cost_usd=end.get('total_cost_usd'), inference_ms=end.get('duration_api_ms'),
                           usage=end.get('usage'), model_usage=end.get('modelUsage'), skill_loaded=summary['skill_loaded'])
            if end.get('session_id'):
                attempt['session_id'] = end['session_id']
            files = collect_artifacts(name)
            for path,body in files.items():
                save(folder/'artifacts'/path,body,[pat,token])
            checks = artifact_checks(row['id'],files,FIXTURES)
            save(folder/'checks.json',checks)
            save(folder/'tool-calls.json',summary['tool_calls'],[pat,token])
            save(folder/'claude-initialization.json',summary['initialization'],[pat,token])
            save(folder/'output.md',end.get('result','No final response; inspect transcript.'),[pat,token])
            time.sleep(2)
            audit_result = audit(claims['sub'],attempt['session_id'],since,attempt['bridge_session']['session_id'])
            save(folder/'audit.json',audit_result)
            if audit_result.get('requests'):
                server_cost = sum(r['cost_microdollars'] for r in audit_result['requests'])/1_000_000
                attempt['server_cost_usd'] = server_cost
                if attempt['cost_usd'] is not None:
                    attempt['accounting_difference_usd'] = server_cost-attempt['cost_usd']
                    attempt['cost_usd'] = max(server_cost,attempt['cost_usd'])
            models = attempt.get('model_usage') or {}
            if not models or any('haiku-4-5' not in m for m in models):
                budget.unknown = True
                row['status'] = 'FAIL—MODEL ROUTING'
            elif not summary['skill_loaded'] and not (resume and prior.get('skill_loaded')):
                row['status'] = 'FAIL—SKILL NOT LOADED'
            elif result.returncode or end.get('is_error') or not all(checks.values()):
                row['status'] = 'FAIL—EXECUTION'
            else:
                row['status'] = 'REVIEW REQUIRED'
        except Exception as error:
            row['status'] = 'BLOCKED—ENVIRONMENT'
            attempt['error'] = str(error) if isinstance(error, RuntimeError) else type(error).__name__
            if not dispatched:
                attempt['cost_usd'] = 0
            diagnostic = cx(name,['cat','/home/tester/enrollment.log'],check=False)
            save(folder/'enrollment.txt', diagnostic.stdout, [pat,token])
        finally:
            if dispatched:
                budget.record(attempt.get('cost_usd'))
            attempt['client_retained'] = args.keep_clients
            attempt['outcome'] = row['status']
            if resume and prior.get('skill_loaded'):
                attempt['skill_loaded'] = True
            save(folder/'metrics.json', attempt, [pat,token])
            row['attempts'].append(attempt)
            state.update(spent_usd=str(budget.spent),unknown_cost=budget.unknown)
            state.pop('inflight',None)
            report(out,state)
            if not args.keep_clients:
                remove_client(name,state['run_id'])
            print(row['id']+': '+row['status']+'; cumulative USD '+str(budget.spent),flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    auth = parser.add_mutually_exclusive_group()
    auth.add_argument('--pat-file',type=Path)
    auth.add_argument('--code-file',type=Path,help='Private file containing a fresh live bridge connect code; exchanged PAT remains private')
    parser.add_argument('--prepare-only',action='store_true')
    parser.add_argument('--public-probe',action='store_true')
    parser.add_argument('--only',help='Comma-separated exact skill IDs')
    parser.add_argument('--retry',action='store_true',help='Preserve prior attempts; retries share the same US$5 ledger')
    parser.add_argument('--keep-clients',action='store_true',help='Retain scenario containers for reviewed follow-up turns; credentials stay private inside Docker')
    parser.add_argument('--followup-file',type=Path,help='Exact reviewed follow-up prompt; requires one --only skill and its retained client')
    parser.add_argument('--enable-reviewed-writes',action='store_true',help='Allow Atlassian executeWrite only for a reviewed follow-up')
    parser.add_argument('--cleanup',action='store_true',help='Remove only this run\'s retained containers; no remote data deletion')
    parser.add_argument('--local-skill-files',action='store_true',help='Explicit local patched retest, after recording the live install; never counted as deployed behavior')
    parser.add_argument('--review-file',type=Path,help='Per-criterion operator review with evidence paths; never substitutes for missing execution proof')
    parser.add_argument('--installer-file',type=Path,help='Explicit local bootstrap fix; downloaded bridge and skills still come from live endpoint')
    args = parser.parse_args()
    if args.followup_file and (not args.only or ',' in args.only):
        parser.error('--followup-file requires exactly one --only skill')
    if args.enable_reviewed_writes and not args.followup_file:
        parser.error('Writes require a reviewed --followup-file')
    if args.only and not set(args.only.split(',')) <= {c['id'] for c in cases()}:
        parser.error('Unknown skill ID in --only')
    if args.local_skill_files and (not args.retry or args.followup_file):
        parser.error('Local patched tests require --retry and a fresh scenario, not a follow-up')
    PRIVATE.mkdir(parents=True,exist_ok=True)
    PRIVATE.chmod(0o700)
    os.umask(0o077)
    out=args.output.resolve()
    # Separate processes must not each spend against the same stale ledger.
    run_lock = (PRIVATE/(sha(str(out).encode())+'.lock')).open('w')
    try:
        fcntl.flock(run_lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    except BlockingIOError:
        parser.error('Another process is using this run ledger')
    if (out/'results.json').exists():
        state=json.loads((out/'results.json').read_text())
        if state.get('gateway') != GATEWAY or state.get('model') != MODEL:
            raise RuntimeError('Output belongs to a different environment/model')
        if state.get('inflight'):
            state['unknown_cost'] = True
            state['blocking_reason'] = 'Interrupted model dispatch needs audit reconciliation before more spending'
    else:
        state=prepare(out,out.name)
    try:
        if args.review_file:
            apply_review(out,state,args.review_file)
            return 0
        if args.cleanup:
            for row in state['rows']:
                for attempt in row['attempts']:
                    if attempt.get('client_retained'):
                        remove_client(attempt['container'],state['run_id'])
                        attempt['client_retained'] = False
            return 0
        if args.public_probe:
            public_probe(out,state['run_id'])
        if not args.prepare_only:
            run(args,out,state)
    except Exception as error:
        state['blocking_reason']=str(error) if isinstance(error,RuntimeError) else type(error).__name__
        for row in state['rows']:
            if not row['attempts']:
                row['status']='BLOCKED—ENVIRONMENT'
    finally:
        report(out,state)
        run_lock.close()
    print(json.dumps({'output':str(out),'known_cost_usd':state['spent_usd'],'blocking_reason':state.get('blocking_reason')}))
    return 0 if args.prepare_only else (0 if all(r['status'].endswith('PASS') for r in state['rows']) else 1)


if __name__=='__main__':
    sys.exit(main())
