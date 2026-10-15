# Vista List (antes tree) — Referencia completa Odoo CE v18/v19

> Fuente verificada: `list_arch_parser.js`, `list_renderer.js`, `list_controller.js`, `views/utils.js`, `fields/field.js` y vistas reales de addons estándar.

---

## `<tree>` vs `<list>` — Veredicto definitivo

| Aspecto | v18 | v19 |
|---|---|---|
| Tag root en archivos XML de addons | `<list>` (0 archivos con `<tree>`) | `<list>` (0 archivos con `<tree>`) |
| Tag que parsea `list_arch_parser.js` | `"list" === node.tagName` | `"list" === node.tagName` |
| `ir.ui.view` campo `type` (servidor) | `('list', 'List')` — no existe `'tree'` | `('list', 'List')` — no existe `'tree'` |
| Registro en `viewRegistry` | `registry.category("views").add("list", listView)` | igual |

**Conclusión:** `<tree>` está completamente eliminado en v18 y v19, tanto en el JS (arch parser solo detecta `"list"`), como en el servidor Python (tipo `'list'`), como en los XMLs de addons oficiales. No hay alias ni compatibilidad hacia atrás documentada. Usar siempre `<list>`.

---

## Atributos del elemento raíz `<list>`

Todos verificados en `list_arch_parser.js` bloque `"list" === node.tagName`.

### Atributos básicos

| Atributo | Tipo | Qué hace | Ejemplo |
|---|---|---|---|
| `string` | string | Título de la vista (nombre en selector de vistas, cabecera). No parseado en `treeAttr` pero leído por el framework general. | `string="Pedidos de venta"` |
| `class` | string | Clases CSS aplicadas al elemento raíz de la vista. | `class="o_sale_order"` |
| `sample` | `"1"` / `"0"` / bool-expr | Muestra datos ficticios cuando la lista está vacía. Se evalúa con `evaluateBooleanExpr`. | `sample="1"` |
| `js_class` | string | Sustituye el tipo `"list"` en el registro de vistas. Permite usar un controlador/renderer custom. Leído en `view.js`. | `js_class="account_moves_list"` |

### Edición en línea

| Atributo | Tipo | Qué hace | Ejemplo |
|---|---|---|---|
| `editable` | `"top"` / `"bottom"` | Activa edición inline. `"top"` inserta nuevas filas arriba, `"bottom"` abajo. Si está ausente la vista es solo lectura. | `editable="bottom"` |
| `multi_edit` | bool-expr | Permite editar el mismo campo en varias filas seleccionadas a la vez. Solo funciona si `editable` está presente y `activeActions.edit` es true. | `multi_edit="1"` |
| `open_form_view` | bool-expr | Solo cuando `editable`. Muestra un botón en cada fila para abrir el formulario completo. | `open_form_view="1"` |

### Acciones CRUD

Gestionados por `getActiveActions()` en `views/utils.js`. Por defecto todos `true`.

> **Son de PRESENTACIÓN, no permisos — medido en v18 y v19** (`tools/bench-orm::TestVistaNoEsPermiso`).
> El servidor manda estos atributos en el arch y **no comprueba ninguno al escribir**: con
> `create="0" edit="0" delete="0"`, un `create`/`write`/`unlink` por RPC funciona igual, y `web_save`
> —el camino real del cliente— escribe hasta un campo con `readonly="1"` en la vista. Quitan botones;
> la barrera son las ACL (`perm_write`, `perm_unlink`) y las record rules. Detalle en `form.md`.

| Atributo | Qué deshabilita | Ejemplo |
|---|---|---|
| `create` | Botón "Nuevo" y línea de creación | `create="0"` |
| `edit` | Edición inline (implica `multi_edit=false`) | `edit="0"` |
| `delete` | Opción "Suprimir" en menú Acción | `delete="0"` |
| `duplicate` | Opción "Duplicar". Solo aplica si `create="1"`. | `duplicate="0"` |
| `export_xlsx` | Botón exportar a Excel. Default `true`. | `export_xlsx="0"` |

#### Atributos nuevos en v19 (no existen en v18)

| Atributo | Qué controla | Default |
|---|---|---|
| `group_create` | Botón "Añadir grupo" en vista agrupada | `true` |
| `group_edit` | Editar nombre del grupo directamente | `true` |
| `group_delete` | Suprimir grupo completo | `true` |

```xml
<!-- v19 only -->
<list group_create="0" group_delete="0">
```

### Orden y paginación

| Atributo | Tipo | Qué hace | Ejemplo |
|---|---|---|---|
| `default_order` | string | Orden inicial. Sintaxis: `"campo [desc], campo2"`. Un `default_order` explícito **prevalece** sobre `widget="handle"` (el handle solo ordena si NO hay `default_order` — `web/static/src/views/list/list_arch_parser.js:235`, `ListArchParser.parse`). | `default_order="date_order desc, name"` |
| `limit` | entero | Número de registros por página. | `limit="80"` |
| `count_limit` | entero | Límite para el conteo total de registros (evita `SELECT COUNT(*)` costosos). | `count_limit="10000"` |
| `groups_limit` | entero | Número de grupos por página cuando la vista está agrupada. | `groups_limit="10"` |

### Agrupación y expansión

| Atributo | Tipo | Qué hace | Notas de versión |
|---|---|---|---|
| `default_group_by` | string | Campo(s) por los que agrupar al cargar. **v18:** string simple, un solo campo. **v19:** admite varios campos separados por coma (se convierte a array). | v18: `default_group_by="stage_id"` / v19: `default_group_by="stage_id,user_id"` |
| `expand` | `"1"` / `"0"` / bool-expr | Si los grupos se expanden automáticamente al cargar. Se pasa como `rawExpand` al modelo. | `expand="1"` |

### Comportamiento de apertura

| Atributo | Tipo | Qué hace | Ejemplo |
|---|---|---|---|
| `no_open` | bool-expr | Desactiva la navegación al formulario al hacer clic en una fila. | `no_open="1"` |
| `action` + `type` | string + string | Abre una acción personalizada al hacer clic en una fila (en lugar del formulario estándar). Ambos deben estar presentes. | `action="%(action_id)d" type="action"` |

### Decoraciones de fila

Ver sección dedicada más abajo.

---

## Columnas `<field>`

Atributos parseados en `Field.parseFieldNode()` y en `list_arch_parser.js`.

### Atributos básicos de campo

| Atributo | Qué hace | Ejemplo |
|---|---|---|
| `name` | Nombre del campo del modelo. Obligatorio. | `name="partner_id"` |
| `string` | Etiqueta de la columna. Sobreescribe el label del campo. | `string="Cliente"` |
| `widget` | Widget alternativo para renderizar el campo. | `widget="many2one_avatar_user"` |
| `options` | Objeto JSON con opciones para el widget. | `options="{'no_create': True}"` |
| `context` | Contexto adicional para el campo. | `context="{'default_type': 'out'}"` |
| `domain` | Dominio para campos relacionales. | `domain="[('active','=',True)]"` |

### Visibilidad de columna

| Atributo | Qué hace | Ejemplo |
|---|---|---|
| `column_invisible` | Expresión Python: oculta la columna entera (no ocupa espacio). Usa variables del contexto del registro padre o del `list`. Cuando es `True` también pone `invisible="True"`. | `column_invisible="parent.state == 'draft'"` |
| `invisible` | Oculta el widget dentro de la celda (la celda sigue existiendo). | `invisible="type == 'service'"` |
| `optional` | Columna ocultable por el usuario desde el menú de columnas opcionales. Valores: `"show"` (visible por defecto), `"hide"` (oculta por defecto), `"hidden"` (alias de `hide`, también usado en vistas reales). | `optional="hide"` |

**Comportamiento de `optional`:** El estado se persiste en `localStorage`. Si no hay valor guardado, `"show"` → visible, cualquier otro valor → oculto.

### Edición y restricciones

| Atributo | Qué hace | Ejemplo |
|---|---|---|
| `readonly` | Expresión Python o `"1"`. El campo no se edita en modo inline. | `readonly="state != 'draft'"` |
| `required` | Expresión Python o `"1"`. Campo obligatorio al editar. | `required="1"` |
| `force_save` | `"1"`: fuerza guardar el campo aunque sea readonly. | `force_save="1"` |

### Presentación

| Atributo | Qué hace | Ejemplo |
|---|---|---|
| `nolabel` | `"1"`: oculta la cabecera de columna (el `<th>`). | `nolabel="1"` |
| `width` | Ancho de la columna en píxeles o porcentaje (pasa a CSS). También rompe agrupación de botones cuando se usa en `<button>`. | `width="100px"` |
| `class` | Clases CSS en la celda. Soporta `oe_edit_only` y `oe_read_only`. | `class="oe_read_only"` |
| `groups` | XML id de grupo de seguridad. La columna solo es visible para miembros del grupo. Prefijo `!` para excluir. | `groups="stock.group_stock_user"` |
| `decoration-*` | Ver sección dedicada. | `decoration-bf="1"` |

### Agregados / totales (footer)

Aplican solo a campos numéricos (`integer`, `float`, `monetary`). El valor del atributo es el **texto del tooltip** que aparece en el footer.

| Atributo | Función | Ejemplo |
|---|---|---|
| `sum="Etiqueta"` | Suma total de la columna | `sum="Total"` |
| `avg="Etiqueta"` | Media de la columna | `avg="Media"` |
| `max="Etiqueta"` | Valor máximo | `max="Máximo"` |
| `min="Etiqueta"` | Valor mínimo | `min="Mínimo"` |

Solo uno de los cuatro puede usarse por campo. Para campos `monetary` con múltiples divisas, el sistema convierte automáticamente a la divisa activa de la empresa.

```xml
<field name="amount_untaxed" sum="Total Tax Excluded"/>
<field name="amount_total" sum="Total"/>
```

---

## `<button>` en filas

Los botones consecutivos sin atributo `width` se agrupan en una misma columna (`button_group`). Si un botón tiene `width`, rompe el grupo y arranca uno nuevo.

| Atributo | Qué hace | Ejemplo |
|---|---|---|
| `name` | Nombre del método (si `type="object"`) o id de acción (si `type="action"`). | `name="action_confirm"` |
| `type` | `"object"` (método Python), `"action"` (acción ventana), `"edit"` (abre formulario en edición). | `type="object"` |
| `string` | Texto del botón. | `string="Confirmar"` |
| `icon` | Clase FontAwesome. Si hay `icon` sin `string`, se muestra solo el icono. | `icon="fa-check"` |
| `title` | Tooltip al pasar el ratón. | `title="Confirmar pedido"` |
| `class` | Clases CSS. Default rank: `btn-link`. | `class="btn-primary"` |
| `invisible` | Expresión Python. Oculta el botón (la celda sigue). | `invisible="state != 'draft'"` |
| `column_invisible` | Expresión Python. Oculta la columna entera. En grupos de botones: la columna se oculta si **todos** los botones del grupo son `column_invisible` (combinación AND). | `column_invisible="not show_buttons"` |
| `context` | Contexto adicional. Default `"{}"`. | `context="{'active_test': False}"` |
| `options` | JSON con opciones. | `options="{}"` |
| `readonly` | Expresión Python. Botón deshabilitado. | — |
| `width` | Fija anchura y rompe agrupación con botones adyacentes. | `width="50px"` |
| `display` | Visibilidad contextual. Default `"selection"` (solo cuando hay filas seleccionadas). | — |

```xml
<button name="action_confirm" type="object" string="Confirmar"
        invisible="state != 'draft'" class="btn-primary"/>
<button name="action_cancel" type="object" icon="fa-times"
        title="Cancelar" column_invisible="not can_cancel"/>
```

---

## `<header>` — Botones sobre selección múltiple

Elemento hijo directo de `<list>`. Contiene `<button>` que aparecen en la barra de acción cuando hay filas seleccionadas. Solo soporta `<button>` como hijos.

```xml
<list>
    <header>
        <button string="Crear facturas"
                name="%(action_advance_payment)d"
                type="action"
                class="btn-secondary"/>
    </header>
    ...
</list>
```

---

## `<control>` — Fila de creación (editable inline)

Solo tiene efecto cuando `editable="top|bottom"`. Aparece como última fila, con enlaces para añadir registros. En v18 la variable interna se llamaba `creates`; en v19 se renombró a `controls`.

### Hijos de `<control>`

| Tag | Atributos | Qué hace | Versión |
|---|---|---|---|
| `<create>` | `string`, `context`, `invisible` | Enlace "Añadir X" con contexto opcional. `invisible` soportado desde v19. | v18+, `invisible` solo v19 |
| `<delete>` | `invisible` | Enlace "Eliminar" en la fila de control. | Solo v19 |
| `<button>` | atributos estándar de button | Botón personalizado en la fila de control. | v18+ |

```xml
<!-- v18/v19 -->
<control>
    <create string="Añadir un producto"/>
    <create string="Añadir una sección"
            context="{'default_display_type': 'line_section'}"/>
    <create string="Añadir una nota"
            context="{'default_display_type': 'line_note'}"/>
</control>

<!-- v19: control con delete e invisible -->
<control>
    <create string="Añadir línea" invisible="parent.state == 'done'"/>
    <delete invisible="not allow_delete"/>
</control>
```

---

## `widget="handle"` — Reordenamiento drag-and-drop

Campo especial que activa el drag-and-drop de filas. El campo del modelo debe ser de tipo `integer` (secuencia).

```xml
<field name="sequence" widget="handle"/>
```

- Si NO hay `default_order`, el handle ordena por `sequence, id`. Un `default_order` explícito tiene prioridad sobre el handle (`web/static/src/views/list/list_arch_parser.js:235`, `ListArchParser.parse`).
- Requiere que el modelo tenga el campo de secuencia correctamente definido.
- El renderer usa `.o_handle_cell` y `.o_row_draggable` para la interacción.

---

## `<groupby>` — Botones y campos en cabecera de grupo

Permite añadir botones y campos extra en la fila de cabecera de un grupo (cuando la vista está agrupada por un campo relacional). El atributo `name` debe ser un campo Many2one del modelo.

```xml
<groupby name="partner_id">
    <button name="edit" type="edit" icon="fa-edit" title="Editar"/>
</groupby>

<groupby name="move_id">
    <button name="action_post" type="object" string="Publicar"
            class="btn-primary" invisible="state != 'draft'"
            groups="account.group_account_user"/>
    <field name="state"/>
</groupby>
```

---

## `<widget>` — Columna widget

Columna que no corresponde a un campo del modelo sino a un componente JS registrado.

```xml
<widget name="web_ribbon" title="Archivado" bg_color="text-bg-danger"
        invisible="not active"/>
```

---

## Decoraciones (`decoration-*`)

Las decoraciones aplican clases CSS condicionales a **filas enteras** (en `<list>`) o a **celdas individuales** (en `<field>`). La condición es una expresión Python evaluada contra los datos del registro.

### En `<list>` (fila completa)

```xml
<list decoration-muted="state == 'cancel'"
      decoration-info="state == 'draft'"
      decoration-success="state == 'done'">
```

### En `<field>` (celda individual)

```xml
<field name="name" decoration-bf="1"/>
<field name="amount" decoration-danger="amount < 0" decoration-bf="amount > 1000"/>
```

### Todas las decoraciones soportadas

| Atributo | Clase CSS aplicada | Efecto visual |
|---|---|---|
| `decoration-bf` | `fw-bold` | Negrita (Bootstrap `fw-bold`) |
| `decoration-it` | `fst-italic` | Cursiva (Bootstrap `fst-italic`) |
| `decoration-danger` | `text-danger` | Texto rojo |
| `decoration-warning` | `text-warning` | Texto amarillo/naranja |
| `decoration-info` | `text-info` | Texto azul claro |
| `decoration-success` | `text-success` | Texto verde |
| `decoration-primary` | `text-primary` | Texto azul primario |
| `decoration-muted` | `text-muted` | Texto gris atenuado |

Fuente: `getClassNameFromDecoration()` en `views/utils.js`:
- `"bf"` → `"fw-bold"`
- `"it"` → `"fst-italic"`
- cualquier otro → `"text-${decoration}"`

El sistema acepta **cualquier** sufijo después de `decoration-` (no hay lista cerrada en el parser). Los nombres arriba son los usados en los addons oficiales.

### Evaluación de la condición

La condición es una expresión Python evaluada con el contexto del registro (`record.evalContextWithVirtualIds`). Tiene acceso a todos los campos del registro cargados en la vista. Para campos no cargados, la evaluación puede fallar silenciosamente.

```xml
<!-- Ejemplos reales de addons oficiales -->
<list decoration-info="state in ['draft', 'sent']"
      decoration-muted="state == 'cancel'">

<field name="virtual_available"
       decoration-danger="virtual_available &lt; 0"
       decoration-bf="1"/>
```

---

## Resumen de nodos hijo de `<list>`

| Nodo | Cuándo usar |
|---|---|
| `<field>` | Columna de datos del modelo |
| `<button>` | Botón de acción en cada fila |
| `<widget>` | Columna de componente JS |
| `<header>` | Botones para selección múltiple |
| `<control>` | Fila de creación (solo con `editable`) |
| `<groupby>` | Botones/campos en cabecera de grupo (solo con campo M2o) |

---

## Diffs confirmados v18 → v19

| Cambio | v18 | v19 | Fuente |
|---|---|---|---|
| Variable interna `<control>` | `creates` (array) | `controls` (array) | `list_arch_parser.js` línea 63 / 63 |
| `<delete>` dentro de `<control>` | No soportado | `{ type: "delete", invisible }` | `list_arch_parser.js` L180-183 |
| `<create invisible="">` | No soportado | Atributo `invisible` parseado | `list_arch_parser.js` L178 |
| `default_group_by` formato | String simple (un campo) | Array (split por `,`, múltiples campos) | `list_arch_parser.js` L207-209 |
| `group_create` / `group_edit` / `group_delete` | Ausentes | Nuevos en `activeActions`, default `true` | `list_arch_parser.js` L192-194 |
| `<tree>` en XMLs oficiales | 0 archivos (ya eliminado en v18) | 0 archivos | `grep -rl "<tree"` en addons |
| `<tree>` en `ir.ui.view` tipo | `('list', 'List')` sin `'tree'` | Igual | `ir_ui_view.py` L153 |
| Conteo de archivos `<list>` | 365 vistas | 378 vistas | grep sobre addons |
