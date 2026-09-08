# SLS - Store Locator — Tasks

## Solution Overview

<!-- Concise executive summary of the solution this queue implements — projected from the proposal
     Executive Summary + the Technical Design Overview. Orientation only; scope boundaries and per-phase
     detail live in the upstream docs and the phases below. -->

This change replaces the legacy static `/stores` directory with a modern Storefront Next store locator:
shoppers search by postcode, city, or current location and get an accessible, proximity-ordered result
list, an interactive map lazily loaded off the critical path and synchronized with that list, store
details and services, service filtering, and directions handed off to their own map app. The
maps/geocoding and address-autocomplete capabilities are consumed through feature-side seams — the
storefront holds no vendor key — and store data comes from the existing Stores API. Delivered as seven
vertical, independently approved phases, with accessibility and localization woven into each rather than
deferred to a clean-up phase; the legacy directory is retired at launch.

## Planning Source

| Source | Location |
|---|---|
| Technical design (decomposition + high-level architecture) | `openspec/changes/add-store-locator/technical-design.md` |
| Behaviour specs (RQ/WBS trace, behaviour bounds) | `openspec/changes/add-store-locator/specs/**/*.md` |
| Delivery stories | `openspec/changes/add-store-locator/stories.md` |
| FSD (client-facing context) | `openspec/changes/add-store-locator/fsd/store-locator.md` |
| Framework conventions the low-level tasks author against | `sfnext-routing`, `sfnext-data-fetching`, `sfnext-components`, `sfnext-state-management`, `sfnext-i18n`, `sfnext-performance`, `sfnext-test-ids` |

## Story Trace

| Story | Title | Phase |
|---|---|---|
| `TBD-1` | SLS.SRCH — Store search by postal code, city, or location | P1 |
| `TBD-2` | SLS.AC — Address autocomplete suggestions | P2 |
| `TBD-3` | SLS.MAP — Interactive map with a synchronized list | P3 |
| `TBD-4` | SLS.DTL — Store details | P4 |
| `TBD-5` | SLS.FLT — Filter stores by service | P5 |
| `TBD-6` | SLS.DIR — Get directions | P6 |
| `TBD-9` | GH.UTL — Navigation entry and retired directory URL | P7 |
| `TBD-8` | SLS.I18N — Localization | Cross-cutting — copy key-backed per surface as each phase adds it; the `store-locator` catalog + sync/compile live in P1 (`#### Translation Sync`); no separate localization phase (per technical-design) |
| `TBD-7` | SLS.A11Y — Accessible, keyboard-operable store locator | Cross-cutting — built into every phase per surface (semantic markup / `sfnext-test-ids`); functional a11y is QA-owned, out of this pipeline |
| Cross-cutting | Capability seams, feature data types, unit test setup, verification | Seams + types folded into P1; Test Setup and Verification sections |

## Prerequisites

- [ ] 0.1 **@SHARED** Confirm delivery status of the Cartora maps/geocoding capability (grooming P1) and the Addressly autocomplete capability (grooming P2); both are DEFERRED-APPROVED, so build and test against the Q1 seams now and release the geocoded-search, map, and autocomplete paths only after the capabilities deliver
- [ ] 0.2 **@SHARED** Confirm the commerce Stores API is available (grooming P3 — already integrated); no new store data model in v1 — the loader reads it through the existing client from `createApiClients(context)`

## Test Setup

<!-- TDD red phase: developer-owned UNIT tests written before implementation (Vitest, co-located
     `*.test.ts` / `*.test.tsx`, `sfnext-test-ids`). Each test targets an interface authored in a Delivery
     Phase and names the phase it proves. Cross-cutting — NOT projected to a Jira sub-task. Functional
     testing (E2E / manual QA / a11y / UAT) is QA-owned and out of this pipeline — never queue it here. -->

### Unit Tests

- [ ] 1.1 **@FE** `_app.store-locator.test.ts` — loader: search-param parsing (`q`, `service`), proximity ordering pass-through, service-filter narrowing; Cartora/Addressly seams stubbed (Q1) — RQ-701, RQ-705 (P1, P5)
- [ ] 1.2 **@FE** `use-address-suggestions.test.ts` — suggestion adapter against the stubbed Addressly seam; manual submit still works when suggestions are unavailable — RQ-710 (P2)
- [ ] 1.3 **@FE** `_app.store-locator.test.ts` (action) — use-my-location submission asserts coordinates stay out of the URL, history, and the analytics payload — RQ-701 (P1)
- [ ] 1.4 **@FE** `store-locator-context.test.tsx` — list↔map selection sync both ways (list→map and map→list) — RQ-702 (P3)
- [ ] 1.5 **@FE** `store-detail.test.tsx` — renders regular weekly hours and services; no holiday-hours field (grooming Q1) — RQ-703 (P4)
- [ ] 1.6 **@FE** `get-directions-link.test.tsx` — builds a native map deep link and makes no maps-vendor call — RQ-704 (P6)
- [ ] 1.7 **@FE** `_app.stores.test.ts` — retired-URL loader returns a permanent (301) redirect to `/store-locator` — RQ-709 (P7)
- [ ] 1.8 **@SHARED** Verify all new developer-owned unit tests fail before implementation (TDD red phase)

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

Each `### P#` closes one shopper-observable increment (its Story), delivered and approved on its own; apply
processes every phase through the [Phase Processing Rules](#phase-processing-rules) above (implement →
independent verify → stop for human approval → next). What we're building overall is in
[Solution Overview](#solution-overview).

<!-- Each `### P#` is one Delivery Phase projected from technical-design.md → Delivery Phases; it links to
     exactly ONE story and becomes one Jira sub-task under that story (summary = phase title; description =
     the `Delivers` line + the child task TITLES — the Interface/Steps/Done-when stay here for /opsx:apply
     and are NOT pushed to Jira). Sequencing lives ONLY in each phase's `Depends on:` (a phase-level
     consumer → producer edge from the technical-design Delivery Phases); no separate dependency map, no
     inline `[depends:]` — tasks inside a phase run top-to-bottom.
     Each phase also carries a `Process:` slot (backlink + tier), a `Verify` block (the frozen acceptance +
     manual-style scenario + evidence + verdict the agent self-checks at apply time), and a `Status:` slot
     (pending → implemented → verified → approved, or blocked). Those three are `-` sub-content, never
     `- [ ]`, so they stay invisible to the progress + structure gates.
     Each child task is a column-0 `- [ ]` with exactly one @FE/@BE/@SHARED tag, then INDENTED low-level
     content (Files / Interface / Steps / Done-when / Trace) authored per the Variant B contract: the named
     identifiers come from the technical-design Building Blocks; the signatures/types are authored here,
     grounded in the sfnext conventions. Indented content is invisible to the progress + structure gates. -->

### P1 — Search, results list, and the shared locator shell

- **Story:** `TBD-1`
- **Delivers:** typed postcode/city and use-my-location search returns a proximity-ordered, accessible result list on the `/store-locator` shell; shopper coordinates never reach the URL, history, or analytics — RQ-701 · SLS.SRCH
- **Areas:** @FE
- **Depends on:** —
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · **bounds: shopper-coordinates (P1.2, P1.3 — the island requests the device position and the route action receives it, never persisting it)** · **tier: sensitive**
- **Jira:** —

- [ ] P1.1 **@FE** Capability seams and feature data types — Cartora + Addressly, stubbed until they deliver (Planning Q1)
  - **Files:** `src/lib/store-locator/capabilities.ts` (new) · `src/lib/store-locator/types.ts` (new)
  - **Interface:**

    ```ts
    // types.ts — feature-side shapes for data read through the EXISTING Stores API (no new model)
    export interface GeoPoint { lat: number; lng: number }
    export interface WeeklyHours { day: number; open: string; close: string; closed?: boolean }
    export interface Store {
      id: string;
      name: string;
      address: string;            // full, display-ready address
      phone: string;
      hours: WeeklyHours[];       // regular weekly hours only — no holiday hours in v1 (grooming Q1)
      services: string[];         // opaque service ids; offered set is BM-configured (grooming Q5)
      coordinates: GeoPoint;
    }
    export interface RankedStore extends Store { distance: number }  // proximity from the search origin
    export interface LocatorConfig {  // Business-Manager configuration (grooming Q3/Q4/Q5)
      defaultRadius: number;
      radiusUnit: 'km' | 'mi';
      useMyLocationEnabled: boolean;
      pageSize: number;
      loadMoreBatch: number;
      offeredServices: string[];  // v1 seed: click-and-collect, returns, gift-wrapping
    }
    export interface AddressSuggestion { id: string; label: string; query: string }

    // capabilities.ts — the ONLY import surface for the consumed capabilities; no vendor key / SDK here (Q1 stub)
    export function geocode(query: string): Promise<GeoPoint | null>;          // Cartora, server-side
    export function suggestAddresses(input: string): Promise<AddressSuggestion[]>; // Addressly
    export function loadMapEntrypoint(): Promise<StoreMapApi>;                  // Cartora map, lazy client import
    ```

  - **Steps:**
    1. Author the types in `types.ts`; keep `services` an opaque `string[]` so the BM-configured set is honoured (do not hard-code a closed enum).
    2. Author the three seam functions in `capabilities.ts` with the signatures above; back them with typed stubs (fixed sample data) until the Cartora/Addressly capabilities deliver.
    3. Keep the map entrypoint behind `loadMapEntrypoint()` so it can be lazily imported and never lands in the initial route bundle.
  - **Done when:** the seams compile and are the single import surface for Cartora/Addressly; no vendor key or SDK import exists in feature code (Q1 stub).
  - **Trace:** RQ-701, RQ-702, RQ-710

- [ ] P1.2 **@FE** `/store-locator` route — loader (typed/city search) + action (use-my-location)
  - **Files:** `src/routes/_app.store-locator.tsx` (new)
  - **Interface:**

    ```ts
    import type { LoaderFunctionArgs, ActionFunctionArgs } from 'react-router';
    import type { GeoPoint, RankedStore, LocatorConfig } from '@/lib/store-locator/types';

    export interface StoreLocatorSearch { q?: string; service?: string }  // both are URL search params
    export interface StoreLocatorData {
      query: StoreLocatorSearch;
      origin: GeoPoint | null;          // resolved search origin; null when nothing searched / unresolved
      stores: Promise<RankedStore[]>;   // streamed, proximity-ordered; [] => empty-results state
      config: LocatorConfig;            // BM configuration for the search/list/filter surfaces
    }
    // typed/city search: read URL params, geocode via the Cartora seam, read stores, rank + filter by service
    export async function loader({ request, context }: LoaderFunctionArgs): Promise<StoreLocatorData>;
    // use-my-location: coordinates arrive in the POST body via an ephemeral fetcher (NEVER the URL)
    export async function action({ request, context }: ActionFunctionArgs): Promise<{ origin: GeoPoint; stores: RankedStore[] }>;
    export default function StoreLocatorPage({ loaderData }: { loaderData: StoreLocatorData }): JSX.Element;
    ```

  - **Steps:**
    1. In `loader`, read `q`/`service` from `new URL(request.url).searchParams` and read `LocatorConfig` from merchant config (radius/units/page size/use-my-location/offered services).
    2. When `q` is present, `await geocode(q)` for the `origin`; read stores through the existing Stores API client from `createApiClients(context)`; rank by proximity and apply the `service` filter; return `stores` as a streamed promise so the shell renders first.
    3. In `action`, read `{ lat, lng }` from the posted form data, rank stores around that origin, and return them — never echo coordinates into the URL, redirect target, or logs.
    4. `StoreLocatorPage` SSR-renders the search + accessible list shell, streaming the ranked results with `<Suspense>`/`<Await>`.
  - **Done when:** typed/city search returns a proximity-ordered `stores` promise; the shell renders server-first; `action` returns ranked stores for a coordinate POST with no coordinate leakage to URL/history/analytics.
  - **Trace:** RQ-701, RQ-702

- [ ] P1.3 **@FE** `StoreSearch` island — postcode/city input + permission-gated use-my-location
  - **Files:** `src/extensions/store-locator/components/store-search/index.tsx` (new)
  - **Interface:**

    ```ts
    import type { StoreLocatorSearch } from '@/routes/_app.store-locator';

    export interface StoreSearchProps {
      query: StoreLocatorSearch;
      useMyLocationEnabled: boolean;  // from LocatorConfig (grooming Q3); hides the button when false
    }
    export function StoreSearch(props: StoreSearchProps): JSX.Element;
    ```

  - **Steps:**
    1. Render a GET form whose text input submits `q` as a URL search param (drives the loader).
    2. Render "use my location" only when `useMyLocationEnabled`; on an explicit click, request the browser Geolocation permission, then POST the coordinates with `useFetcher().submit(form, { method: 'post' })` to the route `action` — coordinates go in the body, never the URL.
    3. On denied permission, show an explanatory message and keep postcode/city search available; send no request.
  - **Done when:** typed and use-my-location search both work; the button is hidden when BM disables it; denied permission falls back to manual search; coordinates never appear in the URL, history, or analytics.
  - **Trace:** RQ-701

- [ ] P1.4 **@FE** `StoreList` — accessible, server-rendered results list
  - **Files:** `src/extensions/store-locator/components/store-list/index.tsx` (new)
  - **Interface:**

    ```ts
    import type { RankedStore } from '@/lib/store-locator/types';

    export interface StoreListProps {
      stores: RankedStore[];
      selectedStoreId: string | null;             // from the selection context (P3)
      onSelect: (storeId: string) => void;
    }
    export function StoreList(props: StoreListProps): JSX.Element;
    ```

  - **Steps:**
    1. Render a semantic list (`<ul>` with list-item roles) that is server-rendered and usable before and while the map loads.
    2. Each item shows the store summary and a keyboard-operable select control that calls `onSelect`; reflect `selectedStoreId` as the active item.
    3. Add stable `data-testid`s per `sfnext-test-ids`; ensure focus order and screen-reader labels (RQ-706, woven a11y).
    4. Show the empty-results message when `stores` is empty.
  - **Done when:** the list renders server-first, is fully keyboard-operable, announces results to a screen reader, and stays usable before the map island loads.
  - **Trace:** RQ-702, RQ-706

#### Translation Sync

<!-- Localization is CROSS-CUTTING per the technical-design (no separate phase): each phase key-backs the
     copy for the surface it adds, and the `store-locator` catalog + sync/compile workflow is established
     here in the shell phase. Delivers the "Localization resources" Building Block (TBD-8 / RQ-707).
     Dev-owned extract/sync/compile only; functional locale verification is QA-owned, out of this pipeline. -->

- [ ] P1.5 **@FE** Establish the `store-locator` i18n namespace and key-back the shell + results copy
  - **Files:** locator locale resources (per technical-design "Localization resources" Building Block) (new) · locator surfaces (edit)
  - **Steps:**
    1. Create the `store-locator` translation namespace and replace hard-coded shell/results strings with keys (`sfnext-i18n`); each later phase adds its surface's keys to the same namespace as it is built (no end-of-project rework).
  - **Done when:** the shell and results copy are key-backed under the `store-locator` namespace with no literal strings left in those components.
  - **Trace:** RQ-707

- [ ] P1.6 **@FE** Sync / upload the locator translation keys
  - **Files:** locator locale resources (edit)
  - **Steps:**
    1. Run the `sfnext-i18n` sync/upload and confirm no `store-locator` keys are missing from the catalog.
  - **Done when:** all `store-locator` keys are present in the translation catalog with no missing entries.
  - **Trace:** RQ-707

- [ ] P1.7 **@FE** Compile translations and verify a non-default locale renders translated
  - **Files:** compiled locale bundles (build output)
  - **Steps:**
    1. Compile translations; load the locator in a non-default locale and confirm labels/messages render translated, not as raw keys.
  - **Done when:** the compiled catalog renders the locator's translated strings (not raw keys) in a non-default locale; later phases extend the same catalog as they add surfaces.
  - **Trace:** RQ-707

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 typed postcode/city → proximity-ordered list on the shell; A2 no match → empty-results state; A3 use-my-location → results returned AND coordinates absent from the URL, history, and analytics payload (privacy bound, RQ-701); A4 shell + list keyboard-operable and screen-reader-announced (RQ-706).
  - **Unit:** run `1.1`, `1.3` (Test Setup) → attach output.
  - **Scenario (live, manual-style):**
    1. Given `/store-locator` on a clean session
    2. When I type "10001" and submit → Then a proximity-ordered list renders (screenshot: default)
    3. When I search a no-match query → Then the empty-results state shows (screenshot: empty)
    4. When I use "use my location" (grant permission) → Then results render; inspect the URL, history, and analytics/network payload → Then no coordinates appear (evidence: redacted network log)
    5. When permission is denied → Then manual search still works and no request is sent
  - **Visual:** screenshots of default / empty / loading / error at desktop + mobile; console clean.
  - **Security pass (sensitive):** confirm A3 — coordinates are never serialized to the URL, history, redirect target, logs, or analytics.
  - **Verdict:** SHIP_WITH_HUMAN_REVIEW (sensitive → always to human)
- **Status:** pending — real-Cartora geocode re-verify BLOCKED on capability delivery (0.1, DEFERRED-APPROVED); implement + verify against the Q1 stub now, re-verify on delivery

### P2 — Address autocomplete

- **Story:** `TBD-2`
- **Delivers:** typing an address in the store search offers Addressly suggestions; selecting one submits the search; manual search still works when no suggestion is used — RQ-710 · SLS.AC
- **Areas:** @FE
- **Depends on:** P1
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · bounds: none · tier: standard
- **Jira:** —

- [ ] P2.1 **@FE** `useAddressSuggestions` hook wired into `StoreSearch`
  - **Files:** `src/extensions/store-locator/hooks/use-address-suggestions.ts` (new) · `src/extensions/store-locator/components/store-search/index.tsx` (edit)
  - **Interface:**

    ```ts
    import type { AddressSuggestion } from '@/lib/store-locator/types';

    export interface UseAddressSuggestions {
      suggestions: AddressSuggestion[];
      isLoading: boolean;
      query: (input: string) => void;   // debounced; calls the Addressly seam via an interactive fetcher
    }
    export function useAddressSuggestions(): UseAddressSuggestions;
    ```

  - **Steps:**
    1. Implement the hook following the `useScapiFetcher` interactive-fetch pattern (debounced), delegating to `suggestAddresses()` from the P1.1 seam.
    2. In `StoreSearch`, render the suggestion list under the input as the shopper types; selecting a suggestion sets `q` and submits the search for that address.
    3. When suggestions are unavailable, the typed value still submits and returns results (no dead end).
  - **Done when:** suggestions appear while typing, selecting one runs the search, and manual submit works when suggestions are unavailable.
  - **Trace:** RQ-710

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 typing an address shows Addressly suggestions; A2 selecting a suggestion runs the search for that address; A3 manual submit still returns results when no suggestion is used (no dead end) — RQ-710.
  - **Unit:** run `1.2` (Test Setup) → attach output.
  - **Scenario (live, manual-style):**
    1. Given the store search input
    2. When I type a partial address → Then suggestions appear (screenshot: suggestions)
    3. When I pick a suggestion → Then the search runs for that address (screenshot: results)
    4. When suggestions are unavailable and I submit the typed value → Then results still return
  - **Visual:** screenshots of the suggestion dropdown + no-suggestion fallback; console clean.
  - **Verdict:** SHIP (then human approval)
- **Status:** pending — real-Addressly re-verify BLOCKED on capability delivery (0.1, DEFERRED-APPROVED); verify against the Q1 stub now, re-verify on delivery

### P3 — Interactive map synchronized with the list

- **Story:** `TBD-3`
- **Delivers:** the map loads on demand and stays off the critical JS path; the list stays usable before/while it loads; list selection updates the marker and marker selection updates the list; desktop shows map beside list, mobile/tablet is list-first behind a toggle — RQ-702 · SLS.MAP
- **Areas:** @FE
- **Depends on:** P1
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · bounds: none · tier: standard
- **Jira:** —

- [ ] P3.1 **@FE** `StoreMap` — lazy client island off the critical path
  - **Files:** `src/extensions/store-locator/components/store-map/index.tsx` (new)
  - **Interface:**

    ```ts
    import type { RankedStore } from '@/lib/store-locator/types';

    export interface StoreMapProps {
      stores: RankedStore[];
      selectedStoreId: string | null;
      onSelect: (storeId: string) => void;
    }
    export function StoreMap(props: StoreMapProps): JSX.Element;  // default export, imported via React.lazy
    ```

  - **Steps:**
    1. Load the map with `const StoreMap = lazy(() => import('.../store-map'))` and render it inside `<Suspense>` so it never sits on the initial route bundle (`sfnext-performance`).
    2. Initialise the map via `loadMapEntrypoint()` (P1.1 seam); render a marker per store; on marker click call `onSelect`; reflect `selectedStoreId`.
    3. Desktop: render beside the list and keep it visible; mobile/tablet: collapsed behind a toggle, list-first (grooming Q6).
  - **Done when:** the initial bundle contains no map SDK; the list is usable before/while the map loads; the map renders markers and drives selection.
  - **Trace:** RQ-702

- [ ] P3.2 **@FE** List↔map selection sync — feature context provider
  - **Files:** `src/extensions/store-locator/context/store-locator-context.tsx` (new)
  - **Interface:**

    ```ts
    import type { ReactNode } from 'react';

    export interface StoreLocatorState { selectedStoreId: string | null }
    export type StoreLocatorAction =
      | { type: 'select'; storeId: string }
      | { type: 'clear' };
    export function StoreLocatorProvider(props: { children: ReactNode }): JSX.Element;
    export function useStoreLocator(): StoreLocatorState & {
      select: (storeId: string) => void;
      clear: () => void;
    };
    ```

  - **Steps:**
    1. Implement the provider with `useReducer` per `sfnext-state-management`; expose `useStoreLocator()`; scope the provider to the locator route subtree only.
    2. `StoreList` and `StoreMap` both read `selectedStoreId` and call `select()` so list→map and map→list stay in sync.
  - **Done when:** selecting a store in the list highlights its marker and selecting a marker highlights the list entry, both driven by the shared context.
  - **Trace:** RQ-702

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 initial `/store-locator` bundle contains no map SDK (lazy island); A2 list is usable before/while the map loads; A3 selecting a list item highlights its marker and selecting a marker highlights the list entry (two-way sync); A4 desktop shows map beside list, mobile/tablet is list-first behind a toggle — RQ-702.
  - **Unit:** run `1.4` (Test Setup) → attach output.
  - **Scenario (live, manual-style):**
    1. Given the results list rendered
    2. When the route first loads → Then the list is interactive before the map island loads (screenshot: list-first / loading)
    3. When I select a list item → Then its marker activates (screenshot: map selection)
    4. When I click a marker → Then the list entry activates
    5. On a mobile viewport → Then the map is collapsed behind a toggle, list-first
  - **Visual:** screenshots at desktop + mobile (map-beside-list / list-first toggle / selection); console clean.
  - **Verdict:** SHIP (then human approval)
- **Status:** pending — real-Cartora map re-verify BLOCKED on capability delivery (0.1, DEFERRED-APPROVED); verify against the Q1 stub/entrypoint now, re-verify on delivery

### P4 — Store detail

- **Story:** `TBD-4`
- **Delivers:** a selected store shows name, full address, phone, regular weekly opening hours, and services; holiday hours stay out of v1 per grooming Q1 — RQ-703 · SLS.DTL
- **Areas:** @FE
- **Depends on:** P1
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · bounds: none · tier: standard
- **Jira:** —

- [ ] P4.1 **@FE** `StoreDetail` component
  - **Files:** `src/extensions/store-locator/components/store-detail/index.tsx` (new)
  - **Interface:**

    ```ts
    import type { Store } from '@/lib/store-locator/types';

    export interface StoreDetailProps { store: Store }
    export function StoreDetail(props: StoreDetailProps): JSX.Element;
    ```

  - **Steps:**
    1. Render name, full address, phone, `store.hours` (regular weekly hours), and `store.services`.
    2. Do NOT render holiday/special hours (grooming Q1); only present what the store record exposes.
    3. Use semantic markup + `data-testid`s (RQ-706, woven a11y).
  - **Done when:** the detail shows the five approved fields from the store record with no holiday-hours surface.
  - **Trace:** RQ-703

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 a selected store shows name, full address, phone, regular weekly hours, and services; A2 NO holiday/special hours surface (grooming Q1) — RQ-703.
  - **Unit:** run `1.5` (Test Setup) → attach output.
  - **Scenario (live, manual-style):**
    1. Given a result selected
    2. When the detail opens → Then the five approved fields render (screenshot: detail)
    3. Then no holiday-hours field is present
  - **Visual:** screenshot of the store detail; console clean.
  - **Verdict:** SHIP (then human approval)
- **Status:** pending

### P5 — Filter stores by service

- **Story:** `TBD-5`
- **Delivers:** click-and-collect, returns, and gift-wrapping filters narrow the same result set; clearing restores full results; a no-match empty state is shown; offered services follow Business Manager — RQ-705 · SLS.FLT
- **Areas:** @FE
- **Depends on:** P1
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · bounds: none · tier: standard
- **Jira:** —

- [ ] P5.1 **@FE** `ServiceFilter` component driving the `service` URL param
  - **Files:** `src/extensions/store-locator/components/service-filter/index.tsx` (new)
  - **Interface:**

    ```ts
    export interface ServiceFilterProps {
      offeredServices: string[];   // from LocatorConfig (grooming Q5 seed: click-and-collect, returns, gift-wrapping)
      active: string | null;       // current `service` URL search param
    }
    export function ServiceFilter(props: ServiceFilterProps): JSX.Element;
    ```

  - **Steps:**
    1. Render the BM-configured `offeredServices` as filter controls; selecting one sets `?service=<id>` (the P1.2 loader narrows the ranked set); both list and map reflect the narrowed set.
    2. Clearing the filter removes the `service` param and restores the full search results.
    3. When the filter matches no stores, render the empty-results state for the filtered view.
  - **Done when:** applying a service narrows list + map, clearing restores full results, an empty state shows on no match, and only BM-configured services are offered.
  - **Trace:** RQ-705

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 a service filter narrows list + map to the same set; A2 clearing restores full results; A3 a no-match filter shows the empty state; A4 only BM-configured services are offered — RQ-705.
  - **Unit:** covered by `1.1` (service-filter narrowing) → attach output.
  - **Scenario (live, manual-style):**
    1. Given results shown
    2. When I apply "click-and-collect" → Then list + map narrow to that set (screenshot: filtered)
    3. When I clear the filter → Then full results restore
    4. When a filter matches nothing → Then the empty state shows (screenshot: empty)
  - **Visual:** screenshots of filtered / cleared / empty; console clean.
  - **Verdict:** SHIP (then human approval)
- **Status:** pending

### P6 — Get directions

- **Story:** `TBD-6`
- **Delivers:** directions opens the shopper's own map provider with the store as destination; no additional maps-vendor call and no key exposure — RQ-704 · SLS.DIR
- **Areas:** @FE
- **Depends on:** P4
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · bounds: none · tier: standard
- **Jira:** —

- [ ] P6.1 **@FE** `GetDirectionsLink` — native map deep link
  - **Files:** `src/extensions/store-locator/components/get-directions-link/index.tsx` (new)
  - **Interface:**

    ```ts
    import type { Store } from '@/lib/store-locator/types';

    export interface GetDirectionsLinkProps { store: Store }
    export function GetDirectionsLink(props: GetDirectionsLinkProps): JSX.Element;
    ```

  - **Steps:**
    1. Build a native map deep link (platform maps URL / `geo:` URI) with the store address/coordinates as the destination.
    2. Render it as an anchor that opens the shopper's own map provider; make NO Cartora/maps-vendor API call and expose no key.
  - **Done when:** activating the action opens the device map provider with the store as destination and triggers no maps-vendor request.
  - **Trace:** RQ-704

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 activating directions opens the device's own map provider with the store as destination; A2 NO maps-vendor API call is made and no key is exposed — RQ-704.
  - **Unit:** run `1.6` (Test Setup) → attach output.
  - **Scenario (live, manual-style):**
    1. Given a store detail
    2. When I activate "get directions" → Then the native map provider opens with the store as destination (screenshot: directions affordance + target)
    3. Then the network shows no maps-vendor request (evidence: network log)
  - **Visual:** screenshot of the directions affordance; console + network clean of vendor calls.
  - **Verdict:** SHIP (then human approval)
- **Status:** pending

### P7 — Navigation entry, retired URL, and directory retirement

- **Story:** `TBD-9`
- **Delivers:** the footer "Our stores" link points to `/store-locator`; a bookmarked `/stores` permanently redirects; the legacy Page Designer store-directory content is retired — one shopper-visible switch-over — RQ-709 · GH.UTL, store-directory REMOVED
- **Areas:** @FE
- **Depends on:** P1
- **Process:** per [Phase Processing Rules](#phase-processing-rules) · bounds: none · tier: standard
- **Jira:** —

- [ ] P7.1 **@FE** `/stores` retired-URL route — permanent redirect
  - **Files:** `src/routes/_app.stores.tsx` (new)
  - **Interface:**

    ```ts
    import { redirect } from 'react-router';
    import type { LoaderFunctionArgs } from 'react-router';

    // permanent (301) redirect to the locator (grooming Q2); no client component
    export function loader(args: LoaderFunctionArgs): Response;  // return redirect('/store-locator', 301)
    ```

  - **Steps:**
    1. Add the flat route file `_app.stores.tsx` (maps to `/stores`) exporting only a `loader`.
    2. Return `redirect('/store-locator', 301)` so bookmarked/indexed links land on the locator.
  - **Done when:** requesting `/stores` returns a permanent (301) redirect to `/store-locator`.
  - **Trace:** RQ-709

- [ ] P7.2 **@FE** Re-point the global footer "Our stores" link to `/store-locator`
  - **Files:** global footer navigation component (per technical-design Building Block) (edit)
  - **Steps:**
    1. Change the "Our stores" link target (under "About") from the static directory to `/store-locator`.
  - **Done when:** the footer "Our stores" link navigates to `/store-locator`.
  - **Trace:** RQ-709

#### CMS / Page Designer

- [ ] P7.3 **@FE** Retire the legacy `/stores` Page Designer content page
  - **Files:** Page Designer `/stores` content page + its authoring entry (Business Manager)
  - **Steps:**
    1. Retire the static directory content page and remove its authoring entry; no content migration (store data now comes from the Stores API).
  - **Done when:** the legacy `/stores` Page Designer content is retired with no shopper-facing entry left.
  - **Trace:** store-directory REMOVED

- **Verify** (agent self-check → evidence for phase approval):
  - **Acceptance (frozen):** A1 requesting `/stores` returns a permanent (301) redirect to `/store-locator`; A2 the footer "Our stores" link navigates to `/store-locator`; A3 the legacy `/stores` Page Designer content is retired with no shopper-facing entry — RQ-709, store-directory REMOVED.
  - **Unit:** run `1.7` (Test Setup) → attach output.
  - **Scenario (live, manual-style):**
    1. When I request `/stores` → Then a 301 redirect lands on `/store-locator` (evidence: response status + Location header)
    2. When I click the footer "Our stores" → Then it navigates to `/store-locator` (screenshot: footer)
    3. Then the legacy directory content page is no longer reachable
  - **Visual:** screenshot of the footer link; evidence of the 301 response.
  - **Verdict:** SHIP (then human approval)
- **Status:** pending

## Verification

<!-- Developer-owned automated checks only; they run across the whole change, not a specific phase.
     Cross-cutting — NOT projected to a Jira sub-task. Functional testing (E2E / manual QA / a11y / UAT)
     is QA-owned and out of this pipeline. -->

### Automated (developer-owned, must pass in CI)

- [ ] 4.1 **@SHARED** Unit tests pass: `npm test`
- [ ] 4.2 **@SHARED** No regressions in existing developer-owned test suites
- [ ] 4.3 **@SHARED** ESLint passes on changed files
- [ ] 4.4 **@SHARED** TypeScript check passes: `npx tsc --noEmit`
- [ ] 4.5 **@FE** Bundle check: the initial `/store-locator` bundle contains no map SDK code — the map island is lazy (see the `sfnext-performance` skill) — RQ-702

## Review & Approval

- **Approved for implementation:** no
- **Approved by:** —
- **Date:** —
