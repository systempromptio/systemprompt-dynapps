# Preparación — las 13 condiciones previas

Lo que `scripts/comprobar_preparacion.py` comprueba antes de recorrer nada. Cada
condición va con **por qué importa** y **qué síntoma da si falta**, porque el
síntoma casi nunca dice la verdad: un camino que muere por falta de dato parece
un defecto del vertical y se reporta como tal.

**Si el script sale distinto de 0, el recorrido no empieza.**

---

## 1 · Compañía en el país y la divisa de la obra

*Comprobado:* `res.company.country_id` y `currency_id` (por defecto España / EUR;
configurables).

**Por qué.** FIEBDC-3 es un estándar español y los importes del `.bc3` no traen
divisa: se interpretan en la de la compañía. El país es además lo que decide qué
localización contable y qué impuestos hay disponibles.

**Síntoma si falta.** Los totales importados no comparan con la cabecera del
`.bc3` y toda la aritmética de certificación se mide contra una base equivocada.
Con el país sin localizar, no existen los impuestos de venta que las facturas de
certificación necesitan (condición 2).

## 2 · Impuestos de venta de la localización

*Comprobado:* existe al menos un `account.tax` de venta al 21 % y otro al 10 %
(porcentajes configurables).

**Por qué.** Los productos de certificación tienen que llevar impuesto; los
impuestos no se pueden inventar al vuelo cuando ya hay una factura montada.

**Síntoma si falta.** La factura de obra sale con base y total iguales. Nadie
avisa: la factura se crea, se publica y se envía sin IVA.

## 3 · Plan contable localizado cargado

*Comprobado:* número de `account.account` por encima de un mínimo (200 por
defecto; el plan genérico trae unas 47).

**Por qué.** Sin plan localizado no hay cuenta de retención (condición 6) ni
cuentas de ingreso coherentes para las facturas de certificación.

**Síntoma si falta.** Los caminos de retención mueren antes de empezar. Y el
arreglo tardío es caro: **Odoo solo deja cargar un plan contable si la compañía
no tiene ningún apunte** (`account/models/company.py:912-915`, `ResCompany._existing_accounting` — cuenta `account.move.line`; lo consulta `account/models/chart_template.py:206`, `AccountChartTemplate._load`), así que en
cuanto se emita la primera factura de certificación ya no se puede cambiar de
plan sin borrarla.

## 4 · Cron de importación FIEBDC-3 activo y con `numbercall` distinto de 0

*Comprobado:* el `ir.cron` de FIEBDC3, leído con `active_test = False`, tiene
`active = True` y `numbercall != 0`.

**Por qué.** La importación de un `.bc3` es asíncrona. El botón **«Queue»** del
formulario del trabajo **no procesa nada**: pone `state = queued` y crea un
`ir.cron.trigger`. Quien procesa es el obrero de cron.

**Síntoma si falta.** El presupuesto se queda vacío para siempre con un aviso
azul de «importación en progreso (0 %)» y un enlace «Refrescar» que solo recarga
la página. Parece que la importación falla, y no ha llegado a empezar. Dos
agravantes:

- El cron del módulo **se apaga solo tras su primera ejecución** (no fija
  `numbercall`, así que vale el defecto 1, baja a 0 y Odoo lo desactiva). Es el
  defecto conocido nº 10.
- Un cron inactivo es **invisible a un `search` normal**: hay que leerlo con
  `context={"active_test": False}` o parece que no existe.
- Con `numbercall = 0`, poner `active = True` **no basta**: la consulta de crones
  pendientes descarta los que tienen `numbercall != 0`. Hay que dejarlo en `-1`.

*Salida de emergencia si el obrero de cron va lento:* llamar por RPC a
`fiebdc3.job` → `trigger_run`, que es el mismo código que ejecuta el cron.

## 5 · Los dos productos de certificación, referenciados en la compañía y **con impuesto**

*Comprobado:* `res.company.certification_product_id` (consolidada) y
`certification_previous_product_id` (certificaciones anteriores) están fijados; y
aviso explícito si alguno **no lleva impuesto de venta**.

**Por qué.** Son los productos con los que el vertical arma la línea consolidada
y la línea negativa de «certificaciones anteriores». Ojo: los dos campos apuntan
a **`product.template`**, no a `product.product`.

**Síntoma si falta.**
- Sin producto de anteriores y con facturación **a origen**: `ValidationError` al
  **guardar el proyecto** (no al facturar).
- Sin producto consolidado: `UserError` **al crear la factura**. Y solo se
  alcanza si está vacío en el proyecto **y** en la compañía: el código cae al de
  la compañía sin protestar.
- **Sin impuesto en el producto: la factura de obra sale sin IVA y nada avisa.**
  Es exactamente lo que pasó al montar el escenario de facturación: la primera
  factura salió limpia de impuestos y solo se vio comparando totales.

## 6 · Cuenta y diario de retención en la compañía

*Comprobado:* `retention_account_id` y `retention_journal_id` fijados.

**Por qué.** La retención de obra genera un asiento aparte, con su cuenta y su
diario propios.

**Síntoma si falta.** Los dos caminos de retención se quedan `bloqueado`, no
`defecto`. Y un matiz que confunde aunque esté todo bien: **el asiento de
retención no nace al pulsar «Crear factura», nace al PUBLICAR la factura**. Quien
mire justo después de crearla verá un solo movimiento y creerá que la retención
no funciona.

## 7 · Un tipo de pedido que crea proyecto

*Comprobado:* existe algún `indaws.sale.order.type` con `create_project = True`.

**Por qué.** Es la entrada del bloque de proyecto: sin él, el presupuesto
confirmado no ofrece «Crear proyecto». El modelo se llama
**`indaws.sale.order.type`**, no `sale.order.type`.

**Síntoma si falta.** El botón no aparece y los ocho caminos de proyecto —y por
arrastre los de material, certificación y facturación— se quedan sin punto de
entrada. Parece que el vertical no instaló sus vistas.

## 8 · Los parsers de porcentaje BC3, en su configuración de fábrica

*Comprobado:* existen los 11 parsers y, con `--solo-parsers`, que
`import_mode` / `price_effect` / `apply_global` / `cost_category_id` siguen como
los deja el módulo (`global` / `none` / `False` / vacío).

**Por qué.** El bloque de parsers los cambia a propósito para recorrer sus modos.
Si alguien los dejó movidos, la línea base contra la que se comparan los cuatro
modos ya no es la de fábrica y las conclusiones no valen.

**Síntoma si falta.** Además de medir mal: si algún parser quedó en
`import_mode = 'cost'`, **la siguiente importación revienta a nivel SQL** y deja
el trabajo atascado en `running` con el campo `error` vacío — y un trabajo en
`running` bloquea futuras importaciones sobre ese pedido, así que la obra queda
inutilizable. Es el defecto conocido nº 7.

**Restaurar los parsers al terminar es obligatorio** (SKILL, *ALWAYS* 4).

## 9 · Un cliente del país con identificación fiscal

*Comprobado:* algún `res.partner` del país configurado con `vat` relleno.

**Por qué.** El asistente de importación exige cliente cuando no se le da pedido
de destino, y la factura de certificación no se publica sin identificación fiscal
válida en una localización que la valide.

**Síntoma si falta.** El asistente de importación **no da ningún mensaje**: marca
la etiqueta del campo en rojo y se queda abierto. Es facilísimo concluir que el
botón «Importar» no funciona.

## 10 · Categorías de coste con su código FIEBDC-3

*Comprobado:* existen `sale.order.line.cost.category` con `fiebdc3_code` `1`
(mano de obra), `2` (maquinaria) y `3` (materiales).

**Por qué.** Es lo que traduce el tipo del registro `~C` del `.bc3` a una
categoría de coste de Odoo. Vienen pobladas por los datos del módulo.

**Síntoma si falta.** Los costes importados caen todos en «sin clasificación»: el
resumen de costes por categoría del pie del presupuesto queda con una sola fila y
el margen por categoría deja de decir nada. Y un parser en modo «línea de coste»
sin categoría no se puede ni guardar.

## 11 · Datos maestros sin duplicar

*Comprobado:* los productos de certificación configurados aparecen una sola vez y
el diario de retención tiene un único registro con su código.

**Por qué.** Preparar la instancia dos veces es lo normal. Si la preparación no es
idempotente, quedan pares de productos con la misma referencia.

**Síntoma si falta.** La compañía apunta a uno y las pruebas miran el otro:
cambios que «no se aplican», importes que no cuadran y medidas irreproducibles
entre dos ejecuciones del mismo camino.

## 12 · Todas las listas de precios en la divisa de la compañía

*Comprobado:* ninguna `product.pricelist` en una divisa distinta de la de la
compañía.

**Por qué.** Cargar el plan contable cambia el país y la divisa de la compañía
pero **no toca la lista de precios**.

**Síntoma si falta.** El presupuesto importado nace en la divisa vieja con la
compañía ya en la nueva: los totales del pedido no comparan con la cabecera del
`.bc3`, y el error se atribuye a la importación. Pasó de verdad, y por eso esta
comprobación existe.

---

## Lo que estas 13 condiciones NO cubren

- **Los ficheros `.bc3`.** Sin un `.bc3` de **tres niveles** (capítulo → partida →
  subpartida) hay caminos que no se pueden distinguir. Ver `ficheros-bc3` en la
  documentación del recorrido: qué desbloquea cada fichero.
- **Un proyecto por modo de facturación.** La configuración de certificación **se
  congela con la primera certificación** del proyecto: los nueve modos necesitan
  nueve proyectos.
- **Que haya obreros de cron vivos.** En algunos alojamientos se ejecutan a
  rachas y con retraso de una hora.


---

## 13. Ninguna tarifa aplica reglas *(avisa, no bloquea)*

Una tarifa con reglas es legítima —una política de margen— pero hace que el
total del pedido importado **no sea comparable** con la cabecera del `.bc3`, y
entonces el margen se lee como descuadre.

Medido en una 19.0 con un recargo global del 60%: el mismo fichero daba
**15.210,08** en vez de los **9.506,30** de su cabecera. Repetida la importación
sobre un pedido con una tarifa sin reglas, salió **9.506,30 exacto**.

Para los caminos que comparan importes contra el fichero, usa un pedido con
tarifa sin reglas. Para los demás, la tarifa del cliente es lo correcto.

## 14. Las rutas de almacén tienen ubicación de destino *(avisa, no bloquea)*

Sólo importa si está `inbuilt_sale_project_stock_material`. Con ese módulo,
generar las tareas de un presupuesto **también genera el material de obra** de
cada coste, y para eso la ruta de la línea necesita una primera regla con
ubicación de destino. Si le toca una ruta sin ella:

```
UserError: Destination location is not set for material [MT001] ... on task ...
```

Y con eso se cae **todo** el bloque de proyecto menos «no crear tareas».

Es configuración de almacén que falta, **no un defecto del vertical**. La
comprobación nombra las rutas cojas para poder decidir: o se configuran, o se
marcan C-02..C-06 como `bloqueado` diciendo por qué.

Ojo con el criterio: no basta con que *alguna* ruta sirva. Basta **una** ruta
coja para tumbar el bloque, si es la que le toca a una línea. Una comprobación
que dijera «hay 3 rutas bien» pasaría con el problema delante.

Para medir los árboles de tareas a pesar de esto: retira las líneas de coste del
pedido antes de crear el proyecto —sin costes no hay material ni horas— y **dilo
en el informe**. El árbol medido es válido; la precondición se reporta aparte.

## Sobre la 6 (retención): avisa, no suspende

Que falte la cuenta o el diario de retención sólo apaga **F-10 y F-11**. El
resto del recorrido es válido sin ella, así que la comprobación avisa y sigue.

Crear la cuenta y el diario toca el **plan contable del cliente**: no lo hagas
en un staging ajeno sin permiso explícito. Marca F-10 y F-11 como `bloqueado`,
di por qué, y continúa.
