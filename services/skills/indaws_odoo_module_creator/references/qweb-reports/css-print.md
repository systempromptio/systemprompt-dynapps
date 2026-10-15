# CSS para Reports QWeb-PDF

## Cómo cargar CSS custom

En `__manifest__.py`:
```python
'assets': {
    'web.report_assets_common': [
        'my_module/static/src/scss/report.scss',
    ],
},
```

En v14 (legacy, pre-asset bundles):
```xml
<template id="report_assets_inherit" inherit_id="web.report_assets_common">
    <xpath expr="link[last()]" position="after">
        <link rel="stylesheet" type="text/scss" href="/my_module/static/src/scss/report.scss"/>
    </xpath>
</template>
```

## Reglas fundamentales

1. **No hay `@media print`** — wkhtmltopdf renderiza TODO como "print". Todo el CSS aplica.
2. **Bootstrap está disponible** — BS4 en v14-v15, BS5 en v16+. Usa sus clases.
3. **Unidades**: mm o cm para márgenes y page layout, pt para font-size, px para bordes finos.
4. **Fuentes**: solo fuentes incluidas en el bundle. Google Fonts no se cargan (sin acceso externo).

## Page Breaks

```scss
// Evitar corte dentro de un elemento
.no-break {
    page-break-inside: avoid;
}

// Forzar nueva página antes
.page-break-before {
    page-break-before: always;
}

// Forzar nueva página después
.page-break-after {
    page-break-after: always;
}

// Aplicar a filas de tabla para evitar corte
table tr {
    page-break-inside: avoid;
}
```

Uso en QWeb:
```xml
<tr class="no-break">
    <td>This row won't be split across pages</td>
</tr>
<div style="page-break-before: always;"/>
```

## Tablas

```scss
// Tabla con bordes limpios
.o_main_table {
    width: 100%;
    border-collapse: collapse;

    th {
        background-color: #f8f9fa;
        border-bottom: 2px solid #dee2e6;
        padding: 4px 8px;
        font-size: 9pt;
    }

    td {
        padding: 4px 8px;
        border-bottom: 1px solid #dee2e6;
        font-size: 9pt;
        vertical-align: top;
    }

    // Repeat header on every page
    thead {
        display: table-header-group;
    }

    // Don't break rows
    tr {
        page-break-inside: avoid;
    }
}
```

`thead { display: table-header-group; }` repite el header de la tabla en cada página — imprescindible para tablas largas.

## Fonts y tamaños

```scss
// Tamaños recomendados para PDF
.page {
    font-size: 10pt;       // Cuerpo
    line-height: 1.4;
}

h2 { font-size: 14pt; }   // Título documento
h3 { font-size: 12pt; }   // Subtítulos

.small-text {
    font-size: 8pt;        // Notas, footer
}

// Monospace para referencias, códigos
.ref-code {
    font-family: monospace;
    font-size: 9pt;
}
```

## Márgenes y espaciado

```scss
// El div.page ya tiene márgenes del paper format
// Solo añadir espaciado interno
.page {
    // NO añadir margin/padding al .page — los márgenes vienen del paper format

    // Espaciado entre secciones
    .section {
        margin-bottom: 15mm;
    }
}

// Forzar contenido al fondo de página (para firmas, etc.)
.push-to-bottom {
    position: absolute;
    bottom: 0;
    width: 100%;
}
```

## Colores

```scss
// Usar colores que se impriman bien en blanco y negro
$report-header-bg: #f8f9fa;        // Gris muy claro
$report-border: #dee2e6;           // Gris borde
$report-text: #212529;             // Negro casi puro
$report-text-muted: #6c757d;      // Gris texto secundario
$report-accent: #495057;           // Gris oscuro para énfasis

// EVITAR colores claros como fondo — se pierden en impresión
// EVITAR transparencias — wkhtmltopdf las renderiza inconsistentemente
```

## Imágenes

```scss
// Logo empresa
.company-logo {
    max-height: 60px;
    max-width: 200px;
}

// Imagen producto en línea de tabla
.product-image {
    max-height: 40px;
    max-width: 40px;
    object-fit: contain;
}

// Código de barras
.barcode {
    height: 30px;
}
```

## Patterns para documentos específicos

### Factura / Albarán (2 columnas info)

```scss
.report-info-block {
    display: flex;
    flex-wrap: wrap;
    gap: 15px;
    margin-bottom: 20px;

    .info-item {
        min-width: 120px;

        strong {
            display: block;
            font-size: 8pt;
            color: $report-text-muted;
            text-transform: uppercase;
        }
    }
}
```

### Etiquetas (multi-column)

```scss
.label-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);  // 3 columnas
    gap: 2mm;

    .label-item {
        border: 1px solid #000;
        padding: 2mm;
        height: 30mm;
        width: 60mm;
        page-break-inside: avoid;
    }
}
```

### Landscape con tabla ancha

```scss
// Forzar font más pequeño para caber en landscape
.landscape-table {
    font-size: 8pt;

    th, td {
        padding: 2px 4px;
        white-space: nowrap;
    }
}
```

## Debugging CSS en reports

1. Renderizar como HTML primero: añadir `?debug=assets` a la URL
2. Usar report_type `qweb-html` temporalmente durante desarrollo
3. Inspeccionar con DevTools del navegador antes de generar PDF
4. Verificar que `thead { display: table-header-group; }` funciona (wkhtmltopdf a veces lo ignora con tablas muy complejas)
