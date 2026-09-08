# Per-Artifact Review Gate — session log (discovery, add-cartora-maps-integration)

This is the record of the review gate that ran after `discovery.md` was first
drafted: the agent stopped, the reviewer chose "Walk the questions", and the
questions were handled one at a time. Everything below already landed in the body
of the artifact. The session ended with the reviewer asking to see the file again
before approving.

---

**Agent:** Discovery is drafted. Three open questions need you: whether typed
postcodes are geocoded client-side or through a server proxy, the key and quota
guardrails on the Cartora account, and which map style ships at v1 together with
who authors it. No change-request candidates — the appointment-booking idea from
the workshop belongs to the functional surface, not to this integration. Want to
walk the questions?

**Reviewer (Dane Whitfield, SA lead):** Yes, walk them.

---

**Agent:** Q1 — geocoding typed postcodes client-side against Cartora, or through
a server proxy we own. It decides where the key is exposed, where throttling and
error handling live, and how billing is governed.

**Reviewer:** Leave it open. I have a view — I lean to the proxy — but this is a
grooming decision with the client's platform people in the room, because it
changes who operates the throttling. Do not write my lean into the artifact as if
it were settled.

**Agent:** Understood. Q1 stays Open, and the trace keeps the call approach
undecided on RQ-711.

---

**Agent:** Q3 — the v1 map style. The connector renders a vendor-hosted Map Style
ID published from Cartora Map Studio, so two things are unclear: which style ships
at v1, and who authors it.

**Reviewer:** That one I can close, I had the call yesterday. Katya's brand team
already publishes a "Brand Street" style in Map Studio — street map type, the
brand palette, and the custom pin set all in one published Style ID. That is what
v1 renders. Her team owns authoring it and any later change to it; we only
reference the published ID and we build no configuration surface. Write it down
and close the question.

**Agent:** Recorded. Q3 is resolved with your sign-off: v1 renders the brand
team's published "Brand Street" Map Style ID; the brand/content team owns
authoring; the connector references the ID and owns no configuration surface. The
Map Studio dependency row stays, and the presentation boundary is now settled
rather than open.

---

**Reviewer:** While you have the Dependencies table open — the Cartora account and
the domain-restricted key are not in it. That cannot be right. We cannot call
Cartora at all without a restricted key issued on the client's account, and the
quota and billing sit there too. Everything downstream already assumes it.

**Agent:** You are right, it is missing. Adding it: the Cartora account plus the
domain-restricted API key, client-administered, used by RQ-711 and RQ-712. Its
status stays unconfirmed, because the exact restriction and quota policy is
precisely what Q2 asks — so the dependency is recorded, the policy stays open.

**Reviewer:** Good. Q2 stays open.

---

**Reviewer:** Last thing, and this one is a mistake in the first pass. You have
store detail and the service filter — SLS.DTL and SLS.FLT — attributed to a
"commerce store-data integration" as though a separate integration change owned
them. There is no such change. The commerce Stores API is an existing, already
integrated backend service; nobody is building an integration for it. Those two
requirements are functional surface, and they belong to the Store Locator FSD
along with the rest of the SLS functional rows. Fix the attribution.

**Agent:** Understood — the requirements stay out of scope for this integration
either way, but the owning workstream on those two rows changes from a
non-existent integration change to the functional Store Locator change.

**Reviewer:** Right. Now re-show me the artifact and I will approve it.
