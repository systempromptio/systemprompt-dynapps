---
name: indaws-odoo-accounting-developer
version: 0.1.16
effort: high
model: opus
description: >-
  Desarrollo contable en Odoo (v18/v19, CE+EE): flujos account.move/account.move.line
  (posting, _post, sync líneas dinámicas, line_ids vs invoice_line_ids), motor de
  impuestos account.tax (compute_all, repartos, price_include, redondeo, tax_totals),
  widget de conciliación bancaria (kanban OWL EE), motor de informes contables
  account.report (_report_custom_engine_*, EE), y EDI/e-factura (account_edi + UBL/CII
  + contexto España). NOT scaffolding de módulo (→ odoo-module-creator), NOT el PDF de
  factura QWeb (→ odoo-qweb-report-builder).
  Triggers: "account.move", "asiento contable", "override _post", "compute_all",
  "impuesto Odoo", "repartición de impuesto", "price_include", "tax_totals",
  "conciliación bancaria", "bank rec widget", "account.report", "informe contable
  personalizado", "_report_custom_engine", "account_edi", "e-factura", "TicketBAI",
  "Veri*factu", "Facturae", "UBL", "extender contabilidad Odoo".
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep, AskUserQuestion, EnterPlanMode]
---

# Odoo Accounting Developer

> Convenciones de código: `references/shared/coding-standards.md`.
> Fuente Odoo a verificar: resolver vía `references/shared/odoo-source.md`
> (NO asumir `$ODOO_SRC`). Citas `fichero::símbolo`: el **símbolo manda sobre la
> línea** (anclado a snapshot v18/v19; puede desplazarse al parchear la fuente).
> Reverificado contra fuente 2026-07.

Capa de **dominio contable** sobre Odoo v18/v19 (CE+EE). Cubre las cinco superficies
que ningún otro skill toca: el flujo de `account.move` y sus líneas, el motor de
impuestos `account.tax`, el widget OWL de conciliación bancaria (EE), el motor de
informes `account.report` (EE) y el framework EDI/e-factura. Casi todo se hace
**extendiendo/parcheando** (`super()`, `patch()`, `t-inherit`, herencia XML), no
reescribiendo piezas.

## Base: `odoo-module-creator` (NO reimplementar scaffolding)

Capa de DOMINIO sobre `odoo-module-creator` — contrato completo en
`references/shared/base-domain-delegation.md`. Al invocarse, este skill orquesta y
**delega el esqueleto** (manifest, seguridad ACL+ir.rules, multi-company, README, icono,
provenance, gate anti-laxitud, tests, + los subagentes module-scanner/version-api-reference)
a la base. Aquí vive SOLO el **dominio**: overrides de `account.move._post` /
`account.tax.compute_all`, engines de `account.report`, parches del widget de conciliación,
hooks de generación EDI y las gotchas contables. La mayoría de tareas **extienden un módulo
que ya existe** → no hay scaffolding → la base no se invoca.

## Workflow

1. **Confirmar versión (v18 vs v19).** Es DECISIVO: v19 parte el ORM en `odoo/orm/`
   (usar `Domain` en vez de `odoo.osv.expression`), **elimina el TransientModel
   `bank.rec.widget`** (la conciliación pasa a `account.bank.statement.line` + servicio
   OWL) y migra `_sql_constraints` a `models.Constraint`. La API núcleo de account
   (`_post`, `compute_all`, engines) NO se renombra. Antes de tocar nada consulta
   `references/version-diffs.md`.
2. **Localizar la superficie.** ¿Es asiento/impuesto (Python `account.move`/`account.tax`),
   conciliación (OWL EE), informe (`account.report` EE) o EDI/e-factura?
3. **Asientos e impuestos → `references/move-and-tax-engine.md`.** El eje crítico:
   `line_ids` vs `invoice_line_ids` y la sincronización de líneas dinámicas (lo que más
   se rompe), cómo overridear `_post` sin romper secuencia/lock, y la firma real de
   `compute_all` + repartos + `price_include` + redondeo + widget `tax_totals`.
4. **Conciliación bancaria → `references/reconciliation-widget.md`** (EE, `account_accountant`):
   NO es un `ir.actions.client` — es un `act_window` kanban con `js_class="bank_rec_widget_kanban"`.
   El punto de extensión es `patch(BankRecKanbanController.prototype, {...})`; en v19 el
   modelo transitorio `bank.rec.widget` desaparece (lógica en `account.bank.statement.line`).
5. **Informes contables → `references/report-engine.md`** (EE, `account_reports`): el
   quarteto `account.report`/line/column/expression, escribir un `_report_custom_engine_*`
   (firma + shape de retorno), y añadir un informe o una línea.
6. **EDI/e-factura → `references/edi-einvoice.md`**: mapa honesto v18/v19 (framework legacy
   `account_edi` vs builders UBL/CII), dónde adjunta el XML, cómo afloran errores, y el
   contexto España (solo lo verificable en fuente).
7. **Extender, no reescribir.** `super()` en Python, `patch(Clase.prototype, {...})` +
   `t-inherit` en OWL. Si el módulo NO existe: [delega] el esqueleto en
   `odoo-module-creator`. Tests → `odoo-test-generator`. Gotchas funcionales de negocio →
   `odoo-deep-knowledge` (dominio account).

## Reference Files

| File | Content |
|------|---------|
| [move-and-tax-engine.md](references/move-and-tax-engine.md) | `account.move` posting/estados, `line_ids` vs `invoice_line_ids`, sync de líneas dinámicas, override de `_post`; `account.tax.compute_all` firma+retorno, repartos, `price_include`, redondeo, `tax_totals` |
| [reconciliation-widget.md](references/reconciliation-widget.md) | Kanban OWL de conciliación (EE, `act_window` + `js_class`), controller `BankRecKanbanController`, receta de extensión (`patch`), desaparición de `bank.rec.widget` en v19 |
| [report-engine.md](references/report-engine.md) | `account.report`/line/column/expression, engines, `_report_custom_engine_*`, añadir informe/línea, client action (EE) |
| [edi-einvoice.md](references/edi-einvoice.md) | `account_edi` (legacy) vs builders UBL/CII, adjuntado del XML/errores, contexto España (TicketBAI/Veri*factu/Facturae), hooks de override |
| [version-diffs.md](references/version-diffs.md) | Diffs reales v18→v19 en contabilidad (ORM, move/tax, widget, report engine, EDI) |
| [gotchas.md](references/gotchas.md) | Fallos silenciosos anclados a `fichero::símbolo` |

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Scaffold base module | — | **DELEGATE → `odoo-module-creator`** (solo si el módulo no existe) |
| Diseñar + generar código dominio | **opus** | Overrides account/tax, engines, parches de widget |
| Verificar contra fuente | sonnet/opus | Firmas y símbolos exactos v18/v19 (símbolo manda) |

## Gotchas (Top — ver `references/gotchas.md` para la lista completa)

1. **`line_ids` es el asiento COMPLETO; `invoice_line_ids` es solo el subconjunto editable.**
   Las líneas de impuesto y de vencimiento son *dynamic lines*: las materializa
   `account.move._sync_dynamic_lines`, un **`@contextmanager` que envuelve las escrituras**, no un método que se llame (`account/models/account_move.py:3260`, `class AccountMove` v18 / `:3746` v19). Usa `tax_totals` o `skip_invoice_sync`; detalle en `references/move-and-tax-engine.md`. Pero **NO** impide dejar mal una línea de impuesto: escribir su `balance` a mano **persiste** hasta que algo re-sincroniza (`references/gotchas.md` §medidas).
2. **`_post(soft=True)` es el default** (futuros quedan en `draft`). Al overridear `_post`
   actúa sobre el **RETURN**, no sobre `self`, y llama `super()` (si no, omite secuencia/lock/reversas).
3. **`price_include` ya NO es Boolean store DESDE v18** (no v19): compute+search sin store en `account/models/account_tax.py:127-130`, `class AccountTax` v18, idéntico en v19 `:137-140`; en `odoo17/…:124` era Boolean almacenado. El editable es `price_include_override` Selection (`v18:132`/`v19:142`). v17: `True`→`'tax_included'`.
4. **`compute_all` es un wrapper legacy** sobre el motor nuevo; con `rounding_method=None` usa el de la compañía → round-per-line vs round-globally cambia el total **y la base imponible**. Pásalo explícito. Números medidos, y por qué una llamada suelta a `compute_all` no distingue los métodos: `references/gotchas.md` §medidas.
5. **`tax_totals` (Binary computed) SÍ tiene `inverse`** — escribir solo ajusta importes por delta
   de grupos existentes; cambios de base/estructura se ignoran; NO-factura → `None`.
6. **Conciliación es EE-only y v19 elimina `bank.rec.widget`** (TransientModel): overridearlo
   revienta en v19. La carpeta es `bank_reconciliation` en AMBAS versiones (NO hay rename de dir).
7. **`aml.partner_id` es SIEMPRE el partner COMERCIAL**, no el de la factura (`account_move_line.py::_compute_partner_id`, v18 `:489`/v19 `:535`): con factura a un contacto hijo, validar sobre `move.line_ids` y exportar desde `move.partner_id` lee **dos contactos distintos** → usa `move.commercial_partner_id` en ambos lados.

## Prohibitions

- NUNCA overridear `_post()` sin `super()` ni asumiendo `self` (usa el RETURN; `soft=True` deja fuera los futuros).
- NUNCA editar `line_ids`/líneas de impuesto a mano esperando que persista: lo sobrescribe `_sync_dynamic_lines`.
- NUNCA confiar en escribir `tax_totals` para cambiar la base/estructura: su `inverse` solo ajusta importes de impuesto por delta.
- NUNCA overridear el modelo `bank.rec.widget` para v19 (no existe): la lógica está en `account.bank.statement.line`.
- NUNCA inventar firmas versión-específicas de memoria: verificar contra fuente (símbolo manda).

**Provenance marker (ALWAYS):** cada fichero generado (salvo `.csv`/`.json`) termina con
`Made with odoo-accounting-developer · v<version del frontmatter de esta skill>`, en el comentario del tipo de fichero,
idempotente. Regla completa: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| **odoo-module-creator** | developer | **BASE** — scaffold del esqueleto (manifest, seguridad ACL+ir.rules, multi-company, README, icono, provenance, gate, tests). Corre module-scanner + version-api-reference. | Subagent/skill (solo si el módulo no existe) |
| odoo-orm-modeler | developer | Modelos/campos genéricos del módulo | Subagent (optional) |
| odoo-view-designer | developer | Vistas form/list genéricas | Subagent (optional) |
| odoo-qweb-report-builder | developer | PDF de factura QWeb (frontera reforzada) | Subagent (optional) |
| odoo-owl-component-builder | developer | Componente OWL backend genérico (no el widget concreto) | Subagent (optional) |
| odoo-deep-knowledge | odoo-knowledge | Comportamiento oculto/gotchas funcionales del dominio account | Subagent (optional) |
| odoo-version-api-reference | common | Dudas de API por versión (dato documentado) | Subagent (optional) |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-module-creator | developer | Cuando un módulo nuevo extiende contabilidad (`depends` incluye `account`/`account_accountant`/`account_reports`), enruta el dominio contable aquí |
| odoo-integration-builder | developer | Conectores contables/EDI dentro de una integración |

## Success Criteria

- `_post`/`compute_all` override respeta firma y `super()`; totales correctos.
- `_report_custom_engine_*` devuelve el shape exacto y la columna se rellena.
- Parche del widget de conciliación con registro/dir correcto de la versión objetivo.
- EDI adjunta el XML y los errores afloran (no se tragan). Código en inglés + provenance marker.
