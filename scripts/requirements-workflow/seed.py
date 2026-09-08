#!/usr/bin/env python3
"""Seed only the owned, clearly labelled migration corpus through connected Atlassian MCP."""
import argparse
import hashlib
import json
from pathlib import Path
from urllib.parse import urlparse, parse_qs
from mcp_client import McpClient

ROOT = Path(__file__).resolve().parents[2]


def atomic_write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, indent=2)+'\n')
    temporary.replace(path)


def records(payload):
    return payload.get('results', payload.get('issues', []))


def run(args):
    corpus = json.loads((ROOT/'demo/fixtures/requirements-migration/corpus.json').read_text())
    manifest_path = args.state/'seed-manifest.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {'dataset': corpus['dataset'], 'records': {}}
    client = McpClient(args.gateway.rstrip('/')+'/api/v1/mcp/atlassian/mcp', args.token_file.read_text().strip())
    resources = client.call('getAccessibleAtlassianResources', {})['resources']
    if len(resources) != 1:
        raise RuntimeError('Select one connected Atlassian site before seeding')
    cloud = resources[0]['cloudId']
    def call(name, **kwargs):
        return client.call(name, dict(cloudId=cloud, **kwargs))
    spaces = call('listConfluenceSpaces', keys=corpus['space'])['results']
    space = next(s for s in spaces if s['key'] == corpus['space'])
    types = call('listJiraProjectIssueTypesMetadata', projectIdOrKey=corpus['project'])['issueTypes']
    if not {'Epic', 'Story', 'Task', 'Bug'} <= {t['name'] for t in types}:
        raise RuntimeError('Required issue types are unavailable')
    instructions = call('getConfluenceSpaceInstructions', spaceId=space['id'])
    if instructions:
        atomic_write(args.state/'space-instructions.json', instructions)
        # The initial execution has inspected the instruction state. A later change must be reviewed.
        previous = manifest.get('space_instructions')
        if previous is not None and previous != instructions:
            raise RuntimeError('Space instructions changed; review them before rerunning the seeder')
    manifest['space_instructions'] = instructions
    manifest['cloud_id'] = cloud
    manifest['space_id'] = space['id']
    atomic_write(manifest_path, manifest)
    pages = []
    cursor = None
    while True:
        payload = call('listConfluenceContent', type='page', spaceId=space['id'], limit=250, **({'cursor': cursor} if cursor else {}))
        pages.extend(records(payload))
        next_url = payload.get('_links', {}).get('next')
        cursor = parse_qs(urlparse(next_url or '').query).get('cursor', [None])[0]
        if not cursor:
            break
    for fixture in corpus['pages']:
        key = fixture['fixture_id']
        wanted = hashlib.sha256(fixture['body'].encode()).hexdigest()
        saved = manifest['records'].get(key)
        matches = [p for p in pages if p.get('title') == fixture['title']]
        if len(matches) > 1:
            raise RuntimeError('Duplicate demo page title: '+fixture['title'])
        page_id = saved.get('id') if saved else None
        if not page_id and matches:
            page_id = matches[0]['id']
        if page_id:
            page = call('getConfluenceContent', content_id=page_id, detail='full', content_format='markdown', include_metadata=True)
            if corpus['dataset'] not in page.get('body', {}).get('value', ''):
                raise RuntimeError('Refusing to modify a page without the demo ownership marker')
            if not saved or saved.get('hash') != wanted:
                token = page.get('snapshotToken') or page.get('metadata', {}).get('snapshotToken')
                if not token:
                    raise RuntimeError('Page update requires its advertised snapshot token')
                call('updateConfluenceContent', contentId=page_id, snapshotToken=token, body={'format': 'markdown', 'value': fixture['body']})
        else:
            if saved and saved.get('status') == 'creating':
                raise RuntimeError('Uncertain prior create for '+key+'; reconcile before another create')
            manifest['records'][key] = {'status': 'creating', 'title': fixture['title']}
            atomic_write(manifest_path, manifest)
            parent = {'spaceId': space['id']}
            if key != 'PAGE-01':
                parent['parentContentId'] = manifest['records']['PAGE-01']['id']
            created = call('createConfluenceContent', parent=parent, contentType='page', title=fixture['title'], body={'format': 'markdown', 'value': fixture['body']})
            page_id = created.get('id') or created.get('content', {}).get('id')
            if not page_id:
                atomic_write(args.state/'last-create-response.json', created)
                raise RuntimeError('Created page has no recognized ID; inspect response and reconcile')
        manifest['records'][key] = {'id': page_id, 'hash': wanted, 'title': fixture['title'], 'status': 'created'}
        atomic_write(manifest_path, manifest)
        call('addLabelsToConfluenceContent', contentId=page_id, labels=[{'name': x, 'prefix': 'global'} for x in fixture['labels']])
        print(key, page_id, 'ready', flush=True)
    found, cursor = [], None
    while True:
        args_search = {'jql': f'project = {corpus["project"]} AND labels = "{corpus["dataset"]}"', 'fields': ['summary', 'labels'], 'view': 'full', 'maxResults': 100}
        if cursor:
            args_search['nextPageToken'] = cursor
        payload = call('searchJiraIssuesUsingJql', **args_search)
        found.extend(payload.get('issues', []))
        cursor = payload.get('nextPageToken')
        if not cursor:
            break
    for fixture in corpus['issues']:
        key = fixture['fixture_id']
        wanted = hashlib.sha256(json.dumps(fixture, sort_keys=True).encode()).hexdigest()
        saved = manifest['records'].get(key)
        matches = [i for i in found if key.lower() in i.get('fields', {}).get('labels', [])]
        if len(matches) > 1:
            raise RuntimeError('Duplicate owned issue '+key)
        issue_key = saved.get('id') if saved else None
        if not issue_key and matches:
            issue_key = matches[0]['key']
        if issue_key:
            issue = call('getJiraIssue', issueIdOrKey=issue_key, fields=['summary','labels'], view='full')
            if corpus['dataset'] not in issue.get('fields', {}).get('labels', []):
                raise RuntimeError('Refusing to modify an issue without demo label')
            if not saved or saved.get('hash') != wanted:
                call('editJiraIssue', issueIdOrKey=issue_key, fields={'summary': fixture['title'], 'description': fixture['body'], 'labels': fixture['labels']})
        else:
            if saved and saved.get('status') == 'creating':
                raise RuntimeError('Uncertain prior issue create '+key+'; reconcile before retrying')
            manifest['records'][key] = {'status':'creating','title':fixture['title']}
            atomic_write(manifest_path, manifest)
            values = {'projectKey':corpus['project'],'summary':fixture['title'],'issueType':fixture['type'],'description':fixture['body'],'labels':fixture['labels']}
            if fixture.get('epic'):
                values['parent'] = manifest['records'][fixture['epic']]['id']
            created = call('createJiraIssue', **values)
            issue_key = created.get('key')
            if not issue_key:
                atomic_write(args.state/'last-create-response.json', created)
                raise RuntimeError('Created issue has no key; reconcile response')
        manifest['records'][key] = {'id':issue_key,'hash':wanted,'title':fixture['title'],'status':'created'}
        atomic_write(manifest_path,manifest)
        print(key,issue_key,'ready',flush=True)
    atomic_write(args.state/'destination.json', {'space_id':space['id'],'parent_id':manifest['records']['PAGE-01']['id']})
    print('Seed complete: 60 issues and 20 source pages. Actual workflow approvals remain pending.',flush=True)


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--gateway',required=True)
    parser.add_argument('--token-file',type=Path,required=True)
    parser.add_argument('--state',type=Path,default=ROOT/'storage/requirements-workflow')
    run(parser.parse_args())
