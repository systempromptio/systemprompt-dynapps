# Catálogo de caminos del vertical inbuilt — 83 caminos

Un camino es lo que un usuario puede recorrer con el ratón. Cada uno tiene
**entrada** (dónde se pincha), **qué debe pasar** y **canal**.

**El canal decide quién lo ejecuta** (pasos 3 y 4 de la SKILL):

| Canal | Qué se comprueba | Quién lo ejecuta |
|---|---|---|
| `visual` | **Por defecto.** Todo lo que un usuario vería en pantalla: un mensaje de error, un aviso que no aparece, un campo que se oculta, la forma de un árbol de tareas, el tipo de producto en su ficha, el número de líneas de un asistente | El piloto de navegador (paso 3). La captura **es** el aserto |
| `visual+api` | Lo mismo, **más** una cifra que hay que cuadrar al céntimo o un recuento exacto: un total, el importe de un pie de documento, un subtotal de sección, un número de líneas o de tareas | El piloto deja la captura (paso 3) y la API la corrobora (paso 4). Si las dos discrepan, una miente y eso ya es un hallazgo |
| `api` | **Excepción, con nota obligatoria.** Solo lo que no tiene representación en pantalla | Script o `odoo-flow-tester` (paso 4) |
| `humano` | Un **juicio** que ninguna automatización puede emitir: si algo «se lee mal», si un documento es presentable ante un cliente | Una persona, sobre las capturas que deja el piloto |

## Por qué esta clasificación se rehízo

El reparto anterior era **72 `api` / 7 `visual-auto` / 4 `humano`** y estaba mal.
De los **14 defectos** del primer recorrido completo, **13 eran visibles en
pantalla**: un total inflado en el pie del presupuesto, un presupuesto que se
queda a cero, una factura de cliente en negativo, un mensaje duplicado, un aviso
que no llega a la cabecera, un asistente que suma líneas al marcar la casilla que
debía quitarlas. El único que no se veía era el contador de ejecuciones de un
cron.

Clasificar como `api` un camino que el usuario vería en pantalla convierte la
validación en un test de datos y deja de ser una prueba de usuario. De ahí la
regla nueva: **si se ve, se mira**; la API corrobora cifras, no sustituye a haber
mirado.

Reparto real, contado camino a camino:

| Canal | Caminos |
|---|---|
| `visual` | **25** |
| `visual+api` | **51** |
| `api` puro | **3** (A-06, F-24, G-10 — cada uno con su nota) |
| **Total** | **79** |

Los **4 juicios de canal `humano`** van listados al final: salen de las capturas,
no son caminos por sí mismos. Cinco caminos están **bloqueados** y se explican
también al final.

La última columna, **qué debe recoger la captura**, dice qué tiene que quedar
dentro de la imagen para que sirva de evidencia. Los 76 caminos `visual` y
`visual+api` llevan captura obligatoria, con el id del camino en el nombre.

---

## Los cinco estados de un camino

El catálogo cubre **todo lo que los módulos del vertical pueden hacer**, no lo que
hace una instalación concreta. Cada instancia recorre el subconjunto que le
corresponde, y la diferencia entre «aquí no existe» y «no se pudo comprobar» es lo
que separa un informe útil de uno alarmista.

| Estado | Qué significa | ¿Es un hueco? |
|---|---|---|
| `ok` | Se vio y hace lo que declara | No |
| `defecto` | Se vio y **no** hace lo que declara | Sí, del producto |
| `no-aplica` | **El módulo que lo habilita no está instalado.** El camino no existe en esta instancia | **No.** Se informa y se salta |
| `bloqueado` | Debería poder probarse aquí y **no se pudo**: falta dato de prueba o no se encontró el punto de entrada | Sí, de la validación |
| `pendiente` | Sin recorrer. El arnés rechaza el informe si queda alguno | — |

`scripts/reconocer_instancia.py` calcula los `no-aplica` en el paso 0b, antes de
empezar: dice qué caminos no existen en esa instalación y por qué módulo. No los
marques `bloqueado` — eso haría parecer un hueco de cobertura lo que es una
diferencia de alcance del cliente.

---

## Bloque A — importación (13 caminos)

Asistente `fiebdc3.import`. **No hay menú:** el botón «Importar FIEBDC3» vive en
el header de la vista **lista** de presupuestos (y como acción contextual sobre
un presupuesto abierto).

| id | Título y entrada | Qué debe pasar | Canal | Qué debe recoger la captura |
|---|---|---|---|---|
| A-01 | Importar creando presupuesto nuevo, con costes, plantillas y mediciones · Ventas → Presupuestos → marcar fila → Importar FIEBDC3 | Pedido nuevo; capítulos como secciones y partidas como líneas; `cost_ids` en las partidas con `~D`; mediciones en las que tienen `~M`; el trabajo termina en `done`. **La base tiene que ser la de la cabecera del `.bc3`** | visual+api | El árbol de líneas y el **total del pie** en la misma imagen: el descuadre por doble conteo de subpartidas se ve ahí |
| A-02 | *Track Inventory* desmarcado | Productos de tipo bien **sin** seguimiento de inventario (en 17.0, `type='consu'`) | visual | La ficha de producto, que no debe mostrar el botón de existencias |
| A-03 | *Product Type* = Services | Un producto de tipo servicio por concepto; la casilla de inventario deja de tener efecto | visual | El campo de tipo en la ficha del producto creado |
| A-04 | Producto genérico fijado | Todas las líneas al mismo producto; **el catálogo no crece**; las descripciones siguen siendo las del `.bc3` | visual+api | Las líneas con el producto repetido y el contador de la lista de productos, antes y después |
| A-05 | *Import Cost* desmarcado | Presupuesto con sus líneas pero sin `cost_ids` | visual+api | El presupuesto entero: si se queda solo con las secciones y a 0,00, el defecto salta a la vista |
| A-06 | Costes sí, *Create Cost Templates* no | Las partidas traen costes; no se crea ninguna plantilla de coste nueva | **api** | **Sin pantalla:** las plantillas de coste no tienen vista por la que contarlas (lo confirma G-10). El recuento va por API y **con nota en el informe** |
| A-07 | *Import Measurements* desmarcado | Ninguna línea con mediciones; la cantidad es la del `~C`. Alimenta E-02 | visual+api | Las líneas sin mediciones y el total del pie: aquí el total cuadra, y eso diagnostica A-01 |
| A-08 | *Recalculate prices from costs* marcado | El precio de las partidas con costes deja de ser el del `~C` y pasa a derivarse de sus costes | visual+api | **BLOQUEADO** |
| A-09 | El defecto de la casilla lo pone la compañía | Con el ajuste de compañía a cierto, el asistente abre ya marcado; al devolverlo a falso, abre desmarcado | visual | El asistente recién abierto, con la casilla en el estado que le imponga la compañía |
| A-10 | Importar **sobre un presupuesto existente** · abrir un borrador → acción contextual | No se crea pedido nuevo: las líneas se añaden al elegido, conservando cliente y líneas previas | visual+api | Dónde queda colocada la línea previa dentro del árbol importado |
| A-11 | NEGATIVO — fichero que no es `.bc3` | `ValidationError`: *«The FIEBDC3 file must be a BC3 file»*. Ni pedido ni trabajo | visual | El diálogo de error con su texto |
| A-12 | NEGATIVO — sin cliente y sin pedido de destino | **No sale ningún mensaje**: la etiqueta del campo se pone en rojo y el asistente se queda abierto. Se comprueba por ausencia: captura del asistente y aserto de que **no** hay diálogo ni notificación de error | visual | El asistente con el campo en rojo y **sin** diálogo ni notificación en pantalla |
| A-13 | NEGATIVO — pedido con un trabajo FIEBDC3 en curso | `UserError`: *«The sale order … already has a FIEBDC3 job.»* Solo bloquea si el trabajo **no** está en `done` ni `failed` | visual | El diálogo, con el nombre del pedido interpolado |

## Bloque B — parsers de porcentaje (9 caminos)

`fiebdc3.percentage.map`. Ventas → Configuración → **BC3 Percentage Parsers**.
Matriz `import_mode` (4) × `price_effect` (2): 6 combinaciones válidas + 3
negativas. Todos se miden **sobre el mismo `.bc3`**, cambiando solo el parser.

| id | Título y entrada | Qué debe pasar | Canal | Qué debe recoger la captura |
|---|---|---|---|---|
| B-01 | `import_mode=none` + `price_effect=none` | Combinación inerte: ni línea global, ni hija, ni de coste, ni cambio de precio. **Es la línea base de comparación** | visual+api | El pedido con su total y su número de líneas: es la imagen contra la que se comparan B-02 a B-06 |
| B-02 | `none` + `add` | Ninguna línea nueva, pero el `price_unit` de cada partida sube el porcentaje respecto a B-01 | visual+api | El precio unitario de la misma partida, junto al de B-01 |
| B-03 | `cost` + `none` (con categoría) | Una línea de coste por partida con el porcentaje sobre su base; el precio de venta no cambia, cambia el margen | visual+api | El pedido resultante y el estado del trabajo: un pedido a 0,00 con el trabajo colgado es lo que ve el usuario |
| B-04 | `cost` + `add` | La línea de coste **y** la subida de precio. Es la única pareja donde `add` convive con un modo que crea registro | visual+api | Ídem que B-03 |
| B-05 | `line` | Una línea de venta **hija por partida** con el importe del porcentaje; el total sube en su suma | visual+api | La línea hija bajo su partida y el total del pie |
| B-06 | `global` | **Una sola** línea al final del pedido, con el porcentaje sobre el total; las partidas no se tocan | visual+api | La línea única al final del pedido y el total |
| B-07 | NEGATIVO — `add` con `line` | `ValidationError` al guardar, nombrando `'line'`. El registro no se guarda | visual | El diálogo, comprobando que nombra `'line'` |
| B-08 | NEGATIVO — `add` con `global` | El mismo error **interpolando `'global'`**, no un texto genérico | visual | El diálogo, comprobando que nombra `'global'` |
| B-09 | NEGATIVO — `cost` sin categoría de coste | `ValidationError`: *«Cost Category is required when Import Mode is 'Cost Line'.»* | visual+api | El diálogo, para contar cuántas veces sale la frase |

## Bloque C — proyecto (8 caminos)

`inbuilt.project.create`. Presupuesto confirmado (con tipo de pedido que crea
proyecto) → botón **«Crear proyecto»**. Eje `create_tasks` (4 valores) + dos
casillas condicionales.

| id | Título y entrada | Qué debe pasar | Canal | Qué debe recoger la captura |
|---|---|---|---|---|
| C-01 | *No create tasks* | Proyecto creado con su cuenta analítica y **cero** tareas | visual+api | El botón inteligente de tareas del proyecto marcando 0 |
| C-02 | *By chapters* | Una tarea por capítulo, sin subtareas; las partidas no generan tarea | visual+api | La lista de tareas del proyecto |
| C-03 | *By parts*, sin subtareas de capítulo | Una tarea por partida, **todas al mismo nivel**; los capítulos no generan tarea | visual+api | La lista de tareas, sin sangrado |
| C-04 | *By parts* + `chapter_subtasks` | Una tarea padre por capítulo y, colgando, una subtarea por partida. **La jerarquía se comprueba en la vista de subtareas, no con el recuento** | visual+api | La vista de subtareas: capítulo padre arriba, partidas sangradas debajo |
| C-05 | *By sub parts*, sin `parts_subtasks` | Una tarea por subpartida, todas planas. Requiere el `.bc3` de tres niveles | visual+api | La lista de tareas, todas planas, con su recuento a la vista |
| C-06 | *By sub parts* + `parts_subtasks` | **Dos padres, no cuatro**: solo las partidas que se descomponen en subpartidas se vuelven padre; las que cuelgan de la sección de capítulo quedan planas. Esperar cuatro padres es leer mal el asistente | visual+api | La vista de subtareas: dos padres, cinco hijas y dos sueltas |
| C-07 | La casilla conserva su valor al ocultarse | Cambiar de desplegable oculta la casilla pero **no la limpia**: sigue valiendo cierto sin verse. El piloto la marca, cambia de opción, vuelve y captura; y compara el árbol resultante | visual | Tres capturas del asistente en la secuencia, más el árbol resultante |
| C-08 | Modo **Actualizar**: relanzar sobre un pedido que ya tiene proyecto | El asistente abre sin los campos de plantilla ni nombre y con el botón rotulado **«Update»**; se actualiza el proyecto existente, no aparece un segundo proyecto | visual | El asistente sin los campos de plantilla, el rótulo del botón, y la lista de proyectos sin duplicado |

## Bloque D — material de obra (6 caminos)

`project.stock.move`. Proyecto → botón **«Materials»**, y pestaña de material de
la tarea.

| id | Título y entrada | Qué debe pasar | Canal | Qué debe recoger la captura |
|---|---|---|---|---|
| D-01 | Alta de material desde el proyecto | Línea con producto, cantidad y tarea de destino, guardada colgando del proyecto. **`uom_id` es obligatorio y por API no se rellena solo** | visual | La línea de material guardada, con su producto, su cantidad y su tarea |
| D-02 | El selector ofrece productos **almacenables** | El desplegable lista los almacenables además de los consumibles. Si no aparecen, el dominio de la 18.0 se coló en la 17.0 y es defecto | visual+api | El desplegable abierto |
| D-03 | Abrir una tarea con material | La tarea **abre sin error de dominio** al pintar la vista y deja añadir material ligado a esa tarea | visual | El formulario de la tarea con sus botones inteligentes y sin pantalla de error |
| D-04 | Generar el movimiento del material | Nace el documento de abastecimiento ligado al proyecto y, en la línea, a la tarea; el botón inteligente del proyecto sube | visual+api | El documento de abastecimiento y el botón inteligente del proyecto |
| D-05 | Validar y ver el **coste real** en la tarea | El documento pasa a Hecho y el coste real aparece imputado en tarea/proyecto (distinto de cero) | visual+api | El coste real en la tarea: un 0,00 con el material ya recibido es el defecto, y está en la pantalla |
| D-06 | Rama de fabricación | Se genera una orden de fabricación ligada al proyecto. Sin lista de materiales, el camino es `bloqueado`, no `defecto` | visual | La orden de fabricación ligada y el botón inteligente de fabricaciones de la tarea |

## Bloque E — certificación (12 caminos)

Proyecto → **Certifications** → Nuevo → botón **«Añadir líneas»**.
**El asistente de añadir líneas exige secciones: sin capítulos ofrece cero
líneas, en silencio.**

**E-10, E-11 y E-12 cubren los tres botones de estado del formulario**, que hasta
la versión 1.0 de este catálogo no tenían camino. Los dos primeros llevan control
por facturas vinculadas: hay que probar el caso permitido y el bloqueado. El
tercero manda un correo al cliente, así que se comprueba que el diálogo abre con
el destinatario y la plantilla correctos **y se cierra sin enviar**.

| id | Título y entrada | Qué debe pasar | Canal | Qué debe recoger la captura |
|---|---|---|---|---|
| E-01 | Certificar un periodo desde las mediciones | Las cantidades se derivan de las mediciones marcadas y **no** son editables a mano; el importe del periodo cuadra con su suma | visual+api | El diálogo de mediciones, con sus dos columnas de porcentaje |
| E-02 | Certificar **sin mediciones**, escribiendo el % a mano | Al escribir el % se recalculan cantidad e importe y viceversa; los tres campos quedan coherentes **y la cabecera de la certificación los recoge** | visual+api | Línea y cabecera en la misma imagen: la cabecera a cero con la línea llena es el defecto |
| E-03 | Segundo periodo: origen acumula, actual es la diferencia | `actual = origen − anterior`, al céntimo. Hay que cerrar la primera certificación antes (ver E-07) | visual+api | La cabecera con origen, anterior y actual a la vez |
| E-04 | Certificar **por encima del 100 %** | **Debe dejar**, con los módulos de exceso instalados: se guarda, el porcentaje pasa del 100 % y se registra el exceso. Que salte el tope superior sería el defecto | visual+api | El aviso de exceso de la cabecera |
| E-05 | NEGATIVO — certificar menos que lo ya certificado | `ValidationError`: el tope **inferior** sí se conserva | visual | El diálogo de error con su texto |
| E-06 | NEGATIVO — editar a mano una línea con mediciones | `ValidationError`. Salta tanto al escribir cantidad como porcentaje | visual | El diálogo, en los dos intentos |
| E-07 | NEGATIVO — segunda certificación en borrador | `ValidationError`: solo una en borrador por proyecto | visual | El diálogo de error con su texto |
| E-08 | NEGATIVO — porcentaje fuera de 0–100 | `ValidationError`. **El campo se guarda como fracción**: anotar qué se tecleó exactamente | visual | El diálogo **y** el valor tal como se tecleó en el campo |
| E-09 | NEGATIVO — borrar una línea de una certificación no borrador | `UserError`; la línea sigue ahí | visual | El diálogo y, detrás, la línea que sigue en su sitio |
| E-10 | Devolver a borrador · botón **Restablecer a borrador** | **Sin facturas se permite** y vuelve a `draft`. **Con factura activa bloquea**: «No puede cambiar esta certificación a borrador porque tiene facturas. Debe eliminarlas primero.» Probar los dos casos | visual | El estado en la barra antes y después, y el diálogo del caso bloqueado |
| E-11 | Cancelar · botón **Cancelar** | **Sin facturas se permite** y pasa a `cancel`. **Con factura activa bloquea**: «No puede cancelar esta certificación porque tiene facturas vinculadas. Debe cancelarlas o eliminarlas primero.» | visual | El estado en la barra y el diálogo del caso bloqueado |
| E-12 | Enviar por correo · botón **Enviar por correo electrónico** | Abre el compositor `mail.compose.message` en diálogo, con la plantilla de certificación y el cliente del proyecto. **SALE HACIA EL CLIENTE: comprobar destinatario y plantilla y CERRAR SIN ENVIAR** | visual | El diálogo abierto con destinatario, asunto y adjunto |

## Bloque F — facturación (25 caminos)

Certificación en borrador → botón **«Create Invoice»**.
Ejes: `certification_billing_type` (current/origin) ×
`certification_billing_lines` (consolidated/all_lines/certified_lines) ×
`certification_previous_lines` (all/unique_line).

**Cada modo exige SU PROPIO PROYECTO**: la configuración se congela en cuanto el
proyecto tiene una certificación.
**Y los modos a origen exigen DOS periodos facturados**: la línea negativa no
existe en el primero. El tercero es el que destapa el defecto grave.

| id | Título y configuración | Qué debe pasar | Canal | Qué debe recoger la captura |
|---|---|---|---|---|
| F-01 | `current` + `consolidated` | **Una** línea con el producto consolidado, por lo certificado en el periodo. Sin negativa | visual+api | La factura con su línea única y su total |
| F-02 | `current` + `all_lines` | Una línea por cada línea del pedido, incluidas las de importe cero. Sin negativa | visual+api | Si la factura conserva las secciones del presupuesto y sus subtotales |
| F-03 | `current` + `certified_lines` | Solo las líneas con certificación distinta de cero: **menos líneas que F-02** sobre la misma certificación | visual+api | La factura junto a la de F-02, para ver qué líneas desaparecen |
| F-04 | `origin` + `consolidated` + `previous=all` (1er periodo) | Una línea por el acumulado a origen; sin negativa porque no hay facturas anteriores | visual+api | La factura completa, con su total |
| F-05 | `origin` + `consolidated` + `unique_line` (1er periodo) | Idéntico a F-04 en el primer periodo. Fija la línea base de F-18 | visual+api | La factura completa: es la base de comparación de F-18 |
| F-06 | `origin` + `all_lines` + `all` (1er periodo) | Todas las líneas del pedido con el acumulado a origen | visual+api | Dónde cae la negativa a partir del 2º periodo y qué subtotal deja la sección |
| F-07 | `origin` + `all_lines` + `unique_line` (1er periodo) | Todas las líneas a origen; sin negativa en el primero | visual+api | Ídem |
| F-08 | `origin` + `certified_lines` + `all` (1er periodo) | Solo las certificadas, con el acumulado a origen | visual+api | Las líneas certificadas y la base del pie |
| F-09 | `origin` + `certified_lines` + `unique_line` (1er periodo) | Solo las certificadas a origen; sin negativa | visual+api | Ídem, con la etiqueta de la negativa cuando aparezca |
| F-10 | **Retención** en modo actual | Factura **y** asiento de retención con su cuenta y su diario; retenido = base × porcentaje. **Nace al PUBLICAR, no al crear** | visual+api | El asiento de retención con su cuenta y su diario, y la factura antes y después de publicar |
| F-11 | **Retención** en modo origen | Anotar sobre qué base se calcula: acumulado o neto del periodo tras la negativa. Es el cruce que nadie había mirado | visual+api | El asiento de retención junto a la cabecera de la certificación, para leer sobre qué base salió |
| F-12 | NEGATIVO — origen sin producto de anteriores | `ValidationError` **al guardar el proyecto**, no al facturar | visual | El diálogo, y que sale al guardar el proyecto |
| F-13 | NEGATIVO — consolidado sin producto consolidado | `UserError` **al facturar**. Solo se alcanza vaciándolo en el proyecto **y** en la compañía | visual | El diálogo de error con su texto |
| F-14 | NEGATIVO — certificación sin líneas | `UserError` | visual | El diálogo de error con su texto |
| F-15 | NEGATIVO — importe a certificar cero | `UserError` | visual | El diálogo de error con su texto |
| F-16 | NEGATIVO — facturar dos veces | Debería salir *«This certification already has an invoice.»*. Comprobar además que el asiento de retención **no** cuenta como factura previa | visual | El diálogo que sale **de verdad**, que no es el que se esperaba |
| F-17 | NEGATIVO — certificación que no está en borrador | `UserError` | visual | El diálogo de error con su texto |
| F-18 | Negativa en `origin`+`consolidated`+`unique_line` · **2º periodo** | **Una sola** negativa agregada con el producto de anteriores; el total iguala lo certificado en el periodo | visual+api | La negativa agregada y el total del pie |
| F-19 | Negativa en `origin`+`consolidated`+`all` · **2º periodo** | **Una negativa por cada factura anterior**, etiquetada con su factura. Con una sola factura previa es indistinguible de F-18 salvo por el texto: **hacen falta tres periodos** | visual+api | Las dos negativas del tercer periodo con su etiqueta de factura |
| F-20 | Negativa en `origin`+`all_lines`+`unique_line` · **2º periodo** | Todas las líneas a origen más **una** negativa al final; el neto coincide con lo certificado | visual+api | La negativa dentro de la última sección y el subtotal que deja |
| F-21 | Negativa en `origin`+`all_lines`+`all` · **2º periodo** | Todas las líneas a origen más una negativa por factura anterior | visual+api | Ídem |
| F-22 | Negativa en `origin`+`certified_lines`+`unique_line` · **2º periodo** | Solo certificadas a origen más una negativa. **Ojo al caso de una línea certificada en el periodo 1 y no tocada en el 2: comprobar si sigue apareciendo** | visual+api | La factura del tercer periodo, para ver si la línea no tocada sigue apareciendo |
| F-23 | Negativa en `origin`+`certified_lines`+`all` · **2º periodo** | Solo certificadas a origen más una negativa por factura anterior. **Llevarlo al tercer periodo dejando una partida sin tocar** | visual+api | La factura completa si sale en negativo: base, impuesto y total en la misma imagen |
| F-24 | Reparto analítico | Cada línea con el 100 % a la cuenta analítica del proyecto. Si el proyecto no la tiene, el reparto sale vacío y la factura se crea igual: **comprobar los dos casos y anotar cuál se dio** | **api** | **Sin pantalla:** el reparto analítico va en una columna **opcional** que la vista de la factura no pinta por defecto. Va por API y **con nota en el informe**; si en la instancia esa columna está habilitada, el camino pasa a `visual+api` |
| F-25 | El triple descuento viaja del presupuesto a la factura de certificación · partida con `discount`/`discount2`/`discount3` → confirmar → certificar → **Crear factura** | **Lo que se asegura: el importe cuadra.** La base de la factura debe ser exactamente la del presupuesto con sus tres descuentos aplicados. **Lo que hay que OBSERVAR Y ANOTAR, sin darlo por bueno ni por malo:** cómo queda representado el descuento en la línea de factura — si conserva los tres campos, o si se aplana al precio unitario dejándolos a cero. Anotar precio unitario y los tres descuentos de la línea de factura junto a los del presupuesto | visual+api | La línea de factura con su precio unitario y sus columnas de descuento, al lado del total |

## Bloque G — márgenes, costes y compras (10 caminos)

Casi todo detrás del grupo de costes avanzados de ventas
(`indaws_sale_margin.group_sale_advanced_costs`): **sin ese grupo activo en el
usuario, la mitad de este bloque no se ve**.

| id | Título y entrada | Qué debe pasar | Canal | Qué debe recoger la captura |
|---|---|---|---|---|
| G-01 | Coste y margen por partida | La partida muestra sus líneas de coste con categoría e importe; su coste es la suma y el margen se recalcula al tocarlas | visual+api | El bloque de costes desplegado en la línea |
| G-02 | Resumen de costes por categoría en el pie del presupuesto | Una fila por categoría más una para el coste sin categoría; la suma cuadra con el coste total | visual+api | El pie del presupuesto (el bloque solo aparece si hay costes) |
| G-03 | Asistente Descuento → **Margen (%)**, sin *apply on all lines* | Solo cambian de precio las líneas con subtotal positivo; el margen resultante es el pedido | visual+api | Los precios unitarios antes y después, en dos capturas |
| G-04 | El mismo asistente **con** *apply on all lines* | Se recorren todas las líneas válidas, incluidas las de subtotal cero. Comparar el conjunto afectado con G-03 | visual+api | **BLOQUEADO** |
| G-05 | Margen ≥ 100 % | El asistente **informa de éxito y no toca ningún precio**. Es `ok`: limitación declarada, no defecto | visual | La notificación de éxito y, detrás, los precios intactos |
| G-06 | Triple descuento **aditivo** | El descuento efectivo es la suma de los tres | visual+api | **BLOQUEADO** |
| G-07 | Triple descuento **multiplicativo** | El descuento efectivo es la composición, no la suma: subtotal mayor que en G-06 | visual+api | **BLOQUEADO** |
| G-08 | Venta → Compra agrupando (`purchase_type = grouped`) | Las líneas del mismo proveedor se acumulan en un único pedido de compra, reutilizando uno existente si lo hay | visual+api | El asistente mientras se rellena: el número de líneas propuestas **cambia** al marcar la casilla de excluir, y ahí está el defecto |
| G-09 | Venta → Compra creando pedido nuevo + *exclude product line* | Un pedido nuevo por proveedor y **sin** las líneas de producto excluidas. Comparar el conjunto de líneas con G-08. Ojo: el asistente hay que abrirlo con el contexto de la acción | visual+api | Los pedidos de compra resultantes con sus líneas y su total |
| G-10 | Plantillas de coste reutilizables | Una plantilla creada a mano se aplica a una partida, que hereda sus líneas de coste y recalcula coste y margen. Las que crea la importación deben ser igual de reutilizables | **api** | **BLOQUEADO** y **sin pantalla:** no hay punto de entrada en la interfaz para aplicar una plantilla a una línea, así que hoy solo se puede constatar por API que las plantillas existen. Nota obligatoria en el informe |

---

## Los siete caminos cuyo aserto de pantalla ya está escrito

Estos siete llegaban del recorrido anterior con el aserto de navegador ya
redactado. Se conservan tal cual porque ahorran trabajo al piloto; su canal es el
que dice la tabla de su bloque.

| Camino | Canal | Qué tiene que ver el piloto | Cómo lo aserta |
|---|---|---|---|
| A-12 | visual | Que **no** aparece diálogo ni notificación de error | Ausencia de `.o_notification`, `.modal-title` de error y presencia de la clase de campo inválido |
| C-04 | visual+api | Un árbol de dos niveles: capítulo padre, partidas hijas | Vista de subtareas del proyecto, captura y recuento de filas con sangrado |
| C-05 | visual+api | Siete tareas **planas**, ninguna con padre | Lista de tareas agrupada, captura |
| C-06 | visual+api | Dos padres, cinco hijas y dos sueltas | Vista de subtareas, captura |
| C-07 | visual | La casilla se oculta al cambiar de desplegable y **sigue valiendo cierto** | Tres capturas del asistente en la secuencia, más el árbol resultante |
| C-08 | visual | El asistente sin campos de plantilla y con el botón rotulado «Update» | Captura del asistente y texto del botón |
| D-03 | visual | La tarea **abre** y pinta sus botones inteligentes de material | Captura del formulario y ausencia de pantalla de error |

## Los cuatro juicios de canal `humano`

No son caminos: son las preguntas que quedan cuando la captura ya está hecha y
un número no las contesta. Se entregan **con la imagen delante**, nombrando a
quien las responde.

| Juicio | Sale de | La pregunta |
|---|---|---|
| H-1 | F-06 / F-07 / F-20 / F-21 | Un capítulo de la factura con subtotal negativo, ¿es presentable ante el cliente de la obra? |
| H-2 | A-10 | La línea previa intercalada dentro de una partida descompuesta, ¿rompe la lectura del presupuesto? |
| H-3 | E-01 | Dos columnas contiguas con la misma magnitud en unidades distintas (`100%` y `1,00`), ¿induce a error a un jefe de obra? |
| H-4 | Interfaz en general | Mezcla de idiomas y divisa antigua en listas, ¿es aceptable para entregar la implantación? |

## Los cinco caminos bloqueados

`bloqueado` **no es un defecto**: es que el dato de prueba o el alcance de módulos
no permitió distinguir el camino. Se documentan para que nadie los cuente ni como
defectos ni como comprobaciones hechas.

| Camino | Por qué se bloqueó | Qué haría falta para cerrarlo |
|---|---|---|
| **A-08** — recalcular precios desde costes | En el `.bc3` fabricado el precio de la partida **ya** coincide con la suma de sus costes, así que recalcular no cambia nada y el camino no prueba nada | Un `.bc3` donde el precio de la partida difiera de la suma de su descomposición |
| **G-04** — *apply margin on all lines* | Idéntico a G-03: en el presupuesto de prueba **todas** las líneas tienen subtotal positivo, así que no hay ninguna a cero que la casilla pudiera incluir o excluir | Un presupuesto con al menos una línea de subtotal cero |
| **G-06** — triple descuento aditivo | El módulo de triple descuento **de ventas** estaba sin instalar: `sale.order.line` solo tenía `discount`. El de facturas sí estaba | Instalar el módulo, o trasladar la comprobación al lado de facturas |
| **G-07** — triple descuento multiplicativo | Mismo motivo que G-06 | Lo mismo |
| **G-10** — plantillas de coste reutilizables | Las plantillas **existen** (las crea la importación), pero `sale.order.line` no tiene ningún `many2one` que apunte a una: no se encontró el punto de entrada | Probar el asistente de costes de línea, que no se llegó a ejercitar |

Antes de dar por bloqueado un camino en otra instancia, comprobar si el motivo
sigue siendo el mismo: los cuatro primeros dependen del **dato de prueba** y G-06
y G-07 del **alcance de módulos** (paso 2).

## Dos comportamientos que parecen fallos y no lo son

- **E-04** — certificar por encima del 100 % **debe** dejar: los módulos de exceso
  quitan el tope superior a propósito. Lo que sería defecto es que saltara.
- **G-05** — margen ≥ 100 %: no existe precio finito que lo dé, así que el
  asistente informa de éxito y no hace nada. Es `ok` por declarado, pero para el
  usuario es indistinguible de un fallo silencioso y merece salir en el informe.

## Cómo se decidió el canal

Para que dentro de un año se entienda por qué un camino está en un canal y no en
otro, el criterio, en este orden:

1. **¿Un usuario lo vería en pantalla? → `visual`.** Es el caso por defecto y
   gana los empates. Cuenta como «se ve» todo esto: un mensaje de error con su
   texto, un aviso que **no** aparece, un campo que se oculta o se pone en rojo,
   el rótulo de un botón, el tipo de producto en su ficha, la forma de un árbol
   de tareas, el número de líneas que propone un asistente. La captura no es
   adorno: **es** el aserto.
2. **¿Además hay una cifra que cuadrar al céntimo o un recuento exacto? →
   `visual+api`.** Todo camino cuyo resultado esperado nombre un importe —un
   total, el pie de un documento, un subtotal de sección, una retención, una
   línea negativa— o un número exacto de líneas, tareas o productos. La captura
   prueba **qué se vio**; la API confirma el céntimo. Si las dos discrepan, una
   miente, y ese desacuerdo es en sí un hallazgo. Es el canal mayoritario porque
   este vertical es aritmética de obra: casi todo acaba en un importe.
3. **¿No tiene representación en pantalla? → `api`, y con nota.** Es la
   excepción, y quien la use tiene que escribir en el informe por qué. Solo tres
   caminos la merecen: A-06 (contar plantillas de coste, que no tienen vista),
   F-24 (el reparto analítico vive en una columna opcional que la factura no
   pinta por defecto) y G-10 (no existe punto de entrada en la interfaz).
4. **¿Es un juicio y no una comprobación? → `humano`.** Los cuatro del apartado
   anterior. No se automatizan porque la pregunta no es «¿cuánto?» sino «¿esto se
   puede entregar?».

Dos avisos sobre el paso 3:

- **Los asertos legítimos de solo API casi nunca son caminos.** El contador
  `numbercall` de un cron, el estado instalado/no instalado de un módulo, el
  número de apuntes analíticos o un indicador interno como `is_composition` son
  comprobaciones del **paso 2** (alcance) o diagnósticos de un defecto ya visto,
  no caminos de usuario. Por eso `api` puro se queda en tres y no en setenta.
- **El montaje de datos por API no cambia el canal.** Duplicar presupuestos,
  confirmar pedidos o crear nueve proyectos se hace siempre por API, incluso en
  los caminos `visual`: el ratón se reserva para el tramo que se comprueba.
