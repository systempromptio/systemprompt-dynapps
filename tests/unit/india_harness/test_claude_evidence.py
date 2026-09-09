"""Run with python3 -m unittest discover -s tests/unit/india_harness."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'scripts/india'))
from claude_cases import FIXTURES, cases, prompt
from claude_evidence import Budget, artifact_checks, exact, redact, summarize


class EvidenceTests(unittest.TestCase):
    def test_missing_cost_stops_dispatch(self):
        ledger=Budget()
        ledger.record(None)
        self.assertTrue(ledger.unknown)
        self.assertEqual(ledger.allowance(),0)

    def test_budget_reserves_inflight_request_and_counts_retries(self):
        ledger=Budget()
        ledger.record('2.40')
        ledger.record('2.09')
        self.assertEqual(str(ledger.allowance()),'0.01')
        ledger.record('0.03')
        self.assertEqual(ledger.allowance(),0)
        with self.assertRaises(ValueError): Budget('50')

    def test_nan_negative_and_infinity_are_not_free_usage(self):
        for value in ['NaN','Infinity','-1','garbage']:
            ledger=Budget()
            ledger.record(value)
            self.assertEqual(ledger.allowance(),0)

    def test_extra_missing_and_duplicate_grants_fail(self):
        self.assertFalse(exact(['india','europe'],['india'])['pass'])
        self.assertFalse(exact([],['india'])['pass'])
        self.assertFalse(exact(['india','india'],['india'])['pass'])
        self.assertTrue(exact(['india'],['india'])['pass'])

    def test_redacts_credentials_without_damaging_json(self):
        data={'output':'Bearer abcDEF123; sp-live-private_value; explicit-test-secret',
              'url':'postgres://user:pass@host/db', 'tokens':123}
        clean=redact(data,['explicit-test-secret'])
        self.assertNotIn('private_value',json.dumps(clean))
        self.assertNotIn('explicit-test-secret',json.dumps(clean))
        self.assertNotIn('pass@host',json.dumps(clean))
        self.assertEqual(clean['tokens'],123)

    def test_model_narration_does_not_prove_skill_loading(self):
        raw=json.dumps({'type':'result','result':'I loaded ba-tracker-management successfully','total_cost_usd':.02})
        self.assertFalse(summarize(raw,'ba_tracker_management')['skill_loaded'])
        raw='\n'.join([json.dumps({'type':'assistant','message':{'content':[{'type':'tool_use','id':'t1','name':'Skill','input':{'skill':'astound-india-ba:ba-tracker-management'}}]}}),raw])
        self.assertFalse(summarize(raw,'ba_tracker_management')['skill_loaded'])
        failure=json.dumps({'type':'user','message':{'content':[{'type':'tool_result','tool_use_id':'t1','is_error':True,'content':'Skill not found'}]}})
        self.assertFalse(summarize(raw+'\n'+failure,'ba_tracker_management')['skill_loaded'])
        raw+='\n'+json.dumps({'type':'user','message':{'content':[{'type':'tool_result','tool_use_id':'t1','content':'Loaded skill instructions'}]}})
        self.assertTrue(summarize(raw,'ba_tracker_management')['skill_loaded'])

    def test_csv_checks_reject_corruption_and_lost_rows(self):
        files=dict(FIXTURES, **{'output/review.md':'Saved'})
        files['.astound/ba/trackers/Questions.csv']+='"09-09-2026 | 10:00",Demo,Checkout,"Should tax, shipping be rounded?","Yes, half-to-even."\n'
        files['.astound/ba/trackers/Risk_Register.csv']+='RISK-002,Rounding,High,High,Test,Open\n'
        self.assertTrue(all(artifact_checks('ba_tracker_management',files,FIXTURES).values()))
        files['.astound/ba/trackers/Questions.csv']=files['.astound/ba/trackers/Questions.csv'].replace('"Should tax, shipping be rounded?"','Should tax, shipping be rounded?')
        self.assertFalse(artifact_checks('ba_tracker_management',files,FIXTURES)['question_roundtrip'])

    def test_all_imported_skills_have_unique_cases_and_rubrics(self):
        inventory=json.loads((ROOT/'docs/india-skills-inventory.json').read_text())['skills']
        self.assertEqual({s['id'] for s in inventory},{c['id'] for c in cases()})
        self.assertEqual(len(cases()),34)
        for case in cases():
            self.assertGreaterEqual(len(case['rubric']),3)
            self.assertIn(case['id'].replace('_','-'),prompt(case,'run-test'))

    def test_import_adaptations_are_idempotent_and_inventory_matches(self):
        spec=importlib.util.spec_from_file_location('importer',ROOT/'scripts/india/import-skills.py')
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        for skill in json.loads((ROOT/'docs/india-skills-inventory.json').read_text())['skills']:
            p=ROOT/skill['destination'];body=p.read_text()
            self.assertEqual(module.adapt_reviewed_content(skill['id'],body),body,skill['id'])
            self.assertEqual(module.sha(p.read_bytes()),skill['destination_sha256'])
            for ref,digest in skill.get('reference_sha256',{}).items():
                self.assertEqual(module.sha((p.parent/ref).read_bytes()),digest)

    def test_no_credential_blocks_without_auth_fallback_or_model_calls(self):
        spec=importlib.util.spec_from_file_location('runner',ROOT/'scripts/india/e2e-claude-code.py')
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        state={'rows':[{'id':c['id'],'attempts':[]} for c in cases()]}
        with patch.object(module,'request') as http, patch.object(module,'model_turn') as turn:
            module.run(SimpleNamespace(pat_file=None),Path('/unused'),state)
            http.assert_not_called();turn.assert_not_called()
        self.assertTrue(all(r['status']=='BLOCKED—ADMIN LOGIN' for r in state['rows']))

    def test_command_flags_restrict_local_connectors_and_keep_haiku(self):
        spec=importlib.util.spec_from_file_location('runner',ROOT/'scripts/india/e2e-claude-code.py')
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        with patch.object(module,'cx') as execute:
            module.model_turn('isolated',{'mode':'local'},'input','session',.10)
            argv=execute.call_args.args[1]
            self.assertIn('claude-haiku-4-5-20251001',argv[-1])
            self.assertIn('mcp__*',argv[-1])
            self.assertEqual(argv[-1].count('--disallowedTools'),1)
            self.assertNotIn('bypassPermissions',argv[-1])

    def test_review_cannot_promote_unrun_or_failed_case(self):
        spec=importlib.util.spec_from_file_location('runner',ROOT/'scripts/india/e2e-claude-code.py')
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); review=root/'review.json'
            review.write_text(json.dumps({'ba_tracker_management':{'attempt':1,'checks':[]}}))
            for status in ['NOT RUN','BLOCKED—ADMIN LOGIN','FAIL—SKILL NOT LOADED']:
                state={'rows':[{'id':'ba_tracker_management','status':status,'attempts':[{}]}]}
                with self.assertRaises(RuntimeError): module.apply_review(root,state,review)

    def test_live_review_requires_correlated_mcp_proof(self):
        spec=importlib.util.spec_from_file_location('runner',ROOT/'scripts/india/e2e-claude-code.py')
        module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
        case=next(c for c in cases() if c['id']=='ba_jira_management')
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp); folder=root/'skills'/case['id']/'attempt-01';folder.mkdir(parents=True)
            (folder/'audit.json').write_text('{"status":"UNAVAILABLE"}')
            review=root/'review.json';review.write_text(json.dumps({case['id']:{'attempt':1,'checks':[
                {'criterion':r,'passed':True,'evidence':['audit.json']} for r in case['rubric']]}}))
            state={'rows':[{'id':case['id'],'status':'REVIEW REQUIRED','attempts':[{'content_origin':'live-installer'}]}]}
            with self.assertRaises(RuntimeError): module.apply_review(root,state,review)
