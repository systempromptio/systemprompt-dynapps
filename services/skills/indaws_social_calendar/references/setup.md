# Modo setup: dejar el calendario funcionando en la máquina de su dueño

Este modo se ejecuta **una vez, en la máquina de quien va a llevar el calendario** — hoy
Álvaro. No lo ejecuta quien montó la skill: las tareas programadas pertenecen a la cuenta y al
equipo donde se crean, y corren solo mientras esa persona tiene la aplicación abierta. Montadas
en la máquina de otro, el calendario sigue dependiendo de esa otra persona, que es justo lo que
este modo existe para evitar.

**Se dispara** cuando alguien dice «setup del calendario», «monta mi calendario», «configura
social-calendar» o abre la guía de puesta en marcha.

## Paso 0 · Di quién eres y qué va a pasar

Antes de crear nada, enseña la lista de lo que se va a montar y **pide confirmación**. Son
tareas que van a dispararse solas cada semana: nadie debería descubrirlas por sorpresa.

Y comprueba **de quién es esta máquina**. Si quien ejecuta no es el dueño del calendario, dilo
y para: montarlo aquí no resuelve nada.

## Paso 1 · Comprueba los accesos antes de prometer nada

Cuatro, y cada uno con su consecuencia si falta:

| Acceso | Para qué | Si falta |
| :--- | :--- | :--- |
| La página del calendario | leer y escribir los huecos | el resto no sirve: pide el enlace y para |
| El conector de documentación | leer la doctrina del space 12 | avisa: la skill funcionará contra su copia local, que puede estar vieja |
| La base del scraper de Odoo | el bloque de producto | las tres piezas de producto del mes se quedan sin fuente |
| Odoo | leer sectores donde se gana | el ángulo del mes se elige a ciegas |

**Comprueba, no supongas.** Una lectura real de cada uno, y si uno falla se dice cuál y qué se
pierde, en vez de montarlo todo y que reviente un jueves a las nueve.

## Paso 2 · Crea las dos tareas programadas

En la máquina de quien ejecuta, con estas cadencias:

| Tarea | Cuándo | Qué hace |
| :--- | :--- | :--- |
| `social-calendar-weekly` | jueves 09:00 | propone la semana siguiente |
| `social-calendar-monthly` | día 25, 09:00 | propone los temas del mes siguiente |

El prompt de cada una tiene que ser **autosuficiente** —cada ejecución arranca sin memoria de
nada— y llevar dentro: el enlace de la página, las colecciones, la orden de invocar esta skill
primero, y las reglas que no se saltan. Los prompts de referencia están en las tareas que ya
existen; se copian, no se reinventan.

Después de crearlas, **di que corren solo con la aplicación abierta** y recomienda ejecutar la
semanal a mano una vez: aprueba de antemano los permisos que necesita y llena la página con
material real, así la primera mirada no es a un calendario vacío.

## Paso 3 · Las obligaciones que ninguna tarea programada puede cubrir

Estas son de una persona y hay que dejarlas apuntadas donde esa persona las vea — su gestor de
tareas, su calendario, lo que use. **Pregunta dónde** antes de crearlas en ningún sitio.

| Obligación | Cadencia | Por qué no la puede hacer una tarea automática |
| :--- | :--- | :--- |
| Aprobar el calendario de la semana | jueves a lunes | la aprobación es lo que abre la producción, y es un juicio |
| Fijar el objetivo del mes | antes del día 25 | sale de una conversación con dirección, no de un dato |
| Revisión mensual con dirección | mensual | es el único control independiente que existe, porque el aprobador también produce |
| Pedir y registrar el consentimiento de imagen | por cada caso y cada evento | ninguna comprobación lo detecta; sin él la pieza no sale |
| Renovar las campañas de UTM | enero | arrastrar la del año anterior deja el año viejo inflado y el nuevo vacío |
| Revisiones de campaña a 90 y 180 días | por campaña | el ciclo B2B es de 90 a 180 días; medir antes no dice nada |
| Refirmar la doctrina | anual | un documento `decided` caduca al año |

## Paso 4 · Recoge lo que falta y escríbelo

Cuatro decisiones que la skill necesita y que **nadie puede tomar por su dueño**. Se preguntan
de una en una, con una recomendación cada una, y se escriben donde vivan:

1. **El objetivo del mes** en demos realizadas y registros a evento, con plazo → al documento
   `temas/<YYYY-MM>` de la página. Sin él la pasada del día 25 no propone temas.
2. **El suplente de aprobación y el plazo máximo** de una propuesta sin respuesta → a
   `config/calendario`.
3. **De dónde sale la analítica de LinkedIn**: exportación accesible o entrada a mano → a
   `config/calendario`, porque cambia el paso de medición del jueves.
4. **Si los prescriptores llevan carril propio** en el calendario → a `config/calendario`.

## Paso 5 · Firma la doctrina

Los dos documentos del space 12 —el modelo y esta guía— están escritos por un agente, con
`truth_source: decided` y **sin dueño ni fecha de refirma**. Hasta que su dueño los firme son
texto, no doctrina.

Si quien ejecuta el setup es ese dueño y dice que está de acuerdo, se escribe su login en
`owner` y la fecha de hoy en `reviewed_on`, en los dos documentos. **El login, no el correo.**
Si no está de acuerdo con algo, se corrige el documento primero y se firma después — nunca al
revés.

## Paso 6 · Cierra diciendo la verdad

Un resumen de qué quedó montado, qué accesos fallaron y qué decisiones siguen abiertas. Y una
frase que conviene no maquillar: **lo que está guardado en Odoo está pendiente de publicación
al repositorio**, que va con la publicación programada y no es un paso de nadie de aquí.

## Lo que el setup no hace

- **No crea tareas en la cuenta de otra persona.** Se ejecuta en la máquina de su dueño, o no
  se ejecuta.
- **No inventa el objetivo del mes** ni lo deduce del mes anterior.
- **No firma la doctrina en nombre de nadie.**
- **No rellena `owner` ni `reviewed_on` «por rellenar»**: destruye las dos únicas consultas que
  justifican esos campos.
- **No publica** en ningún canal, ni durante el setup ni después.
