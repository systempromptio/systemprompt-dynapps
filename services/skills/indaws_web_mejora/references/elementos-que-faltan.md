# Elementos que faltan: tablas, figuras e imágenes

Arreglar lo que hay no basta cuando lo que falta es la mitad. `/es/campaigns/que-es-odoo`
—la página que concentra el 88,5 % del ETV de todo el sitio— tiene **cero tablas, cero
figuras, diez preguntas en sus encabezados y ningún JSON-LD**. Diez preguntas sin marcar
es la pieza más citable del sitio invisible para quien la citaría.

`scripts/extraer.py` cuenta lo que la página ya tiene. Esta referencia dice qué proponer
cuando ese recuento sale corto, y qué **no** proponer.

## La regla que decide

Un elemento nuevo entra si **responde una pregunta que la página ya provoca** o si
**lleva un dato que en prosa se lee peor**. No entra para «romper el texto», ni para
rellenar, ni porque la página parezca sosa. Un elemento decorativo es peso que carga el
usuario en su móvil.

Y un tope: **uno o dos elementos nuevos por página y pasada**. El plan limita a 20
cambios por semana para poder medir qué movió qué; una página que estrena cinco cosas a
la vez no enseña nada.

## Tablas — lo más rentable, y con diferencia

Son texto: se pegan como un párrafo, no hay que producir nada, y son lo que los
asistentes citan literalmente. Además atacan un problema medido: varias páginas `/es`
andan por 400-700 palabras cuando el objetivo es más de 800, y una tabla añade sustancia
donde otro párrafo solo añadiría relleno.

| Tipo de página | Tabla que suele faltar |
|---|---|
| Producto / campaña | Qué incluye de serie, qué necesita desarrollo |
| Comparativa | La comparación que el texto **ya afirma** (la home dice que Odoo gana a SAP, Dynamics y NetSuite en tres criterios y no lo sostiene con nada) |
| Precios | Rangos por alcance, con lo que entra y lo que no |
| Versiones | Odoo 17 vs 18 vs 19 — lo pide el propio plan (C11) |
| Servicio | Qué hace cada servicio, para quién y qué entrega |
| Caso de éxito | Antes / después con cifras del cliente |

Cómo se escribe una que sirva:

- **Cabecera con significado.** «Criterio | Odoo | SAP» sirve; «Columna 1 | Columna 2» no.
- **Máximo seis columnas**, o en el móvil no se lee. Si hacen falta más, son dos tablas.
- **Unidades y fecha en la celda**, no en una nota al pie: la celda es lo que se cita.
- **Cada cifra, con fuente.** Si no está en la hoja canónica (P11 del plan) o en un dato
  del cliente publicado, no se pone.
- **Nada de celdas vacías.** Un guion es una respuesta; un hueco parece un error.
- La frase anterior a la tabla dice qué se va a ver. Esa frase es la que se indexa.

## Preguntas sin marcar

Si la página tiene H2 o H3 terminados en `?`, ya es un FAQ: solo le falta el marcado. Se
entrega el JSON-LD escrito, con las preguntas **tal cual están en la página** y la
respuesta en una o dos frases que se sostengan solas.

```html
<script type="application/ld+json">
{"@context":"https://schema.org","@type":"FAQPage","mainEntity":[
 {"@type":"Question","name":"<la pregunta exacta del H2>",
  "acceptedAnswer":{"@type":"Answer","text":"<respuesta de 1-2 frases>"}}
]}
</script>
```

Ojo: el marcado tiene que decir **lo mismo** que la página. Un FAQPage con respuestas que
no están en el texto visible es motivo de acción manual de Google, no un atajo.

## Figuras y gráficos

Un gráfico es una imagen: lo que cuenta **no se indexa**. Por eso nunca va solo.

Se propone con esta ficha, y siempre con su tabla equivalente debajo:

```
FIGURA · dónde va (sección, tras qué bloque)
Qué muestra:   una frase
Datos:         las cifras exactas, con su fuente
Formato:       SVG inline, legible en claro y oscuro, sin texto dentro menor de 12 px
Alt:           el texto alternativo
Tabla gemela:  la misma información en filas, debajo de la figura
```

Si los datos no se pueden citar, no hay figura. Un gráfico inventado es peor que ninguno.

## Imágenes: se encargan, no se proponen

La skill no ve las fotos y no puede decir qué sale en ellas. Lo que entrega es el encargo:

```
IMAGEN · dónde va
Qué debe mostrar:  ...
Nombre de fichero: minúsculas-con-guiones.webp
Alt:               ...
Quién la produce:  marketing / el cliente / captura de Odoo
```

Para capturas de pantalla de Odoo no se improvisa: existe la skill
`consultant-skills:odoo-screenshot-generator` y se delega en ella.

## Personas y citas

Una página entera sin que hable nadie es la señal más clara de que la escribió un
traductor. Y para un asistente, una frase con nombre y cargo detrás vale más que tres
párrafos en primera persona del plural: es lo único atribuible.

Cuándo falta una cita, en orden de urgencia:

| Página | Quién habla | Qué aporta |
|---|---|---|
| Caso de éxito | El cliente, con nombre y cargo | Es la página; sin su voz es una ficha técnica |
| Servicio | El responsable de ese servicio en Dynapps | Pone cara a quien lo va a prestar |
| Sector | Un cliente de ese sector | Demuestra que se conoce el sector, no solo el software |
| Blog | El autor, con su cargo | Es lo que Google llama experiencia y lo que `Article.author` necesita |
| Quiénes somos | El equipo, con nombre y foto | Hoy la página habla de «un equipo local» sin un solo nombre |

Y las reglas, que aquí son duras:

- **Una cita se transcribe, no se escribe.** Si no existe la frase, no hay cita: se
  propone el encargo («pedir a X dos frases sobre Y»), nunca un texto de ejemplo con
  pinta de real. Inventar el testimonio de un cliente es fabricar una declaración de
  otro, y en una web pública eso es un problema legal antes que de estilo.
- **Con nombre, cargo y empresa.** «Un cliente del sector industrial» no es atribuible y
  no lo cita nadie. Si el cliente no quiere aparecer, no hay cita: hay un dato.
- **Consentimiento antes de publicar**, tanto de la frase como de la foto, y del logo si
  va al lado. Se marca en la entrega como requisito, igual que el calendario editorial
  bloquea una pieza con la imagen del cliente sin consentimiento.
- **El linter no toca lo citado y es a propósito.** `es-lint` se salta las comillas: un
  testimonio se queda como lo dijo su autor, con sus muletillas y su sintaxis. Corregirle
  el estilo a un cliente es convertir su cita en tu copia.
- **Nada de marcar `Review` ni `AggregateRating`** en las citas sobre Dynapps en la web
  de Dynapps. Las reseñas sobre uno mismo no son elegibles para resultados enriquecidos y
  marcarlas es buscarse una acción manual. La cita va como texto y, si acaso, dentro del
  `Article` o del caso; las estrellas viven en sitios de terceros.
- Tres candidatos reales ya publicados y con caso propio: **Prima Protección**,
  **Plastimyr** y **Tunergia**. Si una de esas páginas no lleva la voz de su cliente, ese
  es el primer hueco que hay que tapar.

## Textos alternativos

- **Solo se escribe el alt cuando el contenido se deduce** del contexto: el caso de
  éxito, la copia alrededor, el nombre del fichero. Si es una foto que nadie ha mirado,
  se entrega marcada **«hay que verla»**. Inventar lo que muestra una imagen es mentir en
  el sitio donde solo leen quienes no pueden verla.
- **Los iconos decorativos llevan `alt=""` a propósito.** En la home, cinco iconos de
  sector van junto a su etiqueta de texto: describirlos hace que un lector de pantalla
  diga dos veces lo mismo.
- Descriptivo y concreto, **hasta unos 125 caracteres**. Sin «imagen de» ni «foto de»:
  el lector de pantalla ya anuncia que es una imagen.
- Sin meter la keyword a la fuerza. Si encaja de forma natural, bien; si no, fuera.
- El alt de una imagen de DatoCMS se edita **en el asset**, no en la página: cambiarlo
  afecta a todas las páginas que lo usan. Eso se avisa en la entrega.

## Lo que arrastra, y no se pega

El marcado estructurado es código y va al bloque **«esto no se arregla pegando»**, con el
JSON-LD ya escrito para que el equipo web solo lo copie. El sitio tiene **cero** hoy, así
que casi siempre hay algo que llevarse:

| Dónde | Tipo |
|---|---|
| Home y `/es/about` | `Organization` + `LocalBusiness` (Valencia, con `sameAs` al perfil de odoo.com) |
| Páginas de servicio | `Service` |
| Páginas con preguntas | `FAQPage` |
| Blog | `Article` con `datePublished` y `author` |
| Todas | `BreadcrumbList` |
