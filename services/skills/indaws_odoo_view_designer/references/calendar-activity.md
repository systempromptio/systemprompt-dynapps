# Vistas Calendar y Activity (Odoo v18 / v19)

Referencia verificada contra fuente. Todo lo aquí documentado está confirmado
leyendo los arch parsers reales. Cuando un atributo NO aparece en el parser, se
indica explícitamente (no se inventan atributos).

Fuentes:
- Calendar v19: `$ODOO_SRC/odoo19/addons/web/static/src/views/calendar/calendar_arch_parser.js`
  (+ `calendar_model.js`, `calendar_renderer.js`).
- Calendar v18: mismo path en `odoo18`.
- Activity v18 y v19: `addons/mail/static/src/views/web/activity/activity_arch_parser.js`
  (+ `activity_renderer.xml`, `activity_record.xml`). **No** está en `web/views`.

---

# 1. Vista CALENDAR

La etiqueta raíz es `<calendar>`. El parser lee los atributos del nodo raíz y
recorre los `<field>` hijos para construir el popover y los filtros laterales.

```xml
<calendar date_start="start" date_stop="stop" color="user_id" mode="month"
          event_open_popup="1" quick_create="1">
    <field name="user_id" filters="1" avatar_field="image_128"/>
    <field name="partner_id"/>
    <field name="description" invisible="1"/>
</calendar>
```

## 1.1 Atributos del nodo `<calendar>`

Los atributos de campo (mapeo a campos del modelo) están en la constante
`FIELD_ATTRIBUTE_NAMES` del parser. El resto son atributos de configuración.

### Atributos de campo (FIELD_ATTRIBUTE_NAMES)

| Atributo | Qué hace | Ejemplo | Notas de versión |
|----------|----------|---------|------------------|
| `date_start` | **OBLIGATORIO**. Campo Date/Datetime que marca el inicio del evento. Si falta, el parser lanza `CalendarParseArchError`. | `date_start="start_date"` | v18 y v19 |
| `date_stop` | Campo de fin del evento. Si se define, el evento ocupa el rango start→stop. | `date_stop="end_date"` | v18 y v19 |
| `date_delay` | Alternativa a `date_stop`: campo Float con la duración en horas. Se usa cuando no hay fecha de fin explícita. | `date_delay="duration"` | v18 y v19 |
| `all_day` | Campo Boolean que indica si el evento es de día completo (sin hora). | `all_day="allday"` | v18 y v19 |
| `color` | Campo cuyo valor agrupa eventos por color (M2o, selección o entero). También alimenta los colores de los filtros laterales. | `color="user_id"` | v18 y v19 |
| `create_name_field` | Campo que se rellena con el título al hacer quick create (por defecto `name`/`display_name`). | `create_name_field="title"` | v18 y v19 |
| `recurrence_update` | Campo para gestionar el alcance de actualización de eventos recurrentes. | `recurrence_update="recurrence_update"` | **Solo v18**. Eliminado de `FIELD_ATTRIBUTE_NAMES` en v19. |

### Atributos de configuración

| Atributo | Qué hace | Ejemplo | Notas de versión |
|----------|----------|---------|------------------|
| `mode` | Escala inicial de la vista. Valores válidos: `day`, `week`, `month`, `year`. Debe estar incluido en `scales` o lanza error. | `mode="month"` | v18 y v19 |
| `scales` | Lista (CSV) de escalas habilitadas; filtra contra `["day","week","month","year"]`. | `scales="week,month"` | v18 y v19 |
| `create` | Permite crear eventos (default `true`). | `create="0"` | v18 y v19 |
| `edit` | Permite editar eventos (default `true`). | `edit="0"` | v18 y v19 |
| `delete` | Permite borrar eventos (default `true`). | `delete="0"` | v18 y v19 |
| `quick_create` | Activa el formulario rápido al pulsar en una celda (default `true`). | `quick_create="0"` | v18 y v19 |
| `quick_create_view_id` | ID de un form view a usar como popup de quick create. Solo se lee si `quick_create` es true. | `quick_create_view_id="42"` | v18 y v19 |
| `event_open_popup` | Si es true, al pinchar un evento abre el popover/diálogo de edición en lugar de navegar al form. Default `false`. | `event_open_popup="1"` | v18 y v19 |
| `event_limit` | Nº máximo de eventos por celda antes de mostrar "ver más". Debe ser entero o lanza error. Default `5`. | `event_limit="3"` | v18 y v19 |
| `form_view_id` | ID del form view a abrir al editar/crear un evento. | `form_view_id="55"` | v18 y v19 |
| `hide_date` | Oculta la fecha en el popover del evento. Default `false`. | `hide_date="1"` | v18 y v19 |
| `hide_time` | Oculta la hora en el popover del evento. Default `false`. | `hide_time="1"` | v18 y v19 |
| `show_unusual_days` | Resalta días no laborables (festivos/fines de semana según calendario). Default `false`. | `show_unusual_days="1"` | v18 y v19 |
| `js_class` | Registra una clase JS personalizada para la vista. | `js_class="my_calendar"` | v18 y v19 |
| `aggregate` | **NUEVO v19**. Campo numérico a agregar; formato `campo` o `campo:operador` (sum por defecto). Muestra totales agregados por filtro en el panel lateral. | `aggregate="amount:sum"` | **Solo v19** |
| `multi_create_view` | **NUEVO v19**. Vista a usar para creación múltiple de eventos. | `multi_create_view="my_view"` | **Solo v19** |
| `show_date_picker` | **NUEVO v19**. Muestra/oculta el mini date picker lateral. Default `true`. | `show_date_picker="0"` | **Solo v19** |
| `month_overflow` | **NUEVO v19**. Controla si la vista mes desborda días de meses adyacentes. Default `true`. | `month_overflow="0"` | **Solo v19** |

### Atributos que NO existen en el arch parser

- `quick_add` → **no existe**; el atributo real es `quick_create`.
- `filter_field` y similares NO van en `<calendar>`, van en `<field>` (ver abajo).

## 1.2 `<field>` dentro de `<calendar>`

Cada `<field>` se parsea con `Field.parseFieldNode(...)` y se guarda en
`popoverFieldNodes` (campos mostrados en el popover del evento). Además, si el
campo tiene atributos de filtro, se construye una entrada en `filtersInfo`
(panel lateral de filtros).

Un campo entra en `filtersInfo` cuando NO es `invisible` (o tiene `filters`) **y**
declara alguno de: `avatar_field`, `write_model`, `write_field`, `color`, `filters`.

| Atributo en `<field>` | Qué hace | Ejemplo |
|-----------------------|----------|---------|
| `filters="1"` | Genera una sección de filtros laterales para este campo (típicamente un M2o). Los registros relacionados se listan como casillas filtrables. | `<field name="user_id" filters="1"/>` |
| `color` | Dentro de un campo con `filters`, indica el campo que aporta el color de cada filtro (`colorFieldName`). | `<field name="user_id" filters="1" color="color"/>` |
| `avatar_field` | Campo de imagen (avatar) a mostrar junto a cada filtro. En v18, si hay `filters` y el campo apunta a res.users/res.partners/hr.employee, se autoasigna `image_128`. | `<field name="user_id" filters="1" avatar_field="image_128"/>` |
| `write_model` | Modelo donde persistir el estado de los filtros (filtros guardados por usuario). Se mapea a `writeResModel`. | `write_model="calendar.filters"` |
| `write_field` | Campo del `write_model` que referencia al valor filtrado. Se mapea a `writeFieldName`. | `write_field="partner_id"` |
| `filter_field` | Campo que guarda el estado activo/inactivo del filtro. Se mapea a `filterFieldName`. | `filter_field="partner_checked"` |
| `invisible` | Excluye el campo del popover; si NO lleva además `filters`, también lo excluye del panel de filtros. | `<field name="description" invisible="1"/>` |

Estructura de cada entrada de `filtersInfo` (lo que construye el parser):
`avatarFieldName`, `colorFieldName`, `fieldName`, `filterFieldName`, `label`
(= `field.string`), `resModel` (= `field.relation`), `writeFieldName`,
`writeResModel`. En v19 se añade `context` (tomado de `fieldInfo.context`).

## 1.3 Popover y formulario inline

- **Popover**: al pinchar un evento se muestra un popover con los campos de
  `popoverFieldNodes` (todos los `<field>` declarados, incluidos los invisibles
  según su flag). `hide_date`/`hide_time` ajustan qué metadatos temporales se ven.
- **Edición inline / diálogo**: con `event_open_popup="1"` la edición ocurre en
  diálogo; en caso contrario navega al form (`form_view_id` si se especifica).
- **Quick create**: `quick_create` activa el alta rápida; `quick_create_view_id`
  permite sustituir el formulario rápido por uno propio; el título se vuelca en
  `create_name_field` (o `name`/`display_name`).

---

# 2. Vista ACTIVITY

La etiqueta raíz es `<activity>`. Es una vista del módulo **mail** (no web). El
arch parser es **idéntico en v18 y v19** (mismo contenido byte a byte).

```xml
<activity string="Actividades">
    <field name="partner_id"/>
    <templates>
        <div t-name="activity-box">
            <img t-att-src="activity_image('res.partner', 'avatar_128', record.partner_id.raw_value)"
                 class="rounded-circle"/>
            <div>
                <field name="name" display="full"/>
                <field name="email"/>
            </div>
        </div>
    </templates>
</activity>
```

## 2.1 Atributos del nodo `<activity>`

El parser solo lee DOS atributos del nodo raíz:

| Atributo | Qué hace | Ejemplo | Versión |
|----------|----------|---------|---------|
| `string` | Título de la vista (`title` en el resultado del parser). | `string="Actividades"` | v18 y v19 |
| `js_class` | Clase JS personalizada; se pasa a `Field.parseFieldNode` para resolver widgets. | `js_class="my_activity"` | v18 y v19 |

No hay más atributos en el nodo raíz: el parser no lee `create`, `edit`,
`color`, etc. La rejilla de actividades por tipo la genera el renderer, no el arch.

## 2.2 Estructura interna: `<field>` y `<templates>`

El parser recorre el XML (`visitXML`) y distingue tres casos:

1. **Nodos con `t-name`** → se guardan en `templateDocs[t-name]`. Es la plantilla
   QWeb que pinta cada "tarjeta" de registro (la celda de la columna izquierda,
   renderizada por `mail.ActivityRecord` → `activity_record.xml`). Convención
   habitual: `t-name="activity-box"`.

2. **Nodos `<field>`** → se parsean con `Field.parseFieldNode(node, models,
   modelName, "activity", jsClass)` y se guardan en `fieldNodes`. A cada campo se
   le asigna un `field_id` único (`{name}_{n}`) para permitir el mismo campo
   varias veces. Estos campos quedan disponibles para usarse dentro de las
   plantillas con `<field name="..."/>`.

3. **Nodos `<img>`** → si su `t-att-src` contiene `activity_image`, el parser
   inyecta automáticamente el campo `write_date` (como `write_date_0`) en
   `fieldNodes`, para poder recargar la imagen cuando el registro cambie
   (cache-busting). Solo lo añade si `write_date` no estaba ya declarado.

Resultado del parser: `{ fieldNodes, templateDocs, title }`. Nada más.

## 2.3 Cómo se pintan las actividades por tipo

Esto lo hace el renderer (`activity_renderer.xml`), no el arch:

- Se construye una **tabla**: una **fila por registro** (`activityResIds`) y una
  **columna por tipo de actividad** (`mail.activity.type`).
- La cabecera (`mail.ActivityViewHeader`) muestra el nombre de cada tipo y, si el
  tipo tiene `template_ids` (plantillas de correo), un desplegable para enviar el
  email de plantilla. Incluye barra de progreso (`ColumnProgress`) por estado
  (planificado / hoy / atrasado).
- Cada celda (`mail.ActivityViewCell`) representa el grupo de actividades de ese
  registro y ese tipo: si hay estado, renderiza `ActivityCell` (con ids, fecha de
  vencimiento, conteo por estado, adjuntos, usuarios asignados); si está vacía,
  muestra un `+` para crear una actividad nueva.
- La celda izquierda de cada fila (`mail.ActivityRecord`) renderiza la plantilla
  QWeb `t-name` con el contexto del registro (`getRenderingContext()`).
- El pie (`mail.ActivityViewFooter`) ofrece el botón "Schedule activity".
- Si no hay tipos de actividad, se muestra `web.NoContentHelper`.

---

# 3. Diffs v18 → v19 (confirmados en fuente)

## Calendar

1. **Refactor del parser**: v18 lee los atributos dentro de un `case "calendar"`
   del recorrido `visitXML`; v19 los lee directamente del nodo raíz (`xmlDoc`)
   antes de recorrer los hijos. Resultado funcional equivalente para los
   atributos comunes.
2. **`recurrence_update` eliminado** de `FIELD_ATTRIBUTE_NAMES` en v19 (estaba en
   v18).
3. **Atributos nuevos en v19**: `aggregate`, `multi_create_view`,
   `show_date_picker` (default true), `month_overflow` (default true). No existen
   en el parser de v18.
4. **Default de escala**: v18 usa `sessionStorage` (`calendar-scale`) cayendo a
   `"week"`; v19 calcula la escala desde `scales` (prefiere `week`, si no el
   primero) y solo la sobreescribe con `mode`. v19 ya no consulta sessionStorage
   en el parser.
5. **`filtersInfo` con `context`**: v19 añade el campo `context` a cada entrada de
   filtro (tomado de `fieldInfo.context`); v18 no lo incluía.
6. **Autoasignación de avatar**: v18 tenía lógica para autoasignar
   `image_128` a filtros sobre res.users/res.partners/hr.employee; en v19 esa
   rama desaparece del parser (el `avatarFieldName` se toma solo del atributo).

## Activity

- **Sin cambios**: el `activity_arch_parser.js` es idéntico en v18 y v19 (mismos
  atributos `string` y `js_class`, misma lógica de `templateDocs`, `fieldNodes` e
  inyección de `write_date` por `activity_image`).
