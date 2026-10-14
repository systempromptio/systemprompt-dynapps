# Sector Language Glossary — internal enablement asset

> v2 Value-Based Sales System §8.6. The glossary is **NEVER shown to the client**. It is a
> context asset generated **per opportunity** that governs the vocabulary used in: the
> "How we understand your business" opener, the app cards (`intro`/`fit`/`features`) and
> the case-study copy. Goal: make the client feel we speak **their sector's language**,
> not Odoo jargon.

## Idea

The sales rep/SDR knows the lead's sector. Before writing the content (STEP 4), a map is
built: **Odoo/technical term → the word the client's sector actually uses**. That map is
saved in `extra_data.sector_glossary` (for traceability) and, above all, used to **write**
the JSON's copy with the right words.

Golden tone rule (Golden Rule 10): short sentences, concrete numbers, **zero brochure
adjectives** ("innovative", "leading", "end-to-end"). Test: *if any competitor could sign
the sentence, delete it.*

## Canonical terms to map (baseline)

For every opportunity, translate at least these into the sector's vocabulary. The
right-hand examples below are illustrative — for a Spanish-market deal, produce the
Spanish equivalents; for other markets, use that market's real sector vocabulary.

| Odoo / generic term | How the sector names it (examples) |
|---|---|
| Customer / partner | (construction) developer, main contractor · (healthcare) patient/facility · (industry) account |
| Product | (food) SKU/item · (services) deliverable · (fashion) item/size-color |
| Bill of Materials (BoM) | recipe (food), formulation (chemicals), cut list (metalwork) |
| Manufacturing order (MRP) | production order, production batch, run |
| Delivery / picking | delivery note, warehouse slip |
| Lot / traceability | heat number (steel), best-before/batch (food), serial number |
| Project | job/site (construction), case file (professional services), matter (legal) |
| Opportunity (CRM) | tender, bid, quote in progress |
| Invoice | progress certificate (construction), fee note (legal) |
| Warehouse | site/yard, branch, distribution center |

(Adapt and extend per sector — this list is not exhaustive.)

## Format in `extra_data.sector_glossary`

```json
"sector_glossary": {
  "sector": "PPE wholesale distribution",
  "map": {
    "Producto": "item / PPE reference",
    "Cliente": "B2B account",
    "Albarán": "delivery note",
    "Tienda online": "B2B ordering portal"
  },
  "evitar": ["pipeline", "lead", "workflow"],
  "preferir": ["sales book", "opportunity", "workflow"]
}
```

- `map`: term → the sector's word (use it when writing copy).
- `evitar` / `preferir`: loanwords or jargon to replace with the sector's own word.

Note: the `map` keys stay as the Odoo/technical term (often the Spanish spreadsheet
key when writing for an ES-market client); the values are the sector words to use in
the client's language.

## How to use it

1. Generate the glossary (opus, STEP 4) from the lead's sector + intake data.
2. **Write** `understanding`, `app_overrides.*` and the case-study `resultado` copy using
   `map`/`preferir` and avoiding `evitar`.
3. Save it in `extra_data.json` (it isn't rendered; it stays as a trace of why the
   copy uses those particular words).
