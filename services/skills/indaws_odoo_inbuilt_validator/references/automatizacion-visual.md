# Automatizar el canal visual — dos vías y sus trampas

Lo que en la primera versión de esta skill era un checklist para una persona lo
ejecuta ahora la skill misma, pilotando un navegador. Esto es lo aprendido
haciéndolo.

---

## Las dos vías NO son intercambiables

Esto es lo primero que hay que entender, porque la primera versión de esta skill
lo daba a elegir como si fuera un ajuste, y no lo es: **cada vía tiene un
conductor distinto**.

| | **Extensión de Chrome** | **Playwright + Firefox** |
|---|---|---|
| **Quién conduce** | **Claude, con sus propias herramientas de navegador.** No hay script | **Un script**: `scripts/recorrido_visual.py` |
| Cómo se invoca | Herramientas `mcp__claude-in-chrome__*` desde la conversación | `python3 scripts/recorrido_visual.py ...` por Bash |
| Si no está disponible | No hay alternativa: se cambia a Playwright | Basta con instalar Playwright |

**Consecuencia práctica:** `recorrido_visual.py` **no sabe pilotar Chrome y nunca
lo sabrá.** Un script de Python no puede conducir la extensión; la extensión la
maneja Claude. Quien elija la vía Chrome no ejecuta ningún script: sigue el
procedimiento de más abajo con sus herramientas.

**Y una comprobación previa que no se puede saltar:** las herramientas de
navegador **pueden no existir** en la sesión —en una ejecución desatendida, por
cron o headless, no están—. Si no están, la vía Chrome no es una opción aunque el
usuario la prefiera: se le dice y se pasa a Playwright.

---

## Procedimiento de la vía Chrome, paso a paso

No hay script, así que el procedimiento **es** la implementación:

1. `list_connected_browsers` — enumera los navegadores conectados a la cuenta.
   **Si devuelve lista vacía, la vía no es viable**: pide al usuario que abra
   Chrome con la extensión y la conecte, o cambia a Playwright.
2. **Pregunta cuál usar** con `AskUserQuestion`, listando cada uno con su nombre y
   su identificador. Nunca elegir por cuenta propia: el usuario puede tener
   abierta la producción de otro cliente en la ventana de al lado.
3. `select_browser` con el identificador que haya elegido.
4. `tabs_context_mcp` con `createIfEmpty` y luego `tabs_create_mcp`: **pestaña
   propia**, no reutilizar una del usuario.
5. `navigate` a `<url>/web` — en 17.0 la entrada es `/web`, no `/odoo`.
6. **Pide al usuario que inicie sesión** y espera. Esta skill **no teclea
   contraseñas en formularios**, y menos en el navegador de otra persona.
7. Recorre con `computer` (clic, escritura, captura), `find` para localizar
   elementos por descripción y `read_page` cuando haga falta el árbol.
   `browser_batch` agrupa varias acciones en una llamada.
8. Captura con `computer` y `save_to_disk`, y **copia la imagen al directorio de
   evidencias con el id del camino en el nombre**: si se queda en el temporal, no
   es evidencia de nada.
9. Cierra las pestañas que hayas abierto al terminar.

**Trampas propias de esta vía**, sufridas:

- El renderizador **se congela**: la captura agota su tiempo de espera. Tras dos o
  tres fallos seguidos, para y pide al usuario que recargue. Insistir no lo
  arregla.
- Una ruta con fragmento (`#id=...`) que apunte a un registro **borrado** deja el
  cliente web en un estado del que no sale navegando: hay que recargar limpio.
- Las capturas se guardan primero en un temporal del sistema; el paso de copiarlas
  al directorio de evidencias **es parte del recorrido**, no un extra.

---

## Cuándo elegir cada una


| | **Extensión de Chrome** | **Playwright + Firefox** |
|---|---|---|
| Qué pilota | El navegador **real del usuario**, con su sesión ya abierta | Un navegador propio, arrancado por el script |
| Necesita a alguien delante | **Sí** | No |
| Inicio de sesión | Lo hace el usuario, a mano | Lo puede hacer el script con credenciales del entorno |
| Ve lo que ve el cliente | Sí: sus extensiones, su zoom, su idioma, su tamaño de ventana | Solo lo que se configure |
| Headless | No | Sí, configurable |
| **Cuándo usarla** | Hay alguien acompañando la validación y se quiere reproducir *su* entorno, o la instancia tiene un acceso que el script no sabe atravesar (SSO, doble factor, VPN con portal) | **Por defecto**: recorridos largos, desatendidos, o repetidos |

### Vía Chrome — reglas de uso

1. **Listar los navegadores conectados y PREGUNTAR cuál usar.** Nunca elegir uno
   por su cuenta: el usuario puede tener abierta la producción de otro cliente en
   la ventana de al lado.
2. **Si no hay ninguno conectado, no se puede.** No es un fallo que se resuelva
   reintentando: hay que pedir al usuario que conecte el navegador, o cambiar a
   Playwright.
3. **El inicio de sesión lo hace el usuario.** La skill **no teclea contraseñas
   en formularios de una sesión ajena**: se pide al usuario que entre y luego se
   conduce la sesión ya abierta.
4. Antes de tocar nada, confirmar en qué **entorno** se está. Los caminos que
   escriben no se recorren sobre producción.

### Vía Playwright — reglas de uso

1. Credenciales **del entorno o del `config.json`**, nunca en el código.
2. `headless` configurable: `true` por defecto para recorridos desatendidos,
   `false` cuando se está depurando un selector.
3. Es la vía por defecto **cuando no hay nadie delante**.
4. Firefox: menos ruido de extensiones y perfiles que un Chrome instalado, y el
   pintado de Odoo no cambia de forma apreciable.

---

## Trampas que cuestan tiempo

### El renderizador se puede congelar

Pasa: la pestaña deja de repintar y todas las capturas siguientes salen iguales,
en blanco o a medio pintar. **Si fallan dos o tres capturas seguidas, parar y
pedir una recarga** — insistir solo produce un montón de imágenes inútiles con
nombre de camino, que es peor que no tener ninguna.

`scripts/recorrido_visual.py` lleva ese corte dentro: `--max-fallos` (3 por
defecto) aborta el recorrido y lo dice.

### Hay que esperar a que Odoo pinte

La navegación de Odoo va por **fragmento** (`#action=…&id=…`): la URL cambia al
instante y el DOM tarda. Una captura tomada nada más navegar sale en blanco o con
la pantalla anterior.

Lo que funciona, en este orden:

1. Esperar al selector del contenedor de la acción (`.o_action_manager`,
   `.o_content`).
2. Esperar a que **desaparezcan** los indicadores de carga
   (`.o_loading_indicator`, `.o_blockUI`).
3. Una pausa corta y fija al final: la animación de apertura de un diálogo no
   está cubierta por ningún selector.

Y siempre **volver a `/web#home` entre camino y camino**: dejar la sesión en la
pantalla anterior es la primera causa de captura equivocada.

### En 17.0 la entrada es `/web`, no `/odoo`

`/odoo` es de la 18.0 en adelante. En una 17.0 devuelve 404 y el piloto se queda
mirando una página de error que, en miniatura, parece un Odoo cargando.

### Los diálogos tapan lo que hay que ver

Un `.modal` abierto se lleva el foco y oscurece el fondo. Para las capturas de
subtotales o de árboles de tareas, cerrar el diálogo antes; para las de mensajes
de error, capturar **con** el diálogo, que es justo la evidencia.

### La ausencia también se aserta

El camino A-12 comprueba que **no** sale ningún mensaje. Eso no se ve en una
captura por sí sola: hay que **aserta que el selector no existe**
(`.o_notification`, `.modal`) y adjuntar la captura como respaldo. Una imagen sin
aserto no prueba una ausencia.

### Adjuntar ficheros

El campo binario de un asistente de Odoo es un `input[type=file]` oculto detrás
de un botón. Se rellena por el `input`, no por el botón, y **hay que esperar a
que el nombre del fichero aparezca en el widget** antes de pulsar Importar: si no,
se envía el asistente con el campo vacío y sale el camino negativo en lugar del
positivo.

---

## Las capturas son la evidencia

- **Una captura por camino, con el id del camino en el nombre**:
  `A-12-asistente-sin-aviso.png`. El id delante, siempre: es lo que permite
  cruzarlas con `resultados.yaml` sin abrirlas.
- Guardarlas **en disco**, en el directorio de evidencias del recorrido. El gate
  de cobertura comprueba que cada evidencia declarada existe y no está vacía.
- Cuando un camino necesita varias (C-07 necesita tres), numerarlas dentro del
  camino: `C-07-1-casilla-marcada.png`, `C-07-2-oculta.png`, …
- Nombre en minúsculas, sin espacios ni acentos.

## Lo que la automatización sigue sin poder hacer

Emitir un **juicio**: si un capítulo en negativo es presentable ante el cliente,
si una mezcla de idiomas es aceptable para entregar. El piloto deja la imagen; la
respuesta la da una persona. Son los cuatro juicios de canal `humano` del
catálogo, y van al final de `templates/recorrido-visual.md`.
