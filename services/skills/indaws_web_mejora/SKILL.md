---
name: indaws-web-mejora
description: >
  Coge una URL de www.dynapps.es y devuelve la página entera reescrita: español de
  España impecable y SEO on-page resuelto, campo por campo de DatoCMS, lista para
  copiar y pegar. Junta el lint de tono (`es-lint`) con la auditoría SEO de sep-2026:
  extrae el contenido en vivo, lo trocea por campo, lo linta, lo cruza con las
  keywords que esa URL ya posiciona y entrega «actual → propuesto» bloque a bloque,
  más lo que no se arregla pegando. Usar cuando pidan "mejora esta página", "pásale el
  mejorador", "optimiza /es/loquesea", "esta página está traducida", "reescribe esta
  URL para SEO", "esto suena a inglés", o cuando peguen sin más una URL de dynapps.es.
  Por defecto no publica: entrega para pegar. Si la sesión trae el MCP de DatoCMS,
  puede además dejar los cambios guardados como borrador, nunca publicados. Para
  lintar un texto suelto sin URL, la skill es `es-lint`.
version: 1.4.1
effort: high
metadata:
  owner: victor
  actualizado: 2026-09-18
---

# Mejorador de páginas · www.dynapps.es

Una página de `/es/` tiene tres problemas: **está traducida** (el H3 de `/es/services`
dice «Tren», por Train), **no está optimizada** (sin canonical, titles repetidos) y **le
falta lo que no se escribe en párrafos** (que-es-odoo concentra el 88 % del ETV con cero
tablas y diez preguntas sin marcar).

## Important

- **No publica.** Por defecto entrega texto pegable. Con el MCP de DatoCMS delante sí
  puede escribir, y entonces **guarda como borrador y nunca llama a `publish`**: el
  registro es compartido con otros diez mercados y suele llevar trabajo ajeno a medias.
  El procedimiento entero está en `references/datocms-escritura.md`.
- **La home `/es`, el `<head>` y el pie son [STOP]**: se entregan marcados «requiere
  aprobación» y no se pegan sin OK. El aprobador del área es Víctor (Knowledge 5192).
- **Sin el MCP, el mapeo a campo es conjetura del DOM**: cada bloque sale con
  `campo_probable` y `confianza`. Con el MCP se lee el schema real del proyecto
  **208685** y la conjetura sobra — úsalo antes de dar por buenos nombres de campo.
- **La capa SEO viaja dentro** (`data/baseline-seo.json`); la carpeta de la auditoría
  es opcional. Lo que hace falta es `es-lint`, que busca `scripts/lintar.sh`: si no
  está, la entrega **lo dice**.
- **Quién ejecuta lo que no es copia:** el CMS Next.js y sus redirecciones, el equipo
  web de Dynapps (grupo). Toni solo lleva el openresty de `indaws.es`.

## Lo que manda por encima de esta skill

1. La doctrina, en Odoo Knowledge space 12: `que-publicamos-con-nuestro-nombre.md`
   (voz, «tú», ortografía) y `como-se-lleva-el-seo.md` (**en la web se adapta al
   mercado local, no se traduce**).
2. `PLAN-SEO-recuperacion-dynapps-2026-09.md`: si la URL ya tiene title/meta en su
   Anexo B o una acción asignada (C12, T11, P1…), **se parte de ahí**. Si algo de
   aquí los contradice, ganan ellos y se dice en la entrega.

## Las cinco capas

| Capa | Dónde vive | Qué decide |
| :--- | :--- | :--- |
| Extracción | `scripts/extraer.py` | Qué texto hay, en qué orden y en qué campo cae |
| Idioma | `es-lint` + `references/calcos-web-dynapps.md` | Qué está mal escrito, traducido o suena a máquina |
| SEO | `scripts/seo_contexto.py` + `references/seo-onpage.md` | Qué keyword defiende la página y qué title/meta admite |
| Entidad | `references/entidad-y-asistentes.md` | Si la página es citable por un asistente y nombra la entidad como el resto del mundo |
| Lo que falta | `references/elementos-que-faltan.md` | Qué tabla, figura, imagen o cita de una persona no está y hace falta |

## Workflow

Los comandos se lanzan desde la carpeta de esta skill. Todo va **inline**; con un lote,
los pasos 1 y 3 son mecánicos y admiten `Agent(model: sonnet)` como subagente, uno por
página, y los de criterio (2, 4 y 5) se quedan en opus, el principal. La delegación es
**opcional**: hay configuraciones que la prohíben.

**Paso 1 · Extraer** [mecánico]

```bash
python3 scripts/extraer.py https://www.dynapps.es/es/services -o /tmp/wm
```

Deja `*.bloques.json` (ids `B01`…), `*.campos.json` (entrada del linter),
`*.inventario.md` —con el recuento de tablas, figuras, imágenes y preguntas— y, si la
trae, `*.flight.json` con los modelos de DatoCMS.

**Paso 2 · Lintar** [criterio]

```bash
./scripts/lintar.sh /tmp/wm/es-services.campos.json --all
```

Las claves de `campos.json` son las que el pack conoce, así que el límite por campo
salta solo. Antes de reescribir se lee la referencia `es.md` de `es-lint`. Después,
a mano, `references/calcos-web-dynapps.md`: los calcos de esta web son palabras
españolas correctas en el sitio equivocado y ninguna regex los caza sin falsos
positivos.

**Paso 3 · Situar la página** [mecánico]

```bash
python3 scripts/seo_contexto.py /es/services --kw "odoo partner"
```

Keywords que esa URL ya posiciona, crawl y decisión del plan, de
`data/baseline-seo.json`. Sin esto, proponer un title es opinión. Es línea base del
crawl del 17-sep: para rehacerla, `scripts/refrescar_baseline.py` y la cadencia de
«Cómo re-medir» en `references/seo-onpage.md`. **Nunca se inventa un volumen.**

**Paso 4 · Escribir y verificar** [criterio]

Se redacta el «propuesto» de cada campo y se **vuelve a lintar** con el comando del
paso 2 sobre un JSON con los textos nuevos. Una propuesta con bloqueantes no se entrega.
Aquí entra lo que falta: el recuento de estructura del paso 1 contra
`references/elementos-que-faltan.md`, con tope de **uno o dos elementos nuevos por
página**. Una tabla se entrega pegable; una imagen o una cita, como encargo.

**Paso 5 · Entregar** [criterio]

Formato en `references/salida.md`: una fila por campo que cambia, el «propuesto»
pegable tal cual, caracteres contados en title y meta, y al final el bloque **«esto
no se arregla pegando»** con su destinatario. [STOP] si toca home, `<head>` o pie.

**Paso 6 · Escribir en el CMS** [criterio, opcional]

Solo si la sesión trae el MCP de DatoCMS **y lo han pedido**. El procedimiento completo
—aislamiento de locales, pasada en seco, `current_version`, verificación y publicación
selectiva— está en `references/datocms-escritura.md`. El resumen que no se negocia: se
comprueba que el locale no comparte bloques con otros mercados, se ensaya en el sandbox
de solo lectura, se guarda con `meta.current_version` y **no se llama a `publish`**.

## NEVER / ALWAYS

- **Nunca** se cambia una URL o un slug: sin su 301 es tráfico perdido (T1/T5 del plan).
- **Nunca** se inventa una cifra ni una cita. Las cifras salen de la hoja canónica P11;
  una cita que no existe se pide, no se redacta.
- **Nunca** se traduce la página inglesa: se adapta la española al mercado español.
- **Siempre** se vuelve a pasar la calibración de `es-lint` si se toca su pack.
- **Siempre** una página por entrega, en el orden del plan y con el tope de **20
  cambios por semana**: es lo que permite saber qué movió qué.
- **Nunca** se llama a `items.publish` sin `content_in_locales`, y nunca sin que lo
  pidan: el registro lleva locales de otros diez mercados.
- **Siempre** se dice con qué cuenta firma el MCP antes de escribir. No es la de quien
  pide el cambio.

## GOTCHAS

1. **`/es/` con barra final devuelve 308 y urllib no lo sigue.** El extractor trae un
   handler propio; cualquier otro fetch se comerá el error con la URL que uno pega.
2. **Tirar los `<header>` se lleva el H1.** Dentro de `<main>` un header es cabecera de
   sección: el extractor solo quita `<nav>`. Sin H1, sospechar de esto antes que del CMS.
3. **La home no posiciona bajo `/es`, sino bajo `/`**: sus 13 keywords cuelgan de la URL
   sin idioma y contar solo `/es` da cero. Es el hallazgo T3, no un fallo.
4. **`longitud_campos` del pack de `es-lint` no es el schema de DatoCMS.** Dice «se
   cortará al renderizar» y **no está verificado**: el hero de `/es` tiene 348 caracteres
   con límite de 160 y se renderiza entero. Es regla de redacción, no hecho técnico.
5. **El title aprobado de la home disparaba `MARCA-VIEJA`.** «Dynapps (antes Indaws)» es
   doctrina SEO (P9) y era bloqueante hasta que se añadió la excepción al pack. Si un
   bloqueante contradice al plan, mirar primero cuál de los dos está mal.
6. **El title que muestra Google no es el del CMS.** El 15-sep la home salía como
   «Indaws»: se compara con el snippet real, no solo con el `<title>`.
8. **El texto inglés del CMS lleva espacios duros (U+00A0).** Invisibles en cualquier
   volcado, `JSON.stringify` incluido. En Nobi había cinco en un párrafo, de un pegado
   desde Word o Docs. Sin normalizar `\u00a0` a espacio antes de buscar, cualquier match
   exacto falla: tumbó 14 de 57 reemplazos en la primera pasada.
9. **Que `/es/` salga en inglés no significa que el español esté vacío heredando.** El
   `es-ES` puede tener sus propios bloques con inglés dentro, y hasta algún `alt` ya
   traducido. Se mira el registro; no se deduce de la página.
10. **`status: updated` significa borrador de alguien.** En Nobi era `en-BE`, a medias.
    Comparar `version: 'current'` contra `'published'` locale a locale para saber de
    quién es antes de tocar nada.
11. **El MCP firma con la cuenta que lo autorizó**, no con la de quien pide el cambio.
    Aquí, `elena.soler@dynapps.es`. Se avisa antes de escribir, no después.
12. **Los `alt` que faltan pueden no ser de esta página.** Si viven en el registro del
    *upload*, son globales a los once mercados y quedan fuera de «solo esta página».
    Los que van dentro del bloque sí son del locale y sí se pueden arreglar.

13. **Que te digan «ya está publicado» no significa que TU página lo esté.** El equipo
    publica en lote y el tuyo puede quedarse fuera. Se comprueba con `meta.published_at`
    del registro y comparando `version: 'published'` contra `'current'`, no preguntando.
    Pasó con Nobi el 22-sep: la tanda salió y esa página se quedó dentro sin publicar.
14. **Un registro puede estar publicado y vacío en tu locale.** `firesystem` está
    `published` con **0 bloques** en `es-ES` y en `en-ES`. Publicado no es igual a tener
    contenido; hay que contar bloques por locale.
15. **`en-ES` casi nunca es una versión para España**: en 21 de 24 casos es un **clon
    idéntico** de `en-CH`/`en-BE`/`en-NL`. Si te piden «la inglesa del web español»,
    normalmente no existe como tal: existe el inglés genérico duplicado.
16. **Hay registros que no se pueden publicar por campos sin rellenar.** Se ven con
    `meta.is_valid` y `meta.is_current_version_valid`. No es cosa de la copia: se escala al
    dueño de la página.
17. **Los CTA sin destino son sistémicos, no anecdóticos**: 202 de 5.488 bloques
    `cta_block` sin `page` ni `external_url`. Al entregar una página se comprueban sus
    CTA, no solo el texto. Ojo: un bloque sin destino puede ser huérfano, así que el
    recuento crudo es el techo, no los botones rotos que ve un visitante.
18. **`data/baseline-seo.json` envejece en días.** La colección de casos pasó de **21 a
    30** registros entre el 17 y el 22-sep, y uno tiene el slug **`template`**. Cuenta
    en el CMS antes de afirmar cuántas páginas hay.
19. **El idioma se comprueba sobre `version: 'published'`.** Sobre el borrador te dice
    lo que alguien dejó a medias, no lo que ve el visitante. Es la diferencia entre
    «Nobi está en español» y «el español de Nobi sigue sin publicarse».

## EFICIENCIA

Lo que costó vueltas la primera vez que esta skill escribió en el CMS (18-sep-2026).
No son manías de estilo: cada línea es un round-trip que no hace falta repetir.

- **Ensaya en el sandbox de solo lectura.** `upsert_and_execute_safe_script` bloquea las
  escrituras, así que el mismo script vale de pasada en seco sin riesgo. Cuando cuadra,
  se le añade el `update` con `mode: "patch"` y se relanza con el tool de escritura:
  **el script se guarda por nombre**, así que no hay que reenviar el diccionario entero.
- **Itera con `mode: "patch"`, nunca con `"full"`.** Reenviar doscientas líneas para
  arreglar una es lo que más tokens quema de todo esto.
- **Pide los method tokens de una vez.** `get_api_methods` acepta hasta 20 pares
  `{resource, method}`; pedirlos de uno en uno es una vuelta por método. Y pasa en
  `have` los tokens ya obtenidos para que no te reenvíe documentación que ya tienes.
- **`nested: true` o no hay texto que leer.** Sin él los bloques vienen como ids. Es la
  causa número uno de payloads rotos.
- **En el JSON crudo el contenido cuelga de `attributes`.** Un recorrido que salta esa
  clave «para quitar ruido» devuelve bloques vacíos y parece que el registro no tiene
  contenido. Pasó, y costó una vuelta entera.
- **El sandbox prohíbe `any` y `unknown`.** Para recorrer JSON genérico, declara
  `type Json = string | number | boolean | null | Json[] | { [k: string]: Json }`.
- **Los locales son un tipo literal, no `string`.** `Object.keys(rec.sections)` devuelve
  `string` y el indexado no compila. Declara `const LOCALES = [...] as const` y recorre
  eso.
- **No vuelques el registro entero**: son ~22.000 caracteres por locale, y hay once.
  Vuelca las hojas de texto con su ruta y un `slice`, que es lo único que hace falta
  para armar el diccionario.
- **Un diccionario de cadenas exactas que aborta si alguna no casa** vale más que editar
  campo a campo: una sola pasada dice si has entendido el contenido, y si algo no casa
  todavía no has escrito nada.
- **`items.list` tope a 500 por página y no avisa.** Un recuento sin paginar miente:
  pareció que había 500 bloques `cta_block` en el proyecto y hay **5.488**. Pagina con
  `page: { offset, limit }` hasta que una página venga corta.
- **Una verificación por lista de palabras no sustituye a leer el texto seguido.** El
  barrido de inglés residual daba limpio y aun así quedaba «Modules:» en negrita: no
  contenía ninguna palabra de la lista. Renderiza el locale en orden de lectura y
  míralo antes de dar nada por terminado.

## REFERENCES

- `references/seo-onpage.md` — title/meta/H1/enlaces, competidores reales y cómo re-medir
- `references/calcos-web-dynapps.md` — los calcos confirmados en vivo en este sitio
- `references/entidad-y-asistentes.md` — nombre único de la entidad y párrafos citables por un asistente
- `references/elementos-que-faltan.md` — tablas, figuras, imágenes, citas de personas y el JSON-LD que arrastran
- `references/salida.md` — el formato de entrega
- `references/datocms-escritura.md` — cómo escribir en el CMS sin romper otro mercado
