# Registro del recorrido visual — <cliente/implantación> · <fecha>

Plantilla de registro del recorrido **automatizado** de los caminos de canal
`visual-auto`: lo que no se puede aserta por API y sí se puede ver en pantalla.
Lo ejecuta `scripts/recorrido_visual.py`; esto es lo que queda escrito de ello.

| | |
|---|---|
| Entorno | <staging / producción — **los caminos que escriben no van a producción**> |
| Versión de Odoo | <17.0 / 18.0 / 19.0 — en 17.0 la entrada es `/web`, no `/odoo`> |
| Vía usada | <Playwright + Firefox (headless) / extensión de Chrome sobre el navegador de <persona>> |
| Quién inició sesión | <el script, con credenciales del entorno / <persona>, a mano> |
| Directorio de capturas | `<ruta relativa>` |
| Ejecutado por | <agente / persona> el <fecha y hora> |

Si la vía fue **Chrome**: navegadores conectados que se listaron y **cuál eligió
el usuario**: <…>. La skill no elige por su cuenta ni teclea contraseñas en la
sesión de nadie.

---

## Los siete caminos de canal `visual-auto`

Un bloque por camino. `observado` es lo que se vio, no lo que se esperaba ver.

### A-12 — Sin cliente y sin pedido de destino, **no sale ningún aviso**

| | |
|---|---|
| Entrada | Ventas → Presupuestos (vista lista) → marcar fila → Importar FIEBDC3 → Importar con los campos vacíos |
| Aserto | No existe `.o_notification` ni diálogo de error; el campo obligatorio queda marcado como inválido |
| Capturas | `<A-12-...png>` |
| Observado | <…> |
| Estado | ok / defecto / bloqueado |

### C-04 — *By parts* + subtareas de capítulo: árbol de dos niveles

| | |
|---|---|
| Entrada | Presupuesto confirmado → Crear proyecto → *By parts* + casilla de subtareas |
| Aserto | Una tarea padre por capítulo, una subtarea por partida; comprobado en la vista de subtareas, no con el recuento |
| Capturas | `<…>` |
| Observado | <…> |
| Estado | |

### C-05 — *By sub parts* sin la casilla: tareas planas

| | |
|---|---|
| Entrada | Presupuesto de **tres niveles** confirmado → Crear proyecto → *By sub parts* |
| Aserto | Todas las tareas al mismo nivel, ninguna con padre |
| Capturas | `<…>` |
| Observado | <…> |
| Estado | |

### C-06 — *By sub parts* + la casilla: **dos padres, no cuatro**

| | |
|---|---|
| Entrada | Presupuesto de **tres niveles** confirmado → Crear proyecto → *By sub parts* + casilla |
| Aserto | Solo las partidas que se descomponen en subpartidas se vuelven padre; las que cuelgan de la sección de capítulo quedan planas |
| Capturas | `<…>` |
| Observado | <…> |
| Estado | |
| Trampa | Esperar cuatro padres es leer mal el asistente, no un defecto |

### C-07 — La casilla oculta conserva su valor y **surte efecto**

| | |
|---|---|
| Entrada | Asistente → *By parts* + marcar la casilla → cambiar a *By sub parts* (se oculta) → volver a *By parts* |
| Aserto | Tres capturas de la secuencia + el árbol resultante: si la casilla que el usuario ya no veía decide el árbol, es una trampa de interfaz |
| Capturas | `<C-07-1-...png>`, `<C-07-2-...png>`, `<C-07-3-...png>` |
| Observado | <…> |
| Estado | |

### C-08 — Modo **Actualizar** sobre un pedido que ya tiene proyecto

| | |
|---|---|
| Entrada | Pedido con proyecto → botón Crear proyecto de nuevo |
| Aserto | El asistente abre sin campos de plantilla ni nombre y el botón se rotula «Update»; no aparece un segundo proyecto |
| Capturas | `<…>` |
| Observado | <…> |
| Estado | |
| Trampa | Relanzarlo con **otra** opción puede acumular tareas en vez de sustituirlas: contar antes y después |

### D-03 — Una tarea con material **abre sin error de dominio**

| | |
|---|---|
| Entrada | Proyecto → Tareas → abrir una tarea con material |
| Aserto | El formulario pinta y muestra sus botones inteligentes; no hay pantalla de error |
| Capturas | `<…>` |
| Observado | <…> |
| Estado | |

---

## Capturas añadidas a caminos de canal `api`

Caminos que se asertan por API pero arrastran algo que hay que ver. La captura es
evidencia de apoyo; el veredicto lo da el flujo.

| Camino | Qué había que ver | Captura | Observado |
|---|---|---|---|
| A-02 | La ficha de producto sin botón de existencias | `<…>` | <…> |
| A-10 | Dónde queda la línea previa dentro del árbol importado | `<…>` | <…> |
| B-09 | Cuántas veces sale la frase en el diálogo de validación | `<…>` | <…> |
| D-02 | El desplegable de producto abierto | `<…>` | <…> |
| E-01 | El diálogo de mediciones y sus dos columnas de porcentaje | `<…>` | <…> |
| E-02 | Línea e importe junto a la cabecera a cero | `<…>` | <…> |
| E-04 | El aviso de exceso en la cabecera | `<…>` | <…> |
| F-02 | Si la factura conserva secciones y subtotales | `<…>` | <…> |
| F-06 / F-07 | Dónde cae la línea negativa y qué subtotal deja | `<…>` | <…> |
| F-20 / F-21 | Ídem, en segundo periodo | `<…>` | <…> |
| F-23 | La factura completa, si sale en negativo | `<…>` | <…> |
| G-01 | El bloque de costes desplegado en la línea | `<…>` | <…> |
| G-02 | El pie del presupuesto con el resumen por categoría | `<…>` | <…> |

---

## Incidencias del piloto

Lo que le pasó al navegador, no al vertical. Se anota para que la próxima
ejecución no repita el gasto.

| Momento | Qué pasó | Qué se hizo |
|---|---|---|
| <…> | <p. ej. dos capturas seguidas en blanco> | <se paró y se recargó, según la regla de los 3 fallos> |

Recordatorios: la navegación por fragmento tarda y una captura temprana sale en
blanco; un diálogo abierto tapa lo que hay debajo; la ausencia de un aviso se
aserta con un selector, no con una imagen.

---

## Lo que sigue necesitando ojos humanos

El piloto deja la imagen; **el juicio no lo puede emitir**. Estas preguntas se
entregan con la captura delante y **a una persona nombrada**.

| # | Pregunta | Evidencia | Se la entrega a | Respuesta |
|---|---|---|---|---|
| H-1 | Un capítulo de la factura con subtotal negativo, ¿es presentable ante el cliente de la obra? | `<F-06 / F-20>` | <nombre> | <…> |
| H-2 | La línea previa intercalada dentro de una partida descompuesta, ¿rompe la lectura del presupuesto? | `<A-10>` | <nombre> | <…> |
| H-3 | Dos columnas contiguas con la misma magnitud en unidades distintas (`100%` y `1,00`), ¿induce a error a un jefe de obra? | `<E-01>` | <nombre> | <…> |
| H-4 | Mezcla de idiomas y divisa antigua en las listas, ¿es aceptable para entregar la implantación? | `<…>` | <nombre> | <…> |

Sin respuesta y sin nombre, estas cuatro filas quedan abiertas en el parte: no se
dan por buenas por omisión.
