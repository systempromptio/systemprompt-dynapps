# Vista Kanban (Odoo CE) — Referencia verificada v18 / v19

Referencia practica de la vista **kanban** de Odoo Community, contrastada contra
el codigo fuente de v18 y v19. Foco especial en el **rework del templating**
(`kanban-box` antiguo vs `card` nuevo), porque es el punto donde mas falla la
gente al migrar.

Fuentes citadas (rutas reales):
- v19: `$ODOO_SRC/odoo19/addons/web/static/src/views/kanban/`
- v18: `$ODOO_SRC/odoo18/addons/web/static/src/views/kanban/`
- Ficheros clave: `kanban_arch_parser.js`, `kanban_compiler.js`,
  `kanban_record.js`, `kanban_renderer.js`, `kanban_controller.js`.

Convencion de versionado en este doc: cuando digo "v18" me refiero a 18.0 y
cuando digo "v19" a 19.0. La plantilla `card` se introdujo en **v18** (no en
v17: el parser de v17 solo conoce `kanban-box`, `web/static/src/views/kanban/kanban_arch_parser.js:25`, `LEGACY_KANBAN_BOX_ATTRIBUTE`, y `web/static/src/views/kanban/kanban_arch_parser.js:148-150`, `KanbanArchParser.parse`);
v18 acepta las dos y v19 convierte `card` en la unica forma valida (ver seccion
final de diffs). **Este doc esta verificado contra v18 y v19**; lo que diga de
v17 lleva su cita o no se afirma.

---

## TL;DR del templating (lo importante)

- **v19**: la plantilla `<t t-name="card">` es **OBLIGATORIA**. `kanban-box`
  **ya NO existe** — no hay capa de compatibilidad. Si falta `card`, el parser
  lanza `Error("Missing 'card' template.")`.
  Verificado: `web/static/src/views/kanban/kanban_arch_parser.js:139-142`, `KanbanArchParser.parse` (v19), no hay ninguna constante
  `LEGACY_*` en el fichero.
- **v18**: la plantilla recomendada es `card`, pero `kanban-box` **sigue
  soportado** como capa de compatibilidad (legacy). Si usas `kanban-box`, Odoo
  funciona pero emite `console.warn("'kanban-box' is deprecated, define a 'card'
  template instead")`.
  Verificado: `web/static/src/views/kanban/kanban_arch_parser.js:25-28`, `LEGACY_KANBAN_BOX_ATTRIBUTE`, y `web/static/src/views/kanban/kanban_arch_parser.js:150-160`, `KanbanArchParser.parse` (v18), constantes
  `LEGACY_KANBAN_BOX_ATTRIBUTE = "kanban-box"` y
  `LEGACY_KANBAN_MENU_ATTRIBUTE = "kanban-menu"`.
- Dato adicional: en v18 los propios addons del core **ya no usan** `kanban-box`
  en ningun `views/*.xml` (`grep -rl 'kanban-box' $ODOO_SRC/odoo18/addons/*/views`
  da 0 resultados). El soporte legacy existe solo para vistas custom/Studio sin
  migrar; el core esta 100% en `card`.

---

## Estructura de la vista

### Forma NUEVA (v18 / v19) — usar SIEMPRE

```xml
<kanban>
    <!-- Campos "sueltos" fuera de templates: se cargan en el dataset pero NO se
         pintan; sirven para agrupar, progressbar, dominios, group_by_tooltip... -->
    <field name="stage_id"/>
    <field name="color"/>

    <!-- Progressbar (opcional), va al nivel raiz, no dentro de la tarjeta -->
    <progressbar field="activity_state"
                 colors='{"planned": "success", "today": "warning", "overdue": "danger"}'/>

    <templates>
        <!-- Menu desplegable (3 puntos) de la tarjeta -->
        <t t-name="menu">
            <a role="menuitem" type="open" class="dropdown-item">Edit</a>
            <a role="menuitem" type="delete" class="dropdown-item">Delete</a>
            <div role="separator" class="dropdown-divider"/>
            <field name="color" widget="kanban_color_picker"/>
        </t>

        <!-- La tarjeta en si -->
        <t t-name="card">
            <field name="name" class="fw-bold fs-5"/>
            <footer class="pt-1">
                <field name="priority" widget="priority"/>
                <field name="user_id" widget="many2one_avatar_user" class="ms-auto"/>
            </footer>
        </t>
    </templates>
</kanban>
```

### Forma ANTIGUA (deprecada) — NO usar

```xml
<kanban>
    <templates>
        <t t-name="kanban-box">
            <div class="oe_kanban_global_click">
                <field name="name"/>
            </div>
        </t>
        <t t-name="kanban-menu"> ... </t>
    </templates>
</kanban>
```

Esto:
- En **v18** funciona pero avisa por consola que esta deprecado.
- En **v19** NO funciona: el parser no reconoce `kanban-box`, busca `card`, no
  lo encuentra y peta con `Missing 'card' template.`

### Sub-plantillas (`t-name`) reconocidas

El parser recorre el arch y guarda en `templateDocs` cualquier nodo con
`t-name` (`kanban_arch_parser.js`, bucle `visitXML`). Las que el runtime
realmente consume:

| `t-name`     | Que es                                  | v18 | v19 |
|--------------|-----------------------------------------|-----|-----|
| `card`       | La tarjeta (OBLIGATORIA en v19)          | si  | si  |
| `menu`       | Menu desplegable de la tarjeta           | si  | si  |
| `kanban-box` | Tarjeta legacy (deprecada)               | si (warn) | NO |
| `kanban-menu`| Menu legacy (deprecado)                  | si  | NO  |

Verificado:
- v19 `web/static/src/views/kanban/kanban_arch_parser.js:8-9` solo declara `KANBAN_CARD_ATTRIBUTE="card"` y
  `KANBAN_MENU_ATTRIBUTE="menu"`.
- v19 `web/static/src/views/kanban/kanban_record.js:179-197`, `KanbanRecord`: `showMenu = "menu" in templates`. No hay
  rastro de `kanban-menu` ni de `isLegacyArch`.
- v18 `web/static/src/views/kanban/kanban_record.js:195-217`, `KanbanRecord`: existe `isLegacyArch`, `menuTemplateName`
  conmuta entre `kanban-menu` (legacy) y `menu`, y la tarjeta se resuelve en
  `:375-377` eligiendo `kanban-box` o `card` segun `isLegacyArch`.

Nota sobre **multiples roots** en la tarjeta: el comentario de cabecera de
`kanban_arch_parser.js` (v18, lineas 8-23) documenta que `kanban-box` admitia
varios nodos raiz (cada uno generaba una card); se recomienda hacerlos
mutuamente excluyentes. En la API `card` lo habitual es un unico arbol con
`<t t-if>` internos.

NO confirmado en fuente: no existe una sub-plantilla `tooltip` parseada por el
arch_parser del kanban en v18 ni v19 (el tooltip de agrupacion se configura via
`group_by_tooltip` en options de un `<field>`, no via `t-name="tooltip"`). Si
buscas tooltips de cabecera de columna, es `group_by_tooltip`, no una plantilla.

---

## Atributos de `<kanban>`

Todos los siguientes estan **leidos por el arch_parser** salvo donde se indique.
Default = valor por defecto que aplica el parser.

| Atributo | Que hace | Ejemplo | v18 | v19 |
|----------|----------|---------|-----|-----|
| `class` | Clase CSS de la raiz del kanban | `class="o_kanban_small_column"` | si | si |
| `default_group_by` | Campo(s) de agrupacion por defecto | `default_group_by="stage_id"` | si | si |
| `default_order` | Orden por defecto de los registros | `default_order="sequence, id"` | si | si |
| `records_draggable` | Permite arrastrar tarjetas (default `true`) | `records_draggable="0"` | si | si |
| `groups_draggable` | Permite reordenar columnas (default `true`) | `groups_draggable="0"` | si | si |
| `group_create` | Mostrar boton "+ Columna" (default `true`) | `group_create="false"` | si | si |
| `group_delete` | Permitir borrar columnas (default `true`) | `group_delete="false"` | si | si |
| `group_edit` | Permitir editar columnas (default `true`) | `group_edit="false"` | si | si |
| `archivable` | Permitir archivar grupo entero (default `true`) | `archivable="false"` | si | si |
| `quick_create` | Habilita el quick-create inline (default `true`, requiere `create`) | `quick_create="false"` | si | si |
| `quick_create_view` | Form view a usar en el quick-create | `quick_create_view="crm.quick_create_opportunity_form"` | si | si |
| `on_create` | Comportamiento al crear (`quick_create` o XMLID de accion) | `on_create="quick_create"` | si | si |
| `create` / `edit` / `delete` | Acciones activas (via `getActiveActions`) | `create="false"` | si | si |
| `can_open` | Si la tarjeta abre el registro al click (default `true`) | `can_open="false"` | si | si |
| `sample` | Datos de muestra cuando no hay registros | `sample="1"` | si | si |
| `js_class` | Registra un controller/renderer JS custom | `js_class="crm_kanban"` | si | si |
| `limit` | Nº de registros por columna/pagina | `limit="20"` | si | si |
| `count_limit` | Tope para el contador de la cabecera (rendimiento) | `count_limit="100"` | si | si |
| `highlight_color` | Campo cuyo valor pinta el borde/color de la tarjeta | `highlight_color="color"` | si | si |
| `action` + `type` | Accion a abrir al click en vez del form (juntos) | `action="..." type="object"` | si | si |
| `examples` | Clave de ejemplos de columnas (dialog "configurar etapas") | `examples="project"` | si | si |

Verificado contra v19 `web/static/src/views/kanban/kanban_arch_parser.js:14-42` y v18 `:31-59`. `archivable`
sale de `getAttribute("archivable")` en ambas (`activeActions.archiveGroup`).

### Atributos que la gente cree que existen pero NO en el arch_parser

- **`groups_limit`**: NO es un atributo parseado de `<kanban>` ni en v18 ni en
  v19. En v19 solo aparece como constante interna del controller
  (`web/static/src/views/kanban/kanban_controller.js:319`, `KanbanController.onRecordSaved`, `groupsLimit: Number.MAX_SAFE_INTEGER`). No lo
  pongas en el XML esperando que limite columnas: el parser lo ignora.
- **`color`** (como atributo de `<kanban>`): el mecanismo de color de tarjeta en
  la API nueva es el atributo **`highlight_color`** apuntando a un campo. El
  viejo atributo `color="color"` del kanban clasico ya no es lo que pinta la
  tarjeta en v18/v19; usa `highlight_color`. Verificado: `cardColorField =
  xmlDoc.getAttribute("highlight_color")` en ambos arch_parsers, y
  `kanban_record.js` aplica `o_kanban_color_<idx>` a partir de
  `archInfo.cardColorField` (v19 `:259-261`).

---

## Dentro de la tarjeta (`<t t-name="card">`)

### Pintar campos

Dos formas, ambas validas:

1. `<field name="..."/>` — el compiler lo convierte en un `<span t-out=...>` con
   el valor formateado si NO tiene `widget`; si tiene `widget`, instancia el
   widget. Verificado: `kanban_compiler.js compileField`.
   - **Muchos2muchos**: si pones `<field name="tag_ids"/>` sin `widget`, el
     parser le pone automaticamente `widget="many2many_tags"` (v19
     `web/static/src/views/kanban/kanban_arch_parser.js:88-95`, `KanbanArchParser.parse`).

2. Acceso al objeto `record` en QWeb: `record.<campo>.value` (valor formateado /
   legible) y `record.<campo>.raw_value` (valor crudo del ORM). Verificado:
   `getFormattedRecord` (`web/static/src/views/kanban/kanban_record.js:95`, `getFormattedRecord` en v18; `:90` en v19) construye
   `formattedRecord[fieldName] = { value, raw_value }` para cada
   `record.fieldNames`, mas un `id` especial.

   ```xml
   <t t-if="record.expected_revenue.raw_value">
       <field name="expected_revenue" widget="monetary"/>
   </t>
   <span t-esc="record.partner_id.value"/>
   ```

   Importante: para usar `record.X.value`/`.raw_value` el campo debe estar
   cargado, es decir tener un `<field name="X"/>` en el arch (dentro o fuera de
   `templates`). Si no aparece, no esta en `record`.

### Atributos dinamicos y QWeb

Dentro de la card valen las directivas OWL del whitelist del compiler (v19
`web/static/src/views/kanban/kanban_compiler.js:173-188`, `KanbanCompiler.compileField`):
`t-name, t-esc, t-out, t-set, t-value, t-if, t-else, t-elif, t-foreach, t-as,
t-key, t-att.*, t-call`.

- `t-att-<attr>` / `t-attf-<attr>` para atributos calculados:
  ```xml
  <div t-att-class="record.is_late.raw_value ? 'text-danger' : ''"/>
  <div t-attf-style="width: #{record.progress.value}%"/>
  ```
- **OJO**: `t-att*` sobre un `<field>` NO esta soportado y lanza error explicito:
  `"t-att on <field> nodes is not supported"` (v19 `web/static/src/views/kanban/kanban_compiler.js:140-141`, `KanbanCompiler.compileField`).
  Pon el `t-att` en un wrapper (`div`/`span`), no en el propio `<field>`.

### Widgets tipicos en la tarjeta

Son widgets de campo estandar; los mas usados en kanban:
- `priority` — estrellas de prioridad.
- `many2one_avatar_user` / `many2one_avatar` — avatar del responsable.
- `many2many_tags` (con `options="{'color_field': 'color'}"`).
- `kanban_activity` — el reloj de actividades.
- `kanban_color_picker` — selector de color (tipico en `t-name="menu"`).
- `monetary`, `image`, `badge`, `boolean_favorite`, `handle` (drag manual).

`<widget name="..."/>` (component widgets, no de campo) tambien valen, p.ej.
`<widget name="web_ribbon" .../>` para la cinta de "Lost"/"Archived". Verificado
en ejemplo real CRM (v19 `crm/views/crm_lead_views.xml`).

### Color de tarjeta

- API nueva: atributo `highlight_color="<campo>"` en `<kanban>`. El runtime
  aplica la clase `o_kanban_color_<index>` calculando el indice a partir del
  valor del campo (`web/static/src/views/kanban/kanban_record.js:37`, v19; el uso, `:261`).
- Tambien hay tinte por progressbar: si el valor del registro cae en un color de
  la progressbar, la tarjeta recibe `oe_kanban_card_<color>` (v19
  `web/static/src/views/kanban/kanban_record.js:251-258`, `KanbanRecord.createWidget`).

### Menu de acciones (dropdown 3 puntos)

Se define con `<t t-name="menu">`. Dentro, los `<a>` con `type=` disparan
acciones especiales reconocidas por el compiler (v19 `web/static/src/views/kanban/kanban_compiler.js:19-28`, `ACTION_TYPES`,
`SPECIAL_TYPES`): `open`, `delete`, `url`, `set_cover`, `archive`, `unarchive`,
mas los `ACTION_TYPES` = `action`, `object`.

```xml
<t t-name="menu">
    <t t-if="widget.editable">
        <a role="menuitem" type="open" class="dropdown-item">Edit</a>
    </t>
    <t t-if="widget.deletable">
        <a role="menuitem" type="delete" class="dropdown-item">Delete</a>
    </t>
    <a role="menuitem" type="object" name="action_custom" class="dropdown-item">Hacer X</a>
    <div role="separator" class="dropdown-divider"/>
    <field name="color" widget="kanban_color_picker"/>
</t>
```

- `widget.editable` / `widget.deletable`: flags expuestos por
  `kanban_record.js createWidget` (`deletable` y `editable` segun activeActions y
  readonly). Verificado v19 `web/static/src/views/kanban/kanban_record.js:227-240`, `KanbanRecord`.
- Los botones `type="action"`/`type="object"` se **debouncean** 300ms
  automaticamente (v19 `web/static/src/views/kanban/kanban_compiler.js:54-60`, `KanbanCompiler.compileButton`).
- `set_cover` usa atributos extra `data-field` y `auto-open` (v19
  `web/static/src/views/kanban/kanban_compiler.js:63-69`, `KanbanCompiler.compileButton`) para fijar imagen de portada.

### Progressbar

Va al nivel raiz de `<kanban>` (no dentro de `card`). Atributos REALES leidos por
`parseProgressBar` (v18 `:197-205`, v19 `:178-186`):

| Atributo    | Que hace | Obligatorio |
|-------------|----------|-------------|
| `field`     | Campo (selection/estado) que define los segmentos | si |
| `colors`    | JSON `{valor_del_campo: color_bootstrap}` | si |
| `sum_field` | Campo numerico a sumar para el total mostrado | no |
| `help`      | Texto de ayuda del progressbar | no |

```xml
<progressbar field="activity_state"
             colors='{"planned": "success", "today": "warning", "overdue": "danger"}'
             sum_field="expected_revenue"
             help="Filtra por actividades programadas."/>
```

Colores validos = clases de estado Bootstrap (`success`, `warning`, `danger`,
`info`, `muted`, `primary`...). `colors` se parsea con `JSON.parse`, asi que debe
ser JSON valido (comillas dobles dentro, comillas simples fuera en el XML).

NO confirmado en core: atributos como `recurring_revenue_sum_field` que aparecen
en el kanban de CRM **no** los lee el `parseProgressBar` estandar — los anade el
`js_class="crm_kanban"` (extension JS de CRM). En un kanban CE sin js_class
custom, ceniete a `field`, `colors`, `sum_field`, `help`.

---

## Quick create (creacion rapida)

- Se activa cuando hay agrupacion y `quick_create` no esta a `false` (default
  `true`, pero requiere que `create` este activo). Verificado:
  `activeActions.quickCreate = activeActions.create && exprToBoolean(...
  quick_create, true)` (v18 `:46-47`, v19 `:26-27`).
- `quick_create_view="modulo.id_form"` define un form reducido para la creacion
  inline. Si no se especifica, Odoo usa un quick-create por defecto (solo
  `display_name` / campo de la columna).
- `on_create`:
  - `on_create="quick_create"` fuerza el flujo de quick-create al pulsar "Nuevo".
  - `on_create="modulo.action_xxx"` abre una accion/wizard en vez del form.

---

## Agrupacion y columnas

- `default_group_by` define la columna inicial.
  - **v18**: es un string simple (`getAttribute("default_group_by")`,
    `:36`).
  - **v19**: se hace `.split(",")` → soporta **agrupacion multinivel** desde el
    arch (`defaultGroupBy = ... split(",")`, v19 `:28-30`). Es uno de los diffs
    reales v18→v19.
- Botones de columna controlados por `group_create` / `group_delete` /
  `group_edit` / `archivable`.
- `<field name="X" options='{"group_by_tooltip": {"campo": "Etiqueta"}}'/>`
  añade tooltip al pasar por la cabecera de la columna agrupada por X
  (verificado: el parser guarda `tooltipInfo[name]` desde
  `fieldInfo.options.group_by_tooltip`, v19 `:104-106`).

---

## Drag & drop

- `records_draggable` (default `true`): arrastrar tarjetas entre/dentro de
  columnas. Reordenar dentro de columna requiere un campo `handle` (p.ej.
  `<field name="sequence" widget="handle"/>`); el parser detecta el handle y, si
  no hay `default_order`, ordena por `<handle>, id` (v19 `:145-148`).
- `groups_draggable` (default `true`): reordenar columnas (cuando el campo de
  agrupacion lo permite, p.ej. `stage_id`).

---

## `<header>` y `<control>`

- `<header>`: botones a nivel de vista (barra superior). El parser recoge
  `<button>` hijos como `headerButtons` (v19 `:53-61`).
  - Diff: en v18 los header buttons se filtran por `invisible` en el parser
    (`:78`); en v19 ese filtrado por `invisible` ya NO se hace en el parser
    (`:53-61`), se delega al render.
- `<control>`: personaliza la zona de creacion (botones / create / delete).
  - Diff de nombre interno: v18 lo devuelve como `creates`
    (`web/static/src/views/kanban/kanban_arch_parser.js:62`, `KanbanArchParser.parse`, y `web/static/src/views/kanban/kanban_arch_parser.js:174`); v19 lo renombra a `controls`
    (`:46,156`) y ademas el `<create>` admite `invisible` y `class`, y soporta un
    `<delete>` dentro de `<control>` (v19 `:69-83`). En v18 el `<create>` solo
    lee `context` y `string`.

---

## Diffs v17 → v18 → v19 (confirmados en fuente)

### Templating

| Aspecto | v17 | v18 (`card` se introduce aqui) | v19 |
|---------|-----|-------------------------------|-----|
| Plantilla tarjeta | **`card` NO existe**: el parser solo busca `kanban-box` (`web/static/src/views/kanban/kanban_arch_parser.js:25`, `LEGACY_KANBAN_BOX_ATTRIBUTE`, y `web/static/src/views/kanban/kanban_arch_parser.js:148`, `KanbanArchParser.parse`) y lanza `Missing 'kanban-box' template.` (`:150`) si falta | `card` recomendado | `card` **obligatorio** |
| `kanban-box` | **unica forma valida** | soportado con `console.warn` deprecado | **eliminado**, lanza `Missing 'card' template.` (`:141`) |
| `kanban-menu` (menu legacy) | unica forma | soportado | **eliminado** |
| `menu` (menu nuevo) | si | si | si |
| Capa de compat (`isLegacyArch`, `LEGACY_*`) | presente | **presente** (`web/static/src/views/kanban/kanban_arch_parser.js:25-28`, `LEGACY_KANBAN_BOX_ATTRIBUTE`, y `web/static/src/views/kanban/kanban_arch_parser.js:150-160`, `KanbanArchParser.parse`; `web/static/src/views/kanban/kanban_record.js:195-217,375-377`, `class KanbanRecord`) | **ausente** (no hay constantes ni ramas legacy) |
| Compiler legacy (clases `bold`/`display`, `params.isLegacy`) | si | si (`web/static/src/views/kanban/kanban_compiler.js:131-148`, `KanbanCompiler.compileField`) | **no** (bloque eliminado) |
| Ficheros legacy en el dir | — | `kanban_record_legacy.scss`, `kanban_color_picker_legacy.{js,xml,scss}` presentes | **eliminados** |

Resumen practico: **el corte esta entre v17 y v18, y es simetrico**. Una vista con
`card` funciona en v18/v19 y **revienta en v17** (`Missing 'kanban-box' template.`);
una vista con `kanban-box` funciona en v17/v18 (con warning en v18) y **revienta en
v19** (`Missing 'card' template.`). No hay una forma que valga para las tres. Migrar = renombrar `kanban-box`→`card`, `kanban-menu`→`menu`, y revisar
clases/markup porque la card nueva ya no necesita el wrapper
`oe_kanban_global_click` ni los helpers `bold`/`display` del legacy.

### Atributos / parser

| Cambio | v18 | v19 |
|--------|-----|-----|
| `default_group_by` | string plano | `.split(",")` → multinivel |
| `<control>` devuelto como | `creates` | `controls` (+ soporta `<delete>`, `invisible`, `class` en `<create>`) |
| Filtrado de `<header>` buttons por `invisible` | en el parser | fuera del parser |
| `highlight_color` (color de card) | si | si (igual) |
| `count_limit` | si | si |
| `groups_limit` como atributo XML | no (solo interno) | no (solo interno) |
| Comentario doc del modulo | incluye nota larga sobre `kanban-box` deprecado | comentario eliminado (ya no aplica) |

---

## Checklist de migracion a v19

1. Renombrar `t-name="kanban-box"` → `t-name="card"`.
2. Renombrar `t-name="kanban-menu"` → `t-name="menu"`.
3. Quitar el wrapper `<div class="oe_kanban_global_click">` (el click lo gestiona
   `can_open`/`o_kanban_record`; ya no hace falta el div manual).
4. Sustituir el viejo color por `highlight_color="<campo>"` en `<kanban>`.
5. Revisar `t-att-*` colocados sobre `<field>` (mover a un wrapper; en v19/v18
   peta con "t-att on <field> nodes is not supported").
6. Quitar atributos legacy de `<field>` tipo `bold` / `display="right|full"`
   (solo los procesaba el compiler legacy; en v19 se ignoran).
7. Verificar `colors` del progressbar como JSON valido.
