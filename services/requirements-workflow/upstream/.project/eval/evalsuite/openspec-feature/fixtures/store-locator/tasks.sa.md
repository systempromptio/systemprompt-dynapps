# Cartora Maps Integration — Tasks

## Solution Overview

<!-- Concise executive summary of the capability this queue implements — projected from the discovery
     Executive Summary + the Technical Design Overview. Orientation only; scope boundaries and per-phase
     detail live in the upstream docs and the phases below. -->

This change delivers an agnostic Cartora maps integration capability: a governed, reusable way for
storefront features to geocode and render maps without each reinventing API keys, billing limits, or
privacy handling. It publishes a server-side geocode interface — a custom SCAPI endpoint backed by a
quota-aware Cartora service that resolves a postcode/city to proximity-ranked coordinates — and a
lazy-loaded map-rendering entrypoint that consuming features call on demand; the vendor key stays
server-side or domain-restricted, consumers hold none, and shopper coordinates stay transient. Map
style/type and markers are consumed from the brand team's published Cartora presentation by reference
(with a vendor-default fallback), not built here. Delivered as three published phases; consumer UI and
store data stay out of scope (owned by the Store Locator change).

## Planning Source

| Source | Location |
|---|---|
| Technical design (decomposition + high-level architecture) | `openspec/changes/add-cartora-maps-integration/technical-design.md` |
| Behaviour specs (RQ/WBS trace, behaviour bounds) | `openspec/changes/add-cartora-maps-integration/specs/**/*.md` |
| Delivery stories | `openspec/changes/add-cartora-maps-integration/stories.md` |
| ISD (engineering contract / publication context) | `openspec/changes/add-cartora-maps-integration/isd/cartora-maps-integration.md` |
| Framework conventions the low-level tasks author against | `b2c-webservices`, `b2c-custom-api-development`, `b2c-custom-caches`, `b2c-metadata`, `sfnext-performance` |

## Story Trace

| Story | Title | Phase |
|---|---|---|
| `TBD-1` | 3PI.MAP.GEO — Geocoding with proximity ranking | P1 |
| `TBD-2` | 3PI.MAP.RND — Governed map rendering | P2 |
| `TBD-3` | 3PI.MAP.STY — Consumed map presentation (style and map type) | P3 |
| `TBD-4` | 3PI.MAP.MRK — Consumed store markers | P3 |
| Cross-cutting | Shopper coordinate privacy (RQ-711), server-side key handling, unit test setup, verification | Privacy woven into P1 (server key) and P2 (domain-restricted key); Test Setup and Verification sections |

## Prerequisites

- [ ] 0.1 **@SHARED** Confirm the client's platform owner will issue the domain-restricted, origin-locked Cartora key before real-vendor verification and first release (grooming P1, DEFERRED-APPROVED); develop and test mock-first until it lands
- [ ] 0.2 **@SHARED** Confirm the client's Cartora account has quota and billing guardrails in place (design: quota/billing guardrails before load)

## Test Setup

<!-- TDD red phase: developer-owned UNIT tests written before implementation. Backend: `int_cartora`
     cartridge unit suite (vendor service mocked, fixtures follow the design data mapping). Frontend:
     Vitest for the lazy map entrypoint. Each test targets an interface authored in a Delivery Phase and
     names the phase it proves. Cross-cutting — NOT projected to a Jira sub-task. Functional testing
     (E2E / manual QA / a11y / UAT) is QA-owned and out of this pipeline — never queue it here. -->

### Unit Tests

- [ ] 1.1 **@BE** `cartoraGeocode.test.js` — vendor mapping + error mapping: valid postcode, valid city, unresolvable input; `results[].lat`/`lng` → `coordinates`, `results[].score` → `proximityRanking`, `results[].label` dropped; vendor service mocked (`mockCall`) — RQ-711 (P1)
- [ ] 1.2 **@BE** `cartoraGeocode.test.js` (cache) — bounded-lifetime hit/miss against `CartoraGeocodeCache` (24h TTL, Q2) — RQ-711 (P1)
- [ ] 1.3 **@BE** `cartoraGeocodeService.test.js` — credential-handling: the vendor key is read from the service credential and never appears in a browser-visible request or an unfiltered log line (`filterLogMessage`) — RQ-711 (P1)
- [ ] 1.4 **@FE** `entrypoint.test.ts` — lazy gate: the Cartora Maps SDK bundle loads only on consumer demand and stays off the critical path — RQ-712 (P2)
- [ ] 1.5 **@FE** `entrypoint.test.ts` (presentation) — the map inits with the configured published Map Style ID and falls back to the vendor default when it cannot be resolved; markers plot from the published set with a vendor-default fallback — RQ-713, RQ-714 (P3)
- [ ] 1.6 **@SHARED** Verify all new developer-owned unit tests fail before implementation (TDD red phase)

## Phase Processing Rules

<!-- INVARIANT house-standard block — identical in every tasks.md (fixed by the template; reproduce it
     verbatim, do not re-author). How /opsx:apply processes EVERY phase below; each phase's `Process:`
     slot links here. Governs apply-time execution only — it does NOT replace `## Review & Approval`
     (approve the whole plan before apply starts). Formal functional / regression / UAT stays QA-owned and
     out of this pipeline (no section). The per-phase `Verify` blocks are a PLAN the agent executes at
     apply time per `verification-standards.mdc`; authoring the plan here is in scope, running it is an
     apply-time action. -->

Each `### P#` is delivered as an independent, human-approved increment. Apply runs one phase at a time in
`Depends on:` order, following `verification-standards.mdc` (browser verification is mandatory; code
review is not verification; `[x]` only when acceptance is fully met):

1. **Implement to green** — complete the phase's tasks (Files / Interface / Steps); the phase's Test-Setup
   unit tests start red and are driven to pass.
2. **Verify (independent self-check)** — a fresh, read-only pass that did NOT write the code grades the
   phase against its frozen `Verify` acceptance: run the phase's Test-Setup unit tests (attach output);
   walk the phase's **Scenario** live on the running app (a manual-style flow — NOT an authored automated
   e2e suite and NOT human QA). Capture evidence — an @FE phase drives the browser with `playwright-cli`
   (navigate, act, screenshot the key states: default / empty / loading / error) and checks a clean
   console; an @BE / integration phase captures request/response + log lines + cache behaviour. Report
   defects with file:line evidence; emit a **Verdict**: `SHIP` · `SHIP_WITH_HUMAN_REVIEW` · `ITERATE` ·
   `BLOCKED`.
3. **Iterate** — on `ITERATE`, fix only the reported defects and re-verify fresh (max 3 rounds). `BLOCKED`
   stops and is recorded in the phase `Status` with its blocker.
4. **Stop for human approval** — apply STOPS and hands the human the phase's evidence + verdict; the agent
   never self-approves. Every phase is approved by a human before the next starts.
5. **Commit** — an approved phase lands as ONE commit before the next phase starts (branch, message
   format and OpenSpec flow per the `git-commit` skill), carrying the phase's code together with its
   `tasks.md` updates: the phase's `- [x]` flips and its `Status`. Record the short SHA in that `Status`
   (`approved (<sha>)`), so every phase is auditable back to a single revision. An unapproved,
   `ITERATE`, or `BLOCKED` phase is never committed.
6. **Advance** — only an approved, committed phase unlocks the next.

**Sensitive tier consequence** — a `tier: sensitive` phase (the tier is decided when the queue is
authored) has its Verdict forced to `SHIP_WITH_HUMAN_REVIEW` (never auto-`SHIP`), its Verify carries a
security pass on the recorded bound, and human approval is mandatory. Evidence over assertion: a phase is
"verified" only with durable artifacts, never "looks good".

## Delivery Phases

Each `### P#` closes one consumer-callable increment of the capability (its Story), delivered and approved
on its own; apply processes every phase through the [Phase Processing Rules](#phase-processing-rules) above
(implement → independent verify → stop for human approval → next). What we're building overall is in
[Solution Overview](#solution-overview).

<!-- Each `### P#` is one Delivery Phase projected from technical-design.md → Delivery Phases; it links to
     exactly ONE story (P3 carries TBD-3 + TBD-4, which close through the same published-presentation
     mechanism) and becomes one Jira sub-task under that story (summary = phase title; description = the
     `Delivers` line + the child task TITLES — Interface/Steps/Done-when stay here for /opsx:apply and are
     NOT pushed to Jira). Sequencing lives ONLY in each phase's `Depends on:`; tasks inside a phase run
     top-to-bottom. Each phase also carries a `Process:` slot (backlink + tier), a `Verify` block (frozen
     acceptance + scenario + evidence + verdict the agent self-checks at apply time), and a `Status:` slot
     (pending → implemented → verified → approved, or blocked) — all `-` sub-content, never `- [ ]`.
     Each child task is a column-0 `- [ ]` with exactly one @FE/@BE/@SHARED tag, then INDENTED low-level
     content (Files / Interface / Steps / Done-when / Trace) authored per Variant B: the named identifiers
     come from the technical-design Building Blocks; the signatures/contracts are authored here, grounded in
     the SFCC conventions (`b2c-webservices`, `b2c-custom-api-development`, `b2c-custom-caches`) and the
     resolved planning decisions (Q1 endpoint contract, Q2 24h TTL). -->

### P1 — Server-side geocode interface

- **Story:** `TBD-1`
- **Delivers:** a stable, server-side custom SCAPI geocode endpoint that resolves a postcode/city to proximity-ranked coordinates through a governed, quota-aware Cartora service; the vendor key stays server-side and shopper coordinates stay transient — RQ-711 · 3PI.MAP.GEO
- **Areas:** @BE
- **Depends on:** —
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · **bounds: vendor-credential (P1.1, P1.5 — the service definition and its Business Manager credential hold the Cartora key), shopper-coordinates (P1.2 — the geocode implementation receives a postcode and returns coordinates, keeping them transient)** · **tier: sensitive**
- **Jira:** —

- [ ] P1.1 **@BE** `int_cartora` cartridge + Cartora Geocoding HTTP service definition
  - **Files:** `cartridges/int_cartora/cartridge/scripts/services/cartoraGeocodeService.js` (new) · `cartridges/int_cartora/package.json` (new)
  - **Interface:**

    ```js
    // LocalServiceRegistry HTTP service; the credential holds the domain-restricted key (server-side only)
    var LocalServiceRegistry = require('dw/svc/LocalServiceRegistry');
    module.exports = LocalServiceRegistry.createService('cartora.geocode.http', {
      createRequest: function (svc, params) {
        svc.setRequestMethod('GET');
        // GET {credential.URL}/v1/geocode?q=&country=&limit=&key={credential secret}
        svc.addParam('q', params.q);
        svc.addParam('country', params.country);
        svc.addParam('limit', params.limit);
        return null;
      },
      parseResponse: function (svc, client) { return JSON.parse(client.text); }, // { results: [{ lat, lng, score, label }] }
      filterLogMessage: function (msg) { return msg.replace(/key=[^&\s]+/g, 'key=***'); }, // never log the key
      mockCall: function (svc, req) { /* fixtures per the design data mapping until the key is issued (P1) */ }
    });
    ```

  - **Steps:**
    1. Scaffold the `int_cartora` cartridge; register the service `cartora.geocode.http` via `LocalServiceRegistry`.
    2. In `createRequest`, build the HTTPS `GET …/v1/geocode` call with `q`/`country`/`limit`; the key comes from the service credential, sent as a query parameter — never hard-coded, never client-visible.
    3. Add `filterLogMessage` to mask the key; add `mockCall` returning design-mapped fixtures so tests run before the real key lands.
  - **Done when:** the service compiles, calls `…/v1/geocode` with the credential key server-side, and runs mock-first with no key in any log line.
  - **Trace:** RQ-711

- [ ] P1.2 **@BE** Server-side geocode implementation + vendor→contract mapping
  - **Files:** `cartridges/int_cartora/cartridge/scripts/geocode/cartoraGeocode.js` (new)
  - **Interface:**

    ```js
    /**
     * Resolve a postcode/city to proximity-ranked coordinates. Coordinates are transient:
     * never persisted, never logged, never sent to analytics.
     * @returns {{ results: Array<{ coordinates: { lat: number, lng: number }, proximityRanking: number }> }
     *          | { error: { type: string, message: string } }}  // error result carries NO coordinates
     */
    function geocode(q, country, limit) {}
    module.exports = { geocode };
    ```

  - **Steps:**
    1. Call `cartora.geocode.http` (through the cache wrapper, P1.3); on `result.ok`, map each vendor row `lat`/`lng` → `coordinates`, `score` → `proximityRanking`, and DROP `label` (not part of the consumer contract).
    2. For input Cartora cannot resolve, return the clear `error` result with no coordinates.
    3. Do not persist, log, or emit coordinates anywhere.
  - **Done when:** valid postcode/city return proximity-ranked `coordinates`; unresolvable input returns a clear error with no coordinates; `label` never leaks into the contract.
  - **Trace:** RQ-711

- [ ] P1.3 **@BE** Quota-aware geocode cache (bounded 24h TTL, Planning Q2)
  - **Files:** `cartridges/int_cartora/caches.json` (new) · `cartridges/int_cartora/package.json` (edit)
  - **Interface:**

    ```json
    // caches.json — 24h TTL respects the Cartora quota (Planning decision Q2)
    { "caches": [ { "id": "CartoraGeocodeCache", "expireAfterSeconds": 86400 } ] }
    ```
    ```js
    // usage in cartoraGeocode.js — populate on miss via get(key, loader)
    var CacheMgr = require('dw/system/CacheMgr');
    var cache = CacheMgr.getCache('CartoraGeocodeCache');
    var mapped = cache.get(country + '|' + limit + '|' + q, function () { return callAndMapService(q, country, limit); });
    ```

  - **Steps:**
    1. Define `CartoraGeocodeCache` with `expireAfterSeconds: 86400` in `caches.json`; add the `"caches": "./caches.json"` field to the cartridge `package.json` so the region registers.
    2. Wrap the service call in `cache.get(key, loader)`; scope the key by `q`/`country`/`limit`; never put shopper-identifying data in the key.
  - **Done when:** repeat geocode requests within 24h hit the cache; the region registers (no missing-registration error).
  - **Trace:** RQ-711

- [ ] P1.4 **@BE** Custom SCAPI geocode endpoint + schema (Planning Q1) — `cartora-geo` v1, `GET /geocode`
  - **Files:** `cartridges/int_cartora/cartridge/rest-apis/cartora-geo/api.json` (new) · `.../schema.json` (new) · `.../geocodeApi.js` (new)
  - **Interface:**

    ```jsonc
    // api.json — custom SCAPI registration (Planning Q1)
    { "name": "cartora-geo", "version": "1.0.0",
      "endpoints": [ { "method": "GET", "path": "/geocode", "schema": "schema.json", "implementation": "geocodeApi" } ] }
    // schema.json — request/response/error contract per Q1
    //   request:  { q: string, country: string, limit: integer }
    //   response: { results: [ { coordinates: { lat: number, lng: number }, proximityRanking: number } ] }
    //   error:    { error: { type: string, message: string } }   // carries NO coordinates
    ```
    ```js
    // geocodeApi.js — wire the endpoint to the cached geocode implementation
    var cartoraGeocode = require('*/cartridge/scripts/geocode/cartoraGeocode');
    exports.get = function (ctx) { /* read q/country/limit; return cartoraGeocode.geocode(...) as the Q1 response/error */ };
    ```

  - **Steps:**
    1. Author `api.json` registering the `cartora-geo` v1 API with `GET /geocode`.
    2. Author `schema.json` with the Q1 request (`q`, `country`, `limit`), response (`results[].{coordinates,proximityRanking}`), and error (`{ error: { type, message } }`, no coordinates).
    3. Implement `geocodeApi.get` calling `cartoraGeocode.geocode(...)`; keep coordinates transient — never persisted, logged, or sent to analytics.
  - **Done when:** `cartora-geo` v1 `GET /geocode` returns the Q1 response for resolvable input and the Q1 error (no coordinates) otherwise; consumers build against this contract holding no key.
  - **Trace:** RQ-711

- [ ] P1.5 **@BE** Business Manager service + credential metadata (server-side key)
  - **Files:** `cartridges/int_cartora/metadata/services.xml` (new — service config + profile + credential)
  - **Interface:**

    ```xml
    <!-- services.xml — HTTP service, profile (rate limit + circuit breaker), and credential -->
    <service-credential credential-id="cartora.geocode.cred">
      <url>https://api.cartora.example</url> <!-- key lives in the credential secret; server-side only -->
    </service-credential>
    <service service-id="cartora.geocode.http" credential-id="cartora.geocode.cred" profile-id="cartora.default" enabled="true"/>
    ```

  - **Steps:**
    1. Define the service configuration, a profile (rate limit + circuit breaker to respect quota), and the credential holding the domain-restricted key.
    2. Keep the key in the credential only — it never reaches the browser; the key is issued by the client's platform owner before first release (grooming P1).
  - **Done when:** the service/credential import cleanly; the key is server-side only; real-vendor verification is gated on P1 key issuance.
  - **Trace:** RQ-711

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 `cartora-geo` v1 `GET /geocode` returns the Q1 response `results[].{coordinates, proximityRanking}` for a resolvable postcode/city; A2 unresolvable input → the Q1 error `{ error: { type, message } }` with NO coordinates; A3 vendor `label` is dropped from the contract; A4 a repeat request within 24h hits `CartoraGeocodeCache` (Q2); A5 the key is server-side only, `filterLogMessage` masks it, and coordinates are never persisted, logged, or sent to analytics — RQ-711.
  - **Unit:** run `1.1`, `1.2`, `1.3` (Test Setup) → attach output.
  - **Scenario (live, no screenshots — @BE):**
    1. Call `GET /geocode?q=…&country=…&limit=…` with a valid postcode → capture request/response; assert the Q1 response shape (evidence: request/response capture)
    2. Repeat the same request → assert a cache hit within the 24h TTL (evidence: cache hit/miss log)
    3. Call with unresolvable input → assert the Q1 error with no coordinates
    4. Inspect the service log → the key is masked and no coordinates appear (evidence: redacted service log)
  - **Security pass (sensitive):** the key exists only in the service credential (never in a browser-visible request or an unfiltered log line); shopper coordinates stay transient.
  - **Verdict:** SHIP_WITH_HUMAN_REVIEW (sensitive → always to human)
- **Status:** pending — real-vendor (real-key) re-verify BLOCKED on Cartora key issuance (0.1, DEFERRED-APPROVED); develop + verify mock-first now, re-verify on issuance

### P2 — Governed map rendering entrypoint

- **Story:** `TBD-2`
- **Delivers:** a lazy-loadable Cartora Maps entrypoint that consuming features import on demand; the SDK stays off the consumer's critical path and initialises with the capability's domain-restricted key — consumers hold no key — RQ-712 · 3PI.MAP.RND
- **Areas:** @FE
- **Depends on:** —
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · **bounds: vendor-credential (P2.2 — SDK init loads the domain-restricted maps key; consumers hold none)** · **tier: sensitive**
- **Jira:** —

- [ ] P2.1 **@FE** Lazy Cartora Maps SDK entrypoint
  - **Files:** `src/lib/cartora-maps/entrypoint.ts` (new)
  - **Interface:**

    ```ts
    export interface MapMarker { id: string; lat: number; lng: number }
    export interface CartoraMapOptions { container: HTMLElement; markers?: MapMarker[] }
    export interface CartoraMapInstance { setMarkers(markers: MapMarker[]): void; destroy(): void }

    // dynamically imports the Cartora Maps SDK (kept off the consumer's critical bundle) and initialises
    // the map with the capability-governed domain-restricted key + published Map Style ID
    export function loadMap(options: CartoraMapOptions): Promise<CartoraMapInstance>;
    ```

  - **Steps:**
    1. Load the SDK with a dynamic `import()` inside `loadMap` so the bundle never sits on the consumer's initial JS path (`sfnext-performance`).
    2. Initialise the map with the domain-restricted key from `getCartoraMapsConfig()` (P2.2); render markers passed by the consumer.
    3. The consumer's content stays usable before and while the map loads.
  - **Done when:** the SDK loads only when `loadMap` is called; a bundle check confirms it is absent from the consumer's critical path.
  - **Trace:** RQ-712

- [ ] P2.2 **@FE** Domain-restricted maps key + config at SDK init (consumers hold no key)
  - **Files:** `src/lib/cartora-maps/config.ts` (new)
  - **Interface:**

    ```ts
    export interface CartoraMapsConfig {
      key: string;         // domain-restricted, origin-locked; governed by the capability
      styleId: string;     // published Map Style ID (see P3)
    }
    export function getCartoraMapsConfig(): CartoraMapsConfig;  // from platform configuration; never a consumer prop
    ```

  - **Steps:**
    1. Read the domain-restricted key + style id from platform configuration; expose only through `getCartoraMapsConfig()`.
    2. Consuming features configure no Cartora key of their own; map requests carry the capability key at init.
  - **Done when:** the entrypoint initialises with the governed key and consumers supply none.
  - **Trace:** RQ-712

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 the Cartora Maps SDK loads only when `loadMap()` is called and is absent from the consumer's critical bundle; A2 the map initialises with the domain-restricted key from `getCartoraMapsConfig()` (platform config), consumers supply none; A3 the key is domain-restricted / origin-locked and never a consumer prop or a hard-coded literal — RQ-712.
  - **Unit:** run `1.4` (Test Setup) → attach output.
  - **Scenario (live, on a consumer harness):**
    1. Load the harness WITHOUT calling `loadMap` → the SDK is absent from the initial bundle (evidence: bundle report)
    2. Call `loadMap(...)` → the SDK loads and the map initialises with the config-sourced governed key (evidence: init log)
    3. Attempt a consumer-supplied key → it is NOT used; the key comes only from platform config
  - **Security pass (sensitive):** the key is sourced only from platform configuration, is domain-restricted, and is never a consumer prop or a hard-coded literal.
  - **Verdict:** SHIP_WITH_HUMAN_REVIEW (sensitive → always to human)
- **Status:** pending — real-key re-verify BLOCKED on Cartora key issuance (0.1, DEFERRED-APPROVED); develop mock-first now

### P3 — Consumed brand presentation and markers

- **Story:** `TBD-3`
- **Delivers:** the map renders against the published "Brand Street" Map Style ID held by reference (so a re-publish reaches consumers with no release) and plots stores with the published marker set; both fall back to the vendor default when the reference/marker set cannot be resolved — RQ-713 · 3PI.MAP.STY, RQ-714 · 3PI.MAP.MRK
- **Areas:** @FE
- **Depends on:** P2
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · bounds: none · tier: standard
- **Jira:** —

- [ ] P3.1 **@FE** Published Map Style ID by reference + vendor-default fallback
  - **Files:** `src/lib/cartora-maps/entrypoint.ts` (edit) · `src/lib/cartora-maps/config.ts` (edit)
  - **Steps:**
    1. Initialise the SDK with the configured `styleId` (v1 = the brand team's published "Brand Street" style) held as a reference, never a copied asset — a re-published style reaches consumers with no release.
    2. When the style reference cannot be resolved, initialise on the vendor's default presentation so the consumer's render does not fail.
  - **Done when:** the map renders against the published style; an unresolvable reference still yields a usable map on the vendor default.
  - **Trace:** RQ-713

- [ ] P3.2 **@FE** Plot stores with the published marker set + default-marker fallback
  - **Files:** `src/lib/cartora-maps/entrypoint.ts` (edit)
  - **Steps:**
    1. Plot store locations using the marker set carried by the published presentation; define no marker artwork in this capability.
    2. When the published presentation carries no custom marker set, plot with the vendor's default marker.
  - **Done when:** brand-authored markers plot the stores, and a presentation without custom markers falls back to the vendor default marker.
  - **Trace:** RQ-714

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 the map initialises with the configured published Map Style ID held by reference (a re-publish reaches consumers with no release); A2 an unresolvable style reference falls back to the vendor default (usable map); A3 stores plot from the published marker set; A4 a presentation with no custom markers falls back to the vendor-default marker — RQ-713, RQ-714.
  - **Unit:** run `1.5` (Test Setup) → attach output.
  - **Scenario (live, on a consumer harness):**
    1. Initialise with the published `styleId` → the map renders that presentation (evidence: init log / render assertion)
    2. Simulate an unresolvable style reference → the map initialises on the vendor default
    3. Plot with the published marker set → markers render; simulate no custom marker set → vendor-default markers
  - **Verdict:** SHIP (then human approval)
- **Status:** pending

## Verification

<!-- Developer-owned automated checks only; they run across the whole change, not a specific phase.
     Cross-cutting — NOT projected to a Jira sub-task. Functional testing (E2E / manual QA / a11y / UAT)
     is QA-owned and out of this pipeline. -->

### Automated (developer-owned, must pass in CI)

- [ ] 4.1 **@SHARED** Unit tests pass: the frontend Vitest suite and the `int_cartora` cartridge unit suite
- [ ] 4.2 **@SHARED** No regressions in existing developer-owned test suites
- [ ] 4.3 **@SHARED** ESLint passes on changed frontend files
- [ ] 4.4 **@SHARED** TypeScript check passes: `npx tsc --noEmit`
- [ ] 4.5 **@FE** Bundle check on a consumer harness: the Cartora Maps SDK stays off the consumer's critical JS path and loads only on demand (see the `sfnext-performance` skill) — RQ-712

## Review & Approval

- **Approved for implementation:** no
- **Approved by:** —
- **Date:** —
