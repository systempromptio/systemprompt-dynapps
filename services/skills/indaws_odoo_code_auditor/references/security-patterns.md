# Security Patterns — Detection Guide

For the complete security rules (PASS/FAIL examples, 12 gates), see `odoo-version-api-reference` → `secure-coding-blocking.md` (7 bloqueantes) + `secure-coding-warning.md` (5 warnings).

This file focuses on **how to detect** vulnerabilities during an audit.

> **Exposing Odoo to an agent?** If the module ships an MCP server, JSON-RPC/OpenAPI bridge, or any tool where the caller picks the model / fields / domain at runtime, also read `mcp-exposure-security.md` — that surface fails in ways plain modules don't (exposure≠ACL, metadata leaks, runtime-domain injection, single-principal blind spots).

## Grep patterns for detection

```bash
# SQL Injection — string formatting in SQL
grep -rn 'cr.execute.*[%f"]' --include="*.py" .
grep -rn 'cr.execute.*\.format(' --include="*.py" .

# eval/exec usage (should be safe_eval only)
grep -rn '\beval(' --include="*.py" . | grep -v safe_eval
grep -rn '\bexec(' --include="*.py" .

# XSS — t-raw in QWeb templates
grep -rn 't-raw=' --include="*.xml" .

# Unsafe Markup concatenation
grep -rn 'Markup.*+.*[^escape]' --include="*.py" .

# sudo() without justification
grep -rn '\.sudo()' --include="*.py" .

# Public endpoints without auth
grep -rn "auth='none'" --include="*.py" .
grep -rn "auth='public'" --include="*.py" .

# Hardcoded secrets
grep -rn 'API_KEY\|SECRET\|PASSWORD\|TOKEN' --include="*.py" . | grep -v '#'

# Missing ACL (models without ir.model.access.csv entries)
# Compare: grep _name in models/*.py vs entries in security/ir.model.access.csv

# Fields without groups on sensitive data
grep -rn "fields\.\(Char\|Text\|Float\).*\(salary\|bank\|api_key\|secret\|password\|token\)" --include="*.py" . | grep -v "groups="
```

## False positives to watch for

- `cr.execute("... %s ...", params)` → SAFE (parameterized, not string formatting)
- `sudo()` on `ir.config_parameter.get_param()` → NORMAL (ICP requires sudo)
- `sudo()` on `ir.sequence.next_by_code()` → NORMAL (sequences require sudo)
- `auth='none'` on webhook endpoints with HMAC validation → ACCEPTABLE
- `t-raw` on `record.description` where field has `sanitize=True` → SAFE (already sanitized by ORM)
