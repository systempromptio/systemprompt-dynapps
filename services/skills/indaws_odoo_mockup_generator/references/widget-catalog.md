# Catálogo de widgets Odoo (dato de fuente) + cómo se DIBUJAN

La promesa "anclado a la fuente" se apoya aquí. **Regla:** un mockup solo puede usar un widget de esta
lista; y debe **dibujarlo como Odoo lo dibuja** (no un valor genérico). Lo que no esté = **OWL custom (DEV)**.

Refresca la lista para la versión objetivo con `scripts/list_widgets.sh <odoo_root> [--all]` (extrae los
nombres reales del registro `category('fields').add(...)`). Clases verificadas en
`web/static/src/views/fields` de Odoo 18.

## Field widgets (web core v18 — 64)
`ace attachment_image badge badge_selection binary boolean boolean_favorite boolean_icon boolean_toggle
char code color color_picker dashboard_graph domain email embed_viewer filterable_selection float
float_factor float_time float_toggle gauge handle html iframe_wrapper image image_url integer jsonb
kanban_color_picker label_selection many2many many2many_binary many2many_checkboxes many2many_tags
many2many_tags_avatar many2many_tags_avatar_popover many2one many2one_avatar many2one_barcode
many2one_reference many2one_reference_integer monetary one2many pdf_viewer percentage percentpie phone
priority progressbar properties property_tags radio reference remaining_days selection selection_badge
signature state_selection statinfo statusbar text timezone_mismatch url`

Otros viven en sus módulos (refresca con `--all`): `account` (p.ej. account_type_selection), `stock`,
`mail`, y **Enterprise**: `web_gantt` (vista `<gantt>`), cohort, map, dashboard, sign, etc.

## Snippets visuales (dibuja el widget como Odoo)

Incluye estos estilos y usa el markup real; así el cliente reconoce el widget al instante.
```css
/* priority → estrellas (o_priority_star + fa-star) */
.o_priority{display:inline-flex;gap:2px}.o_priority_star{color:#f0ad4e;cursor:pointer}
/* state_selection → punto de color + label (o_status) */
.o_status{display:inline-block;width:12px;height:12px;border-radius:50%;background:#adb5bd;vertical-align:middle}
.o_status.text-success{background:#28a745}.o_status.text-danger{background:#dc3545}.o_status.text-warning{background:#f0ad4e}
.o_status_label{margin-left:5px;font-size:12.5px}
/* many2many_tags → pills de color (badge + o_tag) */
.o_tag{display:inline-flex;align-items:center;gap:4px;border-radius:11px;padding:2px 9px;font-size:11.5px;font-weight:600;margin:2px 3px 2px 0}
.o_tag .dot{width:8px;height:8px;border-radius:50%}
/* boolean_toggle → switch Bootstrap (form-switch) */
.o_switch{width:34px;height:18px;background:#28a745;border-radius:10px;position:relative;display:inline-block}
.o_switch.off{background:#adb5bd}.o_switch:after{content:"";position:absolute;width:14px;height:14px;background:#fff;border-radius:50%;top:2px;right:2px}.o_switch.off:after{right:auto;left:2px}
/* handle → arrastre (o_row_handle) */
.o_row_handle{color:#adb5bd;cursor:grab}
/* progressbar / percentpie */
.o_progress{background:#e9ecef;border-radius:4px;height:16px;overflow:hidden;min-width:90px;display:inline-block;vertical-align:middle}
.o_progress .bar{background:#714B67;height:100%;color:#fff;font-size:10px;text-align:center;line-height:16px}
/* statinfo (smart button) → cifra grande + label */
.o_statinfo .value{font-weight:700;font-size:15px;color:#714B67}.o_statinfo .text{font-size:11.5px;color:#6b7280}
```
```html
<!-- priority --> <span class="o_priority"><i class="fa fa-star o_priority_star"></i><i class="fa fa-star o_priority_star"></i><i class="fa fa-star-o o_priority_star" style="color:#ccc"></i></span>
<!-- state_selection --> <span class="o_status text-success"></span><span class="o_status_label">Listo</span>
<!-- many2many_tags --> <span class="o_tag" style="background:#e7f1ff;color:#0d6efd"><span class="dot" style="background:#0d6efd"></span>Mecánica</span>
<!-- boolean_toggle --> <span class="o_switch"></span>  (off: <span class="o_switch off"></span>)
<!-- monetary --> <span style="font-variant-numeric:tabular-nums">1.150,00 €</span>  (siempre alineado a la derecha en listas)
<!-- many2one_avatar --> <span class="avatar" style="width:22px;height:22px;font-size:10px">JS</span> J. Soler
<!-- progressbar --> <span class="o_progress"><span class="bar" style="width:65%">65%</span></span>
<!-- handle --> <span class="o_row_handle">⣿</span>  (columna de arrastre en listas ordenables)
<!-- statinfo (dentro de oe_stat_button) --> <div class="o_statinfo"><span class="value">3</span><span class="text">Facturas</span></div>
```
Requiere FontAwesome para `fa-star` etc.; en un mockup autocontenido sustituye por `★`/`☆` si no hay FA
(sin CDNs). El objetivo es la forma reconocible, no el icono exacto.

## Mapa campo→widget típico (para elegir bien)
- selection de estado con etapas → `statusbar` (cabecera) · selection con color puntual → `state_selection` o `badge`.
- prioridad/valoración 0-3 → `priority` (estrellas). Booleano de acción → `boolean_toggle`.
- etiquetas multi → `many2many_tags` (pills). Importe → `monetary`. Duración → `float_time`. % → `percentage`/`progressbar`.
- responsable/usuario → `many2one_avatar`. Adjunto imagen → `image`. PDF → `pdf_viewer`. Firma → `signature`.
- líneas hijas → `one2many` (list editable). Contador en smart button → `statinfo`.

Si el campo pide algo que **ningún widget** de la lista cubre (timer en vivo, drag&drop táctil, tabla
anidada con subtotales, lienzo libre) → se dibuja igual pero se marca **OWL custom (DEV)** en la anotación.
