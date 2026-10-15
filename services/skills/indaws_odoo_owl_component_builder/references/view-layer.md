# La capa de vistas en JS (v18)

Cómo funciona por dentro una vista del cliente web y cómo engancharse a ella: el modelo
relacional, el `Record`, y las tres formas de extender una vista estándar.

**Alcance.** Esto es el **JS**. El `arch` XML de las vistas (qué columnas, qué dominios, qué
atributos) es `odoo-view-designer`. Registrar un **tipo de vista nuevo** (que aparezca en el
switcher) es `odoo-view-designer/references/custom-view-type.md`. Aquí se trata de modificar cómo
se comporta una vista que ya existe.

Verificado contra `odoo18` el 2026-08-14. Rutas relativas a
`addons/web/static/src/`.

---

## Las cinco piezas

Una vista no es un componente: es un **descriptor** con cinco piezas que el registry monta.
El de lista, entero (`views/list/list_view.js`):

```js
export const listView = {
    type: "list",
    Controller: ListController,      // orquesta: barra de botones, acciones, estado de página
    Renderer: ListRenderer,          // pinta las filas; es donde vive casi todo lo visual
    ArchParser: ListArchParser,      // convierte el <list> XML en archInfo
    Model: RelationalModel,          // trae y mantiene los datos
    buttonTemplate: "web.ListView.Buttons",
    canOrderByCount: true,
    limit: 80,
    props: (genericProps, view) => { /* parsea el arch y compone las props */ },
};

registry.category("views").add("list", listView);
```

Las props comunes a toda vista están en `views/standard_view_props.js`: `resModel`, `arch`,
`domain`, `context`, `groupBy`, `orderBy`, `fields`, `resId`, `limit`… Un `Controller` recibe eso
más lo que añada el `props()` del descriptor.

## Cómo decide Odoo qué descriptor usar

Por el atributo **`js_class`** en la raíz del arch. Sin él, usa el tipo de vista:

```js
// views/view.js:345-355
const jsClass = archXmlDoc.hasAttribute("js_class")
    ? archXmlDoc.getAttribute("js_class")
    : props.jsClass || type;
if (!viewRegistry.contains(jsClass)) {
    await loadBundle("web.assets_backend_lazy");   // o _lazy_dark según el color scheme
}
const descr = viewRegistry.get(jsClass);
```

Dos consecuencias que importan:

- **La clave del registry ES el `js_class`.** Si registras `mi_lista`, ninguna vista la usa hasta
  que alguna ponga `js_class="mi_lista"` en su arch.
- Si la clave no está registrada, Odoo **carga primero el bundle lazy** y vuelve a mirar. Si
  tampoco está ahí, `viewRegistry.get` lanza `KeyNotFoundError`
  (`core/registry.js`, `get()`), no devuelve `undefined`. Un `js_class` mal escrito rompe la
  vista entera con ese error, no la degrada al tipo base.

## Las tres formas de extender, y cuál toca

### 1. `patch()` sobre el prototipo — afecta a TODAS las vistas de ese tipo

```js
import { patch } from "@web/core/utils/patch";
import { ListRenderer } from "@web/views/list/list_renderer";

patch(ListRenderer.prototype, {
    getPropertyFieldColumns(_, list) {
        const columns = super.getPropertyFieldColumns(...arguments);
        // ...
        return columns;
    },
});
```

Patrón real: `mail/static/src/views/web/list_renderer.js`. **Toda lista de la base de datos**
pasa por aquí, así que sirve para un comportamiento transversal (como hace `mail`) y es la
elección equivocada para "en mi módulo quiero que...". Si el cambio no vale para la lista de
asientos contables y para la de leads a la vez, no va aquí.

Medido el 2026-08-15: un `patch(ListRenderer.prototype)` que añade una clase en `getRowClass`
apareció en las filas de la lista de **`res.users`**, un modelo que no tiene nada que ver con el
módulo que puso el parche. No hay ámbito: es global a la instancia.

### 2. Descriptor nuevo + `js_class` — afecta solo a quien lo pida

Es el camino normal. Patrón real, `hr_recruitment/static/src/views/recruitment_list_view.js`:

```js
export class RecruitmentListRenderer extends ListRenderer {
    static template = "hr_recruitment.RecruitmentListRenderer";
    static components = { ...ListRenderer.components, RecruitmentActionHelper };
}

export const RecruitmentListView = {
    ...listView,                       // hereda Model, ArchParser, limit, buttonTemplate…
    Controller: RecruitmentListController,
    Renderer: RecruitmentListRenderer,
};

registry.category("views").add("recruitment_list_view", RecruitmentListView);
```

y en el XML (`hr_recruitment/views/hr_job_views.xml:298`, `hr_job_view_tree_inherit`):

```xml
<attribute name="js_class">recruitment_list_view</attribute>
```

Tres detalles del ejemplo que no son adorno:

- **`...listView` primero.** Copiar solo `Controller` y `Renderer` deja fuera `Model`,
  `ArchParser` y `props()`, y la vista no monta.
- **`static components = { ...ListRenderer.components, X }`.** Reasignar el objeto sin
  desestructurar el del padre quita los componentes que la plantilla heredada usa.
- **La plantilla se hereda con `t-inherit`**, no se reescribe; el `static template` apunta a la
  tuya.

### 3. Tipo de vista nuevo

Cuando no es "una lista distinta" sino un modo de visualización que no existe. Va con registro
Python además del JS: `odoo-view-designer/references/custom-view-type.md`.

---

## El `Record`: lo que de verdad manipulan los widgets

`model/relational_model/record.js`. Es el objeto que llega como `props.record` a cualquier field
widget, y su API es más pequeña de lo que parece.

**Leer**: `record.data` — pero ojo con qué es (abajo). `record.resId`, `record.isNew`,
`record.isValid`, `record.isInEdition`, `record.hasData`.

**Escribir**: solo `record.update(changes, { save })`.

```js
await record.update({ [props.name]: value });          // marca dirty; el onchange, SOLO si el campo lo tiene
await record.update({ amount: 10 }, { save: true });   // y además guarda
```

**Pero en una lista NO editable, `update()` no escribe nada y no lo dice.** Medido el 2026-08-14
con un tour real: el mismo widget, pulsado en un `<list>` sin `editable`, deja el valor donde
estaba —sin error, sin aviso en consola— y con `editable="bottom"` o en la vista form escribe
normalmente. Si estás depurando un widget que "no guarda", mira primero en qué vista está.

**Ciclo**: `save(options)`, `discard()`, `delete()`, `archive()` / `unarchive()`,
`duplicate()`, `switchMode(mode)`, `load()`.

**Validez**: `checkValidity()`, `isFieldInvalid(name)`, `setInvalidField(name)`,
`resetFieldValidity(name)`.

### `record.data` es una foto, no el estado

```js
// record.js:89
this.data = { ...this._values, ...this._changes };
```

Un objeto plano nuevo, fusión de lo que vino del servidor (`_values`) y lo cambiado
(`_changes`) — y los dos están en `markRaw`, o sea fuera de la reactividad. Escribir
`record.data.campo = x`:

- no dispara `onchange` (ni siquiera cuando el campo sí lo tendría),
- no marca el registro como sucio, así que el botón de guardar no se entera,
- y desaparece en cuanto algo llame a `_setData` (una recarga, un onchange de otro campo).

Es un fallo que no da ningún error: el valor se ve en pantalla hasta que se va solo.

### `invisible` no protege nada: se escribe y se guarda

Medido el 2026-08-15, y es la cara opuesta de lo del `readonly`. Un campo con `invisible="1"` en
el arch, escrito con `record.update()` desde el widget de **otro** campo:

```
web_save | vals=['hidden_note']
→ en base de datos: hidden_note='escrito-en-invisible'
```

Viaja y persiste. El filtro de `_getChanges` mira **`readonly`, no `invisible`**
(`web/static/src/model/relational_model/record.js:632-641`, `Record._getChanges`), así que la asimetría es real y va justo al revés de lo que se suele
suponer:

| En el arch | ¿Se puede escribir desde un widget? | ¿Llega al servidor al guardar? |
|---|---|---|
| `readonly="1"` | sí, y se ve en pantalla | **no**, salvo `force_save="1"` |
| `invisible="1"` | sí | **sí** |

Consecuencia que importa: **esconder un campo no impide que se escriba**. Si un valor no debe
poder cambiarse desde el cliente, eso se decide en el servidor —permisos, `readonly` de verdad en
el campo, o una comprobación en `write()`—, no ocultándolo en la vista. Y en el sentido útil: si
tu widget necesita escribir un campo que no quieres pintar, `invisible="1"` funciona; el que no
funciona es `readonly="1"`.

### `isDirty()` es **asíncrono**

```js
// record.js:207-210
async isDirty() {
    await this.model._askChanges();
    return this.dirty;
}
```

Espera a que los campos en edición confirmen sus cambios pendientes antes de responder.
`if (record.isDirty())` sin `await` evalúa una Promise, que siempre es truthy: te dirá que hay
cambios **siempre**, incluso recién cargado. La propiedad cruda `record.dirty` sí es síncrona,
pero puede ir por detrás de lo que el usuario acaba de teclear.

### El guardado: qué llega al servidor y qué se pierde por el camino

Medido el 2026-08-15 en el banco (`tools/bench-odoo18`), con un tour de 9 pasos en verde y un
override de `web_save` que registra lo que recibe.

**1. Un campo `readonly` escrito por un widget se pierde al guardar, y se ve en pantalla hasta
que se pierde.** Es lo más caro de esta página. Dos campos `readonly="1"` en el arch, los dos
escritos con `record.update()` desde un widget, los dos mostrando el valor en la interfaz. Al
guardar:

```
web_save | vals=['forced_note']          <- locked_note NO viaja
→ en base de datos:  locked_note=False   forced_note='escrito-por-widget'
```

La diferencia entre los dos es un atributo del arch: `force_save="1"`. El filtro está en
`_getChanges` (`web/static/src/model/relational_model/record.js:632-641`, `Record._getChanges`):

```js
if (!withReadonly && fieldName in this.activeFields
    && this._isReadonly(fieldName) && !this.activeFields[fieldName].forceSave) {
    continue;                                   // se descarta del payload
}
```

y `_save` lo llama **sin** `withReadonly` (`web/static/src/model/relational_model/record.js:1003`, `Record._save`), o sea que en el guardado ese filtro
siempre actúa. `forceSave` sale de `force_save` en el arch (`web/static/src/views/fields/field.js:235-236`, `Field`).

El usuario ve el valor, le da a guardar, el formulario dice «guardado» y el dato no existe.
Ningún error, ninguna traza. Si tu widget escribe un campo que la vista marca readonly, o pones
`force_save="1"`, o el valor no llega.

**1c. Ni siquiera sobrevive lo que calculó el propio servidor.** Un `@api.onchange` que rellena
un campo `readonly="1"` del arch: el valor llega, **se pinta en pantalla** —el tour lo afirma— y
al guardar tampoco viaja.

```
web_save | vals=['amount']       <- ni onch_ro ni echo, los dos rellenados por el onchange
→ en base de datos:  amount=1.0   echo=False   onch_ro=False
```

El filtro de `_getChanges` **no mira de dónde vino el valor**: descarta todo lo readonly sin
`force_save`, lo hubiera escrito un widget, el usuario o el propio onchange del servidor. O sea
que el patrón «onchange que rellena un campo informativo readonly» **no persiste**: se ve mientras
el formulario está abierto y desaparece al guardar.

Lo que sí funciona para eso es un **campo calculado y almacenado** (`compute` + `store=True`), que
se resuelve en el servidor y no depende del payload del cliente. Si tiene que ser onchange, el
campo necesita `force_save="1"`.

**1b. Y lo peor: un `readonly` CONDICIONAL descarta lo que se escribió cuando NO lo era.** Medido
el 2026-08-15, con `readonly="amount == 1"` en el arch:

1. Con `amount = 0` el campo es editable — el widget lo confirma (`readonly=false`).
2. Se escribe. El valor aparece en pantalla.
3. Se cambia `amount` a 1 **sin guardar**. El modificador **se re-evalúa en vivo** y el campo pasa
   a readonly.
4. Se guarda:

```
web_save | vals=['amount']     <- cond_note NO viaja
→ en base de datos:  amount=1.0   cond_note=False
```

El usuario escribió el valor cuando tenía todo el derecho a hacerlo, lo vio en pantalla, y lo
perdió sin un aviso. La causa es que `_getChanges` evalúa `_isReadonly` **en el momento de
guardar** (`web/static/src/model/relational_model/record.js:635`, `Record._getChanges`), no cuando se escribió, y para entonces la condición ya es cierta.

**Esto no es un caso rebuscado**: es el patrón `readonly="state != 'draft'"` de media base de
código de Odoo. Rellenar campos y confirmar en la misma sesión, sin guardar por medio, tira lo
rellenado. Si tu flujo se parece a eso, o guardas antes de que cambie la condición, o el campo
lleva `force_save="1"`.

**Los modificadores se re-evalúan contra lo no guardado.** El paso 3 lo prueba: bastó cambiar
`amount` en memoria para que el campo pasara a readonly. Se evalúan con
`evalContextWithVirtualIds` (`web/static/src/model/relational_model/record.js:699`, `Record._isReadonly`), igual que el `context` del arch.

**2. Solo viajan los campos que cambiaron.** En la medida, `vals` traía un único campo aunque el
registro tenía ocho activos. El `specification` de vuelta sí los lleva todos, incluidos los
invisibles.

**3. Guardar sin cambios no hace RPC.** Dos clics en guardar, **una sola** llamada a `web_save`.
Con el registro ya creado y sin cambios pendientes, `_save` limpia el estado y devuelve `true`
sin hablar con el servidor (`web/static/src/model/relational_model/record.js:1004-1012`, `Record._save`). Un test que cuente llamadas para comprobar
que «guardó» puede estar contando cero y estar bien.

### El guardado urgente al salir de la página — medido

**Irse de una página con cambios sin guardar los guarda**, sin que nadie pulse nada. La cadena
completa, verificada en v18 y **medida en v18 y v19** el 2026-08-15:

```
window "beforeunload"                       search/action_hook.js:77
  └─ FormController.beforeUnload            form_controller.js:498-504
      └─ record.urgentSave()                record.js:267-272
          └─ navigator.sendBeacon(...)      record.js:1013-1046
```

El `sendBeacon` va a `/web/dataset/call_kw/<modelo>/web_save`, y solo entra por ahí si el modelo
lleva `useSendBeaconToSaveUrgently`, que el formulario pone a `true` (`web/static/src/views/form/form_controller.js:383`, `FormController.modelParams`) y la
vista de lista no.

**Cómo se reconoce en un log de servidor, que es lo útil:** el guardado urgente llega con
`specification` **vacía** (`web/static/src/model/relational_model/record.js:1030`, `Record._save`), mientras que un guardado normal manda la
especificación entera de campos. Medido, la misma sonda en la misma sesión:

| Origen | `web_save` recibido |
|---|---|
| botón de guardar | `vals=['amount']` · `spec_keys=['amount', 'cond_note', 'display_name', …]` |
| salir de la página | `vals=['amount']` · **`spec_keys=[]`** |

Así que **un `web_save` con `specification` vacía en producción es alguien cerrando una pestaña**,
no una integración rara.

Lo que **no** mide esto: si el navegador entrega el beacon cuando el proceso muere de golpe (matar
la pestaña, apagón). Eso es política del navegador, no de Odoo. Sí se mide la salida por navegación,
que es el caso frecuente.

Y el límite: si los cambios no caben en el beacon (~64k), `sendBeacon` devuelve `false` y Odoo saca
una notificación pegajosa pidiendo guardar a mano (`web/static/src/model/relational_model/record.js:1038-1046`, `Record._save`) — esa rama sigue siendo
lectura de fuente, porque provocarla pide un payload de 64k.

**Para medirlo hay que descargar la página de verdad.** Un `dispatchEvent(new Event("beforeunload"))`
sirve en v18, pero **v19 vigila ese evento y aborta el tour** si el paso no declara
`expectUnloadPage: true` (`web_tour/static/src/tour_service/tour_automatic.js:142-153`, `TourAutomatic.start`), y con la clave puesta exige que la página se
descargue de verdad en 20 s (`:65-69`). La sonda navega (`window.location.href = "/odoo"`), que
además es más fiel a lo que hace un usuario.

(Lo de `_checkValidity` antes de cualquier RPC sí está medido, en el caso de la línea inválida.)

### `domain` y `context` del arch: se evalúan en el cliente, con lo no guardado

Medido el 2026-08-15. Un many2one con las dos cosas escritas como expresión sobre campos del
propio registro:

```xml
<field name="partner_id"
       domain="[('name', 'like', name)]"
       context="{'dyn_ctx_amount': amount, 'dyn_ctx_name': name}"/>
```

Lo que recibe el servidor en el `name_search` del autocompletado:

```
name_search | name='' | domain=[['name', 'like', 'PROBE-NAME']]
            | contexto: {'dyn_ctx_amount': 1, 'dyn_ctx_name': 'PROBE-NAME'}
```

Tres cosas:

- **El dominio llega resuelto, no como expresión.** El cliente lo evalúa con los valores del
  registro (`getFieldDomain`, `web/static/src/model/relational_model/utils.js:313`, `getFieldDomain`) y manda una lista literal. En
  el servidor no hay forma de saber qué expresión lo generó.
- **El contexto también, y con lo que el usuario aún no ha guardado.** En la medida, `amount`
  valía 1 porque se había pulsado el botón antes de la búsqueda y **no se había guardado**: el
  contexto se compone con `makeContext([...], record.evalContext)`, y ese `evalContext` es el
  estado en memoria. Un `context` del arch ve lo tecleado, no lo persistido.
- **La primera búsqueda sale con `name=''`.** El autocompletado dispara al abrir, antes de que
  haya nada escrito. Si tu `name_search` es caro o tiene efectos, se ejecuta también con la
  cadena vacía.

### El onchange, visto desde el JS

Medido el 2026-08-15 con un módulo de sonda y un tour en Odoo 18: un modelo con
`@api.onchange('amount')`, un campo `plain` sin onchange, y el mismo widget en los dos.

**1. El widget NO decide si hay onchange.** Lo decide un flag del campo:

```js
// record.js:1204-1206
const onChangeFields = Object.keys(changes).filter(
    (fieldName) => this.activeFields[fieldName] && this.activeFields[fieldName].onChange
);
if (!onChangeFields.length) { return {}; }
```

Por defecto es `false` (`web/static/src/model/relational_model/utils.js:44`, `makeActiveField`) y sale de la spec que manda el
servidor, no de tu componente. Medido en pantalla: `amount` llegó con `onChange=true` y `plain`
con `onChange=false`.

**2. Un `update()` sobre un campo sin ese flag escribe y no llama al servidor.** Sin error, sin
aviso. En el banco, pulsar el botón de `plain` subió el valor y el campo que rellena el onchange
**se quedó vacío**. Si esperas que tu escritura dispare lógica de servidor y el campo no tiene
onchange declarado, no pasa nada y no hay nada que leer.

**2b. Ojo: que el onchange rellene un campo no significa que se guarde.** Lo medido en esta
sección es lo que pasa **en pantalla**. Si el campo que rellena es `readonly` en el arch, el valor
se ve y luego se pierde al guardar — ver «El guardado», punto 1c.

**3. Cuando sí dispara, viaja TODO lo pendiente, no solo el campo que lo provocó.** Esto es lo
que más sorprende. El log del servidor en el banco, con `plain` cambiado antes y sin guardar:

```
onchange | field_names=['amount']
         | values_keys=['amount', 'plain']
         | spec_keys=['amount', 'display_name', 'echo', 'hidden_note', 'name', 'plain']
```

- `field_names` — solo el campo que disparó.
- `values` — **todos los cambios pendientes del registro**, `plain` incluido, aunque no tenga
  onchange y no esté guardado (`web/static/src/model/relational_model/record.js:1211-1214`, `Record._getOnchangeValues`, con `withReadonly: true`).
- `spec` — lo que se pide de vuelta, **incluidos los invisibles**: `hidden_note` iba
  `invisible="1"` en el arch y llegó igual (`getFieldsSpec(..., {withInvisible: true})`,
  `web/static/src/model/relational_model/relational_model.js:605`, `RelationalModel._onchange`).

**En un x2many se adjunta el estado del padre**, bajo la clave del `relationField`
(`web/static/src/model/relational_model/record.js:1215-1221`, `Record._getOnchangeValues`). Medido el 2026-08-15 con un `@api.onchange` en la línea, ensuciando el
padre **antes** de tocar la línea:

```
al añadir la fila:   field_names=[]      keys=['probe_id']
                     probe_id = {'id': …, 'plain': 1}
al cambiar la qty:   field_names=['qty']  keys=[hidden_cell, hidden_col, locked_note, name,
                                                probe_id, qty]
                     probe_id = {'id': …, 'line_ids': …, 'plain': 1}
```

Tres cosas de ahí:

- El padre viaja como **dict**, con sus cambios **sin guardar** dentro: `plain: 1` estaba tocado y
  no guardado.
- En la segunda llamada el padre lleva además **`line_ids`**, es decir los comandos pendientes de
  las otras líneas. Un onchange de línea puede ver el resto del conjunto sin guardar.
- **Añadir una fila dispara un onchange con `field_names=[]`** — el de valores por defecto —, y
  ese solo lleva `probe_id`. Si tu onchange de línea no comprueba `field_names`, corre también
  ahí.

**4. Si el onchange revienta, el valor vuelve atrás.** El cliente aplica los cambios y los
revierte a propósito antes de relanzar, con el comentario *"to force a render of the Field
components"* (`web/static/src/model/relational_model/record.js:1229-1233`, `Record._getOnchangeValues`). Medido el 2026-08-15: un `@api.onchange` que lanza
`UserError` al llegar a 2; sale el diálogo de error y, al cerrarlo, **el campo muestra el valor
anterior**, no el que se intentó poner. O sea que el usuario no se queda con un valor a medias en
pantalla.

**Al instrumentarlo: `onchange` NO lleva `@api.model`.** La firma es
`onchange(self, values, field_names, fields_spec)` (`addons/web/models/models.py:866`, `Base.onchange`). Si
sobreescribes con `@api.model`, el primer argumento deja de consumirse como recordset y llegan
cinco posicionales: `TypeError: onchange() takes 4 positional arguments but 5 were given`. Pasó
montando este banco.

### x2many: `StaticList`, y qué comandos salen de verdad

Un campo one2many o many2many llega como `StaticList`
(`model/relational_model/static_list.js`), no como un array. Lo útil:
`records`, `currentIds`, `resIds`, `editedRecord`, `addNewRecord()`, `delete(record)`,
`linkTo(resId)`, `unlinkFrom(resId)`, `moveRecord()`, `resequence()`, `sortBy(fieldName)`,
`enterEditMode(record)` / `leaveEditMode()`.

Medido el 2026-08-15 en el banco, con un o2m `editable="bottom"` y un `web_save` instrumentado.
En los tres casos el `vals` del padre iba **vacío**: solo viajó `line_ids`.

| Acción en la interfaz | Lo que llega al servidor |
|---|---|
| Añadir una línea | `[[0, 'virtual_3', {'name': 'L1', 'qty': 3}]]` |
| Cambiar la cantidad de una línea existente | `[[1, 2, {'qty': 9}]]` |
| Borrar una línea | `[[2, 3]]` |

Cuatro cosas que no se adivinan:

- **El id de una línea nueva es una cadena `'virtual_N'`, no un `0`.** En Python se escribe
  `(0, 0, vals)`; lo que manda el cliente lleva el id virtual en la segunda posición. Si
  instrumentas o interceptas el payload, no esperes un cero.
- **En un `write` de línea solo viaja el campo tocado**, no la línea entera: `{'qty': 9}` y nada
  más, aunque la línea tuviera otros tres campos.
- **Al borrar sale el comando 2 (DELETE), no el 3 (UNLINK)**, y son dos elementos, sin dict. La
  fila desaparece de verdad de la tabla (comprobado en base de datos).
- **El descarte de los readonly también actúa dentro de las líneas, incluso al crearlas.** En el
  banco, la línea nueva llevaba un campo `readonly="1"` escrito por un widget: no apareció en el
  payload y quedó a `False` en base de datos. Es el mismo filtro de `_getChanges`, que en el ramo
  x2many llama a `_getCommands({withReadonly})` con el mismo valor (`web/static/src/model/relational_model/record.js:648`, `Record._getChanges`).

**Un x2many que pasa a `readonly` con líneas pendientes NO pierde los datos: bloquea el
guardado.** Medido el 2026-08-15 con `readonly="amount == 1"` en el propio `line_ids`: se añade
una línea con la condición falsa, se cambia el campo del que depende, se pulsa guardar — y salen
**cero llamadas a `web_save`**. El formulario se queda sucio y no avanza.

Es un comportamiento **distinto** al del campo escalar, y conviene no confundirlos:

| Lo que se vuelve readonly con cambios pendientes | Qué pasa al guardar |
|---|---|
| Un campo escalar | el guardado **sigue** y el valor se descarta en silencio |
| Un x2many | el guardado **no se intenta**: cero RPC, formulario sucio |

O sea que con las líneas el usuario se queda atascado en vez de robado, que es menos grave pero
igual de desconcertante si nadie te lo ha contado. **El mecanismo exacto no lo he determinado** —
lo medido es el número de llamadas y el estado del formulario.

**Dentro de una línea, `invisible` y `column_invisible` tampoco protegen nada.** Medido el
2026-08-15 con los dos a la vez en la misma línea, escritos desde el widget de una celda visible:

```
lines=[[0, 'virtual_3', {'name': 'L-oculta', 'qty': 0,
                         'hidden_col': 'col-escrito', 'hidden_cell': 'cell-escrito'}]]
→ en base de datos:  hidden_col='col-escrito'   hidden_cell='cell-escrito'
```

Los dos viajan y los dos persisten, igual que a nivel de padre. El único que se cae sigue siendo
el `readonly` (`locked_note` no aparece en ese payload). O sea que la regla es la misma dentro y
fuera de un x2many: **oculta el `readonly`, no lo `invisible`.**

**Y al CREAR una línea viaja todo lo activo, no solo lo que tocaste.** En esa medida, `qty: 0`
está en el payload y nadie la escribió: es el valor por defecto de un campo activo. Contrasta con
el `write` de una línea existente, que llevaba **solo** el campo cambiado (`{'qty': 9}`). Si
cuentas con que el servidor solo recibe lo tecleado, en la creación no es así.

**Un `required` condicional que se activa con el campo vacío también bloquea, y lo dice.**
Medido: con `required="amount == 1"` y el campo vacío, cambiar `amount` a 1 y pulsar guardar marca
la celda como inválida y no guarda. Aquí el usuario **sí ve** por qué, al contrario que en el
`readonly`, que se lleva el dato sin mediar palabra.

**Una línea inválida bloquea el guardado del padre sin un solo RPC.** Con el `required` de la
línea vacío, pulsar guardar marca la celda como inválida y **no sale ninguna llamada a
`web_save`** — cero, medido. La validación corre antes de componer el payload
(`web/static/src/model/relational_model/record.js:1002`, `Record._save`), así que no hay nada que mirar en el servidor cuando alguien dice "le doy a
guardar y no pasa nada": está pasando en el navegador.

### Reaccionar a cambios: `useRecordObserver`

```js
import { useRecordObserver } from "@web/model/relational_model/utils";

setup() {
    useRecordObserver((record) => {
        // se ejecuta en el setup y cada vez que cambie algo que este callback LEA
    });
}
```

Solo vale **en un componente de campo**, porque depende de `props.record`
(`web/static/src/model/relational_model/utils.js:682-690`).

Medido el 2026-08-15 en el banco, con un callback que lee **solo** `record.data.amount` y lleva
un contador en pantalla:

| Lo que pasa | El contador |
|---|---|
| Montar el componente | `1` — corre una vez en el setup |
| Cambiar `plain`, que el callback **no lee** | sigue en `1`: no se despierta |
| Cambiar `amount`, que **sí lee** | `2` — **una sola vez**, no varias |

O sea que el seguimiento es **por lectura, no por registro**: lo que el callback no toque no lo
despierta, por mucho que el registro cambie. Y no se dispara en ráfaga: un cambio, una ejecución.

Un detalle que sale del mismo banco: el widget estaba montado sobre el campo `name` y observaba
`amount`. Un componente de campo puede observar **cualquier** campo del registro, no solo el
suyo.

---

## GOTCHAS

1. **`js_class` mal escrito rompe la vista, no la degrada.** `viewRegistry.get()` lanza
   `KeyNotFoundError` (`core/registry.js`). Antes de eso Odoo carga `web.assets_backend_lazy`,
   así que el fallo llega con retraso y puede parecer un problema de assets.
2. **`patch(ListRenderer.prototype, …)` toca todas las listas de la instancia**, no solo las de
   tu módulo. Para algo de tu módulo, descriptor nuevo + `js_class`.
3. **Un descriptor sin `...listView` no monta.** Faltan `Model`, `ArchParser` y `props()`, y el
   error no dice eso.
4. **`static components = {...}` sin heredar los del padre** deja la plantilla heredada
   apuntando a componentes que ya no existen.
5. **Mutar `record.data` no escribe nada.** Es una fusión plana en `markRaw` (`web/static/src/model/relational_model/record.js:89`, `Record._setData`);
   usa `record.update({...})`, que pasa por el mutex del modelo.
10. **`record.update()` NO siempre dispara el onchange, y esto no da error.** Solo se manda al
   servidor lo que lleve `onChange: true` en su `activeField` — `_getOnchangeValues` filtra por
   ahí (`web/static/src/model/relational_model/record.js:1204-1206`, `Record._getOnchangeValues`) y el valor por defecto es `false` (`web/static/src/model/relational_model/utils.js:44`, `makeActiveField`).
   El flag viene de la spec de campos, no de tu widget: si el campo no lo trae, tu `update()`
   escribe y el onchange del modelo no corre. Verificado contra v18 el 2026-08-15; **no probado
   ejecutándolo**.
6. **`await record.isDirty()`**, siempre. Sin `await` es una Promise y siempre da truthy
   (`web/static/src/model/relational_model/record.js:207`, `Record.isDirty`).
7. **El `Renderer` es donde vive lo visual, el `Controller` no.** Tocar el Controller para
   cambiar cómo se pinta una fila es el error de reparto más común; el Controller lleva botones,
   acciones y estado de página.
8. **El `ArchParser` corre una vez, en `props()`.** Lo que dependa de datos del servidor no puede
   resolverse ahí: `archInfo` se calcula al montar la vista, antes de que el modelo cargue.
9. **`record.update()` en una lista sin `editable` no hace nada, en silencio.** Ni excepción ni
   log. Medido con tour: mismo widget, `1 → 1` en la lista de solo lectura y `1 → 2` en form y en
   lista `editable="bottom"`. El mecanismo exacto no está fijado a una línea de fuente; el
   comportamiento sí está medido y se reproduce.

## Qué de esto está probado ejecutándolo

El 2026-08-14 se montó un módulo de sonda contra un Odoo 18 limpio y se corrió con tours en
Chrome headless. **Probado en verde**: el estado del padre que viaja en el onchange de una línea · que un onchange fallido devuelve el valor anterior · que ni siquiera un valor calculado por el propio `@api.onchange` sobrevive en un campo readonly · que un `readonly` condicional tira lo escrito cuando el campo aún era editable · que el `domain` y el `context` del arch llegan al servidor resueltos y con los valores sin guardar · que `invisible` y `column_invisible` sí se guardan —también dentro de una línea x2many— y `readonly` no · que `patch(ListRenderer.prototype)` alcanza listas de otros módulos · `useRecordObserver` (sigue por lectura, una ejecución por cambio, y puede observar campos que no son el suyo) · los comandos x2many (crear/editar/borrar, el id virtual, el readonly descartado también en líneas, y que una línea inválida bloquea sin RPC) · el guardado (readonly descartado salvo `force_save`, solo viajan los cambios, sin cambios no hay RPC) · el onchange entero (flag por campo, que un `update()` sin flag no llama al servidor, que viajan todos los cambios pendientes y que el spec lleva los invisibles) — banco de 9 pasos, 0 failed · el descriptor propio con `...listView` + `js_class` + la
plantilla del renderer heredada con `t-inherit` (el banner apareció), `record.update()` en form y
en lista editable, y `await record.isDirty()` devolviendo un booleano de verdad. **Salió del
banco** el gotcha 9, que no estaba escrito. Todo lo de esta página está medido salvo lo que se marca como leído en fuente en su propio párrafo.

## v18 frente a v19: medido, no leído

El 2026-08-15 el banco se convirtió en **suite de regresión** —cada hallazgo de esta página es un
assert— y se corrió contra los dos árboles con la misma imagen y el mismo módulo:

| | v18 | v19 |
|---|---|---|
| Sondas en verde | **20 de 20** | **19 de 20** |

**Todo el grupo de la pérdida silenciosa se comporta igual en las dos versiones**: el `readonly`
descartado al guardar, el `readonly` condicional que tira lo escrito, el valor del onchange que no
sobrevive, lo `invisible` que sí persiste, los comandos x2many, y `useRecordObserver`. Nada de eso
cambió en v19.

**El `domain` y el `context` se comportan igual en v19.** Medido: el `name_search` de v19 recibe
exactamente lo mismo que el de v18 —dominio resuelto, contexto con los valores sin guardar, y la
primera llamada con `name=''`—. Lo que cambió es **la estructura del DOM**: v19 reescribió el
many2one y `web.Many2OneField` solo delega en un componente `<Many2One>` que envuelve todo en
`<div class="o_many2one">` (`web/static/src/views/fields/many2one/many2one.xml:19`, `web.Many2One`). Un tour de v18 que
apunte al `input` por dentro del campo no lo encuentra.

De ahí sale el único rojo que queda en v19 (`TestCtxDomain`), y es del tour, no del producto: con
el desplegable del autocompletado abierto, el botón de descartar no se deja pulsar. La medición ya
está tomada; cerrar ese paso es higiene pendiente del banco.

> **Corrección del 2026-08-15**, y vale la pena leerla porque es un error de método. Una primera
> pasada dio *17 de 20* y de ahí escribí que en v19 `discussComponentRegistry` «ya no sustituye el
> diálogo del core». **Era falso.** Lo que pasaba es que una sonda de mail mal portada lanzaba una
> excepción y **un solo error de JS tumba el bundle entero**, así que arrastraba a otras dos
> sondas. Portada bien, `discussComponentRegistry` funciona igual en v19. La regla: antes de
> anunciar un cambio de versión, bisecciona las sondas y corre lo mismo en la otra.

### Lo que además se ve leyendo la fuente de v19

El descriptor y las rutas **no cambian** (`@web/model/relational_model/…` sigue igual, y el shape
de `listView` es el mismo salvo reordenación). v19 añade
`readonly: genericProps.readonly || !archInfo.activeActions?.edit` en las props del descriptor de
lista, y un `operation.js` nuevo en el modelo relacional — ninguno de los dos se ha medido.
