# Gate Criteria & Thresholds

## Gate 1 · Requirements Alignment

| Criterion | Threshold |
|---|---|
| Every stated requirement maps to an implementation | 100% coverage |
| Requirements are testable (have measurable acceptance criteria) | All must be verifiable |
| No undocumented assumptions or implicit requirements | Zero tolerance |

**How to evaluate:**
- Compare the requirements (commit messages, PR description, task description, inline comments) against the implementation
- Flag any requirement that is unaddressed, ambiguous, or untestable
- If no explicit requirements are available, infer from the code's intent and flag as WARN

## Gate 2 · Code Quality & Coverage

| Criterion | Threshold |
|---|---|
| Unit test pass rate | 100% (zero failures) |
| Line/branch coverage on changed files | >= 80% |
| No critical static analysis violations | Zero high/critical severity |
| Functions <= 40 lines | Advisory (WARN if breached) |
| Cyclomatic complexity <= 10 | Advisory (WARN if breached) |

**How to evaluate:**
- Review code structure, naming, error handling, and separation of concerns
- Identify specific functions or modules that violate thresholds
- Check for: unused imports, dead code, overly broad exception handling, missing type hints (if project uses them)
- Run available linters/test commands if a test runner is configured
- For Python: check PEP 8 compliance, docstring presence on public APIs
- For JavaScript/TypeScript: check ESLint rules, proper async/await usage
- **Front Odoo (si el diff toca `static/src`)** — tres fallos que NO produce ningún error y que
  ningún test ve, así que hay que mirarlos aquí:
  - Cada `var(--x)` usado en un `.scss` está DECLARADO. Un `var()` inexistente resuelve a vacío
    (blanco sobre blanco pasa todos los tests). Comprobación: `scripts/scss_undeclared_vars.sh
    <módulo> <fuente-de-odoo>` (resuelve la fuente con
    `references/shared/odoo-source.md`; no la des por supuesta) — **es un generador de
    candidatos, no un veredicto**, y cada uno se
    confirma en segundos mirando la línea que imprime.

    > El one-liner que había aquí antes **no servía, y se comprobó ejecutándolo**: su patrón
    > `[a-z-]` trunca cualquier nombre con mayúsculas, dígitos o `_` —o sea casi todos los de Odoo,
    > que usa `--Avatar-size` y `--DateTimePicker__Cell-size-md`—, así que sobre
    > `web/static/src/core/avatar/avatar.scss` lo que reportaba era `--` a secas. Y comparaba
    > declaraciones y usos **del mismo fichero**, cuando en Odoo se declaran en otro.
    >
    > El script cubre las cuatro formas de declarar que hay en el árbol real (`--x:`, el mixin
    > `print-variable(x, …)`, el `t-attf-style` de una plantilla QWeb, y cualquier `.css`), ignora
    > `var(--x, fallback)` —que no produce el fallo— y salta `static/lib` (código de terceros que no
    > es tuyo). **Lo que no ve**, y por eso es candidato y no veredicto: lo que declara Bootstrap
    > generando nombres desde un mapa Sass (Odoo lo compila con `$variable-prefix: ''`, de ahí
    > `--primary` o `--border-radius`), las interpolaciones `--#{$x}-size`, y lo que pone el JS con
    > `style.setProperty`. Sobre el `addons/web` de una v18 deja **6 candidatos**, todos de esa familia.
  - Ningún control interactivo depende SOLO de `:hover` para ser visible (inalcanzable en táctil,
    por teclado y para un tour).
  - Ningún texto de usuario dentro de un `t-att-*` (no se extrae para traducir, ver
    `developer-skills:odoo-i18n-manager`).

## Gate 3 · Test Readiness

| Criterion | Threshold |
|---|---|
| Test environment reproducible from provided config | Single command setup |
| Test data/fixtures included or documented | 100% of tests runnable |
| No external service dependencies without mocks/stubs | Zero unmocked externals |

**How to evaluate:**
- Could a new developer clone, install, and run the full test suite in under 10 minutes with no tribal knowledge?
- Check for: hardcoded paths, missing fixture files, undocumented environment variables
- Verify test isolation — tests must not depend on execution order or shared mutable state

## Gate 4 · User Acceptance

| Criterion | Threshold |
|---|---|
| All acceptance criteria from Gate 1 demonstrated | 100% |
| No Severity 1 or Severity 2 open bugs | Zero S1/S2 |
| Edge cases and error states handled gracefully | Errors show actionable messages |
| **Escenario de flujo funcional EJECUTABLE presente** (estándar de entrega) | Salvo nivel *trivial*: existe en `tests/` un flujo end-to-end re-ejecutable (tour/HttpCase/escenario `odoo-flow-tester`) que cubre el cambio. Ausente → **FAIL** |

**Estándar de entrega (Desarrollador 2.0):** el nivel (trivial/estándar/complejo) sale de
`references/shared/delivery-thresholds.json` según señales del cambio (modelos tocados,
base/compartido, cross-módulo, migración, seguridad, horas). Detalle: `references/shared/delivery-standard.md`.
El gate verifica la **evidencia en el repo** (escenario ejecutable + README); la sección "Pruebas
funcionales realizadas" de la tarea Odoo la revisa el consultor re-ejecutando el escenario. Un override
`NIVEL: <nivel> — <razón>` en el diff/PR baja/sube la exigencia (déjalo constar en el veredicto).

**"Tour verde" ≠ "verificado".** Un tour headless prueba que los selectores existen y que el flujo
encadena; NO ve color, contraste, espaciado, ni si algo es alcanzable sin ratón, ni si el texto está
en el idioma del usuario. Al reportar el gate, distingue explícitamente:
`verificado en navegador` · `cubierto por tour/test` · `solo revisado en código`. Marcar como
verificado algo que solo tiene tour es la forma más común de que un fallo visual llegue al cliente.

**Severity definitions:**
- **S1 (Critical):** System crash, data loss, security vulnerability, complete feature failure
- **S2 (Major):** Feature partially broken, significant UX degradation, workaround required

**How to evaluate:**
- Walk through each user-facing flow conceptually based on the code
- Flag broken paths or missing error handling with specific reproduction steps
- Check for: unhandled exceptions, missing validation, unclear error messages, silent failures

## Gate 5 · Production Readiness

| Criterion | Threshold |
|---|---|
| README / deployment docs updated | All changed components documented |
| **Manual/README de uso-configuración** (estándar de entrega) | Solo nivel *complejo*: el módulo tocado tiene README de uso/config creado o actualizado en el repo. Ausente → **FAIL** |
| Rollback plan defined | Must specify exact rollback steps |
| Environment variables documented (not hardcoded) | Zero hardcoded secrets |
| Database migrations reversible | Down migration exists (if applicable) |

**How to evaluate:**
- Could an on-call engineer deploy, monitor, and roll back this change at 3 AM without the original author?
- Check for: hardcoded URLs, API keys, passwords, connection strings
- Verify logging is adequate for debugging production issues
- Check for: missing migration files, irreversible schema changes
