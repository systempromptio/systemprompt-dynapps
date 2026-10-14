# El informe al cliente: texto, marca, PDF y verificación

## Posicionamiento (criterio de la casa)

El informe **no sustituye a ninguna gestoría**: complementa al departamento de administración,
financiero o controlling, les ahorra tiempo y les deja analizar para decidir mejor.

- Registro positivo: «para qué sirve» primero, el límite al final y en una línea.
- **Prohibido citar normas** (NRV 9ª, Reglamento de facturación, LIVA). Lo que importa se explica
  por su **efecto en el sistema**: el 347 no se puede generar, la línea no llega al 303.
- No se dictamina: se señala qué falta y dónde se completa. La decisión contable o fiscal es de
  la empresa.
- **El riesgo no es decirlo, es callarlo**: si el dato sale de nuestro ERP y no lo advertimos, el
  reclamable es el partner. Ejemplo: NIF vacío → el 347 no sale completo.

## Marca

Cargar `dynapps-brand` antes de tocar el CSS de `informe.py`. Portada Midnight con el logo
vertical de `assets/`, nunca redibujado; Readex Pro; palabra clave del titular en color de
acento; superficies claras y una sola oscura; degradado sobre el pie; dominio `www.dynapps.es`.
**Ni una raya larga**: se cuelan por los textos de los chequeos.

## Estructura, siete apartados

`01 Resumen · 02 Alcance y cobertura · 03 Lo que hay que atender · 04 <el punto con más peso> ·
05 Lo que está correcto · 06 Plan de cierre · 07 Cómo se cierra un punto`

- El titular lleva el mes y el año del corte («Dieciséis puntos antes del cierre de agosto de 2026»).
- El 02 va segundo a propósito: un informe que no dice dónde no ha podido mirar no permite saber
  qué vale lo que afirma. Abre con para qué usa Odoo la empresa.
- El 03 agrupa en **puntos de cliente** (`puntos.py`), no uno por chequeo. Cuatro líneas por
  ficha: qué ocurre / qué implica / qué hacer / quién y cuándo. Las recomendaciones al final, en
  su bloque, sin contar en el titular.
- El 04 desarrolla el punto con más peso, generado de ese punto.
- Descartados: en el 02, desde `<HC_OUT>/descartes.json`; un descartado no sale además como punto.
- Sin apartado de «lo que no le aplica»: el número va en una línea del 02.
- El aviso de prototipo solo con `HC_PROTOTIPO=1`, y cierra el 07 junto al aviso de alcance.

## Puntos de cliente (`puntos.py`)

`DEF` agrupa chequeos en puntos con título, prioridad, «qué implica», «qué hacer» y «quién». Los
textos de `DEF` son **genéricos**: la cifra concreta llega en «qué ocurre», que sale del texto de
cada chequeo. Un hueco que no cae en ningún punto sale como punto propio con el título del
catálogo y la marca `generico`: **redactar su «qué implica» y «qué hacer» antes de entregar**, o
añadirlo a `DEF` si va a repetirse.

## El PDF (`a_pdf.py`, WeasyPrint)

Excepción declarada a `pdf-renderer`: los marcadores en niveles y el índice con número de página
real (`target-counter`) son de WeasyPrint. Se genera del HTML intermedio, que no se entrega.

- Renderizar desde `HTML(filename=...)`, nunca desde cadena, o las anclas no resuelven.
- Marcadores: `section h2 { bookmark-level:1; bookmark-label:content(text) }` y el título de ficha
  en nivel 2. La etiqueta de prioridad va **fuera** del `h3` o se pega al marcador.
- Índice: `target-counter(attr(data-h url), page)` en un `<span>` hijo; con `attr(href url)` en un
  `<a>` `display:flex` devuelve 0.
- `background-clip:text` no existe en impresión (números invisibles): color plano. `clamp()` con
  `vw` no resuelve: tamaños en pt.
- Portada a sangre (`@page :first { margin:0 }`) sin cajas de pie; un solo título en portada.
- `print-color-adjust: exact`; `break-inside: avoid` en fichas, tablas y filas.

## Verificación antes de entregar

1. Títulos de fichas del PDF = puntos del Excel; total de evidencias del Excel = cifra del PDF.
2. Ningún LIMPIO con universo vacío; ningún punto `generico` sin redactar.
3. `a_pdf.py` imprime marcadores y páginas: revisarlos. Render de portada, índice y dos páginas
   con `pdftoppm` y **mirarlas**: contraste y degradados solo se ven mirando.
4. Grep en el HTML: ningún nombre técnico de módulo, ninguna cita de norma, ninguna raya larga.
5. Ningún dato del cliente anterior: todo sale de `config.json` y de `descartes.json` de su
   carpeta. Buscar el nombre del anterior en el motor: no debe aparecer.
6. **Chequeos de saldos contrastados con el informe que ve el cliente**: abrir el balance de Odoo
   a la fecha de corte (`account.report`, `get_report_information`). Lo que es cierto en el mayor
   puede ser falso en el balance, y es el balance lo que lee el cliente.
7. `carga.py` no ha avisado de topes alcanzados (`truncados`).
