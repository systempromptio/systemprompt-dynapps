## What the user asked for

The user said:

> "I'm the BA on the Store Locator. I want to prepare the functional side — the
> storefront feature shoppers actually use. The integration work is owned
> separately by the architect, so I'm not specifying that here; just treat those
> integrations as dependencies I rely on."

## Source D — Visual reference

There is no committed UX prototype or design asset for this feature; the client
agreed to use the current live site as the visual reference. Every in-scope
requirement carries a Visualization: it describes, in words, the screen(s) to
capture for that requirement — the meaningful states and the desktop/mobile
surfaces — and the BA attaches the actual images to the published page
afterwards. Do not invent an image file, a design-tool link, or a state the
behaviour does not imply.

## Source E — Responsive layout and merchant configuration (functional)

Two functional facts the BA carries into the FSD:

- **Responsive result layout.** The map + synchronized list renders on both
  desktop and mobile/tablet, and the layout differs by surface: on desktop the
  interactive map is shown beside the list and stays visible; on mobile and
  tablet the view is list-first with the map collapsed behind a toggle, so the
  list is the primary view and the accessible list alternative is protected.
- **Merchant configuration in Business Manager.** Several locator behaviours are
  configured by the client's own team (administrators / business users) in SFCC
  Business Manager, not hard-coded: the default search radius and units and
  whether the use-my-location option is offered; the number of stores per page
  and the load-more batch; which services are offered as filter options (seeded
  in v1 with click-and-collect, returns, gift wrapping); and the store records
  (name, address, phone, hours, services) the store detail draws from. These are
  the client's own merchandising/admin controls. Third-party vendor keys, quotas,
  and call approach for the consumed maps/geocoding and address-autocomplete
  services are NOT part of this — they are owned by the separate integration
  changes and never appear in the functional FSD.

## Source F — Page Designer content authoring on the locator landing (functional)

> project-context result metadata — source_type: `confluence` (Page Designer SRD export) · date: `2026-02-06` · date_source: `frontmatter` — stands in for `.cursor/.project/project-context/inbox/confluence/page-designer-srd-requirements.csv.txt`.

The store-locator **landing** keeps a Page-Designer-authored promotional/editorial
content region — the same authoring model the legacy `/stores` content page used —
maintained by the client's content team. The locator page **renders** that authored
slot. Which promo content ships in the slot at v1 is a content-team / Page Designer
decision.

Excerpt from the Page Designer workstream's signed component list (owned by the
Page Designer FSD, `PD` WBS family — a separate change):

```
Highlevel WBS,Lowlevel WBS,Assigned WBS Code,ID,Requirement,Requirement Details,Notes/Dependencies,Re-usability,SML Analysis,Release,"MoSCoW 
Rating",Client Approval,Approved By,Astound Acceptance,Approval Date,Last Update,Creation Date,Created By,Updated By,Package,Current Release?,Req No.
PD - Page Designer,PD.PT.SLP - Store Locator content fragment,PD.PT.SLP,RQ-751,"As a Content Manager, I want a Store Locator content fragment so that the content team can author the promotional/editorial region shown on the store locator landing.","1. A Page Designer page-fragment type that authors the locator landing's promo/editorial region.
2. Current Component ID `page.storelocator`.","- Owned by the Page Designer FSD (PD workstream); authors the store locator landing's promo/editorial region.",New,Small,,Should have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,51
PD - Page Designer,PD.CMP.STC - Featured-Store promo card,PD.CMP.STC,RQ-752,"As a Content Manager, I want a Featured-Store promo card so that the content team can highlight selected stores or offers on the store locator.","1. A Page Designer asset component rendering a featured-store / offer promo card.
2. Current Component ID `assets.storeCard`.","- Owned by the Page Designer FSD (PD workstream); renders a featured-store / offer promo card.",New,Small,,Could have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,52
PD - Page Designer,PD.CMP.HMH - Homepage hero banner,PD.CMP.HMH,RQ-753,"As a Content Manager, I want a Homepage hero banner so that I can promote seasonal campaigns on the storefront homepage.","1. A Page Designer asset component rendering the storefront homepage hero.
2. Current Component ID `assets.heroBanner`.","- Owned by the Page Designer FSD (PD workstream); storefront homepage merchandising.",New,Small,,Should have,Approved,Ben Okafor,,,06/02/2026,04/02/2026,Roman Dolzhanskyi,Roman Dolzhanskyi,3,Y,53
```
