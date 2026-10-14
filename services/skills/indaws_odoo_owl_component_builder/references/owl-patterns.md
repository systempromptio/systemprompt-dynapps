# OWL Patterns — Hooks, Props/Slots, Registries (v17-v19)

Patrones del framework OWL aplicados al cliente web backend de Odoo. Firmas
exactas y diffs por version: `odoo-version-api-reference`.

## Hooks de OWL

Import desde `@odoo/owl` (lifecycle/estado) o `@web/core/utils/hooks` (helpers
de Odoo como `useService`, `useBus`, `useChildSubEnv`).

- **`useState(obj)`** — estado reactivo. Solo lo envuelto dispara re-render.
  Muta propiedades (`state.x = 1`), no reasignes el objeto entero.
- **`onWillStart(async () => {...})`** — carga ANTES del primer render. Aqui van
  los fetch de datos iniciales (`orm.read`, etc.).
- **`onMounted(() => {...})`** — el DOM ya existe. Para integraciones DOM (charts,
  focus, libs externas). No lo uses para datos; no devuelvas promesa.
- **`onWillUnmount(() => {...})`** — cleanup (listeners, timers).
- **`onWillUpdateProps(async (next) => {...})`** — reaccionar a props nuevas.
- **`useRef("name")`** — referencia a un nodo del template con `t-ref="name"`.
  Acceso real al DOM solo tras `onMounted` (`this.myRef.el`).
- **`useService("name")`** — inyecta un service (orm, action, dialog, notification,
  rpc, user, company, ...). SIEMPRE dentro de `setup()`.

Esqueleto tipico:
```js
setup() {
    this.orm = useService("orm");
    this.root = useRef("root");
    this.state = useState({ loading: true, records: [] });
    onWillStart(async () => {
        this.state.records = await this.orm.searchRead("my.model", [], ["name"]);
        this.state.loading = false;
    });
    onMounted(() => { /* DOM listo: this.root.el */ });
}
```

## Props y slots

- **Props** declaradas con `static props = {...}` (validacion en dev). Para field
  widgets, extiende `standardFieldProps`.
- **Slots**: contenido proyectado. Definir con `<t t-slot="default"/>` /
  `<t t-slot="footer"/>`; rellenar con `<t t-set-slot="footer">...</t>`.
- **`static components = { Child, Dialog }`** — declara sub-componentes usables en
  el template del padre.
- Comunicacion hijo→padre via callbacks en props (`props.onConfirm()`), no via
  eventos del DOM para logica de negocio.

## Gotchas de reactividad OWL

- **Granularidad de `useState`**: solo lo que esta dentro del proxy de `useState`
  es reactivo. Reasignar `this.state = {...}` rompe el binding; muta propiedades
  (`this.state.x = v`). Para colecciones, reemplaza el array entero
  (`state.rows = [...]`) — push muta in-place y SI re-renderiza porque el proxy
  intercepta `length`, pero reasignar es mas predecible.
- **No mutes `props`**: las props son del padre. Para escribir un field, usa
  `props.record.update({...})`; para comunicar al padre, llama un callback en props
  (`props.onChange(v)`). Mutar `props.foo` no propaga ni re-renderiza el padre.
- **`useSubEnv` vs `useChildSubEnv`**: ambos extienden `env` para descendientes.
  `useChildSubEnv` (preferido en Odoo, en `@web/core/utils/hooks` no — viene de
  `@odoo/owl`) crea un env nuevo SOLO para los hijos sin tocar el del componente
  actual; `useSubEnv` reemplaza tambien el `this.env` propio. Usa `useChildSubEnv`
  para inyectar contexto (p.ej. un bus local) a los hijos.
- **`useEffect(fn, () => [deps])`** (de `@odoo/owl`): corre tras el render cuando
  cambian las deps; el `fn` puede devolver una funcion de cleanup que corre antes
  del siguiente efecto y en unmount. Ideal para sincronizar con libs externas/DOM.
- **`useExternalListener(target, event, cb)`**: añade listener a un target FUERA
  del componente (`window`, `document`) con cleanup automatico en unmount. No uses
  `addEventListener` manual sin cleanup.
- **`onError(cb)`** (error boundary): captura errores de render/lifecycle de los
  hijos para no tumbar todo el cliente. El core lo usa en `ErrorHandler`; envuelve
  zonas riesgosas (widgets de terceros) en un componente con `onError`.

## Services del frontend (conexion a datos)

Catalogo real confirmado listando `web/static/src/core/*/` +
`grep category("services").add` (v19). Servicios mas usados al construir:

| Service | Para | Origen verificado |
|---------|------|-------------------|
| `orm` | CRUD/`call` a modelos | `core/orm_service.js` |
| `action` | `doAction`, `switchView`, `loadState` | `web/static/src/webclient/actions/action_service.js:1883` |
| `dialog` | `dialog.add(Component, props)` | `web/static/src/core/dialog/dialog_service.js:103`, `dialog` (registro del service; `:27` es la def del componente Dialog) |
| `notification` | `notification.add(msg, {type})` | `web/static/src/core/notifications/notification_service.js:71`, `add` |
| `rpc` | endpoints HTTP custom (no service en sí; ver abajo) | `web/static/src/core/network/rpc.js:93`, `rpc` |
| `bus_service` | `subscribe(type, cb)` / `start()` (longpolling) | `bus/static/src/services/bus_service.js:158`, `startWorker` |
| `ui` | bloqueo/activeElement, tamaño pantalla | `core/ui/ui_service.js` |
| `hotkey` | atajos de teclado | `core/hotkeys/hotkey_service.js` |
| `popover` | popovers anclados | `core/popover/popover_service.js` |
| `tooltip` | tooltips | `core/tooltip/tooltip_service.js` |
| `title` | titulo de pestaña del navegador | `core/browser/title_service.js` |
| `effect` | rainbow man / efectos | `core/effects/effect_service.js` |
| `command` | command palette | `core/commands/command_service.js` |
| `menu` | menus de apps | `webclient/.../menu_service.js` |
| `field` / `name` / `view` | metadata de campos/displayName/arch | `core/field_service.js`, `core/name_service.js` |
| `localization` | formatos l10n | `core/l10n/localization_service.js` |

- **`orm`** (detalle): `read`, `searchRead`, `searchCount`, `create`, `write`,
  `unlink`, `call(model, method, args, kwargs)`, `webSearchRead`, `readGroup`.
- **`rpc`**: NO se inyecta con `useService("rpc")`; importa la función:
  `import { rpc } from "@web/core/network/rpc"` y llama `await rpc(url, params)`.
- **`user` / `company` NO son services**: se importan
  `import { user } from "@web/core/user"` (campos `userId`, `name`, `context`,
  `isAdmin`, `hasGroup(g)`, `activeCompany`). En v18 `company` SÍ era service
  (`webclient/company_service.js`); en v19 desapareció → usa el import `user`.

Regla: para datos de modelos usa `orm`; reserva `rpc` para endpoints custom.

## Registries y categorias

`registry.category(name).add(key, value, { sequence })`. Categorias clave del
backend:

| Categoria | Para | Valor a registrar |
|-----------|------|-------------------|
| `fields` | field widgets | objeto descriptor `{component, supportedTypes, ...}` |
| `view_widgets` | widgets de vista (`<widget>`) | `{component, ...}` |
| `views` | vistas custom / `js_class` | `{...baseView, Controller, Renderer}` |
| `actions` | client actions | clase del Component |
| `systray` | iconos systray | `{Component}` (+ sequence) |
| `main_components` | overlays globales | `{Component}` |
| `services` | services | objeto `{dependencies, start}` |
| `user_menuitems` | menu de usuario | callback que devuelve item |

## Patrones comunes

- **Cargar antes de pintar**: datos en `onWillStart`, no en `setup` sincrono.
- **Loading state**: flag en `useState` + spinner en template (`t-if`).
- **Refrescar tras accion**: re-ejecuta el fetch en el callback, no manipules el
  DOM a mano.
- **t-on-click**: handlers como metodos del componente; evita logica inline larga.
- **t-if / t-foreach**: control de flujo en el template; `t-key` obligatorio en
  bucles para reconciliacion correcta.
- **Herencia de componentes estandar**: extiende la clase (`extends ListController`),
  llama a `super.setup()` y registra la variante en `views` con `js_class`.

## Surfaces avanzados (v18/v19, verificado contra fuente)

- **Streaming server→cliente** (progreso de un job, salida de un contenedor, notificación
  en vivo): el servidor empuja con `self.env['bus.bus']._sendone(target, notification_type,
  message)` (`bus/models/bus.py:110`, `ImBus._sendone`; `target` = un `res.partner`/canal no adivinable); el
  componente escucha con `bus_service.subscribe(notificationType, callback)`
  (`bus/static/src/services/bus_service.js:158`, `startWorker`). Emparéjalos por el mismo `notificationType`.
  Para un stream de alta frecuencia (terminal, logs) considera agrupar mensajes.
- **Contexto del controller actual** (saber sobre qué `res_model`/`res_id`/vista estás):
  `useService("action")` → la pila de controllers (`controllerStack[last]` en
  `action_service.js`) expone `currentController.action` y `.views`. Útil para un systray o un
  main_component que actúa sobre el registro/acción en pantalla sin recibirlo por props.
- **Vendorizar una lib de terceros (UMD → global)**: declara el bundle UMD en
  `web.assets_backend` (manifest `assets`); la lib se expone como global (`window.X`) y la
  importas en tu componente como global, no como módulo ES (ej. `xterm.js` para una terminal
  embebida). Cárgala en `onWillStart` si es pesada.

## Rendimiento

- **`t-key` en `t-foreach`**: OBLIGATORIO y único/estable (el id del registro, no
  el índice). Sin `t-key` correcto OWL reconcilia mal y reusa nodos equivocados,
  perdiendo estado de inputs/foco.
- **`t-out` vs `t-esc`**: `t-esc` escapa SIEMPRE (texto plano seguro). `t-out`
  renderiza markup si el valor es `markup(...)` (de `@odoo/owl`), si no escapa
  igual que `t-esc`. Usa `t-out` solo cuando necesitas HTML ya saneado; nunca
  inyectes HTML de usuario sin `markup` deliberado.
- **Sub-render**: aísla zonas que cambian mucho en sub-componentes con su propio
  `useState`; así un cambio de estado local re-renderiza solo ese sub-árbol, no
  todo el componente padre. OWL re-renderiza desde el componente cuyo estado
  cambió hacia abajo.
- **Evita trabajo pesado en getters de template**: los getters se ejecutan en cada
  render. Memoiza en `useState`/`onWillStart` lo costoso (formateos, agregados).
