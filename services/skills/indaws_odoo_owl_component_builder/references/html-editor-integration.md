# Integrar `@html_editor` en un componente OWL (client action / widget)

Gotchas **empíricos verificados** montando un editor de documento sobre `@html_editor`
(el Wysiwyg del web client, v18). No son teoría: cada uno costó una sesión de depuración.
Aplica cuando embebes el editor rico dentro de un componente propio (client action,
inspector, editor de plantillas).

## 1. NO uses `position:absolute;inset:0` en la raíz del client action

`position:absolute; inset:0` en la raíz del componente **rompe el montaje del
`@html_editor`** → papel en blanco (el editor no calcula bien su caja al montar).
- **Fix**: no posicionar en absoluto la raíz. Para el scroll, **acota las áreas
  scrollables** con `max-height: calc(100dvh - <alto de barras>px)` en los sub-paneles
  (documento/preview/fuente), no en la raíz.

## 2. `h-100` colapsa si la cadena de altura no está acotada

`h-100` de Bootstrap necesita que TODOS los ancestros tengan altura definida; si la
cadena se rompe, colapsa a 0 y el contenido largo se recorta sin scroll.
- **Fix**: mismo patrón que arriba — `max-height: calc(100dvh - Xpx)` + `overflow:auto`
  en el panel concreto. Es robusto y no depende de la cadena de ancestros.

## 3. En `onLoad`, `editor.editable` es `null`

El callback `onLoad` del Wysiwyg entrega el editor **antes** de que exista el DOM editable
→ `editor.editable === null`. Registrar listeners sobre `editor.editable` en `onLoad`
peta (`Cannot read properties of null`).
- **Fix**: en `onLoad` guarda solo la referencia al editor. Los listeners de clic sobre
  el contenido van por **burbujeo** en el contenedor OWL (`t-on-click` en el wrapper,
  p.ej. `.ird-sheet`), no sobre `editable`.

## 4. Odoo NO scopea el CSS de tus componentes

No hay CSS scoping por componente (no es Vue/Svelte). **`:deep()` no existe** y un
`.scss` suelto no se aísla: tus reglas afectan a todo el backend y las del backend te
afectan a ti.
- **Fix**: namespacea TODAS tus clases (`.o-mimodulo-*`) y declara el `.scss` en el
  bundle correcto. No asumas encapsulación.

## 5. Remontar el editor = cambiar su `t-key`

Para forzar un remonte limpio del Wysiwyg (p.ej. tras restaurar una versión / cambiar
de documento), incrementa una clave y pásala como `t-key` al componente editor. Odoo
lo destruye y reconstruye. Sin `t-key`, el editor conserva estado viejo.

## 6. Anchos de tabla dentro del editor (mismo gotcha que en wkhtml)

`text-nowrap` / `table-layout:auto` descuadran los anchos también **dentro del editor**
(las columnas se estiran). Reparto estable = `table-layout:fixed` + `width` por columna
(en toda la columna, no solo la cabecera). Ver `odoo-qweb-report-builder/references/wkhtmltopdf-gotchas.md §1`.

---

## S5 — Debug-in-the-UI cuando no tienes la consola del usuario

En cliente (navegador del usuario, sin acceso a su F12) no supongas: **instrumenta con
`notification.add()`** volcando valores reales.

```js
this.notification = useService("notification");
// ...
const rect = el.getBoundingClientRect();
this.notification.add(`ancho tabla=${Math.round(rect.width)}px cont=${Math.round(contW)}px`,
                      { type: "info" });
```
Convierte "creo que el ancho no llega" en un dato medido. Fue lo que localizó una
regresión de ancho de tabla que ningún razonamiento a priori explicaba.

## S6 — Reutilizar `ModelFieldSelector` para navegación de campos nativa

Para elegir un campo del modelo (path con relaciones, drill-down) NO reinventes el
selector: reutiliza `ModelFieldSelector` del core.

Import: `@web/core/model_field_selector/model_field_selector`.
Props (verificadas contra `web/static/src/core/model_field_selector/model_field_selector.js`, v18):

| Prop | Tipo | Nota |
|------|------|------|
| `resModel` | String | **requerido** — modelo raíz |
| `path` | (opcional) | path actual (`"partner_id.country_id.name"`) |
| `readonly` | Boolean (opt, default `true`) | ponlo `false` para editar |
| `showSearchInput` | Boolean (opt, default `true`) | buscador dentro del popover |
| `followRelations` | Boolean (opt, default `true`) | permite bajar por m2o/o2m |
| `update` | Function (opt) | `(path, fieldInfo, debug=false) => …` — callback al elegir |

```xml
<ModelFieldSelector resModel="'sale.order'" path="state.path"
    readonly="false" followRelations="true" showSearchInput="true"
    update="(path, fieldInfo) => this.onFieldPicked(path, fieldInfo)"/>
```

Hermano relacionado: `DynamicPlaceholderPopover`
(`@web/views/fields/dynamic_placeholder_popover`) para insertar placeholders — patrón
que usa Studio. Para mutar attrs de nodos ProseMirror en un editor 3rd-party (transacción
`setNodeMarkup`), ver `report-designer-developer/references/superdoc.md §S7`.

## Breadcrumb duplicado al abrir un client action

Si el `ir.actions.client` que devuelve un botón de un form usa `"name": self.name` (el
nombre del propio registro), la miga de pan se duplica ("X / X") y al salir queda un
residuo colgando. La miga padre ya muestra el registro: dale al action un `name` distinto
y estable (p.ej. `self.env._("Editor")`), no el nombre del registro.

Made with odoo-owl-component-builder · v0.1.0
