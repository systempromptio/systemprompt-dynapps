"""Offline adapter checks against the pinned upstream fixture and projection."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / 'services/requirements-workflow/upstream/.project/eval/evalsuite/openspec-feature/fixtures/store-locator'


class EvaluatorTests(unittest.TestCase):
    def workflow(self):
        artifacts = {}
        for phase in ('discovery', 'design', 'proposal', 'grooming'):
            artifacts[phase] = {'files': {phase + '.md': (FIXTURE / phase / 'final.ba.md').read_text()}}
        artifacts['specs'] = {'files': {
            'specs/' + str(p.relative_to(FIXTURE / 'specs.ba')): p.read_text()
            for p in (FIXTURE / 'specs.ba').rglob('*.md')
        }}
        return {'name': 'adapter-smoke', 'evidence': [], 'artifacts': artifacts}

    def run_adapter(self, workflow, phase, operation='validate'):
        with tempfile.TemporaryDirectory() as directory:
            source, output = Path(directory) / 'input.json', Path(directory) / 'output.json'
            source.write_text(json.dumps({'workflow': workflow, 'phase': phase, 'operation': operation}))
            process = subprocess.run(['node', str(ROOT / 'scripts/requirements-workflow/validate.mjs'), str(source), str(output)], capture_output=True, text=True, timeout=60)
            return process, json.loads(output.read_text()) if output.exists() else None

    def test_projection_resolves_approved_wbs_into_fsd_and_stories(self):
        process, result = self.run_adapter(self.workflow(), 'fsd', 'scaffold')
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(set(result['files']), {'fsd/store-locator.md', 'stories.md'})
        self.assertIn('Acceptance Criteria', result['files']['fsd/store-locator.md'])
        self.assertIn('SLS.SRCH', result['files']['stories.md'])

    def test_fsd_cannot_drop_the_projected_document(self):
        workflow = self.workflow()
        workflow['artifacts']['fsd'] = {'files': {'fsd/unrelated.md': '# Unrelated'}}
        process, _ = self.run_adapter(workflow, 'fsd')
        self.assertNotEqual(process.returncode, 0)
        self.assertIn('Supply every projected FSD document', process.stderr)

    def test_missing_discovery_structure_fails_before_model_judging(self):
        workflow = self.workflow()
        workflow['artifacts']['discovery']['files']['discovery.md'] = '# Unsupported draft\n'
        process, result = self.run_adapter(workflow, 'discovery')
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertFalse(result['passed'])
        self.assertEqual(result['reports'][0]['semantic_status'], 'not_run_structure_failed')


if __name__ == '__main__':
    unittest.main()
