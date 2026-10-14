# Vistas Pivot y Graph — Referencia de Atributos

Fuentes verificadas:
- v19: `$ODOO_SRC/odoo19/addons/web/static/src/views/pivot/pivot_arch_parser.js`
- v19: `$ODOO_SRC/odoo19/addons/web/static/src/views/graph/graph_arch_parser.js`
- v18: mismos paths en `odoo18`
- Atributos genéricos (`sample`, `js_class`): `$ODOO_SRC/odoo19/addons/web/static/src/views/view.js` líneas 344–401
- Intervalos: `$ODOO_SRC/odoo19/addons/web/static/src/search/utils/dates.js`

---

## Vista PIVOT

### Elemento `<pivot>`

| Atributo | Tipo | Qué hace | Ejemplo |
|---|---|---|---|
| `string` | string | Título de la vista, sustituye al nombre por defecto en breadcrumbs | `string="Análisis de ventas"` |
| `disable_linking` | bool | Desactiva la navegación a lista al pinchar en una celda | `disable_linking="1"` |
| `display_quantity` | bool | Muestra la columna "Recuento" junto a las medidas | `display_quantity="True"` |
| `default_order` | string | Ordena las filas por medida. Formato: `nombre_campo [desc]` | `default_order="amount_total desc"` |
| `sample` | bool | Rellena la vista con datos de muestra cuando no hay registros reales (procesado en `web/static/src/views/view.js:356`, `sample`) | `sample="1"` |
| `js_class` | string | Sustituye el componente JS de la vista por una subclase registrada (procesado en `web/static/src/views/view.js:344`, `View.loadView`) | `js_class="my_pivot"` |

**Atributos NO parseados en el arch_parser de pivot** (v18 y v19): `sample` y `js_class` son genéricos y se procesan en `view.js` antes de pasar al parser específico. No aparecen en `PivotArchParser`.

### Elemento `<field>` dentro de `<pivot>`

| Atributo | Qué hace | Notas |
|---|---|---|
| `name` | Nombre técnico del campo (obligatorio) | Validado por RNG |
| `type` | Rol del campo: `row`, `col` o `measure` | Sin `type` el campo solo almacena `fieldAttrs` (string, widget…) pero no se activa |
| `interval` | Granularidad temporal para campos date/datetime. Valores: `day`, `week`, `month` (defecto), `quarter`, `year` | Se concatena al nombre: `date_order:month` |
| `string` | Etiqueta personalizada para cabecera de fila/col/medida | `string="Mes pedido"` |
| `invisible` | Oculta el campo del selector de medidas/agrupaciones. Valores `"True"` o `"1"` | Si es invisible, el resto de atributos se ignoran |
| `widget` | Widget de formato para la medida (ej. `monetary`, `percentage`) | Se indexa con el fieldName incluyendo interval |
| `operator` | Operador de agregación alternativo (`+`, `-`, `*`…). Si está presente el campo se trata como medida aunque no tenga `type="measure"` | Poco común; preferir `type="measure"` |
| `options` | Diccionario Python evaluado con `evaluateExpr`. Pasa opciones al widget del campo | **Solo v19** — ver diff abajo |

#### Medidas por defecto

Si ningún `<field>` tiene `type="measure"`, la vista muestra únicamente `__count` (recuento de registros). Cada `<field type="measure">` o `<field operator="…">` se añade a `activeMeasures` y se activa al cargar la vista.

#### Ejemplo completo pivot

```xml
<pivot string="Ventas por mes" disable_linking="1" display_quantity="True" default_order="amount_total desc" sample="1">
    <field name="partner_id" type="row"/>
    <field name="date_order" type="col" interval="month"/>
    <field name="amount_total" type="measure" string="Total" widget="monetary"/>
    <field name="product_id" invisible="1"/>
</pivot>
```

### Diffs v18 → v19 (PIVOT)

| Cambio | v18 | v19 |
|---|---|---|
| **`options` en `<field>`** | No existe. El bucle sobre atributos en v18 solo excluye `name`, `type`, `operator`, `interval`, `string`, `widget` y pasa el resto como string crudo | v19 añade detección explícita de `options` y lo evalúa con `evaluateExpr` (importado de `@web/core/py_js/py`). Permite pasar un dict Python: `options="{'digits': [6,2]}"` |
| **Import `evaluateExpr`** | No importado | `import { evaluateExpr } from "@web/core/py_js/py"` añadido en línea 3 |
| **Bucle sobre atributos extra** | v18 no tiene el bucle `for (const { name, value } of node.attributes)` | v19 itera todos los atributos no reconocidos y los almacena en `fieldAttrs` (excepto `options` que se evalúa) |

---

## Vista GRAPH

### Elemento `<graph>`

| Atributo | Tipo | Qué hace | Ejemplo |
|---|---|---|---|
| `type` | string | Tipo de gráfico: `bar` (defecto), `line`, `pie` | `type="bar"` |
| `stacked` | bool | Apila las series en gráficos de barra. Solo aplica a `type="bar"` | `stacked="1"` |
| `cumulated` | bool | Acumula los valores a lo largo del eje X (suma progresiva). Solo aplica a `type="line"` o `type="bar"` | `cumulated="True"` |
| `cumulated_start` | bool | Incluye el punto inicial (valor 0) en la acumulación. Requiere `cumulated="1"` | `cumulated_start="1"` |
| `order` | string | Ordena las barras/slices. Valores: `ASC`, `DESC` (case-insensitive; se normaliza a mayúsculas) | `order="DESC"` |
| `string` | string | Título de la vista | `string="Ventas mensuales"` |
| `disable_linking` | bool | Desactiva la navegación a lista al pinchar en una barra/slice | `disable_linking="1"` |
| `sample` | bool | Datos de muestra cuando no hay registros (genérico, procesado en `view.js`) | `sample="1"` |
| `js_class` | string | Sustituye el componente JS por una subclase registrada (genérico, procesado en `view.js`) | `js_class="my_graph"` |

**Nota `order`:** Los valores válidos son `"ASC"`, `"DESC"`, `"asc"`, `"desc"` o `null`. Cualquier otro valor se ignora silenciosamente (el parser verifica contra `ORDERS`).

### Elemento `<field>` dentro de `<graph>`

| Atributo | Qué hace | Notas |
|---|---|---|
| `name` | Nombre técnico del campo (obligatorio) | `id` se ignora explícitamente (línea 46 v19) |
| `type` | `measure` para usar el campo como medida agregada. Sin `type` (o cualquier otro valor) se trata como agrupación (groupBy) si el tipo de campo es agrupable | El último `type="measure"` definido en el arch se usa como medida activa inicial |
| `interval` | Granularidad para campos date/datetime. Valores: `day`, `week`, `month`, `quarter`, `year` | Solo aplica a campos sin `type="measure"` |
| `string` | Etiqueta personalizada para la leyenda | |
| `invisible` | Oculta el campo. Valores `"True"` o `"1"` | Si es invisible, `interval` y `type` no se procesan |
| `widget` | Widget de formato para la medida | Disponible v18 y v19; almacenado en `fieldAttrs` |

#### Agrupaciones y medidas por defecto

- **groupBy** (eje X / colores): todos los campos sin `type="measure"` cuyo tipo de campo esté en `GROUPABLE_TYPES` (many2one, selection, char, date, datetime, boolean…).
- **measure** (eje Y): el **último** `<field type="measure">` definido. Si ninguno, se usa `__count`.
- La vista graph solo admite **una medida activa** a la vez (a diferencia de pivot).

#### Ejemplo completo graph

```xml
<graph string="Pedidos por mes" type="bar" stacked="1" order="DESC" disable_linking="1" sample="1">
    <field name="date_order" interval="month"/>
    <field name="partner_id"/>
    <field name="amount_total" type="measure" string="Total"/>
</graph>
```

#### Ejemplo line acumulado

```xml
<graph type="line" cumulated="1" cumulated_start="1">
    <field name="date" interval="month"/>
    <field name="revenue" type="measure"/>
</graph>
```

### Diffs v18 → v19 (GRAPH)

Los parsers de graph en v18 y v19 son **idénticos** línea por línea. No hay ningún diff confirmado entre ambas versiones en `graph_arch_parser.js`. Los atributos `cumulated_start`, `stacked`, `cumulated`, `order`, `disable_linking`, `type`, `string` y el manejo de `<field>` son los mismos en v18 y v19.

> Para graph, los cambios entre v18 y v19 (si los hay) se producen fuera del arch_parser: en `graph_model.js`, `graph_renderer.js` o librerías de Chart.js.

---

## Resumen comparativo

| Capacidad | Pivot | Graph |
|---|---|---|
| Múltiples medidas simultáneas | Sí (todas las `type="measure"` activas) | No (solo la última definida) |
| Agrupación en columnas | Sí (`type="col"`) | No |
| Agrupación en filas | Sí (`type="row"`) | Sí (eje X) |
| `interval` en fechas | Sí | Sí |
| `stacked` | No | Sí (solo `bar`) |
| `cumulated` | No | Sí (solo `bar`/`line`) |
| `order` | `default_order` (por medida) | `order` ASC/DESC (barras) |
| `display_quantity` | Sí | No |
| `sample` | Sí (genérico) | Sí (genérico) |
| `js_class` | Sí (genérico) | Sí (genérico) |
