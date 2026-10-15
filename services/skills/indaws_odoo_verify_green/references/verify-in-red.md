# Verificar EN ROJO — el procedimiento

> El paso que más falsos verdes caza y el que más se salta, porque cuesta un run extra y **el
> incentivo está en contra**: ya lo tienes verde y verificar en rojo sólo puede darte malas noticias.
> Ese es exactamente el motivo por el que funciona.

## La idea en una frase

Un test que pasa **no prueba que tu arreglo haga algo**. Lo prueba que **falle cuando el arreglo no
está**.

## El procedimiento

1. **Copia el fichero** a un sitio fuera del árbol. No confíes en deshacerlo a mano.
2. **Neutraliza el arreglo, no lo borres.** La neutralización tiene que simular el comportamiento
   ANTERIOR, no romper el código de cualquier manera — ver «El error de neutralización» abajo.
   Formas limpias: `return []` al principio de la función, `if False:` en la guarda, invertir el
   orden de un bucle, devolver el valor por defecto.
3. **Corre SÓLO los tests implicados**, por nombre. Un run entero tarda y entierra la señal.
4. **Comprueba tres cosas, no una:**
   - caen los que esperabas,
   - **no** cae ninguno que no esperabas (si cae, tu modelo mental del cambio está mal),
   - **el mensaje es el correcto** — un test puede caer por el motivo equivocado.
5. **Restaura y confirma** que no quedan marcas: `grep -c NEUTRALIZADO` debe dar 0.
6. **Escribe el resultado.** «3 de 5 caen; los otros 2 pasan en rojo a propósito porque guardan
   comportamiento que ya existía» es una frase honesta. «Verificado en rojo» a secas, no.

## Qué significa cada resultado

| Lo que pasa al neutralizar | Qué significa |
|---|---|
| Caen los que esperabas, con el mensaje esperado | El arreglo está protegido. Sigue. |
| **No cae ninguno** | O el test es teatro, o el código no hacía nada. **Las dos son hallazgo.** |
| Cae uno que no esperabas | Tu cambio hace más de lo que crees. Mira antes de seguir. |
| Cae con un mensaje distinto | El test lo detecta por accidente, no por diseño. |
| Todo cae | Has roto la instalación, no has neutralizado. Vuelve al paso 2. |

## El error de neutralización, con un caso real

Un test afirmaba que un multi-if evalúa sus reglas **en orden** y gana la primera que casa.

**Primera neutralización:** cambiar el `return` por una asignación, para que «siguiera mirando». No
simulaba «gana la última»: rompía el bucle de otra forma y el flujo se iba por la rama de reserva.
Resultado: el test del ORDEN **no cayó**, y con él sin caer la verificación no valía nada.

**Segunda:** `for rule in reversed(self._rules())`. Eso sí es «el orden al revés», que es justo lo
que el test afirma que importa. Cayó.

**La lección:** la neutralización tiene que ser la NEGACIÓN de lo que el test afirma. Si no lo es, un
test que no cae no te dice que sea teatro — te dice que has neutralizado mal.

## Un caso donde el rojo encontró el teatro

Un test de join AND (esperar a que lleguen todas las ramas) pasaba **con y sin** el AND. Las dos
ramas tenían el mismo largo, así que la cola llegaba al join después de ambas de todas formas y el
orden salía igual.

Con ramas de largos distintos —`start → a → join` y `start → b1 → b2 → join`— un join que **no**
espera corre en medio (`a, b1, join, b2`) y uno que espera corre el último. Ahí sí discrimina.

El test llevaba horas en verde **sin afirmar nada**.

## Cuándo NO hace falta

- Cuando el test es de **capacidad nueva** y sin el código ni siquiera compila o el modelo no existe:
  el rojo es trivial y no informa.
- Cuando el test guarda **comportamiento preexistente** (una regresión), no un arreglo tuyo. Ahí
  pasa en rojo a propósito — y **hay que decirlo**, o parecerá cobertura que no es.

## Cómo se escribe en el informe

Bien:

> 5 tests nuevos. Los **3 que protegen un arreglo, verificados en rojo** — caen los tres con su
> mensaje. Los otros dos pasan en rojo **a propósito**: el método ya existía, no prueban un arreglo
> mío, son guardas de regresión.

Mal:

> 5 tests nuevos, todos en verde. ✅
