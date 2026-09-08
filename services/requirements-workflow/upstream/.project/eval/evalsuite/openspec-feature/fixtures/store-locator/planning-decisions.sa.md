# add-cartora-maps-integration — Planning Approval (recorded outcome)

The planning approval session is COMPLETE. Every CRITICAL Open Question in `technical-design.md` was approved by a named human, so the **Planning Gate has PASSED** and `tasks.md` may be generated. The `per Q#` deferrals in the technical design are now RESOLVED to the approved answers recorded in the Open Questions below — the decision lives in each question's `Recommendation / Decision` cell, with Status flipped to APPROVED / DEFERRED-APPROVED and the named human in `Approved by`.

## Open Questions (approved)

| Q# | Severity | Question | Recommendation / Decision | Owner | Status | Approved by |
|---|---|---|---|---|---|---|
| Q1 | CRITICAL | What is the consumer-facing contract of the geocode interface (endpoint namespace/path and response schema)? | The geocode capability's consumer-facing contract is a custom SCAPI API `cartora-geo` v1 with `GET /geocode`; request params `q`, `country`, `limit`; response `results[]` each `{ coordinates: { lat, lng }, proximityRanking }`; error result `{ error: { type, message } }` carrying no coordinates. Consumers and tests build against this contract. | dev lead / SA | APPROVED | Dane (SA lead) |
| Q2 | NICE-TO-HAVE | What bounded lifetime (TTL) should cached geocode responses use? | Cached geocode responses use a bounded 24h TTL to respect vendor quota (`b2c-custom-caches`). | dev lead | APPROVED | Dane (SA lead) |
