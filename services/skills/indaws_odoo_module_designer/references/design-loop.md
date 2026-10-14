# Design loop — FORMA → REUSO → DETALLE

El bucle de cierre de diseño. La lección de campo: **el orden evita reprocesos**. En una ejecución real,
la premisa de arquitectura llegó tras ~94 decisiones ya cerradas y forzó reescribir arquitectura, fasing y
scaffold; y el scan de reuso llegó tras el scaffold y obligó a revisar manifests dos veces. No repitas eso.

## 1. FORMA (primero, siempre)

La "forma del producto" condiciona TODO. Cierra antes de cualquier detalle:
- **¿Configurable simple→complejo?** Si sí, decide el mecanismo:
  - **Módulos por check `module_*`** (recomendado para "instala lo que usas"): cada capacidad es un módulo;
    el check en `res.config.settings` lo instala/desinstala. Verifica el patrón a fuente.
  - **Gating por grupos** dentro de un módulo (más simple, menos aislable).
- **Granularidad** (si módulos): un módulo por feature es el techo; **fusiona lo que va SIEMPRE junto**
  (dep dura + mismo concepto). Mantén separado lo que un cliente activaría por su cuenta o lo que depende de
  un módulo externo opcional (glue). No hagas 10 módulos si 5 cubren las mismas activaciones reales.
- **Multi-company, portabilidad** (¿depende de la suite del cliente o del Odoo estándar? afecta deps y a si
  el módulo es vendible fuera del parque).

Salida: **mapa de módulos** (base + capacidades + bridges) + cadena de deps provisional.

## 2. REUSO (segundo, antes de deps definitivas y scaffold)

Scan del código custom del cliente/suite para no reinventar. Delega a `odoo-module-scanner` (modo parque) o
un subagente sonnet con grep dirigido sobre el repo de módulos:
- Busca dominio equivalente (¿ya existe el modelo/flujo?) → si existe, no lo reconstruyas.
- Busca **mixins/piezas transversales** reutilizables (valoración, enlaces factura↔albarán, helpers base).
- Salida: tabla `{pieza, qué aporta, dónde encaja, confianza}`. **Ajusta las deps del Step 1 con esto** antes de scaffoldear.

Honestidad: si solo verificaste la superficie (nombres de campos/mixin) y no el comportamiento, márcalo como
"reutilizar, confirmando encaje al construir", no como certeza.

## 3. DETALLE — el BUCLE iterativo de Q&A (⭐ núcleo de valor)

Es la parte que más cierra el diseño. **La skill genera las preguntas** — el usuario no piensa qué preguntar,
solo elige o escribe. Es un BUCLE, no una lista: rondas que se reciclan con análisis de gaps hasta converger.

### El bucle
```
mientras queden gaps que bloqueen una fase:
    1. la skill PROPONE la ronda de preguntas de la capa actual
       (cada una: opciones sensatas + recomendación marcada)
    2. lanzarlas UNA A UNA con AskUserQuestion (elegir o escribir)
    3. cada respuesta -> decisión D## -> aplicarla al doc (fuente única)
    4. ANÁLISIS DE GAPS entre rondas: ¿qué queda abierto que bloquee una fase?
       ¿qué NUEVAS preguntas destapa lo ya decidido? -> preparar la siguiente ronda
```

Capas, en orden (pero el bucle vuelve atrás si algo se reabre):
1. **Alcance** — qué entra en v1, qué al backlog, límites funcionales.
2. **Diseño de datos** — modelos, relaciones, campos clave, naming (con prefijo de suite).
3. **Comportamiento** — máquina de estados, flujos, permisos, edge cases.
4. **Reconciliación** — cuando un cambio (p.ej. de arquitectura) derogue decisiones, **reciclarlas explícitamente**.

### Cómo generar buenas preguntas (la palanca de valor)
- Una decisión por pregunta; **3-4 opciones** concretas con su trade-off; **marca la recomendada** ("(Recomendado)").
- Ordénalas por dependencia: primero las que condicionan otras (bloqueantes de fase) — márcalas 🔴.
- Da también la vía "escribe tu respuesta" — muchas decisiones reales no son ninguna de las opciones.
- Tras cada respuesta que reabra algo, dilo y lanza la pregunta derivada; no acumules deuda silenciosa.

### Disciplina
- **Cada decisión = un ID** (`D01`…) con su respuesta. En el doc, una tabla por bloque.
- **Verifica a fuente** toda sintaxis/API versión-específica. Nunca de memoria.
- **Fuente ÚNICA de verdad**: un solo documento; decisión derogada = **anotada como derogada**, no borrada ni viva contradiciendo.

### Convergencia y parada (honesta)
- Para cuando **ningún gap bloquee ninguna fase** — no cuando te canses.
- **"Acotado" NO es "pocas preguntas".** Pueden ser muchas rondas; lo acotado es el *criterio*: solo lo
  *load-bearing* (bloquea una fase o cambia la arquitectura). Los detalles de implementación (matriz ACL fila
  a fila, nombres de ubicación, N de un aviso, alcance de demo) se deciden CONSTRUYENDO.
- Señal de parada: si la siguiente ronda serían detalles que un dev resuelve solo → para y dilo.
- Antes de declarar "concluido", ejecuta un health-check y responde explícitamente "¿faltan preguntas o solo
  modificar el plan?" — ese meta-check es parte del bucle.

## Health-check (cada cierto punto y al final)

Semáforo por dimensión, **honesto** (una mala noticia a tiempo > sorpresa ante el cliente):
- Decisiones de diseño · Consistencia interna del doc · Verificación a fuente · Riesgo técnico residual · Preparación de proyecto.

Distingue lo que **cierra una pregunta** de lo que **solo se cierra construyendo** (integraciones con motores
nativos, encaje real de un mixin reutilizado). No pintes en verde un riesgo que solo se prueba con código;
si hace falta, propón un **spike** corto antes de comprometer estimación.

## Estimación

Cifra única: el esfuerzo real con factor IA aplicado, vía `odoo-effort-estimator`. Recalibrable por
fase. La estimación subirá ronda a ronda conforme se descubre trabajo oculto — eso es honesto; deja
**trazabilidad del porqué** de cada subida (no scope gratis).
