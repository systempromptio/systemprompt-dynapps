---
name: indaws-odoo-test-generator
description: Use this skill when the user needs to create tests for Odoo modules (versions 14.0-19.0). Generates Python unit tests (TransactionCase, HttpCase, SingleTransactionCase), performance tests (@warmup, assertQueryCount), tour tests, security/access tests, Form-based onchange tests, QWeb report render tests, and JavaScript component tests. Handles test scaffolding from code analysis, test data setup, assertion patterns, and version-specific test APIs. Supports generating tests for models, views, controllers, wizards, reports, and JS components following Odoo's official test patterns.
version: 2.1.11
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, EnterPlanMode]
effort: medium
model: opus
---
<!-- verified: v18.0 @ 2026-06 -->

# Odoo Test Generator Skill

Generates comprehensive tests for Odoo 14-19 modules. Analyzes code to determine optimal test strategies and produces tests following official Odoo patterns from core modules (`sale`, `account`, `stock`, `base`).

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Scan module | sonnet | Delegate to scanner |
| Determine test strategy | **opus** | Which test types per component |
| Generate test code | **opus** | Correct Python test classes |

## Coverage Target

When generating a complete test suite (Option C), the skill targets **>= 80% line coverage** measured with `coverage.py`. This aligns with quality-gate (Gate 2) and migration-assistant (Gate 2) thresholds. Options A and B do not enforce a coverage target.

## References (read on demand)

| Ref | When to read |
|-----|-------------|
| [references/workflow-steps.md](references/workflow-steps.md) | **Always** — Start here. Full 7-step workflow with [STOP] points |
| [references/test-types.md](references/test-types.md) | When deciding which test types to generate |
| [references/test-templates.md](references/test-templates.md) | When writing test code — copy-paste templates for all 9 types |
| [references/test-patterns.md](references/test-patterns.md) | Index pointing to per-pattern files in patterns/ (01–23) for on-demand loading |
| [references/detection-matrix.md](references/detection-matrix.md) | When analyzing module to detect testable components |
| [references/patterns/INDEX.md](references/patterns/INDEX.md) | When loading specific test patterns — load ONLY what you need |
| [references/version-differences.md](references/version-differences.md) | When targeting non-v18 versions |
| [references/tour-patterns.md](references/tour-patterns.md) | When generating tour (E2E UI) tests — source-verified step schema, `run` vocab, v18→v19 dir move |
| [references/e2e-flow-tests.md](references/e2e-flow-tests.md) | E2E de **flujo de negocio ORM** scoped al módulo (Gate 4 Dev 2.0): qué cadena vía `map_module_to_flows.py`, `TransactionCase` `@tagged('post_install')` que recorre la cadena y **aserta la contribución del módulo**, anti-hueco |
| [references/js-test-hoot.md](references/js-test-hoot.md) | When generating JS unit/component tests — Hoot (v18+, no existe en v17) + QUnit (≤v17 en exclusiva, pero sigue presente en v18/v19) diff |
| [references/test-gotchas.md](references/test-gotchas.md) | Before delivering tests — failure patterns, **y su §«Falsos VERDES»: las reglas de escritura que salen de cada verde falso** (el catálogo de los 39 casos vive en `common-skills:odoo-verify-green`) |
| [references/adversarial-cases.md](references/adversarial-cases.md) | **Always when generating tests** — boundary/adversarial cases per data category (money/encoding/multi-company/idempotency/security-as-user/state/dates/empty) + 2 golden assertion rules (input must reach output; assert exact invariant, not a proxy). Prevents happy-path "test theater". |

## Quick Start

1. Read [references/workflow-steps.md](references/workflow-steps.md)
2. Follow STEP 0 → gather info [STOP]
3. Follow STEP 0.5 → plan (if Option B/C) [STOP]
4. Follow STEP 1 → **Scan module [SONNET SUBAGENT]:** Launch Agent(model: sonnet, subagent_type: general-purpose, description: "scan module") — invoke odoo-module-scanner on {path}. Output: models, fields, views, security, controllers, JS inventory. Then **Load API ref [SONNET SUBAGENT]:** Launch Agent(model: sonnet, subagent_type: general-purpose, description: "load API ref v{version}") — read odoo-version-api-reference v18-base/{topics} + diffs/v{version}-diff.md if != v18. Output: filtered test API summary. Load patterns from [patterns/INDEX.md](references/patterns/INDEX.md) [STOP]
5. Follow STEP 2 → generate code using [test-templates.md](references/test-templates.md). Apply [adversarial-cases.md](references/adversarial-cases.md): for each component add boundary tests (money/encoding/multi-company/idempotency/security-as-user/...) and obey the 2 golden assertion rules.
6. Follow STEP 5 → validate + present summary. Before delivering, re-check every test against [adversarial-cases.md](references/adversarial-cases.md): "would it pass even if the feature were broken?" → strengthen proxy assertions.
7. Follow STEP 6 → execute coverage (Option C only)
8. **Antes de entregar el «verde», verifícalo** → `common-skills:odoo-verify-green`: cuenta lo ejecutado (`odoo.tests.stats`, no el exit code), confirma base limpia (`-i`, no `-u`), y **rompe a propósito el código que el test protege para verlo caer**. Un test que no se ha visto en rojo no está entregado, está supuesto.

For shared ecosystem patterns, see common:references/shared/.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-verify-green | common | Antes de entregar: comprobar que el pase ejecutó algo y que los tests nuevos caen al neutralizar el código |
| odoo-version-api-reference | common | Test APIs per version |
| odoo-module-scanner | common | Detect components to test |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-module-creator | developer | Generate tests after module creation |
| odoo-flow-tester | common | Generate automated tests for verified flows |

## Related Skills

Los de arriba (Delegates / Consumed by) + **odoo-code-auditor** (developer) — análisis de calidad del
código bajo prueba, incluida la detección de tests huecos.

## Absolute Prohibitions

- NEVER generate tests that depend on demo data
- NEVER use `time.sleep()` — use `freeze_time` or `patch`
- NEVER use hardcoded database IDs
- NEVER use `name_get()` in v18+ — use `display_name`
- NEVER use old-style tuples `(0, 0, {...})` in v16+ — use `Command.create()` (`Command` existe desde **v15**: 0 ocurrencias en v14)
- NEVER assert on translatable strings — assert on structure/keys/technical values
- NEVER use string import paths for mocking — use `patch.object(type(record), "method")`
- NEVER pass plain `int` to Many2many — `Command.set([id])`. Medido: una **lista** de ids sí funciona (v18/v19) y equivale a `(6,0,ids)` — reemplaza sin decirlo; el int suelto da `ValueError: Wrong value…`
- NEVER **escribir** con `Form()` en un campo invisible **en v17/v18/v19** (el assert es el mismo: `odoo/tests/form.py:334`, `Form.__getattr__` v17, `:361` v18, `:372` v19) — call onchange directly. **Leerlo sí se puede** (medido), y por eso un test puede afirmar sobre lo que la pantalla no muestra
- NEVER filter `account.account` by `company_id` in v18+ — es `company_ids` (m2m) desde v18; el `company_id` que verás en ese fichero es de `account.group`
- NEVER call `cr.commit()`, `cr.rollback()` **ni `cr.close()`** en un test — los tres están parcheados. Pero *«forbidden by framework»* **solo desde v18**: en v17 no hay guardián y el commit pasa en silencio, corrompiendo el cursor del test en vez de fallar (ver Gotchas)

**Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-test-generator · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Gotchas

**TEST HONESTY (non-negotiable)** — a test that always passes is worse than no test, y **el runner también miente**: ¿aparece el módulo en `odoo.tests.stats`? ¿cuántos tests se **ejecutaron** (recogidos − saltados) o solo miras el exit code? ¿hay tours `-standard` que el pase por defecto no corre? Reglas de escritura en `references/test-gotchas.md` §«Falsos VERDES»; los 39 casos medidos y el procedimiento de verificar en rojo, en `common-skills:odoo-verify-green` — el más barato: **un módulo cuyo nombre acaba en `py` aporta 0 tests y sale verde**.
- The method name must describe EXACTLY what it asserts. `test_invoice_total` must assert the total; if it only checks the invoice was created, name it `test_invoice_is_created`.
- Assert **behavior/values**, never mere absence of an exception. `record.action_confirm()` followed by no assertion is not a test — assert `state == 'sale'`, the computed amount, the created move, etc.
- A tour must end with Python assertions on the resulting records (see tour-patterns.md §2), not just "the tour finished".
- Coverage % is NOT proof of correctness. 80% lines executed with weak/no assertions = 0% verified behavior. Lines hit ≠ behavior checked.
- No tautological assertions (`assertEqual(x, x)`), no asserting framework internals.

**Reports in tests render HTML, not PDF** — under `--test-enable`, `_render_qweb_pdf` falls back to `_render_qweb_html` (`odoo/addons/base/models/ir_actions_report.py:1011`, `IrActionsReport._pre_render_qweb_pdf`). Assert on `_render_qweb_html(...)`, not on a `%PDF` byte stream; pass `.with_context(force_report_rendering=True)` only if you truly need the wkhtml path. Run report tests with `--test-tags /<module>`. Detail: `references/test-gotchas.md` §16.

**Code in English** — test class/method names, comments, docstrings and assertion messages always in English regardless of client locale. See `references/shared/coding-standards.md`.

**v18 AND v19 FORBID `cr.commit()` inside a test** — not a v19 novelty. The guard patches **three** cursor methods (`commit`, `rollback`, `close`) and its `AssertionError` text is byte-identical in both — *"Cannot commit or rollback a cursor from inside a test, this will lead to a broken cursor when trying to rollback the test. Please rollback to a specific savepoint instead or open another cursor if really necessary"*: v18 `odoo/tests/common.py:1059-1065`, `setUpClass.forbidden` @99f5a33c64`, v19 `:1069-1078 @d1d161033`. **La implementación NO es la misma, y en v19 deja una puerta** (medido 2026-08-17): v18 usa `cls.classPatch(cls.cr, 'commit', forbidden)` (`:1063-1065`) — que por dentro ya es `startClassPatcher(patch.object(...))`, `odoo/tests/common.py:432-434`, `BaseCase.classPatch` — y **no guarda referencia**; v19 crea los patchers y **los deja colgados de la clase**: `cls.commit_patcher`, `cls.rollback_patcher`, `cls.close_patcher` (`:1073-1078`). O sea que en v19 un test puede hacer `cls.commit_patcher.stop()` y saltarse el guardián; en v18 no hay a qué agarrarse. Si revisas un test que commitea y «funciona», mira si alguien paró un patcher. **In v17 the guard does not exist** (0 occurrences) — a test that commits there will corrupt the cursor silently instead of failing loudly. To prove something is visible to ANOTHER transaction (a write on a separate connection: progress indicators, crash-safe logs, sinks), open an independent connection — `from odoo.sql_db import db_connect; conn = db_connect(self.env.cr.dbname)` — do the INSERT/SELECT there and clean up in a `finally`. A `TransactionCase` cannot see its own uncommitted rows from that connection, so create the fixture row there too.

**A hover-only control cannot be clicked by a tour** — and that is a fair proxy for "no touch device and no keyboard can reach it either". If a tour times out on a button that clearly exists, check for `opacity: 0` / `display:none` until `:hover` before suspecting the selector. Fix the UI (dim, don't hide), not the tour.

**`assertRaises` de Odoo envuelve el bloque en un SAVEPOINT** (`BaseCase.assertRaises` → `self.cr.savepoint()`), y ese savepoint se DESHACE al saltar la excepción. Así que todo lo que la llamada fallida registró antes de lanzar —el contador de errores, el "pausada", el log de la incidencia— desaparece, y el test mide el rollback en vez del comportamiento. Para "registra el fallo y luego lanza", captura con `try/except` y afirma sobre el registro.

**Dos elementos mutuamente excluyentes NO deben compartir la clase que dice QUÉ son** — un `t-if="state.busy"`/`t-else` con `class="o_ai_send"` en ambos hace que `.o_ai_send` case con el que esté en pantalla: el tour apunta a Enviar y pulsa PARAR si el turno anterior aún transmite, y el síntoma aparece dos pasos después ("la respuesta no llega"). El tiempo decide qué botón te toca → rojo intermitente en la suite completa y verde en aislamiento. Clase compartida para la FORMA, clase propia para el SIGNIFICADO; y un test estático sobre la plantilla lo fija mejor que el tour.

**`odoo.tests.result` does not survive `docker compose run --rm`** — the tally line you can trust in a disposable container is `odoo.tests.stats`. Grepping for the wrong one makes a green run look silent and a red one look green.

**`at_install` runs while the registry is still being built** — creating a `res.partner` there can fail on another module's required field whose default is not set up yet (`autopost_bills`). If a test blows up on a field your code never touches, tag it `post_install`, `-at_install` before debugging further.

Test-specific gotchas (15 v18 entries, 15 v19 entries) are in [references/test-gotchas.md](references/test-gotchas.md). For version-specific API gotchas shared across skills, see `/odoo-version-api-reference` → `references/version-differences.md`.

**Always check gotchas before delivering tests.**

**ASSESSING existing coverage** ("does this already have a test?") — NEVER grep the method name: Odoo tests rarely name the method under test, so name-grep gives massive false positives AND negatives. Only `coverage.py` line/branch measurement (see Test Execution) is trustworthy. Per-type detection patterns in [references/test-gotchas.md](references/test-gotchas.md).

## Test Execution

```bash
# All tests for a module
./odoo-bin -d testdb --test-tags /module_name -u module_name --stop-after-init

# Specific class
./odoo-bin -d testdb --test-tags /module_name:TestClassName --stop-after-init

# Docker (avoid port conflict)
docker exec container python3 /path/odoo-bin --test-tags /module -u module --stop-after-init --no-http --xmlrpc-port=18199 -d DB -c /etc/odoo/odoo.conf

# With coverage (Option C — complete suite)
pip install coverage 2>/dev/null
coverage run --source=<module_path> odoo-bin -d testdb --test-tags /module -u module --stop-after-init --no-http
coverage report --include="<module_path>/*.py" --omit="*/tests/*" --fail-under=80
coverage report -m --include="<module_path>/*.py" --omit="*/tests/*"  # show missing lines
```

## Manifest Integration

Tour JS files must be declared:
```python
'assets': {'web.assets_tests': ['<module>/static/tests/tours/**/*']}
```
