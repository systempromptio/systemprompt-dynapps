# Knowledge Article HTML Template

Plantilla HTML obligatoria para el campo `body` de los artículos `knowledge.article` de Indaws Core.

**REGLA**: Esta plantilla es OBLIGATORIA. TODOS los artículos de Knowledge Base (ES y EN) deben seguir esta estructura exacta. No omitir ningún elemento aunque el paquete tenga un solo módulo.

---

## Template ES (Español)

```html
<!-- 1. CABECERA — Siempre presente -->
<h1>📦 [CORE] {Nombre del Paquete}</h1>
<p><i>Documentación de usuario — Paquete de Indaws Core (ES)</i></p>

<!-- 2. ÍNDICE DE MÓDULOS — Siempre presente, incluso con 1 solo módulo -->
<div style="background-color: #f8f8f8; border: 1px solid #ddd; border-radius: 6px; padding: 16px 24px; margin: 16px 0;">
<h3 style="margin-top: 0;">📋 Módulos incluidos en este paquete</h3>
<table style="width: 100%; border-collapse: collapse;">
<thead>
<tr style="border-bottom: 2px solid #875A7B;">
<th style="text-align: left; padding: 8px 12px; color: #875A7B;">Módulo</th>
<th style="text-align: left; padding: 8px 12px; color: #875A7B;">Nombre técnico</th>
</tr>
</thead>
<tbody>
<tr style="border-bottom: 1px solid #eee;">
<td style="padding: 8px 12px;"><b>1. {Nombre Comercial Módulo 1}</b></td>
<td style="padding: 8px 12px;"><code style="background-color: #e0e0e0; padding: 2px 6px; border-radius: 3px; font-family: monospace;">{nombre_tecnico_1}</code></td>
</tr>
<!-- Repetir <tr> para cada módulo adicional del paquete -->
</tbody></table></div>

<!-- 3. SEPARADOR 40px — Siempre entre índice y primer módulo -->
<div style="margin: 40px 0;"></div>

<!-- 4. POR CADA MÓDULO: H2 con nombre COMERCIAL + Badge técnico + Contenido -->
<h2>{Nombre Comercial del Módulo}</h2>
<div style="background-color: #f0f0f0; border-left: 4px solid #875A7B; padding: 8px 16px; margin: 8px 0 16px 0; font-size: 14px; border-radius: 0 4px 4px 0;"><b>Nombre técnico:</b> <code style="background-color: #e0e0e0; padding: 2px 6px; border-radius: 3px; font-family: monospace;">{nombre_tecnico}</code></div>

<!-- Contenido convertido del Markdown: descripción, secciones h3, imágenes base64, listas, FAQ -->
<p>{Descripción general del módulo...}</p>
<h3>1. {Primera sección}</h3>
<p>{Contenido...}</p>
<p><img src="data:image/png;base64,{...}" alt="{descripción}" style="max-width: 100%; border: 1px solid #ddd; border-radius: 4px;" /></p>
<!-- ... más contenido ... -->

<!-- 5. SEPARADOR ENTRE MÓDULOS — Solo si hay más de un módulo -->
<div style="margin: 40px 0; border-top: 3px solid #875A7B;"></div>

<!-- Repetir paso 4 para cada módulo adicional -->
```

### Elementos clave ES

| Elemento | Descripción | Obligatorio |
|----------|-------------|-------------|
| `<h1>` | `📦 [CORE] {Nombre del Paquete}` | Siempre |
| Subtítulo | `Documentación de usuario — Paquete de Indaws Core (ES)` en `<i>` | Siempre |
| Índice | Tabla dentro de div gris `#f8f8f8` con cabecera violeta `#875A7B` | Siempre (incluso 1 módulo) |
| H3 del índice | `📋 Módulos incluidos en este paquete` | Siempre |
| Separador | `<div style="margin: 40px 0;"></div>` | Siempre (entre índice y módulo) |
| `<h2>` | Nombre **COMERCIAL** del módulo (NO el nombre técnico) | Siempre |
| Badge | Div con borde izquierdo violeta `#875A7B`, fondo `#f0f0f0` | Siempre |
| Imágenes | Base64 embebido con `max-width: 100%`, borde `#ddd`, border-radius 4px | Siempre |

---

## Template EN (English)

```html
<!-- 1. HEADER -->
<h1>📦 [CORE] {Package Name EN}</h1>
<p><i>End-user documentation — Indaws Core Package (EN)</i></p>

<!-- 2. MODULE INDEX -->
<div style="background-color: #f8f8f8; border: 1px solid #ddd; border-radius: 6px; padding: 16px 24px; margin: 16px 0;">
<h3 style="margin-top: 0;">📋 Modules included in this package</h3>
<table style="width: 100%; border-collapse: collapse;">
<thead>
<tr style="border-bottom: 2px solid #875A7B;">
<th style="text-align: left; padding: 8px 12px; color: #875A7B;">Module</th>
<th style="text-align: left; padding: 8px 12px; color: #875A7B;">Technical name</th>
</tr>
</thead>
<tbody>
<tr style="border-bottom: 1px solid #eee;">
<td style="padding: 8px 12px;"><b>1. {Module Commercial Name EN}</b></td>
<td style="padding: 8px 12px;"><code style="background-color: #e0e0e0; padding: 2px 6px; border-radius: 3px; font-family: monospace;">{technical_name}</code></td>
</tr>
<!-- Repeat <tr> for each additional module -->
</tbody></table></div>

<!-- 3. SPACER -->
<div style="margin: 40px 0;"></div>

<!-- 4. PER MODULE: H2 commercial name + Technical badge + Content -->
<h2>{Module Commercial Name EN}</h2>
<div style="background-color: #f0f0f0; border-left: 4px solid #875A7B; padding: 8px 16px; margin: 8px 0 16px 0; font-size: 14px; border-radius: 0 4px 4px 0;"><b>Technical name:</b> <code style="background-color: #e0e0e0; padding: 2px 6px; border-radius: 3px; font-family: monospace;">{technical_name}</code></div>

<!-- Content converted from English Markdown -->
<p>{Module general description...}</p>
<h3>1. {First section}</h3>
<p>{Content...}</p>
<p><img src="data:image/png;base64,{...}" alt="{description}" style="max-width: 100%; border: 1px solid #ddd; border-radius: 4px;" /></p>

<!-- 5. MODULE SEPARATOR (only if multiple modules) -->
<div style="margin: 40px 0; border-top: 3px solid #875A7B;"></div>
```

### Elementos clave EN

| Element | Description | Required |
|---------|-------------|----------|
| `<h1>` | `📦 [CORE] {Package Name EN}` | Always |
| Subtitle | `End-user documentation — Indaws Core Package (EN)` in `<i>` | Always |
| Index | Table in gray div `#f8f8f8` with violet headers `#875A7B` | Always (even 1 module) |
| H3 index | `📋 Modules included in this package` | Always |
| Spacer | `<div style="margin: 40px 0;"></div>` | Always |
| `<h2>` | **COMMERCIAL** module name in English (NOT technical name) | Always |
| Badge | Div with left violet border `#875A7B`, background `#f0f0f0` | Always |
| Images | Base64 embedded, same screenshots as ES version | Always |

---

## Errores comunes a evitar

### 1. Omitir el índice o el H2 para paquetes de 1 módulo
**INCORRECTO**: Poner solo el badge técnico sin H1, índice ni H2.
**CORRECTO**: Seguir la plantilla completa aunque sea 1 solo módulo.

### 2. Usar el nombre técnico como H2
**INCORRECTO**: `<h2>indaws_hr_timesheet_attendance</h2>`
**CORRECTO**: `<h2>Fichaje por Proyecto</h2>` (nombre comercial del __manifest__.py o traducido)

### 3. Duplicar el nombre técnico en el contenido
**INCORRECTO**: Tener el badge + `> **Nombre técnico:** indaws_xxx` en el contenido.
**CORRECTO**: El nombre técnico aparece SOLO en el badge. Eliminar la línea `> **Nombre técnico:**` del Markdown al convertir.

### 4. Olvidar el separador de 40px entre índice y primer módulo
Siempre poner `<div style="margin: 40px 0;"></div>` entre el cierre del div del índice y el `<h2>` del primer módulo.

### 5. No verificar balance de tags HTML
Antes de publicar, contar opens vs closes de: `<ul>`, `<ol>`, `<li>`, `<p>`, `<h3>`, `<h4>`, `<table>`, `<tr>`, `<td>`, `<div>`.
