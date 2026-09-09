"""Build the fictional, versioned storefront migration source corpus."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
NOTICE = '[DEMO] Fictional Linden & Rowe migration. Not a Charles Tyrwhitt project or statement. Dataset: demo-spa-migration. Fixture version: 1.'
AREAS = [
 ('Category browsing', [
  ('Category identity and breadcrumbs', 'Show the linen-polo category title and a home-to-polos breadcrumb; each ancestor opens its category.', 'A direct category URL and navigation from Polos show the same heading and breadcrumb.'),
  ('Product cards and prices', 'Show image, product name, colour and current price on every available product card; show a previous price only when a genuine promotion applies.', 'Selecting a card opens the matching product and colour; unavailable price data never renders as zero.'),
  ('Filter and sort state', 'Support size and colour filters plus price ascending, price descending and recommended sorting; show selected filters and a clear-all control.', 'Combining filters narrows results; sorting preserves selected filters; clearing resets filters without leaving the category.'),
  ('Shareable category navigation', 'Persist filter and sort selections in the URL and restore them on refresh and browser back/forward.', 'A shared category URL restores the same selections; unknown values are ignored with the remaining valid selections retained.'),
  ('Loading, empty and failure states', 'Show a loading state during catalog requests, a helpful empty state when no products match, and a retry action when catalog loading fails.', 'A failed request never appears as a successful zero-results response; retry retains the current selections.')]),
 ('Product detail', [
  ('Variant selection', 'Require a valid size and colour before adding the item to the bag.', 'Changing colour updates the product image and availability; unavailable variants cannot be added.'),
  ('Product content', 'Render fabric composition, care guidance and a size guide from commerce content.', 'Missing optional care content does not hide the purchasable product.'),
  ('Stock messaging', 'Show available, low-stock or unavailable messaging using the commerce availability response.', 'A stale availability response is rechecked when adding to the bag.'),
  ('Image gallery', 'Provide keyboard-operable product imagery and meaningful alternative text.', 'The selected image can be changed without a pointer and focus remains visible.'),
  ('Product recommendations', 'Show recommendations as a separately loaded optional section.', 'Recommendation failures do not block product detail or add-to-bag.')]),
 ('Search', [
  ('Search submission', 'Allow a shopper to submit a textual catalog query from global navigation.', 'Whitespace-only input is rejected; a valid query opens a shareable results URL.'),
  ('Search result refinement', 'Support size and colour filters on search results.', 'Selected filters are visible and can be removed independently.'),
  ('No-result guidance', 'Show the original query and a clear path to browse categories when search returns no matches.', 'The empty screen preserves the query and offers a new search.'),
  ('Search service failure', 'Distinguish search service errors from no matches.', 'Retry keeps the query and filters.'),
  ('Search suggestions', 'Defer type-ahead suggestions to phase two.', 'Phase-one search works without suggestions and does not call a suggestions service.')]),
 ('Cart', [
  ('Add to bag', 'Add the chosen variant and quantity to the active shopper bag.', 'A confirmed add displays the correct variant; a failed add does not increment the bag count.'),
  ('Update quantities', 'Support valid quantity changes and remove-item actions.', 'Server-rejected quantities restore the previous accepted quantity and show a useful message.'),
  ('Cart totals', 'Display subtotal, discounts and estimated total from the commerce basket response.', 'The UI never computes a conflicting promotional total independently.'),
  ('Guest cart persistence', 'Restore the guest basket after refresh within the agreed basket lifetime.', 'An expired basket produces an empty-bag explanation rather than a server error.'),
  ('Cart stock conflict', 'Explain items that become unavailable before checkout.', 'An unavailable line is clearly identified and cannot proceed unchanged.')]),
 ('Checkout', [
  ('Guest checkout', 'Allow checkout without account creation.', 'A guest can provide contact and delivery details without a forced sign-in.'),
  ('Delivery selection', 'Present available delivery methods and quoted charges.', 'Changing address refreshes eligible methods and totals before payment.'),
  ('Payment handoff', 'Consume the separately owned payment integration and handle its success, cancellation and failure results.', 'Cancelled payment returns to checkout without presenting a successful order.'),
  ('Order confirmation', 'Show the server-confirmed order reference and summary after successful placement.', 'Refreshing confirmation does not place a second order.')]),
 ('Accounts', [
  ('Customer sign-in', 'Consume the existing identity service for sign-in and sign-out.', 'An expired session returns the shopper to sign-in without leaking cached account details.'),
  ('Order history', 'Display the signed-in customer order list and details.', 'Unauthenticated requests cannot view another customer order.'),
  ('Address book', 'Support creating, editing and selecting account delivery addresses.', 'A rejected address change retains the previous saved address.'),
  ('Password recovery', 'Offer the existing identity service password-recovery journey.', 'The response does not reveal whether an email address exists.')]),
 ('Content and experience', [
  ('Global navigation', 'Render centrally managed category navigation across storefront routes.', 'The current category remains navigable when optional campaign content fails.'),
  ('Campaign placement', 'Consume approved campaign content slots without hard-coding dated promotions.', 'Expired campaigns disappear according to the content schedule.'),
  ('Accessibility baseline', 'Make the phase-one journeys keyboard usable with labelled controls and announced errors.', 'Focus order remains logical and dynamic catalog errors are announced.'),
  ('Analytics consent', 'Emit optional analytics only after the relevant consent choice.', 'Declining analytics leaves shopping functionality usable.')]),
 ('Migration readiness', [
  ('Legacy route redirects', 'Maintain a reviewed mapping from legacy category and product URLs to SPA routes.', 'Every approved mapping resolves without a redirect loop.'),
  ('Catalog reconciliation', 'Compare representative migrated product and variant counts against SFCC.', 'Missing variants and price discrepancies are reported before release.'),
  ('Release rollback', 'Maintain an operator-reviewed rollback plan for the storefront cutover.', 'A rehearsal records the restored legacy route and ownership of unresolved incidents.'),
  ('Operational monitoring', 'Record failed commerce calls and route errors with trace identifiers.', 'Operators can correlate a failed shopper request without logging payment secrets.')])]


def build():
    issues, requirements = [], []
    for area_index, (area, stories) in enumerate(AREAS, 1):
        epic = f'EPIC-{area_index:02}'
        issues.append({'fixture_id': epic, 'type': 'Epic', 'title': f'[DEMO] {area}', 'body': NOTICE+f'\n\nOwner workstream: {area}. This epic owns its RQ/WBS rows; dependencies on other workstreams do not transfer delivery ownership.', 'labels': ['demo-spa-migration', epic.lower()]})
        for item_index, (title, requirement, acceptance) in enumerate(stories, 1):
            number = len(requirements)+1
            rq, wbs = f'RQ-SPA-{number:03}', f'{area_index}.{item_index}'
            disposition = 'deferred' if title == 'Search suggestions' else 'phase-one baseline'
            row = {'id': rq, 'wbs': wbs, 'owner': area, 'title': title, 'requirement': requirement, 'acceptance': acceptance, 'disposition': disposition}
            requirements.append(row)
            issues.append({'fixture_id': rq, 'type': 'Story', 'epic': epic, 'title': f'[DEMO] {rq} {title}', 'labels': ['demo-spa-migration', rq.lower()],
                           'body': NOTICE+f'\n\nRQ: {rq}\nWBS: {wbs}\nOwner: {area}\nDisposition: {disposition}\n\nRequirement\n{requirement}\n\nAcceptance criteria\n{acceptance}\n\nSource authority: fictional baseline requirements register. Workflow artifact approval must come from the actual reviewer, never from this fixture.'})
        issues.append({'fixture_id': f'BUG-{area_index:02}', 'type': 'Bug', 'epic': epic, 'title': f'[DEMO] Legacy regression evidence — {area}', 'labels': ['demo-spa-migration', f'bug-{area_index:02}'],
                       'body': NOTICE+f'\n\nObserved in the fictional legacy test run: {stories[0][0]} fails under interrupted network conditions.\nSteps: open the relevant journey; interrupt the backing request; retry.\nExpected: preserve the last confirmed state and explain the failure.\nActual: legacy screen loses context.\nThis is supporting evidence, not approval to expand the new scope.'})
        issues.append({'fixture_id': f'TASK-{area_index:02}', 'type': 'Task', 'epic': epic, 'title': f'[DEMO] Evidence and ownership review — {area}', 'labels': ['demo-spa-migration', f'task-{area_index:02}'],
                       'body': NOTICE+f'\n\nCollect baseline evidence for {area}, verify the RQ/WBS mapping and record open decisions.\nDone when every owned row has a source and acceptance criterion and no unapproved change request is marked confirmed.\nActual workflow review is pending.'})
    register='| RQ | WBS | Owner | Requirement | Acceptance | Disposition |\n|---|---|---|---|---|---|\n'+'\n'.join('| '+' | '.join([r['id'],r['wbs'],r['owner'],r['requirement'],r['acceptance'],r['disposition']])+' |' for r in requirements)
    documents=[
      ('Programme overview','Linden & Rowe is a fictional apparel retailer migrating a legacy Salesforce Commerce Cloud storefront into an SPA. Phase one covers the existing shopping journeys. The linen-polo category is the first functional discovery slice. Charles Tyrwhitt is a public reference, not the client.\n\nReference: https://www.charlestyrwhitt.com/us/shirts-polo-shirts/linen-polo-shirts/\n\nAll stakeholder names, project dates, requirements, bugs and architecture statements in this corpus are invented demo data. No workflow approval has been granted.'),
      ('Baseline requirements register',register),
      ('Legacy estate inventory','Fictional baseline: SFCC owns catalog, variants, pricing, promotions, baskets and orders. The legacy frontend renders templates on the server. CMS campaign content is centrally authored. Catalog browsing, product detail and search have separate routes. Backend replacement is not in this programme. These are demo assumptions, not observations about the reference retailer.'),
      ('Target boundaries','The SPA owns shopper interaction and client routing. SFCC remains the system of record. Payment and identity connector work are separate service changes. The functional linen-polo change consumes the established catalog adapter; no authentication, billing or proxy decisions remain open for that adapter in this fictional baseline. Any new connector decision triggers a separate integration discovery.'),
      ('Workstream ownership','\n'.join(f'{i}. {a}: named owner of its WBS prefix {i}. Other workstreams consume it as a dependency.' for i,(a,_) in enumerate(AREAS,1))),
      ('Linen-polo discovery workshop','Fictional participants: Maya Reed (commerce lead), Theo Vale (frontend), Iris Cole (QA). They request the five Category browsing baseline requirements. Proposed result: browsing-only functional scope, retaining SFCC. This workshop does not approve a generated artifact. Checkout, search and account implementation stay with their named workstreams.'),
      ('Browsing baseline specification','Existing capabilities: global-navigation and product-catalog. The category browsing change modifies these and adds shareable category-state behaviour. Existing adapter contract: category identifier and optional filters/sort in; products, prices, available facets and request outcome out. The adapter already owns SFCC connectivity. No new API integration is owned by the functional slice.'),
      ('Linen-polo merchandising examples','Fictional products: LP-100 linen polo in slate, LP-110 in olive, LP-120 in sand. Fictional variants use sizes S–XL. Prices and promotions are supplied by SFCC, never copied permanently from the reference site. Include available, unavailable and discounted variants in acceptance evidence.'),
      ('Filter behaviour notes','The baseline includes colour and size. A workshop suggestion adds fabric weight, but it is unapproved CR-DEMO-01 and must not enter confirmed scope. Multi-filter behaviour follows the baseline requirement. Visual styling is a design refinement; whether a new filter belongs is a scope decision.'),
      ('Navigation and URL notes','Direct loading, refresh and back/forward must preserve supported selections. The legacy redirect map is owned by Migration readiness, WBS 8.1. Category browsing consumes the reviewed route, rather than taking ownership of the whole redirect programme.'),
      ('Accessibility review notes','Keyboard-operable filter controls, visible focus, labelled sort selection and announced loading failures support the existing accessibility baseline. Formal audit results are not fabricated: actual verification remains pending. Proposed visual treatment is discussed at specification review.'),
      ('Content and analytics boundaries','Campaign placement and analytics consent have separate named workstreams, WBS 7.2 and 7.4. Category browsing must not invent a tracking contract or a new campaign editor. The source register determines acceptance ownership.'),
      ('Catalog integration contract','The fictional pre-existing catalog adapter returns a request ID, products, prices, facets and a success/error outcome. Server pricing is authoritative. Credential exchange and rate limits belong to its platform owner. Any change to that boundary requires integration discovery and its own ISD.'),
      ('Checkout and payment dependencies','Checkout consumes the independently owned payment integration. Payment authorisation, callbacks and tokenisation are not functional linen-polo discovery. Do not merge payment with catalog as one integration deliverable.'),
      ('Customer identity dependencies','Identity service sign-in, session expiry and recovery are owned by Accounts. No customer migration or identity provider replacement is approved by this programme brief. A request to replace either is new scope.'),
      ('Change request log','CR-DEMO-01: fabric-weight filter, proposed, not approved. CR-DEMO-02: reserve in store, proposed, not approved. Both add WHAT and must stay outside confirmed requirements. Search suggestions already exist as a deferred signed baseline row and are not a new change request.'),
      ('Conflicting stakeholder notes','Earlier fictional note: a network failure may show an empty category. Later baseline RQ-SPA-005 explicitly distinguishes failure from zero products and is authoritative. Discovery should cite the conflict and preserve the baseline, not merge both behaviours.'),
      ('Specification review agenda','Open design refinements: exact loading indicator, retry copy and focus-restoration treatment. Proposed decisions require actual review in the workflow browser. The agenda is not a signed scope decision. No fixture may mark the generated review bundle approved.'),
      ('Migration cutover runbook','Proposed sequence: reconcile sample catalog data, verify reviewed routes, rehearse rollback, obtain release approval, switch traffic and observe failures. Release operation is outside this requirements-to-FSD demonstration. No deployment approval is implied.'),
      ('Acceptance and evidence index','Acceptance evidence must trace to RQ/WBS, actual Jira IDs and Confluence pages created by the seed manifest. Select the relevant source subset explicitly. Generated context and specifications are outputs and must be excluded from future source snapshots. The end-to-end demonstration stops for actual context approval and again before publication.')]
    pages=[{'fixture_id': f'PAGE-{i:02}', 'title':'[DEMO] '+title, 'body':NOTICE+'\n\n'+body, 'labels':['demo-spa-migration',f'page-{i:02}']} for i,(title,body) in enumerate(documents,1)]
    assert len(issues)==60 and len(pages)==20 and len(requirements)==36
    return {'dataset':'demo-spa-migration','version':1,'project':'EAICOMM','space':'EAICOMM','issues':issues,'pages':pages}

if __name__=='__main__':
    target=ROOT/'demo/fixtures/requirements-migration/corpus.json'
    target.write_text(json.dumps(build(),indent=2,ensure_ascii=False)+'\n')
    print('Prepared 60 issues, 20 pages and 36 traced requirements')
