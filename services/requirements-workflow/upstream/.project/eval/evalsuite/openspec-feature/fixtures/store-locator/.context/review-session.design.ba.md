# Per-Artifact Review Gate — session log (design, add-store-locator)

This is the record of the review gate that ran after the functional `design.md`
was first drafted: the agent stopped, the reviewer chose "Walk the questions", and
the items were handled one at a time. Everything below already landed in the body
of the artifact. The session ended with the reviewer asking to see the file again
before approving.

---

**Agent:** The store-locator design is drafted. Two open questions need you:
whether store detail should surface special/holiday opening hours (Q1, depends on
the Stores API) and the redirect type/target for the retired `/stores` URL (Q2).
Want to walk them?

**Reviewer (Elena Sokolova, BA):** Yes.

---

**Agent:** Q1 — special/holiday opening hours in store detail. The Stores API
exposes regular hours; whether it exposes holiday/special hours is unconfirmed, so I
raised it for grooming rather than committing the field.

**Reviewer:** Keep it open — grooming, with the store-data integration. Don't
promise holiday hours in the design.

**Agent:** Q2 — the retired `/stores` redirect. The design fixes the target as
`/store-locator` (D6) but leaves the redirect TYPE (permanent vs temporary) to
grooming.

**Reviewer:** Right, leave the type for grooming. The target is correct.

---

**Reviewer:** One correction to the decisions. The whole design leans on the
accessible list being the real surface and the map being an enhancement — it's in
the rendering decision and the accessibility gap row — but it is never stated as a
decision in its own right. I've seen technical-design read a design like this and
treat the map as load-bearing because list-primacy was only implied. Make it an
explicit architecture decision so it can't be missed.

**Agent:** Agreed. I'll add an architecture decision recording it:

- D7 — the accessible, server-rendered result list is the canonical store-finding
  surface and the no-JS / screen-reader path; the map is a progressive enhancement,
  never a prerequisite for any locator function. Rationale: it is implied across D1
  and the SLS.A11Y gap, but making it first-class protects the WCAG 2.2 AA target
  and stops downstream treating the map as load-bearing. Alternatives: leave it
  implicit (rejected — leaves the accessibility contract to inference); map-first
  with a list fallback (rejected — inverts the guarantee). Trade-off: commits every
  locator function to work through the list alone.

No scope changes, no new section — it formalises a stance the design already took.

**Reviewer:** Good. That's the one I wanted on the record. Re-derive the reader map
and the consumption guide so the decision count is right, and show me the file again
before I approve.

---

**Agent:** Recorded. Architecture Decisions now runs D1–D7, and a Design Source
Inventory row notes this review session. The Solution Overview and Document
Consumption Guide are re-derived from the changed body — the overview now names
seven decisions (D1–D7) and the guide's Architecture Decisions row reads D1–D7.
Nothing else in the body moved.
