# SEO on-page de www.dynapps.es — lo que se decide por página

Esta referencia solo cubre lo que se arregla **escribiendo texto**. Lo técnico
(canonical, 301, sitemap, hreflang, `?ref=indaws-redirect`) no se arregla
pegando en DatoCMS: vive en el plan de recuperación y lo ejecuta el equipo web.
Ver `PLAN-SEO-recuperacion-dynapps-2026-09.md`, secciones T1-T15.

## La regla que manda

`como-trabajamos/como-se-lleva-el-seo.md` (Odoo Knowledge, space 12): en la web
**se adapta al mercado local, no se traduce**. Una página en `/es/` no es la
página inglesa en español: es la página que un comprador español busca. Si esta
referencia y ese documento se contradicen, gana ese documento y hay que decirlo.

## Title (`seo_title`)

| Regla | Por qué |
|---|---|
| ≤ 60 caracteres **contando** « \| Dynapps» | Google corta por píxeles ~600; 60 car. es el proxy que usa el plan. El límite lo comprueba es-lint (`longitud_campos.seo_title`) |
| Keyword principal al principio | Lo que se lee primero pesa más y sobrevive al corte |
| Marca al final, siempre « \| Dynapps» | Patrón único de sitio (acción C17 del plan) |
| Sin emojis, sin ➕⭐, sin MAYÚSCULAS de reclamo | Google los elimina o los ignora; en el SERP quedan como spam |
| Único en todo el sitio | Hay titles repetidos entre páginas (`/es/services` y `/es/contact` comparten uno) y titles cruzados del registro contiguo: es un fallo del CMS, revisar la colección entera |
| «Partner», no «Socio» | Es el término del programa de Odoo y la keyword real |

## Meta description (`seo_description`)

- 120-155 caracteres. Por debajo desperdicia, por encima se corta (es-lint lo mide).
- Propia de la página. La meta genérica «Dynapps España es un Socio Odoo Gold
  especializado en implementación ERP…» está repetida en al menos seis páginas:
  cada una que se toque estrena la suya.
- Concreta y verificable: cifra, ciudad, versión, plazo. Nada de «soluciones a medida».
- Las cifras salen de la hoja canónica (P11 del plan), no de la memoria: Gold
  Partner desde 2020, 158 referencias en odoo.com (revisable, o «más de 150»),
  sede en Valencia, oficina en Madrid.
- Llamada a la acción al final, en «tú» y en imperativo de tú: «Pide una demo».

## H1 (`hero_title`)

- **Uno y solo uno.** Hay páginas sin H1 (`/es/contact`): eso es un hueco, no un estilo.
- Lleva la keyword en lenguaje humano. No es el title: el title vende en el SERP,
  el H1 confirma al que ya ha entrado.
- ≤ 70 caracteres (`longitud_campos.hero_title`).

## H2 / H3 (`section_heading`, `card_title`)

- Jerarquía sin saltos: H1 → H2 → H3. Un H3 que cuelga de nada es un bloque mal montado.
- Un H2 por intención de búsqueda: «qué es», «cómo funciona», «cuánto cuesta»,
  «para quién». Son los que AI Overviews y los asistentes citan literalmente.
- Los H2 de una página pilar enlazan a páginas que **existen**. Si no existe, o se
  crea o no se enlaza.

## Cuerpo (`section_body`, `list_item`)

- Longitud objetivo: servicio/producto > 800 palabras, blog > 1.200. Varias páginas
  `/es` viven en 400-700: el hueco se rellena con contenido que responde, no con relleno.
- La keyword aparece de forma natural, incluida en el primer párrafo. Sin repetirla
  cada tres líneas: el lint de tono caza antes la monotonía que Google el *stuffing*.
- La primera frase de cada sección responde su H2 en una línea. Es lo que se extrae
  como respuesta en los asistentes.
- Datos con fuente. Una cifra sin fuente no se publica (P11).

## Enlaces internos

- Mínimo 3 enlaces internos por página, con anchor exacto y descriptivo
  («implantación de Odoo», no «aquí» ni «leer más»).
- Un anchor por destino y por página: dos enlaces con el mismo anchor a destinos
  distintos se pisan.
- Nunca enlazar a una URL con parámetro ni a una que redirija: el destino final, en 200.

## Imágenes (`alt_text`)

- Toda imagen con contenido lleva `alt` descriptivo en español, con la keyword solo
  si es honesta. Las decorativas, `alt=""` a propósito.
- El extractor cuenta las que faltan; en `/es` faltaban 11 de 109 el 15-sep.

## Con quién se compara (medido, no heredado)

En la SERP real de «odoo partner» (15-sep-2026) los que salen son **hotaka.io,
abconsultores.es, nanobytes.es, trey.es, landoo.es y garber.es**. La lista de
competidores que arrastra el skill `marketing-skills:seo-manager` —Process Control,
Tecnativa— **no aparece**: es de la etapa indaws.es y no se usa para comparar.

## Cómo re-medir

La foto se rehace con `scripts/refrescar_baseline.py`, y hay dos cadencias porque hay
dos costes. Las fija la sección 9 del plan, no esta skill:

| Cadencia | Comando | Qué trae | Coste |
|---|---|---|---|
| **Semanal** (lunes) | `refrescar_baseline.py --sin-labs` | Crawl del sitemap: códigos, redirecciones, titles, H1, canonical y palabras. Caza los titles cruzados del CMS y las páginas que se quedan sin H1 | 0 |
| **Mensual** (día 15) | `refrescar_baseline.py` | Lo anterior **más** las keywords de DataForSEO Labs | ~1-2 $ |

Cada pasada deja un `data/informe-AAAA-MM-DD.md` con el movimiento contra la foto
anterior: posiciones que suben y bajan, keywords nuevas y perdidas, y titles que han
cambiado en el CMS sin avisar.

Dos cosas que el script respeta y conviene no romper a mano:

- **El baseline se fusiona, nunca se reemplaza.** Un crawl parcial o un sitemap corto
  no pueden borrar la decisión del plan de una página. Lo que desaparece del sitemap
  se marca `fuera_del_sitemap`, que es un hallazgo, y no se tira.
- **Hay dos fechas, `_fecha_crawl` y `_fecha_keywords`**, y casi nunca coinciden: el
  crawl es de hoy y las keywords pueden ser de hace un mes. Una sola fecha haría pasar
  por fresco un dato de Labs que no lo es.

Y una advertencia de medición que está en el plan: **los datos de Labs se actualizan
con semanas de retraso** respecto a la SERP real. Medir más a menudo cuesta dinero y
no dice nada nuevo; el control de 30 días se decide con Search Console, no con Labs.

## Cómo se recogió la primera vez

El contexto que sirve `scripts/seo_contexto.py` es la línea base del 15-sep-2026. Para
refrescarlo hacen falta las credenciales de DataForSEO en el entorno
(`DATAFORSEO_LOGIN` y `DATAFORSEO_KEY`; la auditoría completa costó ~1,5 $). Se re-ejecutan los
recolectores propios de la carpeta de auditoría y luego su análisis, y se compara
contra la línea base: los scripts del `seo-manager` del plugin **no valen**, llevan
`indaws.es` hardcodeado y asumen ejecución por la API de Odoo.

Cuidado con la API: `domain_rank_overview`, `serp/.../live/advanced` y
`backlinks/summary` aceptan **una sola task por petición**; `ranked_keywords`,
`bulk_ranks` y `search_volume` sí admiten lotes.

## Qué NO se toca sin aprobación de dirección

La home `/es`, el `<head>` global y el pie. Cualquier propuesta sobre ellos sale
marcada **«requiere aprobación»** y no se pega hasta el OK explícito (§10 del plan).
