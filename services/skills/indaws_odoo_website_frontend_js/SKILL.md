---
name: indaws-odoo-website-frontend-js
version: 0.1.17
effort: high
model: opus
description: >-
  Escribe JS público/interactivo del frontend de Odoo (v17-v19): publicWidget
  (v17/v18) y framework Interaction (v19: dynamicContent, t-on-/t-att-, registry
  public.interactions), rpc desde frontend, animaciones, lazy-loading. NOT backend
  web client (→ odoo-owl-component-builder); NOT controllers Python
  (→ odoo-website-controller).
  Triggers: "publicWidget", "JS de un snippet", "interaction v19", "dynamicContent",
  "animacion en pagina web", "rpc desde frontend", "JS publico Odoo",
  "public widget", "website js", "snippet animation js".
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep]
---

# Odoo Website Frontend JS

> Convenciones de codigo y conexion a datos: ver `references/shared/coding-standards.md` y `references/shared/odoo-connection.md`.

Escribe el JS **publico/interactivo** que se ejecuta en el navegador del visitante de
una pagina web de Odoo (v17-v19): el JS que da vida a snippets, formularios publicos,
animaciones y micro-interacciones. NO es el web client del backend (eso es OWL puro,
`odoo-owl-component-builder`), NO es el HTML del contenido (`odoo-frontend`), NO son los
controllers Python que sirven la pagina (`odoo-website-controller`).

Dos frameworks segun version:
- **v17/v18 — `publicWidget`** (`@web/legacy/js/public/public_widget`): clase legacy con
  `selector`, `start()`, `events` (jQuery-style), `destroy()`. Una instancia por elemento que
  casa con `selector` al cargar la pagina.
- **v19 — `Interaction`** (`@web/public/interaction`): sustituto moderno sin jQuery, con
  `dynamicContent` declarativo (`t-on-`/`t-att-`/`t-out`), ciclo `setup/willStart/start/destroy`
  y helpers (`addListener`, `waitFor`, `debounced`). `publicWidget` aun existe en v19 por
  compatibilidad pero lo nuevo va en `Interaction`.

**"Coexisten" NO es un empate**: ficheros CE que lo importan, medido el 2026-08-17 —
`publicWidget` 155 → 161 → **10**; `Interaction` 0 → 0 → **207**. `publicWidget` nuevo en v19 es
escribir contra la salida (detalle en `references/interaction-v19.md`).

## Workflow

1. **Confirmar version y framework.** v17/v18 → `publicWidget`; v19 → `Interaction`.
   Si el modulo debe soportar v18 y v19 con el mismo codigo, mantenlos en ficheros separados
   por version. Carga `references/publicwidget-v18.md` o `references/interaction-v19.md`.
2. **Definir el `selector`.** El selector CSS de los elementos que activan el JS (p.ej.
   `.s_mi_snippet`). Es el unico punto de entrada: una instancia por match. Sin `selector`, nada corre.
3. **Escribir el widget/interaction.** Esqueleto JS + registro + bundle `web.assets_frontend`
   en el manifest. `publicWidget`: `references/publicwidget-v18.md`. `Interaction`:
   `references/interaction-v19.md` (incluye tabla de migracion publicWidget -> Interaction).
4. **Cablear datos / animaciones / lazy.** rpc/ajax al servidor, patrones de animacion
   (requestAnimationFrame, scroll), lazy-loading y manejo del **modo edicion** del builder:
   `references/frontend-rpc-and-patterns.md`.
5. **Probar.** Tours end-to-end (`HttpCase` + tour) que navegan la pagina publica. [delega] en `odoo-test-generator`.

## GOTCHAS

1. **Sin `selector` el widget no corre nunca.** `publicWidget` y `Interaction` se instancian
   SOLO por match del `selector` al iniciar la pagina; no hay otra via automatica. `selector: false`
   es el default (`web/static/src/legacy/js/public/public_widget.js:496`, `PublicWidget`, igual en v18 y v19) y el codigo lo trata como
   "no instanciar" sin decir nada (`:603`, `if (!this.selector)`). En v19 un `selector` vacio LANZA
   error (`web/static/src/public/interaction_service.js:104-108`, `InteractionService.startInteractions`), y el mensaje delata la causa real: haberlo puesto en
   la instancia en vez de `static` en la clase.
1b. **La regla del hermano es la INVERSA y el mismo bucle lanza dos veces.** Justo despues,
   `web/static/src/public/interaction_service.js:109-113`, `InteractionService.startInteractions`: *"The dynamic content object should be defined on the
   instance, not on the class"*. O sea `selector` **static en la clase**, `dynamicContent`
   **campo de instancia** — invertidos, y se confunden porque se escriben pegados.
2. **Bundle equivocado.** El JS publico va en `web.assets_frontend`, NO en `web.assets_backend`.
   Si lo declaras en el bundle backend no se carga en la pagina publica y el widget no aparece.
3. **`disabledInEditableMode` (v18).** Los widgets de animacion estan DESHABILITADOS en modo
   edicion por defecto (`website/static/src/js/content/snippets.animation.js:59`). Si tu JS debe correr tambien en edicion,
   ponlo a `false` explicitamente; si no, asegurate de que no se ejecute o rompes el editor.
4. **`start()` corre tras el render, `willStart()` antes.** En `publicWidget`, datos asincronos
   que el render necesita van en `willStart` (debe devolver Promise), no en `start`
   (`web/static/src/legacy/js/public/public_widget.js:553-574`, `PublicWidget` vs `587-589`). Mismo principio en `Interaction` (`web/static/src/public/interaction.js:156`, `Interaction.willStart` willStart / `163` start).
5. **`this.$el` es jQuery (v18), `this.el` es DOM nativo (v19).** `publicWidget` da
   `this.$el`/`this.$()` (`web/static/src/legacy/js/public/public_widget.js:722-723`, `PublicWidget`, y `web/static/src/legacy/js/public/public_widget.js:746-751`); `Interaction` solo `this.el`
   (`web/static/src/public/interaction.js:122-125`, `Interaction.constructor`) — sin jQuery, usa `querySelector`.
6. **Eventos: `events` dict (v18) vs `dynamicContent` (v19).** v18 `events: {'click .btn': '_onClick'}`
   delegado por jQuery (`web/static/src/legacy/js/public/public_widget.js:756-805`, `PublicWidget`); v19
   `dynamicContent = {'.btn': {'t-on-click': (ev) => this.onClick(ev)}}` (`web/static/src/public/interaction.js:79-110`, `class Interaction`).
   `events` NO existe en `Interaction`.
7. **Limpieza en `destroy()`.** Los widgets/interactions SI se destruyen al entrar en modo
   edicion. Si dejas `setInterval`, listeners o `requestAnimationFrame` sin limpiar, fugas y
   bugs en el editor. v18: limpia en `destroy()` (ej. `s_countdown/000.js`). v19: `registerCleanup(fn)`
   (`web/static/src/public/interaction.js:453`, `Interaction.registerCleanup`) o helpers que auto-limpian — `addListener` (`:365`), `debounced` (`:284`),
   `throttled` (`:316`). **`addListener` NO pasa por `registerCleanup`**: empuja solo desde el motor
   (`web/static/src/public/colibri.js:124`, `Colibri.addListener`) al mismo array. No le registres limpieza a mano, la duplicas.
7b. **`addListener` LANZA si lo llamas antes de `start()`** — no es un no-op silencioso:
   `web/static/src/public/colibri.js:64-68`, `Colibri.addListener` comprueba `isReady`. En `setup()`/`willStart()` no vale, y `willStart()` es
   justo donde tienta si vienes de `publicWidget`, que no tenia esa restriccion.
8. **`rpc` es import, no `this._rpc`.** `import { rpc } from "@web/core/network/rpc"` y
   `rpc("/mi/ruta", {params})` (real en `website_sale/static/src/js/website_sale.js:7`, `cartHandlerMixin`, y `website_sale/static/src/js/website_sale.js:216`, `_changeCartQuantity`). `this._rpc` es legacy de
   `publicWidget` via `ServicesMixin`.
9. **`t-att`/`t-out` de `dynamicContent` NO son OWL.** Sintaxis parecida, motor distinto
   (colibri): los valores son funciones `() => value`, no expresiones de plantilla
   (`web/static/src/public/interaction.js:104-106`, `class Interaction`). `false`/`null`/`undefined` quitan el atributo, `true` pone el nombre.
10. **Registro v18 vs v19 distinto.** v18 asigna a un objeto,
    `publicWidget.registry.miNombre = publicWidget.Widget.extend({...})` (`s_searchbar/000.js:13`, `_render`);
    v19 `registry.category("public.interactions").add("modulo.nombre", MiInteraction)`
    (`website/static/src/interactions/image_lazy_loading.js:47`, `website.image_lazy_loading`).
11. **En v19, dos cosas van en registries APARTE, no dentro de la interaction**: el comportamiento
    de modo edicion (mixin en `public.interactions.edit`, fichero `*.edit.js`,
    `website/static/src/interactions/search_modal.edit.js:12-16`) y el montaje de OWL (`this.mountComponent` o `t-component`,
    `web/static/src/public/interaction.js:87`, `Interaction`, y `web/static/src/public/interaction.js:465`, `Interaction.mountComponent` (v19); root `<owl-root contenteditable="false">`,
    `web/static/src/public/interaction_service.js:76`, `InteractionService.prepareRoot`). Detalle de los dos: `references/interaction-v19.md`.
12. **`auth='public'` y datos.** El JS publico llama a rutas que el visitante anonimo puede
    ver; el control de acceso vive en el controller Python (`odoo-website-controller`), NO en
    el JS. No asumas permisos: una `rpc` a una ruta `auth='user'` fallara para el anonimo.

## NEVER

- NUNCA pongas JS publico en `web.assets_backend`; va en `web.assets_frontend`.
- NUNCA uses esta skill para el web client backend (field widgets, vistas, client actions) → `odoo-owl-component-builder`.
- NUNCA escribas aqui el HTML del contenido del snippet/pagina → `odoo-frontend`; ni los controllers Python → `odoo-website-controller`.
- NUNCA dejes `setInterval`/listeners/rAF sin limpiar en `destroy()` (v18) o sin `registerCleanup`/helper auto-limpiable (v19): rompe el modo edicion.
- NUNCA arrastres a v19 lo de v18: `events: {...}` no existe (usa `dynamicContent` con `t-on-`), no hay jQuery (`Interaction` solo expone `this.el`), y `this._rpc` es legacy — importa `{ rpc } from "@web/core/network/rpc"`.
- NUNCA codigo, identificadores, plantillas ni comentarios en otro idioma que no sea ingles; solo strings de cara al usuario se traducen (`_t`).

## ALWAYS

- SIEMPRE define un `selector` (es el unico disparador) y namespacea el nombre del registry con el modulo (`modulo.nombre`).
- SIEMPRE declara el bundle `web.assets_frontend` en el manifest.
- SIEMPRE confirma version objetivo antes de elegir framework: v17/v18 → `publicWidget`, v19 → `Interaction`.
- SIEMPRE carga datos asincronos en `willStart` (no en `start`) si el render los necesita, y limpia efectos en `destroy()`/cleanup decidiendo el modo edicion: `disabledInEditableMode` (v18) o variante `.edit.js` (v19).
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-website-frontend-js · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuando |
|-------|-------|--------|
| odoo-owl-component-builder | developer | Componentes OWL del web client backend (field widgets, vistas, client actions, systray) |
| odoo-frontend | common | Disenar el HTML/Bootstrap del contenido del snippet o pagina |
| odoo-website-controller | developer | El controller Python que sirve la pagina y las rutas `rpc` que consume el JS |
| odoo-version-api-reference | common | Firmas exactas y diffs del framework JS por version (v17/v18/v19) |
| odoo-test-generator | developer | Tours (`HttpCase`) que navegan la pagina publica y verifican la interaccion |
| odoo-module-creator | developer | Scaffolding del modulo base y manifest si no existe |

## Consumed by

- `odoo-theme-developer` (developer) — JS público / widgets / interacciones del frontend del tema.
- `odoo-website-snippets` (developer) — interactividad pública del snippet (`publicWidget`/`Interaction`).

## REFERENCES

| Fichero | Contenido | Cargar cuando |
|---------|-----------|---------------|
| [references/publicwidget-v18.md](references/publicwidget-v18.md) | `publicWidget` v17/v18: esqueleto completo (`selector`, `start`/`willStart`, `events`, `destroy`), registro en `publicWidget.registry`, `disabledInEditableMode`, `this.$el`/`this.$()`, ejemplos reales del core | v17/v18, paso 3 |
| [references/interaction-v19.md](references/interaction-v19.md) | Framework `Interaction` v19: ciclo `setup/willStart/start/destroy`, `dynamicContent` (`t-on-`/`t-att-`/`t-out`/`t-component`), `addListener`/`waitFor`/`debounced`, registro en `public.interactions`, `.edit.js`, montar OWL, mas TABLA de migracion publicWidget -> Interaction | v19, paso 3 |
| [references/frontend-rpc-and-patterns.md](references/frontend-rpc-and-patterns.md) | `rpc`/ajax desde el frontend, patrones de animacion (rAF, scroll), lazy-loading de imagenes, manejo del modo edicion del builder, limpieza de efectos | Paso 4, datos/animaciones/lazy/edicion |
