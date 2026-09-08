// Deterministic renderability gate for the drawio-diagrams judge.
//
// The LLM judge scores wording and scope but is unreliable at simulating a
// sequence's call stack, so an unrenderable spec (e.g. an unclosed `call`, R2)
// could still score as "fabrication-free". This adapter closes that gap the same
// way the deterministic structure gate does for templated artifacts: it answers
// only "would this actually render?" — content correctness stays the judge's job.
//
// It reuses the SKILL's OWN spec validator (the exact rules `generate.mjs`
// enforces before it writes any file), so the gate and the real pipeline can
// never disagree. It couples to the skill by relative path — like
// render-candidates.mjs in this same folder — which is fine here: this module is
// evalsuite glue, NOT part of the schema-agnostic harness in lib/**. The harness
// only knows "run the validator the schema declared and call validateArtifact".

import { extractDrawioBlocks, validateSpec, parseYaml } from '../../../../../skills/plan/drawio-diagrams/scripts/lib/spec.mjs'

/**
 * Validate a candidate artifact for renderability by construction.
 *
 * Every case asks for a diagram, so a candidate must hold exactly one
 * ` ```drawio:<type>:<id> ` block; a candidate with no block answered something
 * other than the task and fails the gate. Each block is reconstructed exactly as
 * generate.mjs would (header carries type+id, body is the scenario) and run through
 * the skill's validateSpec; any rule violation is a finding, so the gate fails the
 * run.
 *
 * @param {string} candidateMd the stripped candidate markdown the judge scored
 * @returns {{ ok: boolean, findings: Array<{ issue: string, evidence?: string }> }}
 */
export function validateArtifact(candidateMd) {
  const blocks = extractDrawioBlocks(String(candidateMd ?? ''))
  if (!blocks.length) {
    return {
      ok: false,
      findings: [{
        issue: 'The candidate carries no ` ```drawio:<type>:<id> ` block, so there is no diagram to render',
        evidence: 'the ask names a diagram type; a prose-only answer does not satisfy it',
      }],
    }
  }

  const findings = []
  for (const { type, id, body } of blocks) {
    const label = id ? `"${id}"` : '(no id)'
    let spec
    try {
      spec = { type, id, ...(parseYaml(body) || {}) }
    } catch (err) {
      findings.push({
        issue: `Diagram ${label} (${type || 'no type'}) is not renderable: spec body is not valid YAML`,
        evidence: String(err?.message ?? err).slice(0, 200),
      })
      continue
    }
    for (const e of validateSpec(spec)) {
      findings.push({
        issue: `Diagram ${label} (${type || 'no type'}) is not renderable: ${e}`,
        evidence: 'violates a drawio spec rule (same check generate.mjs runs before writing files)',
      })
    }
  }
  return { ok: findings.length === 0, findings }
}
