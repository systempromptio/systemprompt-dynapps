# Receta: añadir un elemento insertable nuevo

El patrón para meter un nuevo elemento (un chip, una tabla, un gráfico, un bloque)
en el editor. **Los 4 pasos son obligatorios**; omitir uno deja el chip visible como
atributo muerto en el PDF, o sin efecto, o rompe el emisor.

## 1. Frontend (OWL) — `static/src/report_designer/`

En `report_designer.js`:
- Añadir un método `insert<Cosa>()` que construye el `<span data-... >` (o `<tr>`, `<div>`)
  y lo inserta con `editor.shared.dom.insert(node)` sobre el **editor activo** (`_ae()`).
- Exponerlo como:
  - item del **menú contextual** (`IrdContextMenu`, en el nivel adecuado), y/o
  - item de `＋ Insertar ▾` en la barra, y/o
  - comando "/" (powerbox) vía `IrdCommandsPlugin` (user_commands + powerbox_items).
- Si el elemento es seleccionable/editable, añadir una **rama en `selectChip`** (o el
  handler de clic) que abra la pestaña/panel correcto del inspector.
- Panel de modo en `report_designer.xml` (+ estilos en `report_designer.scss`).

Recordar: el listener de clic-chip vive en `t-on-click` de `.ird-sheet` (burbujeo), no en `onLoad`.

## 2. Emisor — `models/report_emitter.py`

- Añadir un **pase** en `_transform_qweb` en la posición correcta del orden
  (ver `architecture.md`: campos→expr→helper→linenum→barcode→i18n→show/style-if→loops→col-cond→strip).
  El pase busca los nodos por su `data-*`, valida (p.ej. `_resolve_field`, `_expr_is_safe`)
  y reemplaza por la sintaxis QWeb correspondiente.
- Si DOCX se retoma: pase equivalente en `_transform_docx` (`{{ }}` / `{%tr%}` / `{% if %}`).
- **Añadir el nuevo `data-*` a las DOS listas `strip_attributes`** (una en el flujo QWeb,
  otra en el DOCX). Si no, el atributo sobrevive al PDF o rompe el pase de limpieza.

## 3. Test — `tests/test_report_designer.py`

Patrón de los 96 tests existentes:
```python
designer = self.env['indaws.report.designer'].create({...})
designer.body_html = '<p><span data-mi-cosa="...">X</span></p>'
qweb = designer._transform_qweb(designer.body_html)   # o el método público equivalente
# asserts:
#  - el chip se tradujo al t-* esperado
#  - el XML resultante está bien formado (lxml.etree.fromstring)
#  - ningún data-* sobrevive (strip aplicado)
#  - caso límite: campo/loop inválido → se emite vacío, NO rompe
```
Cubrir el caso feliz **y** el inválido (bind inexistente, loop sin o2m, expr insegura).

## 4. Ciclo EUROLOTES

```bash
cd $PROJECTS_ROOT/EUROLOTES18 && docker compose down && docker compose up -d
docker compose exec -T odoo bash -lc '$ODOO_SRC/odoo18/odoo-bin -c /etc/odoo/odoo.conf -d EUROLOTES -u indaws_report_designer --stop-after-init --http-port=8072 --test-enable --test-tags /indaws_report_designer' 2>&1 | tail -30
```
Y luego **eyeball en navegador** (el frontend OWL no se testea por CLI): insertar el
elemento, ver que el chip aparece, previsualizar (paged.js) e imprimir prueba (PDF real).

## Checklist rápido

- [ ] `insert<Cosa>()` + entrada de menú/powerbox en el frontend
- [ ] rama en `selectChip` / handler de clic (si es editable)
- [ ] panel de modo en `.xml` + `.scss`
- [ ] pase en `_transform_qweb` en la posición correcta
- [ ] (`_transform_docx` si DOCX activo)
- [ ] `data-*` nuevo en las DOS listas `strip_attributes`
- [ ] test caso feliz + caso inválido
- [ ] ciclo EUROLOTES + eyeball navegador

Made with report-designer-developer · v0.1.0
