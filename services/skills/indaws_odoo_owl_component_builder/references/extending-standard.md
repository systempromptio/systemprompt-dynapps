# Extender componentes, services y field widgets estándar — `patch()` (v17-v19)

Para modificar piezas estándar del web client **sin reemplazarlas** se usa
`patch` (`@web/core/utils/patch`). Es el mecanismo central de extensión en Odoo
moderno. Firma: `const unpatch = patch(objToPatch, extension)` → devuelve la
función de unpatch.

## Componente OWL estándar (controller, renderer, widget, dialog…)
Se parchea el `prototype` de la clase:
```js
/** @odoo-module **/
import { patch } from "@web/core/utils/patch";
import { FormController } from "@web/views/form/form_controller";

patch(FormController.prototype, {
    setup() {
        super.setup();            // SIEMPRE llama al original
        // lógica extra
    },
    async saveButtonClicked(params) {
        const res = await super.saveButtonClicked(params);  // llama al original
        // post-proceso
        return res;
    },
});
```
- El original se invoca con **`super.metodo(...)`** dentro del método parcheado:
  la `patch` moderna reconstruye la cadena de prototipos. El viejo
  `this._super.apply(this, arguments)` es **solo** widgets legacy (no OWL).
- Omitir `super.setup()` rompe la inicialización del componente.

## Service estándar
- Service definido como objeto `{ dependencies, start }` → patch del objeto:
  ```js
  import { patch } from "@web/core/utils/patch";
  patch(someService, { /* sobrescribe start o helpers */ });
  ```
- Si el service expone una clase, patch de su `prototype` (igual que un componente).

## Field widget existente
1. **Variante nueva** (recomendado): registra con clave propia y úsala con
   `widget="mi_char"`. No toca el estándar.
2. **Extender el componente**: `patch(CharField.prototype, { ... })`.
3. **Override total** re-registrando la MISMA clave con `{ force: true }`:
   ```js
   registry.category("fields").add(
       "char",
       { ...charField, component: MyCharField },
       { force: true },
   );
   ```
   Sin `{ force: true }` **no es que se ignore: revienta**. `registry.add()` lanza
   `DuplicatedKeyError` (`odoo18/addons/web/static/src/core/registry.js:100`, `Registry.validateSchema`) y el
   fichero deja de ejecutarse ahí, así que se pierde también todo lo que registrara
   más abajo.

## Override de la plantilla XML de un componente estándar
Hereda la plantilla con `t-inherit` + `xpath`, declarada en TU bundle:
```xml
<t t-name="mi_modulo.CharFieldExt" t-inherit="<t-name real del componente>"
   t-inherit-mode="extension">
    <xpath expr="//input" position="attributes">
        <attribute name="placeholder">...</attribute>
    </xpath>
</t>
```
Busca el `t-name` real en el `.xml` del componente estándar antes de heredar.

## Unpatch en tests
`patch` devuelve la función de unpatch: guárdala y llámala en el teardown para
no contaminar otros tests.
```js
const unpatch = patch(X.prototype, { ... });
// ... test ...
unpatch();
```

## GOTCHAS
1. **Olvidar `super.setup()`** (o el `super.metodo()` correspondiente) → pieza rota.
2. **Re-registrar sin `{ force: true }`** → `DuplicatedKeyError` y el módulo JS entero
   no carga (consola: *"The following modules failed to load"*). No es que el override
   no aplique: no aplica nada de ese fichero.
3. **Patch sin unpatch en tests** → fugas de estado entre tests.
4. **Clase vs objeto**: clase → `.prototype`; service objeto `{start}` → el objeto.
5. `this._super` NO existe en OWL: es solo del widget legacy. Usa `super.x()`.
