# Vista Form — Referencia completa (v18 / v19)

> Verificado contra código fuente:
> - v19: `$ODOO_SRC/odoo19/addons/web/static/src/views/form/`
> - v18: `$ODOO_SRC/odoo18/addons/web/static/src/views/form/`
> - Chatter: `$ODOO_SRC/odoo19/addons/mail/static/src/chatter/web/form_compiler.js`
> - Settings: `$ODOO_SRC/odoo19/addons/web/static/src/webclient/settings_form_view/settings_form_compiler.js`

---

## 1. Elemento raíz `<form>`

El compilador (`form_compiler.js`, método `compileForm`) **no copia atributos XML** al div resultante (`doNotCopyAttributes: true`). Los atributos de `<form>` son leídos por `FormArchParser` y `getActiveActions`.

| Atributo | Qué hace | Ejemplo |
|---|---|---|
| `js_class` | Sustituye el componente JS de la vista | `js_class="sale_order_form"` |
| `string` | Título de la vista (cabecera del diálogo) | `string="Pedido de venta"` |
| `create` | Permite crear registros (default: `true`) | `create="0"` |
| `edit` | Permite editar (default: `true`) | `edit="0"` |
| `delete` | Permite borrar (default: `true`) | `delete="0"` |
| `duplicate` | Permite duplicar; hereda `create` (default: `true`) | `duplicate="0"` |
| `disable_autofocus` | Desactiva el autofoco en el primer campo | `disable_autofocus="1"` |

> Fuente: `form_arch_parser.js` líneas 9–11 y `web/static/src/views/utils.js` líneas 157–165.

```xml
<form string="Mi modelo" create="0" delete="0" js_class="mi_form">
  ...
</form>
```

---

## 2. `<header>` — Barra de estado

El compilador genera un `div.o_form_statusbar`. Los `<button>` dentro se envuelven en `StatusBarButtons`; los `<field>` (normalmente el widget `statusbar`) van al lado derecho.

```xml
<header>
  <button name="action_confirm" string="Confirmar" type="object" class="btn-primary"/>
  <button name="action_cancel" string="Cancelar" type="object"/>
  <field name="state" widget="statusbar" statusbar_visible="draft,confirmed,done"/>
</header>
```

### Atributos de `<button>` dentro de `<header>`

Los atributos se dividen en dos grupos según `BUTTON_CLICK_PARAMS` (`views/utils.js` líneas 19–42):

**Parámetros de click** (van al servidor o al framework):

| Atributo | Qué hace |
|---|---|
| `name` | Nombre del método o acción |
| `type` | `object` (método Python), `action` (ir.actions), `url` |
| `args` | Argumentos JSON para el método |
| `context` | Contexto adicional |
| `close` | Cierra el diálogo al terminar |
| `confirm` | Texto del diálogo de confirmación |
| `confirm-title` | Título del diálogo de confirmación |
| `confirm-label` | Etiqueta del botón de confirmación |
| `cancel-label` | Etiqueta del botón de cancelar |
| `special` | `save`, `cancel` |
| `effect` | Efecto visual (p.ej. `{"type": "rainbow_man"}`) |
| `debounce` | Tiempo mínimo entre clics (ms) |
| `block-ui` | Bloquea la UI con spinner durante la acción |
| `noSaveDialog` | No muestra diálogo de guardado en settings |

**Atributos string** (props del componente `ViewButton`):

`string`, `size`, `title`, `icon`, `id`, `disabled`

**Modificadores** (aplican a todos los nodos):

`invisible`, `readonly` (ver sección 6)

---

## 3. `<sheet>` — Hoja del formulario

Contenedor principal de contenido. El compilador genera `div.o_form_sheet_bg > div.o_form_sheet`. Los formularios **sin** `<sheet>` reciben la clase `o_form_nosheet`.

```xml
<sheet>
  <div name="button_box" class="oe_button_box">
    ...
  </div>
  <group>...</group>
  <notebook>...</notebook>
</sheet>
```

---

## 4. Smart buttons — `oe_button_box`

Se declara como un `<div name="button_box" class="oe_button_box">`. El compilador lo detecta por el selector `div[name='button_box']` y genera el componente OWL `ButtonBox`.

- En formularios **con** `<sheet>`: el button_box se mueve al panel de control (no aparece dentro del sheet). En diálogos no aparece.
- Cada hijo recibe automáticamente las clases `oe_stat_button btn btn-outline-secondary flex-grow-1 flex-lg-grow-0`.

```xml
<div name="button_box" class="oe_button_box">
  <button name="action_view_invoices" type="object" class="oe_stat_button" icon="fa-pencil">
    <field name="invoice_count" widget="statinfo" string="Facturas"/>
  </button>
</div>
```

**Visibilidad condicional** en smart buttons:

```xml
<button ... invisible="invoice_count == 0">...</button>
```

---

## 5. `<group>` — Grupos de campos

| Atributo | Qué hace | Default |
|---|---|---|
| `col` | Número máximo de columnas (`maxCols` del componente) | 2 |
| `string` | Título del grupo (genera un `<separator>` interno) | — |
| `colspan` | En hijo: cuántas columnas ocupa este item | 1 |

El compilador distingue entre **OuterGroup** (contiene otros `<group>`) e **InnerGroup** (contiene campos directos).

```xml
<group>
  <group string="Información general" col="2">
    <field name="name"/>
    <field name="ref" colspan="2"/>
  </group>
  <group string="Datos adicionales">
    <field name="date_order"/>
  </group>
</group>
```

### `<newline>` dentro de `<group>`

Fuerza un salto de línea al siguiente item, sin importar las columnas disponibles.

```xml
<group col="4">
  <field name="a"/>
  <field name="b"/>
  <newline/>
  <field name="c"/>  <!-- empieza en nueva fila -->
</group>
```

---

## 6. Modificadores de campo: `invisible`, `readonly`, `required`

**Desde v17**, `attrs=` y `states=` están **eliminados**. Los modificadores se escriben directamente como atributos con expresiones de dominio Python.

> Confirmado en `field.js` `parseFieldNode` (líneas 249–258 v19): solo itera `["invisible", "column_invisible", "readonly", "required"]`. No hay ninguna referencia a `attrs=` ni `states=` en el parsing XML.

| Atributo | Valor | Ejemplo |
|---|---|---|
| `invisible` | `"1"` / `"True"` / expresión Python | `invisible="state != 'draft'"` |
| `readonly` | ídem | `readonly="state == 'done'"` |
| `required` | ídem | `required="type == 'service'"` |
| `column_invisible` | Solo en `<tree>` y subtablas | `column_invisible="parent.state != 'sale'"` |

```xml
<!-- Antes (v16 y anterior) — YA NO VÁLIDO -->
<field name="discount" attrs="{'invisible': [('type', '=', 'service')]}"/>

<!-- Ahora (v17+) -->
<field name="discount" invisible="type == 'service'"/>
```

---

## 7. `<field>` en formulario — Atributos completos

| Atributo | Qué hace |
|---|---|
| `name` | Nombre del campo (obligatorio) |
| `widget` | Widget alternativo (p.ej. `many2many_tags`, `statusbar`) |
| `string` | Etiqueta personalizada |
| `help` | Tooltip de ayuda |
| `placeholder` | Texto de marcador de posición en edit |
| `nolabel` | `"1"` suprime la etiqueta automática en `<group>` |
| `readonly` | Expresión Python (ver sección 6) |
| `invisible` | Expresión Python (ver sección 6) |
| `required` | Expresión Python (ver sección 6) |
| `options` | Diccionario JSON de opciones del widget |
| `context` | Contexto adicional para campos relacionales |
| `domain` | Dominio adicional para campos relacionales |
| `default_focus` | `"1"` pone el foco inicial en este campo |
| `on_change` | `"1"` activa onchange manual |
| `force_save` | `"1"` fuerza guardar aunque el campo sea readonly |
| `decoration-*` | Solo en `<tree>`, **no aplica en form** |

```xml
<field name="partner_id"
       widget="many2one"
       string="Cliente"
       options="{'no_create': True}"
       domain="[('customer_rank', '>', 0)]"
       context="{'default_customer_rank': 1}"
       required="type == 'out_invoice'"
       readonly="state != 'draft'"
       placeholder="Buscar cliente..."/>
```

---

## 8. `<label>` — Etiqueta externa

Permite separar la etiqueta del campo cuando no se usa `<group>`.

| Atributo | Qué hace |
|---|---|
| `for` | Nombre del campo al que se vincula |
| `string` | Texto de la etiqueta (si vacío `""`, suprime la etiqueta) |
| `class` | Clases CSS |

```xml
<label for="name" string="Nombre del pedido"/>
<field name="name" nolabel="1"/>
```

---

## 9. `<separator>` — Separador horizontal

```xml
<separator string="Sección de pagos"/>
```

Admite `invisible` como cualquier otro nodo. Ocupa toda la fila en un `<group>` (se le asigna `itemSpan = maxCols`).

---

## 10. `<notebook>` y `<page>` — Pestañas

### `<notebook>`

| Atributo | Qué hace |
|---|---|
| `class` | Clases CSS del contenedor |

### `<page>`

| Atributo | Qué hace |
|---|---|
| `string` | Título de la pestaña (también sirve como fallback de `name`) |
| `name` | Identificador programático de la pestaña |
| `autofocus` | `"autofocus"` la selecciona por defecto en registros nuevos |
| `invisible` | Oculta la pestaña completa |
| `class` | Clases CSS de la pestaña |

```xml
<notebook>
  <page string="Información" name="info" autofocus="autofocus">
    <group>
      <field name="name"/>
    </group>
  </page>
  <page string="Notas" name="notes" invisible="type != 'service'">
    <field name="notes"/>
  </page>
</notebook>
```

> **v19**: el compilador registra los campos de cada página (`fieldNames`) para optimizar la carga diferida. Fuente: `form_compiler.js` líneas 580–584.
>
> **v18**: también soporta navegación por anclas `href="#id"` dentro del notebook (la misma feature existe en v18 y v19).

---

## 11. `<footer>` — Botones del diálogo

Solo relevante cuando el formulario se abre en diálogo. Se ubica **fuera de `<sheet>`**.

| Atributo | Qué hace |
|---|---|
| `replace` | `"0"` añade los botones por defecto (Guardar/Descartar) además de los personalizados |

```xml
<footer>
  <button name="action_validate" string="Validar" type="object" class="btn-primary"/>
  <button string="Cancelar" class="btn-secondary" special="cancel"/>
</footer>
```

---

## 12. Chatter v18+ — `<chatter>`

**Desde v18** (NO v17), el chatter se declara con la etiqueta `<chatter>`, que el módulo `mail`
registra vía el registry `form_compilers`. En **v17 hay que usar `<div class="oe_chatter">`**: su
compiler declara `selector: "div.oe_chatter"` y la etiqueta `<chatter>` no existe.

> Fuente: v17 `addons/mail/static/src/views/web/form/form_compiler.js:74`, `chatter_compiler` (la entrada `chatter_compiler` del registro `form_compilers`) → `selector: "div.oe_chatter"`.
> v18 `mail/static/src/chatter/web/form_compiler.js:45-48`, `chatter_compiler`, la entrada `chatter_compiler` del registro `form_compilers` y v19 `:47` → `selector: "chatter"`.
> (El fichero además cambia de sitio: `views/web/form/` en v17, `chatter/web/` en v18+.)

```xml
<!-- Sintaxis actual (v18+; en v17 usa <div class="oe_chatter">) -->
<chatter/>

<!-- Opcional: con atributos de recarga -->
<chatter reload_on_activity="1"
         reload_on_attachment="1"
         reload_on_follower="1"
         reload_on_post="1"
         open_attachments="1"/>
```

### Atributos de `<chatter>`

| Atributo | Qué hace | v18 | v19 |
|---|---|---|---|
| `reload_on_activity` | Recarga el registro al cambiar actividades | No | Sí |
| `reload_on_attachment` | Recarga al añadir/eliminar adjuntos | Sí | Sí |
| `reload_on_follower` | Recarga al actualizar seguidores | Sí | Sí |
| `reload_on_post` | Recarga al publicar mensaje | Sí | Sí |
| `open_attachments` | Abre el panel de adjuntos visible inicialmente | Sí | Sí |

> Nota: `has_activities` **no se escribe en XML**; lo inyecta `FormArchParser` del mail addon leyendo el metadato del modelo (`models[modelName].has_activities`).

El chatter **no se renderiza dentro de diálogos** (`t-if="!__comp__.env.inDialog"`).

---

## 13. Vista Settings (`res.config.settings`) — Estructura especial

La vista de settings usa `SettingsFormCompiler` en lugar de `FormCompiler`. Fuente: `settings_form_compiler.js`.

### Estructura jerárquica

```xml
<form>
  <app string="Ventas" name="sale_management" logo="/sale/static/description/icon.png">
    <block title="Pedidos" help="Configuración de pedidos de venta">
      <setting string="Política de facturación"
               title="¿Cuándo facturar?"
               help="Texto de ayuda en tooltip"
               documentation="/odoo/docs/link"
               company_dependent="1"
               id="sale_invoice_policy">
        <field name="group_sale_order_dates"/>
        <!-- Contenido adicional opcional -->
      </setting>
    </block>
  </app>
</form>
```

### `<app>`

| Atributo | Qué hace |
|---|---|
| `string` | Nombre de la app en el menú lateral |
| `name` | Clave del módulo (también URL del icono si no hay `logo`) |
| `logo` | Ruta al icono (default: `/{name}/static/description/icon.png`) |
| `notApp` | `"1"` oculta toda la sección (módulo técnico sin UI propia) |

### `<block>`

| Atributo | Qué hace |
|---|---|
| `title` | Título de la sección |
| `help` | Subtítulo o texto de ayuda |

### `<setting>`

| Atributo | Qué hace | v18 | v19 |
|---|---|---|---|
| `string` | Etiqueta principal del setting | Sí | Sí |
| `title` | Tooltip del signo `?` | Sí | Sí |
| `help` | Descripción extendida bajo el campo | Sí | Sí |
| `info` | Información adicional (nuevo) | No | **Sí** |
| `documentation` | URL de documentación (icono enlace externo) | Sí | Sí |
| `company_dependent` | `"1"` muestra indicador de dependencia por compañía | Sí | Sí |
| `id` | Identificador para búsqueda y anclas | Sí | Sí |
| `type` | `"header"` convierte en `SettingHeader` en lugar de `SearchableSetting` | Sí | Sí |

El primer `<field>` dentro de `<setting>` se trata como el campo principal (checkbox, select…). El resto del contenido es texto/HTML descriptivo.

---

## 14. `<widget>` — Widget standalone

Permite embeber componentes OWL sin un campo real del modelo.

```xml
<widget name="web_ribbon" title="Archivado" bg_color="bg-danger"
        invisible="active"/>
```

---

## 15. Diffs v18 → v19 (confirmados en fuente)

| Elemento | v18 | v19 | Fuente |
|---|---|---|---|
| **`<chatter reload_on_activity>`** | No existe (atributo ignorado) | Nuevo atributo `reload_on_activity` → prop `hasParentReloadOnActivityChanged` | `mail/.../form_compiler.js` línea 15 v19 |
| **`<chatter>` prop `record`** | `webRecord: "__comp__.props.record"` | `record: "__comp__.props.record"` (renombrado) | `mail/.../form_compiler.js` línea 21 v18 vs línea 22 v19 |
| **`<setting info=>`** | No existe | Nuevo atributo `info` pasado al componente `Setting` | `form_compiler.js` línea 597 v19 |
| **`<header>` sticky shadow** | `div.o_form_statusbar` sin shadow dinámico | Añade `t-att-class="{ 'shadow-sm': __comp__.state.isStatusbarStickyPinned }"` | `form_compiler.js` línea 443 v19 |
| **`<notebook>` fieldNames** | No trackea campos por página | Registra `fieldNames` por `<page>` para carga diferida optimizada | `form_compiler.js` líneas 580–584 v19 |
| **`<notebook>` `onWillActivatePage`** | No existe | Nuevo prop `onWillActivatePage` en el componente Notebook | `form_compiler.js` líneas 532–533 v19 |
| **`autofocusFieldId`** | Existe (legacy `autofocusFieldId` + array) | Eliminado el singular; solo queda array `autofocusFieldIds` | `form_arch_parser.js` v18 líneas 18–19 vs v19 |
| **`<header>` StatusBarDropdownItems** | Soportado (mobile, `asDropdownItems`) | **Eliminado** del compilador v19 | `form_compiler.js` v18 línea 463 — ausente en v19 |

### Diff más destacado: `webRecord` → `record` en `<chatter>` (v18→v19)

En v18 el prop del componente Chatter se llama `webRecord`; en v19 se renombra a `record`. Esto afecta a cualquier override o extensión del componente Chatter que acceda a ese prop.

```js
// v18  (mail/static/src/chatter/web/form_compiler.js línea 21)
webRecord: "__comp__.props.record",

// v19  (misma ruta, línea 22)
record: "__comp__.props.record",
```

## 16. Gotchas de resolución y visibilidad de formularios (v15+)

Trampas al depurar "no veo mi cambio en el formulario". Válidas en todas las versiones (comportamiento del ORM/cliente, no sintaxis).

- **Varias form NO-heredadas para el mismo modelo → compiten; gana `(priority ASC, id ASC)`.** Si dos módulos definen cada uno un `<record model="ir.ui.view">` con `<field name="model">` igual (form standalone, sin `inherit_id`) y **misma `priority`** (por defecto 16), al abrir sin `view_id` explícito Odoo elige la de **id más bajo** (la creada antes). La otra form queda "muerta" salvo que una acción la fuerce. Consecuencia: editas una form pero el usuario ve la otra.
- **Una acción con `view_ids`/`view_id` de tipo form fuerza esa form concreta**, saltándose la resolución por prioridad. Así un menú "operarios" puede usar una form distinta de la del menú general → el mismo modelo se ve con dos formularios según el menú. Al depurar, mira qué `act_window` abre la vista y si fija `view_id`.
- **Regla práctica:** si hay forms standalone duplicadas, **cualquier cambio de vista hay que replicarlo en TODAS** (o refactorizar a una base + `inherit_id`). Preferir herencia sobre duplicar formularios.
- **`groups=` en `<page>`/`<field>` oculta el elemento a quien no tenga el grupo, aunque el arch esté correcto.** Si "el campo/pestaña está en la vista pero no aparece", comprueba `groups=` en el nodo (o en un `<div>`/`<page>` padre) y si el usuario pertenece al grupo. **Admin NO ve grupos custom que no tenga asignados** (admin no es superusuario para `groups=` en vistas).
  **Quién lo quita, medido el 2026-08-15 en v18 y v19** (`tools/bench-orm`, `TestGroupsEnLaVista`):
  lo quita **el servidor**, no el render. `get_views` devuelve un arch **sin el nodo** para quien no
  tiene el grupo. Así que no se puede "verificar en BD que el arch combinado tiene el nodo": el arch
  que llega ya viene recortado, y por eso el valor tampoco se pide al leer esa vista.
- **`groups="..."` en un `<div>`** oculta todo su contenido; y `<attribute name="groups"></attribute>` (vacío) por herencia lo elimina.
- **`groups=` en la vista NO es una barrera de datos, y ni siquiera esconde que el campo existe.**
  Medido con un campo que solo lleva la restricción en la vista (el modelo no la tiene):

  | Lo que pide un usuario sin el grupo | Qué recibe |
  |---|---|
  | el `arch` de la vista | **sin el nodo** — el servidor lo recorta |
  | los campos que acompañan a la vista (`get_views` → `models[modelo]['fields']`) | **`solo_admin` incluido**, con su etiqueta y su tipo |
  | `read(['solo_admin'])` por RPC | **el valor**, sin error |

  O sea: quita el campo de *esa pantalla* y nada más. Para que el dato esté protegido de verdad, el
  `groups=` va **en el campo del modelo** — y aun así, en v18 el acceso por atributo lo devuelve
  (`odoo-security-designer/references/access-model.md`; v19 lo cerró).
- **`create="0"`, `edit="0"`, `delete="0"` y `readonly="1"` los cumple el NAVEGADOR, no el servidor
  — MEDIDO** (v18 y v19, `tools/bench-orm` → `TestVistaNoEsPermiso`). El servidor **manda esos
  atributos tal cual** en el arch (no los aplica: los delega) y no comprueba ninguno al escribir:

  | Con la vista puesta a `create="0" edit="0" delete="0"` | Qué pasa por RPC |
  |---|---|
  | `create(...)` como usuario normal | **crea** |
  | `write(...)` | **escribe** |
  | `unlink()` | **borra** |
  | `web_save({'campo_readonly': 'x'})` — el camino real del cliente al guardar | **lo escribe** |

  `web_save` es un `write`/`create` pelado (v18 `addons/web/models/models.py:68-74`, `Base.web_save`): no mira
  modificadores de vista. Por eso el `readonly` de la vista se nota —el navegador no manda el valor,
  y por eso *se pierde lo que escribiste antes de que el campo se bloqueara*— pero **cualquier otra
  cosa que hable ese mismo RPC lo escribe**: un script, un `curl`, otro cliente, o el propio
  navegador si el modificador se evalúa distinto.

  Es lo mismo que ya estaba medido para `groups=`: **la vista decide qué se pinta; los permisos
  viven en ACL y record rules** (`odoo-security-designer`). Si el requisito es "este perfil no puede
  borrar", `delete="0"` es cosmética — el `perm_unlink` de la ACL es la barrera.
- **El cliente web cachea el arch tras un `-u`.** Tras actualizar el módulo, la BD tiene el arch nuevo pero el navegador puede seguir mostrando la vista vieja → **hard refresh (Ctrl+F5)**. Si dudas de si el cambio está aplicado, contrasta con `fields_view_get`/el `arch_db` del `ir.ui.view` en BD antes de tocar código.
