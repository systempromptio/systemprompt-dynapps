// ISD document model — the INTEGRATION binding of the shared config-driven
// requirement-block section module (reqblocks.mjs). The ISD read-only trio is
// [User Story, Acceptance Criteria, Key behaviours] with NO Visualization (an ISD
// is a contract, not a UI document), so the canonical order is exactly the trio.
// Like the FSD, the ISD script bootstraps the WHOLE document from the isd.md
// template on first run and reconciles the in-scope region in place afterwards;
// the section sits directly after "## Integration Specification", so when an
// existing ISD lacks it (e.g. extending a legacy page) it is inserted before
// "## Out Of Scope / Limitations".

import { makeReqBlocks } from './reqblocks.mjs'

const isd = makeReqBlocks({
  readOnly: ['User Story', 'Acceptance Criteria', 'Key behaviours'],
  canonicalOrder: ['User Story', 'Acceptance Criteria', 'Key behaviours'],
  sectionRe: /^##\s+In Scope Functional Requirements\s*$/,
  sectionMatchRe: /in scope functional requirements/i,
  titleSuffixRe: /\s+ISD\s*$/,
  insertBefore: '## Out Of Scope / Limitations',
  label: 'isd',
})

export const READ_ONLY = isd.READ_ONLY
export const canonRank = isd.canonRank
export const canonName = isd.canonName
export const projectedBodies = isd.projectedBodies
export const parseIsdDoc = isd.parseDoc
export const splitIsdTemplate = isd.splitTemplate
export const fillTemplateBlock = isd.fillTemplateBlock
export const bootstrap = isd.bootstrap
export const surgicalUpdate = isd.surgicalUpdate
