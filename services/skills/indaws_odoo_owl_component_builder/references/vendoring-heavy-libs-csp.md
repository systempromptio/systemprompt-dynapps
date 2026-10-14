# Vendorizar una lib JS pesada bajo la CSP de Odoo

Integrar una librería 3rd-party grande (mapas, gráficos, editores…) en el web client bajo la
**CSP estricta** de Odoo. Mezcla de hechos **verificados en fuente v19** y **observaciones
empíricas** de dyngeogis (MapLibre GL) — marcados por separado; el patrón es general aunque el
ejemplo sea MapLibre.

## Regla base: sin CDN, vendorizar + cargar on-demand

- **NO CDN.** La CSP de Odoo es restrictiva (para binarios servidos aplica `default-src 'none'`;
  `odoo/http.py::Application.set_csp`, ~`:2810`, sobre respuestas `image/*`). Vendoriza la lib
  dentro del módulo (`static/lib/...`) y sírvela como asset propio.
- **Cárgala on-demand** al montar el componente con `loadJS(path)` / `loadCSS(path)` de
  `@web/core/assets` (verificado v19: `web/static/src/core/assets.js:78`, `loadJS`, `:83`
  `loadCSS`). Así el bundle sólo registra un componente ligero y el blob pesado se descarga cuando
  la vista/acción se abre, no en cada arranque del cliente.

```js
import { loadJS, loadCSS } from "@web/core/assets";
// en setup():
onWillStart(async () => {
    await loadCSS("/my_module/static/lib/maplibre/maplibre-gl.css");
    await loadJS("/my_module/static/lib/maplibre/maplibre-gl.js");
});
```

## Web workers bajo CSP (OBSERVADO en dyngeogis — no citable de fuente)

Si la lib usa **web worker**, es el punto frágil bajo la CSP de Odoo:

- Usa la **build CSP** de la lib y fija el worker con su **API explícita**. En MapLibre:
  **`maplibregl.setWorkerUrl(path)`** — ojo: **NO existe `maplibregl.workerUrl`**; asignar esa
  propiedad es un **no-op silencioso** (parece que la fijaste, pero no).
- El worker por **`blob:`** quedó **bloqueado** en nuestro montaje bajo la CSP de Odoo. **Síntoma
  engañoso: el fondo/canvas pinta, pero los datos no se parsean** (worker muerto), **sin error
  claro**. Diagnóstico: `source.isSourceLoaded() === false` / `querySourceFeatures().length === 0`
  con el fondo OK.
- ⚠️ Honestidad: esto es **observación empírica** de dyngeogis, no una directiva CSP que se pueda
  citar de la fuente Odoo. Si tu instancia tiene otra CSP (proxy, `Content-Security-Policy` custom)
  el comportamiento puede diferir — verifícalo en tu navegador.

## Preferir formatos que NO dependan del worker

Cuando la lib ofrezca opción, **elige el formato que no use worker**:

- **Tiles raster** (imágenes PNG/JPEG, decodadas por el navegador, sin worker/glyphs/sprites) →
  robustas bajo CSP.
- **Tiles vectoriales** (worker + glyphs + sprites) → frágiles; pueden salir en blanco sin error.
- En dyngeogis el default se cambió de **vector a raster** justo por esto.

## Contenedor 0-width (pestaña/panel oculto al montar)

Si el mapa/canvas se monta en un contenedor con ancho 0 (pestaña oculta, panel colapsado), el WebGL
no dimensiona. Observa el resize y fuerza el recálculo:

```js
this.resizeObserver = new ResizeObserver(() => this.map?.resize());
this.resizeObserver.observe(this.containerRef.el);
// onWillUnmount: this.resizeObserver.disconnect();
```

## Widget de campo que envuelve la lib (v17+)

Registra un **objeto descriptor** `{component, supportedTypes, extractProps}` en
`registry.category("fields")`, NO la clase (ver GOTCHA 3 del SKILL). Escribe el valor con
`props.record.update({[props.name]: v})`, nunca mutando `props.record.data`.

## Caveat de verificación en navegador (IMPORTANTE)

Un **screenshot remoto** puede NO componer la capa WebGL (sale en blanco) aunque en el navegador
real se vea bien. No des por roto un mapa por un screenshot en blanco. Verifica de verdad con:
- `map.queryRenderedFeatures()` / lectura de píxeles (readPixels),
- o **Performance API**: `performance.getEntriesByType('resource')` para ver qué se descargó
  realmente (tiles, glyphs, worker).
