'use strict';
const fs = require('node:fs');
const path = require('node:path');
const { hash, encode } = require('./native-proof-provenance.cjs');
const scenarios = ['candidate', 'baseline', 'mcp', 'credentials', 'forbidden', 'network', 'failure', 'cancel', 'attempt-bound', 'output-bound', 'deadline', 'judge', 'suggestion'];
const check = (condition, message) => { if (!condition) throw Error(message); };
const sha = value => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value);
function read(filename, maximum = 32 * 1024 * 1024) {
  const descriptor = fs.openSync(filename, fs.constants.O_RDONLY | fs.constants.O_NOFOLLOW);
  try { const stat = fs.fstatSync(descriptor); check(stat.isFile() && stat.size > 0 && stat.size <= maximum, 'Invalid bounded evidence file'); return fs.readFileSync(descriptor); }
  finally { fs.closeSync(descriptor); }
}
function assess(report, client, plans, finals) {
  check(report.status === 'passed' && report.paid_inference === false && report.automated_target_enabled === false, 'Full native run did not pass or unexpectedly enabled a target');
  check(report.provenance?.source_unchanged === true && sha(report.provenance.core_source_digest) && sha(report.provenance.astound_source_digest) && sha(report.provenance.exporter_sha256) && sha(report.provenance.live_exporter_sha256), 'Missing stable sources or actual live exporter provenance');
  check(report.platform === 'linux' && ['x86_64', 'aarch64'].includes(report.architecture), 'Unsupported observed native platform');
  check(/^sha256:[a-f0-9]{64}$/.test(report.image_id), 'Missing immutable image config ID');
  const cases = report.cases.filter(entry => entry.client === client);
  check(cases.length === scenarios.length && scenarios.every(s => cases.filter(e => e.scenario === s).length === 1), 'Missing or duplicate native scenarios');
  const first = cases.find(e => e.scenario === 'candidate');
  const baseline = cases.find(e => e.scenario === 'baseline');
  check(first.environment_digest === baseline.environment_digest && first.materialized_environment_digest === baseline.materialized_environment_digest && sha(first.environment_digest) && sha(first.materialized_environment_digest), 'Baseline/candidate environments differ');
  check(sha(first.executable_sha256) && typeof first.client_version === 'string' && first.client_version.trim() && first.client_version.length <= 128 && !/[\x00-\x1f\x7f-\x9f]/.test(first.client_version), 'Missing executable verification');
  let parity = false, failedSpend = false, settlement = false;
  const sanitized = [];
  for (const entry of cases) {
    const plan = plans[entry.scenario], final = finals[entry.scenario];
    check(entry.status === 'passed' && entry.cleanup_confirmed === true && entry.live_gateway_cleanup_confirmed === true && entry.runtime_user_matches_linux_container_launch === true, `Incomplete native/cleanup contract: ${entry.scenario}`);
    check(sha(entry.raw_version_sha256) && entry.image_id === report.image_id && entry.executable_sha256 === first.executable_sha256 && entry.client_version === first.client_version, 'Mixed native executable/image identity');
    check(plan && plan.client === (client === 'open-code' ? 'opencode' : client) && plan.client_version === first.client_version.trim() && plan.adapter_version === plans.candidate.adapter_version, 'Mixed plan/client/adapter version');
    check(final?.kind === 'finalized' && final.stale_lease_denied === true && final.revoked_token_denied === true && JSON.stringify(final) === JSON.stringify(entry.live_gateway), 'Missing/mismatched actual live finalization');
    check(entry.live_execution_reservation_verified === true && entry.live_metered_requests > 0 && entry.live_unreserved_attempts === 0, 'Native calls lack authenticated atomic reservations');
    for (const boundary of ['forged-token', 'forged-session', 'output-bound', 'unknown-pricing', 'cost-bound']) {
      check(final.boundaries?.filter(p => p.boundary === boundary && p.rejected_without_reservation === true).length === 1, `Missing actual gateway boundary probe: ${boundary}`);
    }
    if (entry.scenario === 'cancel') check(entry.supervision?.primitive === 'ContainerExecution' && entry.supervision.cancelled === true && entry.supervision.cleanup_acknowledged === true && entry.cancelled === true && entry.normalized?.completion !== 'completed', 'Cancellation lacks incomplete evidence');
    if (entry.scenario === 'failure') check(entry.normalized?.completion !== 'completed', 'Failed native execution claimed success');
    if (entry.scenario === 'attempt-bound') check(entry.provider_attempts === 1 && plan.limits.max_turns === 1, 'Native attempt bound not exercised');
    if (entry.scenario === 'output-bound') {
      check(entry.supervision?.primitive === 'ContainerExecution' && entry.supervision.output_bound_rejected === true && entry.supervision.cleanup_acknowledged === true && entry.supervision.observed_output_bytes > entry.supervision.configured_output_bytes && entry.supervision.configured_output_bytes === plan.limits.max_artifact_bytes && entry.output_limit_exceeded === true, 'Native overflow was not rejected by production container supervision');
    } else check(entry.output_limit_exceeded === false, 'Unexpected native output overflow');
    if (entry.scenario === 'deadline') check(entry.supervision?.primitive === 'ContainerExecution' && entry.supervision.deadline_rejected === true && entry.supervision.cleanup_acknowledged === true && entry.normalized?.completion !== 'completed', 'Native deadline lacks production container supervision');
    parity ||= entry.native_usage_parity_verified === true;
    failedSpend ||= final.accounting_failure?.verified === true;
    settlement ||= final.accounting_failure?.journaled_failure_replayed_without_extra_spend === true;
    sanitized.push({ supervision: entry.supervision, raw_version_sha256: entry.raw_version_sha256, scenario: entry.scenario, status: entry.status, cleanup_confirmed: entry.cleanup_confirmed, live_gateway_cleanup_confirmed: entry.live_gateway_cleanup_confirmed, provider_attempts: entry.provider_attempts, executable_sha256: entry.executable_sha256, environment_digest: entry.environment_digest, materialized_environment_digest: entry.materialized_environment_digest, native_usage_parity_verified: entry.native_usage_parity_verified === true, cancelled: entry.cancelled, timed_out: entry.timed_out, harness_deadline_enforced: entry.harness_deadline_enforced === true, output_limit_exceeded: entry.output_limit_exceeded, normalized_completion: entry.normalized?.completion, metered_requests: entry.live_metered_requests, unreserved_attempts: entry.live_unreserved_attempts, stale_lease_denied: final.stale_lease_denied, revoked_token_denied: final.revoked_token_denied, boundaries: final.boundaries.map(({ boundary, rejected_without_reservation }) => ({ boundary, rejected_without_reservation })), failed_spend_verified: final.accounting_failure?.verified === true, idempotent_settlement_verified: final.accounting_failure?.journaled_failure_replayed_without_extra_spend === true });
  }
  check(parity && failedSpend && settlement, 'Missing native token parity, retained failed spend, or repeated settlement evidence');
  const adapter = plans.candidate.adapter_version;
  check(typeof adapter === 'string' && adapter.length > 0 && adapter.length <= 128 && !/[\x00-\x1f\x7f-\x9f]/.test(adapter), 'Missing bounded adapter version');
  return { binding: { client: client === 'open-code' ? 'opencode' : client, platform: report.platform, architecture: report.architecture, client_version: first.client_version.trim(), adapter_version: adapter, image_config_digest: report.image_id.slice(7), executable_digest: first.executable_sha256, core_source_digest: report.provenance.core_source_digest, astound_source_digest: report.provenance.astound_source_digest }, sanitized };
}
function packageProof(directory, client, output) {
  check(['claude-code', 'open-code', 'codex', 'hermes'].includes(client), 'Unknown CLI client');
  const rawReport = read(path.join(directory, 'report.json'));
  const report = JSON.parse(rawReport), plans = {}, finals = {};
  for (const scenario of scenarios) {
    const base = path.join(directory, `${client}-${scenario}`);
    plans[scenario] = JSON.parse(read(path.join(base, 'plan.json'), 1048576));
    finals[scenario] = JSON.parse(read(path.join(base, 'evidence/live-gateway-final.json')));
    const entry = report.cases.find(e => e.client === client && e.scenario === scenario);
    check(entry && hash(read(path.join(base, 'version.stdout'), 65536)) === entry.raw_version_sha256, 'Raw executable version evidence mismatch');
    if (['cancel', 'output-bound', 'deadline'].includes(scenario)) check(JSON.stringify(JSON.parse(read(path.join(base, 'supervision.json'), 65536))) === JSON.stringify(entry.supervision), 'Runtime supervision artifact mismatch');
  }
  const { binding, sanitized } = assess(report, client, plans, finals);
  const artifacts = {};
  for (const name of ['core', 'astound']) {
    const bytes = read(path.join(directory, `${name}-source.json`));
    check(hash(bytes) === binding[`${name}_source_digest`], 'Source manifest digest mismatch');
    const source = JSON.parse(bytes);
    check(source.schema_version === 1 && Array.isArray(source.files) && source.files.length > 0, 'Invalid source manifest');
    artifacts[`${name}-source.json`] = bytes;
  }
  artifacts['native-evidence.json'] = encode({ schema_version: 1, original_report_sha256: hash(rawReport), provenance: report.provenance, contracts: sanitized, limitations: ['Cancellation, deadline and deliberate output overflow use production ContainerLaunch/ContainerExecution primitives, with fixture-only preverified admission. This does not claim the complete production scheduler lifecycle.', 'Sources include tracked and unignored untracked content; exporter binary digests are retained separately.'] });
  const artifactHashes = Object.fromEntries(Object.entries(artifacts).map(([name, bytes]) => [name, hash(bytes)]));
  const isolation = encode({ schema_version: 1, binding, contracts: Object.fromEntries(['pinned_executable', 'configuration_credentials', 'forbidden_tools', 'network', 'cancellation_cleanup', 'attempt_bound', 'output_bound', 'failed_evidence', 'matched_environments', 'judge', 'suggestion'].map(key => [key, true])), artifacts: artifactHashes });
  const metering = encode({ schema_version: 1, binding, contracts: Object.fromEntries(['authenticated_execution_session', 'atomic_reservation', 'native_usage_parity', 'failed_spend', 'unknown_pricing', 'attempt_output_cost_bounds', 'stale_lease', 'revocation', 'idempotent_settlement'].map(key => [key, true])), artifacts: artifactHashes });
  let repositoryDigest = null;
  if (report.image_reference !== report.image_id) {
    check(typeof report.image_reference === 'string' && /^[A-Za-z0-9._/:\-]+@sha256:[a-f0-9]{64}$/.test(report.image_reference) && report.repository_digests.includes(report.image_reference), 'Mutable/unproven image reference; rerun with exact image config ID');
    repositoryDigest = report.image_reference.split('@sha256:')[1];
  }
  const target = { client: binding.client, platform: binding.platform, architecture: binding.architecture, client_version: binding.client_version, adapter_version: binding.adapter_version, image_digest: repositoryDigest || binding.image_config_digest, executable_digest: binding.executable_digest, native_isolation_evidence_digest: hash(isolation), native_metering_evidence_digest: hash(metering) };
  const manifest = encode({ schema_version: 1, target, binding, repository_manifest_digest: repositoryDigest });
  for (const bytes of [isolation, metering, manifest]) check(bytes.length <= 1048576, 'Proof document exceeds registry bound');
  // Create only after every contract is established; never overwrite a retained proof.
  fs.mkdirSync(output, { mode: 0o700 });
  for (const [name, bytes] of Object.entries({ ...artifacts, 'isolation.json': isolation, 'metering.json': metering, 'manifest.json': manifest, 'manifest.sha256': Buffer.from(hash(manifest) + '\n') })) fs.writeFileSync(path.join(output, name), bytes, { flag: 'wx', mode: 0o600 });
  return hash(manifest);
}
if (require.main === module) {
  try { const args = process.argv.slice(2); check(args.length === 3, 'Usage: node native-proof-package.cjs REPORT_DIRECTORY CLIENT NEW_OUTPUT_DIRECTORY'); process.stdout.write(packageProof(path.resolve(args[0]), args[1], path.resolve(args[2])) + '\n'); }
  catch (error) { process.stderr.write(`Native proof not packaged: ${error.message}\n`); process.exitCode = 1; }
}
module.exports = { assess, packageProof };
