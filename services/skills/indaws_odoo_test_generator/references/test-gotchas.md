<!-- verified: v18.0/v19.0 @ 2026-06 -->
# Test-Specific Gotchas

Gotchas specific to test generation. For version-specific API gotchas (name_get, `<tree>`, search `<group>`, etc.), see `/odoo-version-api-reference` → `references/gotchas.md` (48 entries).

---

## v18 (Production-Validated on 17+ modules)

### 1. **`sale.order.line.tax_id` is Many2many**

**Trigger:** Writing to `tax_id` on SOL in any test
**Error:** Using plain `int` assignment
**Fix:** Use `Command.set([id])`, not plain int

### 2. **`Form()` enforces invisible modifiers — al ESCRIBIR, no al leer**

**Trigger:** Using `Form()` to set a field that depends on another field's visibility
**Error:** Setting invisible fields via Form raises `AssertionError: can't write on invisible field`.
**No es de v18**: el assert es textualmente identico en v17 (`odoo/tests/form.py:334`, `Form.__getattr__`), v18 (`:361`) y
v19 (`:372`). Aplica a las tres.
**MEDIDO (v18 y v19, `bench-orm/.../test_testgen.py::TestFormInvisible`): leerlo SÍ se puede.**
`f.campo_invisible` devuelve el valor sin protestar; la guardia solo cubre la escritura. Importa por
honestidad del test: un test puede **afirmar sobre un campo que esa pantalla no muestra** y parecer
que comprueba lo que ve el usuario. Si lo que quieres verificar es la pantalla, comprueba primero
que el campo está en el arch; si lo que quieres es el dato, no uses `Form` — usa el recordset.
**Y hay un SEGUNDO assert que casi nadie ve** (medido 2026-08-17): editar un x2many invisible falla
con **otro mensaje**, desde `X2MProxy._assert_editable` — `assert not ... 'readonly'` y
`assert not ... 'invisible'`, con el texto `field 'X' is not visible` (v17 `odoo/tests/form.py:821`, `X2MProxy._assert_editable`,
v18 `:855`, v19 `:867`; la clase es `X2MProxy`, v18 `:835`, padre de `O2MProxy` y `M2MProxy`).
O sea: `f.campo_invisible = 1` da *"can't write on invisible field"* y
`with f.lineas_invisibles.new() as l:` da *"field 'lineas_invisibles' is not visible"*. Buscar el
primer mensaje en el log y no encontrarlo **no significa** que el problema no sea este.
**Fix:** Call onchange directly instead of using `Form()`

### 3. **`product.composition` NOT NULL on `uom_id`**

**Trigger:** Creating `product.composition` test records
**Error:** Omitting `uom_id` field
**Fix:** Always include `uom_id` when creating product.composition records

### 4. **MO `action_confirm()` may not create SML**

**Trigger:** Testing manufacturing order confirmation
**Error:** Assuming stock.move.line is created automatically
**Fix:** Set `move.quantity` first before asserting on SML

### 5. **Stored computed fields need flush**

**Trigger:** Asserting on computed stored fields after direct ORM create (bypassing UI)
**Error:** Stale cache returns wrong value
**Fix:** Call `flush_all()` + `invalidate_recordset()` before assertion

### 6. **`-u module` requires installed module**

**Trigger:** Running tests on a module not yet installed
**Error:** Test runner fails silently
**Fix:** Use `-i` for first install, `-u` only for already-installed modules

### 7. **Docker port conflicts**

**Trigger:** Running tests inside a container with Odoo already serving
**Error:** Port 8069 already in use
**Fix:** Use `--xmlrpc-port=18199` and `--no-http` for test execution

### 8. **`account.account` has no `company_id`**

**Trigger:** Filtering accounts by company in test domains
**Error:** Field removed in v18 — verificado: v17 lo tiene (`account/models/account_account.py:102`, `AccountAccount.tax_ids`),
v18 y v19 lo sustituyen por **`company_ids` (Many2many)** (`:106` v18, `:97` v19).
**Trampa al comprobarlo:** en ese mismo fichero hay un `company_id` más abajo (`:1455` v18, `:1508`
v19) que es de **`account.group`**, no de `account.account`. Un grep del nombre lo encuentra y parece
desmentir la gotcha.
**Fix:** Don't filter `account.account` by `company_id` — usa `company_ids`

### 9. **Computed stored fields don't trigger write() hooks**

**Trigger:** Testing custom write() hooks on compute/store/readonly=False fields (tax_id, tax_ids)
**Error:** Hook never fires because ORM computes without calling write()
**Fix:** Write with skip context + call hook explicitly

### 10. **Confirmed SO lines can't be unlinked**

**Trigger:** E2E tests that delete/filter SOL then call `action_confirm()`
**Error:** Unlink raises error on confirmed lines
**Fix:** Filter lines BEFORE calling `action_confirm()`

### 11. **Fiscal year dates**

**Trigger:** Any test that uses date ranges (reports, filters, periods)
**Error:** Hardcoding date ranges that don't match fiscal year
**Fix:** Use `compute_fiscalyear_dates()` to get correct boundaries

### 12. **JSON fields with `//` comments**

**Trigger:** Testing modules that allow user-editable JSON config with inline comments
**Error:** `json.loads()` fails on `//` comments
**Fix:** Strip comments before parsing: `re.sub(r'//.*', '', text)` then `json.loads()`

### 13. **`with_context` flags travel with the recordset into later `write()`**

**Trigger:** Building fixtures with a skip flag (`.with_context(_comp_skip_price_redistribute=True, _skip_trigger=True).create({...})`), then exercising write-path logic on that SAME recordset (or its `order_line`/children).
**Error:** The context flag propagates to every subsequent `.write()`, so the behavior under test (redistribute, propagation, sync) is silently skipped → test passes/fails for the wrong reason (e.g. a child stays at its created value).
**Fix:** Re-browse to drop the setup context before acting: `order = self.env['sale.order'].browse(order.id)`. This mirrors a real UI save (clean context). Distinct from #9 (which is about hooks not firing at all).

### 14. **`Form()` runs the onchange cascade; `create()` does NOT (in tests)**

**Trigger:** Testing features that auto-generate *related* lines via onchange (composition children, kit/BoM explosion, downpayments).
**Error:** `self.env['sale.order'].create({'order_line': [Command.create({'product_id': kit.id})]})` yields NO child lines in test context — the stored-compute cascade doesn't materialize them — so E2E assertions on children find nothing (or the test `skipTest`s).
**Fix:** Use `odoo.tests.Form` for the realistic/UI flow (children appear). To cover the API `create()` path specifically, build the child lines manually with the skip context (see #13). Don't assume `create()` == UI.

### 15. **Compute without `@api.depends` goes stale against `setUpClass` data**

**Trigger:** A stored compute (or precompute) that reads records created earlier in the same `setUpClass` — e.g. a line's `has_comp` reading `product.composition_ids`, or `price_unit` precomputed from `product.list_price`.
**Error:** With no `@api.depends`, the field never recomputes: it caches the value from before the source rows existed (empty M2M, price 0), so the line's flag/price is wrong and dependent logic no-ops. `price_unit` set in `create()` vals is also overwritten by the precompute when the line doesn't yet own its guard field (e.g. `comp_child_ids`).
**Fix:** Invalidate the source then force the compute: `product.invalidate_recordset(['composition_ids']); line._compute_has_comp()`. For precomputed `price_unit`, set it AFTER the dependent rows exist via write-with-skip (#13), not in the create vals.

### 16. **`_render_qweb_pdf` returns HTML, not PDF, under `--test-enable`**

**Trigger:** A test that prints a report and asserts on the bytes (`self.assertTrue(pdf.startswith(b'%PDF'))`) or diffs the PDF.
**Error:** With `--test-enable` (or `--test-file`), `_render_qweb_pdf` **falls back to `_render_qweb_html`** — report rendering to real PDF is disabled in the test env to avoid needing wkhtmltopdf workers (verified `odoo/addons/base/models/ir_actions_report.py:1011`, `IrActionsReport._pre_render_qweb_pdf`: `if (config['test_enable'] or config['test_file']) and not context.get('force_report_rendering'): return self._render_qweb_html(...)`). So you get **HTML**, not a `%PDF` stream — the assert fails or you diff the wrong thing.
**Fix:** In tests, assert on `_render_qweb_html(report_ref, ids)` (checks the content/QWeb output) — that's what actually runs. If you genuinely need the real PDF path (e.g. testing the wkhtml pipeline), pass `.with_context(force_report_rendering=True)` to force it. Run report tests with `--test-tags /<module>`.

### 17. **queue_job: `with_delay()` async no se ejecuta solo en un docker dev**

**Trigger:** Verificar comportamiento asíncrono (una acción/flujo que hace `.with_delay(...)`, o un trigger que encola en modo async) contra un Odoo dockerizado, y ver que "no pasa nada".
**Error:** El jobrunner de OCA `queue_job` no corre salvo que esté configurado (`--load=web,queue_job` + `channels` + workers>0). En un docker de dev (workers=0 para debugpy) el job queda en estado `queued` indefinidamente y el efecto nunca ocurre — no hay error, solo silencio.
**Fix:** En tests automáticos usa **`trap_jobs()`** (`from odoo.addons.queue_job.tests.common import trap_jobs`) para capturar y ejecutar el job sin jobrunner. Para verificación manual: invoca el método destino de forma **síncrona** (el equivalente `delay=False`, o llama al método subyacente directamente) en vez de esperar a la cola, o arranca con el jobrunner activo.

### 18. **Un `assertQueryCount` sin `@warmup` cuenta el arranque, no el régimen**

**Trigger:** Un test de rendimiento escrito midiendo la primera pasada.
**Error:** MEDIDO (v18 y v19, `bench-orm/.../test_testgen.py::TestQueryCount`): la **misma** lectura,
como usuario normal, cuesta **más consultas la primera vez que la segunda** — la primera paga además
los permisos y las reglas, que se cachean para el resto de la transacción. El número que fijes
depende entonces de **qué corrió antes**: el test pasa en aislamiento y falla en la suite completa, o
al revés. Es un rojo intermitente que se culpa siempre al código equivocado.
**Fix:** `@warmup` (`from odoo.tests.common import warmup`) — corre el cuerpo dos veces y solo cuenta
la segunda, que es el número de régimen. Existe en las dos (`odoo/tests/common.py:2585`, `warmup` v18,
`:2668` v19).

### 20. **Asertar sobre tracking sin correr el precommit no ve nada**

**Trigger:** un test que comprueba que un `write` dejó su mensaje de seguimiento.
**Error:** MEDIDO (v18 y v19, `bench-orm/addons/dyn_mail`): el tracking **no se escribe en el
`write`**, se encola en `env.cr.precommit` y se materializa al **commit**. Un `TransactionCase` no
commitea nunca, así que justo después del `write` `message_ids` sigue igual y `tracking_value_ids`
está vacío.
**Las dos formas de fallar, y la segunda es peor:**
- el test afirma que hubo mensaje → **rojo falso**, y se acusa al código de negocio;
- el test afirma que **no** hubo mensaje (p. ej. «este cambio no debe notificar») → **pasa siempre**,
  mida lo que mida. Verde falso de manual.
**Fix:** `self.env.cr.precommit.run()` antes de la aserción (o `self.flush_tracking()` donde exista).
Y para el caso negativo, correr el precommit **igualmente**: si no, no estás probando nada.

### 19. **Una LISTA de ids SÍ vale en un x2many; el entero suelto no**

**Trigger:** `create({"tag_ids": [tag.id]})` en un test.
**Error:** ninguno — MEDIDO en v18 y v19: la lista de ids se acepta y **se comporta como
`(6, 0, ids)`**, o sea que *reemplaza* el contenido. El entero suelto sí revienta, y con un mensaje
claro: `ValueError: Wrong value for <modelo>.<campo>: 7`.
**Por qué está aquí:** la prohibición «NEVER pass plain int to Many2many» se lee como si `[id1, id2]`
estuviera prohibido, y no lo está. La razón para usar `Command.set` es que **dice en voz alta que
reemplaza**; la lista de ids hace lo mismo sin decirlo, y ahí es donde se pierde una etiqueta que
alguien esperaba conservar.

---

## v19

See [patterns/21-odoo-v19-gotchas.md](patterns/21-odoo-v19-gotchas.md) for v19-specific gotchas (15 entries).

---

## Test-design honesty (all versions)

### H1. **Name ≠ assertion**
**Trigger:** A method named for behavior X that asserts something weaker.
**Fix:** Rename to match what is actually asserted, or strengthen the assertion. `test_total`
must check the total.

### H2. **Asserting absence of error instead of behavior**
**Trigger:** Calling an action with no follow-up assertion (relying on "it didn't raise").
**Fix:** Assert the resulting state/value/record (`state`, computed amount, created move).

### H3. **Tour with no Python assertions**
**Trigger:** `start_tour(...)` as the last line of the test.
**Fix:** Search the resulting records and assert their state after the tour returns.

### H4. **Coverage mistaken for verification**
**Trigger:** Hitting an 80% target with thin assertions.
**Fix:** Coverage = lines executed, not behavior verified. Each executed branch needs a real
assertion on its outcome.

### H5. **Tautological / framework-internal asserts**
**Trigger:** `assertEqual(x, x)`, asserting ORM internals instead of business behavior.
**Fix:** Assert observable business outcomes only.

## Assessing existing coverage (reverse task)

When the task is "does this functionality already have a test?", **NEVER answer by grepping the
method name in the tests**. In Odoo, tested code almost never names its method in the test:

- a **computed** field is exercised by *reading the field* (`rec.amount_total`)
- a **constraint** by *provoking the error* (`assertRaises(ValidationError)`)
- an **onchange** via `Form()`
- an **action** by *invoking it and asserting the effect/dict*
- a **portal** route via `HttpCase` + `url_open("/my/...")`
- a **cron** via its internal `_cron_*` method

Name-grep yields massive false positives (verified: invoicing/portal/constraints all flagged
"untested" while fully covered) AND false negatives. The only trustworthy answer is
**`coverage.py` line/branch measurement** (see SKILL.md → Test Execution); static analysis at best
tells you a module has *zero* tests, never per-functionality gaps.

## Environment & scope (all versions)

### E1. **El test fija su propia config — NUNCA lee `ir.config_parameter` viva**
Un test que llama a un método config-dependiente y NO fija los parámetros lee el valor de la BD
donde corre (p.ej. producción/staging restaurado) → resultado no determinista. Cicatriz real: un
método así dio **11 tests en rojo** por leer un valor de producción. **Patrón:** en `setUp`/`setUpClass`
haz `self.env['ir.config_parameter'].sudo().set_param(...)` de TODO lo que el flujo lee (feature
flags, endpoints, límites), no asumas defaults.

### E2. **Servicio fuera de Odoo → `unittest` standalone, no `--test-tags`**
Lógica que no es un modelo Odoo (un sidecar/orquestador Python puro, un parser de framing WS, un
reconcile) se testea con `python3 -m unittest` **mockeando** `subprocess`/sockets, no arrancando
Odoo. Cubre lo que el runner de Odoo no puede ver, y corre en milisegundos. Reserva
`--test-tags`/`HttpCase` para lo que toca el ORM o el web client.

---

## Falsos VERDES: qué te obliga a hacer AL ESCRIBIR el test

El catálogo de por qué un verde miente —**39 casos medidos**, con síntoma, causa y cómo se pilla— NO
vive aquí: vive en `common-skills:odoo-verify-green` → `references/false-greens.md`, y el procedimiento de
**verificar en rojo** también. Se dejó una sola copia a propósito: dos catálogos divergen y acabas
citando el que se quedó viejo.

Lo que sí es de esta skill es la **regla de escritura** que sale de cada caso. Aplícalas al generar;
para el diagnóstico, invoca la skill.

| Al escribir un test… | Caso |
|---|---|
| Una batería en un **mixin** necesita `allow_inherited_tests_method = True`, o son «0 tests» en verde | A3 |
| Un **modelo de prueba** declarado en `tests/` se registra pero NO tiene tabla: comprueba la TABLA (`to_regclass`), no el registro | B3 |
| Nada que razone sobre el **registro entero** puede ser `at_install` → `@tagged("post_install")` | B4 |
| Si el test necesita la **ausencia** de algo, créala globalmente: si depende de qué módulos hermanos estén instalados, no es un test | C1 |
| `assertEqual(recordset.campo, 0.0)` sobre un recordset **vacío pasa** (`False == 0.0`): exige primero que la fila exista | C2 |
| Para «registra el fallo y luego lanza», `try/except` a mano — el `assertRaises` de Odoo revierte la traza | C3 |
| Dos ramas del **mismo largo** no prueban un AND: discrimina con largos distintos | C4 |
| Un **tour** no puede afirmar sobre un `<option>` (no es visible) ni casar `select` por etiqueta; el texto se afirma en Python | D2, D3 |
| Generas tours → no entran en el pase por defecto (`-standard`): tocar `static/src/` obliga a correrlos aparte | D1 |
| Migras un módulo a v19 → **comprueba `pg_constraint`**: la lista `_sql_constraints` deja de crear la restricción y los duplicados entran sin que nada falle | B5 |
| Portas tests a otra versión y fallan cosas que no tocaste → **corre lo mismo en la versión vieja antes de culpar a la nueva**: un módulo JS roto tumba el bundle y arrastra a los demás | D8 |
| Rellenas campos y confirmas en el mismo `Form()` → afirma los campos **releyendo el registro**: un `readonly` condicional los tira al guardar y el test pasa igual | C9 |
| Entregas un tour → di también **cómo se corre**: sin `websocket-client` o sin un Chrome que arranque, `SkipTest` y el resumen dice «0 failed». Un tour saltado se lee igual que uno que pasa | D7 |
| Mides con `search_count` → `.with_context(active_test=False)` o el «+0» es mentira | E3 |

Dos que no están en ese catálogo porque son específicos de escribir/ejecutar tests:

### `env.companies` bajo `sudo()` es la del SUPERUSUARIO
Contiene todas las compañías de la base (crear una la añade a la suya). Un test de aislamiento
entre compañías corre con **usuario restringido**, nunca como admin, o prueba cero. Y en producto:
una cláusula de compañía derivada **por debajo** de una elevación se ensancha y **parece puesta**.

### `--no-http` mata las clases `HttpCase` en `setUpClass` (falso ROJO)
`'PreforkServer' object has no attribute 'httpd'`, y parecen fallos de código. Pasa `--http-port`.

---

**Regla de método**, que vive en common porque no es de Odoo: `common-skills:quality-gate` GOTCHA 8 — *una
puerta o un runner que no se ha verificado EN ROJO no está verificado*. Aplicado a un runner de
tests: si no distingue «0 tests ejecutados» de «todo bien», su verde no significa nada.
