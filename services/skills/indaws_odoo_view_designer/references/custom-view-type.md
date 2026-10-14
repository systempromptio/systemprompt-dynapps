# Registrar un TIPO DE VISTA propio (v19)

Cómo **crear un view type nuevo** (no configurar uno existente — eso es el resto de refs). Patrón
idéntico al `map` de Enterprise (`web_map`) y al `hierarchy` de Community (`web_hierarchy`).
Verificado contra fuente v19 (2026-07): CE `odoo19/addons/web` + `web_hierarchy`, EE `web_map`.

Un view type = **dos mitades**: registro Python (para que el tipo exista en `ir.ui.view` y en las
acciones) + JS (Controller/Renderer/Model/ArchParser en el registry de vistas). Sin las dos, no
aparece el switcher.

## 1. Python — hacer que el tipo exista

**a) Añadir el valor a `ir.ui.view.type`** (Selection) + **override `_get_view_info`** (icono +
`multi_record`). La ETIQUETA sale de la tupla del `selection_add`; `_get_view_info` aporta el icono.

```python
# models/ir_ui_view.py
class IrUiView(models.Model):
    _inherit = 'ir.ui.view'

    type = fields.Selection(selection_add=[('mytype', "My Type")])

    def _get_view_info(self):
        return {'mytype': {'icon': 'fa fa-map-o', 'multi_record': True}} | super()._get_view_info()
```
Fuente: `web_hierarchy/models/ir_ui_view.py` (patrón exacto) y CE `addons/web/models/ir_ui_view.py`
(`get_view_info`/`_get_view_info`, base sin custom types). Si tu vista es **qweb-based** (usa
`<templates>`), override también `_is_qweb_based_view` para incluir tu tipo (como hace hierarchy).

**b) Añadir el tipo a `view_mode` de las acciones** (`ir.actions.act_window.view`), con
`ondelete` **cascade**:

```python
# models/ir_action.py
class IrActionsActWindowView(models.Model):
    _inherit = 'ir.actions.act_window.view'
    view_mode = fields.Selection(selection_add=[('mytype', "My Type")], ondelete={'mytype': 'cascade'})
```
Fuente: `web_map/models/ir_action.py` (`selection_add=[('map',"Map")], ondelete={'map':'cascade'}`).

**c) Validación del arch (opcional pero recomendado):** define `_validate_tag_mytype(self, node,
name_manager, node_info)` en el `_inherit` de `ir.ui.view` para validar hijos/atributos del arch
(patrón `_validate_tag_hierarchy` en `web_hierarchy`).

## 2. JS — el view en el registry

```js
// static/src/mytype_view/mytype_view.js
import { registry } from "@web/core/registry";
import { MyTypeController } from "./mytype_controller";
import { MyTypeRenderer } from "./mytype_renderer";
import { MyTypeModel } from "./mytype_model";
import { MyTypeArchParser } from "./mytype_arch_parser";

export const myTypeView = {
    type: "mytype",
    Controller: MyTypeController,
    Renderer: MyTypeRenderer,
    Model: MyTypeModel,
    ArchParser: MyTypeArchParser,
    multiRecord: true,
    // props recibe (genericProps, view, config) — 3 args, y no es novedad v19
    props: (genericProps, view, config) => {
        const parser = new view.ArchParser();
        const archInfo = parser.parse(genericProps.arch);
        return { ...genericProps, archInfo };
    },
};
registry.category("views").add("mytype", myTypeView);
```
Fuente: `web_map/static/src/map_view/map_view.js` (descriptor + `registry.category("views").add`);
**`props(genericProps, view, config)` son 3 args** (no 2), y el 3º NO es nuevo de v19: el
framework pasa los tres desde v17 (`web/static/src/views/view.js:362`, `View.loadView` v17, `:421` v18, `:423` v19 — todas
`descr.props(viewProps, descr, this.env.config)`). Ninguna vista del core declara el 3º, asi que
si lo copias de una, te llevas la firma de 2. El **Model extiende
`@web/model/model`** y lee datos con `this.orm.webSearchRead(resModel, domain, {...specification})`
(fuente `web_map/static/src/map_view/map_model.js`: `extends Model` de `@web/model/model`,
`orm.webSearchRead`).

**Cómo llega el tipo al cliente:** `ir_http.py` inyecta `get_view_info()` en `session.view_info`
(`addons/web/models/ir_http.py::session_info`). El webclient valida `type in session.view_info`
(`views/view.js`) y lee icon/display_name/multi_record de ahí (`action_service.js`). → por eso la
mitad Python (paso 1a) es la que hace aparecer el botón del switcher.

## GOTCHA CRÍTICO (cuesta ~1h) — caché IndexedDB `rpc` del webclient

**El webclient v19 cachea acciones y `get_views` en IndexedDB.** La caché es una db llamada
**`"rpc"`** (`new RPCCache("rpc", session.registry_hash, session.browser_cache_secret)` en
`web/static/src/start.js:36`, `startWebClient`, impl. en `core/network/rpc_cache.js`).

Síntoma: tras cambiar el `view_mode` de un action o el `arch` en desarrollo, **el cliente sigue
sirviendo el viejo** → cae a una lista auto-generada y **NO muestra el switcher del tipo nuevo, sin
error** (el `get_views` del servidor devuelve bien; es la caché del navegador). No es bug de tu
código.

Fix en la consola del navegador (y recargar **DOS veces** — la 1ª rompe el bus del worker-db en
vuelo):
```js
localStorage.clear(); sessionStorage.clear();
for (const d of await indexedDB.databases()) indexedDB.deleteDatabase(d.name);
```
En producción no aplica (el `registry_hash` cambia con el código y auto-invalida); es un dolor de
**iteración en dev** con el mismo hash.
