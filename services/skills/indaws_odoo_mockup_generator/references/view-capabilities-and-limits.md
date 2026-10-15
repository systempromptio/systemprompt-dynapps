# Vistas Odoo: qué es nativo, qué Studio, qué desarrollo — y los límites

El mockup es honesto: cada bloque se etiqueta **ESTÁNDAR** (nativo, sale de crear el campo/vista) ·
**STUDIO** (configurable sin código) · **DESARROLLO** (requiere Python/QWeb/OWL). Aquí el mapa. Para
la **sintaxis exacta por versión** (tags, widgets, deltas) NO tires de memoria → `odoo-version-api-reference`.

## Mapa elemento del mockup ↔ constructo Odoo

| En el mockup | Constructo real | Clase |
|---|---|---|
| statusbar de etapas | campo `state` (selection) o `stage_id` (M2o) con `widget="statusbar"` | ESTÁNDAR |
| botones de workflow (control panel) | métodos de botón (`type="object"`) | DESARROLLO (si el método es custom) / ESTÁNDAR (si es de acción estándar) |
| smart buttons con contador | `oe_stat_button` + campo `count` computado sobre un related | ESTÁNDAR (campo + botón), el `compute` es DESARROLLO |
| grupos de campos 2 col | `<group>` con campos | ESTÁNDAR |
| campo obligatorio (*) | `required=True` (o `required="..."` condicional) | ESTÁNDAR |
| campo solo-lectura | `readonly=True` / computed | ESTÁNDAR |
| notebook con pestañas | `<notebook><page>` | ESTÁNDAR |
| tabla o2m con "añadir línea" | `One2many` con list editable | ESTÁNDAR |
| chatter (mensajes/notas/actividades) | herencia `mail.thread` + `mail.activity.mixin`, tag `<chatter/>` (v18+) | ESTÁNDAR (si el modelo hereda mail.thread) |
| badges de color por estado | `widget="badge"` sobre selection, o decoraciones | ESTÁNDAR/STUDIO |
| kanban con columnas | vista kanban agrupada por `stage_id`/`state` | ESTÁNDAR |
| filtros/facets + agrupar | search view (`<filter>`, `<group>`) | ESTÁNDAR/STUDIO |
| KPIs de dashboard | vista `graph`/`pivot` o Odoo Spreadsheet | ESTÁNDAR (graph/pivot) / config (Spreadsheet) |
| PDF (resguardo, orden…) | informe QWeb (`ir.actions.report`) | DESARROLLO (plantilla QWeb) |
| wizard (diálogo de acción) | `TransientModel` + form | DESARROLLO |

## Qué puede Studio (sin código)
- Añadir **campos** (incl. related, computed simples vía UI), ocultar/mostrar, reordenar `<group>`/`<page>`.
- Añadir **filtros** y vistas (list/kanban/calendar/pivot/graph) a un modelo.
- Cambiar **etiquetas**, `required`/`readonly`/`invisible` sencillos, dominios básicos.
- Botones que llamen a **acciones existentes** o automatizaciones (base automation).
- Añadir `mail.thread` (chatter) a un modelo desde Studio.

## Qué exige DESARROLLO
- Lógica de negocio: `compute` no triviales, `@api.constrains`, `@api.onchange` con lógica, overrides de `create/write/action_*`.
- Motores de precio/valoración, workflows de aprobación, estados derivados (**campo `state` calculado** a partir de hijos).
- **Widgets OWL custom**, JS/OWL a medida, botones que ejecutan métodos nuevos.
- Informes **QWeb** a medida; integraciones/API externas; multi-compañía con reglas custom.
- Agrupaciones/tablas que Odoo no ofrece nativas (ver límites abajo).

## Límites duros (marca DESARROLLO o "no nativo" en la anotación)
1. **o2m con secciones/subtotales anidados "tipo pedido de venta":** las *secciones* de `sale.order.line`
   son un truco de ese modelo (líneas de tipo nota/sección), NO un agrupador genérico de cualquier o2m.
   Agrupar una o2m por un campo con subtotales por grupo → **DESARROLLO** (caso real Autosur).
2. **Estado de la cabecera derivado de los hijos:** un `state` que se calcula agregando el estado de
   líneas/hijos es un campo **computed** → DESARROLLO (no es una etiqueta editable estándar).
3. **statusbar** solo refleja un campo selection/stage; no una combinación arbitraria de condiciones.
4. **smart button** necesita un contador real (campo/`count`); no es decorativo.
5. **Layout libre**: el form se compone con `<group>/<notebook>/<sheet>`; no es un lienzo libre. Diseños
   muy "a medida" (dashboards ricos, drag&drop shop-floor) → OWL custom = DESARROLLO.
6. **Kanban**: la tarjeta tiene una estructura acotada; en **v19 solo `<t t-name="card">`** (sin ella, error).
   Lógica rica en tarjeta = OWL.
7. **Editar en línea una o2m con widgets complejos** (p.ej. sub-o2m dentro de una fila) no es nativo.
8. **Portal/website** es otra capa (no backend) → `odoo-frontend`/`odoo-website-creator`, no este mockup.

## Deltas de versión que afectan al mockup (verificado vs fuente, 2026-06 — confirmar en `odoo-version-api-reference`)
- **v18**: `<tree>` → `<list>` (el tag `<tree>` ya no existe); `<chatter/>` como tag propio (en v17 era `<div class="oe_chatter">`); `attrs=`/`states=` eliminados desde v17 (usar `invisible="..."`, `readonly="..."`, etc.).
- **v19**: kanban **solo** `<t t-name="card">` (en v18 `kanban-box` avisa deprecación); `res.groups.category_id` → `privilege_id` (afecta a `groups=`).
- Ante cualquier duda de sintaxis → delega en `odoo-version-api-reference`; no lo pongas de memoria.

## Regla de oro del mockup honesto
Antes de dibujar algo "chulo", pregúntate: **¿esto sale de configurar un campo/vista, de Studio, o hay
que programarlo?** Si es lo tercero, se dibuja igual pero se **anota como DESARROLLO** con el porqué y, si
Odoo no lo hace nativo, se dice explícitamente. Eso convierte el mockup en un presupuesto de esfuerzo.
