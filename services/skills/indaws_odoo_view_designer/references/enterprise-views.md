# Vistas Enterprise de Odoo (gantt, grid, map, cohort, hierarchy, dashboard)

Reference práctico de las vistas que **NO** vienen en Community. Todo lo de aquí está
verificado contra el código fuente de Odoo v18 y v19 (Enterprise y, donde se indica,
Community). Cada atributo cita su archivo fuente. Lo que no he podido confirmar en
fuente está marcado como **NO CONFIRMADO** y no debe darse por bueno.

## Licenciamiento (leer antes de usar)

| Vista | Tipo | Módulo que la aporta | ¿Requiere licencia EE? |
|-------|------|----------------------|------------------------|
| `gantt` | Enterprise | `web_gantt` (`$ODOO_ENTERPRISE_SRC/{18,19}/web_gantt`) | **Sí (EE)** |
| `grid` | Enterprise | `web_grid` (`$ODOO_ENTERPRISE_SRC/{18,19}/web_grid`) | **Sí (EE)** |
| `map` | Enterprise | `web_map` (`$ODOO_ENTERPRISE_SRC/{18,19}/web_map`) | **Sí (EE)** |
| `cohort` | Enterprise | `web_cohort` (`$ODOO_ENTERPRISE_SRC/{18,19}/web_cohort`) | **Sí (EE)** |
| `hierarchy` | **Community** | `web_hierarchy` (`$ODOO_SRC/odoo{17,18,19}/addons/web_hierarchy`) | **No (CE)** — ver nota |
| `dashboard` | — | no es un view type clásico (board CE / spreadsheet_dashboard EE) | ver sección |

> **Nota importante sobre `hierarchy`:** aunque el enunciado la pidió como "Enterprise",
> en fuente NO está en `$ODOO_ENTERPRISE_SRC`. El tipo de vista `hierarchy` vive en el
> addon **`web_hierarchy` de Community** (confirmado: existe en
> `$ODOO_SRC/odoo17/addons/web_hierarchy`, `$ODOO_SRC/odoo18/addons/web_hierarchy` y
> `$ODOO_SRC/odoo19/addons/web_hierarchy`; NO existe `web_hierarchy` en
> `$ODOO_ENTERPRISE_SRC/18` ni `$ODOO_ENTERPRISE_SRC/19`). Disponible desde v17. Se
> documenta aquí igualmente por petición.

Cada vista se declara como un view type más en el `<record model="ir.actions.act_window">`
(`view_mode="gantt,..."` etc.) y con su `<gantt>` / `<grid>` / `<map>` / `<cohort>` /
`<hierarchy>` como raíz del arch de la `ir.ui.view`.

---

## GANTT

Fuente: `$ODOO_ENTERPRISE_SRC/19/web_gantt/static/src/gantt_arch_parser.js`
(`getInfoFromRootNode`, líneas 219-362) y `gantt_model.js`. Comparado con v18.

Los atributos se leen casi todos en `getInfoFromRootNode`. **Solo lo listado abajo está
confirmado en el arch_parser v19.** Si un atributo aparece en arch de ejemplo pero no en
el parser, lo señalo.

| Atributo | Qué hace | Ejemplo | Versión / fuente |
|----------|----------|---------|------------------|
| `date_start` | Campo de fecha/datetime de inicio de cada pill. Obligatorio. | `date_start="date_start"` | v18, v19 (`dateStartField`, L338) |
| `date_stop` | Campo de fecha/datetime de fin de cada pill. Obligatorio. | `date_stop="date"` | v18, v19 (`dateStopField`, L339) |
| `default_scale` | **Heredado.** Escala/rango por defecto. En v19 es alias de `default_range`. | `default_scale="month"` | v18, v19 (L262: `attrs.default_range || attrs.default_scale`) |
| `default_range` | **v19.** Nombre nuevo de `default_scale`. Valores: `day,week,month,quarter,year`. Si no es válido → `"custom"`. | `default_range="week"` | v19 (L262-269). En v18 NO existe como atributo |
| `scales` | Lista de rangos permitidos (coma-separados). Valores válidos: `day,week,month,quarter,year`. | `scales="week,month,quarter,year"` | v18, v19 (L248-260, set `allowedRanges`) |
| `precision` | Dict por escala con la subdivisión de celda. Formato `{'escala':'intervalo:parte'}` donde parte ∈ `full/half/quarter`. | `precision="{'week':'day:full','month':'day:full'}"` | v18, v19 (L275-289) |
| `color` | Campo cuyo valor (entero) determina el color del pill. | `color="stage_id_color"` | v18, v19 (`colorField`, L334) |
| `decoration-info` / `-success` / `-warning` / `-danger` / `-secondary` | Color del pill según expresión Python sobre el registro. | `decoration-danger="planned_date < today"` | v18, v19 (`DECORATIONS` L10-16, `pillDecorations` L318-326) |
| `dependency_field` | Campo M2M con las dependencias (predecesoras). Activa flechas de dependencia. | `dependency_field="depend_on_ids"` | v18, v19 (`dependencyField`, L244-245) |
| `dependency_inverted_field` | Campo inverso (sucesoras) de la dependencia. | `dependency_inverted_field="dependent_ids"` | v18, v19 (`dependencyInvertedField`, L246) |
| `progress` | Campo (0-100) que pinta una barra de progreso dentro del pill. | `progress="progress"` | v18, v19 (`progressField`, L354) |
| `progress_bar` | Lista de campos (coma-sep) para barras de progreso por grupo (capacidad). | `progress_bar="user_id"` | v18, v19 (`progressBarFields`, L353) |
| `plan` | Si `false`, deshabilita la acción "planificar" (arrastrar registros sin fecha al gantt). Por defecto activo si hay `edit`. | `plan="0"` | v18, v19 (`canPlan`, L227) |
| `cell_create` | Si `false`, no permite crear arrastrando sobre una celda vacía. | `cell_create="0"` | v18, v19 (`canCellCreate`, L226) |
| `consolidation` | Campo numérico que se consolida (suma) por fila al agrupar. | `consolidation="planned_hours"` | v18, v19 (`consolidationParams.field`, L239) |
| `consolidation_max` | Dict `{'campo': max}`: umbral por encima del cual la consolidación se marca en rojo. | `consolidation_max="{'user_id': 100}"` | v18, v19 (L231-235) |
| `consolidation_exclude` | Campo booleano: si True excluye ese registro de la consolidación. | `consolidation_exclude="is_absent"` | v18, v19 (`consolidationParams.excludeField`, L238) |
| `default_group_by` | Campo(s) de agrupación por defecto del gantt. | `default_group_by="user_id"` | v18, v19 (NO está en `getInfoFromRootNode`; se procesa en el modelo/view — confirmado en arch de ejemplo `project_enterprise`) |
| `pill_label` | Si está presente, calcula y muestra una etiqueta de texto en el pill. | `pill_label="1"` | v18, v19 (`computePillDisplayName`, L335) |
| `thumbnails` | Dict `{'campo':'campo_imagen'}` para mostrar miniaturas en las filas agrupadas. | `thumbnails="{'user_id':'image_128'}"` | v18, v19 (L358) |
| `form_view_id` | ID de la vista formulario para crear/editar registros desde el gantt. | `form_view_id="%(my_form)d"` | v18, v19 (`formViewId`, L348) |
| `kanban_view_id` | ID de la vista kanban usada en el panel lateral móvil. | `kanban_view_id="%(view_project_kanban)d"` | v18, v19 (`kanbanViewId`, L349) |
| `total_row` | Muestra una fila de totales. | `total_row="True"` | v18, v19 (`displayTotalRow`, L346) |
| `display_unavailability` | Pinta franjas de indisponibilidad (p.ej. fuera de horario laboral). | `display_unavailability="1"` | v18, v19 (`displayUnavailability`, L347) |
| `disable_drag_drop` | Deshabilita arrastrar/redimensionar pills. | `disable_drag_drop="1"` | v18, v19 (`disableDrag`, L344) |
| `display_mode` | `dense` (por defecto) o `sparse`. Compactación de filas. | `display_mode="sparse"` | v18, v19 (`displayMode`, L345) |
| `on_create` | Acción a ejecutar al crear (en vez del form). | `on_create="%(action)d"` | v18, v19 (`createAction`, L337) |
| `multi_create_view` | Vista para creación múltiple. | `multi_create_view="..."` | v18, v19 (`multiCreateView`, L350) |
| `groups_limit` | Límite de grupos por página (pager). | `groups_limit="20"` | v18, v19 (`pagerLimit`, L351) |
| `dynamic_range` | Si `1`, el rango temporal se ajusta dinámicamente a los datos (en lugar de fijo). | `dynamic_range="1"` | **v18 y v19 (atributo válido, definido en `web_gantt/models/ir_ui_view.py` L30; en v18 además aparece como `dynamicRange` en `gantt_model.js`). NO se procesa en `getInfoFromRootNode` — el modelo lo consume.** Usado en arch real: `website_event_track_gantt` v19 |

Subnodo `<templates>` con `<t t-name="gantt-popover">`: define el popover (cuerpo y
`<footer>`) al pasar/clicar sobre un pill (L175-207). `<footer replace="0">` conserva los
botones genéricos.

Subnodo `<field name="...">`: registra campos extra a cargar (p.ej. los usados en
decorations o en el popover) — se acumulan en `decorationFields` (L170-173).

Ejemplo real (v19, `project_enterprise/views/project_views.xml`):

```xml
<gantt date_start="date_start" date_stop="date"
       default_scale="month" scales="week,month,quarter,year"
       string="Planning" display_unavailability="1"
       js_class="project_gantt"
       precision="{'week': 'day:full', 'month': 'day:full'}"
       default_group_by="user_id" total_row="True"
       kanban_view_id="%(project.view_project_kanban)d"
       color="stage_id_color">
</gantt>
```

---

## GRID

Fuente: `$ODOO_ENTERPRISE_SRC/19/web_grid/static/src/views/grid_arch_parser.js`.
**El arch_parser de grid es BYTE-A-BYTE idéntico en v18 y v19** (confirmado con `diff`:
solo cambia la cabecera `/** @odoo-module */` que v19 omite). Cero diferencias funcionales.

Atributos del nodo raíz `<grid>`:

| Atributo | Qué hace | Ejemplo | Versión / fuente |
|----------|----------|---------|------------------|
| `string` | Título de la vista. | `string="Timesheets"` | v18, v19 |
| `editable` | Permite editar celdas inline. Solo surte efecto si la `measure` es no-readonly y su aggregator es `sum` (L131-135). | `editable="1"` | v18, v19 (L61-63) |
| `create_inline` | Botón "Añadir línea" dentro del grid. | `create_inline="1"` | v18, v19 (L49-51) |
| `display_empty` | Muestra el grid aunque no haya filas. | `display_empty="1"` | v18, v19 (L52-54) |
| `barchart_total` | Pinta un gráfico de barras con los totales por columna. | `barchart_total="1"` | v18, v19 (L44-48, `hasBarChartTotal`) |
| `hide_line_total` | Oculta la columna de total por fila. | `hide_line_total="1"` | v18, v19 (L36-38) |
| `hide_column_total` | Oculta la fila de total por columna. | `hide_column_total="1"` | v18, v19 (L39-43) |
| `form_view_id` | Vista formulario para crear/editar. | `form_view_id="%(form)d"` | v18, v19 (L64-67) |
| `action` + `type` | Acción a abrir al pulsar una celda (requiere ambos). | `action="my_action" type="object"` | v18, v19 (L55-60, `openAction`) |

Subnodos `<field>` (atributo `type` define el rol; por defecto `row`):

| `type` | Qué hace | Notas / fuente |
|--------|----------|----------------|
| `row` | Campo de agrupación en filas (puede haber varios). | L74-93. Soporta `widget=` y `section="1"` (un solo campo marcado como sección agrupa visualmente, L78-86) |
| `col` | Campo de la dimensión columna (normalmente fecha). Contiene los `<range>`. | L94-99 |
| `measure` | Campo numérico medido en cada celda. Acepta `operator` (aggregator, p.ej. `sum`), `widget`, `readonly`, `string`. | L100-110 |
| `readonly` | Campo de medida de solo lectura adicional. Acepta `operator`/`string`. | L111-121 |

> **OJO (v18→v19 sin cambio, pero a tener en cuenta):** NO existen los atributos
> `adjustment` ni `adjust_name` en el grid_arch_parser de v18/v19. (Sí existían en
> versiones antiguas v13-v14.) En v18/v19 el ajuste manual de celdas se gobierna por
> `editable` + el aggregator `sum`. No los uses; no se parsean.

Subnodos `<range>` (hijos de `<field type="col">`), parseados en `_extractRanges`
(L154-176):

| Atributo de `<range>` | Qué hace | Ejemplo |
|------------------------|----------|---------|
| `name` | Identificador del rango. | `name="week"` |
| `string` | Etiqueta visible. | `string="Week"` |
| `span` | Anchura total del periodo mostrado (`day/week/month/year`). | `span="week"` |
| `step` | Granularidad de cada columna dentro del span. | `step="day"` |
| `default` | Rango activo por defecto. | `default="1"` |
| `hotkey` | Atajo de teclado para cambiar a ese rango. | `hotkey="w"` |

Subnodo `<button>`: botón de acción en el grid (procesado con `processButton`, L123-129).

Ejemplo real (v19, `timesheet_grid/views/account_analytic_line_views.xml`):

```xml
<grid string="Timesheets" js_class="timer_timesheet_grid"
      barchart_total="1" create_inline="1" editable="1" display_empty="1"
      form_view_id="%(timesheet_grid.hr_timesheet_line_form_grid)d">
    <field name="project_id" type="row" widget="timesheet_many2one"/>
    <field name="task_id" type="row" widget="timesheet_many2one"/>
    <field name="date" type="col">
        <range name="day" string="Day" span="day" step="day" hotkey="e"/>
        <range name="week" string="Week" span="week" step="day" hotkey="w" default="1"/>
        <range name="month" string="Month" span="month" step="day" hotkey="m"/>
    </field>
    <field name="unit_amount" string="Time Spent" type="measure" widget="timesheet_uom"/>
</grid>
```

---

## MAP

Fuente: `$ODOO_ENTERPRISE_SRC/19/web_map/static/src/map_view/map_arch_parser.js`.
La vista map renderiza registros geolocalizados a través de un campo `res.partner`.

Atributos del nodo raíz `<map>`:

| Atributo | Qué hace | Ejemplo | Versión / fuente |
|----------|----------|---------|------------------|
| `res_partner` | **Obligatorio.** Campo M2O a `res.partner` que aporta la dirección a geolocalizar. | `res_partner="partner_id"` | v18, v19 (L31, `resPartnerField`) |
| `routing` | Si `1`, dibuja la ruta entre los marcadores (en orden). | `routing="1"` | v18, v19 (L40-42) |
| `hide_name` | Oculta el nombre (`display_name`) en marcador/panel. Si false, carga `display_name`. | `hide_name="1"` | v18, v19 (L49-54) |
| `hide_address` | Oculta la dirección en el panel/popup. | `hide_address="1"` | v18, v19 (L46-48) |
| `hide_title` | Oculta el título del panel lateral. | `hide_title="1"` | v18, v19 (L43-45) |
| `panel_title` | Texto del título del panel lateral. | `panel_title="Clientes"` | v18, v19 (L37-39) |
| `default_order` | Orden por defecto de los registros (afecta ruta y panel). | `default_order="date desc"` | v18, v19 (L55-57) — **ver diff** |
| `limit` | Límite de registros a mostrar. | `limit="80"` | v18, v19 (L34-36) |
| `allow_resequence` | Permite reordenar (drag) los registros en el panel. | `allow_resequence="1"` | v18, v19 (L58-60) |

Subnodo `<field name="..." string="...">`: cada campo se añade al popup del marcador
(`fieldNamesMarkerPopup`, L62-68). El `string` opcional es la etiqueta mostrada.

> En fuente **no existe** un nodo `<marker-popup>` como tal: el popup se construye a partir
> de los `<field>` hijos directos del `<map>`. (Si una doc externa menciona
> `<marker-popup>`, no está respaldado por el arch_parser v18/v19.)

Ejemplo real (v19, `crm_enterprise/views/crm_lead_views.xml`):

```xml
<map res_partner="partner_id" routing="1">
    <field name="partner_id" string="Customer"/>
</map>
```

---

## COHORT

Fuente: `$ODOO_ENTERPRISE_SRC/19/web_cohort/static/src/cohort_arch_parser.js` +
constantes en `cohort_model.js`. Analiza retención/abandono a lo largo del tiempo.

Atributos del nodo raíz `<cohort>`:

| Atributo | Qué hace | Valores | Ejemplo | Versión / fuente |
|----------|----------|---------|---------|------------------|
| `date_start` | **Obligatorio.** Campo de fecha de "entrada" en la cohorte. Si falta → error. | campo date/datetime | `date_start="create_date"` | v18, v19 (L24-30) |
| `date_stop` | **Obligatorio.** Campo de fecha de "salida". Si falta → error. | campo date/datetime | `date_stop="close_date"` | v18, v19 (L31-37) |
| `mode` | Modo de cálculo. | `retention` (def.) / `churn` | `mode="churn"` | v18, v19 (L38-48; `MODES = ["retention","churn"]`) |
| `timeline` | Dirección temporal. | `forward` (def.) / `backward` | `timeline="backward"` | v18, v19 (L49-59; `TIMELINES = ["forward","backward"]`) |
| `interval` | Granularidad de las columnas. | `day` (def.) / `week` / `month` / `quarter` / `year` | `interval="month"` | v18, v19 (L61-71; `INTERVALS` con day/week/month/quarter/year) |
| `measure` | Campo agregado a medir. Por defecto `__count`. | campo numérico | `measure="amount"` | v18, v19 (L60) |
| `string` | Título de la vista. | texto | `string="Tickets"` | v18, v19 (L20-23, `title`) |
| `disable_linking` | Si `1`, las celdas no abren la lista de registros al hacer click. | bool | `disable_linking="1"` | v18, v19 (L15-19) |

Subnodos `<field name="...">`: definen campos auxiliares; aceptan `string`, `invisible`
(`True`/`1`) y `widget` (L74-91).

Ejemplo real (v19, `helpdesk/views/helpdesk_ticket_views.xml`):

```xml
<cohort string="Tickets" date_start="create_date" date_stop="close_date"
        interval="day" sample="1">
    <field name="color" invisible="1"/>
</cohort>
```

---

## HIERARCHY (Community, desde v17)

Fuente: `$ODOO_SRC/odoo19/addons/web_hierarchy/static/src/hierarchy_arch_parser.js`.
**El arch_parser es idéntico en v18 y v19** (`diff` solo muestra la cabecera
`/** @odoo-module */` que v19 omite; cero cambios de lógica). Disponible desde v17.
**Es Community, no Enterprise** (ver nota de licenciamiento arriba).

Renderiza un organigrama / árbol jerárquico de un modelo auto-referenciado.

Atributos del nodo raíz `<hierarchy>`:

| Atributo | Qué hace | Ejemplo | Versión / fuente |
|----------|----------|---------|------------------|
| `parent_field` | Campo M2O al padre (debe ser M2O al mismo modelo). Por defecto `parent_id`. Valida tipo y co-modelo, lanza error si no. | `parent_field="parent_id"` | v17, v18, v19 (L28-38) |
| `child_field` | Campo O2M de hijos (debe ser O2M al mismo modelo). Valida tipo y co-modelo. | `child_field="child_ids"` | v17, v18, v19 (L39-49) |
| `draggable` | Permite arrastrar nodos para reasignar el padre. | `draggable="1"` | v17, v18, v19 (L50-52) |
| `icon` | Clase FontAwesome del icono del nodo. Por defecto `fa-share-alt fa-rotate-90 align-text-top`. | `icon="fa-users"` | v17, v18, v19 (L53-55) |
| `default_order` | Orden por defecto de los nodos. | `default_order="name"` | v17, v18, v19 (L11, `stringToOrderBy`) |

Subnodos `<field name="...">`: campos a cargar/mostrar (parseados con
`Field.parseFieldNode`, L56-64).

Subnodo `<templates>` con `<t t-name="hierarchy-box">`: **obligatorio.** Plantilla QWeb de
cada caja/nodo. Si falta, lanza `"Missing 'hierarchy-box' template."` (L68-71). Dentro se
usan `<field>`, `<button>`, `record.<campo>.raw_value`, etc.

Ejemplo real (v19, `hr_org_chart/views/hr_department_views.xml`):

```xml
<hierarchy child_field="child_ids" draggable="1">
    <field name="name"/>
    <field name="color"/>
    <field name="total_employee"/>
    <templates>
        <t t-name="hierarchy-box">
            <div t-attf-class="o_hierarchy_node_header ... o_hierarchy_node_color_{{ record.color.raw_value }}">
                <field name="name"/>
            </div>
            <div class="o_hierarchy_node_body d-flex flex-column">
                <field name="manager_id" widget="many2one_avatar_employee"/>
                <button class="btn btn-link" name="action_employee_from_department" type="object">
                    <t t-out="record.total_employee.raw_value"/> Employees
                </button>
            </div>
        </t>
    </templates>
</hierarchy>
```

---

## DASHBOARD

**No existe un view type `dashboard` con arch clásico** comparable a los anteriores.
Confirmado en fuente: no hay ningún `*dashboard*arch_parser*` en
`$ODOO_ENTERPRISE_SRC/19/.../static/src`, ni nodos `<dashboard>` como raíz de vista. Hay
dos cosas distintas que la gente llama "dashboard":

1. **Board (Community) — el "My Dashboard" clásico.**
   Addon `board` (`$ODOO_SRC/odoo19/addons/board`). No es un view type propio: se monta
   sobre una vista `form` del modelo `board.board`, con un nodo `<board style="...">` que
   contiene `<column>` y dentro `<action>` (sub-vistas embebidas). Confirmado en
   `$ODOO_SRC/odoo19/addons/board/views/board_views.xml`:

   ```xml
   <form string="My Dashboard">
       <board style="2-1">
           <column>
               <!-- <action name="..."/> añadidas por el usuario desde "Add to dashboard" -->
           </column>
       </board>
   </form>
   ```

   El renderizado lo hace `board/static/src/board_view.js` (extiende la vista form). No
   tiene un `arch_parser` de atributos tabulable como las vistas EE. El único atributo del
   `<board>` es `style` (layout de columnas, p.ej. `"2-1"`, `"1-1"`, `"1"`).

2. **Spreadsheet dashboards (Enterprise).**
   Módulos `spreadsheet_dashboard*` (`$ODOO_ENTERPRISE_SRC/19/spreadsheet_dashboard`,
   `spreadsheet_dashboard_crm`, `_stock`, `_hr_payroll`, etc.). **No son un view type
   XML/arch**: son dashboards basados en hojas de cálculo (modelos
   `spreadsheet.dashboard` / `spreadsheet.dashboard.group`), cuyo contenido es JSON de
   spreadsheet, no un `<dashboard>` arch. Requieren licencia EE. No se configuran con
   atributos de vista; se diseñan desde la UI de spreadsheet.

**Conclusión:** si necesitas "una vista dashboard" en código, no hay arch propio. Usas
`board` (CE) embebiendo acciones, o los spreadsheet dashboards (EE) que no son arch.

---

## Diffs v18 → v19 (confirmados en fuente)

### GANTT — **cambios reales**
Comparado `diff` de `gantt_arch_parser.js` v18 vs v19:

- **Modelo de escalas reescrito.** v18 tenía escalas con sufijos (`week`, `week_2`,
  `month`, `month_3`) y cada `SCALES[x]` llevaba su propio `defaultRange: {unit, count}`.
  v19 simplifica `SCALES` a `day/week/month/year` y **separa el concepto de RANGES**
  (`day/week/month/quarter/year`, L146-152) del de SCALES. Las escalas con sufijo numérico
  de v18 (`week_2`, `month_3`) **ya no existen** en v19.
- **Nuevo atributo `default_range`** (L262) como nombre preferente; `default_scale` se
  mantiene como alias retrocompatible. En v18 solo existía `default_scale`.
- **Rango "custom"**: en v19, si `default_range`/`default_scale` no es válido, cae a
  `"custom"` (rango dinámico calculado, `getScaleForCustomRange`, L129-144 — función nueva
  en v19). En v18 no había esta noción de custom range en el parser.
- **`RESCHEDULE_METHODS`** nuevo en v19 (L82-86): el parser devuelve `rescheduleMethods`
  (`manual` / `consumeBuffer` / `maintainBuffer`) y `defaultRescheduleMethod:
  "maintainBuffer"` (L359-360). No existe en v18.
- **Formato de cabeceras**: v19 añade `colHeaderTitle`/`groupHeaderTitle` por escala y
  cambia los formatos de fecha; `formatLocalWeekYear` ahora produce
  `"Week N, dd MMM - dd MMM [año]"` en vez de `"WN año"` de v18.
- **`dynamic_range`**: atributo válido en ambas (`ir_ui_view.py`). En v18 figuraba como
  `dynamicRange` en `gantt_model.js`; lo consume el modelo, no el arch_parser, en ambas
  versiones.
- **Sin cambios** en: `date_start`, `date_stop`, `precision`, `color`, `decoration-*`,
  `dependency_field`, `dependency_inverted_field`, `progress`, `progress_bar`, `plan`,
  `consolidation*`, `pill_label`, `thumbnails`, `form_view_id`, `total_row`, etc. — todos
  presentes y con la misma semántica en v18 y v19.

### GRID — **sin cambios**
`grid_arch_parser.js` es idéntico v18 vs v19 (solo desaparece la cabecera
`/** @odoo-module */`). Ningún atributo añadido, eliminado ni cambiado.

### MAP — **un cambio menor**
Único cambio funcional: el parsing de `default_order`.
- v18: `archInfo.defaultOrder = { name: attr, asc: true }` (siempre ascendente, un solo
  campo).
- v19: `archInfo.defaultOrder = stringToOrderBy(attr)` → ahora soporta sintaxis de orden
  completa (`"campo desc, otro asc"`, multi-campo y dirección). (También cae la cabecera
  `@odoo-module` y se importa `stringToOrderBy`.)
- El resto de atributos (`res_partner`, `routing`, `hide_*`, `panel_title`, `limit`,
  `allow_resequence`) sin cambios.

### COHORT — **sin cambios**
`cohort_arch_parser.js` idéntico v18 vs v19 (solo desaparece la cabecera
`/* @odoo-module */`). `MODES`, `TIMELINES`, `INTERVALS` iguales.

### HIERARCHY — **sin cambios** (y es CE)
`hierarchy_arch_parser.js` idéntico v18 vs v19 (solo cabecera). Recordatorio: es
Community (`web_hierarchy`), disponible desde v17.

### DASHBOARD — n/a
No es un view type con arch; nada que diffear a nivel de atributos de vista.
```

---

## Resumen de qué se pudo confirmar

| Vista | Confirmada en v18 | Confirmada en v19 | EE/CE |
|-------|:-:|:-:|-------|
| gantt | sí | sí | EE |
| grid | sí | sí | EE |
| map | sí | sí | EE |
| cohort | sí | sí | EE |
| hierarchy | sí | sí | **CE** (no EE; desde v17) |
| dashboard | n/a | n/a | no es arch (board CE / spreadsheet EE) |
