// Deterministic assembly of the generation and judge prompts.
//
// Two builders share the same block format so a prompt can be fed to Bedrock or
// handed to a Cursor subagent (via --prompt-only), keeping runners in parity.
//
// - buildGenerationPrompt: the generation contract/procedure + CONTEXT
//   (upstream/initial) + inlined skills/templates + the generator task. It MUST
//   NOT include the golden expected artifact.
// - buildJudgePrompt: the judge rubric (contract + rubric) + inlined sources +
//   CONTEXT (grounding) + optional GOLDEN EXPECTED reference + the CANDIDATE.

function block(title, content) {
  return `\n===== ${title} =====\n${String(content).trim()}\n`;
}

function sourcesBlock(sources) {
  let text = '';
  for (const s of sources) {
    if (s.exists) text += block(`SOURCE: ${s.ref}`, s.content) + substitutionNotes(s.substitutions);
    else text += `\n===== SOURCE: ${s.ref} (NOT FOUND — skipped) =====\n`;
  }
  return text || '\n(no sources declared)\n';
}

// A context id under `.context/` is a raw initial-requirements source; any other
// id (e.g. `discovery`) is an upstream phase artifact whose dispositions are
// authoritative. `labelKind` annotates each block with that role so the eval
// model reads discovery as a produced upstream artifact, not loose text. It is
// opt-in (generation only) so the judge prompt stays byte-identical.
function contextKind(id) {
  return String(id).startsWith('.context/') ? 'initial requirements source' : 'upstream phase artifact';
}

function contextBlock(contextBlocks, { labelKind = false } = {}) {
  let text = '';
  for (const c of contextBlocks) {
    const title = labelKind ? `CONTEXT [${contextKind(c.id)}]: ${c.id}` : `CONTEXT: ${c.id}`;
    if (c.found) text += block(title, c.content);
    else text += `\n===== ${title} (NOT FOUND — skipped) =====\n`;
  }
  return text || '\n(no context declared)\n';
}

// Point-of-use role captions. The full definitions live in the generation
// contract (Input Block Model); these one-line leads reinforce each block's role
// right where the model reads it, because a real Cursor session supplies this
// framing implicitly and the eval model does not get it otherwise.
const SKILLS_LEAD =
  'Role: HOW-TO capability references for this stack, not facts and not decisions. ' +
  'Consult a skill only for a concern the solution actually has (grounded in CONTEXT). ' +
  'A skill being present never justifies adding a decision or a Cross-Cutting row.';
const RULES_LEAD =
  'Role: authoring/process discipline (how the artifact is produced), not facts about this case.';
const CONTEXT_LEAD =
  'Role: the ONLY ground truth. Upstream phase artifacts (e.g. discovery — the ' +
  'reconciled WHAT with authoritative dispositions, read not re-decided) and the ' +
  'initial requirements packet. Every fact, identifier, and decision must trace here.';

/** Render substitution notes attached to a mapped block (see references.mjs). */
function substitutionNotes(subs) {
  if (!subs || !subs.length) return '';
  const lines = subs
    .map((s) => `- [${s.id}] ${String(s.note).trim()}${s.redirectContext ? ` (served by CONTEXT: ${s.redirectContext})` : ''}`)
    .join('\n');
  return block('LIVE-REFERENCE SUBSTITUTIONS (apply to the block above)', lines);
}

/** Render a mapped asset (command / skill / schema instruction) + its substitutions. */
function mappedBlock(title, item) {
  if (!item) return '';
  if (item.exists === false) {
    return `\n===== ${title}${item.ref ? `: ${item.ref}` : ''} (NOT FOUND — skipped) =====\n`;
  }
  return block(`${title}${item.ref ? `: ${item.ref}` : ''}`, item.content) + substitutionNotes(item.substitutions);
}

/**
 * Render the compiled OpenSpec instructions (the text-rendered equivalent of
 * `openspec instructions <artifact> --json`) plus the neutralized live refs.
 */
function openSpecInstructionsBlock(schemaArtifact) {
  if (!schemaArtifact) return '';
  if (schemaArtifact.exists === false) {
    return `\n===== OPENSPEC INSTRUCTIONS: ${schemaArtifact.ref} (NOT RESOLVED: ${schemaArtifact.error ?? 'unknown'}) =====\n`;
  }
  const title = `OPENSPEC INSTRUCTIONS (compiled from openspec CLI): ${schemaArtifact.schemaName}/${schemaArtifact.artifactId}`;
  return block(title, schemaArtifact.rendered) + substitutionNotes(schemaArtifact.substitutions);
}

function skillsBlock(skills) {
  const list = Array.isArray(skills) ? skills : [];
  if (!list.length) return '';
  return list.map((s) => mappedBlock('SKILL', s)).join('');
}

function rulesBlock(rules) {
  const list = Array.isArray(rules) ? rules : [];
  if (!list.length) return '';
  return list.map((r) => mappedBlock('RULE', r)).join('');
}

const GENERATION_SYSTEM = `You are an OpenSpec artifact generator. You produce exactly one artifact document.

The generation contract and output procedure are given at the end of this system
message. Follow them exactly. The artifact-specific structure, section order, and
semantic rules are provided in the OPENSPEC INSTRUCTIONS block in the user message
(the text-rendered equivalent of \`openspec instructions <artifact>\`).`;

const JUDGE_SYSTEM = `You are a strict, evidence-driven evaluator (LLM-as-a-judge) for OpenSpec artifacts.

You are given up to five blocks:
1. STANDARD — the canonical contract and scoring rubric for this artifact type.
2. SOURCES — the authoritative rules/templates the STANDARD depends on (verbatim).
3. CONTEXT — the grounding the artifact must derive from (upstream artifacts or the initial requirements source). Traceability and no-fabrication are judged against this.
4. GOLDEN EXPECTED — an optional reference example of a good artifact for this case. Treat it as guidance on shape/coverage, not as the only acceptable wording.
5. CANDIDATE — the actual artifact produced by the system, which you must judge.

Judge the CANDIDATE only against the STANDARD, SOURCES, and CONTEXT. Do not invent
requirements that are not grounded in those blocks. Prefer concrete evidence
(quote the offending or missing text) over vague impressions. This is an
open-ended document, so evaluate meaning, structure, traceability, and scope
hygiene — not exact wording.

Every rubric dimension listed in the STANDARD MUST have a numeric score. Do not
use null, "n/a", "N/A", or omit a dimension when CONTEXT is present. If a
dimension is uncertain or partially supported, assign a numeric score such as
0.5 or 0.75 and explain the uncertainty in the reason. Only if the required
CONTEXT block is explicitly marked NOT FOUND may you use null for that
dimension.

Where the STANDARD defines a closed-rule gate or sweep — a numbered procedure to
run BEFORE scoring a dimension — you MUST actually run it and record its working
in the verdict's \`gates\` array, one entry per gate. Record the items the gate's
enumeration step produced, the items an exclusion step removed together with the
step that removed them, and the items that reached the firing step. List them;
do not summarise them as a count or a closure sentence.

Every entry in \`critical_findings\` MUST correspond to an item you listed in some
gate's \`survived\` array. A finding you cannot trace to a survived item is not a
critical — either you skipped the enumeration that would have produced it, or an
exclusion step already removed it.

The reverse also holds: a survived item is one the gate's exclusions did NOT
remove, so every survived item MUST carry a disposition — \`critical\` (and it then
appears in \`critical_findings\`), \`warning\`, \`missing_evidence\`, or \`no_breach\`
with a note saying why it reached the firing step without breaching. Never leave a
survived item undispositioned, and never shorten an enumeration or move an item to
\`excluded\` merely to avoid dispositioning it — an exclusion is valid only when a
named step of the gate removes the item.

The output procedure — how to reason first and then emit the verdict — is given
in the OUTPUT PROCEDURE block at the end of this message. Follow it exactly. The
verdict itself is EXACTLY ONE JSON object with this schema:
{
  "pass": boolean,                       // overall pass/fail vs the rubric threshold
  "score": number,                       // overall 0.0..1.0
  "gates": [                             // one entry per closed-rule gate/sweep in the STANDARD
    {
      "gate": string,                    // the gate's name, as the STANDARD calls it
      "enumerated": [ string ],          // every item the enumeration step produced, in full
      "excluded": [                      // items an exclusion step removed, and which step
        { "item": string, "rule": string }
      ],
      "survived": [                      // items that reached the firing step, each dispositioned
        { "item": string, "disposition": "critical" | "warning" | "missing_evidence" | "no_breach", "note": string }
      ]
    }
  ],
  "rubric_scores": {                     // one entry per rubric dimension in the STANDARD
    "<dimension>": { "score": number, "reason": string } // numeric score; never "n/a"
  },
  "critical_findings": [                 // must-fix issues that block acceptance
    { "issue": string, "evidence": string }
  ],
  "warnings": [                          // non-blocking issues worth fixing
    { "issue": string, "evidence": string }
  ],
  "missing_evidence": [ string ],        // expected content not found in the candidate
  "recommendation": string               // one-line verdict + next action
}`;

/**
 * Build the generation prompt (no golden expected).
 * @param {object} args { schema, artifact, generationContract, genReasoning, scenarioBody, scenarioSources, contextBlocks }
 * @returns {{ system, user }}
 */
export function buildGenerationPrompt(args) {
  const {
    schema,
    artifact,
    generationContract,
    genReasoning,
    scenarioBody,
    scenarioSources,
    contextBlocks,
    command,
    skills,
    rules,
    schemaArtifact,
  } = args;
  const task = scenarioBody?.trim()
    ? scenarioBody
    : `Generate a valid OpenSpec "${artifact}" artifact for the "${schema}" schema, following the COMMAND, SKILLS, RULES, OPENSPEC INSTRUCTIONS, and SOURCES, grounded in the CONTEXT.`;
  const contract = String(generationContract ?? '').trim();
  const planningPass = String(genReasoning ?? '').trim();
  const planningBlock = planningPass ? `\n\nPlanning pass to perform:\n\n${planningPass}` : '';
  const procedure =
    `First emit one <plan> ... </plan> block performing the planning pass below. ` +
    `After </plan>, output ONLY the finished artifact as Markdown: no preamble, ` +
    `no explanation, no code fence around the whole document, and nothing after ` +
    `the artifact.${planningBlock}`;
  const system =
    GENERATION_SYSTEM +
    (contract ? block('CONTRACT (generation standard)', contract) : '') +
    block('OUTPUT PROCEDURE', procedure);

  // Block order is placement-sensitive. CONTEXT (the grounding the artifact must
  // derive from) is emitted LAST so the ground truth is the most recent, highest
  // salience material before generation — this protects grounding/no-fabrication,
  // the primary failure axis. The artifact contract (OPENSPEC INSTRUCTIONS) sits
  // just before CONTEXT; supporting/bulk material (command, skills, rules,
  // sources) is in the center. (Measured: emitting the schema LAST instead
  // improved decision polish but worsened fabrication/approval-discipline on both
  // flows and dropped 10/20 -> 7/20, so grounding-last wins.)
  const user =
    `Generate an OpenSpec "${artifact}" artifact for the "${schema}" schema.\n` +
    block('TASK', task) +
    mappedBlock('COMMAND (real workflow command)', command) +
    (skills && skills.length ? block('SKILLS (capability references — not facts, not decisions)', `${SKILLS_LEAD}\n${skillsBlock(skills)}`) : '') +
    (rules && rules.length ? block('RULES (authoring/process discipline — not facts)', `${RULES_LEAD}\n${rulesBlock(rules)}`) : '') +
    (schemaArtifact ? openSpecInstructionsBlock(schemaArtifact) : '') +
    (scenarioSources && scenarioSources.length ? block('SOURCES (additional templates / rules)', sourcesBlock(scenarioSources)) : '') +
    block('CONTEXT (grounding — the only ground truth)', `${CONTEXT_LEAD}\n${contextBlock(contextBlocks ?? [], { labelKind: true })}`) +
    `\nNow follow the OUTPUT PROCEDURE in the system message: the <plan> planning pass, then the finished ${artifact} artifact as Markdown.`;

  return { system, user };
}

/**
 * Build the judge prompt.
 * @param {object} args { schema, artifact, judgeBody, judgeSources, contextBlocks, golden, candidate, passThreshold }
 * @returns {{ system, user }}
 */
export function buildJudgePrompt(args) {
  const {
    schema,
    artifact,
    judgeBody,
    judgeSources,
    contextBlocks,
    golden,
    candidate,
    passThreshold,
    judgeReasoning,
    structureFacts,
  } = args;

  // Reasoning is an injectable, artifact-agnostic routing-pass spec. It is
  // appended as the OUTPUT PROCEDURE: the judge emits a <reasoning> routing pass,
  // then the fenced JSON verdict, in one response. Artifact-specific routing
  // detail lives in each judge contract (STANDARD).
  const routingPass = String(judgeReasoning ?? '').trim();
  const routingBlock = routingPass ? `\n\nRouting pass to perform:\n\n${routingPass}` : '';
  const procedure =
    `First emit one <reasoning> … </reasoning> block performing the routing pass below. ` +
    `After </reasoning>, output EXACTLY ONE JSON verdict object wrapped in a single ` +
    `\`\`\`json code fence, with nothing after the closing fence.${routingBlock}`;
  const system = `${JUDGE_SYSTEM}\n\n===== OUTPUT PROCEDURE =====\n${procedure}`;

  const goldenText =
    golden && golden.found
      ? block('GOLDEN EXPECTED (reference for this case)', golden.content)
      : '';

  // Optional deterministic presence facts (harness-computed, schema-agnostic:
  // emitted only when the resolved artifact opts in and the parse found sets).
  // Placed immediately before CANDIDATE so the authoritative presence lists frame
  // how the judge reads the candidate — closing the judge's hallucinated-absence
  // class without a model change.
  const factsText = String(structureFacts ?? '').trim()
    ? block('DERIVED STRUCTURE FACTS (deterministic — authoritative for presence)', structureFacts)
    : '';

  const user =
    `Evaluate a "${artifact}" artifact for the "${schema}" schema.\n` +
    `Pass threshold on overall score: ${passThreshold}.\n` +
    block('STANDARD (contract + rubric)', judgeBody) +
    block('SOURCES (authoritative rules/templates)', sourcesBlock(judgeSources ?? [])) +
    block('CONTEXT (grounding — traceability is judged against this)', contextBlock(contextBlocks ?? [])) +
    goldenText +
    factsText +
    block('CANDIDATE (artifact under evaluation)', candidate) +
    `\nNow follow the OUTPUT PROCEDURE in the system message: the <reasoning> routing pass, then the single fenced JSON verdict.`;

  return { system, user };
}
