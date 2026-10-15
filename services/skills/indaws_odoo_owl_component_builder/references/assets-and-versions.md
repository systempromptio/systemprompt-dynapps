# Assets & Version Diffs (v17 → v19)

Declaracion de assets, estructura de ficheros estaticos y diferencias del
framework JS entre v17, v18 y v19. Firmas/diffs exhaustivos:
`odoo-version-api-reference`.

## Bundles de assets en el manifest

Los componentes del backend van en `web.assets_backend`. En `__manifest__.py`:
```python
"assets": {
    "web.assets_backend": [
        "mi_modulo/static/src/**/*.js",
        "mi_modulo/static/src/**/*.xml",
        "mi_modulo/static/src/**/*.scss",
    ],
},
```
Notas:
- El XML de plantillas OWL se declara en el MISMO bundle que el JS; no se carga
  solo por estar en `static/src`.
- SCSS idem: si no esta en el bundle, no compila.
- Glob `**/*` recorre subcarpetas; lista ficheros sueltos cuando importe el orden.
- Operaciones de bundle disponibles: append (string), `("prepend", path)`,
  `("before", ref, path)`, `("after", ref, path)`, `("remove", path)`,
  `("replace", ref, path)`.

Otros bundles utiles:
- `web.assets_backend_lazy` — carga diferida (v18+) para piezas pesadas.
- `web.assets_frontend` — website/portal (NO es alcance de esta skill).
- `web.assets_qweb` — desaparecido; el XML OWL va en el bundle JS desde v16+.
- `web.assets_unit_tests` / `web.tests_assets` — tests (Hoot/QUnit).

## Estructura de ficheros estaticos

```
mi_modulo/
├── __manifest__.py
└── static/
    └── src/
        ├── fields/
        │   ├── color_field.js
        │   └── color_field.xml
        ├── views/
        ├── client_actions/
        ├── systray/
        ├── components/      # componentes reutilizables
        ├── services/
        ├── hooks/
        └── scss/
```
Un fichero JS por componente, su `.xml` junto a el. Plantillas namespaceadas
(`mi_modulo.MiWidget`).

## Diffs del framework JS

### v17
- `/** @odoo-module **/` obligatorio para que el fichero SEA un módulo: en v17 el
  comentario es el **único** criterio — `is_odoo_module(content)` mira solo el
  contenido (`odoo/tools/js_transpiler.py:675`, `is_odoo_module`, llamado desde
  `addons/base/models/assetsbundle.py:850`, `JavascriptAsset.with_header`). Sin comentario, script plano. En la
  práctica lo lleva casi todo: 440 de 446 JS de `web/static/src`.
- Field widgets ya en formato objeto descriptor (`{component, supportedTypes,
  extractProps}`).
- Tests JS con **QUnit** (`web.tests_assets`).
- OWL 2 estable; imports desde `@odoo/owl` y `@web/...`.

### v18
- **El comentario `@odoo-module` deja de decidir: decide la RUTA.** No es que
  "ya no haga falta" — es que el criterio cambió de lado y eso tiene dos
  consecuencias que la versión suave se calla. `is_odoo_module` pasa a recibir la
  url (`odoo/tools/js_transpiler.py:675-689`, `def is_odoo_module`, v18; desde `odoo/addons/base/models/assetsbundle.py:813`, `JavascriptAsset.is_transpiled` en v18 y `:818` en
  v19) y devuelve `True` para **todo** lo que cuelgue de `/<addon>/static/src` o
  `/<addon>/static/tests`, con comentario o sin él:
  1. Un script plano que dejes bajo `static/src` **se transpila igual**, lo
     quieras o no. En v17 bastaba con no poner el comentario; en v18 ya no.
  2. Para salirte hay una fórmula nueva, `// @odoo-module ignore`, que el propio
     Odoo usa en 12 ficheros CE18 y 13 CE19 (librerías embebidas como
     `spreadsheet/static/src/o_spreadsheet/o_spreadsheet.js`). En v17 no existe:
     0 usos.
  Y **el transpilador no ha muerto**: `js_transpiler.py` sigue vivo en v19
  (`:713`). Poner el comentario en v18/v19 no molesta, pero no es lo que hace que
  tu fichero sea un módulo.
- Migracion del runner de tests a **Hoot** (reemplaza QUnit progresivamente);
  bundle `web.assets_unit_tests`.
- `web.assets_backend_lazy` para carga diferida.
- **Mueren dos services que en v17 sí existían** (verificado en CE17/CE18):
  `user` (`core/user_service.js`, solo v17) → `import { user } from "@web/core/user"`;
  `rpc` (0 usos de `useService("rpc")` en v18, 67 ficheros en v17) → función
  exportada en `@web/core/network/rpc`. `company` en cambio SIGUE siendo service
  en v18 y no cae hasta v19.
- Ajustes en APIs de vistas (parsers/controllers); revisa rutas de import en
  `odoo-version-api-reference`.
- Algunas utilidades reubicadas dentro de `@web/core/...`.

### v19 (verificado contra el código de v19)
**Field widgets** — la firma genérica es `extractProps(fieldInfo, dynamicInfo)`;
destructura solo lo que uses. El 2º parámetro `dynamicInfo` NO es novedad v19:
existe desde v17 (`web/static/src/views/fields/field.js:216`, `class Field` v17 / `:409` v18 / `:450` v19). Las claves del
1er arg varían por campo; p.ej. `char_field` v19 destructura
`({ attrs, options, placeholder })` SIN recibir `dynamicInfo`
(`web/static/src/views/fields/char/char_field.js:116`, `charField`). `displayName` usa `_t(...)` de
`@web/core/l10n/translation`. Lectura `props.record.data[props.name]`, escritura
`props.record.update({ [props.name]: valor })`.

**Extensión = `patch()`** — el override de componentes/services/widgets estándar
se hace con `patch(Clase.prototype, {...})` de `@web/core/utils/patch`, llamando al
original con `super.metodo()`. Ver `extending-standard.md`. `this._super` es legacy.

**Framework Interaction** (`@web/public/interaction`) — para JS **público/website
ligero**, NO backend; sucesor preferido para JS público nuevo (`publicWidget` sigue
existiendo en v19 en `legacy/js/public/public_widget.js`, coexisten). Es una clase con:
```js
import { Interaction } from "@web/public/interaction";
import { registry } from "@web/core/registry";
export class MyInteraction extends Interaction {
    static selector = ".my_block";        // dónde se monta
    dynamicContent = {                    // binding declarativo
        "button": { "t-on-click": () => this.doStuff() },
    };
    setup() {}                            // sync, antes de willStart
    async willStart() {}                  // carga async previa
    start() {}                            // tras el primer render
    destroy() {}                          // cleanup (auto en la mayoría de casos)
}
registry.category("public.interactions").add("mi_modulo.my", MyInteraction);
```
En backend usa OWL completo; reserva Interaction para website/portal. NO mezcles su
ciclo de vida (`setup/willStart/start/destroy`, `this.el`, `dynamicContent`) con
hooks OWL.

**Dos categorías "public" distintas** (no confundir):
- `public.interactions` → instancias de la clase `Interaction` (JS ligero website).
- `public_components` → componentes OWL montados en páginas públicas vía
  `<owl-component name="...">`.

**Tests** — Hoot es el runner de todo lo nuevo (bundle `web.assets_unit_tests`).
QUnit NO está retirado: queda relegado a `static/tests/legacy` con su propio
bundle `web.qunit_suite_tests`, y `web.tests_assets` sigue cargando la lib
(`web/__manifest__.py:482-483`, `:520`). No escribas QUnit nuevo, pero no asumas
que ha desaparecido del árbol.

**Rutas de import que cambian v18 → v19** (verificado contra el código):
- `company`: era service en v17 y v18 (`webclient/company_service.js`,
  `category("services").add("company", ...)` en `:145` de v18). En **v19 no
  existe**: el dato sale de `import { user } from "@web/core/user"`
  (`web/static/src/core/user.js:254`, `user`) — expone `userId`, `name`, `context`, `isAdmin`,
  `hasGroup()`, `activeCompany`. `useService("company")` revienta en v19.
- `user`: OJO, este cae **un major antes**. `useService("user")` ya no existe en
  **v18** (`core/user_service.js` solo está en v17; 49 usos en CE17, 0 en CE18).
  Si trabajas en v18, impórtalo igual que en v19.
- `rpc`: no es service en v18 ni v19 — **en v17 sí lo era**, y esto conviene
  decirlo fuerte porque circula lo contrario: `core/network/rpc_service.js:140`, `rpc`
  hace `registry.category("services").add("rpc", rpcService)`, con **67 ficheros
  de CE17** (55 de EE17) llamando `useService("rpc")` y **0 en CE18**. Desde v18
  es función exportada en `@web/core/network/rpc`: `import { rpc } from ...`
  (`web/static/src/core/network/rpc.js:65`, `rpc` en v18, `:93` en v19 — la línea se movió, el símbolo
  no).
- Field widgets: firma genérica `extractProps(fieldInfo, dynamicInfo)`; destructura
  solo lo que uses. `dynamicInfo` NO es exclusivo de v19 (existe desde v17). Las
  claves del 1er arg varían por campo: `char_field` v19 usa
  `({ attrs, options, placeholder })` SIN `dynamicInfo`
  (`web/static/src/views/fields/char/char_field.js:116`, `charField`); write-back con `record.update`
  (`web/static/src/views/fields/boolean/boolean_field.js:31`, `booleanField`).
- Plantillas core: muchos `t-name` migraron a prefijo `web.` (p.ej.
  `web.BooleanField`). Si heredas con `t-inherit`, busca el `t-name` real en el
  `.xml` del componente de v19, no asumas el de v18.

**Lo que se deprecó / retiró en v19**:
- **Hoot** es el runner de referencia (`web.assets_unit_tests`); QUnit queda solo
  para los tests legacy pendientes de portar (ver arriba).
- `company` como service (ver arriba). `user` ya había caído en v18.
- APIs legacy de widgets (`widget` Python clásico) — todo vía `registry` + OWL.

**Interaction (v19)** — `@web/public/interaction` (`public/interaction.js`).
Clase base con `static selector`, `dynamicContent`/`dynamicSelectors`,
y ciclo `setup()` (sync) → `willStart()` (async) → `start()` → `destroy()`
(verificado l. 149/156/163/170). Es para website/JS público ligero; sucesor
preferido para JS público nuevo (`publicWidget` sigue existiendo en v19 en
`legacy/js/public/public_widget.js`, coexisten). NO existe en v18. NO la mezcles
con OWL del backend.
Confirma renombrados de categorías/imports con `odoo-version-api-reference` antes
de codificar para v19.

## Checklist de compatibilidad

1. Confirmar version objetivo (v17/v18/v19).
2. Bundle correcto en manifest (`web.assets_backend`) con JS + XML (+ SCSS).
3. Imports desde rutas validas para esa version (verificar con la API reference).
4. Runner de tests acorde: QUnit (v17) / Hoot (v18+).
5. En v19, evaluar si Interaction cubre el caso antes de montar OWL completo.
