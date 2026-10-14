# Reporting / BI (transversal) Gotchas — Odoo 18.0 (+ v19)

<!-- verified: v18.0 @ 2026-06 -->

Prefijo **RPTG**. Cubre los cuatro motores de reporting:
- **RPTG1-24**: QWeb → PDF (wkhtmltopdf).
- **RPTG-SQL1-6**: vistas SQL de análisis (`_auto=False`, `sale.report`…).
- **RPTG-BI1-9**: agregación pivot/graph (`read_group`/`aggregator`).
- **RPTG-AR1-4**: motor account.report (EE).

Para el pipeline y la anatomía ver `core.md`. Para integraciones (account_reports, spreadsheet,
dashboards) `integrations.md`. Diffs de versión en `v19_changes.md`.

Fuentes: `odoo/addons/base/models/ir_actions_report.py`, `addons/web/views/report_templates.xml`,
`addons/web/controllers/report.py`, `odoo/addons/base/models/report_paperformat.py`,
`odoo/models.py`, `odoo/fields.py`, `addons/{sale,account,stock}/report/*.py`,
`addons/web/static/src/views/{pivot,graph}/*`.

---

## PARTE A — QWeb → PDF (wkhtmltopdf)

---

## Motor de renderizado: por qué el PDF != el navegador

**RPTG1. wkhtmltopdf usa un WebKit MUY viejo (QtWebKit ~2012) → flexbox/grid/CSS moderno fallan** — El
PDF NO se renderiza con tu Chrome. `_run_wkhtmltopdf` (`odoo/addons/base/models/ir_actions_report.py:502`, `IrActionsReport._run_wkhtmltopdf`) lanza el binario
`wkhtmltopdf`, que embebe un fork de QtWebKit congelado en ~2012. Síntoma: en el preview del
navegador (`/report/html/...`) el layout es perfecto, pero el PDF descarga roto: `display:flex` y
`display:grid` colapsan, `gap`, `position:sticky`, `object-fit`, custom properties CSS (`var(--x)`)
parcial, `flex-wrap`, y muchas unidades modernas se ignoran o se comportan distinto. Impacto: el
diseño que validaste en el navegador NO es representativo del PDF. **Regla: maqueta informes con
tablas Bootstrap (`row`/`col`, `<table>`) y CSS clásico, nunca flex/grid moderno.** Verifica SIEMPRE
descargando el PDF real, no el preview.

**RPTG2. Página en BLANCO con paperformat custom: `page_width`/`page_height` se ignoran si
`format != 'custom'`** — En `_build_wkhtmltopdf_args` las dimensiones custom solo se emiten cuando
`paperformat_id.format == 'custom'` (`odoo/addons/base/models/ir_actions_report.py:316-318`, `IrActionsReport._build_wkhtmltopdf_args`). Si rellenas Page width/height
pero dejas `format='A4'`, se manda `--page-size A4` y tus mm se descartan. Y al revés: si pones
`format='custom'` pero dejas width/height a 0/False, wkhtmltopdf recibe una página de tamaño inválido
→ **PDF en blanco o vacío**. Impacto: el caso clásico de "el PDF sale en blanco con mi formato
personalizado". Fix: `format='custom'` **Y** width+height en mm > 0.

**RPTG3. Header/footer se REPITEN o DESAPARECEN: el truco `subst()` JS** — En un PDF multi-documento,
todos los headers/footers se acumulan en `#minimal_layout_report_headers/footers` y un script JS
(`subst()` en `web/views/report_templates.xml:238-262`, `minimal_layout`) selecciona el correcto por página
usando la query `?webpage=...` que inyecta wkhtmltopdf en `--header-html`. Si el JS no corre a tiempo
(delay bajo, error JS en el header, assets no cargados), TODOS los headers aparecen apilados o ninguno.
Impacto: header de empresa duplicado N veces o footer vacío. Causa típica combinada con RPTG12. Sube
`report.print_delay`.

**RPTG4. Header/footer desaparecen aleatoriamente: assets no cargan a tiempo (issue #2083)** — Comentario
explícito en `odoo/addons/base/models/ir_actions_report.py:850-855`, `IrActionsReport._render_qweb_pdf_prepare_streams` (link al issue en línea 855): "the header/footer of the
reports randomly disappear because the resources files are not loaded in time". wkhtmltopdf hace
requests HTTP separadas para header-html, footer-html y cada body; si el CSS/JS bundle se sirve
troceado (modo debug) puede no llegar antes del timeout. Por eso Odoo **fuerza `debug=False` en el
render PDF** (`additional_context = {'debug': False}` + `data.setdefault("debug", False)`, líneas
856-857). Impacto: si fuerzas debug en el render, los headers desaparecen intermitentemente. NO
intentes renderizar PDF en modo debug "para depurar"; usa `/report/html`. (En v19 el mismo bloque
está en líneas 873-876.)

**RPTG5. `web.minimal_layout` != `web.external_layout` — confundirlos rompe el PDF** — Son capas
distintas (`core.md` §5). `minimal_layout` (`web/views/report_templates.xml:229`, `minimal_layout`) es el wrapper técnico de PDF
(carga assets, base href, script subst) que Odoo aplica internamente vía `_prepare_html`
(`_get_layout()` → `web.minimal_layout`, `odoo/addons/base/models/ir_actions_report.py:279-280`, `IrActionsReport._get_layout`). `external_layout`
(`web/views/report_templates.xml:802`, `external_layout`) es el marco visual de empresa (logo/dirección) que TÚ incluyes con
`<t t-call="web.external_layout">`. Impacto: si en tu template haces `t-call="web.minimal_layout"`
por error, pierdes el marco de empresa; si tocas/heredas `minimal_layout` para meter estilos, rompes
el render de TODOS los informes. Tu CSS de informe va en `web.report_assets_common`, no en el layout.

**RPTG25. Render QWeb es TODO-O-NADA: un `t-out`/`t-field` que peta tumba el PDF ENTERO** — QWeb
no degrada el nodo que falla: lanza `QWebException` (**`QWebError` en v19** — renombrada sin alias,
`odoo/addons/base/models/ir_qweb.py:522`, `QWebException` v18 → `:534` v19, 0 ocurrencias del nombre viejo en CE19/EE19) y aborta el render
completo. Un `1/0` en una
expresión, un método sobre un recordset vacío (`doc.line_ids[0].x` con 0 líneas), un `t-att-src`
con un binario `False` (`image_data_uri(False)` peta), un `t-field` sobre un path inexistente
(KeyError) → **PDF entero en blanco/error**, no solo esa celda. La validación estática del arch NO
protege: el fallo es en RENDER. Impacto: informes configurables/con expresiones de usuario son
frágiles — una fila mala revienta todo el documento. Patrón de defensa: enrutar toda expresión
dinámica por un método envoltorio `try/except → ''` (nunca `eval`/`safe_eval` crudo en el template);
guardar los `t-field widget=image` / `image_data_uri(...)` con un `t-if` del binario. Ver
`odoo-qweb-report-builder/references/qweb-patterns.md §11`.

---

## CSS / assets que no aplican

**RPTG6. CSS no aplica en el PDF: falta `t-call-assets` o lo metiste en el bundle equivocado** — El
informe NO lleva CSS inline; se inyecta vía `t-call-assets` en el `<head>` del layout
(`minimal_layout` carga `report_assets_pdf` + `report_assets_common`, `web/views/report_templates.xml:234-236`, `minimal_layout`).
Si extiendes tu CSS en `web.assets_backend` (el bundle del cliente web) en vez de
`web.report_assets_common`, NUNCA llega al PDF. Impacto: "mi SCSS funciona en pantalla pero el PDF
sale sin estilo". Fix: añade tu fichero al bundle `web.report_assets_common` en el `__manifest__.py`
(como hacen `account`/`event`/`hr_skills`). Verifica el bundle definido en
`web/__manifest__.py:259` (common) y `:320` (pdf).

**RPTG7. Diferencia preview vs PDF por el reset CSS** — El PDF carga un bundle extra
`web.report_assets_pdf` = `reset.min.css` (`web/__manifest__.py:320-322`) que NO se aplica en el
preview HTML estándar (`report_layout` solo carga `report_assets_common`, `web/views/report_templates.xml:17`, `report_layout`).
Impacto: márgenes/paddings por defecto difieren entre `/report/html` y el PDF descargado. El preview
del wizard de factura usa un layout especial (`report_preview_layout`, `web/views/report_templates.xml:39`, `report_preview_layout`) con
hacks de CSS y un comentario explícito: "it is quite complicated to have an almost exact replica of
the pdf report" (`web/views/report_templates.xml:58`, `report_preview_layout`). No esperes pixel-perfect entre preview y PDF.

---

## Paperformat, márgenes, DPI

**RPTG8. `header_spacing` mal calculado → contenido bajo el header o header pisando el body** —
`header_spacing` (default 35 mm, `odoo/addons/base/models/report_paperformat.py:184`, `class report_paperformat`) → `--header-spacing`
(`odoo/addons/base/models/ir_actions_report.py:339-342`, `IrActionsReport._build_wkhtmltopdf_args`). Es el espacio reservado para el header; el body empieza tras él. Si
tu `margin_top` es menor que el header real, el header pisa el contenido; si `header_spacing` es muy
grande, queda una franja en blanco gigante (síntoma cercano a "página casi en blanco"). Regla de oro:
`margin_top` debe ser >= altura del header + `header_spacing`. Impacto: el primer renglón del body
desaparece o se solapa.

**RPTG9. `data-report-*` del `<html>` raíz SOBRESCRIBEN el report.paperformat** — `_prepare_html`
extrae los atributos `data-report-margin-top`, `data-report-header-spacing`, `data-report-dpi`,
`data-report-landscape` del tag raíz y los pone en `specific_paperformat_args`
(`odoo/addons/base/models/ir_actions_report.py:433-438`, `IrActionsReport._prepare_html`), que tienen **prioridad** sobre el record en `_build_wkhtmltopdf_args`
(líneas 320-322, 326-327, 339-340, 301-302). Impacto: cambias el paperformat en Ajustes y "no hace
nada" porque un template (p.ej. `report_layout` setea `data_report_margin_top`) inyecta el valor por
atributo y gana. Busca dónde se setean esas variables QWeb antes de tocar el record.

**RPTG10. DPI bajo → imágenes/logos pixelados o desproporcionados** — `dpi` (default 90, `required=True`,
`odoo/addons/base/models/report_paperformat.py:186`, `report_paperformat.dpi`) → `--dpi` + (si wkhtmltopdf >= 0.12.2) `--zoom 96/dpi`
(`odoo/addons/base/models/ir_actions_report.py:325-337`, `IrActionsReport._build_wkhtmltopdf_args`). A dpi=90 el zoom es 96/90=1.066, lo que reescala TODO el documento,
no solo imágenes. Impacto: subir/bajar dpi cambia el tamaño aparente de fuentes y márgenes, no solo
la nitidez. Para imágenes nítidas usa imágenes a resolución suficiente (~150 dpi del tamaño físico),
no subas el dpi del paperformat sin entender que reescala el layout entero. En Windows con dpi<=95 se
fuerza a 96 (línea 329-331).

**RPTG11. `css_margins` cambia QUIÉN controla los márgenes** — Con `css_margins=True`
(`odoo/addons/base/models/report_paperformat.py:190`, `report_paperformat.css_margins`), `minimal_layout` añade la clase `o_css_margins` al body
(`web/views/report_templates.xml:270`, `minimal_layout`) y los márgenes pasan a controlarse por CSS en vez de los flags
`--margin-*` de wkhtmltopdf. Impacto: si activas css_margins y no defines los márgenes en CSS,
quedan a 0 (texto pegado al borde). Si NO lo activas, el CSS de márgenes del body lo ignora
wkhtmltopdf y mandan los flags del paperformat. Saber cuál de las dos vías está activa es clave para
depurar "mis márgenes CSS no hacen efecto".

---

## Depuración

**RPTG12. El truco `?debug` y `/report/html` para depurar** — Para ver qué HTML recibe realmente
wkhtmltopdf, abre la URL del controlador `report_routes` (`web/controllers/report.py:23-40`, `ReportController.report_routes`):
`/report/html/<reportname>/<docids>` (ej. `/report/html/account.report_invoice/42`). Devuelve el HTML
crudo renderizado, inspeccionable con DevTools. **Pero ojo (RPTG4)**: el path PDF fuerza `debug=False`
(`odoo/addons/base/models/ir_actions_report.py:856`, `IrActionsReport._render_qweb_pdf_prepare_streams`), así que el HTML de `/report/html?debug=1` carga assets troceados que el
PDF NO usa — útil para ver estilos, NO para reproducir el bug del PDF 1:1. Para el PDF real usa
`/report/pdf/<reportname>/<docids>`. `/report/check_wkhtmltopdf` (`web/controllers/report.py:153`, `ReportController.check_wkhtmltopdf`) devuelve el estado
del binario.

**RPTG13. wkhtmltopdf returncode 1 NO es error; -11 = falta de memoria** — `_run_wkhtmltopdf` acepta
returncodes **0 y 1** (`if process.returncode not in [0, 1]`, línea 602); 1 son solo warnings (assets
no encontrados, etc.) que se loguean pero el PDF se genera igual. returncode `-11` (SIGSEGV) → UserError
"Memory limit too low or maximum file number of subprocess reached" (rama en líneas 603-609). Impacto:
PDF gigante/con muchas imágenes peta con -11 en servidores con poca RAM o `ulimit` de ficheros bajo; no
es un bug del template. Otros códigos → UserError con los últimos 1000 chars del stderr
(`err[-1000:]`, líneas 610-615).

---

## Paginación

**RPTG14. Saltos de página: `page-break-inside: avoid` SÍ funciona, `break-inside` (moderno) NO** —
Como wkhtmltopdf es WebKit viejo (RPTG1), solo entiende las propiedades CSS2 de paginación:
`page-break-before`, `page-break-after`, `page-break-inside: avoid`. La sintaxis CSS3 moderna
(`break-inside`, `break-before`) se ignora. Impacto: una fila de tabla o un bloque se parte a mitad
entre dos páginas porque usaste la propiedad nueva. Fix: usa las `page-break-*` clásicas.

**RPTG15. El split por registro depende de los PDF outlines (`<h1>`)** — Para informes multi-registro
con `attachment`, Odoo parte el PDF por registro usando los outlines (headings) del PDF
(`odoo/addons/base/models/ir_actions_report.py:918-955`, `IrActionsReport._render_qweb_pdf_prepare_streams`). Necesita 1 outline (h-tag) por registro y heading en la página 1
(`has_top_level_heading`, línea ~947). Impacto: si tu template no tiene un `<h1>`/heading por documento
o el primer doc no empieza en página 1, el split falla y cae al render individual recursivo (lento,
línea 956) o devuelve el PDF entero como un solo stream. Síntoma: adjuntos por registro mezclados o
PDF entero adjunto a un solo registro.

**RPTG16. Tablas enormes se parten en bloques de 500 filas (cambia la paginación)** — body > 4 MiB →
`_split_table(tree, 500)` parte cada `<table>` en tablas de 500 filas (`odoo/addons/base/models/ir_actions_report.py:572-595`, `IrActionsReport._run_wkhtmltopdf`,
`_split_table` def línea 70). Es un workaround al coste exponencial de wkhtmltopdf (1 h para 250k
filas). Impacto: en informes con tablas gigantes, los `thead`/estilos de la tabla original se
re-aplican por bloque y pueden aparecer cortes/repeticiones de cabecera inesperados a cada 500 filas.

---

## Infraestructura

**RPTG17. `workers=1` → el PDF nunca se genera (estado `workers`)** — wkhtmltopdf hace una request HTTP
de vuelta a Odoo para obtener el HTML; con un solo worker hay deadlock. Al arrancar, Odoo marca
`wkhtmltopdf_state='workers'` (`odoo/addons/base/models/ir_actions_report.py:113-115`). Impacto típico en local/dev:
"imprimir PDF se queda colgado o da timeout". Fix: arranca con `--workers=2` (o más). Verifica con
`/report/check_wkhtmltopdf`.

**RPTG18. wkhtmltopdf no instalado → UserError solo en runtime, no al crear el informe** — Si el binario
falta, `wkhtmltopdf_state='install'` y `_render_qweb_pdf_prepare_streams` lanza UserError "Unable to
find Wkhtmltopdf on this system" (`odoo/addons/base/models/ir_actions_report.py:844-849`, `IrActionsReport._render_qweb_pdf_prepare_streams`). Impacto: puedes crear el
`ir.actions.report` y el template sin error; revienta al imprimir. El comentario advierte que llamadas
manuales (plantilla de email que adjunta PDF) saltan el check previo del controlador y pueden petar
silenciosamente en el envío del correo.

**RPTG19. Imágenes/QR vía URL en blanco sin cookie de sesión** — wkhtmltopdf resuelve enlaces internos
(`<img src="/report/barcode/...">`, imágenes adjuntas vía `/web/image/...`) con una **cookie de sesión
temporal** que Odoo le pasa por `--cookie-jar` (`odoo/addons/base/models/ir_actions_report.py:539-558`, `IrActionsReport._run_wkhtmltopdf`). Esto solo ocurre si
hay `request` activo (`if request and request.db`). Impacto: al generar PDFs desde un **cron o test**
(sin request) las imágenes que dependen de auth pueden salir vacías. Usa rutas con `auth='public'`
para esos recursos o incrusta las imágenes como base64.

**RPTG24. "Cannot separate file to save as attachment": faltan `data-oe-model`/`data-oe-id` en el
`article`** — Cuando el informe tiene `attachment` y se renderiza para varios registros, Odoo necesita
repartir el PDF por registro leyendo los atributos del `<div class="article">`. Si el set de ids
mapeados (`html_ids`) no coincide con los pedidos, lanza UserError "Report template ... Cannot separate
file to save as attachment because the report's template does not contain the attributes 'data-oe-model'
and 'data-oe-id' as part of the div with 'article' classname" (`odoo/addons/base/models/ir_actions_report.py:864-870`, `IrActionsReport._render_qweb_pdf_prepare_streams`). Impacto:
un template heredado/custom donde el `<div class="article">` no propaga
`t-att-data-oe-model="..." t-att-data-oe-id="o.id"` rompe SOLO cuando hay `attachment`; sin attachment
imprime bien. Fix: asegúrate de que el div con clase `article` lleva ambos atributos por registro (lo
hace `web.html_container`/`external_layout` estándar; se pierde al rehacerlo a mano).

**RPTG-BI9. Solo 11 métodos de agregación válidos (`READ_GROUP_AGGREGATE`)** — El dict
`READ_GROUP_AGGREGATE` (`odoo/models.py:365-377`, `READ_GROUP_AGGREGATE`) fija las únicas funciones admitidas en `field:func`:
`sum, avg, max, min, bool_and, bool_or, array_agg, recordset, count, count_distinct`. Cualquier otra
(`stddev`, `median`, `percentile`…) → ValueError "Invalid aggregate method" en `_read_group_select`
(`odoo/models.py:2064`, `BaseModel._read_group_select`). Impacto: no puedes pedir percentiles/medianas vía `_read_group`; hay que calcularlos
fuera (Python) o con un report SQL a medida. `array_agg`/`recordset` ordenan por `<table>.id`.

**RPTG20. `attachment_use` cachea el PDF: cambios al template no se ven** — Con `attachment` +
`attachment_use=True`, `_render_qweb_pdf_prepare_streams` reusa el PDF guardado como adjunto en vez de
re-renderizar (`odoo/addons/base/models/ir_actions_report.py:817-832`, `IrActionsReport._render_qweb_pdf_prepare_streams`). Impacto: editas el template/paperformat y el PDF sigue
saliendo igual porque devuelve el adjunto cacheado. Fix: borra el adjunto del registro (o
`report_pdf_no_attachment` en context) para forzar regeneración.

---

## Traducción

**RPTG23. El PDF no se traduce al idioma del cliente: falta `lang` en el contexto del registro** — El
render aplica el idioma **por documento** leyendo el atributo `data-oe-lang` del nodo `article`
(`odoo/addons/base/models/ir_actions_report.py:411-415`, `IrActionsReport._prepare_html`): `IrQweb.with_context(lang=node.get('data-oe-lang'))`. Ese
`data-oe-lang` lo pone el template QWeb del registro (típicamente
`t-att-data-oe-lang="o.partner_id.lang"` o vía `o.with_context(lang=...)`). Impacto: si tu template
custom NO setea el lang del partner (o llamas `_render_qweb_pdf` sin `with_context(lang=...)`), el
PDF sale en el idioma del usuario que imprime, no en el del cliente. Las cadenas estáticas del
template se traducen vía el `lang` del contexto QWeb; los **campos `translate=True`** (nombres de
producto, términos) solo si el registro se lee con ese lang. Fix: asegura el `data-oe-lang`/contexto
con el `partner_id.lang` correcto antes del render. Nota: `print_report_name` es `translate=True`
(`odoo/addons/base/models/ir_actions_report.py:167`, `IrActionsReport.print_report_name`) → el nombre del fichero descargado también depende del lang.

---

## PARTE B — Vistas SQL de análisis (`_auto=False`)

**RPTG-SQL1. Editas el SQL de un report `init()`+CREATE VIEW y "no cambia nada": falta `-u`** — Los
reports que usan `init()` con `CREATE OR REPLACE VIEW` (ej. `report.stock.quantity`,
`stock/report/report_stock_quantity.py:33-46`, `ReportStockQuantity.init`; `fleet.vehicle.cost.report`) **materializan la vista en BD solo
cuando se ejecuta `init()`**, es decir al instalar/actualizar el módulo. Impacto: cambias el string
SQL en el `.py`, reinicias Odoo y la vista PG sigue siendo la vieja → datos/columnas viejas. Fix:
`-u <modulo>` para re-disparar `init()`. (Los reports con `@property _table_query` NO sufren esto:
la subquery se re-evalúa en cada lectura, `odoo/models.py:871-899`, `BaseModel._table_sql`.)

**RPTG-SQL2. "column must appear in the GROUP BY clause": columna del SELECT no agregada ni
agrupada** — En el patrón `sale.report`, toda columna del `_select_*()` que NO esté dentro de una
función de agregación (SUM/AVG/COUNT/MIN/MAX) DEBE estar en `_group_by_*()` (`sale/report/sale_report.py:201-230`, `SaleReport._group_by_sale`).
Impacto: añades un `fields.X` nuevo al report, mapeas su columna en `_select` pero olvidas meterla en
`_group_by` → PG revienta al crear la vista o al primer read_group. Es el error nº1 al extender un
report SQL. Regla: campo nuevo = columna en `_select` **Y** (`AS` agregado) **O** entrada en `_group_by`.

**RPTG-SQL3. Filas duplicadas/desaparecidas porque `id` colisiona** — Un report `_auto=False`
necesita una columna `id` única (PK virtual del ORM). `sale.report` la sintetiza con
`MIN(l.id) AS id` (`sale/report/sale_report.py:100`, `SaleReport._select_sale`). Impacto: si tu GROUP BY agrupa filas que comparten el
mismo `MIN(id)` (raro pero posible al cambiar la granularidad del group by), el ORM ve "el mismo
registro" y pierde filas o las pisa. Síntoma: el pivot suma bien pero la list view muestra menos
líneas de las esperadas. Fix: garantiza unicidad del `id` (incluir en el GROUP BY las columnas que
definen la granularidad de cada fila).

**RPTG-SQL4. Datos stale dentro de la misma transacción: falta `_depends`** — Los reports
`_table_query` no tienen tabla real, así que el ORM no sabe qué campos los afectan; declara
`_depends` para forzar flush antes de leer (`account/report/account_invoice_report.py:56-71`, `class AccountInvoiceReport`, usado en
`_table_sql`, `odoo/models.py:884-899`, `BaseModel._table_sql`). Impacto: en un test o en un flujo que crea una factura y acto
seguido lee `account.invoice.report` en la MISMA transacción, sin el campo en `_depends` la subquery
lee la BD antes de que el ORM haya flusheado el cambio → números viejos. Fix: añade el modelo/campo
real al `_depends` del report.

**RPTG-SQL5. No puedes `write()`/`create()` en un report de análisis** — `_auto=False` + columnas
`readonly=True`: el modelo es de **solo lectura**. Impacto: intentar `self.env['sale.report'].create(...)`
o `write()` falla (no hay tabla donde escribir). Estos modelos solo sirven para search/read_group.
Para "corregir" un dato hay que tocar el registro real (sale.order.line) y dejar que la vista lo
refleje.

**RPTG-SQL6. La currency_table se recalcula en cada lectura (coste oculto)** — `sale.report._from_sale()`
llama `_get_simple_currency_table(self.env.companies)` y la mogrifica EN CADA consulta
(`sale/report/sale_report.py:183-195`, `SaleReport._from_sale`). Impacto: en multi-compañía con muchas monedas, cada read_group del pivot
de ventas reconstruye el JOIN de conversión → más lento de lo que parece. No es bug; es el precio del
mecanismo `_table_query` (siempre fresco) frente a la vista materializada.

---

## PARTE C — Agregación pivot / graph (`read_group`)

**RPTG-BI1. `read_group(lazy=True)` solo agrupa por el PRIMER campo** — Default `lazy=True`
(`odoo/models.py:2806`, `BaseModel.read_group` y `odoo/models.py:2847`, `BaseModel.read_group`): `lazy_groupby = groupby[:1]`. Impacto: llamas
`read_group(domain, ['amount:sum'], ['team_id','user_id'])` esperando el cruce equipo×comercial y
obtienes SOLO grupos por `team_id` (el resto en `__context`). Es el gotcha más común al usar
read_group en código. Fix: `lazy=False` para todos los niveles en una llamada. (La vista pivot lo
gestiona expandiendo nivel a nivel con read_groups sucesivos.)

**RPTG-BI2. La clave de conteo cambia de nombre según `lazy`** — Con `lazy=True` y 1 solo groupby la
clave es `<field>_count` (ej. `team_id_count`); en cualquier otro caso es `__count`
(`odoo/models.py:2855-2856`, `BaseModel.read_group`). Impacto: código que lee `result[0]['__count']` peta con KeyError cuando hay
un único groupby lazy. Fix: usa `_read_group` (backend) que devuelve tuplas sin esta ambigüedad, o
pide explícitamente `__count` en los fields.

**RPTG-BI3. Campo computed sin `store=True` NO es agregable ni agrupable** — `_description_aggregator`
(`odoo/fields.py:930-940`, `Field._description_aggregator`) y `_read_group_select` (`odoo/models.py:2047-2073`, `BaseModel._read_group_select`) requieren que el campo sea
materializable en SQL. Impacto: defines un `fields.Float(compute=...)` sin `store=True`, lo pones
como medida en el pivot y no aparece / "Invalid field for aggregate". Síntoma típico: "mi campo
calculado no sale como medida en el pivot/graph". Fix: `store=True` (+ `_depends`) o pre-agrégalo en
un report SQL.

**RPTG-BI4. `group_operator` ya no existe: usar `aggregator` (v18)** — Renombrado en v18
(`odoo/fields.py:482-484`, `Field._get_attrs`, DeprecationWarning). Impacto: módulos viejos / OCA que declaran
`fields.Float(group_operator='avg')` siguen funcionando por el shim pero loguean warning; en código
nuevo usa `aggregator='avg'`. Si un campo numérico no debe sumarse en el pivot, pon
`aggregator=None` explícito (como hace Odoo con `sequence`, `odoo/fields.py:1538-1540`, `Integer._get_attrs`).

**RPTG-BI5. Sumar un campo que no tiene sentido sumar (precio unitario, %)** — Por defecto TODO
`Float`/`Integer`/`Monetary` tiene `aggregator='sum'` (`odoo/fields.py:1534`, `Integer._get_attrs` y `odoo/fields.py:1629`, `Float.__init__` y `odoo/fields.py:1721`, `Monetary.__init__`). Impacto: el pivot
SUMA `price_unit` o `discount %` y da números absurdos (suma de precios unitarios). Los reports bien
hechos ponen `aggregator='avg'` en esos campos (ej. `sale/report/sale_report.py:83-84`, `SaleReport.price_unit`,
`account/report/account_invoice_report.py:51`, `AccountInvoiceReport.price_average`). Si extiendes un modelo y añades un ratio/precio, declara
`aggregator='avg'` o `None`.

**RPTG-BI6. `aggregator='recordset'` solo en campos relacionales** — `_read_group_select` rechaza
`recordset` sobre campos no relacionales (`odoo/models.py:2069-2070`, `BaseModel._read_group_select`): solo válido en relacional o `id`.
Impacto: error "Aggregate method 'recordset' can be only used on relational field".

**RPTG-BI7. El pivot/graph dispara N read_groups (uno por nivel expandido), no uno solo** —
`pivot_model.js` hace un read_group por cada combinación de groupBys expandidos
(`expandedRowGroupBys`/`expandedColGroupBys`). Impacto: un pivot con 3 niveles de fila × 2 de columna
totalmente expandido sobre un report SQL pesado = muchas subqueries → lentitud. No es bug; es por
qué "expandir todo" en un pivot grande tarda. Para datasets enormes, pre-agrega en un report SQL en
vez de read_group sobre la tabla transaccional.

**RPTG-BI8. `<field invisible="1">` en pivot/graph excluye el campo de medidas/groupby** — Los arch
parsers descartan campos con `invisible` `"1"`/`"True"` (`web/static/src/views/pivot/pivot_arch_parser.js:44-45`, `PivotArchParser.parse`,
`web/static/src/views/graph/graph_arch_parser.js:64`, `GraphArchParser.parse`). Impacto: "ese campo no me aparece como medida en el pivot" porque el
arch lo marca invisible (a veces heredado de una vista padre). Revisa el arch efectivo, no solo el
tuyo.

---

## PARTE D — Motor account.report (Enterprise)

**RPTG-AR1. El modelo está en CE pero el cálculo en EE** — `account.report` /
`account.report.expression` se definen en CE `account/models/account_report.py:545-557`, `AccountReportExpression.engine`, pero los
engines (`_compute_*`, render, drill-down) viven en EE `account_reports`. Impacto: en una instancia
**solo CE** puedes ver el modelo y crear expresiones, pero los informes financieros (P&L, Balance,
303) NO se calculan ni renderizan: falta el motor EE. Síntoma: "tengo account.report en la BD pero
el menú de informes financieros no existe / no calcula".

**RPTG-AR2. Engine `domain` SIEMPRE requiere subformula** — Constraint
`domain_engine_subformula_required` (`account_reports/models/account_report.py:586-588`, `AccountReport._get_shifted_dates_period`):
`CHECK(engine != 'domain' OR subformula IS NOT NULL)`. Impacto: creas una expresión engine `domain`
sin subformula (que indica `sum`/`sum_if_pos`/`count_rows`…) → IntegrityError al guardar. La
subformula define CÓMO se agrega el resultado del domain.

**RPTG-AR3. Los `*_formula` shortcut son store=False — no busques su valor en BD** — `domain_formula`,
`account_codes_formula`, `aggregation_formula`, etc. (`account/models/account_report.py:350-355`, `class AccountReportLine`) son campos
`store=False` con `inverse=` que CREAN las `account.report.expression` reales. Impacto: lees
`report_line.aggregation_formula` por SQL/ORM y sale vacío porque no se almacena; el dato vive en
`expression_ids`. Sirven solo para definir el report compacto desde XML.

**RPTG-AR4. engines `tax_tags`/`account_codes` ignoran `next_groupby`** — Están en
`NO_NEXT_GROUPBY_ENGINES` (EE `account_reports/models/account_report.py:45`, `NO_NEXT_GROUPBY_ENGINES`), optimización que les pasa `next_groupby=None`
(`:3143`). Impacto: si esperas drill-down por sub-grupo en una línea con engine tax_tags, no lo
obtienes igual que con engine domain. (Detalle completo en `cross-module/account_reports/`, ARG.)

---

## v19

**RPTG21. v19 sigue con wkhtmltopdf — NO migra a Chromium/Playwright** — Verificado en
`odoo19/odoo/addons/base/models/ir_actions_report.py`: mismo `report_type`
(qweb-html/pdf/text, línea 170-174), mismos estados `install/ok/upgrade/workers/broken`, misma
comprobación >=0.12.0 / zoom >=0.12.2 (líneas 111-121), mismo `web.minimal_layout`/`external_layout`,
mismos campos de `report.paperformat`. **Por tanto TODOS los gotchas RPTG1-RPTG20 aplican igual en
v19.** No esperes que v19 arregle el render moderno de CSS: el WebKit viejo sigue ahí.

**RPTG26. Paper Muncher (v20) — motor PDF propio, NO en v18/v19 (v20 reportado, sin verificar
contra fuente)** — **Verificado (grep en v18 y v19): NO existe** módulo `base_report_paper_muncher`
ni `report_type='qweb-pdf-paper-muncher'` ni métodos `_run_pdf_engine`/`_get_pdf_engine`/
`get_pdf_engine_state`. v18 y v19 son **wkhtmltopdf-only** → RPTG1-24 aplican tal cual.
⚠ **v20 (reportado, NO verificado — sin checkout v20 local)**: Odoo introduciría en v20 un motor de
PDF propio (Paper Muncher), enchufable vía `report_type`, con soporte de CSS moderno (flex/grid). Los
nombres de arriba son los reportados, **sin confirmar contra fuente**. Implicación: la emisión de
informes que quiera relajar los límites de wkhtml debe ser **motor-aware** (comprobar el motor antes
de asumir CSS moderno); en v18/v19 sigue siendo wkhtml y el CSS moderno rompe.

**RPTG22. v19 detecta QT no parcheado en multi-documento** — v19 añade un helper a nivel de módulo
`_run_wkhtmltopdf(args)` con `subprocess.run(..., check=False)` (v19 línea 41-54) y una rama que, al
convertir varios documentos con un QT no parcheado, lanza `UserError("Tried to convert multiple
documents in wkhtmltopdf using unpatched QT")` o `unittest.SkipTest` en tests (v19 líneas 627-628). En
v18 esa rama no existe. Impacto: en v19, informes multi-registro requieren el wkhtmltopdf **con QT
parcheado** (el build oficial de Odoo / wkhtmltopdf.org con patched qt); un binario de distro estándar
puede fallar el multi-doc con ese error explícito en vez de generar un PDF corrupto silenciosamente.
