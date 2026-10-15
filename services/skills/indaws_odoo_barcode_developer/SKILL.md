---
name: indaws-odoo-barcode-developer
version: 0.1.22
effort: high
model: opus
description: >-
  Extiende las apps de Barcode de Odoo 18 Enterprise: el client action
  stock_barcode (recepciones/entregas/transferencias e inventario) y el Shop
  Floor de fabricación (mrp_workorder / mrp_display). Arquitectura verificada
  contra fuente (client action OWL fullscreen + modelo JS NO reactivo sobre
  EventBus + LazyBarcodeCache + contrato RPC batch), cookbook de recetas de
  extensión confirmadas (botones de línea, pasos de escaneo, campos custom,
  comandos de barcode, diálogos, field widgets, nuevos modelos JS, test_types de
  calidad) y los gotchas que rompen extensiones en silencio. NOT componentes OWL
  backend genéricos (→ odoo-owl-component-builder); NOT POS (→ odoo-pos-developer).
  Triggers ES/EN: "extender stock_barcode", "escáner de barcode Odoo", "añadir
  campo al barcode", "botón en la línea de barcode", "shop floor mrp",
  "mrp_display", "control de calidad en tablet", "parsing GS1 Odoo",
  "extender barcode.rule / nomenclatura", "customize barcode app",
  "add button to barcode line".
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep]
---
<!-- verified: v18.0 @ 2026-07 -->

# Odoo Barcode App Developer

> Convenciones de código y conexión a datos: ver `references/shared/coding-standards.md` y `references/shared/odoo-connection.md`.

Extiende las **dos apps OWL de barcode de Odoo 18 Enterprise** con garantías, sin
romper la reconciliación de reservas/lotes ni la carga de datos:

- **`stock_barcode`** — client action fullscreen para recepciones, entregas,
  transferencias internas e **inventario** (mismo componente `MainComponent`,
  distinto `res_model`).
- **Shop Floor** (`mrp_workorder` → `mrp_display`) — app OWL **separada** para
  ejecutar órdenes de fabricación y controles de calidad en tablet.

**Cobertura por versión (ambas verificadas contra fuente, adversarial):**
- **v18** → referencias **sin sufijo** (`NN-*.md`, `gotchas.md`); **v19** → **`.v19.md`**. 13 ficheros por versión, en pares simétricos (verificado 2026-08-07).
- Fuente EE: **se resuelve, no se asume** (`$ODOO_ENTERPRISE_SRC/{major}`, ver `references/shared/odoo-source.md`); aquí había `/opt/odoo_enterprise/18` a pelo.

**Primer paso SIEMPRE: confirma la versión objetivo del proyecto** y carga el set
correcto. NO mezcles: entre v18 y v19 hay ~100% de churn de ficheros y breaking
changes silenciosos (fórmula UoM invertida, renombres de modelo). Los `file:line`
son pistas que driftan; confirma contra la fuente del proyecto antes de emitir código.
El puente v18→v19 para migrar extensiones está en `references/v18-to-v19-delta.md`.

## Modelo mental (léelo antes de tocar nada)

1. **No son vistas Odoo.** Son client actions OWL con **estado propio en cliente**.
2. **El modelo de estado NO es reactivo.** `BarcodeModel extends EventBus`
   (`stock_barcode/static/src/models/barcode_model.js` — **`:14` en v18, `:12` en v19**;
   verificado 2026-08-07: la clase y la herencia de `EventBus` son idénticas, solo se desplaza la
   línea). Se expone como `env.model` (`useSubEnv`, `stock_barcode/static/src/components/main.js:95`, `MainComponent.setup`).
   Toda mutación debe terminar en `this.trigger('update')` o **la UI no refresca**.
3. **El estado replica el ORM vía `LazyBarcodeCache`.** El servidor manda un blob
   batch (`/stock_barcode/get_barcode_data`) que puebla la cache; el cliente
   reconcilia contra `stock.move.line`, reservas, lotes y paquetes.
4. **El contrato real que overrideas con Python** son los métodos
   `_get_stock_barcode_data` / `_get_fields_stock_barcode` / `_get_groups_data`,
   no el JS.
5. **Extender = casi siempre `patch()` a internals JS** → impuesto de migración
   recurrente. Prefiere seams declarativos (`t-inherit` QWeb, overrides Python)
   cuando existan.

Detalle completo del arranque, árbol de componentes y flujo de un escaneo
end-to-end en **`references/00-architecture-overview.md`**.

## Índice de referencias (progressive disclosure)

Carga SOLO la que necesites, **en la variante de tu versión objetivo**. Las dos rutas
van escritas enteras a propósito: el nombre a secas no se resuelve solo.

| v18 · v19 | Cuándo cargarla |
|---|---|
| `references/00-architecture-overview.md` · `references/00-architecture-overview.v19.md` | Antes de cualquier extensión: modelo mental completo, flujo de escaneo. |
| `references/01-bootstrap-owl-tree.md` · `references/01-bootstrap-owl-tree.v19.md` | Registro de client actions, manifest/assets, main_menu → action, árbol OWL. |
| `references/02-state-engine.md` · `references/02-state-engine.v19.md` | `BarcodeModel` base: ciclo de vida, `processBarcode`, eventos, `save()`, patrón de override. |
| `references/03-picking-model.md` · `references/03-picking-model.v19.md` | Transferencias: moves/move.lines, reservas, backorders, lotes, paquetes, `splitLine`. |
| `references/04-inventory-model.md` · `references/04-inventory-model.v19.md` | Ajuste de inventario: `stock.quant`, conteo, aplicar cantidades (v19: `ConfirmQuantDialog`). |
| `references/05-cache-gs1-parsing.md` · `references/05-cache-gs1-parsing.v19.md` | `LazyBarcodeCache`, resolución barcode→registro, nomenclatura/GS1, `BarcodeParser`. |
| `references/06-ui-components.md` · `references/06-ui-components.v19.md` | `line`/`grouped_line`/`package_line`/diálogos, binding a `env.model`, dónde insertar UI. |
| `references/07-server-rpc-contract.md` · `references/07-server-rpc-contract.v19.md` | Superficie RPC cliente→servidor: qué métodos Python overridear. |
| `references/08-shopfloor-mrp.md` · `references/08-shopfloor-mrp.v19.md` | Shop Floor: arquitectura, `mrp_display_record`, quality checks, validación batch. |
| `references/extension-cookbook.md` · `references/extension-cookbook.v19.md` | Recetas verificadas paso a paso (botón, campo, comando, diálogo, widget, test_type…). |
| `references/rpc-server-contract.md` · `references/rpc-server-contract.v19.md` | Tabla consolidada del contrato RPC de ambas apps. |
| `references/gotchas.md` · `references/gotchas.v19.md` | **Los fallos que rompen en silencio.** Léelos antes de dar por buena una extensión. |

Version-neutral (compartidas):

| Referencia | Cuándo cargarla |
|---|---|
| `references/migration-deltas.md` | Deltas v17→v18→v19 e impuesto de migración al parchear internals. |
| `references/v18-to-v19-delta.md` | **Migrar una extensión v18→v19**: qué recetas/anclas v18 rompen y su equivalente v19. |

## GOTCHAS (los que rompen en silencio — v18 y v19; anclas file:line en `references/gotchas[.v19].md`)

1. **El modelo JS NO es reactivo** (`BarcodeModel extends EventBus`). Toda mutación debe cerrar con `this.trigger('update')` o la UI no refresca, sin error. (Se mantiene en v19.)
2. **Campo custom = TRES puntos o crashea/diff fantasma**: `_getFieldToWrite` (diff) + `_createCommandVals` (create) + backend `_get_fields_stock_barcode` (carga).
3. **`BarcodePickingModel.splitLine` es el punto más frágil**: `reserved_uom_qty` es virtual cliente-only; repartir mal la reserva → backorders fantasma. (v19: la fórmula de conversión UoM se **invirtió** — no portes la de v18.)
4. **Un `patch` a `LineComponent.prototype` se propaga** a Grouped/Package; sus props NO son iguales.
5. **v19 mueve/renombra**: `manual_barcode` al core `barcodes`; `product.packaging→product.uom`; `stock.quant.package→stock.package`; `registry.add` sin `{force:true}` lanza error (no last-write-wins).
6. **`file:line` = pista, no contrato**: driftan entre revisiones. Casi toda extensión útil es patch a internals JS = impuesto de migración. Prefiere seams declarativos (`t-inherit`, overrides `_get_*` Python).

## Workflow de extensión — [opus / principal]

> Skill de **conocimiento/referencia**: todos los pasos los ejecuta el **principal (opus)** cargando referencias bajo demanda. No orquesta subagentes (sonnet/opus) — sería un anti-patrón para un flujo lineal de consulta.

1. **Clasifica la tarea** contra la conversación de complejidad: paso/validación,
   campo, integración externa, comando de barcode, UI/UX, o flujo nuevo.
2. **Decide el seam.** Server-side declarativo (override de `_get_*`) > `t-inherit`
   QWeb > `patch()` a internals JS. Cuanto más arriba, menos impuesto de migración.
3. **Flujo nuevo que NO encaja en picking/quant → client action propia aparte**
   reutilizando primitivas (scanning, nomenclatura), no dobles los internals.
4. **Carga la(s) referencia(s)** del índice + SIEMPRE `references/gotchas.md`.
5. **Verifica contra la fuente del proyecto** (versión objetivo real) los
   `file:line` y firmas antes de escribir.
6. **Comprueba los TRES puntos** al añadir campos (diff + create + carga backend,
   gotcha 2) y cierra toda mutación con `trigger('update')` (gotcha 1).

## Reglas de oro

- **No rehagas el motor.** Reimplementar `stock_barcode`/Shop Floor es rehacer un
  producto Enterprise maduro y renunciar a sus updates. Extiende.
- **`file:line` = pista, no contrato.** Confirma contra fuente; presupuesta
  ~20-40% de impuesto de migración por versión sobre lo que parchees.
- **Honestidad:** si un punto de extensión no lo confirmas leyendo el código, dilo.
  Esta KB nació de una verificación adversarial que refutó/corrigió >50 afirmaciones.

**Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment
syntax) ends with `Made with odoo-barcode-developer · v<version del frontmatter de esta skill>`, wrapped in the file-type
comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

- `odoo-module-creator` (BASE, solo si el módulo no existe) — scaffold del esqueleto
  (manifest, seguridad, README, icono, tests). Casi siempre extiendes un módulo que ya
  existe → no hace falta. Cuando sí: pásale `depends: ['stock_barcode']` (o
  `['mrp_workorder']` para Shop Floor). Esta skill sigue de orquestador; contrato en
  `references/shared/base-domain-delegation.md`.
- `odoo-owl-component-builder` — mecánica genérica de field widgets / componentes OWL backend.
- `odoo-data-files` — datos XML de `barcode.rule` / nomenclatura GS1.

## Consumed by

- **odoo-module-creator** — cuando un módulo nuevo extiende barcode/Shop Floor (`depends`
  incluye `stock_barcode`/`mrp_workorder`), enruta aquí la parte de dominio.
- Candidato: `dev-assessment` (common) al evaluar peticiones de funcionalidad sobre
  stock_barcode / Shop Floor.
