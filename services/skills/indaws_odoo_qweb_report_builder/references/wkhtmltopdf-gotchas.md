# wkhtmltopdf Gotchas — Odoo QWeb Reports

wkhtmltopdf es el motor de renderizado PDF de Odoo. Versión mínima: **0.12.5**.
Versión recomendada: **0.12.6 (with patched qt)**.

Verificar: `wkhtmltopdf --version`

---

## §1. CSS soportado vs no soportado

### Soportado (usar libremente)
- Propiedades básicas: `margin`, `padding`, `border`, `color`, `background-color`
- `font-family`, `font-size`, `font-weight`, `font-style`
- `text-align`, `vertical-align`, `line-height`
- `width`, `height`, `max-width`, `max-height`
- `display: block`, `inline`, `inline-block`, `table`, `table-row`, `table-cell`
- `page-break-before`, `page-break-after`, `page-break-inside`
- `float` (con limitaciones)
- Tablas HTML (`<table>`, `<thead>`, `<tbody>`, `<tfoot>`)

### NO soportado / comportamiento inconsistente
- **Flexbox** (`display: flex`) — NO soportado. Usar tablas o floats.
- **CSS Grid** — NO soportado.
- **`position: fixed`** — Ignorado en el cuerpo. Solo funciona en header/footer templates.
- **`position: sticky`** — NO soportado.
- **`calc()`** — Soporte parcial, evitar.
- **`@media print`** — Soportado, pero `@media screen` también se aplica (wkhtmltopdf no distingue bien).
- **`overflow: hidden`** con contenido que desborda — puede cortar contenido inesperadamente.
- **`transform`** — Soporte muy limitado.
- **Fuentes web (`@font-face` con URL)** — No resuelve URLs externas. Ver §5.

### Regla práctica
Si algo funciona en un `<table>` HTML clásico, funcionará en wkhtmltopdf.
Si depende de CSS moderno (flex, grid, sticky), no usar.

### Anchos de tabla que se descuadran (verificado)
`text-nowrap` / `text-truncate` en celdas y `table-layout: auto` hacen que wkhtmltopdf
(y el editor `@html_editor`) **descuadren los anchos**: las columnas se estiran para no
partir texto y se salen de la hoja. Fix:
- Quitar `text-nowrap`/`text-truncate` de las celdas de datos.
- Fijar el reparto con `table-layout: fixed` **+ un `width` explícito por columna**
  (en `<col>` o en las celdas de cabecera), no solo en la tabla.
- `max-width: 100%` en la tabla para que nunca desborde la hoja.

⚠ Cuidado con el otro extremo: `table-layout: fixed` **sin** width por columna puede
partir el texto letra a letra. Necesitas fixed **y** los anchos declarados.

---

## §2. Saltos de página

### Forzar salto
```css
.page-break { page-break-before: always; }
```
```xml
<div style="page-break-before: always;"/>
```

### Evitar salto dentro de un elemento
```css
.keep-together { page-break-inside: avoid; }
```
**ADVERTENCIA:** `page-break-inside: avoid` funciona en wkhtmltopdf 0.12.6
para elementos de bloque simples. Para tablas y elementos con mucho contenido,
puede ignorarse. No confiar en él para elementos que ocupen más de media página.

### Repetición de cabecera de tabla en cada página
```xml
<table>
    <thead>
        <!-- wkhtmltopdf repite <thead> en cada página automáticamente -->
        <tr><th>Col A</th><th>Col B</th></tr>
    </thead>
    <tbody>
        <tr t-foreach="o.order_line" t-as="line">...</tr>
    </tbody>
</table>
```

### `--disable-smart-shrinking`
wkhtmltopdf tiene un algoritmo de "smart shrinking" que puede reducir el
contenido para que quepa en la página, distorsionando el layout.

En Odoo, se activa por paperformat:
```xml
<field name="disable_shrinking" eval="True"/>
```
Activar cuando: tablas anchas se comprimen inesperadamente, fuentes cambian de
tamaño entre páginas, o el layout se rompe.

---

## §3. Cabeceras y pies de página que desaparecen en página 2+

**Síntoma:** El header y footer aparecen en la página 1 pero desaparecen a partir
de la página 2.

**Causa:** wkhtmltopdf genera headers/footers como HTML independiente que se
imprime en cada página, pero los estilos CSS del documento principal no se
aplican a ellos porque se cargan en un contexto HTML separado.

**Soluciones:**

1. **Usar `web.external_layout`** (solución Odoo recomendada). Su cabecera/pie
   forman parte del cuerpo HTML, no del header wkhtmltopdf, por lo que no sufren
   este problema. Para sustituir el bloque de dirección, define `t-set="address"`
   (renderizado por `web.address_layout`) — no existen slots
   `address_header`/`address_footer`.

2. **Inline todos los estilos del header/footer** — Si se usan los argumentos
   `--header-html` / `--footer-html` directamente, los estilos deben ser
   `style=""` inline, no clases externas.

3. **Usar JavaScript para número de página** en footer:
   ```html
   <script>
   function subst() {
       var vars = {};
       var query = window.location.search.substring(1).split('&');
       for (var i in query) {
           var pair = query[i].split('=');
           vars[unescape(pair[0])] = unescape(pair[1]);
       }
       var el = document.getElementById('page');
       el.textContent = vars.page + '/' + vars.topage;
   }
   </script>
   <div id="page" onload="subst()"></div>
   ```
   wkhtmltopdf pasa `page`, `topage`, `date`, etc. como query params al HTML
   del footer.

---

## §4. Imágenes que no cargan

**Síntoma:** El PDF muestra un recuadro vacío o roto donde debería haber una imagen.

**Causa:** wkhtmltopdf ejecuta en el servidor sin acceso al contexto web de Odoo.
Las URLs relativas a `/web/image/...` (`/web/image/res.company/1/logo`) o las URLs
que requieren sesión autenticada no resuelven.

**Excepción — barcode:** la ruta `/report/barcode/...` SÍ funciona en reports
QWeb; la sirve el controlador de Odoo (documentado en
`web/controllers/report.py`). La regla de base64 de abajo aplica a `/web/image/...`,
no al controlador de barcode.

**Solución para imágenes de campo: siempre base64 inline**

```python
# En el AbstractModel o en el contexto del report:
import base64
logo_b64 = base64.b64encode(company.logo_web).decode('utf-8')
```

```xml
<img t-att-src="'data:image/png;base64,' + company.logo.decode('utf-8')"
     style="max-height:80px;"/>
```

**Para campos Binary de Odoo** (ya almacenados en base64):
```xml
<img t-att-src="'data:image/png;base64,' + o.product_id.image_128.decode('utf-8')"/>
```

**NUNCA:**
```xml
<!-- ROTO en wkhtmltopdf -->
<img t-att-src="'/web/image/res.company/' + str(company.id) + '/logo'"/>
```

**Excepción:** Si Odoo está configurado con una URL base accesible desde el
servidor (mismo host), se puede usar `report_base_url` del sistema. Pero base64
es siempre más fiable.

---

## §5. Fuentes web

**Síntoma:** La fuente personalizada no aparece en el PDF, se usa la fuente por defecto.

**Causa:** wkhtmltopdf no resuelve `@font-face` con URLs externas (Google Fonts,
CDNs). Necesita las fuentes instaladas en el sistema o como base64.

**Soluciones:**

1. **Instalar la fuente en el servidor** (Linux):
   ```bash
   cp MiFuente.ttf /usr/share/fonts/truetype/
   fc-cache -fv
   ```
   Luego usar en CSS: `font-family: 'MiFuente', sans-serif;`

2. **Embeber como base64 en CSS inline** (para fuentes pequeñas):
   ```css
   @font-face {
       font-family: 'MiFuente';
       src: url('data:font/truetype;base64,<base64_del_ttf>');
   }
   ```

3. **Usar fuentes del sistema** — `Arial`, `Times New Roman`, `Courier New`,
   `DejaVu Sans` (preinstalada en la mayoría de distribuciones Linux).

---

## §6. Problemas conocidos por versión

| Versión wkhtmltopdf | Problema conocido |
|---------------------|-------------------|
| < 0.12.5 | No soportado por Odoo 17+ |
| 0.12.5 (sin patched qt) | Headers/footers con JS no funcionan |
| 0.12.6 (with patched qt) | Versión recomendada — soporte completo de JS en headers |
| 0.12.6 (sin patched qt) | Sin soporte JS en headers, `page-break-inside` inestable |

Verificar versión completa: `wkhtmltopdf --version`
Debe decir: `wkhtmltopdf 0.12.6 (with patched qt)`

### Motor PDF por versión — los límites de arriba son de wkhtml
**Verificado (v18 y v19): el ÚNICO motor de PDF QWeb es wkhtmltopdf.** No existe
`base_report_paper_muncher` ni `report_type='qweb-pdf-paper-muncher'` en el código de
v18/v19 (grep vacío) → todos los límites de esta página aplican tal cual.

⚠ **Paper Muncher (v20) — reportado, NO verificado contra fuente** (no hay checkout v20
local): Odoo introduce en v20 un motor de PDF propio, enchufable, con CSS moderno
(flex/grid/etc.). Los nombres reportados (`report_type='qweb-pdf-paper-muncher'`,
métodos `_run_pdf_engine`/`_get_pdf_engine`/`get_pdf_engine_state`) **no están verificados**.
Implicación práctica: **no relajes las restricciones CSS de esta página** salvo que
confirmes que el informe se renderiza con Paper Muncher en v20; en v18/v19 sigue siendo
wkhtml y el CSS moderno rompe.

---

## §7. Debugging de renders

```bash
# Generar el HTML que se pasa a wkhtmltopdf (útil para diagnosticar)
# En el shell de Odoo:
env['ir.actions.report']._render_qweb_html('sale_custom.report_sale_order', [record_id])

# Ver el comando wkhtmltopdf que ejecuta Odoo (activar logging)
import logging
logging.getLogger('odoo.addons.base.models.ir_actions_report').setLevel('DEBUG')
```

En desarrollo, activar el modo debug de Odoo y usar "Descargar" en lugar de
"Imprimir" para ver el HTML intermedio.

---

## §8. Checklist antes de enviar a producción

- [ ] wkhtmltopdf >= 0.12.6 with patched qt instalado en el servidor
- [ ] Todas las imágenes en base64 (ninguna URL relativa)
- [ ] Fuentes disponibles en el servidor o embebidas
- [ ] Probado con múltiples registros (multi-página) no solo uno
- [ ] Header y footer visibles en página 2 y última
- [ ] `page-break-inside: avoid` probado — tiene comportamiento esperado
- [ ] CSS sin flexbox ni grid
- [ ] `disable_shrinking` configurado según necesidad
