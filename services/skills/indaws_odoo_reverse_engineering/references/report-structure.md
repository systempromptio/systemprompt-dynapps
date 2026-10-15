# Estructura del informe y métricas

## Cómo se calcula el conocimiento global

`consolidate.py` produce:

- **conocimiento_pct por módulo**: lo estima el agente del módulo (0-100), según cuánto de su
  comportamiento queda entendido. El catálogo (estructura, vistas, automatismos identificados,
  estados) siempre cuenta como conocido; la lógica cerrada lo baja.
- **peso por módulo** = nº de registros que gestiona / total de registros.
- **conocimiento global ponderado** = Σ(conocimiento_pct × registros) / Σ(registros). Es el número
  de portada. También se reporta la media simple como referencia secundaria.

## Totales: modelos distintos y campos personalizados

- **Campos**: todos los que se cuentan son personalizados (creados por el código a medida). Los
  campos estándar de Odoo no aparecen porque no están en el `ir.model.data` de los módulos a medida.
- **Modelos**: se distinguen **nuevos** (creados a medida) de **extensiones** (modelos estándar de
  Odoo a los que el desarrollo añade campos). El total global usa modelos **distintos** para no
  contar varias veces un estándar ampliado por varios módulos (`build_facts.py` ya lo deduplica).

## report_meta.json (parte editorial, opcional)

`build_report.py` lee `<out_dir>/report_meta.json` si existe (ver `assets/report_meta.example.json`).
Permite fijar cliente, fecha, tarea, título, hallazgos destacados, notas de seguridad, recomendación
y próximos pasos sin tocar el código. Si falta, se generan por defecto a partir de los datos (los
hallazgos salen de los módulos de más peso).

### Marca (Indaws / Dynapps)

`"brand": "indaws"` (por defecto) o `"dynapps"`. El generador localiza solo los assets de esa marca
(fuentes, logos) y aplica su paleta. Puedes forzar la ruta de assets con la variable `BRAND_ASSETS`.

### Secciones opcionales (solo aparecen si das los datos)

Para una auditoría de pre-migración más completa que el puro código a medida:

- `"available_source": [{modules, author, license, repo, note}]` → sección **“Módulos con fuente
  disponible”** (lo que NO necesita ingeniería inversa: OCA, conectores open source… con su repo).
- `"risks": [{level: critico|alto|medio, ref, title, body, action}]` → sección **“Riesgos
  priorizados”** con un recuadro que explica el baremo.
- `"irlogging": {intro, found:[{title,detail}], goal, pending:[{title,detail}]}` → sección
  **“ir.logging · evidencia y pruebas pendientes”**.
- Si existe `<out_dir>/oca_report.json` (lista de `{name, installed, oca, repo, state}` con state
  `AL_DIA|DESACTUALIZADO|INSTALADO_MAYOR(fork?)`), se añade la sección **“Auditoría · Módulos OCA”**.

## Render a PDF (motor y alternativa)

Lo previsto es la skill **`pdf-renderer`** (Playwright). Si Playwright no está instalado, hay una
alternativa fiable con Chrome headless del sistema:

```bash
google-chrome --headless=new --disable-gpu --no-sandbox \
  --print-to-pdf=informe.pdf --no-pdf-header-footer \
  "file:///ruta/informe-ingenieria-inversa.html"
```

El HTML ya trae `print-color-adjust:exact`, necesario para que la portada oscura y los chips de color
salgan en el PDF.

## Secciones del informe (orden fijo)

1. Portada (marca Indaws).
2. Resumen ejecutivo + tarjetas (módulos, áreas, modelos, campos, registros).
3. Qué hemos encontrado: modelos nuevos vs modelos de Odoo ampliados; campos personalizados.
4. Estado del diagnóstico: gauge del % global + barras de los 3 niveles de lógica.
5. Criterio de certeza: confirmado / aproximación / no determinable.
6. Mapa por áreas: tabla con modelos, campos, registros y % por área.
7. Resumen por módulo: tabla ordenada por uso, con split nuevo/extensión y peso.
8. Detalle por módulo: descripción, modelos (con ★ los críticos) y lógica en 3 niveles.
9. Hallazgos destacados (+ notas de seguridad si las hay).
10. Conclusión y recomendación + próximos pasos.

## Render a PDF

No reinventes el render. Usa la skill **`pdf-renderer`** sobre el HTML generado
(`<out_dir>/informe-ingenieria-inversa.html`). Tras renderizar, revisa el PDF página a página
(convierte a imágenes con `pdftoppm` y míralas) para detectar desbordes o páginas en blanco antes
de entregarlo.

## Estilo de marca

El informe usa los estilos de la skill **`indaws-brand`** (Montserrat, paleta corporativa, cero
emojis, "tú", ortografía es_ES). `build_report.py` localiza los assets automáticamente; si no los
encuentra, degrada a fuentes del sistema y avisa (define `BRAND_ASSETS` o instala la skill de marca
correspondiente: `indaws-brand` o `dynapps-brand`).
