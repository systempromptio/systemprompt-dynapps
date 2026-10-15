# Secure Coding — Quick Reference (Odoo)

12 gates, 7 BLOQUEANTES + 5 WARNING. Full examples in `secure-coding-blocking.md` + `secure-coding-warning.md`.

## BLOQUEANTES (code MUST pass before delivery)

### Gate 1: Injection
- SQL: ONLY `%s` with params list. ZERO `%`, `f""`, `.format()` for values
- eval: ONLY `safe_eval()`. ZERO `eval()` / `exec()`
- XSS: `html_sanitize()` on user HTML. `t-out`/`t-field` in QWeb, never `t-raw` with user data
- Markup: use `%s` formatting (auto-escapes) or explicit `escape()`

### Gate 2: Access Control
- Every model MUST have `ir.model.access.csv` with per-group permissions
- Models with `company_id` MUST have multi-company record rule
- M2O cross-model: `check_company=True`
- Sensitive fields: `groups='...'`
- `sudo()`: minimal scope, documented reason. Never in public endpoints. Prefer `compute_sudo=True`

### Gate 3: RPC Exposure
- Sensitive methods: prefix `_` or `@api.private`
- Controllers: `auth='none'` ONLY for webhooks with HMAC/token. `csrf=False` only for JSON APIs
- `methods=['POST']` for mutations. Never `sudo()` in controllers without explicit permission check

### Gate 4: Sensitive Data
- NEVER log passwords, API keys, tokens, banking data, NIF
- Secrets in `ir.config_parameter` or fields with `groups='base.group_system'`, `copy=False`
- Personal data (RGPD): restricted `groups=`, model inherits `mail.thread`

### Gate 5: Dependencies
- External deps with minimum secure version. Check CVEs
- No hardcoded config — use `ir.config_parameter` or `res.config.settings`
- Zero `print()` in production. Zero debug endpoints without auth

### Gate 6: Auditability
- Critical models: inherit `mail.thread`, state fields with `tracking=True`, `message_post()` on actions
- NEVER delete `mail.message`, `ir.logging` from code

### Gate 7: Server Actions
- No `cr.commit()` / `cr.rollback()`. No destructive SQL. `sudo()` only justified

## WARNING (flag but don't block)

### Gate 8: Performance as Security
- N+1: prefetch before loops. No `search([])` without limit in endpoints

### Gate 9: Error Handling
- No stack traces to users. No silent `except: pass`. Always `_logger.exception()`

### Gate 10: External Integrations
- `timeout=` on every HTTP call. `verify=True` for SSL. Secrets in headers, not URLs

### Gate 11: Concurrency
- Race conditions: `FOR UPDATE` lock on quantities. Webhooks must be idempotent

### Gate 12: Compliance (RGPD)
- Personal data exportable/deletable. Retention policy. Consent tracked

---

For code examples of each gate, see `secure-coding-blocking.md` + `secure-coding-warning.md`.
