# Workflows del TL

Detalle de cada workflow. El SKILL.md contiene la tabla resumen con triggers.
Leer solo el workflow relevante al contexto actual.

---

## WF0: Entender la Necesidad (SIEMPRE PRIMERO)

> Las tareas de consultores mezclan contexto histórico, tickets, y la necesidad real.
> **No estimar hasta tener la necesidad clara y validada.**

1. Leer la tarea de Odoo y tickets/tareas relacionados
2. Extraer la necesidad en **1 frase**: "{Quién} necesita {qué} que hoy no puede hacer"
3. **Verificar coherencia**: ¿el contexto histórico (tareas previas, tickets) es coherente con la necesidad?
   - **Coherente** → el contexto aporta, usarlo para enriquecer el análisis
   - **Divergente** → avisar al TL: "La tarea referencia X pero la necesidad real parece ser Y, que no tiene relación directa". No asumir que el consultor tiene razón sobre la causa
4. Si no puedes resumirlo en 1 frase o hay incoherencia → **preguntar al TL** antes de seguir
5. Validar con el TL: "¿La necesidad real es X?" — si dice no, iterar antes de estimar

---

## WF0.5: Validar Solicitud de Desarrollo → delega a `dev-task-writer`

> El intake (template + checklist mínimo) está en **`dev-task-writer`** (Step 0).

1. Si llega solicitud de consultor/PM/cliente → invocar `dev-task-writer` para validar contra el template
2. Si la solicitud está incompleta, `dev-task-writer` devuelve el template vacío al solicitante
3. Si está completa → extraer la necesidad en 1 frase (WF0) y continuar con WF1
4. Si propone solución técnica → reformular: "Entiendo que necesitas {problema}. La solución técnica la evaluamos nosotros."

> **Cuándo usarlo**: siempre que llega una solicitud de consultor, PM o cliente.
> **Cuándo NO**: si el TL ya tiene clara la necesidad (ir directo a WF0→WF1).

---

## WF1: Decisión Técnica

1. Recopilar contexto (versión, módulos, restricciones)
2. Leer `decision-frameworks.md` → aplicar pirámide anti-código
3. Si el módulo Core **existe pero no cubre** el caso → es **mejora de Core** (ver tabla de coste)
4. Si duda → scorecard ponderado. Clasificar One-Way/Two-Way Door
5. Generar ADR con `adr-template.md` si relevante

### Quién paga: Core vs Custom

| Situación | Dónde va el código | Quién paga |
|-----------|-------------------|------------|
| No existe en Core, es horizontal | Módulo Core nuevo | Cliente que lo solicita |
| Core existe, no cubre el caso, mejora horizontal | Mejora del Core existente | Cliente que lo solicita |
| Core existe, caso muy específico del cliente | Módulo custom del cliente | Cliente |
| Bug en Core | Fix en Core | Indaws (soporte) |

> **Regla**: que se incorpore al Core beneficia a Indaws (menos deuda técnica), pero el esfuerzo lo asume el cliente que lo motiva.

---

## WF2: Estimación de Esfuerzo

> Leer `effort-estimation-guide.md` ANTES. Capacidad real: 28h/sem (no 40h).

1. Descomponer en componentes atómicos (horas base: delega en la skill `odoo-effort-estimator`)
2. Estimar horas base → aplicar factores riesgo (máx acumulado x3.0)
3. Tres escenarios: optimista / esperado / pesimista (esperado×1.5)
4. Aplicar **factor IA** (delega en la skill `odoo-effort-estimator`)
5. **Cifra única**: esfuerzo real con factor IA + consultoría + buffer iteraciones
6. Generar con `effort-estimation.md`. Comunicar rango, no número único

---

## WF-Valoración: Valoración Completa

> El caso más frecuente del TL. Usar cuando llega una tarea para valorar.

1. **Contexto cliente**: buscar `memory/client_{nombre}.md`. Si no existe → generarlo primero
2. **WF0**: Entender necesidad → verificar coherencia contexto (con el perfil del cliente como referencia)
3. **WF1**: Decidir Core/Custom → quién paga (sabiendo qué tiene ya instalado el cliente)
4. **WF2**: Estimar (cifra única, con factor IA)
5. Generar documento .md con `effort-estimation.md` incluyendo: decisión + desglose + total
6. **Una vez validado por el TL** → generar PDF corporativo con `/indaws-brand` para entregar al consultor/PM

---

## WF3: Redacción de Tarea → delega a `dev-task-writer`

> Movido a `common-skills:dev-task-writer`. Este WF se reduce a invocar la skill federada.

1. Tras WF1 (decisión) + WF2 (estimación), invocar **`dev-task-writer`** pasando:
   - Resultado de la decisión (Standard/Studio/Custom + módulos Core implicados)
   - Estimación (cifra única, con factor IA)
   - Nivel del dev asignado (Junior/Mid/Senior)
   - Tipo de tarea (estándar / migración / bug fix)
2. La skill genera el markdown con las 7 secciones, mapea a `project.task` y opcionalmente crea vía `odoo-pilot`
3. El TL revisa el markdown antes de aprobar la creación o el PDF corporativo

---

## WF4: Code Review

1. Leer `code-review-standards.md` → revisar Python/XML/JS/Seguridad/Tests
2. Clasificar hallazgos (Blocker→Info). Generar con `code-review-checklist.md`

---

## WF5: Deuda Técnica

1. Leer `technical-debt.md` → evaluar 6 tipos (1-5), score global
2. Top 3 prioritarios + plan acción. Generar con `technical-debt-report.md`

---

## WF6: Migración

1. Leer `migration-guide.md` + `version-changes.md`
2. Inventariar módulos (sugerir `/odoo-database-analyzer`)
3. Por módulo: migrar/eliminar/reemplazar. Generar con `migration-assessment.md`
4. **Planificar bloques de trabajo** si hay múltiples módulos:
   - Construir árbol de dependencias por capas (Capa 0 = ya migrado, Capa 1 = solo depende de Capa 0, etc.)
   - Identificar qué módulos se pueden paralelizar (misma capa = paralelos)
   - Asignar devs por cadena completa (un dev = una cadena, evitar context-switching)
   - Definir orden estricto de entrega y puntos de sincronización entre capas
   - Reglas anti-gap: merge diario, Gate 1 en paralelo, buffer entre capas
5. **Por cada módulo a migrar** → invocar **`dev-task-writer`** en modo migración. La skill aplica los 5 gates obligatorios, genera el inventario, los breaking changes, los pasos con skills asignadas y el JSON API listo para `odoo-pilot`. Nombre fijo: `[MIG-V{Y}] {módulo} (Capa {N} — {grupo})`.
6. Si versión destino no publicada → fase de preparación anticipada (tests + auditoría + tracking breaking changes)

---

## WF-Workload: Estado del Equipo

> **El más usado día a día.** "¿Cómo van mis devs? ¿Alguien se queda sin trabajo?"

1. Ejecutar: `python3 scripts/team_workload.py 28`
2. Revisar **ALERTAS** primero:
   - `SIN_TAREAS_PENDIENTES` → dev ya acabó todo, necesita trabajo YA
   - `NADA_PLANIFICADO` → no tiene nada asignado esta semana
   - `PROX_LIBRE(Xh)` → le quedan X horas, preparar siguiente tarea
3. Revisar progreso por dev: slots pendientes vs imputado
4. Mirar semana siguiente: ¿hay devs sin planificar?
5. **Acciones**:
   - Si alguien sin trabajo → asignar tarea pendiente o adelantar del backlog
   - Si alguien próximamente libre → tener tarea preparada (no esperar a que pregunte)
   - Si semana siguiente vacía → lanzar `/team-task-assigner`

---

## WF7: Calibración de Estimaciones

> Usar `estimation_history.py` para validar/ajustar las tablas de estimación.

1. Ejecutar: `python3 scripts/estimation_history.py --team-id 28 --months 6`
2. Analizar ratio global: >1.15 subestimamos, <0.85 sobreestimamos
3. Analizar por tamaño (XS/S/M/L/XL): detectar rangos problemáticos
4. Analizar por dev: detectar patrones individuales (junior subestima, senior sobreestima)
5. Ajustar factores de riesgo en WF2 según datos reales
6. Re-ejecutar cada trimestre para recalibrar

---

## WF8: 1-on-1 con Devs

> Reuniones quincenales de seguimiento individual.

1. Preparar datos del dev: horas, tickets, PRs (via `/dev-assistant` y `/ticket-report`)
2. Consultar skill matrix: `python3 skill_matrix.py show` (en team-task-assigner)
3. Rellenar template `one-on-one.md` con datos reales
4. Temas obligatorios: trabajo actual, crecimiento técnico, feedback bidireccional
5. Action items con fecha → seguimiento en próxima 1-on-1

---

## WF9: Dashboard Semanal

> Visión consolidada del equipo para la semana.

1. Ejecutar en paralelo:
   - `standup_snapshot.py 28` → estado actual
   - `forecast.py 28 --weeks 4` → proyección (en team-task-assigner)
   - `estimation_history.py --months 3` → calibración reciente
   - `skill_matrix.py report` → cobertura skills (en team-task-assigner)
2. Rellenar template `tl-dashboard.md` con los datos
3. Identificar top 3 action items para la semana
4. Compartir resumen con equipo/stakeholders si procede

---

## WF10: Pre-check Code Review (quality-gate)

> Leer `quality-gate-integration.md`. Automatizar primera pasada.

1. Dev marca tarea "En Review" → sugerir al TL: "¿Ejecuto /quality-gate?"
2. Si acepta → ejecutar `/quality-gate` sobre el módulo
3. Si hay Blockers → devolver al dev sin review manual
4. Si limpio → TL hace review manual (WF4) focalizando en lógica de negocio
5. Resultado: TL ahorra tiempo en issues mecánicos

---

## WF-Eval: Evaluacion de Desempeno

> Leer `performance-evaluation-guide.md` ANTES. Proceso bajo demanda — solo cuando el TL lo solicita.

Proceso de 5 subpasos. El TL puede pedir cualquiera de forma independiente o el flujo completo.

### Subpaso 1: Generar formulario de evaluacion

> Trigger: "preparar evaluacion de {nombre}", "formulario evaluacion"

1. Identificar al empleado y su rol (Junior/Mid/Senior/TL)
2. Generar formulario con `evaluation-form.md` adaptado:
   - **Autoevaluacion**: primera persona, para que lo rellene el empleado
   - **Evaluacion TL**: tercera persona, para que lo rellene el TL
3. Guardar como `evaluaciones/{nombre}_{periodo}_autoevaluacion.md` y `evaluaciones/{nombre}_{periodo}_evaluacion_tl.md`

### Subpaso 2: Preparar evaluacion del TL con datos

> Trigger: "rellenar mi evaluacion de {nombre}", "datos para evaluar a {nombre}"

1. Recopilar datos objetivos del empleado:
   - Horas imputadas y ratio facturabilidad → `/dev-assistant` o `/odoo-pilot`
   - Tickets resueltos y prioridad → `/ticket-report`
   - Tareas completadas, en curso, bloqueadas → `/odoo-pilot`
   - Ausencias del periodo → `/odoo-pilot`
2. Presentar datos al TL en formato tabla
3. Aconsejar al TL sobre como rellenar cada seccion:
   - **S1**: "Basandote en estos datos, las 3 contribuciones mas relevantes parecen ser..."
   - **S2**: "Los datos sugieren un nivel de autonomia {X} porque..."
   - **S3**: "Sobre mejora de procesos, hay evidencia de {X}..."
   - **S4**: "Las necesidades que el empleado ha expresado en 1-on-1s previas son..."
   - **S5**: "Dado su trayectoria, un siguiente paso razonable seria..."
4. **NUNCA rellenar por el TL** — solo sugerir y aportar datos. El TL decide

### Subpaso 3: Analizar alineacion

> Trigger: "comparar evaluaciones de {nombre}", "analisis alineacion {nombre}"

1. Leer ambas evaluaciones (empleado + TL)
2. Comparar seccion por seccion:
   - Escalas: calcular gap numerico (0, 1, 2+ niveles)
   - Textos: analizar si hablan de lo mismo o divergen
3. Generar informe con `evaluation-alignment.md`:
   - Resumen ejecutivo con semaforo por seccion
   - Detalle con analisis y puntos de conversacion
   - Desalineaciones priorizadas (mayor gap primero)
   - Puntos fuertes alineados (para empezar la reunion)
   - Input para objetivos (que areas trabajar)
4. Aconsejar al TL sobre como abordar cada desalineacion:
   - Si gap >= 2 en escala → "Esto requiere conversacion directa con datos concretos"
   - Si empleado se sobrevalora → "Preguntale por evidencias antes de presentar las tuyas"
   - Si TL fue vago → "Necesitas prepararte mejor en esta seccion antes de la reunion"

### Subpaso 4: Definir objetivos

> Trigger: "objetivos para {nombre}", "definir objetivos evaluacion"

1. Leer informe de alineacion (si existe) o preguntar por las areas clave
2. Determinar reparto segun perfil:
   - Junior: 3 objetivos (2 ejecucion + 1 aprendizaje)
   - Mid: 3-4 (2 ejecucion + 1 calidad + 1 integracion)
   - Senior: 4 (1 liderazgo + 2 ejecucion senior + 1 integracion/IA)
   - TL: 4 (1 gestion + 1 planificacion + 1 mejora + 1 estrategico)
3. Proponer objetivos con `evaluation-objectives.md`:
   - Cada objetivo alineado con un gap detectado o area de crecimiento
   - Motivo explicito ("por que ESTE objetivo para ESTA persona AHORA")
   - Evidencias medibles y concretas
   - Plan 30/60/90 progresivo
4. Presentar como **borrador** — el TL ajusta antes de la reunion
5. Los objetivos se negocian con el empleado en la reunion

### Subpaso 5: Preparar reunion de formalizacion

> Trigger: "preparar reunion evaluacion {nombre}", "guia reunion {nombre}"

1. Generar guia de reunion con `evaluation-meeting.md`:
   - Agenda estructurada (30-45 min)
   - Puntos alineados para abrir (reforzar confianza)
   - Desalineaciones ordenadas de menor a mayor sensibilidad
   - Para cada desalineacion: dato objetivo + pregunta abierta para el empleado
   - Tabla resumen de objetivos
   - Espacio para compromisos mutuos
2. Aconsejar al TL sobre como manejar situaciones dificiles:
   - Empleado a la defensiva → escuchar, validar, datos
   - Sobrevaloracion → preguntar por evidencias primero
   - Desmotivacion → priorizar escucha
   - Objetivos rechazados → negociar, no imponer
3. Recordar errores a evitar:
   - No comparar empleados entre si
   - No prometer lo que no puedes cumplir
   - Ratio 3:1 (positivo:constructivo)
   - No evaluar solo por recencia
