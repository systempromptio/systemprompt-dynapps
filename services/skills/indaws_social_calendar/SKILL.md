---
name: indaws-social-calendar
version: 1.0.0
effort: medium
model: sonnet
description: >
  Calendario editorial mensual de Dynapps España para LinkedIn, YouTube, blog, newsletter y
  eventos. Propone los huecos de la semana siguiente y los temas del mes siguiente contra los
  cinco bloques fijos —podcast, caso de éxito, campaña, evento y producto Odoo—, cada uno
  recorriendo el embudo entero como escalera, y los deja en estado `propuesto` para que Álvaro
  apruebe. Nunca publica. Usar cuando pidan "calendario de redes", "monta el calendario",
  "qué publicamos la semana que viene", "temas del mes", "propuesta de calendario", "revisa
  el rendimiento de lo publicado", o cuando disparen las tareas programadas del jueves y del
  día 25.
---

# Calendario editorial · Dynapps España

El calendario se llena con **cinco bloques fijos** y **cada bloque es una escalera** que siembra,
profundiza y pide. La doctrina que fija todo eso vive en Odoo Knowledge y **manda sobre esta
skill**: `como-trabajamos/el-calendario-mensual-de-contenido-y-sus-cinco-bloques.md` en el
space 12. Si esta skill y ese documento se contradicen, gana el documento y hay que decirlo.

**No publica nada.** Ni en LinkedIn, ni en YouTube, ni en Odoo. Produce propuestas que una
persona aprueba, y el estado `publicado` lo marca quien publica.

## Quién es quién

| Persona | Papel |
| :--- | :--- |
| **Álvaro** | Dueño del área. Aprueba el calendario y lo que sale con la marca. Decide casos de éxito y eventos con presupuesto y dirección |
| **Elena** | Produce, junto con Álvaro |
| **Víctor** | Presentador del podcast. No aprueba |

Marketing es **independiente del CRM**: ni el SDR ni los comerciales tienen veto sobre lo que
se publica.

## Modo setup: la primera vez, en la máquina de su dueño

Si el calendario todavía no está montado en esta máquina —no hay tareas programadas de
`social-calendar`, o no hay enlace a la página— **el modo setup va primero**, y está en
`references/setup.md`. Se dispara con «setup del calendario», «monta mi calendario» o
«configura social-calendar».

Lo que hay que saber antes de empezarlo: **las tareas programadas pertenecen a la cuenta y a la
máquina donde se crean**, y corren solo mientras esa persona tiene la aplicación abierta. Si el
setup se ejecuta en la máquina de quien no lleva el calendario, el calendario sigue dependiendo
de esa persona. El setup lo comprueba y para si es el caso.

## Los dos disparos

### Pasada semanal — jueves

1. **Lee el estado real** del calendario antes de proponer nada: qué se publicó, qué se quedó
   atascado, qué se descartó y por qué. Nunca se empieza de cero.
2. **Mide lo publicado**, con los desfases de `references/medicion.md`. Nada de LinkedIn de los
   últimos siete días se presenta como dato.
3. **Descubre material** según `references/fuentes.md`.
4. **Compone la semana siguiente** anclada a un bloque, con el mix de `references/embudo.md`.
5. **Valida** contra los guardarraíles de abajo. Un hueco que falla no se propone: se deja
   bloqueado con el motivo escrito.
6. **Escribe los huecos** en estado `propuesto` y avisa a Álvaro con el enlace.

### Pasada mensual — día 25

1. **Comprueba que el mes siguiente tiene objetivo** en demos realizadas y registros a evento,
   con cifra y plazo. **Si no lo tiene, no propone temas**: pide el objetivo y para. Es la misma
   regla que bloquea una campaña sin objetivo medible.
2. Reparte los cinco bloques en las semanas del mes y elige público por bloque entre los cuatro
   perfiles.
3. Calcula el recorte del mes con el criterio de `references/embudo.md` y lo enseña, con nombre
   y apellidos de qué pieza se cae.
4. Fija las fechas de revisión a 90 y 180 días.

## Guardarraíles que bloquean un hueco

Ninguno es negociable y todos son mecánicos, porque el aprobador también produce y la
aprobación no es un control independiente:

- **Sin UTM** escrita y bien formada — ver `references/utm.md`.
- **Más de una llamada a la acción**, o ninguna.
- **Emojis o hashtags** en el texto.
- **«Usted»** en cualquier forma.
- **Sin consentimiento de imagen** del cliente, en cualquier pieza donde aparezca un cliente,
  su marca o su gente. No lo detecta ninguna comprobación automática: es un campo que alguien
  rellena a mano y sin él la pieza no sale.
- **Dos huecos el mismo día**, o menos de dieciocho horas entre dos.
- **Una pieza que pide antes de dos que siembran o profundizan** esa misma semana.
- **Un canal solo**: toda campaña lleva primario y al menos uno de apoyo.
- **Enlace a YouTube en el cuerpo** de una pieza que no sea de las que piden.

## Dónde vive el estado

Un Artifact privado con base de datos, que es la superficie donde Álvaro y Elena trabajan.
Colecciones:

| Colección | Un documento es | Lo que importa |
| :--- | :--- | :--- |
| `slots` | una pieza | fecha, bloque, canal primario, canal de apoyo, etapa de embudo, etapa de registro si es evento, gancho, cuerpo, CTA único, UTM, activo requerido, consentimiento, dueño, estado, fuente |
| `temas` | un mes | objetivo en demos y registros, plazo, público por bloque, reparto de bloques por semana, recorte aplicado |
| `metricas` | una semana | línea base, resultados con su desfase anotado, modelo de atribución declarado |
| `config` | la configuración | bloques activos, mix objetivo, dueños por defecto |

Estados: `propuesto` → `aprobado` → `en produccion` → `programado` → `publicado` → `medido`,
más `descartado` con motivo. **El motivo del descarte es el bucle de aprendizaje**: lo que se
rechaza cada semana es lo que afina el mix del mes siguiente.

## Referencias

| Fichero | Para qué |
| :--- | :--- |
| `references/setup.md` | la puesta en marcha, la primera vez, en la máquina de su dueño |
| `references/embudo.md` | las cinco escaleras, el mix, el criterio de recorte, los dos embudos de un evento |
| `references/canales.md` | matriz de público por canal, límites de SEO y los campos de copy de LinkedIn y YouTube |
| `references/utm.md` | origen, medio y campaña: qué se elige y qué se crea |
| `references/fuentes.md` | de dónde sale el material, con las consultas al barrido de Odoo |
| `references/medicion.md` | qué desfase lleva cada dato y qué no significa lo que parece |

## Lo que no se hace

- **Publicar.** En ningún canal, en ningún estado.
- **Proponer publicidad de pago.** El pago lo lleva la agencia a nivel de grupo.
- **Producir antes de que el calendario esté aprobado.**
- **Crear un correo en Odoo** en cualquier estado que no sea borrador.
- **Crear un origen o un medio de UTM nuevo.** Se eligen del catálogo del grupo.
- **Proponer temas del mes sin objetivo** en demos y registros.
- **Presentar datos de LinkedIn de los últimos siete días** como si fueran de hoy.
- **Dar un informe sin decir el modelo de atribución.**
- **Leer un pico de altas del barrido de Odoo** como crecimiento de mercado.
