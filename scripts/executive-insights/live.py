"""Run a delivered skill through the local gateway and its caller-scoped MCPs.

Requires a caller PAT file, a running local gateway, and Python cryptography.
Raw evidence stays in the supplied private output directory. No write MCP tools
are exposed; reports require human review in addition to the execution summary.
"""
import argparse
import base64
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sys
import time
import urllib.error
import urllib.request

from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'requirements-workflow'))
from mcp_client import McpClient

READ_TOOLS = {
    'getAccessibleAtlassianResources', 'listJiraProjects', 'searchJiraIssuesUsingJql',
    'getJiraIssue', 'listJiraIssueChangelogs', 'listJiraIssueLinkTypes',
    'listJiraIssueComments', 'listJiraIssueRemoteIssueLinks', 'getJiraProjectVersions',
    'listConfluenceSpaces', 'listConfluenceContent', 'searchConfluence',
    'getConfluenceContent', 'listConfluenceContentVersions',
}
KNOWLEDGE_READS = {'search_knowledge', 'list_knowledge_sources'}


def http(base, path, token=None, body=None, extra=None):
    headers = {'Content-Type': 'application/json', **(extra or {})}
    if token:
        headers['Authorization'] = 'Bearer ' + token
    request = urllib.request.Request(base + path, headers=headers,
                                     data=json.dumps(body).encode() if body is not None else None)
    try:
        with urllib.request.urlopen(request, timeout=180) as response:
            return json.load(response), dict(response.headers)
    except urllib.error.HTTPError as error:
        raise RuntimeError(f'HTTP {error.code}: {error.read().decode()[:1500]}') from None


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    out.chmod(0o700)
    token, _ = http(args.gateway, '/v1/auth/bridge/pat',
                    args.credential_file.read_text().strip(), {})
    token = token['token']
    encoded = token.split('.')[1]
    claims = json.loads(base64.urlsafe_b64decode(encoded + '=' * (-len(encoded) % 4)))
    envelope, _ = http(args.gateway, '/v1/bridge/manifest', token)
    pubkey, _ = http(args.gateway, '/v1/bridge/pubkey')
    Ed25519PublicKey.from_public_bytes(base64.b64decode(pubkey['pubkey'])).verify(
        base64.b64decode(envelope['signature']), envelope['payload'].encode())
    manifest = json.loads(envelope['payload'])
    if manifest['user_id'] != claims['sub']:
        raise RuntimeError('Manifest and authenticated caller disagree')
    skill = next(s for s in manifest['skills'] if s['id'] == args.skill)
    instructions = skill['instructions']
    available = {s['name'] for s in manifest['managed_mcp_servers']}
    clients, specs, routes, gaps = {}, [], {}, []
    servers = ['atlassian'] if args.skill.startswith('executive_') else ['atlassian', 'knowledge-bank', 'salesforce']
    for server in servers:
        if server not in available:
            gaps.append(f'{server}: not delivered in caller manifest')
            continue
        if server == 'salesforce':
            raise RuntimeError('This read-only harness has not selected Salesforce read schemas; extend it before claiming a CRM test')
        client = McpClient(args.gateway + '/api/v1/mcp/' + server + '/mcp', token,
                           context_headers={'x-session-id': claims['session_id']}, timeout=90)
        clients[server] = client
        for tool in client.tools():
            selected = tool['name'] in (READ_TOOLS if server == 'atlassian' else KNOWLEDGE_READS)
            if not selected:
                continue
            if server == 'atlassian' and not tool.get('annotations', {}).get('readOnlyHint'):
                raise RuntimeError('Selected Atlassian tool lacks read-only annotation')
            name = server.replace('-', '_') + '__' + tool['name']
            routes[name] = (server, tool['name'])
            specs.append({'name': name, 'description': tool.get('description', ''),
                          'input_schema': tool['inputSchema']})
    if args.web:
        specs.append({'type': 'web_search_20250305', 'name': 'web_search', 'max_uses': 5})
    system = instructions + '\n\nRuntime source availability: ' + '; '.join(gaps)
    system += '\nTools are the currently discovered read schemas. Return the requested report in Markdown.'
    system += '\nCurrent report timestamp: ' + datetime.now(timezone.utc).isoformat()
    messages = [{'role': 'user', 'content': args.prompt_file.read_text()}]
    transcript = []
    if args.resume:
        for entry in json.loads((out / 'transcript.json').read_text()):
            if 'response' in entry:
                if entry['response'].get('stop_reason') in ['max_tokens', 'end_turn']:
                    break
                if messages[-1]['role'] == 'user' and not messages[-1]['content']:
                    messages.pop()
                messages.append({'role': 'assistant', 'content': entry['response']['content']})
                messages.append({'role': 'user', 'content': []})
            elif 'tool' in entry:
                response_text = json.dumps(entry['result'], ensure_ascii=False)
                if len(response_text) > 65000:
                    response_text = json.dumps({'response_truncated': True,
                        'instruction': 'Narrow the query or request fewer fields; this is incomplete evidence.',
                        'prefix': response_text[:65000]})
                response = next(x['response'] for x in reversed(transcript) if 'response' in x)
                used = len(messages[-1]['content'])
                calls = [x for x in response['content'] if x.get('type') == 'tool_use']
                messages[-1]['content'].append({'type': 'tool_result', 'tool_use_id': calls[used]['id'],
                    'content': response_text, 'is_error': entry.get('error', False)})
            transcript.append(entry)
    started = time.time()
    usage = {'input_tokens': 0, 'output_tokens': 0}
    status = 'turn_limit'
    result = None
    for turn in range(args.max_turns):
        result, headers = http(args.gateway, '/v1/messages', token, {
            'model': args.model, 'max_tokens': args.max_output_tokens, 'system': system,
            'messages': messages, 'tools': specs,
        }, {'anthropic-version': '2023-06-01', 'x-session-id': claims['session_id'],
            'User-Agent': 'astound-executive-live-evaluation/1.0'})
        transcript.append({'turn': turn, 'response': result,
                           'request_id': headers.get('x-request-id')})
        for key in usage:
            usage[key] += result.get('usage', {}).get(key, 0)
        content = result.get('content', [])
        messages.append({'role': 'assistant', 'content': content})
        calls = [item for item in content if item.get('type') == 'tool_use']
        print(args.skill, 'turn', turn + 1, result.get('stop_reason'),
              [c['name'] for c in calls], flush=True)
        if not calls:
            if result.get('stop_reason') == 'pause_turn':
                continue
            status = 'completed' if result.get('stop_reason') == 'end_turn' else result.get('stop_reason', 'unknown')
            break
        replies = []
        for call in calls:
            error = False
            try:
                server, name = routes[call['name']]
                data = clients[server].call(name, call['input'])
            except Exception as exc:
                error = True
                data = {'error': str(exc)}
            transcript.append({'tool': call['name'], 'input': call['input'], 'result': data, 'error': error})
            encoded_result = json.dumps(data, ensure_ascii=False)
            if len(encoded_result) > 65000:
                encoded_result = json.dumps({'response_truncated': True,
                    'instruction': 'Narrow the query or request fewer fields; this is incomplete evidence.',
                    'prefix': encoded_result[:65000]})
            replies.append({'type': 'tool_result', 'tool_use_id': call['id'],
                            'content': encoded_result, 'is_error': error})
        messages.append({'role': 'user', 'content': replies})
        (out / 'transcript.json').write_text(json.dumps(transcript, indent=2))
    report_parts = []
    for item in (result or {}).get('content', []):
        if item.get('type') != 'text':
            continue
        rendered = item['text']
        for citation in item.get('citations', []):
            if citation.get('url'):
                rendered += f" [{citation.get('title', 'Source')}]({citation['url']})"
        report_parts.append(rendered)
    report = ''.join(report_parts)
    summary = {'skill': args.skill, 'user_id': claims['sub'], 'session_id': claims['session_id'],
               'manifest_signature_verified': True, 'instruction_sha256': hashlib.sha256(instructions.encode()).hexdigest(),
               'status': status, 'model': (result or {}).get('model'), 'seconds': round(time.time() - started),
               'source_gaps': gaps, 'usage': usage, 'tool_calls': sum('tool' in x for x in transcript),
               'tool_errors': sum(x.get('error', False) for x in transcript),
               'web_search_enabled': args.web,
               'web_search_requests': sum(x.get('response', {}).get('usage', {}).get('server_tool_use', {}).get('web_search_requests', 0) for x in transcript),
               'web_search_errors': sum(1 for x in transcript for block in x.get('response', {}).get('content', [])
                   if block.get('type') == 'web_search_tool_result' and isinstance(block.get('content'), dict)
                   and block['content'].get('type') == 'web_search_tool_result_error'),
               'human_review_required': True}
    (out / 'transcript.json').write_text(json.dumps(transcript, indent=2))
    citations = {}
    for item in (result or {}).get('content', []):
        for citation in item.get('citations', []):
            if citation.get('url'):
                citations[citation['url']] = citation.get('title', citation['url'])
    if citations:
        report += '\n\n## Web sources\n\n' + '\n'.join(f'- [{title}]({url})' for url, title in citations.items())
    (out / 'report.md').write_text(report)
    (out / 'summary.json').write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary), flush=True)
    if status != 'completed':
        raise SystemExit(1)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--gateway', default='http://127.0.0.1:8080')
    parser.add_argument('--credential-file', type=Path, required=True)
    parser.add_argument('--skill', required=True)
    parser.add_argument('--prompt-file', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--model', default='claude-sonnet-5')
    parser.add_argument('--max-turns', type=int, default=12)
    parser.add_argument('--web', action='store_true')
    parser.add_argument('--resume', action='store_true')
    parser.add_argument('--max-output-tokens', type=int, default=16000)
    run(parser.parse_args())
