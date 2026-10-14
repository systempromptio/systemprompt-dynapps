## Alcance real de esta base de conocimiento — SÉ HONESTO

**El detalle a nivel método/línea verificado aquí es Odoo 18 Enterprise** (`stock_barcode` y `mrp_workorder`). Las diferencias con **v19 están verificadas a NIVEL ESTRUCTURA** (árbol de ficheros, altas/bajas, magnitud de cambio en los modelos) — ver sección siguiente, fechada 2026-07 — pero **NO a nivel método/línea**: los `file:line` y firmas de v19 siguen sin verificarse contra fuente. v17 no está verificado en absoluto. Antes de emitir sintaxis para v17/v19 hay que abrir la fuente de esa versión.

## Diferencias v18→v19 (VERIFICADO a nivel estructura, 2026-07)

Comparación directa `diff -rq` de `odoo_enterprise/18` vs `/19`. **Es un cambio masivo, no un delta menor.**

**stock_barcode:**
- **~100% de churn**: TODOS los ficheros de `static/src` difieren (los 3 modelos, `main.js`, todos los componentes, `lazy_barcode_cache.js`, `main_menu`, `barcode_object.js`, widgets, kanban).
- **Modelos medio reescritos**: `barcode_picking_model.js` +1130/−554 líneas (1950→2526, +30%); `barcode_model.js` +524/−341 (1911→2094); `barcode_quant_model.js` +211/−163. Los ~2.900 líneas cambiadas en los 3 modelos hacen inviable un port mecánico.
- **ELIMINADO en v19**: `components/manual_barcode.js`/`.xml` (el componente `ManualBarcodeScanner` que esta KB documenta como punto de extensión). El concepto sigue (referencias `ManualBarcode` en `main.js`, `main_menu.js`, kanban renderer) → **refactorizado/inlineado, no eliminado**: la receta v18 de patchear ese fichero NO aplica en v19.
- **NUEVO en v19**: carpeta `static/src/js/` con `stock_barcode_sml_form.js`; `components/confirm_quant_dialog.js`/`.xml`; `widgets/set_quant_stock_move_line.js`.
- **Manifest**: bundle de tests `web.qunit_suite_tests` → `web.assets_unit_tests` (nueva infra de tests unitarios JS v19; los tests apuntan a `*.test.js`).
- **ESTABLE (verificado)**: `BarcodeModel extends EventBus` sigue en v19 (ahora en `stock_barcode/static/src/models/barcode_model.js:12`, era `:14`) → **el modelo sigue siendo NO reactivo**. El gotcha 1 (cerrar con `trigger('update')`) sigue vigente en v19.

**Shop Floor (mrp_workorder/mrp_display):**
- También **~100% de churn**.
- **ELIMINADO en v19**: `dialog/mrp_quality_check_confirmation_dialog.js`/`.xml`/`.scss` (el diálogo de control de calidad que la KB documenta como extension point EP3), `mrp_record_line/mrp_worksheet.js`/`.xml`, carpeta `mrp_employee_list/`.
- **NUEVO en v19**: `dialog/mrp_employee_dialog.*`, `dialog/mrp_quantity_dialog.*`. El flujo de calidad y el de empleado/cantidad se han reestructurado → las recetas v18 sobre esos diálogos NO aplican tal cual.

**Consecuencia para el que extiende:** cualquier receta v18 que sea patch a un fichero concreto o ancla a `:NNN` hay que **re-verificarla contra v19**; los conceptos y el modelo mental se mantienen, la superficie de código no.

## Lo observable en la estructura v18 (verificado)

- `stock_barcode` es un client action OWL con modelo JS `BarcodeModel extends EventBus` (imperativo, no reactivo). `mrp_display` (Shop Floor) vive en `mrp_workorder`, también client action OWL con validationStack en lote.
- Manifest v18: assets como **tres globs separados** por extensión dentro de `web.assets_backend` (NO brace-expansion `{js,scss,xml}`), con `('remove', **/*.dark.scss)` y gantt movido a `web.assets_backend_lazy` en mrp_workorder.
- Vistas embebidas (form productPage, kanban packagePage) ya usan `<list>`/`<kanban>` en la nomenclatura v18.

## Impuesto de migración recurrente (el hallazgo central)

**El coste de migrar estas apps entre versiones NO es el XML — es que casi todos los puntos de extensión útiles son PARCHES A INTERNALS JS** (`patch(BarcodeModel.prototype,...)`, subclases de `MainComponent`/`MrpDisplayRecord`, overrides de `_processBarcode`/`_validate`/`processValidationStack`). Esos internals:

1. **Drift de números de línea en cada versión.** La propia consolidación de esta KB corrigió decenas de line-refs desfasadas 2-4 líneas dentro de la MISMA v18. Entre majors el drift es total: cualquier ancla `:NNN` es una pista, no un contrato.
2. **Métodos que aparecen/desaparecen o cambian de módulo.** Ejemplo verificado en v18: los `action_*` de quality.check están repartidos entre `mrp_workorder`, `quality_mrp_workorder`, `quality_control(_worksheet)`, `quality_mrp_workorder_worksheet` — una reorganización de módulos entre versiones rompe cualquier override que asuma dónde vive el método.
3. **Firmas engañosas.** El constructor del quant es `constructor(params)` pero reenvía `(resModel, resId, services)` con `...arguments`; el ensamblado de `services` NO es uniforme (dialog solo para picking). Estos detalles no versionados se rompen en silencio al migrar.
4. **Selection y listas hardcodeadas duplicadas.** `barcode.rule.type` (Selection fijo core), `barcodeFieldByModel` (cliente) vs `_get_barcode_field_by_model` (servidor), `gs1LengthsByModel` — cualquiera puede cambiar de forma o contenido entre versiones y romper el matching sin error.

**Conclusión operativa:** al migrar un módulo que extiende estas apps, presupuestar re-verificación completa de cada patch contra la fuente destino, no un port mecánico. Priorizar los seams DECLARATIVOS (t-inherit de sub-templates QWeb, override de `_get_fields_stock_barcode`/`_get_groups_data`/`_get_barcode_config` en Python) que son mucho más estables que los patches JS a internals.

## Deltas v17→v18→v19 del cheatsheet global que AFECTARÍAN a estas apps (NO verificado contra stock_barcode/mrp en esas versiones)

Aplicar con cautela — son reglas generales de Odoo, pendientes de confirmar en las fuentes concretas:
- **v17→v18:** `<tree>`→`<list>` (afecta a las vistas embebidas de los form/list del client action); `<chatter/>` tag reemplaza `<div class="oe_chatter">` (relevante: el Chatter se monta en la rama `infoFormView`); `attrs=`/`states=` ya eliminados en v17.
- **v18→v19:** Kanban exige `<t t-name="card">` (afecta la vista kanban de packagePage y el kanban de tipos de operación que lanza el client action); `res.groups` `category_id`→`privilege_id` (afecta los `has_group` de `_get_groups_data`); reestructuración del ORM a `odoo/orm/`.
- **Field widgets (v17+):** registrar objeto descriptor `{component, extractProps,...}` — ya así en v18 (digipad, image_preview). En v19 `extractProps({attrs, options, placeholder}, dynamicInfo)` cambia firma → los widgets embebidos (digipad, set_reserved_qty_button, image_preview) requieren revisión.

## Lo que NO se puede afirmar (tras el diff estructural de 2026-07)

- **A nivel método/línea en v19**: qué nombres de método sobreviven, firmas, y el reparto exacto client/server dentro de los ~2.900 líneas cambiadas de los modelos. La estructura de ficheros está verificada; el interior no.
- Si `get_specific_barcode_data_batch` (ruta muerta en v18) revive en v19.
- Nada verificado en v17.

Resuelto ya (arriba): el modelo SIGUE siendo `EventBus` no-reactivo en v19.

## Veredicto: ¿diff o entrada nueva para v19?

**Ni diff puro ni skill separada → set de referencias v19 dentro de ESTA skill, regenerado con verificación (no derivable de un diff).**

- **Un diff/delta NO sirve como conocimiento primario**: con ~100% de churn y ~2.900 líneas cambiadas solo en los modelos, un doc de delta útil sería más grande y más frágil que la propia KB, y aun así obligaría a re-parchear mentalmente cada cita `:NNN`. Puntos de extensión documentados (ManualBarcodeScanner, quality confirmation dialog) DESAPARECEN → el delta puede señalarlo pero no darte la forma v19 sin leer fuente v19.
- **Una skill nueva NO**: mismo modelo mental, misma capacidad, mismo cookbook conceptual, mismos gotchas arquitectónicos (EventBus no-reactivo confirmado, campo=3 puntos, splitLine frágil, seams declarativos). Duplicaría ~40% y colisionaría por nombre (gotcha 6).
- **SÍ**: set de referencias v19 propio, regenerado con verificación adversarial.

**HECHO (2026-07):** existe el set v19 completo (`NN-*.v19.md`, `gotchas.v19.md`,
`extension-cookbook.v19.md`, `rpc-server-contract.v19.md`), verificado contra
`odoo_enterprise/19` (17 agentes, ~1,4M tokens). `SKILL.md` es version-aware.
El puente para migrar extensiones v18→v19 está en `v18-to-v19-delta.md`.
Confirmó breaking changes silenciosos (fórmula UoM de `reserved_uom_qty` invertida,
`product.packaging→product.uom`, `stock.quant.package→stock.package`,
`manual_barcode` movido al core `barcodes`, `registry.add` sin `{force:true}` peta).
