# Qué desfase lleva cada dato

Un número mal leído es peor que ninguno: sostiene una decisión equivocada durante meses y nadie
lo cuestiona porque venía con una cifra. El patrón se repite en todos los canales — **la métrica
más fácil de sacar no es la que mide lo que importa**.

La fuente es `como-trabajamos/que-se-mide-y-que-no.md` en el space 12, y manda sobre esta
página.

## LinkedIn: nada de lo que ves es de hoy

- La exportación de analítica lleva **siete días de retraso como mínimo**. Lo de esta semana
  **no se presenta nunca como dato de esta semana**.
- Los seguidores van **entre 24 y 48 horas por detrás**, así que el crecimiento se mira en
  cadencia semanal. Un fotograma diario es ruido, no dato.

## Correo: la apertura ya no sirve

- Métrica primaria: **el clic**. La de calidad: **clic sobre apertura**.
- La apertura está inflada desde iOS 15 por la protección de Apple Mail, que abre correos que
  nadie lee. Un informe que lidera con la apertura mide el comportamiento de un servidor.
- **Entregados = enviados menos rebotados.** La apertura se calcula sobre entregados, nunca
  sobre enviados: mezclarlos infla el número sin que se note.

## Web: usuarios y sesiones no son lo mismo

**Usuarios para alcance, sesiones para volumen.** Intercambiarlos hace que el mismo mes parezca
un 40 % mejor o peor según qué columna se cite.

Y antes de leer una tendencia larga, comprobar **qué dominio está midiendo**: una serie que
apunte al dominio de la marca anterior no es comparable con una que apunte a `www.dynapps.es`.

## Atribución: el contenido de marca sale a cero

Odoo atribuye a **primer contacto** por defecto, así que el contenido que genera conocimiento
pero no cierra la conversión **no recibe ningún crédito**.

De ahí la regla que no es opcional: **en todo informe se dice qué modelo de atribución se está
usando**. Sin esa frase, el informe está diciendo implícitamente que el contenido no sirve para
nada — y eso no es un hallazgo, es el modelo.

Es también el motivo por el que el calendario **se juzga contra demos realizadas y registros a
evento**, no contra pipeline atribuido.

## Estacionalidad

**Agosto y diciembre bajan**, y es normal en el B2B español. Cualquier periodo que los toque se
lee **en interanual**, nunca contra el mes anterior. Presentar la caída de agosto como un
problema es la forma más rápida de que se cancele algo que funcionaba.

## Cuándo se mira el retorno

El ciclo de venta B2B es de **90 a 180 días**. Medir a 30 días siempre sale mal y no dice nada.
Las fechas de revisión se fijan **a 90 y a 180 días, al arrancar**, no cuando alguien pregunta.

## Lo que no se hace

- **Liderar un informe de correo con la tasa de apertura.**
- **Presentar datos de LinkedIn como si fueran de hoy.**
- **Usar usuarios y sesiones como sinónimos.**
- **Dar un informe sin decir el modelo de atribución.**
- **Atribuir la caída de agosto o diciembre a una campaña** sin el interanual delante.
- **Medir el retorno antes de los 90 días.**
- **Meter emojis o hashtags en un informe.** La marca no los usa, y eso vale también hacia
  dentro.
