# Referencia: Vista Search (incluido SearchPanel) — Odoo CE v18/v19

Fuentes verificadas:
- `$ODOO_SRC/odoo19/addons/web/static/src/search/search_arch_parser.js`
- `$ODOO_SRC/odoo18/addons/web/static/src/search/search_arch_parser.js`
- `$ODOO_SRC/odoo19/odoo/addons/base/rng/search_view.rng` + `common.rng`
- `$ODOO_SRC/odoo18/odoo/addons/base/rng/search_view.rng` + `common.rng`
- `$ODOO_SRC/odoo19/odoo/addons/base/models/ir_ui_view.py` (validación servidor)

---

## Elemento raíz `<search>`

Contenido permitido: `<field>`, `<filter>`, `<group>`, `<separator>`, `<newline>`, `<searchpanel>` (máximo uno).

Atributos del elemento raíz:

| Atributo | Descripción |
|---|---|
| `string` | Etiqueta opcional de la vista. |
| `position` | Para herencia de vistas: `before`, `after`, `inside`, `replace`, `attributes`. |

---

## `<field>` dentro de `<search>`

Parsed en `visitField()`. Añade el campo a la barra de búsqueda como input de texto (o selector según tipo).

| Atributo | Descripción | Ejemplo |
|---|---|---|
| `name` | **Obligatorio.** Nombre del campo en el modelo. | `name="partner_id"` |
| `string` | Etiqueta visible. Si se omite usa el `string` del campo. | `string="Cliente"` |
| `operator` | Operador de comparación por defecto. Si se omite: `ilike` para char/text/html/o2m/m2m, `=` para el resto. | `operator="="` |
| `filter_domain` | Dominio completo personalizado. Usa `self` como placeholder del valor escrito. Tiene prioridad sobre `operator`. | `filter_domain="[('name','ilike',self),('ref','ilike',self)]"` |
| `domain` | Dominio estático adicional que se añade al filtrar por este campo. | `domain="[('active','=',True)]"` |
| `context` | Contexto añadido al buscar. Permite `group_by` dentro de `<field>`. | `context="{'group_by':'categ_id'}"` |
| `widget` | Afecta la resolución del operador por defecto en `search_default_*`. Con `widget="many2one"` usa `=`. | `widget="many2one"` |
| `invisible` | Expresión Python/bool. Oculta el campo de la barra. | `invisible="1"` |
| `groups` | Grupos de acceso (gestionado por el servidor). | `groups="base.group_user"` |
| `placeholder` | Texto placeholder del input (RNG, no consumido por el parser JS). | `placeholder="Buscar por nombre..."` |

### `search_default_<name>`

Para activar un campo de búsqueda por defecto desde el contexto de la action:

```xml
<!-- En la action -->
<field name="context">{'search_default_partner_id': 5}</field>
```

El valor es el ID (para m2o), la cadena (para char), o array (para m2m/o2m en v19).

---

## `<filter>`

Parsed en `visitFilter()`. Existen tres tipos según los atributos presentes.

### Atributos comunes a todos los tipos

| Atributo | Descripción | Ejemplo |
|---|---|---|
| `name` | **Obligatorio** (RNG). Identificador usado por `search_default_<name>`. | `name="my_tasks"` |
| `string` | Etiqueta visible. Si se omite usa `name` o `help`. | `string="Mis tareas"` |
| `invisible` | Expresión Python/bool. Oculta el filtro del menú. | `invisible="context.get('hide_draft')"` |
| `help` | Descripción alternativa si no hay `string`. | `help="Filtrar por usuario"` |
| `groups` | Grupos de acceso (servidor). | `groups="base.group_manager"` |

### Tipo 1 — Filtro de dominio (tiene `domain`)

| Atributo | Descripción | Ejemplo |
|---|---|---|
| `domain` | Dominio que se activa al marcar el filtro. Default `[]`. | `domain="[('state','=','done')]"` |
| `context` | Contexto extra cuando el filtro está activo (sin `group_by`). | `context="{'active_test': False}"` |

```xml
<filter name="active" string="Activos" domain="[('active','=',True)]"/>
```

Activar por defecto desde la action:
```xml
<field name="context">{'search_default_active': 1}</field>
```

### Tipo 2 — Agrupar por (tiene `context` con clave `group_by`)

| Atributo | Descripción | Ejemplo |
|---|---|---|
| `context` | Debe contener `{'group_by': 'campo'}` o `{'group_by': 'campo:intervalo'}`. Para campos date/datetime los intervalos válidos son: `year`, `quarter`, `month`, `week`, `day`. | `context="{'group_by':'partner_id'}"` |

```xml
<filter name="by_customer" string="Por cliente" context="{'group_by': 'partner_id'}"/>
<filter name="by_month" string="Por mes" context="{'group_by': 'date:month'}"/>
```

Activar por defecto:
```xml
<field name="context">{'search_default_by_customer': 1}</field>
```

### Tipo 3 — Filtro de fecha (tiene atributo `date`)

| Atributo | Descripción | Ejemplo |
|---|---|---|
| `date` | **Obligatorio para este tipo.** Nombre del campo date/datetime. | `date="date_order"` |
| `default_period` | Períodos preseleccionados separados por coma. Valores: `year`, `year-1`, `year-2`, `month`, `month-1`, `month-2`, `first_quarter`, `second_quarter`, `third_quarter`, `fourth_quarter`, `custom_<nombre>`. | `default_period="month"` |
| `start_month` | Offset del mes más antiguo mostrado. Default: `-2`. | `start_month="-6"` |
| `end_month` | Offset del mes más reciente mostrado. Default: `0`. | `end_month="0"` |
| `start_year` | Offset del año más antiguo mostrado. Default: `-2`. | `start_year="-3"` |
| `end_year` | Offset del año más reciente mostrado. Default: `0`. | `end_year="0"` |

Opciones de fecha personalizadas como hijos:
```xml
<filter name="date_filter" string="Fecha pedido" date="date_order" default_period="month">
    <filter name="this_week" string="Esta semana" domain="[('date_order','>=', (context_today()-relativedelta(weeks=1)).strftime('%Y-%m-%d'))]"/>
</filter>
```
Los hijos `<filter>` de un filtro de fecha requieren `name`, `string` y `domain` obligatoriamente.

---

## `<separator>`

Crea un separador visual / límite de grupo en el menú de filtros. No tiene atributos consumidos por el parser JS.

```xml
<separator/>
```

---

## `<group>`

Envuelve varios `<filter>` o `<field>` para mostrarlos en una columna del dropdown de Filtros/Agrupar por. No tiene atributos consumidos por el parser JS.

```xml
<group expand="0" string="Filtros">
    <filter name="my_tasks" string="Mis tareas" domain="[('user_id','=',uid)]"/>
    <filter name="active" string="Activos" domain="[('active','=',True)]"/>
</group>
```

> **v18 vs v19:** En v18 el RNG `common.rng` declaraba formalmente `expand` y `string` como atributos de `<group>`. En v19 ambos atributos se eliminaron del RNG (aunque el parser JS los ignora en ambas versiones; el efecto visual los soporta igualmente vía CSS).

---

## `<searchpanel>`

Solo puede haber uno por `<search>`. Renderiza un panel lateral con categorías y filtros.

### Atributos de `<searchpanel>`

| Atributo | v18 | v19 | Descripción | Ejemplo |
|---|---|---|---|---|
| `view_types` | sí | sí | Tipos de vista donde aparece el panel (separados por coma). Default: `"kanban,list"`. | `view_types="kanban,list,graph,pivot"` |
| `class` | sí | sí | Clase CSS aplicada al contenedor del panel. | `class="account_root w-auto"` |
| `fold` | **sí** | **eliminado** | Boolean. Si el panel comienza plegado. Procesado con `exprToBoolean()` en v18, eliminado del parser en v19. | `fold="1"` |

```xml
<searchpanel view_types="kanban,list">
    ...
</searchpanel>
```

---

## `<field>` dentro de `<searchpanel>`

Cada `<field>` define una sección del panel. Hay dos tipos según `select`.

### Atributos comunes

| Atributo | Descripción | Ejemplo |
|---|---|---|
| `name` | **Obligatorio.** Campo del modelo (normalmente m2o o selection). | `name="categ_id"` |
| `string` | Etiqueta de la sección. Si se omite usa el `string` del campo. | `string="Categorías"` |
| `select` | `"one"` (default) → categoría (radio, selección exclusiva). `"multi"` → filtro (checkboxes, selección múltiple). | `select="multi"` |
| `icon` | Clase Font Awesome para el icono de la sección. Default: `"fa-folder"` para `one`, `"fa-filter"` para `multi`. | `icon="fa-th-list"` |
| `color` | Color CSS para la cabecera de la sección. | `color="#875A7B"` |
| `enable_counters` | Expresión bool. Activa badges con el número de registros por valor. Default: `false`. Importante: si un filtro tiene `domain`, los contadores se deshabilitan automáticamente con warning. | `enable_counters="1"` |
| `expand` | Expresión bool. Si se muestran todos los valores expandidos por defecto. Default: `false`. | `expand="1"` |
| `limit` | Entero. Máximo de valores cargados. Default: `200`. Usar `"false"` para deshabilitar el límite. | `limit="20"` o `limit="false"` |
| `invisible` | Si es `"True"` o `"1"`, oculta toda la sección antes de procesarla. | `invisible="True"` |
| `groups` | Grupos de acceso (servidor). | `groups="base.group_multi_company"` |

### Atributos exclusivos de tipo `select="one"` (categoría)

| Atributo | v18 | v19 | Descripción | Ejemplo |
|---|---|---|---|---|
| `hierarchize` | sí | sí | Expresión bool. Muestra jerarquía padre-hijo. Default: `"1"` (true). Requiere `parent_id` en el modelo. | `hierarchize="0"` |
| `depth` | **no** | **sí** | Entero. Profundidad máxima de la jerarquía a mostrar. Default: `0` (ilimitado). Añadido en v19. | `depth="2"` |

### Atributos exclusivos de tipo `select="multi"` (filtro)

| Atributo | Descripción | Ejemplo |
|---|---|---|
| `domain` | Dominio que filtra los valores mostrados en la sección. **No compatible** con categorías que tengan `enable_counters` (se auto-deshabilita). Según el servidor, `domain` solo es válido en `select="multi"`. | `domain="[('parent_id','=',False)]"` |
| `groupby` | Campo por el que agrupar los valores del filtro. | `groupby="account_id"` |

### `searchpanel_default_<campo>`

Pre-selecciona un valor en el panel desde el contexto de la action:

```xml
<field name="context">{'searchpanel_default_state': ['submitted']}</field>
```

---

## Ejemplos reales de v19

**`$ODOO_SRC/odoo19/addons/product/views/product_views.xml`** — categoría + multi:
```xml
<searchpanel>
    <field name="categ_id" string="Categories" icon="fa-th-list"/>
    <field name="product_tag_ids" string="Tags" icon="fa-th-list" enable_counters="1" select="multi"/>
</searchpanel>
```

**`$ODOO_SRC/odoo19/addons/hr/views/hr_employee_views.xml`** — view_types extendido:
```xml
<searchpanel view_types="kanban,list,graph,pivot,hierarchy">
    <field name="company_id" groups="base.group_multi_company" icon="fa-building" enable_counters="1"/>
    <field name="department_id" icon="fa-users" enable_counters="1"/>
</searchpanel>
```

**`$ODOO_SRC/odoo19/addons/account/views/account_move_views.xml`** — `groupby` + `limit="false"`:
```xml
<searchpanel class="account_root w-auto">
    <field name="account_root_id" icon="fa-filter" groupby="account_id" limit="false"/>
</searchpanel>
```

**`$ODOO_SRC/odoo19/addons/hr_expense/views/hr_expense_views.xml`** — mezcla de atributos:
```xml
<searchpanel>
    <field name="state" expand="1" select="multi" icon="fa-check-square-o" enable_counters="1"/>
    <field name="employee_id" limit="20" hierarchize="0" select="one" icon="fa-users"/>
    <field name="company_id" expand="1" icon="fa-building" groups="base.group_multi_company"/>
</searchpanel>
```

---

## Diffs v18 → v19 confirmados

| Elemento | v18 | v19 |
|---|---|---|
| `<searchpanel fold="...">` | Soportado (`exprToBoolean()` en parser) | **Eliminado** del parser |
| `<searchpanel><field depth="...">` | No existía | **Añadido** (profundidad de jerarquía) |
| `<group expand="...">` | En `common.rng` | Eliminado del RNG |
| `<group string="...">` | En `common.rng` | Eliminado del RNG |
| `search_default_*` para m2m/o2m | Solo valor único | Soporta array completo con operador `"in"` |
| Función interna `getContextGroubBy` | Nombre con typo | Renombrado a `getContextGroupBy` (typo corregido) |
