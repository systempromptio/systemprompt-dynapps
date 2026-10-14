# Email Templates — Indaws Content Creation

**Technical reference**: Delegate to `email-campaign-builder` for HTML templates, Odoo fields (`body_arch`, `body_html`), and format rules.

## Newsletter Structure

```
┌────────────────────────────────────────────────────────────────┐
│  SUBJECT LINE (max 50 caracteres)                               │
│  Ejemplo: "3 formas de acelerar tu cierre contable"             │
├────────────────────────────────────────────────────────────────┤
│  PREVIEW TEXT (40-90 caracteres)                                 │
│  Complementa el subject, no lo repite                           │
│  Ejemplo: "La tercera te sorprendera. Caso real incluido."      │
├────────────────────────────────────────────────────────────────┤
│  HEADER                                                          │
│  - Logo Indaws                                                   │
│  - Saludo personalizado: "Hola {{object.name}},"                 │
├────────────────────────────────────────────────────────────────┤
│  BLOQUE 1: Contenido destacado                                   │
│  - Titulo + 2-3 lineas + CTA inline                              │
├────────────────────────────────────────────────────────────────┤
│  BLOQUE 2: Contenido secundario                                  │
│  - Titulo + 2-3 lineas + link                                    │
├────────────────────────────────────────────────────────────────┤
│  BLOQUE 3: Recurso o evento                                      │
│  - Titulo + 1-2 lineas + link                                    │
├────────────────────────────────────────────────────────────────┤
│  CTA PRINCIPAL                                                    │
│  - Boton Blue Lilac con texto de accion                          │
├────────────────────────────────────────────────────────────────┤
│  FOOTER                                                           │
│  - Redes sociales, datos contacto, legal, unsubscribe            │
└────────────────────────────────────────────────────────────────┘
```

## Email Channel Writing Guide

| Aspecto | Guia |
|---------|------|
| Longitud | 150-300 palabras por email |
| Tono | Directo, personal ("tu"), orientado a accion |
| Estructura | Scannable: frases cortas, bullets, un solo tema |
| Subject | Max 50 caracteres, sin clickbait, sin emojis |
| CTA | Un solo CTA principal por email. Claro y especifico |
| Personalizacion | Usar campos Odoo ({{object.name}}, {{object.company_id.name}}) |
| Frecuencia | Newsletter 2x/mes. Secuencias: segun flujo |

## Odoo Personalization Fields

| Campo | Descripcion |
|-------|-------------|
| `{{object.name}}` | Nombre del contacto |
| `{{object.company_id.name}}` | Nombre de la empresa |
| `{{object.email}}` | Email del contacto |
| `{{object.phone}}` | Telefono del contacto |
