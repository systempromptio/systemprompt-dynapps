# Search & merchandising — working notes

## Background

SF Next is replacing the built-in SFCC search with **Bloomreach** for search and
merchandising. This ISD documents the integration so reviewers can see what we build
versus what Bloomreach does on its own — the recurring question in review.

## Architecture (from design)

- The shopper searches / browses on **SF Next**.
- SF Next calls **Bloomreach's search & category APIs** for results, facets, and
  merch rules — this client is ours to build (server-side from MRT, per
  sfnext-data-fetching).
- SF Next still reads **cart & pricing** from **SFCC Commerce** (backend of record).
- The **PIM catalog feed** is the product-data source for both sides. The **same
  scheduled feed** refreshes **SFCC**'s own product records and is **ingested by
  Bloomreach** through its own native connector. This is a background / scheduled job,
  **not part of the live request path** — we publish the feed; we do not build or
  operate Bloomreach's ingestion.

## Config / ops (detail)

- Feed export nightly at 02:00 UTC; ~180k SKUs; hourly delta feeds.
- Bloomreach API key in MRT env; 200ms p95 budget on the search call.
- Autocomplete / suggest widget is phase 2 (not this doc).
- Redirects & SEO rules stay in SFCC for now.

## Meeting notes (2026-07-22)

- Merch wants "boosted" pins configurable in the Bloomreach console — no dev work,
  out of scope here.
- Analytics: search events go to GA4 and a Bloomreach pixel (a separate story).
- Legal: no PII in the feed (SKU / price / inventory only) — confirmed.
- Open Q: multi-locale catalogs at launch? feed team to confirm — doesn't change the
  connection map.

## Glossary

PIM = product information management (our catalog source). Inventory is owned by the
OMS (a separate system, not part of this diagram).
