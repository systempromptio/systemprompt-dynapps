"""Evidence and accounting helpers for the clean India Claude Code sweep."""
import csv
from decimal import Decimal, InvalidOperation
import hashlib
import io
import json
import re
from pathlib import Path

SECRET_PATTERNS = [
    r'\bsp-live-[A-Za-z0-9_-]+',
    r'\bsk-(?:ant-)?[A-Za-z0-9_-]{16,}',
    r'\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+',
    r'(?i)(?:Bearer\s+)[A-Za-z0-9._~-]+',
    r'postgres(?:ql)?://[^\s"\']+',
]


def redact(value, secrets=()):
    text = value if isinstance(value, str) else json.dumps(value, ensure_ascii=False)
    for secret in secrets:
        if secret:
            text = text.replace(secret, '[REDACTED]')
    for pattern in SECRET_PATTERNS:
        text = re.sub(pattern, '[REDACTED]', text)
    return text if isinstance(value, str) else json.loads(text)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def exact(actual, expected):
    a, e = list(actual), list(expected)
    return {'pass': set(a) == set(e) and len(a) == len(set(a)),
            'missing': sorted(set(e)-set(a)), 'extra': sorted(set(a)-set(e)),
            'duplicates': sorted(x for x in set(a) if a.count(x) > 1)}


class Budget:
    """Reserve before dispatch; missing accounting stops subsequent calls."""
    def __init__(self, cap='5', spent='0', unknown=False):
        self.cap, self.spent = Decimal(str(cap)), Decimal(str(spent))
        self.unknown = unknown
        if not self.cap.is_finite() or self.cap <= 0 or self.cap > 5:
            raise ValueError('This sweep is authorized for at most US$5')

    def allowance(self, requested='0.12'):
        # Haiku 200k context at up to $2/M cache write plus 1536 output tokens
        # fits $0.50. Claude checks its budget between, not during, requests.
        if self.unknown:
            return Decimal('0')
        return max(Decimal('0'), min(Decimal(requested), self.cap-self.spent-Decimal('0.50')))

    def record(self, amount):
        try:
            number = Decimal(str(amount))
        except InvalidOperation:
            self.unknown = True
            return
        if not number.is_finite() or number < 0:
            self.unknown = True
            return
        self.spent += number


def events(raw):
    parsed = []
    for line in raw.splitlines():
        try:
            event = json.loads(line)
            if isinstance(event, dict):
                parsed.append(event)
        except json.JSONDecodeError:
            continue
    return parsed


def summarize(raw, skill):
    stream = events(raw)
    results = [e for e in stream if e.get('type') == 'result']
    result = results[-1] if results else {}
    calls = []
    succeeded = set()
    for event in stream:
        for block in event.get('message', {}).get('content', []) if isinstance(event.get('message'), dict) else []:
            if isinstance(block, dict) and block.get('type') == 'tool_use':
                calls.append({'id': block.get('id'), 'name': block.get('name'), 'input': block.get('input', {})})
            if isinstance(block, dict) and block.get('type') == 'tool_result' and not block.get('is_error', False):
                succeeded.add(block.get('tool_use_id'))
    names = [c['input'].get('skill', '') for c in calls if c['name'] == 'Skill' and c['id'] in succeeded]
    loaded = any(n.split(':')[-1].replace('-', '_') == skill for n in names)
    # Slash invocation can be expanded before any model call; evidence is its
    # command expansion event rather than counting the prompt as success.
    for event in stream:
        if event.get('type') == 'user':
            blocks = event.get('message', {}).get('content', '')
            content = str(blocks)
            error = isinstance(blocks, list) and any(isinstance(b, dict) and b.get('is_error') for b in blocks)
            if not error and 'Base directory for this skill:' in content and skill.replace('_', '-') in content:
                loaded = True
    return {'result': result, 'skill_loaded': loaded, 'tool_calls': calls,
            'initialization': next((e for e in stream if e.get('type') == 'system' and e.get('subtype') == 'init'), {})}


def installation(manifest, disk, inventory):
    markets = [m['id'] for m in manifest.get('marketplaces', [])]
    keys = [f'{p}@{m["id"]}' for m in manifest.get('marketplaces', []) for p in m.get('plugin_ids', [])]
    expected_skills = [s for s in manifest.get('skills', []) if not s.get('hosts') or 'claude-code' in s['hosts']]
    checks = {
        'marketplaces': exact(disk['marketplaces'], markets),
        'registry': exact(disk['registry'], keys),
        'enabled_registry': exact(disk['enabled'], keys),
        'skills': exact(sorted(set(disk['skills'])), [s['id'].replace('_', '-') for s in expected_skills]),
        'india_subset': exact([s['id'] for s in expected_skills if s['id'].startswith(('ba_', 'sf_'))], [s['id'] for s in inventory]),
    }
    manifest_bodies = {s['id'].replace('_', '-'): s['instructions'].strip() for s in expected_skills}
    mismatches = []
    for item in disk['bodies']:
        expected = manifest_bodies.get(item['name'])
        if expected is None or expected not in item['body'].strip():
            mismatches.append(item['path'])
    checks['skill_bodies'] = {'pass': not mismatches, 'mismatches': mismatches}
    refs = [p for p in disk['files'] if '/sf-salesforce-project-context/references/' in p]
    expected_refs = [str(p.relative_to(Path(__file__).resolve().parents[2]/'services/skills/sf_salesforce_project_context'))
                     for p in (Path(__file__).resolve().parents[2]/'services/skills/sf_salesforce_project_context/references').rglob('*') if p.is_file()]
    checks['bundled_references'] = exact(sorted(set(p.split('/sf-salesforce-project-context/')[1] for p in refs)), expected_refs)
    checks['mcp_set'] = exact(disk['mcp_names'], [s['name'] for s in manifest.get('managed_mcp_servers', [])])
    return {'status': 'PASS' if all(c['pass'] for c in checks.values()) else 'FAIL', 'checks': checks,
            'ordinary_developer_isolation': 'UNVERIFIED: admin-only enrollment'}


def artifact_checks(skill, files, original):
    """Mechanical checks do not replace the case's human-reviewed rubric."""
    checks = {'saved_evidence_summary': bool(files.get('output/review.md', '').strip())}
    if skill == 'ba_tracker_management':
        path = '.astound/ba/trackers/Questions.csv'
        rows = list(csv.DictReader(io.StringIO(files.get(path, ''))))
        checks['preserved_question'] = bool(rows) and rows[0].get('answer') == 'Preserve this answer'
        checks['question_roundtrip'] = len(rows) == 2 and rows[-1].get('question') == 'Should tax, shipping be rounded?' and rows[-1].get('answer') == 'Yes, half-to-even.' and None not in rows[-1]
        risks = list(csv.DictReader(io.StringIO(files.get('.astound/ba/trackers/Risk_Register.csv', ''))))
        checks['sequential_risks'] = [r.get('risk_id') for r in risks] == ['RISK-001', 'RISK-002']
    if skill == 'ba_transcript_management':
        prior = '.astound/ba/transcripts/existing.md'
        checks['preserved_transcript'] = files.get(prior) == original[prior]
        checks['new_transcript'] = any('half-to-even' in v for k, v in files.items() if k.startswith('.astound/ba/transcripts/') and k != prior)
    if skill == 'ba_session_memory':
        records = [e for k, v in files.items() if k.endswith('corrections.jsonl') for e in events(v)]
        checks['exact_correction'] = any(e.get('correction_text') == 'Use half-to-even rounding, not half-up.' and all(e.get(f) for f in ['session_id', 'agent_output', 'correction_type', 'extracted_fact']) for e in records)
    if skill == 'ba_notification_sending':
        checks['saved_notification_drafts'] = len([p for p in files if p.startswith('.astound/ba/notifications/')]) >= 2
    return checks
