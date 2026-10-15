# Performance Anti-Patterns — Detection Guide

For the complete performance patterns (precompute, indexes, batch ops), see `odoo-version-api-reference` → `engine-patterns.md`.

This file focuses on **how to detect** performance problems during an audit.

## Grep patterns for detection

```bash
# N+1: search/browse inside loops
grep -rn 'for .* in self' --include="*.py" -A5 . | grep -E '\.(search|browse|read)\('

# N+1: individual writes in loops
grep -rn 'for .* in self' --include="*.py" -A5 . | grep '\.write('

# Missing limit on search (potential full table scan)
grep -rn '\.search(\[' --include="*.py" . | grep -v 'limit='

# Missing limit on search_read
grep -rn '\.search_read(\[' --include="*.py" . | grep -v 'limit='

# Computed fields without depends (always recomputes)
grep -rn 'compute=' --include="*.py" -B2 . | grep -v '@api.depends'

# store=True without precompute (eligible candidates)
grep -rn 'compute=.*store=True' --include="*.py" . | grep -v 'precompute'

# Missing indexes on frequently searched fields
grep -rn "fields.Many2one\|fields.Char.*index" --include="*.py" . | grep -v 'index='

# sudo() in compute methods (should be compute_sudo)
grep -rn 'def _compute_' --include="*.py" -A10 . | grep '\.sudo()'

# Deep depends path (> 2 dots)
grep -rn '@api.depends' --include="*.py" . | grep -oP "'[^']+'" | grep -c '\.' | awk '$1>2'
```

## False positives to watch for

- `search([], limit=1)` → OK (limited)
- `for rec in self:` with only cache reads → OK (prefetch handles it)
- `mapped('partner_id')` before loop → GOOD (prefetch pattern)
- `browse(ids)` outside loop → OK (no N+1)
- `compute_sudo=True` on field → OK (not same as `.sudo()` in method body)
