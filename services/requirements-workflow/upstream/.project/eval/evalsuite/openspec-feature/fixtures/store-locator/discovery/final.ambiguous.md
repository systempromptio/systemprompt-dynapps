# Delivery scope unresolved — split before discovery

I can't author a single `discovery.md` for `add-store-locator` yet: the ask
bundles a client-facing feature surface with real external-integration work, and
no delivery scope was chosen (no role, no selected scope, no named service to
build). A single change owns exactly one delivery scope, so this needs to be
split before discovery. Here is the decomposition and the choice I need from you.

### Why this must split

The source packet describes distinct kinds of deliverable that cannot share one
delivery scope: one functional feature surface plus THREE separate third-party
integrations, each with its own vendor, contract, key, and billing, and each with
an open connector decision no one has made yet.

- A **functional storefront feature** — the Store Locator UI and behaviour
  (search, address autocomplete UI, synchronized map + list, store detail,
  directions, service filter, reachable from global navigation;
  RQ-701..RQ-707, RQ-709, RQ-710).
- A **maps integration** — the Cartora map-rendering + geocoding/proximity
  connector, which carries its own contract, API-key/domain-restriction, billing,
  and the open client-vs-server-proxy geocoding decision.
- A **separate address-autocomplete integration** — the Addressly connector for
  typed-address suggestions, a DIFFERENT third-party supplier with its own key,
  contract, and billing, and the same open client-vs-proxy call decision. It must
  not be folded into the Cartora work.
- A **separate appointment-booking integration** — the SlotBook connector for the
  new "book an in-store appointment" idea, a DIFFERENT vendor with its own
  account/contract and an open embed/iframe-vs-API decision. It is also a new ask
  not in the signed SRD, so it needs client confirmation before it is built.

A functional change references agnostic integrations as dependencies; it does not
own them. Three distinct external services means three distinct agnostic
integration changes, not one bundled integration.

### Candidate changes

- **Store Locator (functional)** → produces an **FSD**.
  Owns the storefront locator capability (search, address autocomplete UI, map +
  list, store detail, directions, service filter, global-nav entry) and the
  retirement/redirect of the legacy `/stores` directory page. Depends on the
  Cartora maps integration, the Addressly integration, the SlotBook integration
  (if CR1 is confirmed), and the existing commerce Stores API.

- **Cartora maps integration** → produces an **ISD**.
  Owns the agnostic map-rendering + geocoding/proximity connector: Cartora SDK
  rendering, postcode/city geocoding, key domain-restriction, billing on the
  client's Cartora account, and the client-side-vs-server-proxy geocoding
  decision. It does not own any storefront UI, address autocomplete, or appointments.

- **Addressly address-autocomplete integration** → produces an **ISD**.
  Owns the agnostic typed-address autocomplete connector: Addressly API contract,
  its own key/quota/billing, and the client-vs-server-proxy call decision. A
  separate supplier from Cartora; it does not own any storefront UI.

- **SlotBook appointment-booking integration** → produces an **ISD**.
  Owns the agnostic appointment-booking connector: SlotBook account/contract and
  the embed/iframe-vs-API integration decision. A separate vendor again; gated on
  CR1 (the appointment idea is a new ask, not in the signed SRD).

Not a separate change: the **commerce Stores API** is an existing, governed,
already-in-use read API with no new store data model in scope (v1), so it is a
dependency of the functional change, not a new integration to build.

### What I need from you

Choose the scope to proceed with, and I'll run discovery for exactly that one:

- proceed as the **functional Store Locator** change (FSD), treating all three
  integrations as dependencies; or
- proceed as the **Cartora maps integration** change (ISD), with the storefront UI
  and the other integrations out of scope; or
- proceed as the **Addressly address-autocomplete integration** change (ISD), with
  the storefront UI and the other integrations out of scope; or
- proceed as the **SlotBook appointment-booking integration** change (ISD), with
  the storefront UI and the other integrations out of scope (and CR1 confirmed).

I have not authored a Requirements Trace, capability decomposition, or Discovery
Gate, and I have invented no requirements, integrations, or decisions beyond the
source packet — I'm holding until the scope is chosen.
