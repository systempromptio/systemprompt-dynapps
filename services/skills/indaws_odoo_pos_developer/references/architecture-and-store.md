# Arquitectura POS, app OWL, bundle y PosStore

> Citas `fichero:linea` verificadas contra v18/v19 en el entorno del autor. Resolver
> tu base de fuente via `references/shared/odoo-source.md`.

## El POS es una app OWL independiente

NO es el web client backend ni la tienda web. Es una SPA OWL propia montada en la
ruta `/pos/ui`. Punto de entrada: `static/src/app/main.js`.

- v18: `point_of_sale/static/src/app/main.js:24` `startPosApp()` monta el componente raiz `Chrome`
  (`@point_of_sale/app/pos_app`, importado en `point_of_sale/static/src/app/main.js:10`) con `mountComponent`
  (`point_of_sale/static/src/app/main.js:25`, `startPosApp` — la IIFE `(async function startPosApp() {`; el `await whenReady()` esta en `:32`). El loader se monta primero, dentro del `whenReady` de `point_of_sale/static/src/app/main.js:14-19`, `loader`.
- `window.posmodel` queda expuesto a la instancia de `PosStore` (util para debug en
  consola del navegador).
- v19 reorganiza: el raiz sigue en `app/pos_app.js`/`main.js`, pero servicios y store
  se mueven a `app/services/` (ver `version-diffs.md`).

## Bundle: declara tu glob (NO fichero a fichero)

El manifest declara el bundle `point_of_sale._assets_pos`
(`point_of_sale/__manifest__.py:129`) que incluye `static/src/**/*` por glob. Todo
JS/XML/SCSS que pongas bajo `static/src/` de tu modulo se carga automaticamente si tu
modulo anade su glob a ese bundle (patron estandar en modulos `pos_*`). NO listes los
ficheros uno a uno: rompe el orden de carga y duplica.

```python
# __manifest__.py de tu modulo pos_*
'assets': {
    'point_of_sale._assets_pos': [
        'mi_modulo/static/src/**/*',
    ],
},
```

## PosStore: el servicio central

`PosStore` es la clase con todo el estado y la logica del POS. Se registra como
**servicio** `"pos"`.

- v18: `static/src/app/store/pos_store.js`
  - `export class PosStore extends Reactive` (`:52`)
  - `static serviceDependencies = [...]` (`:56`)
  - `const posService = { dependencies: PosStore.serviceDependencies, ... }`
    (`:2393`) -> `registry.category("services").add("pos", posService)` (`:2399`)
- v19: `static/src/app/services/pos_store.js`
  - `export class PosStore extends WithLazyGetterTrap` (`:57`)
  - `static serviceDependencies = [...]` (`:73`)
  - `dependencies: PosStore.serviceDependencies` (`:2989`),
    `add("pos", posService)` (`:2997`)

### Extender PosStore

```js
import { patch } from "@web/core/utils/patch";
import { PosStore } from "@point_of_sale/app/store/pos_store"; // v18 path

patch(PosStore.prototype, {
    async setup(...args) {
        await super.setup(...args);
        // tu init
    },
    miMetodo() { /* ... */ },
});
```

Si tu metodo necesita un **servicio nuevo** (p.ej. `useService("mi_service")`),
NO basta con pedirlo: hay que anadirlo a `PosStore.serviceDependencies` (es el array
que alimenta `posService.dependencies`). Si no, el contenedor no lo inyecta y queda
`undefined`.

```js
PosStore.serviceDependencies = [...PosStore.serviceDependencies, "mi_service"];
```

### usePos() en componentes

`usePos()` (`point_of_sale/static/src/app/store/pos_hook.js:8`, `usePos` v18) devuelve `useState(useService("pos"))`: la
instancia envuelta en `useState` para que la plantilla reaccione a cambios. En un
componente OWL:

```js
import { usePos } from "@point_of_sale/app/store/pos_hook";
setup() { this.pos = usePos(); }
```

NUNCA uses `useService("pos")` a pelo en un componente: pierdes reactividad.

## Modelos JS (related_models)

El POS replica modelos del servidor como clases JS en `static/src/app/models/`
(motor `related_models.js`). Cada clase declara a que modelo Python corresponde y se
registra para que el cargador la instancie.

```js
// models/mi_modelo.js
import { registry } from "@web/core/registry";
import { Base } from "@point_of_sale/app/models/related_models";

export class MiModelo extends Base {
    static pythonModel = "mi.modelo";   // cf. product_product.js:16
}
registry.category("pos_available_models").add(MiModelo.pythonModel, MiModelo);
// cf. product_product.js:301
```

Esto es SOLO la mitad JS. La otra mitad (que el servidor envie ese modelo) esta en
`data-loading.md` (`_load_pos_data_models` + `_load_pos_data_fields`).

### Offline / IndexedDB

`models/data_service_options.js` define `databaseTable` (`:4`) y `dynamicModels`
(`:37`): los modelos persistidos en IndexedDB para operar sin red. `data_service.js`
los lee/escribe (`:106,128,163`). Si tu modelo nuevo debe sobrevivir a una recarga
sin conexion, declaralo ahi; si no, vive solo en memoria.

## Capa Python (alto nivel)

- `pos_config.py` — configuracion de cada POS (que metodos de pago, journals, etc.).
- `pos_session.py` — orquesta la carga de datos al abrir el POS (ver `data-loading.md`).
- `pos_order.py` / `pos_payment.py` — pedidos y pagos; `pos.order.sync_from_ui(orders)`
  (`point_of_sale/models/pos_order.py:1110`, `PosOrder.sync_from_ui` v18, `:1238` v19) es el punto de entrada al guardar desde el
  cliente.
- `pos_payment_method.py` — metodos de pago y `_get_payment_terminal_selection`
  (`:11`), clave para terminales.
- `pos_bus_mixin.py` — notificaciones push al cliente POS via bus (longpolling).
- `barcode_rule.py` — reglas de codigo de barras especificas del POS.
