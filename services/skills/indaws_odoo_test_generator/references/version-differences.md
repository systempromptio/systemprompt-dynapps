<!-- verified: v18.0/v19.0 @ 2026-06 -->
# Odoo Test Framework - Version Differences (v14-v19)

Quick reference of what changed in Odoo's test framework between versions.

**Important:** This file covers test-framework-specific differences. For comprehensive Odoo API patterns across all skills, see the **odoo-version-api-reference** skill.

---

## Verified core line references (source: `odoo/tests/common.py`)

**Remedido 2026-08-17** contra CE18 `@99f5a33c64` y CE19 `@d1d161033`.

| Symbol | v18 | v19 |
|--------|-----|-----|
| `assertQueryCount(self, default=0, flush=True, **counters)` | 587 | 604 |
| `class TransactionCase(BaseCase)` | 931 | 990 |
| `class SingleTransactionCase(BaseCase)` | 1165 | 1171 |
| `class HttpCase(TransactionCase)` | 2169 | 2196 |
| `authenticate(self, user, password, *, ...)` | 2287 | 2358 |
| `browser_js(...)` | 2381 | 2450 |
| `start_tour(self, url_path, tour_name, step_delay=None, **kwargs)` | 2479 | 2555 |
| `warmup(...)` decorator | 2585 | 2668 |

> **Esta tabla se pudrió en dos meses, y conviene saber cómo.** Al medirla el 2026-08-17, los ocho
> números de **v19 seguían exactos** y **cinco de los ocho de v18 estaban desfasados**: `1163→1165`
> (+2) y los cuatro de abajo **+58 exactamente** (`2111→2169`, `2229→2287`, `2323→2381`,
> `2421→2479`, `2527→2585`). Los dos primeros no se movieron porque están **por encima** del punto
> donde 18.0 insertó código después de junio.
>
> Esto **no es una cita inventada** —el desplazamiento constante lo demuestra: alguien midió bien y
> la rama siguió avanzando—. Es deriva de una rama viva, y por eso la fecha de la cabecera importa
> tanto como el número.
>
> **Lo que salva a esta tabla es la columna de la izquierda**: como lleva la *firma*, cualquiera la
> reconstruye en un segundo con `grep -n "class HttpCase" odoo18/odoo/tests/common.py`. Una tabla
> de números pelados no se habría podido arreglar, solo desconfiar de ella. **Al citar, deja el
> símbolo, no solo la línea.**

### `warmup` — REAL signature change v18 → v19

```python
# v18 (common.py:2585) — @decorator-wrapped, variadic
@decorator
def warmup(func, *args, **kwargs):
    self = args[0]
    ...

# v19 (common.py:2668) — positional-only param, @wraps inner fn
def warmup(func, /):
    @wraps(func)
    def warmup(self, *args, **kwargs):
        ...
```

Usage as a method decorator is identical (`@warmup` above a test method); only the
internal implementation/signature differs. Still pairs with `assertQueryCount`.

### `start_tour` — REAL behavior change v18 → v19

Signature is byte-identical, but **v19 emits a warning when `step_delay`
is set** (`odoo/tests/common.py:2570-2571`, `users._users`; remedido 2026-08-17, decía `:2569` y ahí hay una línea en blanco) (`"step_delay is only suitable for local testing"`) and wraps the run in
`patch.object(Users, 'tour_enabled', False)`. Never commit a launcher with `step_delay`.

---

## Class Availability by Version

| Class/Feature | v14 | v15 | v16 | v17 | v18 | v19 |
|---------------|-----|-----|-----|-----|-----|-----|
| `TransactionCase` | Y | Y | Y | Y | Y | Y |
| `SingleTransactionCase` | Y | Y | Y | Y | Y | Y |
| `HttpCase` | Y | Y | Y | Y | Y | Y |
| `SavepointCase` | Y | Y | Y | Deprecated | Removed | Removed |
| `HttpSavepointCase` | - | - | Y | Removed | Removed | Removed |
| `Form` class | In common.py | In common.py | In common.py | In form.py | In form.py | In form.py |
| `Like` utility | - | - | - | - | Y | Y |
| `Approx` utility | - | - | - | - | Y | Y |
| Built-in `freeze_time` | - | - | - | - | Y | Y |
| External request blocking | - | - | - | Y | Y | Y |
| `new_test_user()` | Y | Y | Y | Y | Y | Y |

---

## Import Changes

### v14-v16

```python
from odoo.tests.common import TransactionCase, HttpCase, Form, tagged
from odoo.tests import new_test_user
```

### v17

```python
from odoo.tests.common import TransactionCase, HttpCase, tagged
from odoo.tests import Form, new_test_user  # Form moved to odoo.tests
```

### v18-v19

```python
from odoo.tests.common import TransactionCase, HttpCase
from odoo.tests import Form, tagged, new_test_user, freeze_time
# Like and Approx available from odoo.tests.common
```

---

## Form Class Location

| Version | Import Path | Notes |
|---------|-------------|-------|
| v14-v16 | `from odoo.tests.common import Form` | Defined in `common.py` |
| v17 | `from odoo.tests import Form` | Moved to `form.py`, deprecated import from common |
| v18-v19 | `from odoo.tests import Form` | In `form.py` |

---

## Product Type Field

**Critical for test data creation:**

| Version | Storable Product | Consumable | Service |
|---------|-----------------|------------|---------|
| v14-v17 | `'type': 'product'` | `'type': 'consu'` | `'type': 'service'` |
| v18-v19 | `'is_storable': True` | `'type': 'consu'` | `'type': 'service'` |

```python
# v14-v17
cls.product = cls.env['product.product'].create({
    'name': 'Test Product',
    'type': 'product',  # storable
})

# v18-v19
cls.product = cls.env['product.product'].create({
    'name': 'Test Product',
    'is_storable': True,  # storable
})
```

---

## Command API

| Version | Recommended | Old Style |
|---------|-------------|-----------|
| v14-v15 | Tuples `(0, 0, {...})` | - |
| v16+ | `Command.create({...})` | Tuples still work |
| v17+ | `Command` API exclusively | Tuples deprecated |

```python
# v14-v15 style
'order_line': [(0, 0, {'product_id': product.id, 'product_uom_qty': 5})]

# v16+ style (recommended for ALL v16+ tests)
from odoo import Command
'order_line': [Command.create({'product_id': product.id, 'product_uom_qty': 5})]

# Full Command reference:
Command.create(values)      # (0, 0, values)
Command.update(id, values)  # (1, id, values)
Command.delete(id)          # (2, id)
Command.unlink(id)          # (3, id)
Command.link(id)            # (4, id)
Command.clear()             # (5,)
Command.set(ids)            # (6, 0, ids)
```

---

## Assertion Differences

### assertRecordValues

| Version | Notes |
|---------|-------|
| v14-v17 | Standard dict comparison |
| v18+ | Uses Python `match/case` for field type handling, `Approx` for floats |

### assertQueryCount

| Version | Notes |
|---------|-------|
| v14+ | `with self.assertQueryCount(admin=N):` |
| v18+ | Also available: `with self.assertQueriesContain([...]):` |

### Like (v18+ only)

```python
# v18+: Wildcard string matching
from odoo.tests.common import Like

self.assertEqual(
    "SELECT field1, field2 FROM model",
    Like('SELECT ... FROM model')
)
```

### Approx (v18+ only)

```python
# v18+: Float comparison with precision
# Automatically used in assertRecordValues for float/monetary fields
```

---

## Time Mocking

| Version | Method | Import |
|---------|--------|--------|
| v14-v17 | `freezegun` external package | `from freezegun import freeze_time` |
| v18+ | Built-in `freeze_time` | `from odoo.tests import freeze_time` |

```python
# v14-v17
from freezegun import freeze_time

with freeze_time('2024-01-15'):
    record = self.env['model'].create({})

# v18+ (either works, but prefer built-in)
from odoo.tests import freeze_time

@freeze_time('2024-01-15')
def test_time_dependent(self):
    pass
```

---

## HttpCase Differences

| Feature | v16 | v17 | v18 |
|---------|-----|-----|-----|
| `authenticate()` | `(user, password)` | `(user, password, browser=None)` | Same as v17 |
| `browser_js()` success | Hardcoded signal | `success_signal` param | Same as v17 + `debug` param |
| External request blocking | No | Yes (`BlockedRequest`) | Yes |
| `_wait_remaining_requests()` | No | Yes | Yes (with cleanup) |
| Password auth speed | Normal | Normal | Patched `_check_credentials` for speed |

---

## Tour Registration

### v16 (Legacy)

```javascript
// odoo.define() wrapper
odoo.define("module.tour_name", function (require) {
    var tour = require('web_tour.tour');

    tour.register("tour_name", {
        url: "/web",
        test: true,
    }, [
        { trigger: '.selector', content: "Description", run: 'click' },
    ]);
});
```

### v17-v18 (Modern)

```javascript
/** @odoo-module **/
import { registry } from "@web/core/registry";

registry.category("web_tour.tours").add("tour_name", {
    test: true,
    url: "/web",
    steps: () => [
        { trigger: '.selector', content: "Description", run: 'click' },
    ],
});
```

### v18 / v19 tour engine notes

- `TourAutomatic` / `TourInteractive` modes; `TourRecorder` for recording.
- The step schema is validated by `TourStep` (`tour_step.js`). For the FULL, source-verified
  step schema and `run` command vocabulary see **tour-patterns.md §4–§5**.
- `test: true` at tour level is obsolete — every registered tour is runnable.

---

## Tour Step Differences — VERIFIED schema (see tour-patterns.md §4)

The modern engine (v17+) accepts ONLY: `trigger`, `content`, `run`, `tooltipPosition`,
`timeout`, `isActive`, `expectUnloadPage`
(verified `web_tour/static/src/tour_service/tour_step.js:83-89`, `TourStep.stringify`, identical in v18 `tour_service/` and v19 `js/`).

| Property | ≤v16 (legacy) | v17 | v18/v19 | Note |
|----------|---------------|-----|---------|------|
| `trigger` | Required | Required | Required | |
| `content` | Y | Y | Y | `markup(_t(...))` for HTML |
| `run` | `'click'`/fn | + string vocab | + `false`/fn | `"text"` → **`"edit"`** in v17+ |
| `tooltipPosition` | — (`position`) | Y | Y | **replaces** `position` |
| `isActive` | — (`edition`) | partial | Y | **replaces** `edition`/mobile flags |
| `expectUnloadPage` | — | — | Y | full reload |
| `timeout` | none | none | **default 10000ms** | per-step (not "max 60000") |
| `extra_trigger`/`alt_trigger`/`in_modal`/`skip_trigger`/`pause`/`break` | varies | varies | **REMOVED / ignored** | do not generate |

`stepUtils` import path moved: v18 `@web_tour/tour_service/tour_utils` → v19 `@web_tour/tour_utils`.

---

## JavaScript Test Framework

| Version | Framework | File Naming | Default runner |
|---------|-----------|-------------|----------------|
| ≤v16 | QUnit | `*_tests.js` | QUnit |
| v17 | QUnit **solo** (Hoot no existe en v17: 0 imports de `@odoo/hoot`) | `*_tests.js` | QUnit |
| **v18** | **Hoot** | `*.test.js` | **Hoot** |
| **v19** | **Hoot** | `*.test.js` | **Hoot** |

Full source-verified Hoot reference (API exports, matchers, mount, mocking, QUnit→Hoot
migration) is in **js-test-hoot.md**. Quick form:

```javascript
import { describe, expect, test } from "@odoo/hoot";
import { mountWithCleanup } from "@web/../tests/web_test_helpers";

describe.current.tags("desktop");           // mandatory platform tag
test("basic test", async () => {
    await mountWithCleanup(Component);
    expect(".selector").toHaveCount(1);     // toHaveCount = most-used matcher
});
```

Hoot public API and `web_test_helpers` re-exports are **stable v18→v19** (no port needed).

---

## Manifest Assets Declaration

### v16-v18

```python
# Tour JS files must be declared in manifest
'assets': {
    'web.assets_tests': [
        '<module>/static/tests/tours/**/*',
    ],
}
```

### v18 Hoot Tests

```python
# Hoot test files also in assets
'assets': {
    'web.assets_unit_tests': [
        '<module>/static/tests/**/*.test.js',
    ],
}
```

---

## Migration Checklist for Tests

### v16 -> v17
- [ ] Replace `SavepointCase` with `TransactionCase`
- [ ] Remove `HttpSavepointCase` (use `HttpCase`)
- [ ] Update `Form` import to `from odoo.tests import Form`
- [ ] Convert old tuples to `Command` API
- [ ] Update tour registration to `registry.category().add()` pattern
- [ ] Be aware of external request blocking (`BlockedRequest`)

### v17 -> v18
- [ ] Replace `type='product'` with `is_storable=True` in test data
- [ ] Optionally use built-in `freeze_time` instead of `freezegun`
- [ ] Use `Like` for SQL query pattern matching where appropriate
- [ ] Update JS tests from QUnit to Hoot framework
- [ ] Rename JS test files from `*_tests.js` to `*.test.js`
- [ ] Update manifest: `web.assets_unit_tests` for Hoot tests
- [ ] Replace `assert.ok()` with `expect().toBeTruthy()`
- [ ] Replace `getFixture()` + `mount()` with `mountWithCleanup()`
- [ ] **CRITICAL:** `sale.order.line.tax_id` is `Many2many` (compute, store) — use `Command.set()`, never plain int
- [ ] `Form()` strictly enforces `invisible` XML modifiers — use direct `_onchange_*()` calls for mutually exclusive fields *(no es nuevo de v18: mismo assert en v17, `odoo/tests/form.py:334`, `Form.__setitem__`)*
- [ ] `account.account` no longer has `company_id` field — remove company domain filters in test data
- [ ] `product.composition` has NOT NULL on `uom_id` — always include `uom_id` in test creation
- [ ] MO `action_confirm()` may not auto-create `stock.move.line` — set `move.quantity` to trigger SML creation
- [ ] Stored computed fields need `self.env.flush_all()` + `record.invalidate_recordset()` when bypassing normal flow
- [ ] Use `-i module` (not `-u`) when running tests on a module not yet installed (else 0 tests run silently)
- [ ] Use `--xmlrpc-port=<alt>` when running tests inside Docker with Odoo already running on default port

### v18 -> v19 (test-framework specifics)
- [ ] **`warmup`** internals changed (`*args,**kwargs` → `func, /` + `@wraps`); `@warmup`
      usage unchanged but re-verify if you import/wrap it manually
- [ ] **`start_tour`**: drop `step_delay` from committed launchers (v19 warns it is local-only)
- [ ] **Tour `stepUtils`** import: `@web_tour/tour_service/tour_utils` → `@web_tour/tour_utils`
- [ ] Tour `registry.category("web_tour.tours")` API + step schema unchanged (just the dir moved)
- [ ] **Hoot** stays the default runner; API stable — no Hoot test changes needed v18→v19
- [ ] Use `<list>` instead of `<tree>` in any test XML
- [ ] Update search views: no `<group>` tags
- [ ] Check tour steps for v19-specific view changes
- [ ] Verify controller routes haven't changed
- [ ] Remove `nocopy=True` from all `safe_eval()` calls — parameter removed in v19
- [ ] Form class strictly enforces invisible/missing fields — verify view XML before using Form
- [ ] Always set locale-dependent params (csv separators, date formats) explicitly in tests
- [ ] Mock methods that use dynamic `self._attribute = value` on models (`__slots__` blocks it)
- [ ] See [patterns/21-odoo-v19-gotchas.md](patterns/21-odoo-v19-gotchas.md) for comprehensive gotchas
