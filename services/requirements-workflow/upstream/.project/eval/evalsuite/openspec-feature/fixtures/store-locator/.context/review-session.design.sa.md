# Per-Artifact Review Gate — session log (design, add-cartora-maps-integration)

This is the record of the review gate that ran after the integration `design.md`
was first drafted: the agent stopped, the reviewer chose "Walk the questions", and
the items were handled one at a time. Everything below already landed in the body
of the artifact. The session ended with the reviewer asking to see the file again
before approving.

---

**Agent:** The Cartora integration design is drafted. Two open questions need you:
the geocoding call approach (Q1 — server proxy vs direct client call, linked to the
proposed decision D6) and the domain-restricted key policy (Q2). Want to walk them?

**Reviewer (Dmitry Ivanov, SA):** Yes, walk them.

---

**Agent:** Q1 — geocoding call approach. D6 proposes a server proxy so the key
stays off the browser and quota/throttling is centralized; a direct client call is
simpler but exposes the key. I have recorded it as a proposed direction, not a
decision.

**Reviewer:** Leave it open for grooming. I want the platform team's view on the
MRT-vs-custom-SCAPI proxy placement before we commit. Keep D6 proposed and Q1 open.

**Agent:** Q2 — the domain-restricted key policy and quota/billing guardrails.
Still unconfirmed against the sources.

**Reviewer:** Also open. Same route — grooming, with the client's Cartora owner.

---

**Reviewer:** One thing though. Two items in here are being treated as if grooming
owns them, and they don't — they are developer-planning items, not approval
questions. First, the geocode-response cache strategy — the concrete TTL, cache
key, and invalidation. That only exists in the server-proxy branch, so it hangs off
Q1, but the SHAPE of it is a technical-design planning decision, not something
grooming signs. Second, how the lazy entrypoint actually loads and version-pins the
Cartora Maps SDK bundle — that is pure implementation HOW and it does not change the
agnostic interface. Don't leave those as prose buried in the Base Config and D5
notes. Route them to planning explicitly so technical-design picks them up.

**Agent:** Understood — those are planning deferrals, not open questions. I will add
a Planning Deferrals register and record them as `PD#` rows routed to
technical-design planning:

- PD1 — geocode-response cache strategy (TTL / key / invalidation), sourced from the
  Base Configuration Caching row; deferred because it lives only in the Q1
  server-proxy branch and is a bounded developer-HOW that does not change the
  capability contract; owner tech-design.
- PD2 — Cartora Maps SDK bundle load + version-pinning strategy for the lazy
  entrypoint, sourced from D5; deferred because it is implementation-time HOW that
  does not change the agnostic interface; owner SA / tech-design.

The Base Configuration Caching row now points at PD1, and the Design Gate gains a
line that developer-HOW is routed to planning rather than decided here. Nothing
about the capability contract, the endpoints, or the two open questions changes.

**Reviewer:** Right. That is the distinction I want held: grooming approves the
route, planning owns the answer.

---

**Reviewer:** One more, on the Integration Specification itself. During the walk we
ran a small spike against the Cartora sandbox — it confirmed the Maps SDK loads as a
client-only lazy bundle under our CSP nonce, and that the geocoding response shape
matches what the Data Mapping already states. Keep that as evidence so the downstream
ISD and technical-design inherit it instead of re-probing. Record it as a
`### Compatibility Validation` subsection, and while you are there add a short
`### Coverage & Caveats` note stating plainly what the contract covers and what it
leaves open (the Q1 call placement, the PD1 cache). Both are confirmatory — no new
endpoint, field, or obligation.

**Agent:** Understood — two confirmatory subsections inside the Integration
Specification, neither changing the contract. `### Compatibility Validation` records
the CSP/lazy-load and response-shape findings; `### Coverage & Caveats` states the
covered surface and points at Q1 and PD1 for the open edges. Neither adds an endpoint
or a mapping field, so the endpoints, the Data Mapping, and the `## Document
Consumption Guide` row for this section are unchanged.

**Reviewer:** Good. Now re-derive the reader map and the consumption guide so they
reflect the added Planning Deferrals section, AND re-derive the Integration
Specification's own `### Overview` so its signpost lists the two subsections you just
added. Re-show me the artifact before I approve.

---

**Agent:** Recorded. The body now carries a `## Planning Deferrals` section (PD1,
PD2), the two confirmatory Integration Specification subsections (Compatibility
Validation, Coverage & Caveats), a Design Source Inventory row for this review
session, and the Caching / Design Gate updates. All three derived blocks are
re-derived from the changed body: the Solution Overview names the two items passed
forward to planning, the Document Consumption Guide gains a Planning Deferrals row,
and the Integration Specification `### Overview` signpost now ends with the two
review-confirmed subsections.
