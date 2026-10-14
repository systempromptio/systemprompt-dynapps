# Template: Informe de Progreso para el Cliente (por hito)

> Usar cuando el BC o PM necesita comunicar al cliente el estado de la migración.
> Se envía por email formal tras cada hito significativo (aprobación de gate, cambio de fase, etc.).
> Ver `references/communication.md` para protocolo completo.
> IDIOMA: Siempre en español. CERO jerga técnica.

---

## Informe de Progreso — {nombre_cliente}

**Fecha:** {fecha}
**Proyecto:** Actualización de su sistema Odoo
**Su contacto:** {nombre_bc_o_pm} ({email})

---

### Estado general

**Fase actual:** {descripcion_fase_en_lenguaje_cliente}
**Progreso global:** {porcentaje_simplificado}%

> {frase_resumen_1_linea}
> Ejemplo: "Hemos completado la preparación técnica y estamos adaptando los módulos de su sistema."

---

### Qué hemos completado

- {hito_completado_1}
- {hito_completado_2}

---

### En qué estamos trabajando ahora

{descripcion_breve_trabajo_actual}

---

### Próximos pasos

| Paso | Fecha estimada |
|------|---------------|
| {paso_1} | {fecha_1} |
| {paso_2} | {fecha_2} |

---

### Necesitamos de su parte

{lista_acciones_o_"No se requiere ninguna acción por su parte en este momento."}

| # | Acción | Fecha límite | Contacto |
|---|--------|-------------|----------|
| 1 | {accion} | {fecha} | {nombre} |

---

### Fecha estimada de finalización

**{fecha_estimada}**

> {nota_condiciones: "Esta fecha está sujeta a la disponibilidad para las sesiones de validación."}

---

{nombre_bc_o_pm}
{rol} — {empresa}
{telefono} | {email}

---

## Guía de adaptación por fase

El contenido del informe cambia según la fase del proyecto. Usar esta tabla para decidir qué secciones enfatizar y qué lenguaje emplear.

### Fase 0 (Kick-off / Propuesta)
- **Énfasis:** Explicar el proceso completo, plazos generales, qué se espera del cliente
- **Traducción:** "Análisis inicial" en vez de "inventario de módulos custom"
- **Acciones cliente:** Firmar documento de alcance, asignar interlocutores

### Fase 1 (Preparación)
- **Énfasis:** "Estamos preparando todo para la migración"
- **Traducción:** "Análisis técnico de su sistema" en vez de "clasificación de módulos y análisis de APIs deprecadas"
- **Acciones cliente:** Validar flujos de trabajo documentados por el BC

### Fase 2 (Migración de código)
- **Énfasis:** Progreso de adaptación, posibles decisiones funcionales pendientes
- **Traducción:** "Adaptación de los módulos de su sistema" en vez de "migración de código, pipeline CI/CD"
- **Acciones cliente:** Responder consultas funcionales del BC (decisiones de negocio)

### Fase 3 (Migración de BD)
- **Énfasis:** "Estamos trasladando sus datos al nuevo sistema"
- **Traducción:** "Migración y validación de datos" en vez de "upgrade de BD, scripts pre/post-migrate"
- **Acciones cliente:** Ninguna normalmente

### Fase 4 (Testing — CRÍTICA)
- **Énfasis:** Necesidad de participación activa del cliente para validar
- **Traducción:** "Sesiones de validación conjuntas" en vez de "testing funcional en pre-producción"
- **Acciones cliente:** Asistir a sesiones de validación, probar flujos, firmar acta de conformidad

### Fase 5 (Ensayo)
- **Énfasis:** Fecha de puesta en producción, ventana de mantenimiento
- **Traducción:** "Simulación final antes de la puesta en marcha" en vez de "rehearsal/dry-run"
- **Acciones cliente:** Confirmar fecha de Go-Live, preparar a sus usuarios

### Fase 6 (Go-Live)
- **Énfasis:** Ventana de mantenimiento, qué esperar, soporte post
- **Traducción:** "Puesta en marcha del nuevo sistema" en vez de "Go-Live, smoke tests"
- **Acciones cliente:** No usar el sistema durante la ventana, confirmar operación normal después

### Fase 7 (Soporte post)
- **Énfasis:** Canal de soporte, periodo de atención prioritaria, cierre
- **Traducción:** "Periodo de acompañamiento" en vez de "soporte post-migración"
- **Acciones cliente:** Reportar incidencias por el canal prioritario

---

## Tabla de traducción técnico → cliente

| Término interno | Traducción para el cliente |
|----------------|---------------------------|
| Gate aprobado | Hito completado |
| Pipeline CI/CD green | Las pruebas automáticas confirman que todo funciona |
| Migración de código | Adaptación de los módulos de su sistema |
| Migración de BD | Traslado de sus datos al nuevo sistema |
| Testing funcional | Sesiones de validación conjuntas |
| Rehearsal / Dry-run | Simulación final antes de la puesta en marcha |
| Go-Live | Puesta en marcha del nuevo sistema |
| Rollback | Restauración de la versión anterior (medida de seguridad) |
| Scripts pre/post-migrate | Procesos automáticos de conversión de datos |
| Smoke tests | Comprobaciones rápidas de funcionamiento |
| Entorno de testing/staging | Entorno de pruebas (copia segura de su sistema) |
| Acta de conformidad | Documento de validación por su parte |
| Bug/fix | Incidencia / corrección |
| Code review | Revisión de calidad del código |
| Sprint/iteración | Ciclo de trabajo |

---

## Reglas de formato

- **IDIOMA:** Siempre en español
- **EXTENSIÓN:** Máximo 1 página (~400 palabras). Si hay mucho que contar, resumir y ofrecer una reunión
- **TONO:** Profesional, cercano, sin jerga. El cliente no es técnico
- **JERGA:** Usar SIEMPRE la tabla de traducción de arriba. Si aparece un término técnico no listado, traducirlo antes de incluirlo
- **FRECUENCIA:** Por hito (aprobación de gate, cambio de fase, evento relevante). No semanal — eso es comunicación interna
- **CANAL:** Email formal (CC al PM siempre)
- **SECCIÓN "Necesitamos de su parte":** Es la más importante para el cliente. Debe ser accionable y con fechas. Si no hay acciones, indicarlo explícitamente
- **DATOS DE F8:** Si hay datos de `calculate_progress.py` disponibles, simplificar a % de progreso y fecha estimada. NO incluir horas, tareas individuales ni métricas internas
- **PRE-RELLENAR:** Si hay contexto del proyecto (nombre, fase, cliente), pre-rellenar lo que se pueda
