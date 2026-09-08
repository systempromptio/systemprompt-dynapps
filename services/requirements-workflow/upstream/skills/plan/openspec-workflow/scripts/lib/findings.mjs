// Validation-finding framework shared by the functional-artifacts projection (and
// the future ISD sibling). Every finding is actionable and routes the fix UPSTREAM
// so the LLM is never blocked without a way to fix it: it names WHERE, WHAT is
// wrong, the FIX (an upstream edit + re-run), and a DOCS pointer to the governing
// instruction/template. A critical blocks output; a warning prints and proceeds.

// Doc pointers the agent can fetch. The `openspec instructions <artifact>` form is
// named first (path-agnostic on a deployed project); the source template is the fallback.
export const DOCS = {
  proposal:
    'openspec instructions proposal (or .project/templates/openspec/schemas/feature/templates/proposal.md)',
  specs:
    'openspec instructions specs (or .project/templates/openspec/schemas/feature/templates/spec.md)',
  design:
    'openspec instructions design (or .project/templates/openspec/schemas/feature/templates/design.md)',
  fsd: 'openspec instructions fsd (or .project/templates/openspec/schemas/feature/templates/fsd.md)',
  isd: 'openspec instructions isd (or .project/templates/openspec/schemas/feature/templates/isd.md)',
}

export function finding(code, severity, where, problem, fix, docs) {
  return { code, severity, where, problem, fix, docs }
}

// Render an aggregated, human+LLM readable report: errors first (blocking), then
// warnings (non-blocking). `tool` is the CLI name shown in the header.
export function renderFindings(findings, tool = 'generate-functional-artifacts') {
  const errors = findings.filter((f) => f.severity === 'error')
  const warnings = findings.filter((f) => f.severity === 'warning')
  const lines = []
  const block = (list) =>
    list
      .map((f) => `  [${f.code}] ${f.where}\n    ${f.problem}\n    Fix: ${f.fix}\n    See: ${f.docs}`)
      .join('\n\n')
  if (errors.length) {
    lines.push(
      `${tool}: cannot project the artifacts — ${errors.length} blocking issue(s) upstream.`,
      'Fix each upstream (proposal.md / specs), then re-run.\n',
      block(errors),
    )
  }
  if (warnings.length) {
    lines.push(`\n${tool}: ${warnings.length} warning(s) (generation proceeds):\n`, block(warnings))
  }
  return lines.join('\n')
}
