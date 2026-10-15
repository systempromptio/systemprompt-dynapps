# Bateria de Pruebas — Migration Assistant

> Instrucciones: Ejecutar cada test en una conversacion limpia de Claude Code.
> Verificar que el skill se activa y responde segun los criterios indicados.
> Marcar [x] cuando pase, [ ] si falla, y anotar observaciones.
>
> Contexto de prueba: Migracion ficticia de "Eurolotes v16 -> v18"

---

## Seccion A — Deteccion de contexto con consultas naturales

Verifica que el skill detecta rol, fase y proyecto de consultas naturales,
minimizando preguntas al usuario.

### T01 — Contexto completo en un mensaje
**Input:** `Soy dev en la migracion de Eurolotes v16 a v18, estamos en fase 2`
**Verificar:**
- [ ] NO pregunta rol (lo detecta: Dev)
- [ ] NO pregunta fase (la detecta: 2)
- [ ] NO pregunta proyecto (lo detecta: Eurolotes v16->v18)
- [ ] Ofrece ayuda relevante para Dev en Fase 2

### T02 — Rol implicito por actividad
**Input:** `Estoy migrando el codigo de Eurolotes v16 a v18, ¿que tengo que hacer?`
**Verificar:**
- [ ] Infiere rol = Dev (por "migrando el codigo")
- [ ] Infiere fase = 2 (por "migrando el codigo")
- [ ] NO pregunta rol ni fase
- [ ] Ofrece guia de rol Dev en Fase 2

### T03 — PM implicito con indicio de fase
**Input:** `Soy PM, mi desarrollador esta migrando codigo de Eurolotes, ¿que hago?`
**Verificar:**
- [ ] Detecta rol = PM
- [ ] Infiere fase = 2 (por "migrando codigo")
- [ ] Muestra acciones de PM en Fase 2 (seguimiento semanal, alerta desviacion, disponibilidad)
- [ ] NO pregunta fase

### T04 — BC con indicio de fase
**Input:** `Soy consultor de negocio en Eurolotes v16->v18, estamos en testing funcional. ¿Que me toca?`
**Verificar:**
- [ ] Detecta rol = BC (por "consultor de negocio")
- [ ] Detecta fase = 4 (por "testing funcional")
- [ ] Destaca que BC es responsable principal de esta fase
- [ ] Lista acciones de BC en Fase 4

### T05 — Sin rol explicito
**Input:** `Necesito ayuda con la migracion de Eurolotes`
**Verificar:**
- [ ] Pregunta el rol del usuario
- [ ] No asume un rol por defecto
- [ ] NO pide credenciales API

### T06 — Mensaje ambiguo sin rol ni fase
**Input:** `¿Como va lo de Eurolotes?`
**Verificar:**
- [ ] Ofrece consultar el progreso en Odoo (si tiene acceso a API)
- [ ] O pide aclaracion minima de rol/contexto
- [ ] NO pide credenciales de entrada

### T07 — Indicadores contextuales de fase
**Input:** `Soy dev en Eurolotes v16->v18, estamos con el upgrade de la base de datos`
**Verificar:**
- [ ] Infiere fase = 3 (por "upgrade de la base de datos")
- [ ] NO pregunta fase
- [ ] Ofrece guia relevante para Dev en Fase 3

### T08 — Indicadores de go-live
**Input:** `Soy BC, manana es el go-live de Eurolotes. ¿Que me toca?`
**Verificar:**
- [ ] Detecta fase = 6 (por "go-live")
- [ ] Lista acciones BC: smoke tests inmediatos, contacto con cliente, confirmar a PM
- [ ] Menciona tiempo de smoke tests: 15-30 min

### T09 — Indicadores de post-migracion
**Input:** `Soy TL en Eurolotes, ya hicimos el go-live la semana pasada. ¿Que tengo que hacer ahora?`
**Verificar:**
- [ ] Infiere fase = 7 (por "ya hicimos el go-live la semana pasada")
- [ ] Lista acciones TL en Fase 7: logs diarios, dailys, retrospectiva, Gate 7

### T10 — Deteccion de rol alternativo: "lider tecnico"
**Input:** `Soy el lider tecnico en Eurolotes v16->v18, estamos en fase 5. ¿Que me toca?`
**Verificar:**
- [ ] Detecta "lider tecnico" = TL
- [ ] Muestra acciones de TL en Fase 5

---

## Seccion B — Guia de fase (F2)

Verifica que describe correctamente cada fase con actividades, gate y acciones por rol.

### T11 — PM pregunta por Fase 0
**Input:** `Soy PM en la migracion de Eurolotes v16->v18. ¿En que consiste la fase 0?`
**Verificar:**
- [ ] Describe Fase 0: Kick-off, Alcance y Propuesta Comercial
- [ ] Menciona Gate 0 (Aprueba: COO, Ejecuta: PM)
- [ ] Incluye acciones de TODOS los roles en la fase
- [ ] Usa template phase-summary.md como formato
- [ ] NO pide credenciales

### T12 — Dev pregunta por Fase 2
**Input:** `Soy dev, estamos en fase 2 de la migracion de Eurolotes v16->v18. ¿Que pasa en esta fase?`
**Verificar:**
- [ ] Describe Fase 2: Migracion de Codigo
- [ ] Menciona Gate 2 (Aprueba: TL, Ejecuta: Dev)
- [ ] Incluye detalle tecnico (pipeline CI/CD, tests, commits)
- [ ] NO pide credenciales

### T13 — BC pregunta por Fase 4
**Input:** `Soy BC en Eurolotes v16->v18, fase 4. ¿Que pasa en esta fase?`
**Verificar:**
- [ ] Describe Fase 4: Testing Funcional (FASE CRITICA)
- [ ] Destaca que BC es responsable principal
- [ ] Menciona checklist funcional (9 items) y tecnico (7 items)
- [ ] Menciona criterios Go/No-Go

### T14 — TL pregunta por Fase 6
**Input:** `Soy TL, Eurolotes v16->v18, estamos en fase 6. ¿Que hay en esta fase?`
**Verificar:**
- [ ] Describe Fase 6: Puesta en Produccion (Go-Live)
- [ ] Destaca que la decision de abortar es EXCLUSIVAMENTE del TL
- [ ] Menciona Gate 6 (Aprueba: TL + CTO Ops)

---

## Seccion C — Guia de rol (F3)

Verifica que entrega acciones numeradas con entregables concretos para cada rol.

### T15 — Dev en Fase 1: "¿Que tengo que hacer?"
**Input:** `Soy dev en la migracion Eurolotes v16->v18, estamos en fase 1. ¿Que tengo que hacer?`
**Verificar:**
- [ ] Lista acciones numeradas (6 acciones del Dev en Fase 1)
- [ ] Cada accion tiene entregable concreto (ej: "Rama creada y push")
- [ ] Incluye comunicacion requerida (Dev->TL: diario)
- [ ] Menciona Gate 1 y que Dev lo ejecuta
- [ ] NO pide credenciales

### T16 — PM en Fase 2: "¿Que tengo que hacer?"
**Input:** `Soy PM, migracion Eurolotes v16->v18, fase 2. ¿Que hago?`
**Verificar:**
- [ ] Lista 3 acciones del PM en Fase 2
- [ ] Incluye: seguimiento semanal, alerta si desviacion >15%, coordinar disponibilidad
- [ ] Tono ejecutivo, no tecnico

### T17 — BC en Fase 1: "¿Que tengo que hacer?"
**Input:** `Soy consultor de negocio, migracion Eurolotes v16->v18, fase 1. ¿Que me toca?`
**Verificar:**
- [ ] Detecta rol BC aunque diga "consultor de negocio"
- [ ] Destaca que BC es responsable principal de documentar flujos
- [ ] Lista los flujos: Ventas, Compras, Inventario, Fabricacion, Contabilidad, RRHH, Integraciones
- [ ] Indica plazo: completado antes del Gate 1

### T18 — TL en Fase 3: "¿Que tengo que hacer?"
**Input:** `Soy tech lead en Eurolotes v16->v18, fase 3. ¿Que me toca?`
**Verificar:**
- [ ] Lista 3 acciones del TL en Fase 3
- [ ] Incluye: verificar scripts versionados, validar reproducibilidad, firmar Gate 3
- [ ] Cada accion tiene entregable

### T19 — COO en Fase 0: "¿Que tengo que hacer?"
**Input:** `Soy COO, me preguntan por la migracion de Eurolotes v16->v18. ¿Que tengo que hacer?`
**Verificar:**
- [ ] Lista 3 acciones del COO en Fase 0
- [ ] Incluye: revisar propuesta, aprobar condiciones especiales, firmar Gate 0
- [ ] Tono ejecutivo, resumen breve

### T20 — Dev en Fase 4: inferencia de fase por nombre
**Input:** `Soy dev, migracion Eurolotes v16->v18, estamos en testing funcional. ¿Que hago?`
**Verificar:**
- [ ] Detecta "testing funcional" = Fase 4
- [ ] Lista 4 acciones del Dev: checklist tecnico, corregir bugs <24h, documentar, Gate 4
- [ ] Menciona los 7 items del checklist tecnico

---

## Seccion D — Validacion de gate (F4)

Verifica que entrega plantilla copy-paste y entregables por rol.

### T21 — Gate 0
**Input:** `Soy PM en Eurolotes v16->v18. Necesito el Gate 0.`
**Verificar:**
- [ ] Entrega plantilla Gate 0 completa en formato copy-paste (con boxes ASCII)
- [ ] Pre-rellena Proyecto: Eurolotes, Version: v16->v18
- [ ] Indica: Ejecuta PM, Aprueba COO
- [ ] Lista entregables (10 items del Gate 0)

### T22 — Gate 2 via "¿Puedo avanzar?"
**Input:** `Soy dev en Eurolotes v16->v18. ¿Puedo avanzar a fase 3? Estamos en fase 2.`
**Verificar:**
- [ ] Detecta que pregunta por Gate 2
- [ ] Entrega plantilla Gate 2 copy-paste
- [ ] Lista entregables: modulos migrados, pipeline verde, code review, tests
- [ ] Indica que TL aprueba

### T23 — Gate 4 (critico)
**Input:** `Gate 4, migracion Eurolotes v16->v18`
**Verificar:**
- [ ] Entrega plantilla Gate 4 completa (la mas larga: funcional 9 items + tecnico 7 items + Go/No-Go)
- [ ] Incluye seccion ACTA DE CONFORMIDAD
- [ ] Incluye criterios Go/No-Go
- [ ] Formato copy-paste listo para usar

### T24 — Gate 6
**Input:** `Soy TL, necesito validar el Gate 6 de Eurolotes`
**Verificar:**
- [ ] Entrega plantilla Gate 6 con prerrequisitos, protocolo, ejecucion, post go-live
- [ ] Indica doble aprobacion: TL + CTO Ops
- [ ] Destaca entregables del TL para este gate

### T25 — "¿Puedo avanzar?" sin especificar gate
**Input:** `Soy dev en Eurolotes v16->v18, fase 3. ¿Puedo avanzar?`
**Verificar:**
- [ ] Infiere Gate 3 (fin de Fase 3)
- [ ] Entrega plantilla Gate 3

---

## Seccion E — Planificacion tecnica (F5)

Verifica que genera plan tecnico con orden de migracion, scripts y comunicacion.

### T26 — Dev pregunta "¿Por donde empiezo?" (sin API)
**Input:** `Soy dev, empezamos la migracion de Eurolotes v16->v18. Tenemos 3 modulos Standard, 2 Studio, 1 Core y 4 Custom. ¿Por donde empiezo?`
**Verificar:**
- [ ] Ordena: Standard primero -> Core/Studio -> Custom al final
- [ ] Incluye seccion de gestion de scripts (referencia a script-management.md)
- [ ] Incluye protocolo de comunicacion (Dev->BC 2x/semana, Dev->TL diario)
- [ ] Referencia al skill odoo-module-upgrader para patrones de codigo
- [ ] NO pide credenciales (modo sin API)

### T27 — TL pregunta plan tecnico
**Input:** `Soy TL en Eurolotes v16->v18. Necesito un plan tecnico para el equipo.`
**Verificar:**
- [ ] Incluye orden de migracion con criterios
- [ ] Incluye puntos de review y validacion de arquitectura (perspectiva TL)
- [ ] Incluye rama `release/migration-18.0`
- [ ] Incluye pipeline CI/CD

---

## Seccion F — Consultas de proceso (F6)

Verifica respuestas directas a preguntas libres, citando fuente.

### T28 — Pregunta sobre comunicacion
**Input:** `Soy dev en Eurolotes v16->v18. ¿Cada cuanto tengo que informar al BC?`
**Verificar:**
- [ ] Responde: 2 veces por semana con informe estructurado
- [ ] Cita fuente: references/communication.md o protocolo de comunicacion
- [ ] Sin template formal, respuesta directa

### T29 — Pregunta sobre estimacion
**Input:** `¿Cuantas horas se estiman para migrar una BD de 15GB?`
**Verificar:**
- [ ] Delega la formula de BD en `odoo-effort-estimator` (no la reproduce)
- [ ] Cita fuente: references/estimation.md

### T30 — Pregunta sobre escalacion
**Input:** `Soy BC, llevo 3 dias esperando respuesta del dev. ¿Que hago?`
**Verificar:**
- [ ] Indica la regla de 48h y cadena de escalacion
- [ ] Sugiere escalar al TL
- [ ] Cita la Regla de Oro de communication.md

### T31 — Pregunta sobre scripts
**Input:** `¿Como debo nombrar los scripts de migracion?`
**Verificar:**
- [ ] Indica formato: NNN_descripcion_breve.py
- [ ] Menciona estructura de directorios (pre_migrate/, post_migrate/, cleanup/, validation/)
- [ ] Cita fuente: references/script-management.md

### T32 — Pregunta sobre algo no documentado
**Input:** `¿Como se configura el SSO en la version destino?`
**Verificar:**
- [ ] Indica explicitamente que no esta cubierto en las references del skill
- [ ] NO inventa una respuesta
- [ ] Puede sugerir buscar en documentacion oficial de Odoo

### T33 — Pregunta sobre testing en produccion
**Input:** `¿Se puede testear en produccion?`
**Verificar:**
- [ ] Explica Modalidad B (testeo en produccion con reversion)
- [ ] Lista las 5 reglas de seguridad
- [ ] Cita fuente: references/flow-testing.md

### T34 — Pregunta sobre checklists
**Input:** `¿Que checklist hay para la fase 4?`
**Verificar:**
- [ ] Menciona checklist funcional (9 items) y tecnico (7 items)
- [ ] Los lista o referencia references/checklists.md
- [ ] Usa template checklist-output.md

---

## Seccion G — Informes de comunicacion (F7)

Verifica que entrega la plantilla correcta segun el rol.

### T35 — Dev pide plantilla de informe
**Input:** `Soy dev en Eurolotes v16->v18, fase 2. Dame la plantilla de informe para el BC.`
**Verificar:**
- [ ] Entrega template dev-to-bc-report.md
- [ ] Indica frecuencia: 2 veces por semana
- [ ] Si tiene contexto, pre-rellena lo que pueda (proyecto, fase)

### T36 — TL pide plantilla de informe
**Input:** `Soy TL en Eurolotes. Necesito hacer el resumen semanal para el PM.`
**Verificar:**
- [ ] Entrega template tl-to-pm-report.md
- [ ] Indica frecuencia: semanal
- [ ] NO entrega el template dev-to-bc

---

## Seccion H — Informe de progreso (F8)

Verifica que pide credenciales SOLO aqui y usa calculate_progress.py.

### T37 — TL pregunta "¿Como vamos?"
**Input:** `Soy TL en la migracion de Eurolotes v16->v18. ¿Como vamos?`
**Verificar:**
- [ ] Pide credenciales API (o intenta leer de odoo-credentials.md)
- [ ] Pide ID del proyecto en Odoo
- [ ] Indica que usara calculate_progress.py
- [ ] NO intenta calcular manualmente

### T38 — PM pregunta informe de horas
**Input:** `Soy PM, migracion Eurolotes. Necesito un informe de horas.`
**Verificar:**
- [ ] Pide credenciales API
- [ ] Pide ID del proyecto y horas presupuestadas
- [ ] Menciona las alertas automaticas (CRITICAL, WARNING)
- [ ] Usa template migration-status-report.md

---

## Seccion I — Vista de progreso de gates (F9)

Verifica la nueva funcionalidad de consultar gates en Odoo.

### T39 — "¿Que gates faltan?"
**Input:** `Soy PM en Eurolotes v16->v18. ¿Que gates tenemos pendientes?`
**Verificar:**
- [ ] Pide credenciales API (o intenta leer de odoo-credentials.md)
- [ ] Pide ID del proyecto
- [ ] Busca tareas [GATE] en el proyecto
- [ ] Muestra tabla de gates con estado (Aprobado/Pendiente)
- [ ] Indica la fase actual segun gates
- [ ] Muestra barra de progreso

### T40 — "¿Como va la migracion?" (modo gates)
**Input:** `¿Como va la migracion de Eurolotes?`
**Verificar:**
- [ ] Ofrece mostrar progreso por gates o por horas
- [ ] Si elige gates, ejecuta F9
- [ ] Si elige horas, ejecuta F8

### T41 — Sin gates creados
**Input:** (Simular proyecto sin tareas [GATE])
**Verificar:**
- [ ] Indica que no hay gates creados
- [ ] Ofrece crear los 8 gates automaticamente

---

## Seccion J — Creacion de gates (F10)

Verifica la nueva funcionalidad de crear gates en Odoo.

### T42 — Crear gates con preview
**Input:** `Crea los gates para el proyecto de Eurolotes v16->v18`
**Verificar:**
- [ ] Pide credenciales API
- [ ] Pide ID del proyecto
- [ ] Muestra preview de las 8 tareas gate que se van a crear
- [ ] Los nombres siguen el formato: [GATE] Gate N -- {nombre}
- [ ] Pide confirmacion ANTES de crear
- [ ] NO crea sin confirmacion

### T43 — Nombres correctos de gates
**Input:** (Continuacion de T42, tras dar credenciales y project_id)
**Verificar:**
- [ ] Gate 0: [GATE] Gate 0 -- Alcance y Propuesta Comercial
- [ ] Gate 1: [GATE] Gate 1 -- Analisis Tecnico
- [ ] Gate 2: [GATE] Gate 2 -- Codigo Migrado
- [ ] Gate 3: [GATE] Gate 3 -- Base de Datos Migrada
- [ ] Gate 4: [GATE] Gate 4 -- Testing Funcional
- [ ] Gate 5: [GATE] Gate 5 -- Ensayo General (Rehearsal)
- [ ] Gate 6: [GATE] Gate 6 -- Puesta en Produccion (Go-Live)
- [ ] Gate 7: [GATE] Gate 7 -- Cierre del Proyecto

### T44 — Gates con checklist de entregables
**Input:** (Continuacion de T42, verificar descripcion)
**Verificar:**
- [ ] Cada gate tiene descripcion HTML con entregables como checklist
- [ ] Cada gate indica quien ejecuta y quien aprueba
- [ ] Tag Migration aplicado a todos los gates

---

## Seccion K — Deteccion automatica de fase por gates

Verifica que el skill puede determinar la fase actual consultando gates.

### T45 — Dev pregunta sin saber la fase, con acceso a Odoo
**Input secuencia:**
1. `Soy dev en Eurolotes v16->v18. ¿Que tengo que hacer?`
2. (Skill pregunta o infiere que puede consultar Odoo)
3. (Dar credenciales y project_id)
**Verificar:**
- [ ] OFRECE consultar Odoo para detectar la fase automaticamente
- [ ] NO pregunta "¿en que fase estais?" como primera opcion
- [ ] Tras consultar gates, indica la fase actual
- [ ] Muestra guia de rol para Dev en la fase detectada

### T46 — Fase detectada correctamente
**Input:** (Simular gates 0 y 1 aprobados, resto pendiente)
**Verificar:**
- [ ] Skill detecta fase actual = 2 (Migracion de Codigo)
- [ ] Dice: "Segun los gates del proyecto, estais en Fase 2"
- [ ] Muestra acciones de Dev en Fase 2

---

## Seccion L — Credenciales diferidas

### T47 — Conversacion larga sin API
**Input secuencia:**
1. `Soy dev en Eurolotes v16->v18, fase 2`
2. `¿Que tengo que hacer?`
3. `Dame el Gate 2`
4. `¿Cada cuanto informo al BC?`
**Verificar:**
- [ ] En NINGUN momento de los 4 mensajes pide credenciales
- [ ] Responde correctamente a todo usando solo references/

### T48 — Credenciales solo al pedir F8
**Input secuencia:**
1. `Soy TL en Eurolotes v16->v18`
2. `¿Que tengo que hacer en fase 2?` -> (sin credenciales)
3. `¿Como vamos?` -> (AQUI debe pedir credenciales)
**Verificar:**
- [ ] Mensajes 1-2: sin pedir credenciales
- [ ] Mensaje 3: pide credenciales porque necesita API

### T49 — Credenciales al pedir crear gates
**Input secuencia:**
1. `Soy PM en Eurolotes v16->v18`
2. `¿Que hago en fase 0?` -> (sin credenciales)
3. `Crea los gates del proyecto` -> (AQUI pide credenciales)
**Verificar:**
- [ ] Mensajes 1-2: sin pedir credenciales
- [ ] Mensaje 3: pide credenciales y project_id

---

## Seccion M — Adaptacion al rol

### T50 — Misma pregunta, Dev vs PM (Fase 4)
**Input A:** `Soy dev en Eurolotes v16->v18, fase 4. ¿Que tengo que hacer?`
**Input B:** `Soy PM en Eurolotes v16->v18, fase 4. ¿Que tengo que hacer?`
**Verificar:**
- [ ] Respuesta A: detalle tecnico (checklist 7 items, corregir bugs <24h)
- [ ] Respuesta B: resumen ejecutivo (coordinar calendario, asegurar disponibilidad)
- [ ] Las acciones son DIFERENTES para cada rol

### T51 — Misma pregunta, BC vs TL (Fase 5)
**Input A:** `Soy BC en Eurolotes v16->v18, fase 5. ¿Que me toca?`
**Input B:** `Soy TL en Eurolotes v16->v18, fase 5. ¿Que me toca?`
**Verificar:**
- [ ] Respuesta A: observar rehearsal, smoke tests, documentar tiempos
- [ ] Respuesta B: firmar checklist infraestructura, validar tiempos, aprobar Gate 5

---

## Seccion N — Cobertura completa de fases

### T52 — Dev recorre las 8 fases
**Input secuencia (8 mensajes):**
1. `Soy dev, Eurolotes v16->v18, fase 0. ¿Que hago?`
2. `¿Y en fase 1?`
3. `¿Y en fase 2?`
4. `¿Y en fase 3?`
5. `¿Y en fase 4?`
6. `¿Y en fase 5?`
7. `¿Y en fase 6?`
8. `¿Y en fase 7?`
**Verificar:**
- [ ] Fase 0: inventario, clasificar, estimacion, integraciones (4 acciones)
- [ ] Fase 1: analisis codigo, clasificar, rama, testing, infra, Gate 1 (6 acciones)
- [ ] Fase 2: migrar modulos, informe 2x/semana, escalar, log, tests, Gate 2 (6 acciones)
- [ ] Fase 3: iteraciones upgrade, scripts, documentar errores, informar BC, validacion, Gate 3 (6 acciones)
- [ ] Fase 4: checklist tecnico, bugs <24h, documentar, Gate 4 (4 acciones)
- [ ] Fase 5: simulacion, cronometrar, infra, rollback, Gate 5 (5 acciones)
- [ ] Fase 6: ejecutar paso a paso, comunicar, backup post-migracion, Gate 6 (4 acciones)
- [ ] Fase 7: disponibilidad 2 semanas, incidencias SLA, documentar (3 acciones)
- [ ] Contenido NO se repite entre fases
- [ ] NUNCA pide credenciales

### T53 — BC recorre fases clave
**Input secuencia (4 mensajes):**
1. `Soy BC, Eurolotes v16->v18, fase 1. ¿Que hago?`
2. `¿Y en fase 4?`
3. `¿Y en fase 5?`
4. `¿Y en fase 6?`
**Verificar:**
- [ ] Fase 1: documentar TODOS los flujos (responsable principal)
- [ ] Fase 4: responsable principal, testing end-to-end, coordinar cliente, incidencias, acta conformidad
- [ ] Fase 5: observar, smoke tests, documentar
- [ ] Fase 6: smoke tests inmediatos, contacto cliente, confirmar a PM

---

## Seccion O — Referencias cruzadas

### T54 — Referencia a module-upgrader
**Input:** `Soy dev en Eurolotes v16->v18, fase 2. ¿Como migro un modulo con componentes OWL?`
**Verificar:**
- [ ] Referencia al skill odoo-module-upgrader para patrones de codigo
- [ ] NO intenta explicar la migracion OWL en detalle (eso es del otro skill)

### T55 — Referencia a odoo-flow-tester
**Input:** `Soy BC en Eurolotes v16->v18, fase 4. ¿Como ejecuto los tests de flujos?`
**Verificar:**
- [ ] Menciona el catalogo de 14 flujos estandar
- [ ] Referencia al skill odoo-flow-tester para ejecucion automatizada
- [ ] Puede explicar las modalidades A y B de testing

### T56 — Referencia a database-upgrader
**Input:** `Soy dev en Eurolotes v16->v18, fase 3. ¿Como hago el upgrade de la BD?`
**Verificar:**
- [ ] Referencia al skill odoo-database-upgrader para el proceso de upgrade
- [ ] NO duplica el contenido de ese skill

---

## Seccion P — Soporte: crear tareas

### T57 — PM pide crear tareas
**Input:** `Soy PM, quiero crear las tareas de migracion de Eurolotes v16->v18`
**Verificar:**
- [ ] Pide ruta al fichero de analisis local
- [ ] Pide ID del proyecto en Odoo
- [ ] Pide credenciales API
- [ ] Indica que ejecutara --dry-run primero
- [ ] Indica que pedira confirmacion antes de crear

---

## Seccion Q — Consultas en espanol avanzado

Verifica que el skill entiende variaciones del espanol.

### T58 — Uso de "piloto" y jerga
**Input:** `Soy el que programa, estamos con Eurolotes pasandolo de la 16 a la 18. ¿Que me queda por hacer?`
**Verificar:**
- [ ] Infiere rol = Dev (por "el que programa")
- [ ] Infiere migracion v16->v18
- [ ] Responde en espanol
- [ ] Pide fase o intenta inferirla

### T59 — "Mis gates"
**Input:** `Soy TL en Eurolotes. ¿Que gates me tocan a mi?`
**Verificar:**
- [ ] Lista los gates donde TL tiene rol (como ejecutor o aprobador)
- [ ] TL aprueba: Gate 1, 2, 3, 4, 5, 6
- [ ] TL ejecuta: Gate 7 (junto con PM y BC)
- [ ] Datos de references/gate-templates.md

### T60 — Pregunta compuesta
**Input:** `Soy PM en Eurolotes v16->v18, fase 2. ¿Que tengo que hacer y como va el proyecto?`
**Verificar:**
- [ ] Responde a ambas preguntas
- [ ] Guia de rol (F3) para PM en Fase 2
- [ ] Para "como va", pide credenciales o indica que necesita API

---

## Resumen de cobertura

| Seccion | Tests | Cobertura |
|---------|-------|-----------|
| A — Deteccion contexto natural | T01-T10 | Rol por indicios, fase por contexto, minimizar preguntas |
| B — Guia de fase (F2) | T11-T14 | PM, Dev, BC, TL |
| C — Guia de rol (F3) | T15-T20 | Dev, PM, BC, TL, COO |
| D — Validacion gate (F4) | T21-T25 | PM, Dev, TL, generico |
| E — Planificacion tecnica (F5) | T26-T27 | Dev, TL |
| F — Consultas proceso (F6) | T28-T34 | Dev, BC, generico |
| G — Informes comunicacion (F7) | T35-T36 | Dev, TL |
| H — Informe progreso (F8) | T37-T38 | TL, PM |
| I — Progreso de gates (F9) | T39-T41 | PM, generico |
| J — Creacion de gates (F10) | T42-T44 | PM, TL |
| K — Deteccion fase por gates | T45-T46 | Dev |
| L — Credenciales diferidas | T47-T49 | Dev, TL, PM |
| M — Adaptacion al rol | T50-T51 | Dev/PM, BC/TL |
| N — Cobertura fases | T52-T53 | Dev (8 fases), BC (4 fases) |
| O — Referencias cruzadas | T54-T56 | Dev, BC |
| P — Soporte tareas | T57 | PM |
| Q — Espanol avanzado | T58-T60 | Dev, TL, PM |

**Total: 60 tests**
**Roles: PM, TL, BC, Dev, COO/CTO**
**Fases: 0-7 completas**
**Funcionalidades: F1-F10 + soporte**
**Nuevas funcionalidades probadas: F9 (progreso gates), F10 (creacion gates), deteccion automatica de fase**
