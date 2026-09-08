# Per-Artifact Review Gate — session log (proposal, add-cartora-maps-integration)

This is the record of the review gate that ran after the integration `proposal.md` was
first drafted: the agent stopped and summarized the offer, and — the proposal carrying no
new proposal-stage open questions — offered **Approve / Request changes**. The reviewer
chose **Request changes**: one boundary clarification. Everything below already landed in
the body of the artifact. The session ended with the reviewer asking to see the file again
before approving.

---

**Agent:** The Cartora maps-integration proposal is drafted. It locks the integration-only
scope as four consumer-value slices (S1–S4: geocoding with proximity, governed map
rendering, and the two consumed-presentation obligations), carries no CR candidates, and
records the unconfirmed Cartora account/key setup as the blocking prerequisite. No new
proposal-stage questions — discovery's Q1 (geocoding call approach) and Q2 (key/quota
guardrails) stay registered upstream for grooming. Approve, or request changes?

**Reviewer (Dmitry Ivanov, SA):** One change first. S1 has the capability returning
proximity-ranked results from Cartora's geocoding, and that's right — but the offer never
says who owns the place data itself. When a store geocodes to the wrong spot, someone
downstream will file it as a bug against this capability. It isn't: the place data and its
accuracy are Cartora's. State it as a boundary so we don't own vendor-data correction.

**Agent:** Agreed. I'll add an Out of Scope & Boundaries bullet:

- The underlying geocoding place data and its accuracy — owned by Cartora as the vendor.
  This change consumes the geocoding result Cartora returns and owns no place-data
  curation, correction, or coverage surface; a wrong or missing place is a vendor-data
  matter, not a defect of this capability. This is separate from the Q1 call approach,
  which stays open for grooming.

No scope change and no new slice — it draws the line on data ownership the offer already
assumed.

**Reviewer:** Right — that's the distinction I want held: we own the integration, the
vendor owns the geodata. Record the review, re-derive the Executive Summary and the
Document Consumption Guide so the out-of-scope owners are complete, and re-show me the
file before I approve.

---

**Agent:** Recorded. Out of Scope & Boundaries now carries the vendor-geodata boundary,
and a Proposal Source Inventory row notes this review session. The Executive Summary and
Document Consumption Guide are re-derived from the changed body — the summary names the
vendor-owned geocoding place data among the out-of-scope owners it hands downstream, and
the guide's Out of Scope & Boundaries row and Proposal Source Inventory row carry it.
Nothing else in the body moved.
