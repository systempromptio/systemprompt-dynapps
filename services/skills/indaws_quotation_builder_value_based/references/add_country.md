# Onboarding a new country

Adding a country to `quotation-builder-value-based` is **config authoring, not
code forking**. A group counterpart runs `scripts/add_country.py` once against
their **own connected Odoo** (Dynapps MCP / odoo-pilot) and gets a schema-valid
**stub** locale pack under `locales/<code>/`. Translation + fiscal sign-off flip
it to `live` later.

The command:

1. **Copies the `en` reference pack** as the key skeleton (English is the engine
   base — plan DELTA 1). Every key the new pack needs is present, holding EN
   fallback prose until translated.
2. **Discovers** the instance-specific facts from the operator's Odoo via
   `scripts/odoo_discovery.py` (plan DELTA 3 — nothing hardcoded):
   legal entity (name/VAT/address), currency (symbol/position/decimals),
   `sale.order.template` id **resolved by name**, and IA/POC **product ids**.
3. **Interviews** the operator for the non-discoverable country facts
   (plan §5): hourly rates, per-app/complement base prices, Odoo licence price,
   migration params, partner manager, carriers, credibility claim, VAT label +
   e-invoicing regime.
4. **Writes** the pack, leaving `<<VERIFY: …>>` markers on unknown string
   fields and keeping EN defaults on typed (`number`/`array`) fields it still
   needs confirmed.
5. **Validates** with `scripts/validate_locale.py` and reports what is missing
   before the pack can go `live`.

## Discovered vs interviewed vs translated

| Source | Fields |
|---|---|
| **Discovered** (Odoo, runtime) | `legal_entity.*`, `contact.*`, `impl_template_id`, currency format, IA/POC `product_id`s |
| **Interviewed** (operator) | rates, app/complement base prices, `odoo_license` prices `[verify]`, migration `pct`/`months`, partner manager `[verify]`, VAT label, e-invoicing regime, carriers, credibility claim `[verify]` |
| **Translated later** (native speaker / consultant / legal) | `strings.json`, `apps_catalog.json`, `matching.json`, `fiscal.json` prose |

`[verify]` = Odoo-SA-set or legally-adjacent — **confirm, never invent** (plan
§5). Left unanswered, these surface in the report as outstanding items and, for a
`live` pack, fail `validate_locale.py`.

## Usage

```bash
# 1. See the interview (grouped like plan §5) / get a blank answers template
python3 scripts/add_country.py --print-questions
python3 scripts/add_country.py --template > answers_de.json   # then fill it in

# 2. Build the pack from a filled answers file (headless-safe)
python3 scripts/add_country.py de --language German --answers answers_de.json
#   -> writes locales/de/, runs validate_locale.py, prints the readiness report
#   -> add --dry-run to build+report without writing
#   -> add --overwrite to replace an existing locale dir
```

**Live Odoo discovery** requires an injected reader (a callable wrapping the
operator's `orm_search_read` MCP tool). The bare CLI has no MCP client, so it
skips discovery and marks discoverable fields; a programmatic caller wires the
reader:

```python
import add_country as OB
pack, report, validate_result = OB.build_and_write(
    "de", "German", reader=my_orm_reader, answers=answers, pack_status="stub")
```

`reader(model, method, args, kwargs)` mirrors Odoo `execute_kw`
(see `odoo_discovery.py` for the contract and `FakeReader` for the offline
double).

## Going live

The pack ships as `pack_status:"stub"` (the loader blocks stubs from production).
Before flipping to `live`:

1. Translate `strings.json` / `apps_catalog.json` / `matching.json` (EN fallback
   is in place now).
2. Author `fiscal.json` copy under **named local finance/legal sign-off**
   (no machine-translated fiscal/legal text ships).
3. Confirm every `[verify]` field from Odoo — never invent.
4. Re-run `validate_locale.py`, then set `locale.pack_status` to `"live"`.

## Offline smoke

`tests/add_country_smoke.py` builds a throwaway `zz` pack with a `FakeReader` +
sample answers, asserts discovery/interview/marker behaviour and a green
`validate_locale.py`, then deletes the pack:

```bash
python3 tests/add_country_smoke.py
```
