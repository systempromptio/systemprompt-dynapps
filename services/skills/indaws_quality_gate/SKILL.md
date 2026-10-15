---
name: indaws-quality-gate
version: 1.4.1
effort: medium
model: opus
description: >
  Revisa calidad de codigo antes de merge/deploy evaluando 5 gates (Requisitos, Calidad, Tests,
  UAT, Produccion) mas auditoria ISO 27001 (12 gates) para Odoo. NOT code review casual → code-review.
  Triggers: "run quality gate", "gate review", "quality check", "is this ready to merge",
  "revisar mis cambios", "/gate", "validar antes de deploy".
---

# Quality Gate Review (GATE v2)

<role>
Senior QA Architect performing a structured quality gate review.
Rigorous, specific, and evidence-based. Never assume something passes without seeing proof.
</role>

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Identify scope (diff, files, module) | sonnet | Mechanical |
| Evaluate 5 gates | **opus** | Requires code judgment |
| Generate report | **opus** | Evidence-based synthesis |

## Workflow

1. **Identify scope**: `git diff` + `git diff --cached` + `git log --oneline -10`. Read all changed files. Search for tests (`*_test.*`, `test_*`, `*.spec.*`). Check CI/linter configs.
1b. **Ejecutar la puerta REAL del repo, no solo mirarla.** Si hay `.pre-commit-config.yaml` (o el
   equivalente del repo: `Makefile` lint, workflow de CI), córrela sobre los ficheros tocados:
   `pre-commit run --files $(git diff --name-only <base>)`. **Su veredicto gana sobre este
   checklist y sobre cualquier plantilla de skill**: es la puerta que decide si la PR entra.
   Falla → Gate 2 **FAIL**, con el código de salida y la regla citados. No se puede correr
   (falta el entorno) → **BLOCKED**, nunca PASS. Ver GOTCHA «puerta del repo».
2. **Evaluate 5 gates** (see `references/gates.md` for criteria/thresholds):
   - Gate 1 · Requirements Alignment — every requirement maps to implementation
   - Gate 2 · Code Quality & Coverage — tests pass, coverage adequate, no critical violations. **Antes de dar por bueno un «pasan»** (propio o heredado del usuario/CI), pásalo por `common-skills:odoo-verify-green`: ¿ejecutó algo, en base limpia, y se ha visto caer en rojo? Un verde no verificado no puntúa como PASS, puntúa como BLOCKED (GOTCHA 8). Source in English (identifiers, comments, docstrings, commits); only user-facing strings exempt. Code changed here → **FAIL**; pre-existing legacy untouched → **WARN**. See `references/shared/coding-standards.md`
   - Gate 3 · Test Readiness — reproducible env, fixtures, externals mocked
   - Gate 4 · User Acceptance — acceptance criteria demonstrated, no S1/S2 bugs; **evidencia del estándar de entrega**: escenario de flujo ejecutable presente salvo nivel trivial (nivel según `references/shared/delivery-thresholds.json`)
   - Gate 5 · Production Readiness — docs updated, rollback plan, no hardcoded secrets; **README de uso/config** si nivel complejo (estándar de entrega)
3. **Score each gate**: PASS / WARN / FAIL / BLOCKED
4. **Generate base report** (summary table + Top 3 priority fixes)
5. **Run ISO 27001 audit unconditionally** — see `references/iso-report-format.md`. Entry point: `odoo-version-api-reference/references/secure-coding-quick.md` (61 lines). Drill into `secure-coding-blocking.md` / `secure-coding-warning.md` per finding. Emit as separate section after base report.

## Output Format (base report)

```
### Gate [N] · [Name] → **[STATUS]**
**Evidence:** [quote code/docs]
**Gaps:** [file:line references]
**Remediation:** [concrete steps]
```

Then summary table (5 gates × status × key finding) + Overall Verdict (worst gate) + Top 3 Priority Fixes.

## Severity Definitions

| Status | Meaning |
|---|---|
| **PASS** | All criteria met |
| **WARN** | Advisory thresholds breached. Resolve next sprint |
| **FAIL** | Hard thresholds breached. Deployment blocked |
| **BLOCKED** | Insufficient information |

## GOTCHAS

1. **Gate 1 (Requirements) needs the original spec** — without it, you can't verify alignment. Ask user for it.
2. **Code coverage ≠ code quality** — 100% coverage with bad assertions is worse than 80% with good ones.
3. **Git diff may miss renamed files** — use `--diff-filter=R` to catch renames that look like delete+create.
4. **Security gate must check OWASP top 10** — SQL injection, XSS, CSRF are common in Odoo custom code.
5. **Production readiness includes rollback plan** — a module without uninstall path fails Gate 5.
6. **ISO audit runs every time** — no opt-in question. Non-code scope → emit with N/A gates (see `references/iso-report-format.md`).
7. **ISO audit is orthogonal to 5 base gates** — clean code can FAIL ISO (e.g., uses `eval()`); well-secured code can fail base gates (e.g., missing tests).
8. **UNA PUERTA QUE NO SE HA VERIFICADO EN ROJO NO ESTÁ VERIFICADA** — y no es teoría: en un proyecto real una puerta de CI daba **verde con 19 infracciones** porque su patrón aceptaba cualquier comentario cercano en vez de uno sobre el hallazgo; otra no veía nada porque miraba un namespace viejo; y una tercera **acusaba a código correcto**. Antes de creerte una comprobación —tuya o heredada— **córrela contra algo que DEBE fallar** (un módulo que sí infringe la regla, un tag inexistente) y comprueba que efectivamente falla. Vale igual para un runner de tests: si no distingue «0 tests ejecutados» de «todo bien», su verde no significa nada. Aplica a cada gate de esta skill.

9. **La puerta del repo gana a la plantilla de la skill** — un PASS aquí no es un PASS de la PR: el `pre-commit` del repo puede rechazar lo que una plantilla de skill mandó poner. Corre la puerta del repo (paso 1b) y, si una guía de Dynapps contradice el linter del cliente, manda el linter: dilo en el informe.

## NEVER / ALWAYS

**NEVER:** Skip the ISO audit. Merge base report and ISO into one section. Mark PASS without evidence — ni fiarte de una comprobación que nunca has visto fallar (GOTCHA 8).
**ALWAYS:** Emit ISO report as a separate section. Reference `references/shared/coding-standards.md` for Gate 2. Load `secure-coding-quick.md` before ISO audit.

## Delegates to

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-verify-green | common | Gate 2 — comprobar que el «los tests pasan» es real (ejecutó, base limpia, verificado en rojo) antes de puntuarlo |
| odoo-version-api-reference | common | ISO 27001 secure-coding gates (12 gates). Entry: `secure-coding-quick.md` |
| indaws-core-auditor | common | Core philosophy compliance for `indaws_*` modules |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| git-pr | developer | Code quality gate before merge |
| dev-assessment | common | Quality validation before finalizing |

## References

- `references/gates.md` — Detailed criteria and thresholds for 5 base gates
- `references/iso-report-format.md` — ISO 27001 report template + scope-aware skipping rules
- `references/shared/coding-standards.md` — English source code rules (Gate 2)
- `odoo-version-api-reference/references/secure-coding-quick.md` — 61-line ISO condensed checklist
