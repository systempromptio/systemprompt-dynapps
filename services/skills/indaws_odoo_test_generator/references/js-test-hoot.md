<!-- verified: v18.0/v19.0 @ 2026-06 -->
# JavaScript Unit Tests — Hoot (v18+) vs QUnit (≤v17 en exclusiva; sigue presente en v18/v19)

Source-verified against `web/static/lib/hoot/` and real `*.test.js` in `web/static/tests/`.

> **Two different things, don't confuse them:**
> - **Hoot / QUnit** = *unit/component* tests for JS (OWL components, services, utils). Run in
>   a JS test runner, NOT through Python.
> - **Tours** = *end-to-end* UI tests launched from Python `HttpCase.start_tour()`. See
>   `tour-patterns.md`. Tours are not Hoot tests.

---

## 1. Which framework, which version

| Version | Framework | File naming | Asset bundle | Runner default |
|---------|-----------|-------------|--------------|----------------|
| ≤ v16 | QUnit | `*_tests.js` | `web.qunit_suite_tests` | QUnit |
| v17 | **QUnit, solo QUnit** — Hoot NO existe aun | `*_tests.js` | `web.qunit_suite_tests` | QUnit |
| **v18** | **Hoot llega aqui** (runner por defecto); QUnit sigue vivo en paralelo | `*.test.js` (Hoot) / `*_tests.js` (QUnit) | `web.assets_unit_tests` + `web.qunit_suite_tests` | **Hoot** |
| **v19** | **Hoot**; QUnit relegado a `tests/legacy` | `*.test.js` | `web.assets_unit_tests` | **Hoot** |

Verified: Hoot lib lives at `web/static/lib/hoot/`; v19 `web/__manifest__.py:471` declares
`web.assets_unit_tests`; 293 `*.test.js` files ship in v19 `web/static/tests`.

**Hoot no existe en v17** (corregido 2026-08-07, medido): el directorio `static/lib/hoot/` no esta,
y hay **0 imports de `@odoo/hoot`** en todo v17 frente a **1.420 en v18**. Cuidado con grepear
`hoot` a secas en v17: da 2 resultados y los dos son ruido —`emoji_data.js` (la palabra como
keyword de un emoji) y `luxon.js`—. Los 17 `*.test.js` de v17 son especificaciones del
odoo-editor, no tests de Hoot.

Y al reves: **QUnit no muere en v18**. Quedan 489 `QUnit.test(` en v18 y **74 en v19**, todos bajo
`static/tests/legacy`, con su bundle `web.qunit_suite_tests` todavia en el manifest de v19
(`:520`) y la lib cargada por `web.tests_assets` (`:482-483`). Para GENERAR, escribe Hoot en v18+;
para LEER el arbol, no asumas que QUnit desaparecio.

---

## 2. Hoot public API — VERIFIED exports

From `web/static/lib/hoot/hoot.js`:

| Export | Line | Use |
|--------|------|-----|
| `describe` | 48 | Group / suite. `describe("My feature", () => { ... })`. |
| `expect` | 49 | Assertion entry point. |
| `test` | 50 | Define a test. `test("does X", async () => { ... })`. |
| `before` / `after` | 53/54 | Suite-level hooks. |
| `beforeEach` / `afterEach` | 55 | Per-test hooks. |
| `getFixture` | 60 | DOM fixture root (rarely needed directly). |
| `mockDate`, `mockTimeZone` | 80 | Time mocking (no freezegun in JS). |

DOM querying helpers come from a **separate module** `@odoo/hoot-dom`
(`queryAll`, `queryAllTexts`, `click`, `on`, …).

Component/test plumbing comes from `@web/../tests/web_test_helpers` (verified exports):
`mountWithCleanup` (line 41), `contains` (43), `onRpc`, `defineModels`, `models`, `fields`,
`mountView`, `makeMockEnv`, `getService`, `patchWithCleanup` (97), `serverState` (96).

---

## 3. Minimal Hoot skeleton

```javascript
import { describe, expect, test } from "@odoo/hoot";
import { mountWithCleanup } from "@web/../tests/web_test_helpers";
import { Component, xml } from "@odoo/owl";

// Choose ONE environment tag — see §6.
describe.current.tags("desktop");   // or "headless" / "mobile"

describe("MyWidget", () => {
    test("renders the label", async () => {
        class MyWidget extends Component {
            static template = xml`<div class="my-widget"><t t-esc="props.label"/></div>`;
            static props = ["label"];
        }
        await mountWithCleanup(MyWidget, { props: { label: "Hello" } });

        expect(".my-widget").toHaveCount(1);
        expect(".my-widget").toHaveText("Hello");
    });
});
```

`mountWithCleanup` auto-destroys the component after the test (no manual `getFixture()` +
`mount()` + teardown like QUnit).

---

## 4. `expect` matchers — VERIFIED (by real-world frequency)

| Matcher | Meaning |
|---------|---------|
| `expect(sel).toHaveCount(n)` | n elements match selector (most used). |
| `expect(value).toBe(x)` | strict `===`. |
| `expect(value).toEqual(x)` | deep equality. |
| `expect(sel).toHaveText("…")` | element text content. |
| `expect(sel).toHaveClass("…")` | element has CSS class. |
| `expect(sel).toHaveValue("…")` | input value. |
| `expect(sel).toHaveAttribute("name","v")` | attribute. |
| `expect(sel).toBeFocused()` / `.toBeChecked()` / `.toBeEnabled()` | state. |
| `expect(fn).toThrow(...)` | throws. |
| `expect(value).toMatchObject({...})` | partial object match. |

Negate with `.not`: `expect(".x").not.toHaveCount(0)`.

### Step assertions (replaces QUnit `assert.verifySteps`)

```javascript
import { onRpc } from "@web/../tests/web_test_helpers";

onRpc((params) => { expect.step(params.route); return false; });
// ... do work ...
expect.verifySteps(["/web/dataset/call_kw/res.partner/read"]);
```

---

## 5. Mocking RPC and models (no real server)

```javascript
import { defineModels, fields, models, onRpc, mountView } from "@web/../tests/web_test_helpers";

class Partner extends models.Model {
    _name = "res.partner";
    name = fields.Char();
    _records = [{ id: 1, name: "Acme" }];
}
defineModels([Partner]);

test("list shows the record", async () => {
    onRpc("has_group", () => true);
    await mountView({ type: "list", resModel: "res.partner", arch: `<list><field name="name"/></list>` });
    expect(".o_data_row").toHaveCount(1);
});
```

---

## 6. Mandatory tag

Every Hoot test suite must declare a platform tag or it is skipped:

```javascript
describe.current.tags("desktop");   // most UI tests
// "mobile"   → small-screen layout
// "headless" → no DOM mounting (services, utils, pure logic)
```

---

## 7. Manifest declaration

```python
'assets': {
    'web.assets_unit_tests': [
        'my_module/static/tests/**/*.test.js',
    ],
}
```

(Tours go in `web.assets_tests`, not here.)

---

## 8. Running Hoot tests

Hoot runs in the browser, not via `--test-tags`. Two ways:

```bash
# 1. Headless via HttpCase (CI): a Python test calls browser_js on the Hoot runner URL,
#    or use the web test that loads /web/tests. Module JS tests are auto-collected.
./odoo-bin -d testdb -i my_module --test-enable --test-tags /my_module --stop-after-init

# 2. Interactive: open in a browser
#    http://localhost:8069/web/tests?module=my_module
```

### Trampas del runner Hoot (aislar la ejecución) — verificado v19

Correr Hoot vía `HttpCase.browser_js('/web/tests?...')` no aísla tu módulo:

1. **`/web/tests` ejecuta la suite ENTERA** de TODOS los módulos instalados, no solo el tuyo. El `filter=`/`module=` de la URL filtra qué se *muestra*, **no** lo que se carga/ejecuta en el bundle.
2. **`unit_test_error_checker` tumba el run con CUALQUIER mensaje de consola sin prefijo `[HOOT]`** (`addons/web/tests/test_js.py:16`, `unit_test_error_checker`). Un módulo enterprise que loguea un error, o un símbolo no definido en el bundle, rojea el run **global** aunque tu módulo esté verde.
3. → **Ejecuta los tests JS de un módulo en una BD con ese módulo como único extra** (CE, sin enterprise), con Chromium presente en el contenedor. Envuelve tus tests en `describe("<modulo>", ...)`: en el log salen como suite `@<modulo>` (`[HOOT] "@x" ended (passed: N)`), lo que permite verificar que corrió aislado.

---

## 9. QUnit → Hoot migration cheatsheet (v17 → v18)

| QUnit (unico en ≤v17) | Hoot (v18+) |
|--------------|----------------|
| `QUnit.module("X")` | `describe("X", () => { ... })` |
| `QUnit.test("t", async (assert) => {})` | `test("t", async () => {})` |
| `assert.ok(x)` / `assert.equal(a,b)` | `expect(x).toBeTruthy()` / `expect(a).toBe(b)` |
| `assert.containsOnce(t, sel)` | `expect(sel).toHaveCount(1)` |
| `assert.strictEqual(el.textContent, "x")` | `expect(sel).toHaveText("x")` |
| `assert.verifySteps([...])` | `expect.verifySteps([...])` (+ `expect.step()`) |
| `getFixture()` + `mount(C, target)` | `await mountWithCleanup(C)` |
| `*_tests.js` | `*.test.js` |
| import from `web.test_utils` / `@web/../tests/helpers/utils` | `@odoo/hoot`, `@odoo/hoot-dom`, `@web/../tests/web_test_helpers` |
| asset bundle `web.qunit_suite_tests` | `web.assets_unit_tests` |

### QUnit reference (legacy ≤v17 only)

```javascript
QUnit.module("MyModule");
QUnit.test("basic test", async (assert) => {
    const target = getFixture();
    await mount(Component, target, { env });
    assert.containsOnce(target, ".selector");
});
```

---

## 10. v18 → v19 differences (Hoot)

The public Hoot API (`describe`/`test`/`expect`, `mountWithCleanup`, `onRpc`,
`@web/../tests/web_test_helpers`) is **stable across v18 and v19** — verified that both ship
the same `hoot.js` exports and `web_test_helpers` re-exports. No code change is required to
port a Hoot test from v18 to v19. (Contrast with tours, where `stepUtils` moved — see
`tour-patterns.md §11`.)
