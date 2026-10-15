# Barrido de pantallas e informes — por qué no es un camino más

## La pregunta que los caminos no responden

Un camino es **entrada → acción → resultado esperado**. «¿La pantalla se abre?»
no tiene resultado de negocio contra el que comparar: es otra familia de fallo.

Y los caminos no la cubren, porque siguen el proceso: pisan las pantallas del
recorrido feliz y nada más. Medido sobre una instalación real:

| | |
|---|---|
| Vistas que el vertical añade o hereda | **119** en 36 modelos |
| De ellas **heredadas** (modifican una pantalla de Odoo) | **79** |
| En modelos que **ningún camino abre** | **59**, de las cuales **40 heredadas** |

`res.config.settings` sola tenía **14 vistas heredadas** y ningún camino entraba
en Ajustes. Tampoco se abría el formulario de producto, ni el de cliente, ni las
categorías de coste, ni los tipos de pedido.

## Por qué importa: ya nos avisó el propio catálogo

**D-03 existe porque una vista de tarea reventaba al pintarse** con un dominio.
Ese camino se escribió a posteriori, después de encontrar el fallo. Es la prueba
de que «se abre» y «calcula bien» son preguntas separadas.

Y un campo que desaparece entre versiones —`uom.uom.category_id` en la 19.0— no
solo rompe código: **deja muerta cualquier vista heredada que lo cite**, para
todo el mundo, no solo para la prueba.

## La lista no se escribe a mano

Se le pregunta a la instancia qué vistas trae el vertical. Una lista fija se
queda obsoleta en cuanto alguien instala un módulo, y a partir de ahí el barrido
dice que todo está bien **porque ya no mira donde hay que mirar**. Es el mismo
error que llevar el total de un `.bc3` en una constante.

## La unidad es el par (modelo, tipo), no la vista suelta

Esto costó un intento fallido y merece quedar escrito.

Validar vista por vista con `get_view(id, tipo)` **no sirve**: no protesta ni
pasándole una vista de otro modelo. Ignora el id. Un primer barrido dio «119
vistas montan bien» y ese resultado **no valía nada** — probablemente compuso la
vista por defecto 119 veces.

Lo detectó el **control negativo**, que es la única razón por la que no acabó en
el informe como un aprobado. Desde entonces el script lo lleva dentro: antes de
barrer pide una vista imposible, y si eso no revienta, se planta.

Lo que sí vale es componer el par **(modelo, tipo) sin id**: ahí Odoo combina
todas las heredadas, y una xpath que no encaja o un campo que ya no existe hacen
saltar el error. Que es además lo que abre un usuario.

## Los tres niveles y qué caza cada uno

| Nivel | Alcance | Caza | No caza |
|---|---|---|---|
| **API** `barrer_pantallas.py` | **todos** los pares (modelo, tipo) | xpath rota, campo inexistente, modificador inválido | lo que solo falla en el cliente o al pintar con datos |
| **Visual** `--plan` + piloto | solo lo que tiene **entrada de menú** | errores de render con datos reales, JavaScript, dominios | asistentes, diálogos y listas embebidas: no tienen menú |
| **Informes** `barrer_informes.py` | las plantillas QWeb | plantilla que revienta al imprimir | nada más: hay que abrirlas en el navegador |

**El informe tiene que decir qué quedó cubierto solo por API.** Dar por vista una
pantalla que nadie abrió es exactamente el vicio que este recorrido persigue.

## Por qué el barrido de informes es visual por obligación

No es una preferencia de diseño, son dos puertas cerradas:

1. Los métodos de render de `ir.actions.report` son **privados** (`_render_qweb_pdf`)
   y el RPC bloquea todo lo que empieza por `_`.
2. La vía HTTP `/report/html/…` necesita **sesión web**, y una API key no la da:
   comprobado contra una instancia real, `/web/session/authenticate` con la key
   contesta **Access Denied**.

Así que el script localiza los informes, les busca un registro con el que
probarlos y emite las direcciones. Abrirlas es del piloto.

Se usa `/report/html/` y **no** `/report/pdf/`: el HTML se pinta en la pestaña,
falla con la traza a la vista y **no descarga nada**. Un PDF descargaría un
fichero, y eso hay que pedirlo antes.

### Qué mirar en cada informe

- Que sale el documento, no una página de error de Odoo.
- Que no aparece «Internal Server Error» ni una traza de Python.
- Que los importes del documento **cuadran con los del registro**.

Ese último punto sirve de doble comprobación: en una validación real, el informe
de una certificación salió con un total de 270,00 €, exactamente el que había
medido el bloque de facturación. Cuando no cuadren, uno de los dos miente.

### Registros de prueba

El script prefiere registros **del recorrido** —los marcados— y los busca por
nombre y, si el modelo no lo lleva, por sus relaciones habituales (proyecto,
pedido, cliente). Imprimir un documento real del cliente no rompe nada, porque
el informe solo lee, pero **ensucia**: queda registrado y luego nadie sabe si
fue el consultor o la validación.

Cuando no hay más remedio, el plan lo dice con todas las letras:
`REAL DEL CLIENTE: no habia ninguno del recorrido`.
