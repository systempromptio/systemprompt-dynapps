# Integration documentation — Cartora (emulated official-docs / web search)

<!-- Eval-only, design-artifact-only, SA-flow-only context overlay.

     This file emulates the result a real integration design would obtain by following the
     `design` instruction's source policy: search the web for the vendor's OFFICIAL documentation
     first, then local / project-context material, then ask the user. It is loaded ONLY for the
     design artifact's integration (SA) run — the design judge declares the `.context/integration-docs`
     id and only the `.sa.md` overlay exists, so discovery and the functional (BA) design never see it.

     Citation rule (as with initial-context.md): in the artifact, cite realistic anchors for this
     material — e.g. project-context `web` results or the vendor doc URL below — NOT this eval-only
     file. This is the third-party service CONTRACT at documentation level: the design's Integration
     Specification SHOULD state it concretely (the vendor's endpoints/paths, params, auth, field
     mapping, and documented codes/quota), grounded in exactly this — do not invent fields, endpoints,
     auth, or values beyond what is stated here. What stays out of design is OUR build: our file/route
     layout, our wiring/code, and our own (not vendor-documented) tuning numbers — those are
     technical-design. -->

Search summary: the SA searched the web for Cartora's official documentation first (found the
public developer portal below), then checked local project-context (only the scattered technical
notes in the base packet), then confirmed the account/key specifics were still open with the user
(tracked as Open Questions). The relevant public docs found:

- Cartora Maps SDK guide — `https://developers.cartora.example/docs/maps-sdk`
- Cartora Geocoding API reference — `https://developers.cartora.example/docs/geocoding`
- Cartora account, API keys & quota — `https://developers.cartora.example/docs/keys`

## Cartora products used

- **Cartora Maps SDK** — a client-side JavaScript SDK that renders an interactive map into a
  consumer-supplied container. Loaded as a JS bundle + vector tiles from Cartora's CDN. Authorized
  with a domain-restricted API key.
- **Cartora Geocoding API** — an HTTPS/JSON REST endpoint that resolves a postcode or city query to
  ranked coordinate results (used for both geocoding and proximity ordering).

## Geocoding API — endpoint

- **Method / URL:** `GET https://api.cartora.example/v1/geocode`
- **Data format:** JSON response.
- **Authorization / transport:** the request carries a domain-restricted API key in the **`key`
  query parameter** (`GET …/v1/geocode?q=…&key=<API_KEY>`). There is no `Authorization` header and
  no `Bearer` scheme — the key is a plain query-parameter credential. The key is **origin-locked**:
  the Cartora account allow-lists the HTTP referrer/origin(s) it may be called from, so it only works
  from those origins even though it travels in the URL. Billing accrues to the key's Cartora account.
  The key may be attached either by the Maps SDK/host (client call) or by a server the integration
  places in front of the API (server proxy) — the docs do not mandate which; that call placement is
  an integration decision, not a vendor requirement.

### Request query parameters

| Parameter | Required | Meaning |
|---|---|---|
| `q` | yes | Free-text search query — a postcode or a city name. |
| `key` | yes | The domain-restricted API key (origin-locked); how the credential is transmitted. |
| `country` | no | ISO country code to scope results to a market. |
| `limit` | no | Maximum number of ranked results to return. |

### Response body (JSON)

The response contains a `results` array, ordered best-match first. Each element:

| Field | Type | Meaning |
|---|---|---|
| `results[].lat` | number | Latitude of the match. |
| `results[].lng` | number | Longitude of the match. |
| `results[].score` | number | Relevance/proximity score used for ordering. |
| `results[].label` | string | Human-readable label for the match (display convenience). |

### Status codes & errors

| Status | Meaning |
|---|---|
| `200` | Success. `results` is returned (an empty array when nothing matches — not an error). |
| `400` | Malformed request (e.g. missing the required `q` parameter). |
| `401` / `403` | Key missing/invalid, or the calling origin is not on the key's allow-list. |
| `429` | Account quota / rate limit exceeded. |

### Quota, rate limit & caching (documented)

- The standard account tier documents **10,000 geocoding requests/day per key**, rate-limited to
  ~**50 requests/second**.
- Geocode results are cacheable: the API returns `Cache-Control: max-age=86400`, so a result may be
  reused for up to **24h**.

## Maps SDK — rendering channel

- **Channel:** HTTPS load of the Cartora Maps SDK bundle + vector tiles from the Cartora CDN
  (`https://cdn.cartora.example/maps-sdk/...`).
- **Data format:** JavaScript bundle + vector map tiles.
- **Authorization / transport:** the same domain-restricted API key, supplied when the host
  initializes the SDK (`Cartora.init({ key })`); the SDK attaches the key to its tile/bundle
  requests. The SDK is client-side and renders into a consumer-supplied DOM container.

## Keys, quota & billing (from the account docs)

- API keys are **domain-restricted**: a key only works from its allow-listed origin(s).
- Quota and billing are per Cartora account; the account owner bears the cost.
- The docs do not decide where the geocoding call is made from (browser vs a server proxy) — that
  is an integration decision, not a vendor requirement.
