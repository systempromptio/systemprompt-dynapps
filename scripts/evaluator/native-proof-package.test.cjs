'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const { assess } = require('./native-proof-package.cjs');
function evidence() {
  const scenarios = ['candidate', 'baseline', 'mcp', 'credentials', 'forbidden', 'network', 'failure', 'cancel', 'attempt-bound', 'output-bound', 'deadline', 'judge', 'suggestion'];
  const finals = {}, plans = {};
  const report = { status: 'passed', paid_inference: false, automated_target_enabled: false, platform: 'linux', architecture: 'x86_64', image_id: `sha256:${'a'.repeat(64)}`, provenance: { source_unchanged: true, core_source_digest: 'b'.repeat(64), astound_source_digest: 'c'.repeat(64), exporter_sha256: 'd'.repeat(64), live_exporter_sha256: 'e'.repeat(64) }, cases: [] };
  for (const scenario of scenarios) {
    const final = { kind: 'finalized', stale_lease_denied: true, revoked_token_denied: true, accounting_failure: { verified: true, journaled_failure_replayed_without_extra_spend: true }, boundaries: ['forged-token', 'forged-session', 'output-bound', 'unknown-pricing', 'cost-bound'].map(boundary => ({ boundary, rejected_without_reservation: true })) };
    finals[scenario] = final;
    plans[scenario] = { client: 'codex', client_version: '1.2.3', adapter_version: 'adapter-1', limits: { max_artifact_bytes: 65536, max_turns: scenario === 'attempt-bound' ? 1 : 4 } };
    report.cases.push({ raw_version_sha256: '8'.repeat(64), supervision: { primitive: 'ContainerExecution', cancelled: scenario === 'cancel', output_bound_rejected: scenario === 'output-bound', deadline_rejected: scenario === 'deadline', cleanup_acknowledged: true, observed_output_bytes: 65537, configured_output_bytes: 65536 }, client: 'codex', scenario, status: 'passed', cleanup_confirmed: true, live_gateway_cleanup_confirmed: true, runtime_user_matches_linux_container_launch: true, image_id: report.image_id, executable_sha256: 'f'.repeat(64), client_version: '1.2.3', environment_digest: '1'.repeat(64), materialized_environment_digest: '2'.repeat(64), live_gateway: final, live_execution_reservation_verified: true, live_metered_requests: 1, live_unreserved_attempts: 0, native_usage_parity_verified: true, provider_attempts: 1, cancelled: scenario === 'cancel', normalized: { text: '', tool_calls: [], reported_input_tokens: 11, reported_output_tokens: 7, completion: ['cancel', 'failure', 'deadline'].includes(scenario) ? 'failed' : 'completed' }, output_limit_exceeded: scenario === 'output-bound' });
  }
  return { report, plans, finals };
}
test('complete proof inputs produce the exact binding without registering a target', () => {
  const { report, plans, finals } = evidence();
  assert.equal(assess(report, 'codex', plans, finals).binding.executable_digest, 'f'.repeat(64));
});
for (const [name, mutate] of [
  ['interrupted run', e => { e.report.status = 'interrupted'; }],
  ['missing scenario', e => { e.report.cases.pop(); }],
  ['duplicate scenario', e => { e.report.cases[1].scenario = 'candidate'; }],
  ['source drift', e => { e.report.provenance.source_unchanged = false; }],
  ['fixture-only gateway', e => { e.report.provenance.live_exporter_sha256 = null; }],
  ['mixed image', e => { e.report.cases[2].image_id = `sha256:${'b'.repeat(64)}`; }],
  ['mixed adapter', e => { e.plans.judge.adapter_version = 'other'; }],
  ['unmatched baseline', e => { e.report.cases[1].materialized_environment_digest = '3'.repeat(64); }],
  ['missing pricing rejection', e => { e.finals.candidate.boundaries.pop(); }],
  ['cancelled success', e => { e.report.cases.find(c => c.scenario === 'cancel').normalized.completion = 'completed'; }],
  ['unreserved native attempt', e => { e.report.cases[0].live_unreserved_attempts = 1; }],
  ['harness-only overflow', e => { delete e.report.cases.find(c => c.scenario === 'output-bound').supervision; }],
  ['overflow not exercised', e => { e.report.cases.find(c => c.scenario === 'output-bound').supervision.observed_output_bytes = 65536; }],
  ['cleanup failure', e => { e.report.cases[0].cleanup_confirmed = false; }],
  ['no native parity', e => { e.report.cases.forEach(c => { c.native_usage_parity_verified = false; }); }],
  ['no settlement replay', e => { Object.values(e.finals).forEach(f => { f.accounting_failure.journaled_failure_replayed_without_extra_spend = false; }); }],
]) test(`proof rejects ${name}`, () => {
  const e = evidence(); mutate(e); assert.throws(() => assess(e.report, 'codex', e.plans, e.finals));
});

for (const [client, serialized] of [['claude-code', 'claude-code'], ['open-code', 'opencode'], ['codex', 'codex'], ['hermes', 'hermes']]) test(`uses canonical ${client} adapter serialization`, () => {
  const e = evidence();
  for (const entry of e.report.cases) entry.client = client;
  for (const plan of Object.values(e.plans)) plan.client = serialized;
  assert.equal(assess(e.report, client, e.plans, e.finals).binding.client, serialized);
});
test('missing normalized failure evidence cannot count as incomplete execution', () => {
  const e = evidence(); delete e.report.cases.find(c => c.scenario === 'failure').normalized;
  assert.throws(() => assess(e.report, 'codex', e.plans, e.finals));
});
