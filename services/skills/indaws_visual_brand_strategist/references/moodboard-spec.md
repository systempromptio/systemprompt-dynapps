# Esquema del JSON de moodboard

Esta referencia documenta el JSON que consume `scripts/render_moodboard.py` para generar el HTML moodboard. Úsala cuando construyas el JSON desde el agente, sin tener que leer el script.

Si Python no está disponible, usa el mismo modelo mental para sustituir los placeholders `{{...}}` de `assets/moodboard_template.html`. La correspondencia placeholder ↔ campo JSON es directa (ver tabla al final).

---

## Esquema completo

```json
{
  "business": "Funeraria Familiar 'Los Cipreses'",
  "logo_mark_text": "Los Cipreses",
  "attributes": {
    "items": ["Sereno", "Familiar", "Atemporal"],
    "rationale": "Frase corta justificando por qué estos tres adjetivos y no otros."
  },
  "style_direction": {
    "name": "Editorial con guiño Quiet luxury",
    "rationale": "Frase explicando por qué esta dirección sirve a los tres atributos y al sector."
  },
  "logo": {
    "morphology": "2-3 frases describiendo morfología del logotipo: tipo de marca, geometría o gestualidad, metáfora visual."
  },
  "palette": [
    {
      "name": "Gris Piedra",
      "hex": "#6E6A66",
      "role": "Principal",
      "rationale": "Por qué este color (no otro adyacente) le sirve a este negocio en este sector."
    },
    {
      "name": "Verde Salvia",
      "hex": "#A6B19A",
      "role": "Secundario",
      "rationale": "Justificación sectorial."
    },
    {
      "name": "Marfil",
      "hex": "#F2EFE6",
      "role": "Neutro",
      "rationale": "Justificación sectorial."
    },
    {
      "name": "Terracota Mate",
      "hex": "#B27B5C",
      "role": "Acento",
      "rationale": "Justificación sectorial. (Cuarto color opcional.)"
    }
  ],
  "typography": {
    "heading": {
      "font": "Fraunces",
      "google_font_url": "https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,500;9..144,700&display=swap",
      "css_family": "'Fraunces', serif",
      "classification": "Serif transicional con opsz alto",
      "rationale": "Por qué encaja con la dirección de estilo y los atributos."
    },
    "body": {
      "font": "Inter",
      "google_font_url": "https://fonts.googleapis.com/css2?family=Inter:wght@400;500&display=swap",
      "css_family": "'Inter', sans-serif",
      "classification": "Sans-serif humanista",
      "rationale": "Por qué encaja con la dirección de estilo y los atributos."
    }
  },
  "moodboard": {
    "description": "Párrafo de 4-6 frases describiendo el mundo visual: fotografía, texturas, elementos gráficos auxiliares, referencias culturales."
  }
}
```

---

## Campos

### Obligatorios

- `business` *(string)* — Nombre del negocio o título corto. Se usa como `<h1>` del moodboard y como texto del mockup del logo si `logo_mark_text` no se provee.
- `attributes.items` *(array de 3 strings)* — Los tres adjetivos visuales. Exactamente 3, no más, no menos.
- `attributes.rationale` *(string)* — Frase de justificación de los 3 adjetivos.
- `style_direction.name` *(string)* — Nombre de la dirección (puede incluir guiño secundario: "Minimalista con guiño Editorial").
- `style_direction.rationale` *(string)* — Justificación.
- `logo.morphology` *(string)* — 2-3 frases.
- `palette` *(array de 3 o 4 objetos)* — Cada item con `name`, `hex`, `role` y `rationale`.
- `typography.heading` y `typography.body` *(objetos)* — Cada uno con `font`, `google_font_url`, `css_family`, `classification`, `rationale`.
- `moodboard.description` *(string)* — Párrafo de 4-6 frases.

### Opcionales

- `logo_mark_text` *(string)* — Texto que se renderiza como mockup tipográfico del logo. Si no se provee, se usa `business`.

---

## Reglas importantes

1. **`css_family` debe incluir las comillas internas y el fallback.** Ejemplo correcto: `"'Fraunces', serif"`. Si solo pones `"Fraunces"`, los headings romperán en navegadores que necesiten el fallback.
2. **`google_font_url` debe ser una URL completa y no vacía**, con los pesos que vas a usar en la URL (mínimo `wght@400;600` para que se rendericen h1/h2 y `.logo-mark`). Si combinas heading y body en una sola request, repite la URL completa en ambos campos.
3. **La paleta acepta 3 o 4 colores.** El script renderiza tantos como haya. No envíes más de 4 (rompe el grid).
4. **Si propones una fuente de foundry de pago** (Söhne, GT Sectra, Druk), pon en `google_font_url` y `css_family` una alternativa Google Fonts cercana, y deja claro en el `rationale` que es una sustitución.

---

## Correspondencia con el template HTML fallback

Cuando uses `assets/moodboard_template.html` por no tener Python, la equivalencia es:

| Placeholder | Campo JSON |
|---|---|
| `{{BUSINESS}}` | `business` |
| `{{LOGO_MARK_TEXT}}` | `logo_mark_text` (o `business` si no existe) |
| `{{ATTRIBUTE_1}}` / `{{ATTRIBUTE_2}}` / `{{ATTRIBUTE_3}}` | `attributes.items[0..2]` |
| `{{ATTRIBUTES_RATIONALE}}` | `attributes.rationale` |
| `{{STYLE_NAME}}` | `style_direction.name` |
| `{{STYLE_RATIONALE}}` | `style_direction.rationale` |
| `{{LOGO_MORPHOLOGY}}` | `logo.morphology` |
| `{{COLOR_N_NAME}}` / `{{COLOR_N_HEX}}` / `{{COLOR_N_ROLE}}` / `{{COLOR_N_RATIONALE}}` | `palette[N-1].*` |
| `{{HEADING_FONT}}` / `{{HEADING_FAMILY}}` / `{{HEADING_GFONT_URL}}` / `{{HEADING_CLASSIFICATION}}` / `{{HEADING_RATIONALE}}` | `typography.heading.*` |
| `{{BODY_FONT}}` / `{{BODY_FAMILY}}` / `{{BODY_GFONT_URL}}` / `{{BODY_CLASSIFICATION}}` / `{{BODY_RATIONALE}}` | `typography.body.*` |
| `{{MOODBOARD_DESCRIPTION}}` | `moodboard.description` |

Si solo usas 3 colores, elimina íntegramente el bloque entre los marcadores `<!-- BEGIN_OPTIONAL_COLOR_4 -->` y `<!-- END_OPTIONAL_COLOR_4 -->` del template (marcadores incluidos).
