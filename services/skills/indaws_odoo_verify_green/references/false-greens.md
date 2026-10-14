# Los 48 casos — 40 verdes falsos y 8 rojos falsos o inútiles, medidos, no imaginados

> Todos salieron de trabajo real en Odoo 18/19 y **todos se reprodujeron** antes de escribirse. Cada
> entrada trae el SÍNTOMA (lo que ves), la CAUSA (por qué miente) y CÓMO SE PILLA.
>
> El patrón común, y es lo único que hay que recordar si no lees el resto: **el modo de fallo no es
> un rojo, es un verde.** Un rojo lo mira todo el mundo.
>
> Los ocho que van al revés —un **rojo** que no es culpa de tu código, o que no sirve de nada—
> llevan `[ROJO FALSO]` o `[ROJO INÚTIL]` en el título: B1, B2, B7, C3, C6, D5, D8 y D9. Van aquí porque
> hacen el mismo daño por el otro lado: te comes una vuelta de diagnóstico buscando un defecto que
> no existe, o peor, «arreglas» código que estaba bien.

## A. El run no ejecutó lo que crees

### A1 — Sin línea `odoo.tests.result`, no ha corrido
**Síntoma:** exit code 0 y ninguna línea de resultado.
**Causa medida, dos veces distintas:** la variable de módulos salió **vacía** porque el comando
heredó el `cwd` (instaló cero módulos, exit 0); y el puerto de test estaba **ocupado** por el
proceso zombi del run anterior (`Address already in use`, exit 1 tragado por un `| tail`).
**Cómo se pilla:** `grep -c "odoo.tests.result"`. Y si el puerto sigue tomado, matar los `odoo-bin`
de test — **sin tocar** el proceso que sirve la instancia viva.

### A2 — Un módulo que acaba en `py` aporta CERO tests EN VERDE
**Causa:** bug del selector de tags de Odoo — `odoo/tests/tag_selector.py:15`, `TagsSelector` — el punto sin escapar
en el patrón de fichero (`(\/[\w\/\.-]+.py)?`), así que `.py` significa «cualquier carácter + py».
`--test-tags '/mi_modulo_zeroentropy'` se parsea como RUTA, no como módulo: no casa con nada y el
módulo desaparece de la receta.
**Cómo se pilla:** el módulo no aparece en `odoo.tests.stats`. **Regla: ningún módulo acaba en `py`.**

### A3 — Odoo sólo recoge los tests del `__dict__` de la clase
**Síntoma:** «0 tests» en verde para una batería que está escrita.
**Causa:** los tests heredados de un mixin no se recogen sin
`allow_inherited_tests_method = True`.
**Cómo se pilla:** contar los ejecutados (**recogidos − saltados**), no leer el exit code — una
batería entera saltada por falta de credenciales sale con exit 0. Todo runner propio debe imprimir
ese recuento y **fallar si es 0** bajo el tag pedido.

### A4 — Reiniciar el contenedor mata la suite en silencio y con exit 0
**Cómo se pilla:** mismo A1 — sin línea de resultado, no ha corrido.

### A5 — El tick verde de la PR era un linter, no una suite
**Síntoma:** la PR sale en verde y nadie ha ejecutado un test.
**Causa medida (PR real, 2026-08-12):** el workflow tenía **un solo job**, `Lint changed backend
files`, 19 s corriendo `pre-commit`. Cero tests. Un revisor lee el tick y da por hecho que el
módulo pasa; lo que pasa es el linter.
**Cómo se pilla:** listar los jobs antes de fiarte (`gh run view <id> --json jobs`) y buscar
`odoo.tests.stats` en su log. Sin esa línea, la PR **no tiene cobertura de CI**, aunque esté verde.
Se reporta como `[WARNING]`, no como puerta superada.
**Lo que sí sirve:** ejecutarlos tú. En ese PR, 33 tests y exit 0 — dato que la PR no daba.

### A6 — Instalar desde cero corre los tests de TODAS las dependencias
**Síntoma:** `-i mi_modulo --test-enable` lleva 25 minutos, alguien lo mata, y la conclusión es
«no hay forma de correr esto».
**Causa, en la fuente:** `odoo/modules/loading.py:280`, `load_module_graph` — la llamada al runner está **dentro del
bucle por módulo**, condicionada a que ese módulo tenga operación de instalación en el run. Al
instalar desde cero, `sale`, `stock`, `account`, `delivery`… entran todas y corren sus suites.
**Cómo se pilla:** en `odoo.tests.stats` aparece un módulo por línea; si hay más líneas que
módulos tuyos, estás pagando la suite de Odoo.
**El atajo:** instalar sin tests y luego `-u mi_modulo --test-enable --test-tags /mi_modulo`.
Medido en el mismo caso: **25 min → 2,03 s**. Un run que se acaba es la diferencia entre tener
recuento y no tener nada.

### A7 — En v19, un módulo con el manifiesto de otra versión no existe, y sale «0 tests»
**Síntoma:** levantas v19, corres los tests de tu módulo y el resumen dice
`0 failed, 0 error(s) of 0 tests`. Parece que pasaron; no corrió ninguno.
**Causa:** v19 estrena `check_version` (`odoo/modules/module.py:569-579`, `check_version`, v19): si `version` del
manifiesto no empieza por `19.0.`, `odoo/modules/module.py:465-467`, `_load_manifest`, pone `installable=False` y lo dice con un
**WARNING**, no con un error. El módulo desaparece para el instalador. **Ese chequeo no existe en
v17 ni v18**, así que un módulo que venía funcionando se vuelve invisible al saltar a v19.
**Cómo se pilla:** `grep -i "incompatible version"` en el log, y la regla de siempre — contar los
tests **ejecutados**, no leer el exit code. Medido el 2026-08-15.

### A8 — Ordenar los tests en subcarpetas los borra del mapa
**Síntoma:** reorganizas `tests/` en subcarpetas por tema (`tests/orm/`, `tests/security/`…), todo
importa bien, y la pasada dice `0 failed, 0 error(s) of 0 tests` con **exit code 0**.
**Causa:** el cargador solo mira **un nivel** y solo los módulos que **se llaman `test_*`**:
`inspect.getmembers(tests_mod, inspect.ismodule)` filtrado por `name.startswith('test_')`
(`odoo/tests/loader.py:57-66`, `_get_tests_modules`, v18; igual en v19). Un subpaquete llamado `orm` no pasa el filtro, y
uno llamado `test_orm` tampoco sirve: `get_module_test_cases` exige además que la clase tenga
`obj.__module__ == module.__name__` (`:19-26`), y las clases viven en el fichero de dentro, no en el
`__init__` del subpaquete.
**Qué sí funciona:** ficheros **planos** con el tema en el nombre (`test_orm_compute.py`) y una
**etiqueta** por clase (`@tagged(..., "orm")`) para poder correr solo una parte con
`--test-tags "orm"`. Los **modelos** sí pueden ir en subpaquetes: eso es Python normal, no lo toca
el cargador.
**Cómo se pilla:** la de siempre — contar los tests **ejecutados**. Aquí el aviso está incluso en el
log (`WARNING ... of 0 tests`) y aun así el proceso sale con 0. Medido el 2026-08-16 partiendo
`tools/bench-orm` por temas: 126 sondas desaparecidas de golpe.

### A9 - Si el modulo no esta en el `addons-path`, `-u` NO falla: 0 tests y exit 0
**Sintoma:** cambias algo, lanzas la suite, sale `0 failed, 0 error(s) of 0 tests` y **exit code 0**.
Nada te dice que el modulo no se ha encontrado.
**Causa medida (2026-08-16):** montar mal el volumen de docker -un `-v "$PWD/addons"` despues de un
`cd` a otro directorio- deja un `addons-path` vacio. Odoo lo dice, pero en el sitio que nadie mira y
sin cambiar el exit code: `WARNING ... module <x>: not installable, skipped` y
`ERROR ... Some modules have inconsistent states, some dependencies may be missing: ['<x>']`.
El proceso termina **con exito**.
**Como se pilla:** contar los tests ejecutados (la regla de siempre) y, en un runner propio,
**fallar si son 0**. Como red aparte: `grep "inconsistent states"` en el log.
**Nota:** paso dos veces en la misma sesion, las dos por un `cd` previo en el mismo bloque de shell.
Es mas facil de cometer que de detectar.

### A10 — La línea `odoo.tests.stats` NO cuenta tests: cuenta entradas, y `setUpClass` es una
**Síntoma:** el log dice `mi_modulo: 4 tests` y el resumen dice `of 2 tests`. Si te quedas con el
número grande —que es el que trae el nombre del módulo al lado, o sea el que se copia a un
informe— crees que hay el doble de cobertura de la que hay.
**Causa, en la fuente:** `log_stats` cuenta **claves del diccionario `stats`**, una por `test.id()`
(`odoo/tests/result.py:269`, `log_stats`). Y ese diccionario no solo recibe tests: la suite mete a
mano una entrada sintética `<módulo>.<Clase>.setUpClass` y otra `.tearDownClass` por cada clase de
test (`odoo/tests/suite.py:208`, `_handleClassSetUp`). Así que la línea **suma dos por clase**.
**Medido el 2026-09-05** en Odoo 19.0, módulo con una sola clase y dos tests:
`odoo.tests.stats: gate_probe: 4 tests` frente a `odoo.tests.result: ... of 2 tests`. Con
`--log-handler odoo.tests.stats:DEBUG` se ven las cuatro entradas por su nombre.
**Cómo se pilla:** el número de referencia es el del **resumen** (`odoo.tests.result`), y de ese hay
que restar los saltados. `odoo.tests.stats` vale para otra pregunta, que también importa: **¿aparece
mi módulo?** Si no aparece, no aportó nada, diga lo que diga el resumen. Las dos comprobaciones, y
la resta, las hace el gate ejecutable de la suite developer, el script `odoo_gate.py`.

### A11 — Una clase de test que hereda de `object` cuando falta el import NO se salta: desaparece
**Síntoma:** el módulo tiene tests, el log dice `0 failed, 0 error(s) of 0 tests` y el proceso sale
**0**. No hay ni un salto en el recuento: no es que los tests se salten, es que **no existen**. Y
como el resumen es verde, el módulo pasa por probado.

**Causa:** el patrón de import opcional, escrito así:

```python
try:
    from odoo.addons.otro_modulo.tests.test_x import TestX
except ImportError:
    TestX = None

class TestMiCaso(TestX if TestX else object):
    def test_algo(self): ...
```

Cuando el import falla, la clase **hereda de `object`** y deja de ser un `TestCase`, así que la
recolección la descarta: `odoo/tests/loader.py:24`, `get_module_test_cases` en 18.0 —y
`odoo/tests/loader.py:23`, `get_module_test_cases` en 19.0— hacen
`if not issubclass(obj, case.TestCase): continue`. En 17.0 el camino es otro y la regla la misma:
`odoo/tests/loader.py:73`, `make_suite` delega en `unittest.TestLoader().loadTestsFromModule`, que
filtra igual. El módulo aporta cero, Odoo no tiene nada que contar, `wasSuccessful()` es
`failures == errors == 0` y sale verde (`odoo/tests/result.py:159`, `wasSuccessful`).

La intención suele estar documentada en el propio fichero y **es razonable**: importar en cabecera
un módulo ausente rompería el descubrimiento global de tests, porque el loader importa el paquete
`tests` de todos los módulos. El error no es proteger el import, es **cómo**: un test saltado se ve
en el resumen; un test ausente no se ve en ningún sitio.

**Medido el 2026-09-06** en Odoo 18.0, módulo `indaws_mail_account`, cuyo único test hereda de una
clase de la OCA (`mail_composer_cc_bcc`) que no estaba en el `addons-path`. Salió
`0 failed, 0 error(s) of 0 tests`, exit 0. Al poner esa ruta en el `addons-path`, la misma pasada
recogió el test, lo ejecutó y **falló**:

```
AttributeError: 'account.move.send' object has no attribute 'id'
```

Es decir: el verde de cero tests llevaba tapando un test que no pasa. Y el módulo del que depende
**no está en `depends`** —es una dependencia opcional no declarada—, así que en cualquier entorno
sin ella el módulo viaja con una prueba que no se ejecuta jamás y nadie se entera.

**Cómo se pilla:** cuenta los tests ejecutados, no el exit code — es A1/ZERO, y `odoo_gate.py` lo
da RED. Para arreglarlo en el código, la forma correcta deja el salto **contado**:

```python
@unittest.skipUnless(TestX, "falta mail_composer_cc_bcc")
class TestMiCaso(TestX or TransactionCase):
    ...
```

Y para encontrarlos antes de que muerdan, se barre:
`grep -rn "if .* else object\|or object)" --include=*.py */tests/`.

## B. La base no era la que crees

### B1 `[ROJO FALSO]` — `-u` con `--test-enable` revienta en una base con `mail`
**Síntoma:** cualquier `res.users.create()` en un `setUpClass` muere con `null value in column
"notification_type"` (campo `required` con `compute` sin `precompute`). Los mismos tests pasan con
`-i` en base nueva.
**Regla:** verificar siempre con `-i` en base **nueva**. El `-u` vale para desplegar, no para juzgar.
**CONTRADICCIÓN ABIERTA: no se reprodujo en seis intentos (2026-09-05).** Módulo que depende de
`mail`, instalado sin tests en base limpia y luego `-u <módulo> --test-enable --test-tags /<módulo>`
sobre esa misma base. Dos formas del `setUpClass`, las dos en **17.0, 18.0 y 19.0**:

| `setUpClass` hace | 17.0 | 18.0 | 19.0 |
|---|---|---|---|
| `res.users.create({name, login})` | verde, exit 0 | verde, exit 0 | verde, exit 0 |
| ...y además `groups_id`/`group_ids`, que es lo que dispara el compute | verde, exit 0 | verde, exit 0 | verde, exit 0 |

El campo tiene exactamente la forma que describe la causa —`required=True, default='email',
compute=..., store=True`, **sin `precompute`**, e idéntico en las tres series:
`addons/mail/models/res_users.py:29`, `notification_type`. Y no hay auto-`precompute` para campos
`required` (`odoo/orm/fields.py:459`, `_get_attrs`). O sea: la forma está, el fallo no sale.

**Qué queda descartado**: que baste con crear un usuario, con o sin grupos, en cualquiera de las tres
versiones. **Qué NO queda descartado**: que el disparador esté en lo que el módulo bajo prueba le
hace a `res.users` —un `create` sobrescrito, el campo redefinido, el propio `mail` en la lista de
`-u`—, que es justo lo que un caso mínimo no tiene.

**No se borra**: quien lo midió tiene el caso real, y hace falta ese caso, no esta nota. La regla de
no juzgar con `-u` sigue en pie sin depender de esto: la sostienen B2 (un modelo nuevo del kernel no
tiene tabla donde el módulo ya estaba) y B6 (reutilizar base = `-i` no instala nada).

### B2 `[ROJO FALSO]` — Un modelo NUEVO del kernel no tiene tabla donde el módulo ya estaba
**Síntoma:** `Model X has no table`, y el error señala al módulo equivocado.
**Cómo se pilla:** `-u <modulo_kernel>` **sin** `--test-enable`.

### B3 — Un modelo de prueba declarado en `tests/` SE REGISTRA pero NO tiene tabla
**Causa:** con `-i modulo --test-enable` el loader importa `tests` mientras construye el registro,
así que el modelo entra en `env` y su `init_models` ya pasó sin él.
**Cómo se pilla:** guardar sobre la **TABLA** (`to_regclass`), no sobre el registro. Una guarda del
tipo `if MODEL not in registry` se salta exactamente el trabajo que falta, y todo muere con
«relation does not exist».

### B4 — Un test `at_install` ve el registro A MEDIAS
**Síntoma:** un barrido estructural pasa en verde con la fuga abierta.
**Causa:** los tests de un módulo corren justo tras instalar ESE módulo; los modelos de módulos
posteriores aún no existen.
**Cómo se pilla:** `@tagged("post_install")` para todo lo que razone sobre el registro entero — y
verificarlo en rojo, que es como se descubrió.

### B5 — Migras a v19 y la constraint de unicidad ya no existe; los tests siguen verdes
**Síntoma:** el módulo migra, la suite pasa, y en producción empiezan a aparecer duplicados que
antes eran imposibles.
**Causa, medida el 2026-08-15 con el mismo modelo en las dos versiones:** v19 dejó de soportar la
lista `_sql_constraints` (`odoo/orm/model_classes.py:162-163`, `add_to_registry`, v19). No es que el mensaje de
error cambie: **la constraint no se crea en Postgres**.

| | v18 | v19 |
|---|---|---|
| En `pg_constraint` | `['dyn_orm_uniq_dyn_uniq_code']` | **`[]`** |
| Segundo registro con el mismo código | `UniqueViolation` | **aceptado** |

Lo único que avisa es una línea al cargar el módulo: *"Model attribute '_sql_constraints' is no
longer supported, please define models.Constraint on the model"*.
**Por qué los tests no lo cazan:** la garantía la daba la base de datos, no el código, así que
ningún test de negocio la ejercita — nadie escribe un test que intente duplicar algo que "no se
puede duplicar".
**Cómo se pilla:** consultar `pg_constraint` para el modelo, o intentar el duplicado a propósito.
Al migrar a v19, las dos cosas, para cada `_sql_constraints` que traduzcas.

### B6 — Reutilizas una base de una pasada anterior y el `-i` no instala nada

**Síntoma:** el run sale verde y rápido, pero el módulo que querías probar no aparece en
`odoo.tests.stats` — o aparece con menos tests de los que escribiste.

**Causa:** `-i <módulo>` sobre una base donde el módulo YA está instalado no es un error: Odoo lo
salta. Los tests `at_install` de ese módulo no se ejecutan porque no hay instalación que los
dispare, y los `post_install` corren contra el estado que dejó la pasada anterior — incluido, si
venías de una verificación en rojo, el código neutralizado.

**Cómo se pilla:** contar los módulos en `stats` (§A) y comparar el número de tests con el que
esperabas. Un módulo que desaparece o adelgaza es la señal.

**Cómo se evita:** **borrar la base al leer su resultado**, no al final de la sesión. Una pasada,
una base, y se tira. Verificar en rojo y en verde son dos bases por arreglo, así que se acumulan
rápido: medido en una sesión real, **17 bases y 1,1 GB** sobre una única base útil, con el efecto
secundario de que quien entra al selector no sabe cuál mirar. Nombrarlas por lo que prueban
(`edi_red`, `edi_green`) ayuda a leer el desastre, no a evitarlo.

### B7 `[ROJO FALSO]` — La imagen no arranca Odoo: `base` no carga y no has medido nada
**Síntoma:** cuatro escenarios distintos seguidos, los cuatro «en rojo», y el diagnóstico natural es
que algo del módulo está mal. No hay ninguna línea de resultado, y el exit code es **255**, que no se
parece a un fallo de test.
**Causa medida (2026-09-05):** la imagen `indaws/odoo:18.0` del servidor lleva **pyOpenSSL 21.0.0 con
cryptography 49.0.0**, incompatibles. `addons/base/models/ir_mail_server.py` importa `OpenSSL`, y eso
muere con `AttributeError: module 'lib' has no attribute 'GEN_EMAIL'` **antes de tocar la base de
datos**: `CRITICAL ... Couldn't load module base`, en `odoo/modules/module.py:516`, `load_openerp_module`,
y `ERROR ... Failed to load server-wide module 'base'`, en `odoo/service/server.py:1523`, `load_server_wide_modules`.
La misma imagen con `indaws/odoo-slim:18.0` —pyOpenSSL 24.1.0 con
cryptography 42.0.8— corre los cuatro escenarios bien.
**Por qué duele:** es rojo antes del trabajo y rojo después, o sea que **no discrimina nada**, pero se
lee como un rojo normal. Un motor de iteración se gasta el presupuesto entero contra un contenedor.
**Cómo se pilla:** `grep -E "Couldn't load module (base|web)|Failed to load server-wide module"`. Si
aparece, es la máquina; no se toca el código. Lo hace el gate de la suite developer.

## C. El test no probaba lo que decía

### C1 — Un test cuyo resultado depende de qué módulos hermanos estén instalados
**Síntoma:** seis tests que pasan solos y fallan en compañía.
**Causa:** quitar una capacidad a un proveedor no prueba nada si hay otro proveedor sembrado por un
módulo hermano: la degradación que el test esperaba no ocurre.
**Cómo se pilla:** si el test necesita la **ausencia** de algo, créala globalmente (recorriendo todos
los registros), no en uno.

### C2 — `assertEqual(recordset.campo, 0.0)` sobre un recordset VACÍO pasa
**Causa:** `False == 0.0` es cierto en Python. Exige primero que la fila exista.

### C3 `[ROJO FALSO]` — `assertRaises` de Odoo abre un savepoint y lo REVIERTE
**Síntoma:** compruebas que una operación fallida deja traza y el test dice `0 != 1` mientras el
producto funciona.
**Cómo se pilla:** `try/except` a mano.

### C4 — Dos ramas del mismo largo no prueban un AND
**Caso real:** un test de join AND pasaba **con y sin** el AND, porque con las dos ramas iguales la
cola llegaba al join después de ambas de todas formas. Sólo discrimina con largos distintos.
**Cómo se pilla:** verificar en rojo. Es literalmente lo único que lo caza.

### C6 `[ROJO INÚTIL]` — El test dice qué borrar, y borrarlo no lo pone en verde
**Síntoma:** un test centinela falla con un mensaje que prescribe un arreglo («esto sobra, bórralo»).
Se hace lo que dice y **sigue rojo**.
**Causa medida (PR real, 2026-08-12):** el centinela usaba un método de envío «sin clasificar» como
control neutro, dando por hecho que no pasaba por el codigo del modulo. Pero el modulo sobreescribia
`_compute_amount_total_without_delivery` en **`sale.order`**, no en el carrier: aplicaba a TODOS los
pedidos. El control no era neutro, asi que el numero medido no venia de donde el test creia y borrar
lo que el mensaje señalaba no cambiaba nada.
**Cómo se pilla:** antes de escribir un centinela, preguntar **en qué modelo vive cada override del
módulo**. Un filtro en el método hijo (`if not self.x_tipo`) no aisla un override puesto en el padre.
**La regla que queda:** un mensaje de fallo que prescribe un arreglo **se prueba aplicándolo**. Si
tras el arreglo el test sigue rojo, el mensaje miente y manda a la gente en la dirección equivocada.

### C5 — Una puerta de CI que no se ha visto en rojo no está verificada
**Caso real:** una puerta de `sudo` daba verde con 19 de 74 elevaciones sin justificar, porque
aceptaba *cualquier* comentario cercano en vez de uno **sobre la elevación**.

### C7 — El bloqueo que no bloquea: el motor se traga tu `UserError`
**Síntoma:** una validación nueva («no dejes exportar/confirmar si falta X») con sus tests en verde,
y en la pantalla **no pasa nada**: el usuario pulsa el botón, no ve ningún aviso, y el registro de
negocio se queda como si todo hubiera ido bien.
**Causa medida (PR real, 2026-08-13):** el `raise UserError` vivía dentro de un handler que un motor
propio invoca envuelto en `except Exception` para **aislar el fallo por documento** — patrón normal
en colas, crons y motores de integración: no relanza, persiste `state='error'` + `error_message` en
un modelo técnico y sigue con el resto del lote. El aviso existe, pero donde nadie mira. Agravante:
el registro de negocio ya se había marcado como «enviado/pendiente» y el hook que lo corregía
(`_on_send_error`) **no lo llamaba ningún código de producción, solo los tests**.
**Por qué el verde es doble:** el test afirma `assertEqual(doc.state, 'error')` — es decir, afirma
**el estado que deja el `except`**, no que el usuario reciba el error. Código y test son coherentes
entre sí; lo que no existe es la funcionalidad anunciada.
**Cómo se pilla:** ante cualquier «bloquea / avisa / no deja», seguir la excepción hasta la capa del
usuario: `grep -n "except Exception" ` en el motor que invoca el handler. Si alguien la captura y no
relanza, **no hay aviso** — hay una anotación. La validación que debe cortar al usuario va donde el
`UserError` sale (el wizard/botón, antes de crear el registro técnico), y la del motor se queda como
red de seguridad para el cron.

### C8 — `assertTrue(rec.error_message)` no distingue tu caso de cualquier otro fallo
**Síntoma:** el test del bloqueo pasa… y también pasaría si el proceso hubiera muerto por un layout
que no existe, un registro sin contabilizar o un `AttributeError`.
**Causa:** afirmar la **verdad** del campo de error, no su **contenido**. Cualquier excepción por el
camino da el mismo verde.
**Cómo se pilla:** `assertIn("<texto del mensaje>", rec.error_message)`, o el identificador del
registro que motivó el bloqueo. Y verificarlo en rojo rompiendo **otra** cosa: si el test sigue
verde, no estaba probando lo suyo.

### C9 — Un test que escribe y confirma en el mismo `Form()` puede estar perdiendo datos y pasar
**Síntoma:** el test rellena campos, confirma el documento, comprueba el estado final y sale
verde. En producción los campos aparecen vacíos.
**Causa, medida en v18 el 2026-08-15 con un tour:** un `readonly` condicional del tipo
`readonly="state != 'draft'"` se evalúa **en el momento de guardar**
(`odoo18/addons/web/static/src/model/relational_model/record.js:635`, `Record._getChanges`), no cuando se escribió. Si
la confirmación cambia `state` antes del guardado, lo escrito **no viaja** y se pierde sin error.
El test suele pasar porque afirma sobre el estado o sobre lo que confirmó, no sobre los campos que
rellenó.
**Cómo se pilla:** afirmar los campos rellenados **después de recargar el registro**
(`invalidate_recordset()` / releer), no sobre el objeto que se manipuló. Y si el flujo real es
"rellenar y confirmar", probarlo en esa secuencia, no rellenando y guardando por separado.
**Ojo con el espejo:** un campo `invisible="1"` sí se escribe y sí se guarda. La protección que
mucha gente supone —ocultar para que no se toque— no existe.

### C10 — Un test de tracking en negativo pasa siempre
**Síntoma:** un test que afirma «este cambio NO debe dejar mensaje en el chatter» está en verde
desde el primer día.
**Causa medida (v18 y v19):** el tracking de `mail.thread` **no se escribe en el `write`**: se encola
en `env.cr.precommit` y solo se materializa al hacer **commit**. Un `TransactionCase` no commitea,
así que `message_ids` está intacto justo después del write **haga lo que haga el código**. El test
en negativo no puede fallar; el test en positivo da un rojo falso y se acusa al código de negocio.
**Cómo se pilla:** `self.env.cr.precommit.run()` antes de la aserción, y comprobar que el caso
POSITIVO equivalente pasa a verde con eso. Si el negativo sigue verde con el precommit corrido,
entonces sí mide algo.

## D. El navegador: pasos que pasan sin hacer nada

### D1 — La suite NO ejecuta los tours
Llevan `-standard`. Una suite de 1.800 tests puede estar en verde con todos los tours rotos; medido
también al revés: **1.379 tests en verde y tres tours rotos** por un cambio en un defecto del JS.
**Regla:** tocar `static/src/` obliga a correr los tours aparte.
**Cómo se pilla:** correrlos aparte, con un runner que **falle en el verde falso** (si no ejecutó
ningún tour, exit ≠ 0). Y comprueba que ese runner **existe**: una documentación que cita uno
inexistente se lee como cobertura que alguien está ejecutando.

### D2 — Un `<option>` NO se puede afirmar desde un tour
**Causa:** un paso sólo casa elementos **visibles**, y las opciones de un `<select>` cerrado no lo
son. `option[value='x']` **no puede pasar jamás**, por correctos que sean los datos.
**Cómo se pilla:** afirmar sobre el `<select>`, y exponer lo que necesites en un atributo `data-`.

### D3 — `run: "select X"` casa por VALOR, no por etiqueta
Si no existe ese valor, **no cambia nada y el paso pasa**. Para etiquetas, `selectByLabel`.

### D4 — Un `HttpCase` falla ante cualquier error JS no capturado aunque todos los pasos pasen
En el log sale `TOUR ... SUCCEEDED` y el test rojo a la vez. Es la señal, no una contradicción.

### D5 `[ROJO FALSO]` — Una pestaña en segundo plano no renderiza Owl
`document.visibilityState: hidden` → la barra pinta, `document.body` tiene hijos, la acción no monta
y la captura agota el timeout. **Compruébalo antes de diagnosticar una pantalla en blanco.**

### D6 — Los tests de servidor no montan el componente
124 tests de un editor visual, y ninguno lo instanciaba: llegó al navegador con un `props` inválido
que tumbaba el formulario entero.

### D7 — Un tour que NO PUDO correr se cuenta como «0 failed»
El entorno le falta algo, `HttpCase` lanza `SkipTest`, y la última línea del run dice
`0 failed, 0 error(s) of 1 tests`. **Un tour saltado y un tour que pasa se leen igual en el
resumen.** Medido el 2026-08-14, dos veces seguidas sobre el mismo módulo:

- **Sin `websocket-client`** → `raise unittest.SkipTest("websocket-client module is not installed")`
  (`odoo/tests/common.py:1250-1251`, `ChromeBrowser.__init__`). El aviso sale como `WARNING`, dos líneas por encima del
  resumen en verde.
- **Con un Chrome que no arranca** → `Failed to detect chrome devtools port after 10.0s`
  (`:1438`). Ojo con la trampa concreta: en una imagen **Ubuntu**, `apt install chromium` instala
  un paquete de transición a **snap** que en contenedor no ejecuta nada. El binario
  `chromium-browser` existe, `_find_executable` lo encuentra (`:2097`) y falla al lanzarlo. Hay
  seis `SkipTest` más en ese fichero, todos del mismo camino (`:1503`, `:1568`, `:1588`, `:1593`).

**Cómo se pilla:** buscar `skipped` en el log, no leer solo la última línea. En CI, fallar si el
número de tests **ejecutados** es 0. Y montar el runner con `google-chrome` de verdad —el `.deb`
de Google— o apuntar `ODOO_BROWSER_BIN` a un binario que se haya visto arrancar (`:2092`).

**Por qué duele más que otros verdes falsos:** el tour es lo único que ejecuta el JS. Si se salta,
no hay nada más en la suite que lo cubra, y el módulo pasa a producción con el frontend sin probar
y un informe que dice que todo está bien.

### D8 `[ROJO FALSO]` — Un módulo roto tumba el bundle, y eso se lee como «cambió la versión»
**Síntoma:** portas un módulo o una batería a una versión nueva de Odoo y fallan cosas que no has
tocado. La conclusión inmediata —y equivocada— es que esa versión cambió el comportamiento.
**Causa:** un solo error de JS al cargar tumba **el bundle entero**, así que una pieza mal portada
arrastra a todo lo demás. `startModule` marca el módulo como fallido y relanza
(`odoo18/addons/web/static/src/module_loader.js:227-229`, `ModuleLoader.startModule`); lo que salga por consola es *"The
following modules failed to load"*, y lo que ves en los tests es una lista de rojos sin relación
aparente entre sí.
**Medido dos veces el mismo día (2026-08-15), las dos con conclusión falsa antes de comprobar:**

- Un banco de sondas de frontend dio *17 de 20* en v19 y de ahí se escribió que
  `discussComponentRegistry` «ya no sustituye componentes en v19». Falso: una acción de mensaje
  portada a medias lanzaba `Cannot read properties of undefined` y se llevaba por delante otras
  dos sondas. Bien portada: **19 de 20**, y la sustitución funciona igual que en v18.
- Antes, el mismo banco «demostró» que en v19 el guardado no emite RPC. También falso: dos sondas
  compartían el campo del que colgaban sus condicionales, así que una activaba la de la otra y
  bloqueaba el guardado. **En v18 daba exactamente lo mismo**, que es como se descubrió.

**Cómo se pilla, en este orden:**

1. **Corre lo mismo en la versión vieja.** Si también falla, no es la versión: es tu instrumento.
   Es la comprobación más barata y la que resuelve la mayoría.
2. **Bisecciona.** Desactiva las piezas portadas una a una hasta que el rojo desaparezca. Un rojo
   que se arregla desactivando algo que no tiene relación con él es la firma de este caso.
3. **Busca el error de carga**, no el del test: `grep -i "failed to load"` en el log del navegador
   dice qué módulo cayó de verdad.

**La regla que queda:** una diferencia de versión no se anuncia desde un rojo. Se anuncia cuando
la misma sonda, aislada, da resultados distintos en las dos versiones.

### D9 `[ROJO FALSO]` — Un paso de tour que timea porque el SERVIDOR devolvió un 500

**Síntoma:** un paso del tour falla con `TIMEOUT step failed to complete within 10000 ms`, y el
trigger que no aparece es uno que existe seguro (`.o_form_view`, `.o_form_sheet`). Cambiar el
selector no arregla nada: **ningún** paso posterior se completa.

**Causa, medida el 2026-08-15:** una petición RPC devolvió **500** justo antes. A partir de ahí la
página queda inservible y el runner espera un DOM que ya no va a cambiar. El error real estaba en el
log del run, dos líneas por debajo:

```
ERROR odoo.http: Exception during request handling.
TypeError: BaseModel.name_search() got an unexpected keyword argument 'args'
```

(Un override de sonda con la firma de v18: `name_search` cambió su segundo parámetro `args` →
`domain` en v19 — `odoo/models.py:1872`, `BaseModel.name_search`, v18 vs `odoo/orm/models.py:1514-1520`, `BaseModel.name_search`, v19.)

**Por qué engaña tanto:** el síntoma apunta al DOM, así que se investiga el selector, el z-index, el
overlay... y la explicación que se escribe es plausible y falsa. En este caso llegó a quedarse
escrita un día entero como «el desplegable tapa el botón de descartar en v19».

**Cómo se pilla:** ante un timeout de tour, `grep -iE "Exception during request handling|Traceback"`
en el log **antes** de tocar el selector. Y la regla hermana de D8: si el paso que falla es de los
que no pueden fallar, no es el paso.

## E. Lo verificaste en el sitio equivocado

### E1 — `odoo-bin shell` NO verifica el proceso vivo
**Causa:** monta su **propio registro fresco**. Un `-u` con `--stop-after-init` corre en otro proceso
y muere; el servidor que sirve la web se queda con el registro viejo y sirve el `arch` nuevo con los
campos antiguos → `field is undefined` con la base perfecta.
**Cómo se pilla:** `docker restart` tras el `-u`, y comprobar **por HTTP** contra el proceso vivo.

### E2 — `odoo-bin shell` NO committea solo
Una verificación por shell que escribe necesita `env.cr.commit()` explícito o la evidencia se evapora
al salir. (En un TEST es al revés: el commit está **prohibido** en v19.)

### E3 — `search_count` esconde los `active=False`
Una medida «antes/después» sale idéntica y el «+0» es mentira. `.with_context(active_test=False)`.

## F. El ORM se saltó tu código

### F1 — `ondelete="cascade"` es una constraint de POSTGRES: no ejecuta el `unlink()` del hijo
**Medido:** 72 crones → 73 al crear un hijo que posee uno → **73** tras borrar el padre, con el hijo
ya desaparecido y su cron vivo. Todo lo que el `unlink()` del hijo fuera a hacer, no ocurre.
**Cómo se pilla:** un test que borra el **padre**, no el hijo. El que borraba el hijo pasaba.

### F2 — Un atributo de clase en un `_inherit` SUSTITUYE al del padre, no lo fusiona
**Síntoma:** un módulo hermano añade tipos y su mapa de iconos «no aparece»; el arreglo obvio
(redeclarar el atributo) borraría los del padre.
**Cómo se pilla:** todo lo que un módulo hermano deba **ampliar** va como MÉTODO con `super()`.

### F3 — Odoo REPONE `ir.cron.active` después de cada trabajo
Con el valor leído **al adquirir** el trabajo, así que lo que un job escriba sobre su propio `active`
se pisa un instante después. **El estado es la verdad, el flag es fontanería.** Y no lo ve ningún
test que corra con `--max-cron-threads=0`, que suele ser toda la suite.

### F4 — Un worker de larga vida no ve un `ir.config_parameter` cambiado
Dos minutos y tres trabajos con el valor viejo; sólo al **reiniciar** el proceso. Importa cuando el
parámetro es un control de seguridad.

### F5 — Renombras un campo y la validación que lo vigilaba deja de existir, en silencio
**Síntoma:** el módulo instala, la suite pasa entera, y empiezan a entrar registros que antes eran
imposibles.

**Causa, medida el 2026-08-15 en v18 y v19** (`tools/bench-orm`, `probe-broken-refs.sh` +
`test_broken_refs.py`): un `@api.constrains("campo_que_ya_no_existe")` **no da error al instalar y
no se ejecuta nunca**. El registro que la constraint prohibía se crea sin una queja. Lo mismo con
`@api.onchange`: deja de dispararse para siempre.

Es lo contrario de lo que hacen sus vecinos, y por eso engaña: `@api.depends` con un campo
inexistente **revienta al instalar** (`ValueError: Wrong @depends`), y un `related` también
(`KeyError`). Uno espera que el ORM valide los tres decoradores igual. Valida dos.

**Por qué ningún test lo caza:** nadie escribe un test que compruebe que una validación *sigue*
saltando. Los tests de negocio prueban el camino bueno; la constraint sólo aparecía en el malo.

**Cómo se pilla:** al renombrar o borrar un campo, `grep -rn "api.constrains\|api.onchange"` y
mirar las listas de campos a mano — no basta con que el módulo arranque. Y si la constraint importa,
un test que intente crear el registro prohibido y espere el `ValidationError`.

## G. El verde heredado

### G1 — Una deuda declarada no es una deuda medida
**Caso real:** una deuda llevaba días descrita como benigna; al medirla, un `forbid` se degradaba a
`require` — o sea que un humano podía aprobar lo prohibido.

### G2 — Un «verificado» heredado de un documento no vale
Se reejecuta en la misma sesión que lo declara. En una sola sesión aparecieron **tres afirmaciones
falsas** escritas por su propio autor: un recuento de flujos, una arista que «funcionaba» y no
transportaba nada, y una limitación documentada que resultó **más pesimista** que el código.

### G3 — Un docstring del propio Odoo describiendo lo que ya no hace
**Síntoma:** citas la fuente, con fichero y línea, y la afirmación es falsa igualmente.

**Caso real, medido el 2026-08-15:** el docstring de `TransientModel`
(`odoo/models.py:7475-7482`, `TransientModel`, v18) dice que *todos* los usuarios pueden crear registros y que cada
uno *solo ve los que creó*. **Las dos mitades son falsas** en v18 y v19: sin `ir.model.access` un
usuario normal no puede crear, y con él ve perfectamente los registros de otro. No queda código que
lo implemente — la capa de permisos no distingue transient de normal. El mismo docstring dice que la
limpieza «ocurre como mucho cada 5 minutos» y tampoco: los 5 minutos son la **edad mínima de la
fila**, no un intervalo entre ejecuciones.

**Por qué engaña más que un error normal:** una cita `fichero:línea` es la señal que usamos para
distinguir lo verificado de lo recordado, y aquí apunta a un texto **que nadie ejecuta**. Un
docstring no tiene tests; puede llevar años describiendo un comportamiento retirado.

**Cómo se pilla:** citar el docstring vale para *el propósito* de la clase; para *el comportamiento*,
cita el código que lo hace — y si no encuentras ese código, esa es la respuesta. Dos greps y una
sonda de tres líneas separan una cosa de la otra.
