# La mitad JS de mail/Discuss (v18)

El resto de esta skill es Python: `mail.thread`, `message_post`, tracking, plantillas. Esto es lo
otro — el chatter, el composer, Discuss y los mensajes **en el navegador**, que no comparten casi
nada con el servidor salvo los nombres.

Verificado contra `odoo18` el 2026-08-14. Rutas relativas a `addons/mail/static/src/`.

---

## Lo primero que sorprende: mail tiene su propio ORM en el cliente

No usa `relational_model` (el de las vistas, ver
`odoo-owl-component-builder/references/view-layer.md`). Tiene el suyo, en `model/record.js`, con
modelos, campos, relaciones e inversas — un ORM reactivo en memoria que se llena por RPC y por
el bus.

Tres capas, y conviene no confundirlas:

| Capa | Dónde | Qué es |
|---|---|---|
| El ORM | `model/record.js`, `model/store.js` | `Record`, campos `one`/`many`/`attr`, inversas, `insert` |
| Los modelos | `core/common/*_model.js` | `Message`, `Thread`, `Persona`, `Follower`, `Attachment`, `ChatWindow`… |
| Los componentes | `core/common/*.js`, `chatter/`, `discuss/` | `Chatter`, `Composer`, `Thread`, `Message` |

## El store

Es un service normal, registrado como `mail.store`
(`mail/static/src/core/common/store_service.js:802`, `mail.store`), y se coge así — envuelto en `useState`, porque lo que
interesa es que el componente re-renderice cuando el store cambie:

```js
// mail/static/src/chatter/web_portal/chatter.js:28
this.store = useState(useService("mail.store"));
```

Desde ahí se llega a todo: `store.Message`, `store.Thread`, `store.self`, `store.discuss`.

El store depende de `bus_service` (`mail/static/src/core/common/store_service.js:780`, `storeService`), que es lo que trae los mensajes
nuevos sin recargar. Lo que llega por el bus entra por el mismo `insert` que usa el RPC — y ojo,
**lo que no esté declarado como campo se asigna igual, como atributo suelto** (gotcha 5).

## Declarar un modelo

Campos como **propiedades de clase**, y `register()` al final del fichero
(`core/common/message_model.js`):

```js
import { Record } from "@mail/core/common/record";

export class Message extends Record {
    static id = "id";              // clave por la que se deduplica
    static records = {};

    attachment_ids = Record.many("Attachment", { inverse: "message" });
    author = Record.one("Persona");
    body = Record.attr("", { html: true });
}
Message.register();
```

`Record.one(modelo, opts)`, `Record.many(modelo, opts)` y `Record.attr(default, opts)`
(`mail/static/src/model/record.js:227`, `Record.one`, `:253`, `:278`). Opciones útiles: `inverse` (relación bidireccional),
`sort`, `onDelete`, `html`.

## Extender un modelo desde otro módulo

**Aquí está el detalle que no se adivina**: al parchear, los campos NO van como propiedades, van
**dentro de `setup()`**. Patrón real, `core/web/thread_model_patch.js`:

```js
import { Thread } from "@mail/core/common/thread_model";
import { Record } from "@mail/core/common/record";
import { patch } from "@web/core/utils/patch";

patch(Thread.prototype, {
    setup() {
        super.setup();
        this.recipients = Record.many("Follower");
        this.activities = Record.many("Activity", {
            sort: (a, b) => compareDatetime(a.date_deadline, b.date_deadline) || a.id - b.id,
            onDelete(r) { r.remove(); },
        });
    },
    get recipientsFullyLoaded() {
        return this.recipientsCount === this.recipients.length;
    },
    computeIsDisplayed() {
        // ...
        return super.computeIsDisplayed();
    },
});
```

Declarar `recipients = Record.many(...)` como propiedad en el patch no funciona: `patch` copia
propiedades sobre el prototipo, y el campo hay que crearlo por instancia, que es lo que hace
`setup()`.

## Comparar registros: `.eq()`, nunca `===`

```js
// model/record.js:396
eq(record) {
    return toRaw(this)._raw === toRaw(record)._raw;
}
```

Todo lo que sale del store viene envuelto en proxies reactivos de OWL, así que dos referencias al
**mismo** mensaje pueden ser dos proxies distintos y `===` da `false`. Usa `.eq()`, `.notEq()`,
`.in(coleccion)`, `.notIn(coleccion)` (`:396-414`). Es un fallo que no da error: simplemente el
mensaje no se marca como seleccionado, o el hilo activo no se resalta.

## Los tres puntos de extensión que cubren casi todo

### 1. Una acción en el menú de un mensaje

```js
// discuss/message_pin/common/message_actions.js
import { messageActionsRegistry } from "@mail/core/common/message_actions";

messageActionsRegistry.add("pin", {
    condition: (component) =>
        component.store.self.type === "partner" &&
        component.props.thread?.model === "discuss.channel",
    icon: "fa fa-thumb-tack",
    title: (component) => (component.props.message.pinned_at ? _t("Unpin") : _t("Pin")),
    onClick: (component) => component.props.message.pin(),
    sequence: 65,
});
```

El registry es `registry.category("mail.message/actions")`
(`mail/static/src/core/common/message_actions.js:17`, `messageActionsRegistry`). `condition` es lo que evita que tu acción salga en
chatters donde no pinta nada.

**El `sequence` decide si se ve o si se esconde, y esto cuesta una tarde.** La plantilla parte las
acciones en dos: las primeras se pintan como botones (`mail/static/src/core/common/message.xml:157-168`, `mail.Message.actions`) y **el resto van a un
desplegable cerrado** (`messageActions.actions.slice(quickActionCount - 1)`, `mail/static/src/core/common/message.xml:175`, `mail.Message.actions`).
Medido el 2026-08-14 con un tour real: con `sequence: 95` la acción **no aparece** en el chatter
—está registrada y funciona, pero dentro del menú "..."— y con `sequence: 1` se ve. El síntoma es
idéntico a "no se registró", así que antes de buscar el fallo en tu registro, abre el desplegable.
Dónde cae el corte (`quickActionCount`) depende del contexto y no está medido aquí.

### 2. Sustituir un componente de Discuss

`discussComponentRegistry` = `registry.category("discuss.component")`
(`core/common/discuss_component_registry.js`). Se usa para romper dependencias circulares
(`Message` ↔ `MessageConfirmDialog`) y como punto de sustitución.

**Medido el 2026-08-15 y funciona de verdad**: registrando un componente propio bajo
`MessageConfirmDialog` con `{force: true}`, el flujo de borrado del **core** abrió ese diálogo en
vez del suyo. El core lo resuelve por el registry al pulsar borrar
(`mail/static/src/core/common/message_actions.js:17`, `messageActionsRegistry`), así que la sustitución alcanza a código que no es tuyo.
Tour de 7 pasos: publicar una nota en el chatter, desplegar el menú del mensaje, pulsar borrar y
comprobar que el diálogo es el propio.

### 3. El `<chatter/>` del formulario

El tag `<chatter/>` del arch (v18+; en v17 era `<div class="oe_chatter">`) lo convierte un
**form compiler**:

```js
// chatter/web/form_compiler.js:45-48
registry.category("form_compilers").add("chatter_compiler", {
    selector: "chatter",
    fn: compileChatter,
});
```

O sea que el chatter no es un widget de campo ni un componente que montes tú: es una regla de
compilación del formulario. Para cambiar qué props recibe, se parchea `FormCompiler.prototype`
(lo hace el propio `mail` justo debajo, `:55`).

**Medido el 2026-08-15**: registrando `chatter_compiler` con `{force: true}` y una `fn` que
devuelve un `div` cualquiera, en el sitio del `<chatter/>` aparece ese div y **el chatter deja de
existir** en el formulario. Es el punto de control del chatter entero, no un detalle de props.

El componente en sí (`mail/static/src/chatter/web_portal/chatter.js:22-24`, `Chatter`):

```js
static template = "mail.Chatter";
static components = { Thread, Composer };
static props = ["composer?", "threadId?", "threadModel", "twoColumns?"];
```

---

## GOTCHAS

1. **El ORM de mail no es el de las vistas.** `@mail/core/common/record` y
   `@web/model/relational_model/record` se llaman igual, tienen APIs distintas y no son
   intercambiables. El `record` de un field widget no entiende `.eq()`; el de mail no entiende
   `.update({...})` con onchange.
2. **`===` entre registros de mail da `false` aunque sean el mismo.** Proxies reactivos:
   usa `.eq()` (`mail/static/src/model/record.js:396`, `Record.eq`). No lanza error, solo deja de funcionar la selección.
3. **Al parchear un modelo, los campos van dentro de `setup()`**, no como propiedades de clase.
   Como propiedad simplemente no existe el campo, y el síntoma es `undefined` en la plantilla.
4. **`super.setup()` primero, siempre**, o los campos del modelo base no se crean.
5. **Un campo que no declaras NO se descarta: se asigna igual, como atributo suelto.**
   `updateFields` manda al camino de atributo todo lo que no sea una relación declarada
   (`mail/static/src/model/store_internal.js:191-197`, `StoreInternal.updateFields`):

   ```js
   if (!record.Model._.fields.get(fieldName) || record.Model._.fieldsAttr.get(fieldName)) {
       this.updateAttr(record, fieldName, value);      // <- lo NO declarado cae aquí
   } else {
       this.updateRelation(record, fieldName, value);
   }
   ```

   Medido el 2026-08-15: el servidor mandó dos claves al `Thread` por `_thread_to_store`, el
   patch de JS declaró **solo una**, y las dos llegaron con su valor. Lo que pierdes al no
   declarar no es el dato: es el tratamiento **relacional** —`inverse`, `sort`, `onDelete`— que
   solo se aplica a un `Record.one`/`Record.many` declarado.

   <sub>Corregido 2026-08-15. Esta entrada decía «se descarta sin aviso» y mandaba a mirar la
   declaración del modelo cuando un dato «no llega». Era al revés, y escrito de lectura de fuente
   sin comprobarlo.</sub>
6. **`useService("mail.store")` sin `useState` no re-renderiza.** El store es reactivo, pero el
   componente solo se suscribe si lo envuelve (`mail/static/src/chatter/web_portal/chatter.js:28`, `Chatter.setup`).
7. **El `<chatter/>` es una regla del form compiler, no un componente que se monte a mano**
   (`chatter/web/form_compiler.js:45`, `chatter_compiler`). Intentar colocarlo como widget de campo no funciona.
8. **`mail.store` depende de `bus_service`** (`mail/static/src/core/common/store_service.js:780`, `storeService`). En un entorno sin
   longpolling/websocket el chatter carga pero no se entera de nada nuevo, y eso no se distingue
   de "no hay mensajes".

9. **Una acción registrada que no se ve casi nunca es un fallo de registro**: es el `sequence`
   mandándola al desplegable. Ver arriba, con la medición.

## Qué de esto está probado ejecutándolo

El 2026-08-14 se montó un módulo de sonda contra un Odoo 18 limpio y se corrió con un tour en
Chrome headless que publica una nota en el chatter y busca la acción. **Probado en verde**: la
forma de `messageActionsRegistry.add` tal como está escrita arriba (con `sequence: 1`) · que
`patch(Thread.prototype, { setup() {...} })` con campos dentro de `setup()` funciona y **el campo
declarado se rellena** con lo que manda el servidor por `_thread_to_store` · y que una clave **no**
declarada llega igual, que es lo que corrigió el gotcha 5. **Salieron del banco** los gotchas 9 y
5. · que sustituir `MessageConfirmDialog` en `discussComponentRegistry` cambia el diálogo que abre
el **core** al borrar · y que sustituir la entrada `chatter_compiler` hace desaparecer el chatter,
que es lo que prueba de dónde sale. **Ya no queda nada sin probar en esta página.**

## Sobre v19: la estructura se mantiene, la API NO

Ojo con lo que decía esta sección antes: *«comprobado que la estructura se mantiene»*. Es cierto
—mismos ficheros, `mail.store` registrado igual (`mail/static/src/core/common/store_service.js:848`, `mail.store`)— y **engaña**,
porque el ORM de mail cambió de API entera. Se descubrió el 2026-08-15 al intentar correr el banco
de sondas contra v19: el primer error fue `TypeError: Record.attr is not a function`.

| v18 | v19 |
|---|---|
| `Record.one("Persona")` | `fields.One("res.partner")` |
| `Record.many("Attachment", {inverse})` | `fields.Many("ir.attachment", {inverse})` |
| `Record.attr("", {html: true})` | `fields.Html("")` |

`fields` se importa de `@mail/core/common/record` y se define en
`odoo19/addons/mail/static/src/model/misc.js:61`, `fields`; el uso real, en
`mail/static/src/core/common/message_model.js:38-46`, `Message.update`.

**Y los nombres de los modelos cliente también cambiaron**: el `persona_model.js` de v18 no existe
en v19, donde se usan directamente los nombres de Odoo (`res.partner`, `mail.guest`,
`ir.attachment`). Un patch escrito para v18 que declare `Record.one("Persona")` falla dos veces.

**`_thread_to_store` cambió de firma.** En v18 es
`(self, store, /, *, fields=None, request_list=None)`; en v19 no acepta esos posicionales — un
override copiado de v18 revienta con `takes 2 positional arguments but 3 were given`.

**Y las acciones de mensaje cambiaron de forma entera**, medido al portar el banco:

| v18 | v19 |
|---|---|
| `messageActionsRegistry.add(id, def)` | `registerMessageAction(id, def)` (`mail/static/src/core/common/message_actions.js:33`, `EmojiPickerMobile.emojiPickerProps`) |
| `condition: (component) => component.props.thread` | `condition: ({ message, thread, store }) => …` |
| `onClick: (component) => …` | `onSelected: ({ message, owner, thread, store }) => …` |
| `title: (c) => _t("…")` | `name: _t("…")` |

Los callbacks reciben un **objeto destructurado**, no el componente. Un `condition` copiado de v18
lanza `Cannot read properties of undefined (reading 'thread')` — y como un solo error de JS tumba
el bundle, **se lleva por delante media interfaz**, no solo tu acción.

**`_thread_to_store` también**: en v19 `fields` es posicional, y `store.add(..., as_thread=True)`
**reentra** en el método, así que un override necesita guarda de contexto o es recursión infinita.

**Lo que NO cambió**, medido con el banco portado (19 de 20 sondas en verde contra v19): que un
campo declarado en un `patch(Thread.prototype)` se rellena, que una clave **no** declarada llega
igual, y que `discussComponentRegistry` sustituye de verdad el componente que abre el core.

Las sondas portadas están en `tools/bench-odoo18/v19/`.
