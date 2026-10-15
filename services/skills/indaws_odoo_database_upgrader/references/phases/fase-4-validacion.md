# FASE 4 — Validación + comparison

## Smoke test

`smoke_test` step ejecuta `-u all --stop-after-init` y valida `Registry loaded` + 0 CRITICAL.

Leer `references/playbooks/go_live/smoke_test.md` para el recetario completo.

### Post-`run --apply`: `smoke-validator` (Pattern C)

Tras `indaws-migrate run --apply` (ejecución final), NO leer el log
inline — delega a sub-agente que cruza `audit.sqlite` + log + smoke_rpc:

```python
Agent(model="sonnet", description="Validar run --apply final",
      prompt=<ver agent-tree.md §7 — smoke-validator>)
```

Devuelve `{steps_total, steps_ok, steps_failed, rpc_smoke, ready_for_compare}`.
Si `ready_for_compare: false` → loop a FASE 3.

## Comparison vs referencia

Si existe dump trabajado por el cliente / consultor externo:

```bash
indaws-migrate compare \
  --pipeline migration.yaml \
  --reference /path/to/ref.dump \
  --output docs/comparison_v18_ref.md
```

8 niveles: módulos, campos, vistas, server_actions, contable, permisos, técnicos, transaccional.

### `compare-runner` (Pattern B)

El output bruto del `compare` puede ser de miles de líneas. Procesa via
sub-agente:

```python
Agent(model="sonnet", description="Compare 8-level",
      prompt=<ver agent-tree.md §6 — compare-runner>)
```

Devuelve `{level_1..8, blockers, cosmetic, summary}`. Solo los `blockers`
escalan a usuario.

Leer `references/playbooks/go_live/comparison.md` para el recetario.

## Validación PDFs con cliente

Generar samples reales (factura, presupuesto, albarán, COA) y validar con cliente.

**[STOP] Cliente debe firmar samples** antes del ack:

```bash
indaws-migrate ack pdf_validation --user <cliente> --note "OK con obs. X,Y"
```

Leer `references/playbooks/go_live/pdf_validation.md`.

## Helpers de validación automatizada (post-deploy)

Ver `../helpers-catalog.md` §1 para params completos. Pipeline típico:

1. **`odoo_readiness`** — espera Odoo HTTP 200 tras restart contenedor.
2. **`smoke_rpc`** (auto-inyectado si `plan.odoo_url`) — 2 capas RPC:
   - Read profundo de modelos críticos (40+ por defecto + top-N auto-discover)
   - Create+read+unlink draft (account.move ×4, sale.order, purchase.order)
   Detecta firma de override rota (`apply_payment_days`), campos referenciados
   inexistentes (`x_studio_supplier_default`), computes que petan en escritura.
3. **`playwright_smoke`** — opcional, smoke UI navegando menús sin browser
   visible. Clasificador de noise integrado: descarta `Unexpected token '<'
   "<!doctype"`, `/web/image/*` si `filestore_available=False`, beforeunload,
   marketplace 404. Solo reporta FAILs reales.
4. **`override_linter`** (opcional, pre-deploy también) — AST scan detecta
   overrides Python con firma incompatible entre módulos hermanos.
5. **`filestore_extractor`** — extrae filestore del zip Upgrade Service al
   volumen Docker. Necesario si `playwright_smoke` reporta `/web/image 500`.
6. **`migration_report`** — autogenera `docs/iteration_YYYY-MM-DD.md` con
   métricas (`ir_module_module` counts), audit history, warnings.
   Opcionalmente appendea al README del cliente.

Caso real Tecniman 2026-05-15: el pipeline completo capturó 8 hallazgos no
detectables por `-u all` solo (firma override, campo Studio undefined,
vistas huérfanas tras CORE_REPLACE, filestore ausente).
