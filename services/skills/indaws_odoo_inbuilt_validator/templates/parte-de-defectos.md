# Parte de defectos — vertical inbuilt sobre Odoo <versión>

<Cliente o implantación · entorno (staging / producción) · fecha>

**Base de la medida:** <N> caminos recorridos — **<N> `ok`, <N> `defecto`, <N>
`bloqueado`**. Los `bloqueado` **no son defectos**: son caminos que el dato de
prueba o el alcance de módulos no permitió distinguir, y van al final.

**Qué se ha corregido:** <nada / lo que sea>. Si corregir está fuera de alcance,
decirlo aquí: este documento es la entrada de esa discusión, no su resultado.

**Con qué `.bc3` se ha validado:** <fabricados / real del cliente>. Un fichero
fabricado no prueba lo mismo que uno real de obra; si son fabricados, decirlo en
esta línea y no más abajo.

**Cómo se ha decidido si un defecto viene de la migración o ya existía:**
<comparar el fichero entre las dos ramas; si el código señalado no aparece en el
diff, es el mismo que corría antes>. Donde no se haya podido comprobar, decir que
es lectura de código y no medición.

---

# GRUPO 1 — Mueven dinero o impiden cobrar

---

## 1 · <camino(s)> — <titular en una frase, con el hecho dentro>

**Qué se esperaba.** <Lo que dice el catálogo, con el número concreto.>

**Qué pasó.** <Lo medido, con números. Sin adjetivos.>

**Causa medida / dónde parece estar.** <Fichero:línea. Distinguir lo medido de la
hipótesis: si es hipótesis, decirlo.>

**Pasos para reproducirlo.**
1. <…>
2. <…>
3. <…>

**Evidencia.** `<ruta de la captura>`

**Origen.** <viene de la migración / ya existía / no se ha podido determinar, y
por qué se dice eso.>

**Por qué importa.** <Qué le pasa al cliente en la obra. Una o dos frases.>

---

## 2 · <…>

*(Repetir la estructura. Un apartado por defecto o por familia de defectos que
comparten causa.)*

---

# GRUPO 2 — Rompen o vacían una función entera

---

## <n> · <…>

---

# GRUPO 3 — Mensajes, presentación y riesgos silenciosos

---

## <n> · <…>

*(Aquí caben también los caminos con estado `ok` cuyo resultado es un **riesgo
silencioso**: algo que el sistema permite sin avisar. Marcarlo:
«**Estado en el catálogo: `ok`** — va en este parte porque … , no porque el
camino haya fallado».)*

---

# Defectos ya conocidos, confirmados

No se abre apartado para ellos. Se listan y se cita su número de
`references/defectos-conocidos.md`.

| # conocido | Camino(s) | ¿Sigue igual? | Diferencia observada |
|---|---|---|---|
| <n> | <id> | sí / no | <lo que haya cambiado, o «ninguna»> |

---

# Lo que NO es un defecto

**Caminos `bloqueado`.** Ninguno indica un fallo: indican que el dato de prueba o
el alcance de módulos no permitió distinguirlos. Están aquí para que nadie los
cuente como defectos ni como comprobaciones hechas.

| Camino | Por qué quedó bloqueado | Qué haría falta |
|---|---|---|
| <id> | <…> | <…> |

**Comportamientos correctos que parecen fallos.**
- <p. ej. certificar por encima del 100 % debe dejar cuando los módulos de exceso
  están instalados>
- <p. ej. una operación que informa de éxito y no hace nada por una limitación
  declarada: se registra `ok`, pero se anota que para el usuario es
  indistinguible de un fallo silencioso>

---

# Comprobación visual pendiente de juicio humano

<Los caminos que la skill recorrió con el navegador dejan capturas; lo que no se
puede automatizar es el juicio. Listar aquí las preguntas, con la imagen, y **a
quién** se le entregan. Nombrar a la persona.>

| # | Pregunta | Evidencia | Se la entrega a |
|---|---|---|---|
| H-<n> | <…> | `<ruta>` | <nombre> |

---

# Resumen

| # | Camino(s) | Qué rompe | Origen |
|---|---|---|---|
| 1 | <…> | <…> | <…> |

**La conclusión que sale de la columna de la derecha:** <una frase. Si de todo lo
encontrado solo N vienen de la migración, decir cuál y por qué.>
