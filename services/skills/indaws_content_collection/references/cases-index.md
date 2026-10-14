# Case Studies Index

Last updated: 2026-07-08
Total case studies: 22

## Quick Reference

| Client | Modules Used | Date |
|--------|--------------|------|
| [50Five](https://www.dynapps.es/es/cases/50five) | edi, inventory, maintenance |  |
| [Beldico](https://www.dynapps.es/es/cases/beldico) | edi, pos, quality |  |
| [Cyclis Bike Lease](https://www.dynapps.es/es/cases/cyclisbikelease) | crm, edi, project |  |
| [Despraz SA](https://www.dynapps.es/es/cases/despraz) | accounting, edi, pos |  |
| [Ecopower](https://www.dynapps.es/es/cases/ecopower) | edi, inventory, pos |  |
| [Elektro Van Assche](https://www.dynapps.es/es/cases/elektrovanassche) | accounting, crm, edi |  |
| [Espacio Orgánico](https://www.dynapps.es/es/cases/espacioorganico) | ecommerce, edi, inventory |  |
| [Eurofins](https://www.dynapps.es/es/cases/eurofins) | edi, pos, project |  |
| [Evolink](https://www.dynapps.es/es/cases/evolink) | crm, edi, project |  |
| [LFG París](https://www.dynapps.es/es/cases/lfgparis) | crm, edi, sales |  |
| [Lonestar](https://www.dynapps.es/es/cases/lonestaralphalaboratories) | crm, edi, pos |  |
| [Obiz](https://www.dynapps.es/es/cases/obiz) | accounting, crm, edi |  |
| [Orion Automotive](https://www.dynapps.es/es/cases/orionautomotive) | accounting, ecommerce, edi |  |
| [Plastimyr](https://www.dynapps.es/es/cases/plastimyr) | accounting, ecommerce, edi |  |
| [Prima Protection](https://www.dynapps.es/es/cases/primaproteccion) | ecommerce, edi, maintenance |  |
| [Puratos](https://www.dynapps.es/es/cases/puratos) | edi, pos, sales |  |
| [Readytobrand](https://www.dynapps.es/es/cases/readytobrand) | accounting, edi, project |  |
| [Regiechatel](https://www.dynapps.es/es/cases/regiechatel) | edi, pos |  |
| [Sensorfact](https://www.dynapps.es/es/cases/sensorfact) | accounting, crm, edi |  |
| [TM Technics](https://www.dynapps.es/es/cases/tmtechnics) | crm, edi, pos |  |
| [Todotrofeo](https://www.dynapps.es/es/cases/todotrofeo) | accounting, crm, edi |  |
| [Tunergia](https://www.dynapps.es/es/cases/tunergia) | accounting, crm, edi |  |

## Search Patterns

To find cases by industry or module:
```
grep -i "manufacturing" data/cases.json
grep -i "inventory" data/cases.json
```

## Data Schema

Each case record in `data/cases.json` contains:
- `id`: Case slug (from the /cases/<slug> URL)
- `title`: Full case study title
- `client`: Client name
- `headline`: Page H1 headline
- `url`: Full URL to the marketing case page
- `publish_date`: YYYY-MM-DD (usually empty; marketing pages omit it)
- `summary`: Brief summary (meta description or first 500 chars)
- `content`: Cleaned narrative content
- `modules_used`: Array of Core functional areas detected in the text
- `industry`: Industry category (may be empty)