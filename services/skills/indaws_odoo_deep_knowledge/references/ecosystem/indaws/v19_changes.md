<!-- verified: v18.0 baseline @ 2026-06; v19 forward-looking -->
# Indaws Suite — v18 → v19 changes

Funcionalmente, la suite Indaws en v19 es **casi idéntica a v18** a ojos del usuario: mismos módulos, mismas features (triple discount, rappel, composición, picking→invoice, ETL, delivery portal, l10n_es). Lo que cambia es **técnico**: adopción de las nuevas APIs declarativas de Odoo 19, un par de módulos nuevos, un rediseño del descuento triple, y un estado de migración parcial (no toda la suite está aún en v19).

**Fuente:** `indawsodoo/19/` (rama base `19.0`) + docs internos del repo. Verificados presentes a 2026-06: `MIGRATION_GAPS.md`, `ROADMAP.md`, `REVIEW_v19_sql_constraints.md`, `REVIEW_v19_website_native_audit.md`. NO encontrados en el checkout (referencias rotas — antiguas o renombradas): `migration_status.yml`, `SANITIZE_triple_discount.md`, `REVIEW_v18_functional_study.md`.
**Estado a 2026-06 (verificado contra el checkout `19/`):** la suite está **mucho más migrada de lo que afirmaban borradores previos**. `19/` contiene **158 módulos `indaws_*`** (prácticamente el mismo set que los 159 de `18/`) más **7 bundles OCA** (`OCA-account-analytic`, `OCA-account-payment`, `OCA-bank-payment`, `OCA-l10n-spain`, `OCA-mis-builder`, `OCA-queue`, `OCA-server-ux`). La idea de "~72 migrados / ~84 pendientes" es **obsoleta**: hoy la cobertura v19 es casi total. Aun así, sigue valiendo la regla de no asumir — verificar el módulo concreto en `19/` antes de recomendar.

---

## 1. Estado de migración (casi 1:1 con v18)

**Verificado 2026-06:** `18/` tiene 159 módulos; `19/` tiene **158 `indaws_*`** + 7 bundles OCA. La cobertura es prácticamente completa. **Aun así, no asumir** que un módulo concreto está migrado y funcional — verificar contra `19/` antes de recomendar (estar presente en el árbol no garantiza que la migración mecánica/funcional esté terminada).

### Migrados (presentes en `19/`)
- **Capa base completa:** `indaws_base` + los `indaws_base_*` (account, crm, hr, hr_attendance, mass_mailing, mrp, payment, pos, product, project, purchase, sale, stock, survey, website, website_sale).
- **Account:** indaws_account, _edi, _invoice_exchange, _move_merge, _retention, _triple_discount, indaws_invoice_section, _payment_order_modification.
- **Sale, purchase, stock, pos** presentes.
- **Rappel:** `indaws_rappel` (+ sale/purchase) **SÍ está en `19/`** (ver §7 — corrige borradores que lo daban por pendiente).
- **Website/eCommerce:** los 8 `indaws_website_sale*` están presentes en `19/` (incluye `_collect`, `_stock`, `_loyalty`; ver §6 sobre la auditoría de DROP).

### Nuevos en v19 (no existían en v18)
- **`indaws_dataflow`** — módulo BASE nuevo que unifica el motor ETL (ver §3). VERIFICADO: no existe en `18/`.
- **`indaws_hr_timesheet`** — partes de horas / planificación de proyecto corporativa (ver §4). VERIFICADO: en `18/` solo existe `indaws_hr_timesheet_attendance`; los modelos `project.project.hour(.line)`, `project.project.plan.hour`, `project.task.hour` **no existen en `18/`** bajo ningún módulo.

---

## 2. Breaking changes de Odoo 19 adoptados suite-wide

Estos son cambios del CORE de Odoo 19 que la suite ha tenido (o tendrá) que absorber. Relevantes para cualquier módulo custom sobre indaws.

### 2.1 `_sql_constraints` ya NO se aplican (DEPRECADO con warning)
En Odoo 19 el formato legacy `_sql_constraints = [(name, definition, message), ...]` **ya no crea la constraint en BD**: el módulo pierde la integridad UNIQUE/CHECK. **Corrección vs. borrador previo:** NO es 100% silencioso — el core emite una **deprecation warning** (`odoo19/odoo/orm/model_classes.py::add_to_registry`: *"Model attribute '_sql_constraints' is no longer supported, please define models.Constraint on the model."*). La constraint no se aplica, pero queda traza en logs. La clase `Constraint` vive en `odoo19/odoo/orm/table_objects.py::Constraint` (método `apply_to_database()`).

API nueva (declarativa):
```python
from odoo import models

_variant_product_uniq = models.Constraint(
    "UNIQUE (product_tmpl_id, product_variant_id, product_id)",
    "This composition already exists for this variable!",
)
```

**Estado en `19/` (verificado):** los módulos migrados ya están limpios — **0 `_sql_constraints` legacy** en el árbol `indaws_*` (excl. tests/migrations) y **25 instancias `models.Constraint`** adoptadas (no 29; recuento de instancias). Riesgo: módulos que se migren después y conserven el formato viejo perderán constraints (con warning en log). Al actualizar, datos que violen la constraint impedirán crearla → **limpiar duplicados antes**.

### 2.2 `odoo.osv.expression` → `odoo.fields.Domain`
Odoo 19 reemplaza `expression.AND/OR(...)` (listas) por el objeto `Domain` (`from odoo.fields import Domain`; `Domain.AND`, `Domain.OR`, operador `&`/`|`).

**Estado en `19/` (verificado):** migración **parcial y mixta** — **13 ficheros** ya usan `Domain`, **14 ficheros** todavía con `osv.expression`. Al tocar un módulo, comprobar cuál usa antes de copiar patrones. (Core: `Domain` se exporta en `odoo/fields/__init__.py:20`, `Domain`, v19; `osv.expression` sigue presente pero deprecado.)

### 2.3 Seguridad (heredado de core v19) — VERIFICADO
- `res.groups`: `category_id` → `privilege_id` (nuevo modelo `res.groups.privilege`, `odoo19/odoo/addons/base/models/res_groups_privilege.py`; `privilege_id` en `odoo/addons/base/models/res_groups.py:36`, `class ResGroups`). El campo M2M de usuarios es `user_ids` (`odoo/addons/base/models/res_groups.py:17`, `class ResGroups`).
- Verificar grupos/record rules custom de indaws al migrar.

### 2.4 Otros patrones core v19 a vigilar
- **CORRECCIÓN:** compras NO renombra `uom_po_id`→`uom_id` en v19. El campo de la línea de compra ya era **`product_uom_id`** (`odoo19/addons/purchase/models/purchase_order_line.py::product_uom_id`); `uom_po_id` no existe en el addon purchase v19. No hay rename que absorber por aquí.
- OWL `Interaction` reemplaza `publicWidget` en frontend (clase base en `odoo19/addons/web/static/src/public/interaction.js`; website_sale ya extiende `Interaction`). Afecta website (ver §6). Nota: `VariantMixin` aún existe en `website_sale/static/src/js/variant_mixin.js` pero integrado con el flujo Interaction.
- `tax_id`→`tax_ids` confirmado en core v19 (sale/purchase/account order lines, p.ej. `sale/models/sale_order_line.py:162`, `class SaleOrderLine`, `purchase/models/purchase_order_line.py:34`, `class PurchaseOrderLine`).

---

## 3. `indaws_dataflow` — nuevo motor ETL base

Módulo **nuevo en v19** que extrae a una capa base común la maquinaria que en v18 estaba duplicada entre `indaws_data_import` y `indaws_data_export`.

**Modelos** (`indaws_dataflow/models/`):
- `dataflow.scheduler` + `dataflow.scheduler.checkpoint` — scheduler genérico (cron) con checkpoints.
- `dataflow.connection` — conexiones a fuentes externas.
- `dataflow.variable` — mapeo de campos.
- `dataflow.template`, `dataflow.param` + `dataflow.param.mixin` — plantillas y parámetros.
- `dataflow.log` — logging unificado.
- `queue_job.py` — integración async (OCA queue_job).

**Impacto en import/export (v19):**
- `indaws.import.scheduler` pasa a **heredar de `indaws.dataflow.scheduler`** (en v18 era standalone — VERIFICADO: en `18/` `indaws.import.scheduler` solo hace `_inherit = ["mail.thread", "mail.activity.mixin"]`, `indaws_data_import/models/indaws_import_scheduler.py`).
- **MATIZ:** en `18/` **no existe un modelo `indaws.export.scheduler`** standalone — el export se dispara desde el propio `indaws.export` (`indaws_data_export/models/export.py`). La afirmación "`indaws.export.scheduler` igual" es forward-looking respecto al diseño del dataflow base (forward-looking, sin checkout indaws v19 limpio que lo confirme como modelo separado).
- M2M (template↔variable, collector↔variable, scheduler↔connection) ahora cuelgan del dataflow base.

**Gotchas de migración (de `MIGRATION_GAPS.md`):**
- GAP 1 — "`indaws_dataflow` no traía migration scripts": **OBSOLETO/PARCIAL**. El checkout `19/` **ya tiene** `indaws_dataflow/migrations/` con `19.0.1.0.0` y `19.0.2.0.0`. El riesgo de orden de instalación (sus tablas las crea el ORM por `depends`, y los pre-migrate de import/export asumen que existen) sigue siendo el punto válido del gap.
- M2M creadas a mano **sin FK constraints** → rows huérfanas al borrar variables. (GAP 2)
- **Records de scheduler no migrados**: en v18 `indaws.import.scheduler` es standalone; en v19 hereda de dataflow. Los registros configurados se pierden si no se crean los `indaws_dataflow_scheduler` correspondientes. (GAP 3)
- Crons v18 llaman `model.browse(X).execute()`; v19 cambia la firma → actualizar el code de `ir.cron`. (GAP 4)

**Salud del código:** **CORRECCIÓN de cifras (medidas sobre `18/`):** el god-class de import es `indaws_import.py` (no `import_record.py`) con **~1600 líneas** (no 1976); `export.py` **~745 líneas** (no 818). Sigue siendo deuda técnica conocida (roadmap apunta a <500 líneas/fichero). Las cifras exactas en `19/` pueden diferir tras refactor (forward-looking, sin medir sobre indaws v19).

---

## 4. `indaws_hr_timesheet` — nuevo (partes de horas / horas de proyecto)

Módulo **nuevo en v19**. Depende solo de `hr_timesheet` + `indaws_base`. Es el sistema corporativo de planificación y partes de horas de proyecto.

**Modelos:**
- `project.project.hour` + `project.project.hour.line` — horas reales por proyecto.
- `project.project.plan.hour` + `project.project.plan.hour.line` — horas planificadas.
- `project.task.hour` — horas por tarea.
- Extensiones: `project.project`, `project.task`, `account.analytic.line`, `hr.employee`, `hr.employee.public`, `hr.department`.

> Nota operativa (de uso interno Indaws): `project.project.hour.line` **no tiene `date_from`/`date_to`**, usa `date`. Las horas se manejan en formato HH:MM, no decimal.

---

## 5. Rediseño del descuento triple (v19) — "sanitización"

Cambio funcional real (aunque transparente para el usuario final). Objetivo: que el usuario vea **`discount1`, `discount2`, `discount3` físicos** en ventas, compras y factura, y que el `discount` oficial de Odoo quede **oculto** como **agregado calculado** (uso interno: precio de línea, totales, impuestos, factura, informes legales).

**Estado de partida (v18) — VERIFICADO contra `18/`:**
- **Compras/Cuenta:** ya correctos — `discount` = computed/store agregado; `discount1/2/3` visibles. Mixins confirmados: `triple.discount.mixin` (`indaws_account_triple_discount/models/triple_discount_mixin.py:9`, `TripleDiscountMixin._name`, canónico) y `purchase.triple.discount.mixin` (`indaws_purchase_triple_discount/models/purchase_triple_discount_mixin.py:9`, duplicado).
- **Ventas:** el arrastre — NO tenía `discount1`; usaba el `discount` base (de tarifa) como primer descuento + `discount2/3`, agregando con un context-manager transitorio. Confirmado: `indaws_sale_triple_discount/models/sale_order_line.py` define solo `discount2/discount3/discounting_type`, con `_aggregated_discount()` (context-manager) y override de `_compute_amount`/`_convert_to_tax_base_line_dict`.
- **`discounting_type`** (additive/multiplicative) es extensión SOLO de ventas (compras solo multiplicative) — confirmado: existe en sale, ausente en purchase.

**Diseño v19 (ventas se alinea a compras/cuenta):** *(forward-looking salvo donde se indique; el `19/` actual ya muestra el shim discount→discount1 en `indaws_sale_triple_discount/models/sale_order_line.py:90-96`, `SaleOrderLine._prepare_invoice_line`)*
1. `sale.order.line` adopta `discount1` (input visible); `discount` pasa a `compute+store+readonly=False+precompute` = agregado, **oculto** en vistas.
2. El `_compute_discount` base (tarifa) se **relocaliza** para escribir en `discount1`; el mixin recalcula `discount`.
3. **Shim create/write** en el mixin: cualquier `{'discount': X}` entrante (loyalty reward lines, wizard de `indaws_sale_margin`, configurador de `indaws_sale_product_composition`) se redirige a `discount1` automáticamente → **cero cambios en código core de Odoo**.
4. `discounting_type` (additive/multiplicative) se conserva como extensión de ventas (compras solo multiplicative).
5. Se elimina el hack `_aggregated_discount` / override de `_compute_amount` / `_convert_to_tax_base_line_dict`: como `discount` ya es el agregado, el cálculo base nativo lo usa directo.
6. `_prepare_invoice_line` pasa `discount1/2/3` a la factura.
7. **Migración de datos:** `discount` (v18) → `discount1` (v19).

> Gotcha de coherencia (commit `5a2bb74`): en factura el agregado es **aditivo**; revisar mensajes/visualización si se compara con el multiplicative de ventas/compras.

---

## 6. Website / eCommerce — bajo auditoría (candidatos a DROP)

Auditoría feature-a-feature de los `indaws_website_sale*` contra `website_sale` nativo de Odoo 19 (`REVIEW_v19_website_native_audit.md`, presente en el repo). **Nota:** los 8 módulos `indaws_website_sale*` (incl. `_collect`, `_stock`, `_loyalty`) **ya están presentes en `19/`** — la tabla de "DROP" de abajo es una decisión de producto abierta, no un estado actual. *(forward-looking en cuanto a qué se eliminará finalmente.)*

**Veredicto:** ninguna feature es 100% nativa en v19 → ningún módulo se elimina por reemplazo directo. La **palanca** es el concepto custom `website_uom_id` (UoM de venta web ≠ UoM base):

| Módulo | Acción v19 |
|--------|-----------|
| `indaws_website_sale` (core) | MIGRAR (alto) — gate de todo el bloque |
| `indaws_website_sale_stock` | **DROP** si se descarta `website_uom` |
| `indaws_website_sale_collect` | **DROP** si se descarta `website_uom` (v19 trae `website_sale_collect` nativo) |
| `indaws_website_sale_restricted_qty` | ADAPTAR (medio) |
| `indaws_website_sale_hide_price_partner` | MIGRAR (alto) |
| `indaws_website_sale_partner_categ_filter` | MIGRAR (casi mecánico) |
| `indaws_website_sale_product_brand` | MIGRAR (firmas rotas) |

**Cambios técnicos frontend (deltas core v19 CONFIRMADOS):** reescribir como `Interaction` (`website_sale/static/src/interactions/`, base `web/static/src/public/interaction.js`). El RPC de variantes pasa por la ruta `/website_sale/get_combination_info` (existe, acepta `uom_id`, `website_sale/controllers/variant.py`). `_verify_updated_quantity` incluye `uom_id` en su firma (`website_sale/models/sale_order.py:492`, `SaleOrder._prepare_order_line_values`).

---

## 7. Rappel — migrado, sin la pérdida de feature que se temía

**CORRECCIÓN MAYOR (verificado 2026-06):** `indaws_rappel` (y sale/purchase) **SÍ está en `19/`** — no "sigue en v18". Y el revamp **NO eliminó `per_order`/`target_amount`/`_compute_per_order`**: siguen vivos en `19/indaws_rappel/models/rappel.py`:
- `rappel_type` incluye `per_order` ("Per-Order Volume") junto a `fixed`, `variable`, `single_target` (línea 30 ss).
- `target_amount` (Float) presente (línea 74).
- `_compute_per_order(self, order_amount)` presente (línea 260), con su constraint `_check_per_order_configuration` (línea 240).

Por tanto, el "rappel por pedido" (descuento inline como línea negativa cuando el importe supera umbral, `is_rappel_line`/`source_rappel_id`, settlement `state='invoiced_inline'` — todo **confirmado en `18/`**) **se ha conservado en v19**, no se ha descartado. Cualquier borrador previo que dijera "rappel_type queda fixed/variable/formula" o "per_order eliminado" es **incorrecto** frente al checkout actual.

Resto del motor rappel (períodos, `_get_period_volume`, settlements → nota de crédito) también se conserva.

---

## 8. Gotchas v19 Indaws (resumen accionable)

- **IDWG-V19-1** — Constraints: cualquier modelo indaws migrado que conserve `_sql_constraints` pierde UNIQUE/CHECK (con **deprecation warning** en log, no totalmente en silencio). Convertir a `models.Constraint`. Antes de actualizar, limpiar datos duplicados o el ALTER fallará.
- **IDWG-V19-2** — `Domain` vs `expression`: estado mixto en `19/`. No copiar a ciegas; comprobar qué usa el módulo destino.
- **IDWG-V19-3** — `indaws_dataflow` debe instalar ANTES que import/export; sus tablas las crea el ORM, pero los pre-migrate de import/export asumen que existen. Verificar orden.
- **IDWG-V19-4** — Schedulers import/export: los **records** v18 no se migran solos al nuevo base `dataflow.scheduler`. Migrar registros explícitamente o se pierden.
- **IDWG-V19-5** — Triple discount ventas: tras migrar, el valor histórico vive en `discount1`; `discount` es agregado oculto. Integraciones que leían `sale.order.line.discount` como "el descuento" ahora leen el agregado, no el primer tramo.
- **IDWG-V19-6** — No asumir que un módulo está migrado y FUNCIONAL solo porque existe en `19/`. La cobertura es casi total (158 indaws en `19/`), pero estar en el árbol ≠ migración mecánica/funcional terminada. **Rappel y los 8 website_sale SÍ están en `19/`** (corrige borradores que los daban por pendientes).
- **IDWG-V19-7** — `indaws_website_sale_collect`/`_stock` (presentes en `19/`) son candidatos a DROP si se descarta `website_uom_id` — decisión de producto abierta, no hecho consumado. No construir encima sin confirmarla.

---

## 9. Cross-references

- Catálogo de módulos (v18 canónico) → [[catalog]].
- Gotchas generales indaws → [[gotchas]].
- Cómo la suite parchea el core (overrides create/write/compute, triple-discount sale≠account, section.line.mixin, ETL monkeypatch) → [[integrations]].
- HR/timesheet indaws → [[../../functional/hr/indaws]].
- Analytic indaws → [[../../cross-module/analytic/indaws]].
- Para detalle vivo de qué está migrado: inspeccionar `indawsodoo/19/` directamente (el `migration_status.yml` citado en borradores previos **no existe** en el checkout actual). Para gaps de import/export ver `indawsodoo/19/MIGRATION_GAPS.md`.
