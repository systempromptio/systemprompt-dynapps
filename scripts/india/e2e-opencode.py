#!/usr/bin/env python3
"""Clean India enrollment and real-account MCP checks. Missing auth is a failure."""
import argparse
import base64
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
VERSION = '1.18.29'
CONNECTORS = {'atlassian', 'salesforce', 'github'}


def request(url, token=None, data=None, headers=None):
    h = dict(headers or {})
    if token:
        h['Authorization'] = 'Bearer ' + token
    if data is not None:
        h['Content-Type'] = 'application/json'
    req = urllib.request.Request(url, headers=h, data=None if data is None else json.dumps(data).encode())
    with urllib.request.urlopen(req, timeout=30) as response:
        raw = response.read().decode()
        if raw.startswith('event:') or raw.startswith('data:'):
            messages = [json.loads(line[6:]) for line in raw.splitlines() if line.startswith('data: ')]
            value = next((m for m in reversed(messages) if 'result' in m or 'error' in m), {})
        else:
            value = json.loads(raw) if raw.strip() else {}
        return value, dict(response.headers)


def exact(actual, expected, label):
    if set(actual) != set(expected):
        raise RuntimeError(f'{label}: missing={sorted(set(expected)-set(actual))}, extra={sorted(set(actual)-set(expected))}')


def run(args):
    inventory = json.loads((ROOT/'docs/india-skills-inventory.json').read_text())
    skills = {s['id'] for s in inventory['skills']}
    plugins = {'astound-india-ba', 'astound-india-sfcore'}
    pat = args.pat_file.read_text().strip()
    gateway = args.gateway.rstrip('/')
    auth, _ = request(gateway+'/v1/auth/bridge/pat', pat, {})
    token = auth['token']
    claims = json.loads(base64.urlsafe_b64decode(token.split('.')[1] + '==='))
    user = claims['sub']
    wire, _ = request(gateway+'/v1/bridge/manifest', token)
    manifest = wire['payload']
    if isinstance(manifest, str):
        manifest = json.loads(manifest)
    exact((x['id'] for x in manifest['marketplaces']), ['astound-india-dev'], 'marketplaces')
    exact((x['id'] for x in manifest['plugins']), plugins, 'plugins')
    exact((x['id'] for x in manifest['skills']), skills, 'skills')
    exact((x['name'] for x in manifest['managed_mcp_servers']), CONNECTORS, 'connectors')
    args.output.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='india-opencode-') as sandbox:
        home = Path(sandbox)/'home'
        home.mkdir()
        env = {k: v for k, v in os.environ.items() if not any(s in k for s in ('TOKEN', 'API_KEY', 'AUTH_TOKEN'))}
        env.update(HOME=str(home), XDG_CONFIG_HOME=str(home/'.config'), XDG_DATA_HOME=str(home/'.local/share'),
                   XDG_CACHE_HOME=str(home/'.cache'), OPENCODE_DISABLE_AUTOUPDATE='1', ASTOUND_BRIDGE_GATEWAY_URL=gateway)
        binary = args.opencode
        if binary is None:
            subprocess.run(['npm', 'install', '--prefix', sandbox, 'opencode-ai@'+VERSION], env=env, check=True, stdout=subprocess.DEVNULL)
            binary = Path(sandbox)/'node_modules/.bin/opencode'
        actual = subprocess.check_output([str(binary), '--version'], env=env, text=True).strip()
        if actual != VERSION:
            raise RuntimeError(f'Expected OpenCode {VERSION}; got {actual}')
        script = home/'install.sh'
        with urllib.request.urlopen(gateway+'/files/downloads/install.sh', timeout=30) as response:
            script.write_bytes(response.read())
        try:
            # Installer output can contain connect details, so never publish it.
            subprocess.run(['sh', str(script), '--download-base', gateway+'/files/downloads', '--gateway', gateway,
                            '--pat', pat, '--host', 'opencode'], env=env, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            directory = home/'.config/opencode/skills'
            sidecar = directory/'.systemprompt-managed.json'
            for _ in range(60):
                if sidecar.exists():
                    break
                time.sleep(1)
            synced = json.loads(sidecar.read_text())
            exact(synced['marketplaces'], ['astound-india-dev'], 'synced marketplaces')
            exact((p.name for p in directory.iterdir() if p.is_dir() and not p.name.startswith('.')),
                  (s.replace('_', '-') for s in skills), 'installed skills')
            for skill in skills:
                if not (directory/skill.replace('_', '-')/'SKILL.md').is_file():
                    raise RuntimeError('Missing installed skill body: '+skill)
            config_path = next(p for p in [home/'.config/opencode/opencode.jsonc', home/'.config/opencode/opencode.json'] if p.exists())
            # Bridge emits JSON even when the extension is jsonc.
            config = json.loads(config_path.read_text())
            exact(config.get('mcp', {}), CONNECTORS, 'OpenCode MCP configuration')
            loopback = (home/'.config/astound/bridge-loopback.key').read_text().strip()
            for connector in sorted(CONNECTORS):
                url = config['mcp'][connector]['url']
                h = {'Accept': 'application/json, text/event-stream'}
                initialize, response_headers = request(url, loopback, {'jsonrpc':'2.0','id':1,'method':'initialize',
                    'params':{'protocolVersion':'2025-06-18','capabilities':{},'clientInfo':{'name':'india-e2e','version':'1'}}}, h)
                if 'error' in initialize:
                    raise RuntimeError(connector+' initialize failed')
                sid = next((v for k,v in response_headers.items() if k.lower()=='mcp-session-id'), None)
                if sid:
                    h['Mcp-Session-Id'] = sid
                request(url, loopback, {'jsonrpc':'2.0','method':'notifications/initialized'}, h)
                result, _ = request(url, loopback, {'jsonrpc':'2.0','id':2,'method':'tools/list'}, h)
                tools = result.get('result', {}).get('tools', [])
                if not tools:
                    raise RuntimeError(connector+' has no authenticated tools; authorization is required')
            work = home/'work'
            work.mkdir()
            cutoff = datetime.now(timezone.utc).isoformat()
            model = args.model or next(iter(config['provider']['systemprompt']['models']))
            prompt = ('Load the ba-jira-management and sf-core-logic skills. Use each of the atlassian, salesforce and github '
                      'MCP connectors to perform one authenticated read of the current account or accessible metadata. '
                      'Use only advertised read tools, make no remote writes, and do not use shell commands or direct HTTP. '
                      'Any missing authorization is a test failure. Report the tool names and outcomes.')
            with (args.output/'opencode.jsonl').open('w') as transcript:
                subprocess.run([str(binary), 'run', '--format', 'json', '--model', 'systemprompt/'+model, prompt],
                               cwd=work, env=env, check=True, timeout=300, stdout=transcript, stderr=subprocess.STDOUT)
            # Server records, not model narration, determine the live result.
            db_url = os.environ.get('SYSTEMPROMPT_DATABASE_URL')
            if not db_url:
                db_url = json.loads((ROOT/'.systemprompt/profiles/local/secrets.json').read_text())['database_url']
            sql = """SELECT DISTINCT e.server_name FROM mcp_tool_executions e
                JOIN ai_request_tool_calls c ON c.mcp_execution_id = e.mcp_execution_id
                JOIN ai_requests r ON r.id = c.request_id
                WHERE e.user_id = :'uid' AND e.status = 'success' AND e.created_at >= :'cutoff'::timestamptz
                AND r.user_id = :'uid' AND (r.model = :'model' OR r.requested_model = :'model');"""
            proof = subprocess.run(['psql', db_url, '-X', '-tA', '-v', 'ON_ERROR_STOP=1', '-v', 'uid='+user,
                                    '-v', 'cutoff='+cutoff, '-v', 'model='+model], input=sql, text=True, capture_output=True, check=True)
            exact(proof.stdout.splitlines(), CONNECTORS, 'audited real-model connector executions')
            return {'status':'PASS', 'user':user, 'skills':len(skills), 'plugins':sorted(plugins), 'connectors':sorted(CONNECTORS), 'opencode':actual}
        finally:
            # Stop only bridges belonging to this disposable client environment.
            for proc in Path('/proc').glob('[0-9]*'):
                try:
                    environ = (proc/'environ').read_bytes().split(b'\0')
                    command = (proc/'cmdline').read_bytes()
                    if ('HOME='+str(home)).encode() in environ and b'astound-bridge' in command:
                        os.kill(int(proc.name), 15)
                except (OSError, ProcessLookupError):
                    pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--gateway', default='http://127.0.0.1:8080')
    parser.add_argument('--pat-file', type=Path, required=True, help='File containing the real India user Systemprompt PAT; no provider secrets')
    parser.add_argument('--model')
    parser.add_argument('--opencode', type=Path)
    parser.add_argument('--output', type=Path, default=ROOT/'.build/india-e2e')
    args = parser.parse_args()
    try:
        result = run(args)
    except Exception as error:
        # Exception strings from HTTP/process calls may include credentials.
        result = {'status':'FAIL', 'reason': str(error) if isinstance(error, RuntimeError) else type(error).__name__}
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output/'result.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result))
    raise SystemExit(0 if result['status']=='PASS' else 1)


if __name__ == '__main__':
    main()
