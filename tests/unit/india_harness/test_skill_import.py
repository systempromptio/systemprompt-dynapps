"""Selective imports must preserve other suites and retain executable contracts."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import yaml

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location('india_import', ROOT / 'scripts/india/import-skills.py')
IMPORTER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(IMPORTER)


class SkillImportTests(unittest.TestCase):
    def test_ba_import_preserves_sf_and_is_repeatable(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'destination'
            sources = Path(tmp) / 'sources'
            source = sources / IMPORTER.SOURCES[0][1] / IMPORTER.SOURCES[0][2]
            source.mkdir(parents=True)
            (source / 'story-drafting.md').write_text('# Skill: Story Drafting\n\n**One-liner**: Draft stories.\n')
            config = root / 'services/plugins/astound-india-ba/config.yaml'
            config.parent.mkdir(parents=True)
            config.write_text(yaml.safe_dump({'plugin': {'id': 'astound-india-ba', 'version': '2.1.0', 'skills': {'include': []}}}))
            sf_file = root / 'services/skills/sf_example/SKILL.md'
            sf_file.parent.mkdir(parents=True)
            sf_file.write_bytes(b'preserve concurrent Salesforce edits\n')
            inventory = root / 'scripts/india/skills-inventory.json'
            inventory.parent.mkdir(parents=True)
            sf_source = {'plugin': 'astound-india-sfcore', 'commit': 'unchanged'}
            sf_skill = {'plugin': 'astound-india-sfcore', 'id': 'sf_example', 'custom': {'preserve': True}}
            inventory.write_text(json.dumps({'schema_version': 1, 'sources': [sf_source], 'skills': [sf_skill]}))

            def git_output(command, **kwargs):
                return '' if 'status' in command else ('a' * 40 + '\n' if 'rev-parse' in command else 'https://example.com/source.git\n')

            with patch.object(IMPORTER, 'ROOT', root), patch('sys.argv', ['import-skills', '--suite', 'ba', '--source-root', str(sources)]), patch.object(IMPORTER.subprocess, 'check_output', side_effect=git_output):
                IMPORTER.main()
                first = {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()}
                IMPORTER.main()
                self.assertEqual(first, {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()})
            imported = json.loads(inventory.read_text())
            self.assertIn(sf_source, imported['sources'])
            self.assertIn(sf_skill, imported['skills'])
            self.assertEqual(sf_file.read_bytes(), b'preserve concurrent Salesforce edits\n')
            self.assertEqual(yaml.safe_load(config.read_text())['plugin']['version'], '2.1.0')

    def test_missing_source_fails_before_writing(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with patch.object(IMPORTER, 'ROOT', root), patch('sys.argv', ['import-skills', '--suite', 'ba', '--source-root', tmp]):
                with self.assertRaises(SystemExit):
                    IMPORTER.main()
            self.assertEqual(list(root.iterdir()), [])

    def test_standalone_transport_adaptation(self):
        text = '''Use the BA platform confirmation token for all `push_to_jira` calls
Wait for explicit user confirmation in the current turn
Pass it as `parentKey` in `push_to_jira` or `draft_ticket`
| Jira push (via `push_to_jira` or `draft_ticket`) | **Jira Wiki Markup** — example |
### Jira Wiki Markup rules
'''
        adapted = IMPORTER.adapt_ba_standalone(text)
        self.assertNotIn('confirmation token', adapted)
        self.assertNotIn('`parentKey`', adapted)
        self.assertNotIn('current turn', adapted)
        self.assertIn('local draft', adapted)
        self.assertIn('advertised relationship fields', adapted)
        self.assertIn('| Jira remote write | **Markdown-style formatting**', adapted)
        self.assertIn('ADF when required', adapted)
        self.assertIn('wiki-capable schemas only', adapted)

    def test_inventory_and_ba_contracts(self):
        inventory = json.loads((ROOT / 'scripts/india/skills-inventory.json').read_text())
        ba = [s for s in inventory['skills'] if s['plugin'] == 'astound-india-ba']
        self.assertEqual(len(ba), 16)
        self.assertEqual(len(inventory['skills']), 34)
        approval_skills = 0
        for entry in ba:
            raw = (ROOT / entry['destination']).read_bytes()
            body = raw.decode()
            self.assertEqual(hashlib.sha256(raw).hexdigest(), entry['destination_sha256'], entry['id'])
            self.assertIn('**Standalone use:**', body)
            self.assertIn('If Jira succeeds but the tracker write fails', body)
            self.assertIn('Update RTM only after confirmed Jira success', body)
            self.assertIn('Wiki-markup rules below apply only when that\nschema accepts wiki markup', body)
            self.assertNotIn('Example Jira description structure (Markdown style - PREFERRED FORMAT) (wiki-capable schemas only)', body)
            self.assertNotIn('Use the BA platform confirmation token', body)
            self.assertNotIn('`parentKey`', body)
            for obsolete in [*IMPORTER.ADAPTERS, *IMPORTER.LEGACY_TOOLS]:
                self.assertNotIn('`' + obsolete + '`', body, entry['id'])
            if '## ⛔ Human Approval Required' in body:
                approval_skills += 1
                self.assertIn('confirmation of the concrete proposal', body)
        self.assertEqual(approval_skills, 13)
        story = (ROOT / 'services/skills/ba_story_drafting/SKILL.md').read_text()
        for rule in ['**NO Gherkin**', '**Every error state**', '**Every field**', 'Parent Epic Linking', 'Technical Notes', 'maximum of 5 targeted questions', 'Related Stories & Sequencing Rules', 'Blocked-by']:
            self.assertIn(rule, story)


if __name__ == '__main__':
    unittest.main()
