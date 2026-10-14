# Docky per country — what to tune in each localisation

Docky (`dynapps_ai_*`) is a group-wide product (Spain, Belgium/Dynapps NV, and the rest). The
engine is generic and reads the live data of each instance, but there are points that **depend on
the localisation** and have to be reviewed when starting up a new country. This guide lists them.

> General rule: **nothing country-specific should be hardcoded**. Everything comes from the
> database (chart of accounts, taxes, the VAT on `res.company`) or from the per-company config. If
> something forces a code change, document it here.

## 1. Language of the report and of the prompts
- **Status report** (`audit.md`): the skill generates it in ES/EN/FR (`--lang`). Deliver it in the
  language of the country/customer.
- **`ai.extract.config.system_prompt` / `expected_schema`**: the extraction prompt may need
  adjusting to the language and invoice layout of the country. The config is **per company**.

## 2. VAT / supplier identification
- Supplier matching by VAT is **country-agnostic**: it normalises to alphanumerics and strips the
  country prefix when it is a known EU code (`_EU_CC` in the action), matching with or without the
  prefix (`ESB66120171` ↔ `B66120171`, `BE0477472701` ↔ `0477472701`, `A-48/714.489` ↔
  `A48714489`).
- **Per-country tuning**: if a country shows up whose code is not in `_EU_CC`, add it. For non-EU
  suppliers (US, etc.) the invoice usually carries **no VAT** → they resolve by **name** (memory);
  there the key is to seed the memory well and/or let it learn from the first correction.

## 3. Taxes (the most country-sensitive part)
Tax resolution lives in the `python_code` of the *AI: Map Invoice Extraction* action: `account.tax`
memory per supplier → by rate → a `global_tax` fallback; it honours an **explicit 0%**
(intra-EU reverse charge) without slipping the default tax in.
- **Spain**: equivalence surcharge (a compound `VAT + surcharge`), IRPF withholdings (negative
  taxes), intra-community acquisitions. The per-line `taxes` list supports multiple taxes.
- **Belgium / others**: their VAT rates (21/12/6…) already coexist in the database; make sure the
  country's tax chart is there and that the memory/rates resolve to the right tax.
- **Ambiguity**: many taxes share a percentage (e.g. several 21% in Spain: goods / services /
  intra-community / imports). The name filter excludes the `EU/EX/IG/RC/ND` variants — **review
  that filter per country** if their tax names differ.
- **Account with no default tax**: if you want mapping the account to bring the VAT along, set the
  default tax on that country's account or product.

## 4. Chart of accounts
- The chart of accounts is per localisation; expense accounts and their mapping (`account.account`
  memory per supplier) are per company. Review products↔account (one product per account) for the
  country.

## 5. Analytics
- If the country uses analytic accounting, map when it applies (context for the AI). See the report
  (`audit.md`).

## 6. Wildcard supplier/product
- Define a "sundry creditors" equivalent per country so unrecognised documents do not block.

## 7. Provider and API key
- `ai.provider` is **global per database** (`unique(code)`): the API key is shared across the
  companies of one database. Countries on **separate instances** → each one its own key (which is
  the natural setup). A different key per company within the same database → that needs a code
  change.

## 8. Memory (this skill)
- `ai.memory.alias` is **per company**. Run the backfill once per company with its own
  `ODOO_COMPANY_ID` (default scope: invoices with an attachment from the last year).

## 9. Deployment gotcha (applies to every country)
- The action and the config live in `<data noupdate="1">`: they are loaded **on install** and are
  **not** refreshed by `-u`. On an already-installed database, fixes to the action have to be
  applied by hand (rewriting `ir.actions.server`) or by reinstalling. On a **fresh install** they do
  come from the code.
- The opposite case also bites: data files with `noupdate="0"` — such as the Gemini model catalogue
  — **are** reloaded on every upgrade, so anything patched over RPC there is silently reverted.

## 10. Rounding
- Decimal rounding adjustments stay manual after the invoice (a known limitation of the AI).
