# Catalogo de familias de errores de Odoo (runtime)

Cada familia: **firma** (como se reconoce en el traceback/log), **fase**, **sintoma tipico** y **causa raiz mas probable**. Soporta v17-v19; las diferencias por version se anotan donde aplican. Para el fix concreto ver `fix-patterns.md`; para leer el log ver `log-reading.md`.

Regla transversal: en el traceback, el **ultimo frame que pertenece al codigo del proyecto** (no a `odoo/` ni a librerias) es el punto de partida, pero en errores de carga/`-u` la causa real suele estar en un fichero de datos (`.xml`/`.csv`/manifest), no en ese frame Python.

---

## Familia A — Instalacion / actualizacion (`-u`)

### A1. ParseError (XML)
- **Firma:** `lxml.etree.XMLSyntaxError` o `odoo.tools.convert.ParseError: while parsing <file>.xml:<line>`.
- **Fase:** carga de `data`/`demo`.
- **Sintoma:** el modulo no instala/actualiza; el log nombra el fichero y la linea.
- **Causa raiz:** XML malformado (etiqueta sin cerrar, `&` sin escapar), o `ref="..."` a un external id inexistente, o atributo/expresion invalida en el `arch`.

### A2. Validacion de vista (View validation)
- **Firma:** `Error while validating view`, `Element '<field name="x">' cannot be located in parent view`, `Invalid view ...`, o en v17+ `ValidationError` con `Field "x" does not exist in model "y"` al validar el arch.
- **Fase:** carga/validacion de `ir.ui.view`.
- **Sintoma:** falla al crear/heredar la vista.
- **Causa raiz:** xpath que no casa (la vista padre cambio entre versiones), campo en el arch que no existe en el modelo, o atributo de vista retirado. `attrs`/`states` eliminados **en 17.0 y tambien prohibidos en v18/v19** (usar `invisible="..."`, `readonly="..."` directos) — ver E1 para la cita del `ValidationError`. v18: `<tree>` pasa a `<list>` y `<tree>` deja de existir (`odoo/addons/base/models/ir_ui_view.py:163`, `View.arch` v17 con `('tree','Tree')`, `:153` v18 y `:149` v19 con `('list','List')`). v19: mas atributos depurados -> delega en `odoo-version-api-reference`.

### A3. "Field X does not exist"
- **Firma:** `KeyError: 'x'` o `Field x does not exist` al cargar vista/registro/`_sql_constraints`.
- **Fase:** `-u`/instalacion.
- **Causa raiz (3 ramas, descartar en orden):** (1) typo o campo borrado; (2) el modulo que define el campo no esta en `depends`; (3) orden de carga: la vista/dato se carga antes que el modelo (orden de `data` en manifest o `depends` insuficiente).

### A4. Dependencia faltante en manifest
- **Firma:** `KeyError`/`AttributeError` sobre un modelo/campo/external id de otro modulo. **Los
  mensajes exactos del core, por si los grepeas en el log** (verificados v17/v18/v19):
  - **instalando/actualizando desde la UI** → si, lanza: `UserError` con *"You try to upgrade the
    module X that depends on the module: Y.\nBut this module is not available in your system."*
    (`addons/base/models/ir_module.py:711`, `Module.button_upgrade` v17, `:735` v18, `:745` v19; variante de instalacion en
    v17 `:384`).
  - **en la carga del registry** → NO lanza, **poda y sigue**: v17/v18
    `_logger.info('module %s: Unmet dependencies: %s')` (`odoo/modules/graph.py:104`, `Graph.add_modules`); v19
    `_logger.warning('module %s: some depends are not loaded (%s), skipped')`
    (`odoo/modules/module_graph.py:266`, `ModuleGraph._update_depends`). Al final del arranque, un unico
    `_logger.error("Some modules are not loaded, some dependencies or manifest may be missing: %s")`
    (`odoo/modules/loading.py:514`, `def load_modules` v17, `:518` v18, `:495` v19).
- **Causa raiz:** el modulo usa algo de otro modulo sin declararlo en `depends` del `__manifest__.py`.
- **Ojo con el modo de fallo:** por la via de carga **el servidor arranca igual**, con el modulo
  fuera. Si buscas un traceback y no lo encuentras, busca esos WARNING: el sintoma es
  "mi modulo no esta" o un `KeyError` posterior, no una excepcion en el arranque. Y **no falla
  siempre**: el orden lo decide el `depends`, asi que un acoplamiento no declarado puede funcionar
  en una base y romper en otra segun que mas haya instalado.

### A4b. "Mi modulo no aparece" y no hay traceback — las TRES podas del grafo

El grafo de modulos **nunca lanza**: descarta y deja una linea de log. Son tres causas distintas y
en v19 tienen tres mensajes distintos, asi que el log te dice cual es **si sabes que grepear**:

| Causa | v19 (`odoo/modules/module_graph.py`) | v17/v18 (`odoo/modules/graph.py`) |
|---|---|---|
| `'installable': False` en el manifest | `WARNING module %s: not installable, skipped` (`:242`) | mismo mensaje, `WARNING` (`:75`) |
| dependencia declarada que no esta | `WARNING module %s: some depends are not loaded (%s), skipped` (`:266`) | `INFO module %s: Unmet dependencies: %s` (`:104`) |
| **ciclo de dependencias** | `WARNING module %s: in a dependency loop, skipped` (`:277`) — `RecursionError` cazado en `_update_depth` | **no lo distingue**: cae en el mismo `INFO` de arriba |

Dos consecuencias practicas:

1. **En v17/v18 el mensaje es `INFO`, no `WARNING`** — se pierde entre el ruido del arranque, y un
   **ciclo** y una **dependencia ausente** producen exactamente la misma linea. Si sospechas ciclo
   en v17/v18, no lo vas a leer en el log: revisa los `depends` a mano.
2. Un `'installable': False` olvidado (tipico al copiar un modulo plantilla) se ve igual desde
   fuera que una dependencia rota: el modulo simplemente no esta.

Al final del arranque, el resumen unico de todo lo podado:
`_logger.error("Some modules are not loaded, some dependencies or manifest may be missing: %s")`
(`odoo/modules/loading.py:514`, `def load_modules` v17, `:518` v18, `:495` v19). **Ese `ERROR` es el unico sitio donde salen todos
juntos**, y es lo primero que hay que mirar cuando falta un modulo.

### A5. KeyError/AttributeError en carga del registry
- **Firma:** traceback durante `loading modules` / `Loading module X`, con `KeyError`/`AttributeError` en `_setup_*`, `_build_model`, `_inherit`.
- **Causa raiz:** `_inherit` a un modelo no cargado aun, `_name` duplicado, o referencia a un campo/metodo de un modulo que carga despues. Loading order: revisar `depends` y orden de `data`.

---

## Familia B — Base de datos (SQL / psycopg2)

### B1. IntegrityError
- **Firma:** `psycopg2.errors.UniqueViolation` / `ForeignKeyViolation` / `NotNullViolation`, `IntegrityError: ... violates ... constraint "<nombre>"`.
- **Fase:** runtime (commit) o `-u` (al crear constraint sobre datos sucios).
- **Sintoma:** la operacion revierte; el mensaje da el **nombre del constraint**, no el registro.
- **Causa raiz:** dato que viola unique/FK/not-null. Puede ser dato sucio preexistente (no codigo) o un `_sql_constraints` recien anadido que choca con datos existentes.

### B2. UndefinedColumn / UndefinedTable
- **Firma:** `psycopg2.errors.UndefinedColumn: column x.y does not exist`.
- **Fase:** arranque/runtime tras anadir un campo.
- **Causa raiz:** columna nueva aun no creada en BD porque falta el `-u` del modulo, o se anadio `store=True`/un campo computado almacenado sin actualizar. Casi nunca es bug de Python.

### B3. SerializationFailure / Deadlock
- **Firma:** `psycopg2.errors.SerializationFailure`, `could not serialize access`, `deadlock detected`.
- **Causa raiz:** concurrencia (crons solapados, escrituras simultaneas al mismo registro). El fix es de diseno (locking/orden), no del registro.

### B4. DataError
- **Firma:** `psycopg2.errors.NumericValueOutOfRange`, `StringDataRightTruncation`, `InvalidTextRepresentation`.
- **Causa raiz:** valor que no cabe en el tipo de columna (size, precision) o conversion de tipo invalida.

---

## Familia C — Excepciones de negocio (ORM)

### C1. AccessError
- **Firma:** `odoo.exceptions.AccessError: You are not allowed to ...` / `... not allowed to access 'model' (...)`.
- **Causa raiz:** ACL (`ir.model.access.csv`) o record rule (`ir.rule`) que deniega. NO es bug de logica. Mirar grupos del usuario, ACLs del modelo y record rules activas. Tambien por `groups=` en campos.

### C2. ValidationError
- **Firma:** `odoo.exceptions.ValidationError`.
- **Causa raiz:** un `@api.constrains` o un `_sql_constraints` (validacion intencional) que el dato no cumple. El mensaje suele venir del propio constraint.

### C3. UserError
- **Firma:** `odoo.exceptions.UserError`.
- **Causa raiz:** `raise` intencional del codigo (regla de negocio: "no puedes confirmar sin lineas", etc.). El "bug" puede ser el flujo del usuario, no el codigo.

### C4. MissingError
- **Firma:** `odoo.exceptions.MissingError: Record does not exist or has been deleted`.
- **Causa raiz:** se opera sobre un recordset cuyo registro fue borrado (cache obsoleta, `unlink` previo, id pasado por contexto que ya no existe).

### Jerarquia y codigo HTTP — lo que cambia en v19

**La jerarquia es estable** en v17/v18/v19: `AccessDenied`, `AccessError`, `MissingError` y
`ValidationError` heredan todas de `UserError` (`odoo/exceptions.py` — v17 `:18,55,74,95,104`;
v18 `:16,45,64,85,94`; v19 `:9,39,77,99,119`). Y el modulo dice por que importa:
*"Those types are understood by the RPC layer. Any other exception type bubbling until the RPC
layer will be treated as a 'Server error'"* (`:1-5`). Traducido a diagnostico: **un `Server error`
500 generico suele significar que alguien lanzo una excepcion que no es de esta familia.**

**v19 anade el codigo HTTP a la propia clase de excepcion** — no existe en v17 ni en v18 (0
ocurrencias de `http_status` en su `exceptions.py`):

| Excepcion | `http_status` en v19 | Linea |
|---|---|---|
| `UserError` | 422 Unprocessable Entity | `:15` |
| `AccessDenied` | 403 Forbidden | `:50` |
| `AccessError` | 403 Forbidden | `:84` |
| `MissingError` | 404 Not Found | `:106` |
| `LockError` (**nueva en v19**) | 409 Conflict | `:109,116` |
| `ConcurrencyError` (**nueva en v19**, no hereda de `UserError`) | — | `:128` |

Lo consume el despacho HTTP: `werkzeug_default_exceptions[exc.http_status](exc.args[0])` con
fallback a `UnprocessableEntity` (`odoo/http.py:2534-2538`, `HttpDispatcher.handle_error`, v19 — el mecanismo no existe en v18). **Consecuencia para depurar en v19:**
el codigo HTTP que ve el navegador lo decide la clase de la excepcion, asi que un 403/404/409 ya
te dice la familia antes de abrir el log — y `ConcurrencyError`, al no heredar de `UserError`,
**sale como 500**.

---

## Familia D — Cliente (assets / OWL)

### D1. Error de assets / build
- **Firma:** en el **navegador** (consola), no en el log del servidor: `Failed to load resource`, `Cannot read properties of undefined`, errores de bundle.
- **Causa raiz:** asset no declarado en `__manifest__.py` (`assets`), orden de bundles, o SCSS/JS roto. Assets en `__manifest__['assets']` por bundle **desde v15**, verificado contando manifests del core que traen la clave: v13 **0**, v14 **0**, v15 **176**, v16 **206**. Si el modulo viene de v14 o anterior, los assets estaban en plantillas XML heredando `assets_backend`, no en el manifest.

### D2. Error OWL
- **Firma:** consola del navegador: `OwlError`, `An error occured in the owl lifecycle`, template QWeb-JS no encontrado, `t-component` invalido.
- **Causa raiz:** template OWL mal referenciado, props faltantes, o API de componente cambiada entre versiones (v17->v18->v19 mueven utilidades del framework). Pedir SIEMPRE la traza de la consola del cliente; el log del servidor estara limpio.

---

## Familia E — Migracion (tras subir version)

### E1. `-u` falla por API retirada
- **Firma:** tras subir el codigo a una version nueva, el `-u` lanza `AttributeError`/`ParseError`/validacion de vista sobre algo que antes funcionaba.
- **Causa raiz:** API/atributo retirado entre versiones. NO es bug nuevo. Los dos que mas salen,
  con su cita para que no haya que discutirlos:
  - `name_get` **retirado en v18** — en v17 aun existe (`odoo/models.py:1677`, `BaseModel.default_get`) y avisa en ejecucion,
    `warnings.warn("Since 17.0, deprecated method, read display_name instead", DeprecationWarning)`
    (`:1693`); en v18 `def name_get` ya no esta en `models.py`. Sustituto: `display_name` /
    `_compute_display_name`.
  - `attrs=` / `states=` **eliminados en 17.0** y siguen prohibidos en v18 y v19: no se ignoran,
    **impiden cargar el arch** con `ValidationError('Since 17.0, the "attrs" and "states"
    attributes are no longer used…')` (`odoo/addons/base/models/ir_ui_view.py:393`, `View._check_xml` v17, `:387` v18, `:495` v19).
  Tambien `@api.one` y metodos renombrados. Delega en `odoo-version-api-reference` para el diff
  origen->destino.

### E2. Scripts de migracion (pre/post/end)
- **Firma:** traceback dentro de `migrations/<version>/pre-*.py` o `post-*.py`.
- **Causa raiz:** el script asume un esquema/dato que ya no esta, o se ejecuta en la fase equivocada (pre vs post: pre corre antes de cargar el modelo nuevo). Revisar fase y guardas (`if column_exists(...)`).
