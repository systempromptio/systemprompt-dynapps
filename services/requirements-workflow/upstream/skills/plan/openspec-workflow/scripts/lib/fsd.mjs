// FSD document model — the FUNCTIONAL binding of the shared config-driven
// requirement-block section module (reqblocks.mjs). The FSD read-only trio is
// [User Story, Acceptance Criteria, Key behaviours]; Visualization is LLM-owned and
// sits between the story and the criteria in the canonical order. The FSD script
// bootstraps the WHOLE document from the fsd.md template and reconciles the whole
// in-scope region in place. This wrapper preserves the historical export names so
// the functional-artifacts script + test keep importing from ./lib/fsd.mjs.

import { makeReqBlocks } from './reqblocks.mjs'

// The three script-owned sub-sections + their canonical order among all block
// sub-sections (Visualization is LLM-owned but sits between them).
const fsd = makeReqBlocks({
  readOnly: ['User Story', 'Acceptance Criteria', 'Key behaviours'],
  canonicalOrder: ['User Story', 'Visualization', 'Acceptance Criteria', 'Key behaviours'],
  sectionRe: /^##\s+In Scope Functional Requirements\s*$/,
  sectionMatchRe: /in scope functional requirements/i,
  titleSuffixRe: /\s+FSD\s*$/,
  label: 'fsd',
})

export const READ_ONLY = fsd.READ_ONLY
export const canonRank = fsd.canonRank
export const canonName = fsd.canonName
export const projectedBodies = fsd.projectedBodies
export const parseFsdDoc = fsd.parseDoc
export const splitFsdTemplate = fsd.splitTemplate
export const fillTemplateBlock = fsd.fillTemplateBlock
export const bootstrap = fsd.bootstrap
export const surgicalUpdate = fsd.surgicalUpdate
