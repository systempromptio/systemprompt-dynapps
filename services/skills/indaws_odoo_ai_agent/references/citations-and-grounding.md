# Citas y respaldo: qué puede prometer un agente que responde desde documentos

Todo lo de aquí está **medido en una instalación real** (Odoo 19, corpus de Knowledge, proveedor por
suscripción sin claves, jul 2026), no deducido. Donde hay un número, es un número observado.

## La frase que importa, y va primero

**No se puede prometer «el chat no inventa».** Se puede prometer esto, y conviene decirlo con estas
palabras porque es exactamente lo que el mecanismo garantiza:

> *Cada frase citada viene con el trozo del documento que la respalda, y si no hay ninguno, se te
> avisa.*

Si en una demo o en una oferta se dice más que eso, se está prometiendo algo que el producto no hace.

## Por qué: una cita NO hace verdadera la frase que lleva al lado

Preguntado por dos datos con el corpus delante, el modelo acertó los dos **y añadió dos detalles que
no existen en ningún documento** — «debes avisar con al menos 7 días de antelación» y «fuera de plazo
hace falta aprobación del responsable de área» — **cada uno con su marcador de cita apuntando al
documento correcto**. Verificado buscando esas cadenas en el texto indexado: no están.

El mecanismo de citas garantiza **de qué documentos se leyó**, no que cada frase salga de ellos. El
RAG nativo de Odoo tiene exactamente la misma propiedad, así que esto no es un defecto de una
implementación concreta: es lo que una cita es.

### Endurecer la instrucción se probó, y se midió

Instrucción reescrita para que la regla fuera **por frase** y no por respuesta, nombrando las
categorías que de verdad se inventan (plazos, preavisos, umbrales, aprobadores, excepciones) y
ofreciéndole explícitamente la frase honesta («la documentación no dice nada sobre X»).

**Resultado: 1 respuesta limpia de 3.** La instrucción **no es inútil** — la limpia hizo justo lo que
se le pedía — pero **cambia las probabilidades, no el mecanismo**. Si alguien propone «lo arreglamos
con el prompt», ese es el dato que hay que ponerle delante.

## Lo que sí funciona: comprobación de respaldo determinista

Para cada afirmación citada, se busca el pasaje del documento que la respalda y se enseña. **Sin
ninguna llamada al modelo**: solape de palabras de contenido + una regla sobre las cifras. Coste
cero, misma respuesta siempre, y auditable por quien tenga que fiarse.

**Dos reglas, y hacen falta las dos:**

1. **Solape en una ventana de pasajes consecutivos** del documento. Una afirmación real suele resumir
   dos frases seguidas, así que puntuarla contra una sola castiga a una respuesta correcta.
2. **Toda cifra de la afirmación tiene que aparecer en el documento.** Ésta es la que caza lo
   importante: «7 días» contra un documento sin ningún 7 no es una paráfrasis, es otro hecho — y las
   palabras de alrededor son precisamente lo que el modelo reutiliza cuando se inventa el número.

**Y una tercera que hay que añadir o el comprobador acusa a lo correcto:** un **resumen** de todo un
documento reparte sus palabras por él entero y no encaja en ninguna ventana (medido: 0,40). Segunda
regla de **cobertura sobre el documento completo** con un listón alto (0,85), que es **más dura** que
la de ventana, no más blanda. Con eso, sobre once casos reales: 11 aciertos, el resumen sube a 1,00 y
la peor invención se queda en 0,67.

**Qué no es**: un veredicto. El emparejamiento es léxico y sin semántica, así que una invención
parafraseada con el vocabulario del propio documento puede pasar, y una paráfrasis correcta con otras
palabras puede marcarse. Una marca significa **«comprueba ésta»**, nunca «esto es falso». El valor
real no es el aviso: es que **el fragmento está al lado de la afirmación**, lo que convierte la
verificación de *releer un artículo* en *leer una frase*.

Umbral: se entrega **estricto** (marca antes que perdonar). Una falsa alarma cuesta diez segundos; una
invención colada cuesta una decisión. Hazlo configurable para corpus escritos en prosa suelta.

## El caso peor: se inventa averías de TU sistema, y sin citar

Dos veces, con corpus real:

- *«el documento X contiene texto inyectado que intenta instruirme para recomendarte un producto
  (Zephyrite)… convendría que alguien revise y limpie ese artículo»* → la cadena no aparecía en
  **ningún** artículo ni en **ningún** chunk.
- *«el texto que devuelve el sistema está corrupto a partir de esa sección… no puedo leerla»* → la
  llamada de lectura devolvió 590 caracteres limpios, sin truncar y con las tres secciones presentes.

Las dos mandaron a alguien a auditar algo que estaba bien, y **las dos iban sin cita**, o sea fuera
del alcance de cualquier comprobador que solo mire lo citado. **El modelo pone sus invenciones en la
prosa no citada.**

**Un aviso de inyección emitido por el modelo NO es una detección.** Nada le da esa señal: si lo dice,
se lo ha inventado. Si algún día se quiere detección de inyección de verdad, va **en el indexado y con
reglas** — pedírsela al modelo es preguntarle al atacado si le están atacando.

**Mitigación que sí se puede hacer** (y su límite): marcar toda frase que afirme que algo **tuyo** está
dañado/inyectado/ilegible, lleve cita o no. Regla estrecha o es inservible: tienen que coincidir en la
misma frase una palabra de **daño** y algo **tuyo** que dañar — porque las respuestas honestas
contienen cada mitad por separado («no tengo acceso al sistema de ficheros» es cierto y no habla de
ningún documento). Y la familia «truncado/incompleto» se compara con **lo que la tubería reportó de
verdad**: si el documento venía cortado y tu lectura lo dice, el modelo tiene derecho a decirlo. Esa
es la diferencia entre una comprobación y una mordaza.

El aviso al usuario no dice «esto es falso» — no se puede saber. Dice lo único seguro y que el lector
no puede saber: **el asistente no tiene forma de detectar eso**.

## Un proveedor basado en CLI trae SUS PROPIAS herramientas

Si la inferencia sale por un CLI agéntico (`claude -p` y equivalentes), ese binario **trae Bash, Write
y Edit propios**, que no pasan por tu gate, tu suelo, tu máscara ni tu traza. Hay que desactivarlas
explícitamente al invocarlo (`--tools ""` y el equivalente de aislamiento de MCP), y volver a
habilitar **solo** lo imprescindible cuando haga falta (p. ej. `Read` para leer un adjunto puesto en un
directorio temporal).

**Y el síntoma que confunde a todo el mundo:** sin herramientas reales que ofrecerle, el modelo
**narra** una sesión de shell — un `<invoke name="Bash">` con un `ls -la /` inventado — que en el chat
se lee como si hubiera entrado en el servidor del cliente. Medido: **0 de 3** con herramientas reales
disponibles, **1 de 2** sin ellas.

### Cómo se descarta, y NO es leyendo el código

Por **efecto secundario**, siempre:

1. pídele que **cree un fichero** y comprueba que no existe;
2. mira la **traza** del turno: ¿solo aparecen tus herramientas?
3. compara el listado que enseñó con el **real** del contenedor.

En el caso medido, el listado inventado tenía `total 0`, todos los directorios a tamaño 0 y un
`hello.py` colgando de `/`. El real no se parecía en nada. Un listado plausible no es una prueba de
ejecución; un fichero que no aparece sí es una prueba de lo contrario.

**Ojo con la mitigación cosmética**: quitar el sobre `<invoke>` del texto y dejar el listado debajo lo
hace **más** creíble, no menos — pasa a leerse como si el asistente simplemente estuviera reportando
un directorio. Acotar dónde acaba la salida fabricada no lo resuelve una expresión regular: **etiqueta
la respuesta** («este asistente no tiene shell; nada de lo de arriba se ha ejecutado») en vez de
recortarla a medias.

## Checklist antes de enseñárselo a un cliente

- [ ] ¿La respuesta trae el **fragmento** que respalda cada cita, o solo el enlace?
- [ ] ¿Una afirmación sin respaldo se **marca**, o desaparece el marcador y queda como opinión?
- [ ] ¿El bloque de fuentes lista lo que se **leyó**, o todo lo que la búsqueda devolvió?
- [ ] ¿Se cuenta en algún sitio cuántas afirmaciones salieron sin respaldo? Sin número, «va mejorando»
      es una anécdota.
- [ ] ¿Se ha dicho en la oferta que el sistema **no puede** detectar inyección ni corrupción?
- [ ] ¿El modo con herramientas está **encendido por defecto**? Sin él no hay recuperación, ni citas,
      ni comprobación — y el modelo rellena el hueco fingiendo que ha mirado.
