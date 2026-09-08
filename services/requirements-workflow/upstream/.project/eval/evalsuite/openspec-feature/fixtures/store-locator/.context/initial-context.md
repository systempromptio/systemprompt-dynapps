# Source packet — Store Locator

<!-- Eval-only source packet for the store-locator golden case. It models the raw inputs a
     real pipeline would retrieve from project-context: a signed SRD exported as CSV, a raw
     discovery-workshop transcript, some scattered technical notes, and the current OpenSpec
     baseline specs. These are deliberately messy and NOT pre-decomposed: nobody in the sources
     states a delivery scope, and nobody draws the integration boundaries for you. The delivery
     scope must be resolved from these facts, not from the change name.

     Eval source_type map (the content below emulates project-context results; cite realistic
     project-context anchors in artifacts, not this eval-only file):
       - source_type: "confluence" (SRD export) -> Source A, standing in for
         `.cursor/.project/project-context/inbox/confluence/store-locator-srd-requirements.csv.txt`.
       - source_type: "meeting_notes" -> Source B, standing in for
         `.cursor/.project/project-context/inbox/meeting_notes/2026-02-04_storefront-workshop_transcript.txt`.
       - source_type: "confluence" (technical notes) -> Source C, standing in for
         `.cursor/.project/project-context/inbox/confluence/store-locator-technical-notes.md`.
       - source_type: "jira" -> Tracker metadata below; do not infer extra ticket data.
       - source_type: "specs" -> separate baseline capability CONTEXT blocks, standing in for
         `openspec/specs/<capability>/spec.md` search results.
       - Derived summary -> non-authoritative index only; not a project-context source.

     Read order / authority:
       1. Signed SRD rows (Source A) are the canonical WHAT and official RQ/section codes.
       2. Existing OpenSpec baseline specs (separate CONTEXT blocks) define current behaviour.
       3. The workshop transcript (Source B) is raw and provisional: it refines the signed
          source, surfaces new asks, constraints, and open questions, but its throwaway remarks
          are not signed decisions unless the SRD says so.
       4. Technical notes (Source C) explain HOW/risk and usually become Open Questions or
          Planning/Design input.
       5. Derived summaries are convenience indexes only; do not treat them as requirements. -->

Client: **Meridian Outfitters** (the multi-brand apparel retailer described below).
Delivery partner / agency: **Astound**.
Project name: **Meridian Outfitters — Storefront (SFX-4102)**.
Change name: `add-store-locator`
Tracker: JIRA `SFX-4102` — "Store Locator (find a store)"
SRD wiki: `https://wiki.example.com/display/COM/Store+Locator+SRD`

> Each source below carries the `source_type` / `date` / `date_source` a real
> project-context result would return. Weigh them by the `project-context` skill's
> Source Authority & Temporal Ranking contract: signed SRD (Source A) outranks the
> provisional workshop transcript (Source B); a throwaway remark in Source B is a
> candidate, not a signed decision, until a higher-authority source ratifies it.

## Background (business context, not the requirements table)

Meridian Outfitters is a multi-brand apparel retailer with roughly 180 physical
stores across the EU and UK. Today the only way to find a shop on the storefront is a
legacy static "Store Directory" page at `/stores` — a Page Designer content page
listing addresses alphabetically by city, with no search, no map, and no live
store data, linked from the footer "Our stores". The client wants to replace it
with a proper store locator so shoppers can find nearby stores, see the services
each offers, get directions, and (a new idea raised in the workshop) book an
in-store appointment. Once the locator ships, the old static directory is retired.

Storefront is Salesforce Commerce (B2C) on the Storefront Next (React) stack.
Store records already live in the commerce backend and are read through the
existing Stores API. The workshop mentioned three third-party pieces by name —
a maps SDK, an address type-ahead supplier, and an appointment-booking widget —
but nobody wrote down who owns which, how each is called, or which are already
wired up.

## Source A — Signed SRD, exported from the requirements tracker (CSV)

> project-context result metadata — source_type: `confluence` · date: `2026-02-06` · date_source: `frontmatter`

<!-- This is a raw CSV export of the signed SRD requirements sheet, mirroring the real SRD
     column layout — including a separate, multi-line "Requirement Details" column and a two-line
     wrapped header. Columns are noisy and quoting is imperfect, as in a real export. A faithful
     discovery extracts the RQ codes, the verbatim Requirement, and the verbatim Requirement
     Details (preserving the numbering; "—" when a row carries no details), and preserves the
     codes exactly. -->

```
Highlevel WBS,Lowlevel WBS,Assigned WBS Code,ID,Requirement,Requirement Details,Notes/Dependencies,Re-usability,SML Analysis,Release,"MoSCoW 
Rating",Client Approval,Approved By,Astound Acceptance,Approval Date,Last Update,Creation Date,Created By,Updated By,Package,Current Release?,Req No.
Formula Row,,,,,,,,,,,,,,,,,,,,,
SLS - Store Locator,SLS.SRCH - Store search,SLS.SRCH,RQ-701,"As a shopper, I SHALL be able to search for stores by entering a postal code or a city, or by using my current location.","1. Search inputs: postal code, city, or current location.
2. Current-location search uses device geolocation and is user-initiated.","- Maps/geocoding provider for postcode/city lookup and proximity.",New,Medium,,Must have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,1
SLS - Store Locator,SLS.MAP - Map and synchronized list,SLS.MAP,RQ-702,"Search results SHALL be presented as an interactive map and a synchronized list, so that selecting a store in one highlights it in the other.","1. Present results as an interactive map and a synchronized results list.
2. Selecting a store in the list highlights it on the map, and vice versa.","- Maps SDK for interactive map rendering.",New,Medium,,Must have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,2
SLS - Store Locator,SLS.DTL - Store detail,SLS.DTL,RQ-703,"Each store SHALL display its name, full address, phone number, opening hours and the services available at that store.","1. Display store name, full address, phone number, opening hours, and available services.
2. Store data is read from the commerce backend; no new store data model for v1.","- Commerce Stores API (authoritative store data).",New,Medium,,Must have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,3
SLS - Store Locator,SLS.DIR - Get directions,SLS.DIR,RQ-704,"Each store SHALL provide a ""Get directions"" action that opens the shopper's map provider with the store as the destination.","1. Provide a ""Get directions"" action for each store.
2. Opens the shopper's map provider with the store set as the destination.","- Shopper's device/native map provider.",New,Small,,Must have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,4
SLS - Store Locator,SLS.FLT - Filter by service,SLS.FLT,RQ-705,"The shopper SHALL be able to filter stores by available service: click-and-collect, returns, and gift wrapping.","1. Filter stores by available service.
2. v1 services: click-and-collect, returns, and gift wrapping.","- Service attributes from the commerce Stores API.",New,Small,,Should have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,5
SLS - Store Locator,SLS.A11Y - Accessibility,SLS.A11Y,RQ-706,"The store locator SHALL be fully keyboard operable and usable with a screen reader, including an accessible list alternative to the map.",,"- WCAG 2.2 AA target.",New,Small,,Must have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,6
SLS - Store Locator,SLS.I18N - Localizable copy,SLS.I18N,RQ-707,"All shopper-facing text in the store locator SHALL be localizable.",,,New,Small,,Must have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,7
SLS - Store Locator,SLS.INV - Real-time in-store inventory,SLS.INV,RQ-708,"The store detail SHALL show real-time in-store product availability.","1. Show real-time in-store product availability on the store detail.","- Later phase; not in the initial release.",New,Large,,Could have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,N,8
GH - Global Header,GH.UTL - Header utility icons,GH.UTL,RQ-709,"The store locator SHALL be reachable from the global site navigation.",,"- Footer ""Our stores"" link; retires the legacy /stores directory.",Modify,Small,,Must have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,9
SLS - Store Locator,SLS.AC - Address autocomplete,SLS.AC,RQ-710,"As the shopper types a postal code or partial address in the store search, the store locator SHALL offer address autocomplete suggestions.","1. As the shopper types a postal code or partial address, offer address suggestions.
2. Suggestions come from the address type-ahead supplier (see workshop).","- Address type-ahead supplier (not named here; see workshop).",New,Small,,Should have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,10
```

SRD sheet notes (free-text cells at the bottom of the export):
- RQ-708 (real-time in-store inventory) is flagged "Later phase — not in initial
  release" by the client.
- Store data (name, address, geo, hours, services) is authoritative in the
  commerce backend and read via the existing Stores API. "No new store data model
  for v1."
- RQ-710 (address autocomplete): the sheet cell says "suggestions from the
  address type-ahead supplier (see workshop)". It does not name the supplier or
  say who builds it.

## Source B — Discovery workshop transcript (raw, auto-captured)

> project-context result metadata — source_type: `meeting_notes` · date: `2026-02-04` · date_source: `frontmatter`

<!-- Raw transcript excerpt. Speakers: Mara (client PO), Ben (BA), Dane (SA/architect lead),
     Pia (client FE lead), Sam (client platform dev). It is provisional and rambling; remarks
     are not signed decisions. Nobody states a delivery scope or assigns ownership. -->

Workshop — Store Locator, 2026-02-04

Mara (PO): launch scope is "find a store, see services, get directions". The
in-store stock thing (that's 708) is later phase, don't build it now.

Mara (PO): the new locator replaces the old static `/stores` directory — retire
that page at launch and re-point the footer "Our stores" link to the locator.

Dane (SA): the old `/stores` URL is bookmarked and indexed, so we'll need a
redirect so links don't 404. Exact target/type — we'll sort that out later.

Pia (FE): the map itself is Cartora — that's the maps SDK we render with. I think
there's already a Cartora account somewhere? Sam would know. The map key has to
be domain-restricted and the billing sits on the client account.

Sam (dev): honestly not sure the Cartora account is fully set up for this site.
And there's an open question — do we geocode the typed postcode on the client
with Cartora, or proxy it through our own server? Changes who's exposed to the
key and who eats the quota/billing. Not decided.

Pia (FE): the type-ahead in the search box isn't Cartora — that's Addressly. It's
a different supplier, separate account, its own key and contract. It just returns
address suggestions as you type.

Dane (SA): right, Addressly is its own thing. Same open question as the map,
though — do we call Addressly straight from the browser or via a proxy? Same
key-exposure / billing trade-off. Undecided.

Mara (PO): oh — can shoppers book an in-store appointment from the locator? We
use SlotBook for appointments elsewhere. Would be nice to add here.

Sam (dev): SlotBook is the booking widget — usually an embed/iframe, but there's
also an API. Which way we wire it in isn't decided, and it's a separate SlotBook
account/contract from the other two. Also this wasn't in the signed SRD.

Ben (BA): noted, I'll flag the appointment booking as a new ask to confirm with
the client — not silently in scope.

Pia (FE): opening hours — some stores have special/holiday hours. Nice to surface
them, but the backend may not expose them yet. Leave it open.

Dane (SA): privacy — geolocation must be user-initiated (a button), gated on the
browser permission prompt, and the shopper's coordinates must NOT be persisted or
sent to analytics.

Dane (SA): I still need to look at the codebase to see how much of Cartora /
Addressly / SlotBook is already wired vs. green-field. Might come back with more
questions — this feels like more than one piece of work but we didn't scope it.

## Source C — Technical notes (scattered design/planning input)

> project-context result metadata — source_type: `confluence` · date: `2026-02-10` · date_source: `mtime`

- Storefront Next (React, SSR/RSC). The map must be lazy-loaded so it does not
  block initial render or the critical JS path.
- Cartora Maps SDK for map rendering; Cartora also does postcode/city geocoding
  and proximity. Call approach (client vs server proxy) is undecided; affects key
  exposure, quota, and billing.
- "Get directions" is a deep-link hand-off to the shopper's own device map app
  (e.g. Google/Apple Maps) with the store as destination; it does not render
  through the Cartora SDK or call Cartora.
- Addressly for typed-address autocomplete: separate third-party supplier with
  its own key, quota, contract, and billing. Client-side vs server-proxy call is
  undecided. Distinct from Cartora.
- SlotBook appointment-booking widget for the "book an appointment" idea: separate
  vendor, its own account/contract. Embed/iframe vs API integration is undecided.
- Store data via the commerce backend Stores API; no new store custom objects for
  v1. This API is already in use elsewhere on the storefront.
- Privacy: shopper geolocation is client-side only, never persisted, never sent to
  analytics. No PII stored by this feature.
- Accessibility target: WCAG 2.2 AA.
- i18n: all copy via translation keys; the storefront is multi-locale.

## Source D — Project team & roles (standing wiki roster)

> project-context result metadata — source_type: `confluence` · date: `2026-01-20` · date_source: `frontmatter`

<!-- The engagement's standing "Team & roles" wiki page (who's who). It records org
     membership and roles only; it assigns no delivery scope and ratifies no decision. -->

Engagement: Meridian Outfitters — Storefront (SFX-4102). Delivery agency: **Astound**;
client: **Meridian Outfitters**.

**Astound (delivery agency) — Store Locator squad**
- Ben Okafor — BA (Functional Analyst; owns the functional specification / FSD)
- Dane — SA / architect lead (co-owns the delivery specifications; leads the integration / ISD)
- FE / BE / QA engineers — not yet assigned (TBD)

**Meridian Outfitters (client) — stakeholders**
- Mara — Product Owner (client sign-off / approver)
- Pia — FE lead (client-side front-end lead)
- Sam — platform developer (client platform/dev)

Not on the delivery team: Roman Dolzhanskyi maintains the requirements tracker (he is the
SRD sheet's `Created By` / `Updated By`) — a tracker editor, not a Store Locator
delivery-team member and never a document author.

## Derived summary — Known / unknown index (non-authoritative)

> project-context result metadata — source_type: `derived` · date: `(undated)` · date_source: `mtime` — orientation index only; NOT a project-context source, never cite as authority.

Known:
- Feature scope: search (postcode/city/geolocation), address autocomplete, map +
  synchronized list, store detail (address, phone, hours, services), directions,
  service filter, reachable from global nav. Appointment booking raised as a new
  idea, not signed.
- Third-party pieces named in the sources: Cartora (maps + geocoding/proximity),
  Addressly (typed-address autocomplete), SlotBook (appointment booking). Store
  data comes from the existing commerce backend Stores API.
- Services in v1 filter: click-and-collect, returns, gift wrapping.
- Existing storefront state: a legacy static store directory at `/stores` (Page
  Designer content, footer-linked) that the locator replaces and retires.

Unknown / undecided (no decision made yet):
- Cartora call approach (client-side geocoding vs server proxy) — affects key
  exposure, quota, billing.
- Addressly call approach (client-side vs server proxy) — affects key exposure,
  quota, billing.
- SlotBook integration approach (embed/iframe vs API) — separate account/contract.
- Whether special/holiday opening hours are exposed by the backend.
- "Book an appointment" — new ask, needs a client decision.
- Redirect for the retired `/stores` URL (type and exact target).
