# add-store-locator — Planning Approval (recorded outcome)

The planning approval session is COMPLETE. Every CRITICAL Open Question in `technical-design.md` was approved by a named human, so the **Planning Gate has PASSED** and `tasks.md` may be generated. The `per Q#` deferrals in the technical design are now RESOLVED to the approved answers recorded in the Open Questions below — the decision lives in each question's `Recommendation / Decision` cell, with Status flipped to APPROVED / DEFERRED-APPROVED and the named human in `Approved by`.

## Open Questions (approved)

| Q# | Severity | Question | Recommendation / Decision | Owner | Status | Approved by |
|---|---|---|---|---|---|---|
| Q1 | CRITICAL | Which contract shape do the consumed-capability seams code against before the Cartora and Addressly changes publish their interfaces? | Consumed-capability seams are feature-side adapter modules (`src/lib/store-locator/capabilities.ts`) with the agreed signatures, stubbed until the Cartora and Addressly capabilities deliver; the dependent geocoded-search, map, and autocomplete paths release only after those capabilities deliver (grooming P1/P2). | dev lead / SA | APPROVED | Dane (SA lead) |
| Q2 | NICE-TO-HAVE | What caching should locator store-data reads use in v1? | No custom caching for locator store-data reads in v1 — the search loader reads the commerce Stores API directly; edge caching is revisited when traffic warrants it (`b2c-ecdn`). | dev lead | APPROVED | Dane (SA lead) |
