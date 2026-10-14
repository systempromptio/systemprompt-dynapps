# Vistas Odoo: mecanismos comunes + diffs v17 -> v18 -> v19

Fichero transversal. Todo lo de aqui esta **verificado contra fuente** v18 y v19
(rutas locales `$ODOO_SRC/odoo18`, `$ODOO_SRC/odoo19`, `$ODOO_ENTERPRISE_SRC/19`).
Cada afirmacion cita el fichero:linea donde se confirmo. No hay invenciones: el
catalogo de widgets sale del registro real de la carpeta `views/fields/` de v19.

> Fuente única (SSOT) de los diffs **genéricos** de API por versión:
> `odoo-version-api-reference`. Aquí solo lo **específico de vistas** (no dupliques
> el catálogo general).

---

## 1. Herencia de vistas

El motor de herencia vive en `odoo/tools/template_inheritance.py` (funciones
`locate_node` y `apply_inheritance_specs`) y se invoca desde
`odoo/addons/base/models/ir_ui_view.py` (`apply_inheritance_specs`, `_combine`).
Mismas funciones en v18 y v19.

### 1.0. Cómo se ensambla el arch final: `get_combined_arch()`

Antes de renderizar (o de mostrar en el editor), Odoo **combina la vista con todas sus extensiones**:
- `get_combined_arch()` (`odoo/addons/base/models/ir_ui_view.py:933`, `View.get_combined_arch`, v19 `:1043`) sube por `inherit_id` hasta la raíz
  `primary` y aplica, en orden de `priority`, todas las vistas `mode='extension'` que la heredan.
- El conjunto de herederas lo calcula la CTE recursiva **`_get_inheriting_views()`** (`:621`): recoge
  extensiones **desde cualquier nodo** del árbol (no solo hijas directas de la raíz).
- **Branding** (`data-oe-id`/`data-oe-xpath`, lo que usan el editor web y Studio) lo añade
  `inherit_branding()` (`:775`) **solo si el contexto trae `inherit_branding=True`**; en render normal
  no se marca. Por eso un editor de chips que reconstruye el body no puede hacer un diff quirúrgico
  salvo que trabaje con branding.
- `apply_inheritance_specs` acepta la función XPath custom **`hasclass('page')`** (registrada en
  `odoo/addons/base/models/ir_ui_view.py:136-137`, `xpath_utils`): anclar por clase es más robusto que por posición.
- **La herencia se valida al `create`/`write` de la vista**, no al render: un xpath que no ancla lanza
  excepción al guardar. Por script, envuelve cada vista en `with cr.savepoint():` para no abortar el lote.

### 1.1. Como localiza el nodo a modificar

`locate_node(arch, spec)` decide donde aplicar el cambio segun la etiqueta del
nodo "spec" de la vista heredera
(`odoo/tools/template_inheritance.py:71-106`, `def locate_node`):

- **`<xpath expr="...">`**: evalua `expr` como XPath (`etree.ETXPath`). Devuelve el
  primer nodo que casa. Si falta `expr` -> `ValidationError`
  (`odoo/tools/template_inheritance.py:84-93`, `def locate_node`).
- **`<field name="x">`** (atajo): busca el primer `<field>` con ese `name` en
  cualquier nivel (`arch.iter('field')`). No hace falta xpath para un campo
  (`odoo/tools/template_inheritance.py:94-101`, `def locate_node`).
- **Cualquier otra etiqueta** (`<group>`, `<notebook>`, `<button>`, `<page>`,
  `<sheet>`, ...): casa el primer nodo de esa etiqueta cuyos atributos coincidan
  todos con los del spec (excepto `position`). Es el "atajo por atributos": p.ej.
  `<page name="extra" position="...">` localiza el `<page name="extra">` del
  padre (`odoo/tools/template_inheritance.py:103-105`, `def locate_node`).

### 1.2. `position` (valores validos)

El `position` se lee con `spec.get('position', 'inside')` -> por defecto `inside`
(`odoo/tools/template_inheritance.py:159`, `def apply_inheritance_specs`). Valores aceptados (cualquier otro lanza
`ValueError: Invalid position attribute`, `odoo/tools/template_inheritance.py:332-333`, `def apply_inheritance_specs`):

| position | Efecto | Fuente |
|---|---|---|
| `inside` (default) | Inserta el contenido del spec **dentro** del nodo localizado, al final | `odoo/tools/template_inheritance.py:312-318`, `def apply_inheritance_specs` |
| `after` | Inserta el contenido **justo despues** del nodo | `odoo/tools/template_inheritance.py:319-328`, `def apply_inheritance_specs` |
| `before` | Inserta el contenido **justo antes** del nodo | `odoo/tools/template_inheritance.py:329-330`, `def apply_inheritance_specs` |
| `replace` | **Reemplaza** el nodo. `mode="outer"` (def.) sustituye el nodo entero; `mode="inner"` sustituye solo el contenido (hijos), conservando el tag | `odoo/tools/template_inheritance.py:160-234`, `def apply_inheritance_specs` |
| `attributes` | Solo modifica atributos del nodo via `<attribute name="...">` (ver 1.3) | `odoo/tools/template_inheritance.py:235-311`, `def apply_inheritance_specs` |
| `move` | **No es un position de spec de nivel superior**: se usa dentro de un bloque `before`/`after`/`inside`/`replace` para **mover** un nodo existente del padre a esa posicion. Ver 1.4 | `odoo/tools/template_inheritance.py:45`, `add_stripped_items_before`, y `odoo/tools/template_inheritance.py:210-211`, `apply_inheritance_specs` |

Notas de `replace`:
- En `mode="outer"`, el texto especial `$0` dentro del spec se sustituye por una
  copia del nodo original (permite "envolver" el nodo)
  (`odoo/tools/template_inheritance.py:163-172`, `def apply_inheritance_specs`).
- En `mode="inner"` se usa un nodo centinela `<sentinel>` para poder reubicar
  hijos existentes con `position="move"` dentro del nuevo contenido
  (`odoo/tools/template_inheritance.py:214-232`, `def apply_inheritance_specs`).

### 1.3. `position="attributes"` (anadir/quitar/combinar atributos)

Dentro se ponen `<attribute name="X">valor</attribute>`. El parser solo permite
en `<attribute>` las claves: `name` (obligatoria), `add`, `remove`, `separator`,
y cualquiera que empiece por `data-oe-*`; lo demas lanza error
(`odoo/tools/template_inheritance.py:240-251`, `def apply_inheritance_specs`).

- Sin `add`/`remove`: el texto del nodo es el nuevo valor; si el texto es vacio,
  **se borra** el atributo (`odoo/tools/template_inheritance.py:305-311`, `def apply_inheritance_specs`).
- Con `add`/`remove` + `separator`: edicion incremental de un atributo de lista
  (clases CSS, etc.). `separator=" "` colapsa espacios; por defecto `,`
  (`odoo/tools/template_inheritance.py:293-304`, `def apply_inheritance_specs`).
- **Atributos que son expresion Python** (`readonly`, `required`, `invisible`,
  `column_invisible`, `t-if`, `t-elif`, y cualquier `decoration-*`): con
  `add`/`remove` el `separator` debe ser `and` u `or`, y combina con parentesis
  (`PYTHON_ATTRIBUTES`, `odoo/tools/template_inheritance.py:20`, `PYTHON_ATTRIBUTES`, y `odoo/tools/template_inheritance.py:267-292`, `apply_inheritance_specs`). Esto es lo que
  sustituye a la edicion de `attrs` de versiones antiguas.

### 1.4. `position="move"`

Permite reubicar un nodo ya existente del padre. Se escribe como hijo de un spec
con position `before`/`after`/`inside`/`replace`:

```xml
<xpath expr="//field[@name='foo']" position="after">
    <field name="bar" position="move"/>   <!-- mueve 'bar' aqui -->
</xpath>
```

El motor lo extrae del padre (`extract()` lo localiza y lo elimina de su sitio
original) y lo inserta en la nueva posicion
(`odoo/tools/template_inheritance.py:130-147`, `apply_inheritance_specs.extract`, y `odoo/tools/template_inheritance.py:210-211`, `apply_inheritance_specs`). El nodo movido no puede tener hijos
(`odoo/tools/template_inheritance.py:135-138`, `def extract`).

### 1.5. `<data>` como contenedor

Un `<data>` en la vista heredera no aplica nada: el motor mete sus hijos en la
cola de specs y los procesa uno a uno (`odoo/tools/template_inheritance.py:153-155`, `def apply_inheritance_specs`). Sirve
para agrupar varios specs en un mismo `arch`.

### 1.6. Herencia QWeb: `t-inherit` / `t-inherit-mode`

Para plantillas QWeb (no vistas de modelo) la herencia se declara con atributos
en la propia plantilla, no con el campo `inherit_id`:

- `t-inherit="modulo.plantilla_padre"`: id de la plantilla que se hereda.
- `t-inherit-mode="primary"` | `"extension"`: `primary` crea una nueva plantilla
  derivada (renderizable por si misma); `extension` (def. habitual) modifica la
  padre in situ.

El cuerpo usa los mismos `<xpath ... position="...">` que las vistas. El registro
y conversion de estas plantillas se procesa en la carga de datos
(`tools/convert.py`) y comparte el mismo `apply_inheritance_specs`.

### 1.7. Herencia: clave del campo `mode`

En `ir.ui.view`, `mode` es `primary` (vista raiz/standalone) o `extension`
(modifica a su `inherit_id`). El combinado se construye en `_combine`: las
extensiones se aplican antes que las vistas primary hijas
(`odoo/addons/base/models/ir_ui_view.py:967-1038`, `View`). Una vista con `inherit_id` pero sin que cambie el
tipo se trata como `extension`.

---

## 2. Visibilidad / estado por expresion (invisible, readonly, required, column_invisible)

### 2.1. Fin de `attrs=` y `states=` (desde 17.0)

Desde **17.0** los atributos `attrs` y `states` **ya no existen**. Si el arch
combinado contiene `@attrs` o `@states`, la validacion lanza error explicito:

> "Since 17.0, the "attrs" and "states" attributes are no longer used."

Fuente: `odoo/addons/base/models/ir_ui_view.py:492-498`, `IrUiView._check_xml` (v19) — mismo check en v18. Esto es un **error de
carga**, no un warning.

### 2.2. Sustitucion: atributos = expresiones Python directas

`invisible`, `readonly`, `required` y `column_invisible` se escriben ahora como
**expresiones Python** evaluadas en el cliente, directamente en el atributo. El
conjunto canonico de atributos-expresion esta en
`PYTHON_ATTRIBUTES = {'readonly', 'required', 'invisible', 'column_invisible',
't-if', 't-elif'}` (`odoo/tools/template_inheritance.py:20`, `PYTHON_ATTRIBUTES`).

```xml
<!-- v16 y anteriores (YA NO VALIDO) -->
<field name="x" attrs="{'invisible': [('state','=','done')]}"/>

<!-- v17 / v18 / v19 -->
<field name="x" invisible="state == 'done'"/>
<field name="y" readonly="state != 'draft'" required="amount &gt; 0"/>
```

> ⚠️ **Un `readonly` condicional descarta lo que el usuario escribió mientras el campo aún era
> editable.** Medido en Odoo 18 el 2026-08-15 con un tour: con la condición falsa se escribe el
> campo, se ve el valor en pantalla; se cambia el campo del que depende **sin guardar**, el
> modificador se re-evalúa en vivo y el campo pasa a readonly; al guardar, ese valor **no viaja**
> y se pierde sin ningún aviso. La causa: el cliente evalúa el `readonly` **en el momento de
> guardar** (`odoo18/addons/web/static/src/model/relational_model/record.js:635`, `Record._getChanges`), no cuando se
> escribió.
>
> El ejemplo de arriba —`readonly="state != 'draft'"`— es justo el patrón afectado: rellenar
> campos y confirmar en la misma sesión, sin guardar por medio, tira lo rellenado. Si el flujo se
> parece a eso, hay que guardar antes de que cambie la condición, o poner `force_save="1"` en el
> campo. Detalle y medición: la referencia `view-layer.md` de `odoo-owl-component-builder`.
>
> ⚠️ **Lo `invisible` NO se descarta**: se escribe y se guarda igual (medido el mismo día). La
> asimetría es real — ocultar un campo no impide que se escriba, y no sirve como protección.

- `invisible`: oculta el campo/nodo (en cualquier vista). En **listas**,
  `invisible` aplicado a la raiz se compila como `column_invisible`
  (`odoo/addons/base/models/ir_ui_view.py:1551`, `IrUiView._add_missing_fields`, v19 / `odoo/addons/base/models/ir_ui_view.py:1204`, `View._add_missing_fields` v18:
  `'invisible' if root.tag != 'list' else 'column_invisible'`).
- `column_invisible`: especifico de listas; oculta la **columna entera** (no
  depende del registro de la fila). Util cuando la condicion depende del contexto
  o del padre, no de cada linea.
- Cualquier `decoration-*` (decoration-danger, decoration-info, ...) tambien es
  expresion Python (`odoo/tools/template_inheritance.py:267`, `def apply_inheritance_specs`).

### 2.3. Contexto disponible en la expresion

Las expresiones se evaluan sobre el registro actual: se puede usar el **nombre de
cualquier campo presente en la vista** mas variables de contexto. Disponibles
tipicamente:

- Valores de los campos del registro (por su `name`).
- `uid`: id del usuario actual.
- `context`: el contexto de la accion/vista (`context.get('default_x')`, etc.).
- `parent`: en vistas embebidas (one2many/many2many), accede al registro padre
  (`parent.state`, ...).
- `id`: id del registro (False si es nuevo).

Operadores: sintaxis Python (`==`, `!=`, `in`, `not in`, `and`, `or`, `not`,
comparaciones numericas). En XML hay que escapar `<`/`>` como `&lt;`/`&gt;`.

### 2.4. Combinar estas expresiones por herencia

Para anadir/quitar condiciones por herencia sin reescribir toda la expresion, usa
`position="attributes"` con `add`/`remove` + `separator="and"|"or"` (ver 1.3):

```xml
<field name="x" position="attributes">
    <attribute name="invisible" add="state == 'cancel'" separator="or"/>
</field>
```

---

## 3. Seguridad en vistas: `groups`

- **Atributo `groups` en cualquier nodo**: restringe la visibilidad de ese nodo a
  los usuarios que pertenezcan a alguno de los grupos listados (xml_ids separados
  por coma). Aplica a `<field>`, `<button>`, `<page>`, `<group>`, `<menuitem>`,
  `<filter>`, etc. El motor mueve/elimina el contenido segun el grupo del usuario
  (`odoo/addons/base/models/ir_ui_view.py:1317-1407`, `IrUiView`, v19, metodo de aplicacion de restricciones por
  grupo).
- **`<t groups="...">`**: bloque transparente para agrupar varios nodos bajo una
  misma restriccion de grupo; el motor mueve el contenido del `<t>` y aplica el
  grupo a cada hijo (`odoo/addons/base/models/ir_ui_view.py:1347-1360`, `View._postprocess_tag_field`).
- Prefijo `!`: `groups="!base.group_no_one"` niega (oculta para ese grupo).
- **Restriccion en herencia**: una vista de herencia **no puede** declarar
  `groups` en el `<record>`/cabecera; debe ponerlo en los nodos del arch
  (`odoo/addons/base/models/ir_ui_view.py:542`, `View.write`: "Inherited view cannot have 'groups' defined on the
  record. Use 'groups' attributes inside the view definition").
- Relacion con seguridad de modelo: `groups` en la vista es solo presentacion;
  no sustituye a los `groups_id` / reglas de acceso del modelo.

---

## 4. Catalogo de widgets de campo (CE v19)

Generado a partir del **registro real** `registry.category("fields").add(...)` en
`$ODOO_SRC/odoo19/addons/web/static/src/views/fields/` (cada subcarpeta es un
widget; el nombre de carpeta **no siempre** coincide con el nombre de registro).
**68 carpetas de widget** en CE v19.

Aviso clave de nombres (carpeta -> nombre real para `widget="..."`):
`percent_pie` -> **`percentpie`**, `stat_info` -> **`statinfo`**,
`progress_bar` -> **`progressbar`**, `daterange` vive en la carpeta `datetime`
(registro `daterange`, `web/static/src/views/fields/datetime/datetime_field.js:590`, `daterange` (el `registry…add("daterange", dateRangeField)`, v19; la definicion en `:426`)), `ace` -> registra
**DOS** nombres, `ace` **y** `code` (mismo widget, `ace/ace_field.js:82-83`, `ace`).

Hay variantes con prefijo de vista (`list.*`, `kanban.*`, `form.*`,
`calendar.*`): el cliente elige la variante segun donde se use; en el arch se
pone el nombre base (p.ej. `widget="many2many_tags"`).

| Widget (`widget="..."`) | Tipo de campo | Uso | CE/EE |
|---|---|---|---|
| `badge` | char/selection/m2o | Etiqueta tipo pildora | CE |
| `boolean_toggle` | boolean | Interruptor on/off | CE |
| `boolean_favorite` | boolean | Estrella favorito | CE |
| `boolean_icon` | boolean | Icono on/off | CE |
| `handle` | integer (sequence) | Asa de arrastre para ordenar filas | CE |
| `monetary` | float/monetary | Importe con divisa | CE |
| `priority` | selection | Estrellas de prioridad | CE |
| `statusbar` | selection/m2o | Barra de estado (cabecera de formulario) | CE |
| `statinfo` | integer/float | Numero de boton estadistico (stat button) | CE |
| `percentpie` | float | Tarta de porcentaje | CE |
| `percentage` | float | Valor como porcentaje | CE |
| `progressbar` | float/integer | Barra de progreso | CE |
| `gauge` | float | Indicador tipo aguja (kanban) | CE |
| `dashboard_graph` | json | Mini-grafica de dashboard (journal) | CE |
| `image` | binary | Imagen desde binario | CE |
| `image_url` | char | Imagen desde URL | CE |
| `attachment_image` | m2o (ir.attachment) | Imagen de adjunto | CE |
| `contact_image` | binary/char | Imagen de contacto | CE |
| `signature` | binary | Firma manuscrita | CE |
| `email` | char | Enlace mailto + validacion | CE |
| `phone` | char | Enlace tel/sms | CE |
| `url` | char | Enlace clicable | CE |
| `html` | html | Editor HTML enriquecido | CE |
| `text` | text | Area de texto multilinea | CE |
| `ace` / `code` | text/char | Editor de codigo ACE (resaltado + auto-indent). Opcion `mode`: `python` / `javascript` / `qweb` / `xml` (default `qweb`; `xml`->`qweb`). Ej: `widget="ace" options="{'mode': 'python'}"` para un campo de codigo Python en un form | CE |
| `char` | char | Texto de una linea | CE |
| `password` | char | Texto enmascarado | CE |
| `copy_clipboard` (`CopyClipboardChar`/`CopyClipboardURL`/`CopyClipboardButton`) | char/url | Boton copiar al portapapeles | CE |
| `float_time` | float | Horas como HH:MM | CE |
| `float_factor` | float | Float con factor de conversion | CE |
| `float_toggle` | float | Float ciclico por boton | CE |
| `integer` | integer | Entero | CE |
| `float` | float | Decimal | CE |
| `remaining_days` | date/datetime | "Dentro de N dias / hace N dias" | CE |
| `daterange` | date/datetime | Rango de fechas (en carpeta datetime) | CE |
| `datetime` | datetime | Fecha y hora | CE |
| `selection` | selection/m2o | Desplegable de seleccion | CE |
| `selection_badge` | selection | Seleccion como pildoras (list/form) | CE |
| `badge_selection` (`selection_badge_with_filter`) | selection | Pildoras con filtro | CE |
| `label_selection` | selection | Etiqueta con color por valor | CE |
| `state_selection` | selection | Selector de estado con icono color | CE |
| `radio` | selection/m2o | Botones radio | CE |
| `priority` | selection | (ver arriba) | CE |
| `many2one` | many2one | Selector relacional estandar | CE |
| `many2one_avatar` | many2one | M2o con avatar | CE |
| `many2one_barcode` | many2one | M2o con escaneo de codigo de barras | CE |
| `many2one_reference` / `many2one_reference_integer` | reference/int | Referencia dinamica a otro modelo | CE |
| `reference` | reference | Campo referencia (modelo+id) | CE |
| `many2many_tags` | many2many | Etiquetas (tags) | CE |
| `many2many_tags_avatar` (`many2many_tags_avatar_popover`) | many2many | Tags con avatar | CE |
| `many2many_checkboxes` | many2many | Casillas de verificacion | CE |
| `many2many_binary` | many2many (ir.attachment) | Subida de varios ficheros | CE |
| `binary` | binary | Subir/descargar fichero | CE |
| `pdf_viewer` | binary | Visor de PDF embebido | CE |
| `google_slide_viewer` | char (url) | Visor de Google Slides | CE |
| `iframe_wrapper` | html/char | Contenido en iframe | CE |
| `domain` | char (domain) | Editor visual de dominio | CE |
| `field_selector` | char | Selector de campo/ruta | CE |
| `color` / `color_picker` / `kanban_color_picker` | integer/char | Selector de color | CE |
| `json` / `json_checkboxes` (`account_json_checkboxes`) | json | Edicion de JSON / checkboxes desde JSON | CE |
| `properties` (`property_tags`, `hierarchy.properties`, `calendar.properties`, `kanban.properties`) | properties | Campos dinamicos (Properties) | CE |
| `ace` (`code`, `code_ir_ui_view`, `ir_ui_view_ace`) | text | Editor de codigo ACE (XML/Python) | CE |
| `timezone_mismatch` | selection | Aviso de desfase horario | CE |
| `contact_statistics` | json | Estadisticas de contacto | CE |

> Para EE: la carpeta `web_enterprise/static/src/views/fields/` de v19 solo
> redefine `image` y `properties` (no aporta widgets nuevos genericos). Los
> widgets especificos de apps EE (sign, documents, account_asset, ai, ...) viven
> en su propio modulo (`.../static/src/...fields/`) y solo estan disponibles si el
> modulo esta instalado. Regla practica: los de la tabla anterior son **CE**
> (siempre disponibles con `web`); cualquier otro hay que comprobar el modulo.

---

## 5. Atributos comunes del nodo raiz de la vista

Leidos en `web/static/src/views/view.js` (v19):

- `js_class`: registra un componente de vista JS personalizado
  (`web/static/src/views/view.js:344-345`, `View.loadView`).
- `sample`: muestra datos de muestra cuando no hay registros
  (`web/static/src/views/view.js:356`, `View.loadView`, y `web/static/src/views/view.js:400-401`; expresion booleana).
- `create` / `edit` / `delete` (+ `group_create` / `group_delete` /
  `group_edit`): activan/desactivan acciones; `ACTIONS` en `web/static/src/views/view.js:191`, `ACTIONS`.
- `string`: titulo de la vista.
- En kanban ademas: `default_group_by`, `default_order`, `quick_create`,
  `on_create`, `quick_create_view`, `records_draggable`, `groups_draggable`,
  `archivable`, `can_open`, `limit`, `count_limit`, `highlight_color`
  (`kanban_arch_parser.js`).

(`banner_route` no aparece en el parser de vistas de `web` v18/v19; es propio de
otros contextos —p.ej. website/studio— y no se documenta aqui por no estar en
fuente del nucleo de vistas.)

---

## 6. TABLA MAESTRA: diffs v17 -> v18 -> v19

Todos confirmados en fuente. "Introducido en" = primera version donde aplica.

| Cambio | v17 | v18 | v19 | Fuente |
|---|---|---|---|---|
| `<tree>` -> **`<list>`** | tipo `list` ya en el Selection; se renombra el tag | `<list>` | `<list>` (Selection `[('list','List')...]`) | `odoo/addons/base/models/ir_ui_view.py:149`, `IrUiView.type` (v19), `:153`, `View.type` (v18) |
| `attrs=` / `states=` **eliminados** | error desde 17.0 | error | error | `ir_ui_view.py` v17 `:393`, v18 `:387`, v19 `:495` — mismo texto: *"Since 17.0, the 'attrs' and 'states' attributes are no longer used"* |
| `invisible/readonly/required/column_invisible` como **expr Python** | si (sustituye a attrs) | si | si | `odoo/tools/template_inheritance.py:20`, `PYTHON_ATTRIBUTES` |
| Chatter: `<div class="oe_chatter">` -> **`<chatter>`** | `selector: "div.oe_chatter"` | `selector: "chatter"` | `selector: "chatter"` | v17 `web/static/src/views/form/form_compiler.js:74`, `FormCompiler.createLabelFromField`; v18/v19 `chatter/web/form_compiler.js:46-47`, `chatter_compiler` |
| Kanban: `<t t-name="kanban-box">` -> **`<t t-name="card">`** | `kanban-box` | ambos (legacy `kanban-box` deprecado + `card` nuevo) | **solo `card`** (lanza error si falta) | v18 `web/static/src/views/kanban/kanban_arch_parser.js:9-28` (LEGACY_*); v19 `:8-9, 139-141` (sin LEGACY, throw `Missing 'card'`) |
| Kanban: `<t t-name="kanban-menu">` -> **`menu`** | `kanban-menu` | ambos | solo `menu` (`KANBAN_MENU_ATTRIBUTE`) | v18 `:26` / v19 `:9` |
| res.groups: `category_id` (ir.module.category) -> **`privilege_id`** (nuevo modelo `res.groups.privilege`) | category_id | category_id (`odoo/addons/base/models/res_users.py:191`, `class Groups`) | **privilege_id** (`odoo/addons/base/models/res_groups.py:36`, `class ResGroups`) + modelo `res_groups_privilege.py` | v18 `odoo/addons/base/models/res_users.py:191`, `class Groups`; v19 `odoo/addons/base/models/res_groups.py:36`, `class ResGroups`, `odoo/addons/base/models/res_groups_privilege.py:13-14`, `class ResGroupsPrivilege` |

Notas:

- **tree -> list**: NO convivian en v17. El Selection de `ir.ui.view.type` en v17
  tiene **solo** `('tree', 'Tree')` y ningun `list` (`odoo/addons/base/models/ir_ui_view.py:163`, `View.arch`); el
  renombrado entra en v18 (`:153`) y sigue en v19 (`:149`). O sea: `<list>` en v17
  no existe y `<tree>` en v18/v19 tampoco — el corte es duro en los dos sentidos,
  como el de kanban.
- **chatter**: el tag `<chatter>` es valido desde v18 (el compiler de v18 y v19
  usan `selector: "chatter"`). En v17 habia que usar
  `<div class="oe_chatter">`. Por tanto este es un diff **v17 -> v18**.
- **kanban-box -> card**: introducido en v18 (ambos soportados, `kanban-box`
  marcado como deprecado en el propio comentario del parser v18). En **v19 se
  elimina el fallback**: el parser exige la plantilla `card` y lanza error si no
  esta. Diff de sintaxis real en v18, ruptura dura en v19.
- **category_id -> privilege_id**: cambio estructural de v19. En v18 los grupos
  se agrupan por `category_id` (ir.module.category, "Application"); en v19 se
  introduce el modelo `res.groups.privilege` y `res.groups.privilege_id`
  (el privilege a su vez tiene `category_id`). Afecta a vistas/menus que filtran
  o muestran grupos.
- **Numero de widgets de campo CE**: v18 = 62 carpetas, v19 = 68 carpetas
  (`ls views/fields/*/`). v19 anade entre otros: `badge_selection`,
  `badge_selection_with_filter`, `boolean_favorite`, `boolean_icon`,
  `contact_statistics`, `copy_clipboard`.
