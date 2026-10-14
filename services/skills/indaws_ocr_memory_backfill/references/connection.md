# Connection and security

## Credentials (never hardcoded)
Every script reads the connection from environment variables. Ask the person for them and export
them in the session; do not write them into any file or into the code:

```bash
export ODOO_URL="https://customer.odoo.com"
export ODOO_DB="db_name"
export ODOO_USER="user@customer.com"
export ODOO_KEY="password_or_api_key"
export ODOO_COMPANY_ID="1"          # MANDATORY: the memory and its unique index are per company
```

If they would rather not paste a password, have them create an **API key** (Preferences → Account
security → New API key) and use it as `ODOO_KEY`. Customer staging/production credentials are
usually in `~/TRABAJO/Odoo/.credentials/<customer>.env`.

## Two security modes
- **Dry-run (default)**: `_odoo.py` only allows read methods. `scan_and_plan.py` and `verify.py`
  never write.
- **Apply**: `export OCR_BACKFILL_APPLY=1` enables EXCLUSIVELY
  `ai.memory.alias.save_alias`. Any other `create`/`write`/`unlink` stays blocked with a
  `PermissionError`. This is the safety net that stops us touching customer data by mistake.

## Protocol per version
- Odoo ≤ 18 → direct XML-RPC (`_odoo.py` does it with `xmlrpc.client`).
- Odoo ≥ 19 → JSON2 protocol. Lean on the `odoo-pilot` skill for the transport and reuse the logic
  in `scan_and_plan.py` / `apply_seed.py` (the aggregation and the plan do not change).

## Company and context
`_odoo.py` sets `context = {'company_id': <ODOO_COMPANY_ID>, 'allowed_company_ids': [<id>]}` on
every call, so that `env.company` inside `save_alias` is the right company (the memory is per
company). In a multi-company database, run the backfill once per company with its own
`ODOO_COMPANY_ID`.
