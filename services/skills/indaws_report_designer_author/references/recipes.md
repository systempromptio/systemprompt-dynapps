# Recetas — editor visual `indaws_report_designer`

Paso a paso de cada elemento. **El menú contextual (clic derecho sobre el papel)** es la
vía principal; también `＋ Insertar ▾` en la barra y el comando **"/"**.

## Arranque

1. Crea un `indaws.report.designer`: **Modelo** (sobre qué imprime) + **Nombre**.
2. Abre el editor.
3. Arriba, elige un **registro de vista previa** con datos reales.

## Campo (dato del documento)

1. Cursor donde quieras el dato → `＋ Insertar ▸ Campo` (o `/campo`, o panel **☰ Campos**).
2. Elige el campo. Si es una relación (m2o), aparece un **drill-down**: `partner_id ▸ name`,
   `partner_id ▸ country_id ▸ name`, etc. **Entra siempre** hasta un dato simple.
3. Queda una pastilla. Clic en ella → panel **Propiedades**: cambiar campo, etiqueta,
   formato/widget, ocultar si vacío, mostrar-si, estilo-si.

## Tabla de registros (líneas que se repiten)

1. `＋ Insertar ▸ Tabla de registros`.
2. Elige el **one2many** (p.ej. `order_line`, `invoice_line_ids`).
3. Se inserta una fila que se repetirá por cada línea. Para poner datos de la línea, sigue
   con "Campo dentro de la tabla".

## Campo dentro de la tabla

- Pon el cursor **en una celda de la tabla de registros** y luego inserta el campo: el
  ámbito cambia automáticamente a la **línea** (verás campos como `product_id`, `quantity`,
  `price_subtotal`). Fuera de la tabla, esos campos no existen.

## Columnas de la tabla

- Clic derecho en la tabla → **Añadir columna** / **Quitar columna**.
- **Añadir/Quitar fila** y **Combinar celdas** (seleccionar celdas → combinar; o con la de
  la derecha = colspan / con la de abajo = rowspan).

## Bordes (estilo Word)

- Selecciona celda/tabla → panel **Celda/Tabla** → **pluma**: elige grosor (incluye "Quitar")
  y color, y **haz clic en el lado** (arriba/abajo/izq/der) o en "todos". Se aplica al instante.

## Condiciones

- **Columna** (mostrar/ocultar una columna entera): clic derecho en una celda de la tabla →
  **Condición de columna…** → cuantificador (alguna/todas las líneas o documento) + campo +
  operador + valor. La columna aparece/desaparece en el PDF.
- **Fila / sección** (mostrar/ocultar una línea según su contenido): panel **Fila** →
  **Condición de fila** (útil para notas/secciones intercaladas).
- **Documento** (mostrar/ocultar un bloque cualquiera): clic en el elemento → panel
  **Propiedades** → **Mostrar si**.

## Agrupar + subtotales

- En la tabla de registros, pestaña **Agrupar** → elige el campo por el que agrupar. Se
  añaden cabeceras de grupo y **subtotales automáticos** de las columnas numéricas.

## Cabecera / pie

- Pincha en la **zona de cabecera** (arriba) o **pie** (abajo) y edítala como el cuerpo
  (puedes meter campos, logo, nº de página).
- Modo de layout (barra): **custom** (solo tu diseño), **odoo** (cabecera de empresa
  estándar) o **none** (sin nada).
- **Nº de página**: `＋ Insertar ▸ Nº de página` (normalmente en el pie).

## QR / código de barras

- `＋ Insertar ▸ Código` → tipo (**QR**, **Code128**, **EAN13**) + **valor** (un campo o una
  fórmula) + tamaño.

## Imagen

- `＋ Insertar ▸ Imagen`: subir un fichero, **o** insertar un campo de tipo imagen (logo,
  foto de producto…) vía Campo.

## Nº de línea

- Dentro de la tabla → `＋ Insertar ▸ Nº de línea` (numera 1, 2, 3… por línea).

## Expresión (fórmula)

- `＋ Insertar ▸ Expresión`. Ejemplos:
  - `sum(l.price_subtotal for l in doc.order_line)` — suma de subtotales.
  - `len(doc.order_line)` — nº de líneas.
  - `format_lang(doc.amount_total, currency_obj=doc.currency_id)` — importe formateado.
- Botón **Probar** valida antes de guardar. Solo se permiten cálculos seguros (no escribe nada).

## Función guardada

- `＋ Insertar ▸ Función`: usa una función que un **administrador** haya definido
  previamente (menú Report Designer › Funciones). Tú solo la insertas.

## Texto traducible

- `＋ Insertar ▸ Texto traducible`: escribes el texto en cada idioma activo; en el PDF sale
  el del idioma del documento (con un texto por defecto de reserva).

## Maquetación (respetando el motor de PDF)

- **Columnas**: `＋ Insertar ▸ Columnas` (50/50, 3 iguales, 30/70, 25/50/25) — se hace con
  una tabla de maquetación (lo correcto para el motor de PDF).
- **Caja/sección**: un recuadro con borde/padding; clic derecho dentro → partir en 2
  columnas / quitar caja / eliminar.
- **Saltos de página**: insertar salto, saltar tras un bloque, o "no partir bloque".
- ⚠️ No esperes Flexbox/Grid: para poner cosas en columnas usa **Columnas** o tablas.

## Formato / visibilidad de un campo (panel Propiedades)

Clic en el chip → panel derecho:
- **Formato/Widget**: moneda, fecha, porcentaje, imagen… según el tipo del campo.
- **Ocultar si vacío**: no muestra el campo si no tiene valor (ojo: 0 cuenta como vacío).
- **Visible para grupos**: solo ciertos grupos de usuarios lo ven.
- **Mostrar si / Estilo si**: visibilidad o estilo condicional.

## Publicar

1. **Vista previa** (paged.js): aproximada, para ir viendo.
2. **🖨 Imprimir prueba**: genera el **PDF real** del registro elegido → esto es lo fiable.
3. **Generar**: crea el `ir.actions.report` → el informe queda disponible como impresión de
   ese modelo en Odoo.
4. **Plantillas**: Guardar como plantilla / Duplicar / aplicar plantilla.
5. **Historial** (`↶ Historial ▾`): volver a una versión anterior.
6. **QWeb fuente** (avanzado): ver/editar el código generado.

Made with report-designer-author · v0.1.0
