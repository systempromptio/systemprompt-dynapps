# Per-Artifact Review Gate — session log (proposal, add-store-locator)

This is the record of the review gate that ran after the functional `proposal.md` was
first drafted: the agent stopped and summarized the offer, and — the proposal carrying
no new proposal-stage open questions — offered **Approve / Request changes**. The
reviewer chose **Request changes**: one boundary clarification. Everything below already
landed in the body of the artifact. The session ended with the reviewer asking to see the
file again before approving.

---

**Agent:** The store-locator proposal is drafted. It locks the discovery-reconciled
functional scope as nine shopper-value slices (S1–S9, one per in-scope requirement),
carries CR1 (in-store appointment booking) as a declined disposition for grooming, and
records the three consumed capabilities as external dependencies. No new proposal-stage
questions were raised — discovery's Q1 (holiday hours) and design's Q2 (retired-URL
redirect) stay registered upstream for grooming. Approve, or request changes?

**Reviewer (Elena Sokolova, BA):** One change before I approve. The offer leans on the
store records — name, address, phone, opening hours, services — coming from the client's
own store-ops team through the already-integrated Stores API; S4 says as much in passing.
But it is never stated as a boundary. I've seen a downstream FSD read a proposal like this
and assume the locator owns store-record authoring — a CMS screen to edit hours, say —
because nobody drew the line. Draw it: put store-record authoring and maintenance in Out
of Scope & Boundaries, explicitly, so no slice can quietly pick it up.

**Agent:** Agreed. I'll add an Out of Scope & Boundaries bullet:

- Store record authoring and maintenance (name, address, phone, opening hours, services)
  — owned by the client's store-ops team in the commerce backend and read through the
  already-integrated Stores API; the locator reads and presents records, it never authors
  or maintains them. This is distinct from the Stores API service-contract boundary
  already listed (keys / quota / call approach sit with the separate integration).

No scope change and no new slice — it formalizes a boundary the offer already relied on.

**Reviewer:** Good. That's the line I wanted drawn. Record the review, re-derive the
Executive Summary and the Document Consumption Guide so the out-of-scope owners are
complete, and show me the file again before I approve.

---

**Agent:** Recorded. Out of Scope & Boundaries now carries the store-record-authoring
boundary, and a Proposal Source Inventory row notes this review session. The Executive
Summary and Document Consumption Guide are re-derived from the changed body — the summary
names store-record authoring among the out-of-scope owners it hands downstream, and the
guide's Out of Scope & Boundaries row and Proposal Source Inventory row carry it. Nothing
else in the body moved.
