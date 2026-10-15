---
name: indaws-dynapps-product-onepager
description: >
  Crea la ficha de producto de Dynapps en formato one-pager: una página HTML on-brand,
  en light mode, que resume cualquier producto/módulo con tres preguntas — "¿Qué es?",
  "¿En qué mejora a [base]?" y "¿Por qué lo necesitas?". Usa SIEMPRE esta skill cuando
  alguien pida una ficha de producto, one-pager, hoja de producto, ficha comercial,
  resumen de un módulo Dynapps Core, "vende este producto en una página", o quiera
  presentar un producto/servicio Dynapps de forma comercial en una sola vista —
  aunque no diga literalmente "one-pager". Garantiza que todas las fichas del equipo
  compartan la misma estructura, retícula y marca. NO uses esta skill para páginas web
  largas multi-sección (eso es otra pieza) ni para .docx/.pptx/.pdf.
version: 1.1.0
model: sonnet
effort: low
---

# Ficha de producto Dynapps — one-pager

Esta skill produce la **ficha de producto oficial de Dynapps en formato one-pager**: una
sola página HTML autocontenida, pensada para leerse de un vistazo, enviarse por email o
publicarse como enlace. Su objetivo es que **todos los productos del catálogo tengan la
misma plantilla**: misma estructura, misma retícula, misma voz visual.

El corazón de la ficha son **tres preguntas** que traducen lo técnico a lenguaje de negocio:

1. **¿Qué es?** — qué es el producto, sobre qué se construye, para quién.
2. **¿En qué mejora a [Odoo / el estándar / el producto base]?** — las mejoras concretas frente a lo que ya existe.
3. **¿Por qué lo necesitas?** — el dolor que resuelve y por qué merece la pena.

Todo el detalle técnico debe quedar **resumido dentro de esas tres preguntas** (y como mucho
en una línea final con los nombres de módulos). Si te sobra información, va fuera: la ficha
comunica valor, no documenta.

## Cuándo usarla

Cuando el usuario quiere presentar **un** producto o módulo de forma comercial en una página.
Si pide comparar varios productos, un catálogo completo, o una web larga navegable con
secciones y menú, esta no es la pieza (dilo y ofrece la alternativa).

## Workflow

**Paso 1 — Reúne el contenido primero.** Necesitas, como mínimo: nombre del producto, sobre qué se
   construye (Odoo, un módulo base, etc.), 3-6 mejoras o funcionalidades clave, el cliente
   ideal / dolor que resuelve, y los datos de contacto (teléfono y email de demo). Si falta
   algo, pídelo o proponlo y confírmalo — no inventes cifras.

**Paso 2 — Redacta el copy comercial** en tono de valor, en español, **en caja baja o frase**.
   Reparte el contenido en las tres preguntas. Deja lo técnico (nombres de módulos, campos,
   métodos) resumido; los nombres de módulo van en la línea `tech_line` con `<code>`.

**Paso 3 — Rellena el contenido** en un JSON siguiendo `examples/firma-digital-avanzada.json`.

**Paso 4 — Genera el HTML** con el script (embebe el logo en base64 y sustituye los marcadores):

   ```bash
   python scripts/build_onepager.py mi-producto.json ficha-mi-producto.html
   ```

   Nombra el archivo en kebab-case descriptivo (`ficha-<producto>.html`), nunca `index.html`.

**Paso 5 — Revisa contra el checklist** de `references/design-rules.md` antes de entregar.

Alternativamente puedes editar `assets/template.html` a mano sustituyendo los marcadores
`{{...}}`, pero el script es más rápido y evita errores (sobre todo el base64 del logo y el
URL-encoding del asunto del mailto).

Las reglas de diseño —light mode, cero mayúsculas sostenidas, el degradado dosificado, la
estructura fija— y el checklist de entrega están en `references/design-rules.md`. Son la
identidad de la ficha, no preferencias: léelas antes de editar el template a mano.

**El teléfono y el email de demo NO se inventan ni se copian de otra ficha**: son hechos de
empresa y su sitio único es `references/shared/dynapps-identity.md` (wrapper de
`dynapps-identity.json`). Una ficha comercial con un teléfono viejo llega al cliente.

## GOTCHAS

Los cuatro salen de leer `scripts/build_onepager.py`, y los cuatro fallan **en silencio**: el
script termina con `OK ->` y el HTML sale mal.

1. **Un `logo_path` que no resuelve cae al logo de Dynapps Core sin avisar** (`:71-75`). La ruta
   relativa se resuelve contra el directorio del **JSON**, no contra donde ejecutas el comando.
   Escribes la ruta del logo de otro producto, te equivocas, y la ficha sale con el logo
   genérico y un `OK` en pantalla. Comprueba el logo en el HTML, no en la salida del script.
2. **`demo_subject` se URL-encodea siempre** (`:81`). Si lo pones ya codificado en el JSON se
   codifica dos veces: un `%20` acaba como `%2520` y el asunto del correo llega con basura.
   Escríbelo en texto plano.
3. **Un campo del JSON que no esté en la lista `keys` se ignora sin error** (`:83-91`). Inventar
   un nombre de campo no rompe nada: simplemente no aparece. Si algo no sale, el sospechoso
   número uno es el nombre del campo.
4. **Solo tres campos son obligatorios** (`product_title`, `product_title_accent`, `tagline`,
   `:67-69`). Cualquier otro que falte se sustituye por cadena vacía, así que la ficha se genera
   con huecos y hay que verlos mirando el HTML.

## NEVER / ALWAYS

- **NEVER** entregar sin abrir el HTML generado. Los cuatro fallos de arriba solo se ven ahí.
- **NEVER** inventar cifras, clientes o resultados para rellenar; si falta el dato, pídelo.
- **NEVER** nombrar el fichero `index.html` — kebab-case descriptivo, `ficha-<producto>.html`.
- **ALWAYS** sacar teléfono, email y dominios de `dynapps-identity`, nunca de otra ficha.
- **ALWAYS** validar el resultado contra `dynapps-brand` antes de mandarlo a un cliente.

## Agent routing

Todo el flujo corre en el modelo de la sesión (`sonnet`/`low`): redactar copy comercial breve y
ejecutar un script determinista no justifica escalar. **Sin subagentes** — la ficha es una sola
página y partirla entre contextos solo rompe la coherencia de tono.

## Delegates to

`common-skills:dynapps-brand` — fuente de verdad de color, tipografía y logo, y validación del
copy antes de entregar. Los hechos de empresa (teléfono, email, dominios) salen de su referencia
compartida `references/shared/dynapps-identity.md`.

## Consumed by

**Nadie hoy.** Es una skill de entrada: la invoca una persona que necesita la ficha. Se dice en
vez de dejarlo en blanco, que se leería como "pendiente de rellenar".
