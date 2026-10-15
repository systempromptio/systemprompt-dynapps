# Guía por Perfil — Acciones Concretas por Rol y Fase

> Fuente: Artículo 2602 — Secciones "¿Qué tengo que hacer?" por fase
> Formato: acciones numeradas con entregable concreto por rol
> Consultar SIEMPRE esta referencia antes de responder a "¿Qué tengo que hacer?"

---

## Fase 0 — Kick-off, Alcance y Propuesta Comercial

### Si eres PM:
1. Convoca reunión de kick-off con TL, BC y Dev → **Entregable:** Acta de reunión con asistentes y acuerdos
2. Verifica que la propuesta comercial incluye: guardias, fines de semana, horas post-migración → **Entregable:** Propuesta revisada con check de inclusiones
3. Envía al cliente un email explicando el proceso de migración completo → **Entregable:** Email enviado (CC a BC y TL)
4. Recopila y archiva la firma del cliente → **Entregable:** Documento de alcance firmado
5. Rellena y envía la plantilla de Gate 0 al COO → **Entregable:** Gate 0 completado para aprobación

### Si eres TL:
1. Decide con el Dev la estrategia de migración (1:1 vs Refactoring) → **Entregable:** Decisión documentada con justificación
2. Revisa el inventario de módulos: verifica que es completo y no falta ninguno → **Entregable:** Inventario validado
3. Revisa la estimación de horas del Dev: corrige si es necesario, firma la estimación → **Entregable:** Estimación aprobada
4. Participa en la reunión técnica de análisis para consensuar estrategia por módulo → **Entregable:** Estrategia por módulo acordada

### Si eres BC:
1. Documenta los flujos críticos del cliente (Ventas, Compras, Inventario, Fabricación, Contabilidad, RRHH, Integraciones) → **Entregable:** Documento de flujos críticos
   - Formato: para cada flujo, describe paso a paso lo que hace el cliente hoy
   - Incluye: usuarios involucrados, frecuencia, volumen, particularidades
2. Participa en la reunión técnica de análisis → **Entregable:** Validación funcional de la clasificación de módulos
3. Revisa la propuesta comercial desde perspectiva funcional → **Entregable:** Feedback sobre la propuesta

### Si eres Dev:
1. Ejecuta el inventario de módulos custom del repositorio → **Entregable:** Lista completa de módulos con tipo, dependencias y líneas de código
2. Clasifica cada módulo (Standard / Studio / Core / Custom) según la estrategia → **Entregable:** Tabla de clasificación
3. Produce la estimación técnica detallada usando las fórmulas de la sección 13 → **Entregable:** Desglose de horas por módulo + horas BD
4. Identifica integraciones externas (APIs, webhooks, imports/exports) → **Entregable:** Lista de integraciones con endpoints y estado

### Si eres COO/CTO:
1. Revisa la propuesta si supera el umbral económico definido → **Entregable:** Aprobación o feedback
2. Aprueba condiciones especiales (guardias, fines de semana) → **Entregable:** Autorización formal
3. Firma el Gate 0 → **Entregable:** Gate 0 aprobado

---

## Fase 1 — Preparación Técnica

### Si eres PM:
1. Queda informado del progreso → No requiere acción directa en esta fase
2. Si el TL reporta bloqueos, coordina recursos → **Entregable:** Bloqueo resuelto o escalado

### Si eres TL:
1. Revisa el análisis de código del Dev (APIs deprecadas, seguridad, rendimiento) → **Entregable:** Análisis validado o con correcciones
2. Decide con el Dev la clasificación definitiva de módulos → **Entregable:** Clasificación firmada
3. Supervisa que el entorno de testing está operativo → **Entregable:** Confirmación de entorno funcional
4. Revisa la Ficha de Infraestructura del servidor destino → **Entregable:** Ficha validada
5. Firma el Gate 1 → **Entregable:** Gate 1 aprobado

### Si eres BC:
1. **Responsable principal:** Documenta TODOS los flujos críticos del cliente end-to-end → **Entregable:** Documento de flujos con:
   - Ventas: Presupuesto → Pedido → Factura → Cobro (paso a paso)
   - Compras: Solicitud → Pedido → Recepción → Factura → Pago
   - Inventario: Movimientos, rutas, reglas de reabastecimiento
   - Fabricación (si aplica): BOM → OF → Producción → Entrega
   - Contabilidad: Asientos, conciliación bancaria, cierres
   - RRHH/Nóminas (si aplica)
   - Integraciones externas
   **Plazo:** Completado antes del Gate 1

### Si eres Dev:
1. Ejecuta análisis de código del repositorio → **Entregable:** Informe con:
   - APIs deprecadas encontradas (lista con fichero:línea)
   - Vulnerabilidades de seguridad (SQL injection, sudo abuse, eval/exec)
   - Problemas de rendimiento (N+1, search sin limit)
2. Completa la clasificación de módulos (Std/Studio/Core/Custom) → **Entregable:** Tabla actualizada
3. Crea la rama `release/migration-X.Y` según gitflow → **Entregable:** Rama creada y push
4. Configura el entorno de testing → **Entregable:** Entorno funcional con URL de acceso
5. Completa la Ficha de Infraestructura del servidor destino → **Entregable:** Ficha con todos los campos rellenados
6. Rellena y envía la plantilla de Gate 1 al TL → **Entregable:** Gate 1 completado para aprobación

### Si eres COO/CTO:
1. Informado → No requiere acción directa

---

## Fase 2 — Migración de Código

### Si eres PM:
1. Revisa semanalmente horas consumidas vs presupuesto → **Entregable:** Seguimiento semanal
2. Si desviación > 15%, alerta al TL inmediatamente → **Entregable:** Alerta documentada
3. Coordina disponibilidad del equipo si hay conflictos de agenda → **Entregable:** Agenda resuelta

### Si eres TL:
1. Haz code review continuo (NO esperes al final de la fase) → **Entregable:** Reviews aprobados en el PR/MR
2. Valida la arquitectura de módulos complejos (JS/OWL, integraciones) ANTES de que el Dev empiece → **Entregable:** Arquitectura aprobada
3. Monitoriza que el pipeline CI/CD se mantiene verde → **Entregable:** Pipeline verde confirmado
4. Lee los informes Dev→BC 2x/semana para detectar bloqueos → **Entregable:** Intervención si hay bloqueos >24h

### Si eres BC:
1. Responde a las consultas funcionales del Dev en < 24h → **Entregable:** Respuesta documentada en la tarea Odoo
2. Valida que las decisiones de migración preservan los flujos del cliente → **Entregable:** Validación funcional por módulo
3. Empieza a preparar al cliente para la fase de testing (avísale de que pronto le necesitarás) → **Entregable:** Email/llamada de pre-aviso al cliente

### Si eres Dev:
1. Migra módulos en orden de dependencias (sin dependencias primero) → **Entregable:** Módulos migrados con commit por módulo
2. **OBLIGATORIO:** Publica informe Dev→BC 2 veces por semana en la tarea principal → **Entregable:** Informe con formato de sección 3.1.B
3. Escala decisiones funcionales inmediatamente al BC — **NO asumas respuestas** → **Entregable:** Pregunta documentada en tarea con opciones
4. Actualiza el log de avance de cada tarea después de cada sesión de trabajo → **Entregable:** Log actualizado
5. Escribe tests unitarios por módulo migrado → **Entregable:** Tests pasando
6. Al completar todos los módulos, rellena y envía Gate 2 al TL → **Entregable:** Gate 2 para aprobación

### Si eres COO/CTO:
1. Informado → No requiere acción directa

---

## Fase 3 — Migración de Base de Datos

### Si eres PM:
1. Monitoriza tiempos de cada iteración (servirán para estimar la ventana de Go-Live) → **Entregable:** Registro de tiempos por iteración

### Si eres TL:
1. Verifica que los scripts pre/post-migrate están en el repositorio y versionados → **Entregable:** Scripts en repo confirmados
2. Valida que el proceso es reproducible (mismos pasos = mismo resultado) → **Entregable:** Proceso validado
3. Firma el Gate 3 → **Entregable:** Gate 3 aprobado

### Si eres BC:
1. Valida datos migrados desde perspectiva funcional → **Entregable:** Informe de validación con:
   - Saldos contables cuadrados
   - Relaciones cliente-pedido-factura correctas
   - Histórico de movimientos de inventario
   - Datos de RRHH (si aplica)
2. Reporta discrepancias al Dev con ejemplos concretos (IDs, capturas) → **Entregable:** Lista de discrepancias

### Si eres Dev:
1. Ejecuta las iteraciones de upgrade según tamaño del proyecto → **Entregable:** BD migrada en testing
2. Escribe/ejecuta scripts pre-migrate y post-migrate → **Entregable:** Scripts versionados en `scripts/` o `fixes/`
3. Documenta cada error encontrado y su fix en el directorio `fixes/` → **Entregable:** Log de errores + fixes
4. Informa a BC del resultado de cada iteración → **Entregable:** Informe de iteración
5. Produce informe de validación de datos (conteos antes/después por modelo) → **Entregable:** Tabla comparativa de registros
6. Rellena y envía Gate 3 al TL → **Entregable:** Gate 3 para aprobación

### Si eres COO/CTO:
1. Informado → No requiere acción directa

---

## Fase 4 — Testing Funcional (FASE CRÍTICA)

### Si eres PM:
1. Coordina calendario de testing con el cliente → **Entregable:** Agenda de sesiones BC + Dev + Cliente
2. Asegura disponibilidad simultánea de BC, Dev y Cliente → **Entregable:** Confirmación de asistencia

### Si eres TL:
1. Valida que los criterios Go/No-Go se aplican sin excepciones → **Entregable:** Criterios verificados
2. Revisa los checklists funcional y técnico completados → **Entregable:** Checklists revisados
3. Aprueba o rechaza el Gate 4 → **Entregable:** Gate 4 firmado

### Si eres BC:
1. **Eres el responsable principal de esta fase.**
2. Ejecuta testing end-to-end de TODOS los flujos críticos documentados en Fase 1 → **Entregable:** Checklist funcional (9 items) completado con OK/KO
3. Coordina sesiones de testing con el cliente → **Entregable:** Sesiones realizadas con resultado documentado
4. Documenta cada incidencia KO con pasos exactos de reproducción → **Entregable:** Lista de incidencias con pasos para reproducir
5. Tras correcciones del Dev, re-testea los flujos KO → **Entregable:** Re-validación documentada
6. Gestiona la firma del acta de conformidad con el cliente → **Entregable:** Acta firmada por BC + TL + Cliente

### Si eres Dev:
1. Completa el checklist técnico (7 items) → **Entregable:** Checklist técnico con OK/KO por item
2. Corrige bugs reportados por BC en < 24h → **Entregable:** Fix con commit + re-deploy en testing
3. Documenta cada corrección y confirma al BC para re-validación → **Entregable:** Log de fixes
4. Rellena y envía Gate 4 al TL → **Entregable:** Gate 4 para aprobación

### Si eres COO/CTO:
1. Consultado en criterios Go/No-Go si hay dudas → **Entregable:** Decisión si se escala

---

## Fase 5 — Ensayo General (Rehearsal)

### Si eres PM:
1. Confirma la ventana de mantenimiento con el cliente usando los tiempos del rehearsal → **Entregable:** Email formal al cliente con fecha, hora inicio/fin, contactos
2. Prepara comunicación formal de Go-Live → **Entregable:** Borrador de comunicación

### Si eres TL:
1. **Firma el Checklist de Infraestructura junto con el Dev** → **Entregable:** Checklist firmado
2. Valida que los tiempos del rehearsal caben en la ventana de mantenimiento → **Entregable:** Validación de tiempos
3. Aprueba o rechaza Gate 5 → **Entregable:** Gate 5 firmado

### Si eres BC:
1. Observa el rehearsal completo → **Entregable:** Presencia confirmada
2. Ejecuta smoke tests funcionales inmediatamente tras la restauración → **Entregable:** Smoke tests OK/KO
3. Documenta tiempos e incidencias observadas → **Entregable:** Log del rehearsal

### Si eres Dev:
1. Ejecuta simulación completa — exactamente los mismos pasos que en producción → **Entregable:** Rehearsal ejecutado
2. Cronometra CADA paso (backup, cierre, upgrade, scripts, smoke, apertura) → **Entregable:** Tabla de tiempos
3. Completa el Checklist de Infraestructura al 100% → **Entregable:** Checklist completado
4. Prueba el plan de rollback → **Entregable:** Rollback probado y documentado
5. Rellena y envía Gate 5 al TL → **Entregable:** Gate 5 para aprobación

### Si eres COO/CTO:
1. Informado → No requiere acción directa

---

## Fase 6 — Puesta en Producción (Go-Live)

### Si eres PM:
1. Gestiona la comunicación con el cliente durante todo el Go-Live (inicio, avance, fin) → **Entregable:** Comunicaciones enviadas
2. Coordina la apertura gradual de accesos → **Entregable:** Accesos abiertos por fases
3. Activa el equipo de guardia → **Entregable:** Equipo confirmado y disponible
4. Gestiona la comunicación de fin de ventana → **Entregable:** Email de cierre al cliente

### Si eres TL:
1. Supervisa la ejecución en tiempo real → **Entregable:** Supervisión activa durante todo el Go-Live
2. Tiene el plan de rollback preparado → **Entregable:** Plan de rollback listo para ejecutar
3. **La decisión de abortar es EXCLUSIVAMENTE tuya** — ejecuta rollback sin esperar si detectas error bloqueante → **Entregable:** Decisión documentada
4. Firma Gate 6 junto con CTO Ops → **Entregable:** Gate 6 firmado

### Si eres BC:
1. Ejecuta smoke tests funcionales INMEDIATAMENTE tras la migración → **Entregable:** Resultado smoke tests (15-30 min)
2. Es el punto de contacto con el cliente durante el Go-Live → **Entregable:** Comunicación activa
3. Comunica al PM cuando los smoke tests estén OK → **Entregable:** Confirmación de OK

### Si eres Dev:
1. Ejecuta el proceso validado en el rehearsal paso a paso — **NO improvises ni te saltes pasos** → **Entregable:** Cada paso ejecutado y confirmado
2. Comunica progreso en tiempo real al TL y BC → **Entregable:** Updates en el canal acordado
3. Realiza el backup post-migración (punto cero) ANTES de dar acceso a usuarios → **Entregable:** Backup completo confirmado
4. Rellena y envía Gate 6 → **Entregable:** Gate 6 para aprobación

### Si eres COO/CTO:
1. Disponible para escalado de emergencia → **Entregable:** Decisión si se escala
2. CTO Ops firma Gate 6 junto con TL → **Entregable:** Gate 6 aprobado

---

## Fase 7 — Soporte Post-migración y Cierre

### Si eres PM:
1. Mantiene el plan de rollback activo durante 2 semanas → **Entregable:** Plan activo
2. Coordina la retrospectiva al final de las 2 semanas → **Entregable:** Reunión de retrospectiva convocada y realizada
3. Documenta lecciones aprendidas → **Entregable:** Documento de retrospectiva
4. Actualiza esta guía con los aprendizajes → **Entregable:** Guía actualizada
5. Rellena y envía Gate 7 al COO → **Entregable:** Gate 7 para aprobación

### Si eres TL:
1. Revisa logs del sistema diariamente durante la primera semana → **Entregable:** Revisión diaria documentada
2. Lidera daily standups con el equipo la primera semana → **Entregable:** Dailys realizados
3. Lidera la retrospectiva al cierre → **Entregable:** Retrospectiva completada
4. Firma Gate 7 → **Entregable:** Gate 7 firmado

### Si eres BC:
1. Es el canal prioritario del cliente para incidencias → **Entregable:** Respuesta inmediata a incidencias
2. Filtra incidencias: funcional vs técnico → **Entregable:** Incidencias clasificadas y derivadas
3. Coordina con Dev para resolución rápida → **Entregable:** Incidencias resueltas

### Si eres Dev:
1. Disponibilidad prioritaria durante 2 semanas → **Entregable:** Disponibilidad confirmada
2. Resuelve incidencias técnicas con SLA reducido → **Entregable:** Incidencias resueltas con tiempo de respuesta documentado
3. Documenta cada incidencia y su solución → **Entregable:** Log de incidencias post-migración

### Si eres COO/CTO:
1. COO aprueba el cierre formal del proyecto → **Entregable:** Gate 7 aprobado
2. CTO consultado en la retrospectiva → **Entregable:** Feedback aportado

---

## Regla de Oro (aplica a TODAS las fases)

> Ningún perfil debe esperar más de 48h por una respuesta que bloquee su trabajo.
> Cadena de escalación: Dev → BC (24h) → TL → PM (48h) → COO/CTO.
