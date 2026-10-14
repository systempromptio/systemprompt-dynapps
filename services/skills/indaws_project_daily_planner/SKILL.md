---
name: indaws-project-daily-planner
description: >-
  Revisión diaria de UN proyecto de implantación Odoo: lee las comprobaciones de la doctrina en
  Odoo Knowledge (nunca las lleva dentro) y las ejecuta cruzando Odoo con la carpeta del proyecto y
  con el calendario y el correo del cliente, para devolver en el chat el plan del día del consultor
  con propuestas numeradas que él acepta en bloque. Nada se escribe sin aceptación. Guarda estado
  entre días, así que sabe cuántos días lleva abierta una recomendación y qué se comprometió a
  hacer. Usar cuando pidan la revisión de un proyecto, "cómo va X", "qué hago hoy en X", cuando el
  consultor acepte propuestas de una guía anterior ("aplica la 2 y la 4"), o cuando dispare la tarea
  programada de la mañana. También para dar de alta o de baja el seguimiento diario de un proyecto.
  Un proyecto por ejecución. Sustituye a consultant-daily-planner.
version: 1.3.0
effort: medium
allowed-tools: [Read, Write, Bash, Glob, Grep]
---

# project-daily-planner

La revisión diaria de **un** proyecto de implantación Odoo. No produce un diagnóstico: produce el plan del día del consultor y una lista de propuestas que él acepta o descarta.

Esta skill es **solo la mecánica y sus barreras**. Qué se comprueba, con qué umbral, qué se propone y cómo se redacta cada línea vive en el artículo de doctrina y se lee en cada ejecución. Aquí no hay ni un umbral ni una comprobación: si añades una, la has puesto en el sitio equivocado.

## Crítico — lo que nunca hace

Estas barreras **no dependen de la doctrina**: la doctrina puede cambiar qué se comprueba, no enseñar una acción nueva. El catálogo de acciones es cerrado.

- **NUNCA** escribe nada en Odoo sin aceptación explícita: ni fechas, ni alcance, ni facturación, ni borradores de documentación.
- **NUNCA** mueve la fecha de salida a producción dentro de una aceptación en bloque. Se confirma aparte.
- **NUNCA** escribe en una rama de documentación visible para el cliente. La aceptación autoriza a escribir, no a publicar.
- **NUNCA** envía correo ni mensajes al cliente, ni contesta un hilo. Redacta el borrador y lo propone.
- **NUNCA** abre correo fuera de la frontera: contactos y dominios del cliente de este proyecto, más internos que nombren el proyecto, últimos 7 días. No se ensancha sobre la marcha.
- **NUNCA** escribe ni mueve nada en la carpeta del proyecto. Solo lee.
- **NUNCA** borra registros, ni borra y recrea una página: actualizar es editar su cuerpo.
- **NUNCA** ejecuta otra skill ni recalcula la salud: lo propone, con sus ids.

## El artículo de doctrina

| | |
| --- | --- |
| Título | **Revisión diaria de un proyecto** (EN: *Daily project review*) |
| Ruta | `como-trabajamos/metodologia/revision-diaria.md` |
| Documentación | **Consultoría**, bajo Metodología |

**Se localiza por esos datos, nunca por un id**: los ids cambian entre instancias. Las reglas de forma de la guía (identificadores nunca solos, «checklist» y no «paso», cada propuesta se entiende sin abrir Odoo) están en el artículo, en *Cómo se escribe un hallazgo*, y se aplican tal cual.

## Las cuatro fuentes, y no valen lo mismo

- **Doctrina** y **Odoo**: sin cualquiera de las dos, **no hay pasada**. Se para y se dice.
- **Carpeta del proyecto** (el directorio desde el que se ejecuta), **calendario** y **correo**: si faltan, la pasada sigue y la cabecera dice qué bloques no ha podido comprobar.

## Paso 0 — Preflight

1. **Doctrina.** Identifica el conector de documentación por su familia `docs_*`, nunca por nombre; cero o varios candidatos → **[STOP]** pregunta. Resuelve *Consultoría* con `docs_list_spaces`, busca con `docs_search` en lenguaje natural y **elige por título y ruta, no por puntuación** (gotcha 1). Léelo entero con `docs_get` y verifica que trae la tabla de comprobaciones `D1`, `D2`… con su nombre corto. Si no la trae, o el artículo no aparece → **[STOP]** y dilo. Si su ruta o título han cambiado, es un hallazgo: no busques el que más se parezca. Nunca reconstruyas comprobaciones de memoria.
2. **Odoo.** Según `references/shared/odoo-connection.md`: MCP en sesión interactiva contra la corporativa, `odoo-pilot` en headless o instancia de cliente. Sin conexión → **[STOP]**.
3. **Proyecto y carpeta.** Resuelve por id, o por nombre si es inequívoco; varios → **[STOP]** pregunta. Lista la carpeta y anota qué hay. Si está vacía o no es la del proyecto, sigue sin ella y dilo.
4. **Calendario y correo.** Conector de Microsoft 365 por su familia de herramientas. Si no está, sigue y la cabecera lo dice.

## Paso 1 — Cargar el estado

Carpeta, por precedencia: `$PDP_DATA_DIR` → `.project-daily-planner/` en el directorio actual → `$HOME/dynapps-informes/project-daily-planner/`. Un subdirectorio por id de proyecto. Ficheros y formato en `references/estado.md`: `baseline.json` (nunca se sobrescribe), `history.json`, `proposals.json`, `commitments.json`, `dismissals.json`, `schedule.json`.

Primera ejecución: crea el baseline y dilo, para que nadie lea la ausencia de deriva como buena señal.

## Paso 2 — Leer, y decir de dónde se lee

De Odoo, lo que pida la doctrina. De la carpeta, qué ficheros hay y cuáles se tocaron después de la última actividad del proyecto; lo que no se entienda se nombra sin sacar conclusiones. Del calendario, reuniones con el cliente de los últimos 7 días y los próximos 3 hábiles. Del correo, solo lo que cae dentro de la frontera; los contactos del cliente salen del partner del proyecto en Odoo, nunca de una lista escrita a mano.

## Paso 3 — Ejecutar las comprobaciones

Una por una, con el identificador y el nombre corto de la doctrina, cruzando con el estado: cuántos días lleva abierta, si un compromiso aceptado ya venció, y si está descartada (entonces no se muestra). Una comprobación que no se puede ejecutar por falta de dato o de fuente **se reporta como tal**; callarla es mentir por omisión.

## Paso 4 — Escribir la guía

En el chat, corta, en este orden:

1. **Cabecera**: proyecto, artículo raíz de documentación leído con su nombre y su id, fuentes disponibles, día N de M, métricas de avance.
2. **Hoy**: tres o cuatro cosas en orden, cada una con una frase de por qué y un tiempo estimado.
3. **Propuestas**: numeradas, redactadas según las reglas de forma de la doctrina.
4. **Al margen**: lo que no es de hoy pero conviene saber.
5. **Pie**: artículos de doctrina leídos y su fecha de última modificación.

Si nada ha cambiado desde ayer, se dice en una línea. Cuando se propone un borrador (una ficha, un acta), va **redactado dentro de la propuesta**, citando de dónde sale cada afirmación y dejando en blanco lo que no consta.

Antes de terminar, **guarda las propuestas numeradas en `proposals.json`** con su acción y sus registros. Sin eso, un "aplica la 2" en otra sesión no tiene a qué referirse. **[STOP]** — espera la respuesta del consultor.

## Paso 5 — Aceptación

El consultor responde ("aplica la 2 y la 4", "la 3 no aplica porque…"), en el mismo hilo o en otra sesión desde la misma carpeta.

- Resuelve los números contra el `proposals.json` **de la última guía**. Si es de un día anterior al de hoy, **[STOP]**: dilo y ofrece rehacer la pasada, porque los números pueden haber cambiado.
- Aplica solo lo aceptado y solo lo que esté en el catálogo cerrado. Un borrador va a una rama oculta al cliente.
- Un descarte exige motivo escrito; sin motivo, pregunta.
- Registra lo aceptado en `commitments.json` y lo descartado en `dismissals.json`, y confirma lo escrito registro a registro, con nombre e id.

## Paso 6 — Alta y baja del seguimiento

**El seguimiento lo da de alta el propio consultor, ejecutando esta skill desde la carpeta del proyecto.** La tarea programada corre en la carpeta en la que se creó; si se crea desde otra, cada mañana correrá sin carpeta ni estado y no servirá. Procedimiento completo en `references/seguimiento-programado.md`.

- **Alta**: verifica que el directorio actual es la carpeta del proyecto (**[STOP]** si no se puede afirmar), comprueba que los conectores responden, crea la tarea programada diaria (día laborable por la mañana) y anota su id en `schedule.json`.
- **Baja**: la retira y lo anota. Cuando el proyecto pasa a una etapa terminal, la última guía lo avisa y propone la baja.

## ALWAYS

- Un proyecto por ejecución. Nunca mezcles datos de otro proyecto en la misma guía.
- Di en la cabecera qué fuentes has tenido: una pasada sin correo no encuentra lo mismo que una con correo.
- Consume la salud, el tablero y el informe semanal de quien los produce. No los rehagas.
- Si la doctrina y lo que ves en Odoo se contradicen, **es un hallazgo**: nombra los dos y sigue la doctrina mientras alguien lo resuelve.

## GOTCHAS

Los que cuestan un diagnóstico falso nada más empezar. El resto, en `references/gotchas.md`.

1. **La búsqueda de la doctrina no devuelve primero la doctrina.** El índice de Metodología puntúa por encima del artículo (medido el 29/09/2026: 0,964 frente a 0,963). Elige por título y ruta, y verifica el contenido.
2. **Un proyecto puede tener dos árboles de Knowledge**, y el campo apunta a uno. Si es el viejo, la documentación parece abandonada cuando está completa. Por eso el artículo raíz va en la primera línea.
3. **Una aceptación sin `proposals.json` es una adivinanza.** La pasada programada corre sola y el consultor contesta horas después en otra sesión. Si las propuestas no se guardaron, no se aplica nada.
4. **Una tarea programada creada fuera de la carpeta del proyecto corre sin carpeta y sin estado.** No falla: produce cada día una guía sin memoria y sin el bloque de la carpeta.
5. **Los nombres de los conectores cambian por instalación.** Identifícalos por su familia de herramientas; nunca los escribas fijos.

## Delegates to

| Skill | Suite | Cuándo |
| --- | --- | --- |
| `odoo-pilot` | common | Conexión en headless, tarea programada o instancia pre-v19 |
| `dynbrain` | common | Cómo se localiza y se lee la doctrina en Knowledge |
| `project-health-monitor` | consultant | Se propone cuando hace falta recalcular la salud |
| `project-weekly-update` | consultant | Se propone cuando la doctrina lo indica; no se ejecuta sola |
| `project-meeting-minutes` | consultant | Se propone cuando falta el acta de una reunión |
| `odoo-configurator` | consultant | Se propone para ejecutar y documentar checklists |

## Consumed by

La tarea programada diaria de cada proyecto, que corre desde la carpeta de ese proyecto. Y cualquier resumen que agregue lo urgente de varios proyectos de un consultor, que lee las guías ya producidas en vez de volver a Odoo.

## REFERENCES

- `references/estado.md` — ficheros de estado, formato y retención
- `references/seguimiento-programado.md` — alta y baja de la tarea programada desde la carpeta del proyecto
- `references/gotchas.md` — gotchas de datos, fuentes y conectores
