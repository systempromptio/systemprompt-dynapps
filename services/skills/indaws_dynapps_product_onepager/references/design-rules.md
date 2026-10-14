# Reglas de diseño y checklist de entrega

Lo que hace que todas las fichas se reconozcan como Dynapps. Ya está implementado en
`assets/template.html`; esto explica **por qué** es así, para que al editar no se rompa.

## Reglas de diseño (no negociables — son la identidad de la ficha)

Estas decisiones son lo que hace que todas las fichas se reconozcan como Dynapps. La skill
`dynapps-brand` es la fuente de verdad de color/tipografía/logo; aquí se aplican así:

- **Light mode siempre.** Fondo Snow (`#F5F5FA`), tarjetas en blanco. Nada de fondos
  oscuros. El negro azulado Midnight (`#050543`) es la voz principal del texto.
- **Sin mayúsculas sostenidas.** Ni en titulares, ni en etiquetas, ni en botones. Solo se
  permiten acrónimos propios (PDF, URL, CRM…). Todo lo demás en caja baja o frase.
- **El degradado de marca, dosificado.** El degradado 45° (Unicorn Violet → Ferrarri
  Carmine → Orange Oefans) se reserva para elementos pequeños: el punto del badge, los
  números de las preguntas, los bullets, el botón primario y la barra final. Nunca como
  fondo grande ni sobre texto de cuerpo.
- **Jerarquía por tamaño y peso, no por color.** Titular grande en Readex Pro Bold con la
  palabra clave en Carmine; cuerpo en gris medio. Toda la tipografía es Readex Pro.
- **Estructura fija:** cabecera (logo arriba a la izquierda + teléfono + botón demo) →
  intro (badge, titular con acento, bajada de una línea) → tres tarjetas iguales con las
  preguntas numeradas → pie con gancho + dos botones → línea técnica → barra de degradado.
- **Dos botones, mismo peso visual, siempre presentes:** *Llamar* (secundario, `tel:`) y
  *Solicitar demo* (primario, `mailto:`). En móvil se apilan a ancho completo y con el
  mismo tamaño; el texto no se parte (`white-space:nowrap`).
- **Cabe en una vista.** El contenedor usa `min-height:100vh` con flex en columna para
  repartir los bloques; no metas tanto texto que obligue a hacer scroll largo.

Estas reglas están ya implementadas en `assets/template.html`; respétalas al editar y no
introduzcas colores, tipografías ni fondos fuera de este sistema.

## Contenido de los campos

Ver `examples/firma-digital-avanzada.json` para un ejemplo completo. Notas:

- `product_title` + `product_title_accent`: el titular se parte en dos; la parte "accent"
  se pinta en Carmine. Elige como acento la palabra que más vende (ej. "avanzada").
- `q2_title`: adáptalo al producto base — "¿En qué mejora a Odoo?", "¿En qué mejora al
  estándar?", etc.
- Los `*_body` admiten HTML: `<p>…</p>` para párrafos, `<ul><li>…</li></ul>` para listas
  (los bullets ya llevan el degradado). Resalta 2-3 términos por bloque con `<strong>`,
  no más — el exceso de negrita mata la jerarquía.
- Varía el formato entre tarjetas (p. ej. párrafos en la 1 y la 3, lista en la 2) para dar
  ritmo visual, como en el ejemplo.
- `logo_path`: por defecto usa el logo Dynapps Core incluido. Para otro producto/marca,
  apunta a otro PNG/SVG.

## Checklist antes de entregar

- [ ] Light mode, sin fondos oscuros
- [ ] Cero mayúsculas sostenidas (salvo acrónimos)
- [ ] Las tres preguntas presentes y con lo técnico resumido dentro
- [ ] Titular con la palabra-acento en Carmine
- [ ] Logo arriba a la izquierda; degradado solo en elementos pequeños
- [ ] Dos botones (llamar + demo) con teléfono y email correctos
- [ ] Cabe en una vista, sin scroll largo
- [ ] Responsive: en móvil las tarjetas y los botones se apilan a ancho completo
- [ ] Marca coherente con `dynapps-brand` (Readex Pro, paleta, logo)
