# Per-Artifact Review Gate — session log (discovery, add-store-locator)

This is the record of the review gate that ran after `discovery.md` was first
drafted: the agent stopped, the reviewer chose "Walk the questions", and the
questions were handled one at a time. Everything below already landed in the body
of the artifact. The session ended with the reviewer asking to see the file again
before approving.

---

**Agent:** Discovery is drafted. Three open questions need you: special/holiday
opening hours, the redirect for the retired `/stores` URL, and the Page-Designer
promo boundary. One change-request candidate — in-store appointment booking from
the workshop — is still pending a client answer. Want to walk the questions?

**Reviewer (Elena Sokolova, BA):** Yes, walk them.

---

**Agent:** Q3 — the PD↔SLS delivery boundary. The locator landing renders a
promo/editorial region, but the two components behind it (`page.storelocator`,
`assets.storeCard`) are signed under the `PD` WBS family, which is a separate
workstream. Two things are unclear: who delivers them, and what content ships in
that slot at v1.

Options as I see them: (a) the Page Designer FSD delivers both and the locator
only renders them; (b) the locator owns a minimal version of the fragment itself
and Page Designer replaces it later.

My recommendation is (a) — the components are already signed under `PD`, and
owning a second authoring surface here would duplicate the content team's tooling.
That is a recommendation, not a decision.

**Reviewer:** (a), and it is not really a question — Katya's content team already
has both components in their Page Designer backlog for this quarter. The locator
renders the slot, it never authors it. For v1 the slot ships with a single
Featured-Store card that the content team curates; no editorial fragment at
launch. Write that down and close the question.

**Agent:** Recorded. Q3 is resolved with your sign-off: Page Designer FSD delivers
both components; the locator consumes and renders them; v1 ships one curated
Featured-Store card. The dependency row keeps the components, the ownership seam
is now settled rather than open.

---

**Agent:** Q2 — the redirect for the retired `/stores` URL. Type and target are
undecided, and the page is bookmarked and indexed.

**Reviewer:** Leave it open for grooming, I want the SEO agency's view on 301 vs
302 before we commit. But find out how a redirect is actually configured on this
platform so grooming is not guessing at the mechanics.

**Agent:** Looked into it. The redirect is not storefront code: retired routes are
mapped in Business Manager's static URL mapping surface, the same admin surface
that already carries the locator merchant configuration. So whichever type and
target grooming picks, the change consumes that platform capability rather than
shipping a route handler. I have added it to Dependencies against RQ-709; the type
and target themselves stay open as Q2.

**Reviewer:** Good. Q2 stays open.

---

**Agent:** Q1 — special and holiday opening hours on the store detail. Still
depends on whether the commerce Stores API exposes them, and I could not confirm
that from the sources here.

**Reviewer:** Then leave it open too. Same as before: grooming.

---

**Agent:** Last item, not a question — CR1, the in-store appointment booking idea
Mara raised at the workshop.

**Reviewer:** I took that to the client this morning. The answer is no for this
release: appointment booking is a different journey and it needs a booking
provider we have not selected. It becomes its own change later, not scope here.
Record the decision so nobody re-opens it in grooming.

**Agent:** Recorded as declined, with the note that it is carried as a separate
future change.

---

**Reviewer:** One more thing, and this one is my mistake from the first pass. The
real-time in-store inventory requirement — I had you mark it "deferred", meaning
we would do it in a later phase of this change. That is wrong. Inventory
availability is being delivered by the inventory integration workstream, Dmitry's
team, not by us at all. It is somebody else's deliverable, not our later phase.
Fix the disposition.

**Agent:** Understood — that moves it from a later phase of this change to a
requirement a separate change owns, with that owner named.

**Reviewer:** Right. Now re-show me the artifact and I will approve it.
