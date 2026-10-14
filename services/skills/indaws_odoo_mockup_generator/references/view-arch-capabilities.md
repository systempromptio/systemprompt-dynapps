# Capacidades de arch/QWeb de la vista (más allá de campos+widgets)

Una vista Odoo NO es solo `<field>`+widget: la arch (y en kanban/informes, **QWeb**) da mucho más, y
casi todo es **nativo declarativo** (STD/Studio, no dev). No infravalores lo que Odoo da gratis.

## LIST — es más que columnas ✅ (implementado en el motor)
- **Color de fila por condición**: `decoration-danger|warning|success|info|muted="<expr python>"` en `<list>`.
  Ej.: `<list decoration-danger="riesgo == 'excedido'">` → filas rojas. **Nativo**.
- **Totales/agregados en el pie**: `sum="Total"` (o `avg=`) en una columna numérica → pie con la suma. **Nativo**.
- **Columnas opcionales**: `<field ... optional="hide|show"/>` → el usuario las activa desde el icono ⚙. **Nativo**.
- **Columna de botón**: `<button type="object" name="..." icon="fa-..."/>` dentro de la `<list>`. Botón = método (dev si es nuevo).
- **Editable en línea**: `<list editable="top|bottom">`. **Nativo**. · **Orden**: `default_order`. · **Agrupar**: `<groupby name="...">`.

## KANBAN — es QWeb puro ✅ (implementado en el motor)
`t-if`/`t-else`, `t-out`, `t-att-class` (tarjeta condicional/computada) · **color** (`color`/`kanban_color`) ·
**progressbar** de columna (`<progressbar field=.. colors=..>`) · **dropdown** de acciones (`role="menuitem"`) ·
`<widget name="web_ribbon"/>` en la tarjeta · cabeceras de grupo agregadas.

## FORM — extras ✅ (implementado en el motor)
`<widget name="web_ribbon" title=.. bg_color="text-bg-danger" invisible=".."/>` (cinta de esquina) ·
`<div class="alert alert-warning" role="alert" invisible=".."/>` (callout nativo) ·
`<div class="oe_button_box"><button class="oe_stat_button" .../></div>` (botones de estadística) ·
otros *view widgets* (`registry.category('view_widgets')`: signature, notification_alert, attach_document…) ·
HTML libre en el sheet (`<h2>`, `oe_avatar`) · `<field widget=.. options="{...}"/>`.

## SEARCH — panel lateral ✅ (implementado en el motor)
`<search><searchpanel><field name=.. select="one|multi" icon="fa-.." enable_counters="1"/></searchpanel></search>`.
`select="one"` = categoría (una elección) · `select="multi"` = filtros (checkboxes) · `enable_counters` = contadores.
También `<filter>` con dominio · `<separator>` · favoritos · group by múltiple. Aplica a list/kanban (multi-registro).

## VISTAS ANALÍTICAS / TEMPORALES ✅ (implementado en el motor)
**CE (Community, sin licencia extra):**
- `graph` — `<graph type="bar|line|pie"><field name=.. type="row"/><field name=.. type="measure"/></graph>`. Necesita medida numérica + agrupación.
- `pivot` — `<pivot><field type="row"/><field type="col"/><field type="measure"/></pivot>`. Tabla dinámica.
- `calendar` — `<calendar date_start=.. date_stop=.. mode="month" color=..>`. Necesita ≥1 campo fecha.

**EE (Enterprise, requiere licencia + su módulo):**
- `gantt` (`web_gantt`) — `<gantt date_start=.. date_stop=.. default_group_by=..>`. Necesita 2 fechas. Alt. CE: calendar/lista ordenada.
- `cohort` (`web_cohort`) — `<cohort date_start=.. date_stop=.. interval="week" mode="churn|retention">`. Alt. CE: pivot/graph.
- `map` (`web_map`) — `<map res_partner="partner_id"/>`. Necesita m2o a res.partner. Alt. CE: lista + campo dirección.
- `grid` (`web_grid`) — matriz editable (timesheets). Alt. CE: lista editable.

Regla EE: **márcalo siempre como EE** y ofrece la alternativa CE al lado (igual que en `non-native-to-native.md`);
el motor emite el arch pero con comentario `<!-- Requiere Odoo Enterprise (web_*) -->`.

## REPORT (QWeb) — superficie propia (diferida)
`ir.actions.report` + plantilla QWeb (`t-foreach`, `t-field`, `web.external_layout`, tablas, totales).
Es layout de impresión, no interacción → posible **motor de mockups propio**. Ver `spec-to-odoo.md`.

## Regla
Antes de marcar algo dev, pregunta si es **arch declarativa** (decoration/sum/optional/searchpanel/ribbon =
nativo). Solo la lógica (métodos de botón, computes, QWeb de kanban muy custom, plantilla de informe) es dev.
