# Test Types Reference

Single source of truth for available test types, when to use each, and the base class.

## Test Types

| Test Type | Base Class | When to Generate |
|-----------|-----------|-----------------|
| **Unit Tests** | `TransactionCase` | Model logic, computations, constraints, CRUD, workflows |
| **Form/Onchange Tests** | `TransactionCase` + `Form` | Onchange behavior, default values, field visibility |
| **E2E Flow Tests** | `TransactionCase` | Full business cycles: SO→deliver→invoice, PO→receive→invoice |
| **HTTP/Controller Tests** | `HttpCase` | REST endpoints, web controllers, portal access |
| **Tour Tests** | `HttpCase` | End-to-end UI flows via browser automation |
| **Security Tests** | `TransactionCase` | Access rights, record rules, group permissions |
| **Report Render Tests** | `TransactionCase` | QWeb reports (`ir.actions.report`) / `report.*` models — render sin excepción + dato aportado |
| **Performance Tests** | `TransactionCase` | Query counts, execution benchmarks |
| **JavaScript Tests** | `QUnit` (JS) | OWL components, widgets, client-side logic |

## Default Recommendations

- **If user doesn't specify:** Unit Tests + Form/Onchange Tests
- **If module inherits 2+ flow models** (SOL, POL, AML, stock.move, stock.rule): Add E2E Flow Tests (mandatory)
- **If module has controllers:** Add HTTP/Controller Tests
- **If module has JS components:** Add JavaScript Tests
- **If module adds/overrides a QWeb report** (`ir.actions.report` / `report.*`): Add Report Render Tests
- **If module adds tracking/chatter on purpose** (`tracking=True`, `message_post`): Add mail/tracking assertions (was LOW → now HIGH)

## Test Class Hierarchy

```
unittest.TestCase
  → odoo.tests.case.TestCase
    → BaseCase (auto-tags: 'standard', 'at_install')
      ├→ TransactionCase     (savepoint per test)
      │  └→ HttpCase         (HTTP server + Chrome headless)
      └→ SingleTransactionCase (shared transaction, no savepoints)
```

## Assertion Methods (v16+)

| Method | Purpose |
|--------|---------|
| `assertRecordValues(records, [dicts])` | Compare records to expected values |
| `assertQueryCount(default=N)` | Assert SQL query count |
| `assertXMLEqual(a, b)` | Compare XML architectures |
| `assertHTMLEqual(a, b)` | Compare HTML content |

## Decorators

| Decorator | Purpose |
|-----------|---------|
| `@tagged('post_install', '-at_install')` | Run after module install (most common) |
| `@users('user_login')` | Run as specific user |
| `@warmup` | Warm caches before measuring |
| `@mute_logger('odoo.addons...')` | Suppress expected log messages |
