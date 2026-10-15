# Lenguaje de chips — contrato para generar informes con IA

Este es el **contrato que la IA emite** para que `indaws_report_designer` produzca un
informe. La IA escribe **HTML normal** salpicado de "chips" (`<span data-*>` / `<tr data-loop>`);
el addon los traduce a QWeb/DOCX. La IA **no** escribe QWeb ni conoce wkhtml.

Reparto: **IA = el grueso** (estructura + chips), **humano = retoque visual** en el editor.

## Regla de oro: `doc` vs `line`

- Fuera de una tabla de registros, los campos son del **documento** (el registro que se
  imprime, p.ej. `sale.order`).
- Dentro de un `<tr data-loop="...">`, los campos son de la **línea** (el comodel del o2m).
- Un campo de línea (`price_subtotal`, `product_uom_qty`) **fuera** del loop → error de validación.

## Vocabulario de chips

Todos los chips llevan `class="o-ird-field" contenteditable="false"` (salvo tabla/loop). El
texto interior es solo la etiqueta visible en el editor; se sustituye al imprimir.

| Quiero | Chip | Ejemplo |
|--------|------|---------|
| Campo del documento/línea (drill-down con puntos) | `data-field="path"` | `<span class="o-ird-field" contenteditable="false" data-field="partner_id.name">Cliente</span>` |
| …con formato | `+ data-widget="monetary\|date\|percentage\|integer\|float"` | `data-field="amount_total" data-widget="monetary"` |
| …imagen (campo binary) | `+ data-image="1"` | `data-field="product_id.image_128" data-image="1"` |
| …ocultar si vacío | `+ data-hide-empty="1"` | |
| …solo para grupos | `+ data-groups="base.group_user"` | |
| Datos de la empresa emisora | `data-company="path"` (de `res.company`) | `data-company="vat"` · logo: `data-company="logo" data-image="1"` |
| Fecha impresión / usuario | `data-ctx="date\|datetime\|user"` | `<span class="o-ird-field" contenteditable="false" data-ctx="date">hoy</span>` |
| Importe en letra | `data-amount-words="campo_importe"` | `data-amount-words="amount_total"` |
| Expresión / fórmula (segura) | `data-expr="..."` | `data-expr="sum(doc.order_line.mapped('price_subtotal'))"` |
| Función con nombre (helper) | `data-helper="nombre"` | (debe existir en el modelo; ver validación) |
| Nº de línea (dentro de loop) | `data-linenum="1"` | |
| QR / código de barras | `data-barcode="QR\|Code128\|EAN13" data-bc-value="expr" data-bc-w="120" data-bc-h="120"` | `data-barcode="QR" data-bc-value="doc.name"` |
| Texto traducible | `data-i18n='{"en_US":"Quotation"}' data-i18n-def="Presupuesto"` | |
| Gráfico (sobre un o2m) | `<div class="o-ird-chart" contenteditable="false" data-chart='{"type":"bar","loop":"order_line","group":"product_id","measure":"price_subtotal"}'>Gráfico</div>` | |
| Nº de página (en cabecera/pie) | `<span class="o-ird-pagenum" contenteditable="false" data-pagenum="1">Pág</span>` | |
| Salto de página | `<div class="ird-pagebreak" contenteditable="false" data-pagebreak="1">Salto</div>` | |

## Tabla de registros (loop)

Estructura: `<thead>` con las cabeceras + una fila `<tr data-loop="o2m">` con las celdas
que se repiten por línea. Los campos dentro son de la **línea**.

```html
<table class="table table-bordered o_table" style="width:100%;border-collapse:collapse">
  <thead><tr><th>#</th><th>Producto</th><th style="text-align:right">Subtotal</th></tr></thead>
  <tbody>
    <tr data-loop="order_line" data-empty="Sin líneas">
      <td><span class="o-ird-field" contenteditable="false" data-linenum="1">#</span></td>
      <td><span class="o-ird-field" contenteditable="false" data-field="product_id.name">Producto</span></td>
      <td style="text-align:right"><span class="o-ird-field" contenteditable="false" data-field="price_subtotal" data-widget="monetary">0</span></td>
    </tr>
  </tbody>
</table>
```

Modificadores del `<tr data-loop>`:
- `data-empty="Sin líneas"` → fila que aparece solo si el o2m está vacío.
- `data-group-by="campo"` → agrupa por ese campo (m2o de la línea) + cabecera de grupo + subtotales automáticos de columnas numéricas.
- `data-row-if='{"field":"display_type","op":"set"}'` → esa fila solo se pinta para líneas que cumplen (secciones/notas).

Condición de columna (ocultar toda la columna): `data-col-cond='{"loop":"order_line","field":"discount","op":">","val":"0","q":"any"}'` en el `<th>` **y** en el `<td>` de esa columna. `q`: `any`/`all`/`doc`.

## Cabecera / pie

Van en campos separados `header_html` / `footer_html` (no en el body). Ahí caben chips
`data-company`, `data-field`, `data-pagenum`, `data-ctx`. El nº de página solo tiene sentido en el pie.

## Restricciones (motor wkhtml 0.12.5)

- Maquetación en columnas → **tablas** (`<td style="width:50%">`), NUNCA flexbox/grid/`calc()`/`vw`/`vh`.
- Estilos inline sí (border, padding, text-align, width en %/mm/px).
- Ver `wkhtml-limits.md` para el detalle.

## Bucle de generación por IA (RPC / MCP de Indaws / odoo-pilot)

La IA vive **fuera** de Odoo. El addon no llama a ningún LLM.

1. **Contexto**: `get_model_fields(model)` (+ `get_model_fields(comodel)` por cada o2m relevante) → qué campos/loops existen. Loops del modelo: los `is_loop=true`.
2. **Redactar** el `body_html` (y `header_html`/`footer_html`) con el vocabulario de arriba.
3. **Validar en bucle**: crear el designer (`orm_create` con `name` + `model_id`) y llamar
   `validate_body(html)`. Devuelve `{ok, errors:[{kind,detail,value}], warnings, counts}`.
   Corregir hasta `ok=true`. **No entregar con errores**: el emisor se traga los campos malos
   (salen vacíos) y nadie se entera.
4. **Aplicar + generar**: `save_html(body, header, footer)` → `generate(...)` (crea el `ir.actions.report`).
5. **Entregar al humano**: "abre el editor (`action_open_editor`) y retoca lo visual".

`validate_body` es de solo lectura (no escribe ni emite) → seguro para llamar en bucle.
El acceso por MCP/pilot corre como el usuario autenticado **respetando record rules**: la IA
no puede crear helpers ni tocar lo que el usuario no podría.

Made with report-designer-developer · v0.1.0
