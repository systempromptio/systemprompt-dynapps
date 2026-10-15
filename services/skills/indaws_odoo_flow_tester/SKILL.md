---
name: indaws-odoo-flow-tester
version: 3.1.11
effort: high
model: opus
description: >-
  Define, verifica y ejecuta flujos de negocio de Odoo 18.0/19.0 a través de la API (odoo-pilot). 3
  modos: documentación, verificación y ejecución con reversión automática. 190+ flujos atómicos en
  48 secciones, 39 cadenas, 28+ apps. Customization Discovery + Overlay, Test Taxonomy (12 tipos),
  Analysis Engine, Session Persistence. Usar cuando pidan documentar, verificar o ejecutar flujos de
  negocio Odoo o probar cadenas end-to-end.
---

# Odoo Flow Tester v3.1 (API Mode — Odoo 18.0/19.0)

See `references/shared/odoo-connection.md` for connection pattern.

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Paso 0 — Gather | sonnet | Collect requirements |
| Paso 1 — Analysis + overlay | **opus** | Complex flow mapping + risk eval |
| Paso 2 — Verification | sonnet | Mechanical API checks |
| Paso 3 — Execution | **opus** | Error diagnosis, workarounds |
| Paso 4-5 — Revert + report | sonnet | Script execution + template |

## Workflow (9 Pasos)

Full details: `references/workflow-details.md`

### Paso 0 — Recopilación [STOP]
Preguntar: documento de análisis, alcance, modo (A/B/C), conexión Odoo, entorno, versión.

### Paso 0.5 — Customization Discovery [RECOMENDADO]
Detectar personalizaciones (Studio + código custom + configuración). Ver `references/customization-overlay.md`.

### Paso 1 — Análisis y overlay
Leer documento → mapear a catálogo (203 flujos, 39 cadenas) → proponer cadenas → generar overlay.

**El catálogo está repartido en `references/flows/` — 106 ficheros, ninguno de más de ~2K tokens — y se CONSULTA con `scripts/flows.py`.** Antes eran tres monolitos de 310KB (~86K tokens), el mayor gasto de contexto de la suite; hoy abrir uno por error cuesta 2K. `--list`/`--chains` dan el índice (~8K tokens); un ID, `--model` o `--chain` devuelven solo el bloque; `--apartado` da lo que no es un flujo (convenciones, context flags, matrices); `--check` audita. Los `.md` no se tocan: el script los trocea al leer. `flows.py --help`.

### Paso 1.5 — Validación del plan [STOP]
Presentar tabla con flujos + overlay. Esperar aprobación.

### Paso 2 — Verificación (Modos B/C)
Ejecutar `verify_data.py`: permisos, módulos, mail servers, precondiciones, record rules.

### Paso 2.5 — Safe Mode + Snapshot (Modo C) [STOP en production]
`safe_mode.py activate` + `snapshot.py capture`. Ver `references/safety-checklist.md`.

### Paso 3 — Ejecución (Modo C)
Ejecutar flujos con `execute_flow.py`. Prefijo `[TEST-YYYYMMDD-HHMM]`. Ver `references/execution-guide.md`.

### Paso 4 — Reversión (Modo C)
3 tiers: ORM → ORM avanzado → SQL. **El protocolo también se consulta, no se lee**: `scripts/flows.py --revert <modelo>` da lo de ese modelo (~460 tok para `account.payment`) en vez de los 196KB de `references/revert/`.

### Paso 4.5 — Verificación post-revert + restaurar Safe Mode
`snapshot.py compare` + `safe_mode.py restore`.

### Paso 5 — Entregables
Informe HTML/PDF/Markdown con resultados, errores, reversión.

## 3 Modos de Operación

| Modo | Conexión | Qué hace |
|------|----------|----------|
| A — Documentación | No | Análisis + plan de flujos (Markdown) |
| B — Verificación | Sí | A + verifica datos reales en Odoo |
| C — Ejecución | Sí | B + ejecuta flujos + revierte |

### Sub-modo: Migration Integrity (parte de Modo B)

Verifica integridad contable origen→destino. Usa `scripts/verify_migration_integrity.py` (`extract` + `compare`). 21 checks: sumas y saldos, mayor por tercero, conciliación, hash chain l10n_es, IVA, inmovilizado, stock, número de factura. No ejecutar Modo C antes de que este sub-modo pase en verde.

Para verificación **SQL estructural** (v_prev↔v_new): 5 rondas de `references/post-migration-integrity.md`. Principio: toda diferencia se explica o investiga.

Ver catálogo completo en `references/migration-accounting-checks.md`.

## GOTCHAS

1. **Hash fiscal bloquea button_draft(), NO unlink()** — solo audit trail bloquea unlink. Escalar a SQL solo si `posted_before=True` + `check_account_audit_trail=True`
2. **stock.picking done NO se cancela** — usar wizard `stock.return.picking` para devolución inversa
3. **`type='product'` genera ValueError en 18.0+** — usar `is_storable=True`. En **17.0 sigue siendo válido**: `stock` lo añade por `selection_add` (`stock/models/product.py:658-663`, `ProductTemplate`, v17 @3e1233ed1) y ahí `is_storable` no existe. El corte es v18 (`stock/models/product.py:704`, `ProductTemplate.is_storable`, @99f5a33c64)
4. **`hr.expense.sheet` DESAPARECE en 19.0** — el modelo se fusiona en `hr.expense` (0 ocurrencias de `_name`/`_inherit = 'hr.expense.sheet'` en v19 CE y EE; `hr_expense_sheet.py` ya no existe y hay `migrations/2.1/pre-migrate.py`). En **18.0**: `action_sheet_move_create` no existe → usar `action_sheet_move_post`. En **19.0** no existe ninguno de los dos: los métodos viven en `hr.expense` (`action_submit`, `action_approve`, `action_post`, `action_reset`, `action_pay` — `hr_expense/models/hr_expense.py:1128-1205`, `HrExpense.action_submit`, v19)
5. **hr.leave: `action_draft` NO EXISTE en 18.0** — usar `action_reset_confirm`. Estado inicial es `confirm`, no `draft`
6. **Suscripciones: `recurrence_id` NO EXISTE en 18.0** — usar `plan_id` (sale.subscription.plan)
7. **hr.leave: `date_from/date_to` son computed, no se escriben** — usar `request_date_from/request_date_to`. **No es un renombrado de v19**: los dos pares coexisten desde v17 con la misma definición (`hr_holidays/models/hr_leave.py:174-175`, `HolidaysRequest.date_from`, v17 @3e1233ed1; v19 `hr_holidays/models/hr_leave.py:172-173` @d1d161033), y el core lo comenta: *"These dates are computed based on request_date_{to,from} and should therefore never be set directly"*. Aplica a **todas** las versiones, no solo a 19.0
8. **v19: hr.expense.state es computed** — usar `approval_state` o métodos de acción
9. **v19: action_post() puede devolver wizard** — incluir `disable_abnormal_invoice_detection: True` en contexto
10. **Safe mode SIEMPRE debe restaurarse** — incluso si el flujo falló
11. **Un flujo citado puede no existir** — 106 de los 177 IDs que citan las cadenas, y 8 de los 48 `atomic_ref` de `translate_flow.py`, no están definidos (`flows.py --check`, 2026-08-20). Lo que `--chain` marque SIN DEFINIR se resuelve a mano, no se inventa.

## Rules

See `references/rules.md` for full NUNCA (22) + SIEMPRE (10) rules. Key: NUNCA ejecutar sin aprobación (Paso 1.5 STOP) ni crear datos maestros. SIEMPRE prefijo `[TEST-YYYYMMDD-HHMM]` y verificar módulos EDI fiscales antes de flujos contables.

## Scripts (10)

`flows.py` (P1, catálogo) · `verify_data.py` (P2) · `verify_migration_integrity.py` (P2-migración, 21 checks) · `execute_flow.py` (P3) · `snapshot.py` (P2.5/4.5) · `revert_flow.py` (P4, 3 tiers) · `safe_mode.py` (P2.5/4.5) · `validate_flow.py` (P1) · `translate_flow.py` (P1) · `cleanup_manifest.py` (P4)

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | API connection (all modes B/C) | Bash inline (scripts) |
| odoo-screenshot-generator | common | Screenshots for documentation | Bash inline |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-database-upgrader | developer | Verify flows after upgrade (FASE 3) |
| odoo-code-auditor | developer | Audit code issues found during testing |
| odoo-test-generator | developer | Generate automated tests for verified flows |
| odoo-version-api-reference | common | Version-specific flow validation |
| odoo-configurator | consultant | Verify Studio changes work |
| migration-assistant | common | Flow tests reference |

## References

| File | Content |
|------|---------|
| `references/flows/atomic/` | 39 ficheros: 153 flujos atómicos por dominio |
| `references/flows/extended/` | 25 ficheros: 22 modelos extendidos, 59 flujos |
| `references/flows/chains/` | 42 ficheros: 39 cadenas predefinidas |
| `references/MANIFEST.md` | md5 de los seis monolitos originales — el banco verifica que el reparto los reproduce |
| `references/api_compatibility.md` | Compatibilidad API v14.0-v19.0 |
| `references/revert/protocol/` | 21 ficheros: reversión 25+ modelos, 3 tiers |
| `references/customization-overlay.md` | Discovery + overlay (v3.1 core) |
| `references/studio-discovery.md` | Detección Studio/custom/config |
| `references/safety-checklist.md` | 11-point safety checklist (Modo C) |
| `references/execution-guide.md` | API execution, variables, prefixes |
| `references/rules.md` | 22 NUNCA + 10 SIEMPRE rules |
| `references/critical-fixes.md` | Correcciones v2.0/v2.1/v2.2 |
| `references/workflow-details.md` | Full workflow with sub-items |
| `references/app-coverage.md` | 28+ apps coverage table |
| `references/flow-translation.md` | Client doc → YAML translation |
| `references/test-taxonomy.md` | 12 tipos de test |
| `references/analysis-engine.md` | Cross-validation engine |
| `references/session-state.md` | Session persistence format |
| `references/email_safety.md` | Email prevention strategies |
| `references/integration_safety.md` | Integration risk classification |
| `references/revert/sql/` | 34 ficheros: 3 niveles de limpieza, queries SQL |
| `references/precondition_data.md` | Preconditions per flow |
| `references/flow_schema.md` | YAML schema definition |
| `references/revert/sequences/` | 10 ficheros: restauración de secuencias |
| `references/migration-accounting-checks.md` | Migration integrity catalog (sub-modo Modo B) |
| `references/post-migration-integrity.md` | Playbook SQL 5 rondas v_prev↔v_new (upgrade de versión) |
