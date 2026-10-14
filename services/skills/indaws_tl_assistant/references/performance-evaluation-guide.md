# Guia de Evaluacion de Desempeno

Proceso periodico (semestral/anual) para evaluar, alinear y orientar el crecimiento de cada miembro del equipo.

## Filosofia

- La evaluacion **no es un juicio**, es una herramienta de alineacion y crecimiento.
- El TL evalua con **datos objetivos** (horas, tickets, PRs, proyectos), no con impresiones.
- Las desalineaciones no son malas — son **oportunidades de conversacion**.
- Los objetivos deben ser **accionables, medibles y con horizonte 30/60/90**.

## Proceso completo (5 pasos)

| Paso | Quien | Que hace | Output |
|------|-------|----------|--------|
| 1 | Empleado | Rellena autoevaluacion segun su rol | Formulario completado |
| 2 | TL | Rellena la misma evaluacion sobre el empleado | Formulario completado (3a persona) |
| 3 | TL | Analiza alineacion entre ambas evaluaciones | Informe de alineacion |
| 4 | TL | Define objetivos basados en gaps y rol | Objetivos OBJ-XX con 30/60/90 |
| 5 | TL + Empleado | Reunion de formalizacion | Acta con acuerdos |

## Roles y formularios

El formulario base es el mismo para todos los roles del departamento de desarrollo.
Las preguntas estan redactadas en primera persona (autoevaluacion) y se adaptan a tercera persona cuando las rellena el TL.

### Perfiles evaluables

| Perfil | Foco evaluacion | Max objetivos | Reparto recomendado |
|--------|----------------|---------------|---------------------|
| Junior Dev | Aprendizaje, autonomia basica, calidad de codigo | 3 | 2 ejecucion + 1 aprendizaje |
| Mid Dev | Autonomia, calidad, resolucion sin supervision | 3-4 | 2 ejecucion + 1 calidad + 1 integracion |
| Senior Dev (SR) | Liderazgo tecnico, mentoring, prevencion, cross-team | 4 | 1 liderazgo + 2 ejecucion senior + 1 integracion/IA |
| Tech Lead | Gestion equipo, planificacion, calibracion, mejora procesos | 4 | 1 gestion + 1 planificacion + 1 mejora + 1 estrategico |

## Secciones del formulario

### S1: Responsabilidades Clave
**Pregunta**: Detalla cuales consideras que han sido tus 3 principales contribuciones de valor durante este periodo. Centrate en el impacto generado, no en el listado de tareas administrativas.

**Que evalua**: capacidad de identificar impacto real vs trabajo rutinario.

**Senales de alerta para el TL**:
- Empleado lista tareas, no impacto → falta vision de negocio
- Empleado exagera impacto sin datos → necesita feedback sobre evidencias
- TL no puede nombrar 3 contribuciones → posible falta de visibilidad o de impacto real

### S2: Impacto y Autonomia

**Pregunta escala** (seleccion unica):
1. Poco autonomo. He necesitado de validacion y/o de apoyo tecnico o funcional frecuentemente
2. Algo autonomo. Gestiono lo estandar y escalo lo complejo
3. Autonomo. Resuelvo mis funciones sin requerir supervision y garantizo la calidad en mi desempeno
4. Referente. Soy a quien acude el resto del equipo cuando tienen dudas

**Pregunta abierta**: Cual es el logro o resultado concreto de este ano por el que crees que deberias ser reconocido? (basa tu respuesta en datos, hechos o feedback de clientes, no en tiempo de dedicacion)

**Que evalua**: percepcion de autonomia y capacidad de auto-reconocimiento con evidencias.

**Senales de alerta para el TL**:
- Gap de 2+ niveles en autonomia → conversacion prioritaria, percepcion muy desalineada
- Gap de 1 nivel → normal, discutir con ejemplos concretos
- Empleado se puntua bajo → posible falta de confianza o contexto
- TL puntua bajo sin datos → revisar si hay sesgo o falta de visibilidad

### S3: Valores Indaws

**Pregunta escala — Gestion de conflictos** (seleccion unica):
1. Espero a que se resuelva y me aseguro de explicar que no he tenido culpa
2. Mantengo informado a mi responsable directo y espero su decision
3. Informo a mi responsable y equipo, asumo mi parte de responsabilidad y propongo una solucion
4. Gestiono la situacion dificil directamente con el cliente/equipo, me anticipo a las consecuencias/riesgo y genero confianza

**Pregunta escala — Mejora de procesos** (seleccion unica):
1. No. Mi foco ha estado en realizar mi trabajo
2. Si. He ayudado puntualmente, cuando me lo han pedido
3. Si. Comparto mis conocimientos y propongo mejoras a los procesos que realizo, de forma proactiva
4. Si. He creado documentacion y asentado conocimiento que hoy utiliza el equipo o area

**Que evalua**: alineacion con valores de proactividad, ownership y colaboracion.

### S4: Herramientas y Bloqueantes

**Preguntas abiertas**:
- Que necesitas de la compania para rendir mas (recursos/formacion)?
- Que compromiso de mejora personal asumes para el proximo periodo?

**Que evalua**: capacidad de identificar necesidades y asumir compromisos.

**Senales de alerta para el TL**:
- TL responde "no tengo informacion" → **inaceptable**. Debe prepararse con datos antes
- Empleado no identifica necesidades → posible conformismo o falta de ambicion
- Compromisos vagos → ayudar a concretar en la reunion

### S5: Metas y Vision

**Preguntas abiertas**:
- Cual crees que es tu siguiente paso merecido en la compania? (Consolidar puesto, ampliar responsabilidades, gestion de personas...)
- Por que crees que estas preparado? Que competencias o hitos has demostrado?

**Que evalua**: ambicion, autocritica y vision de carrera.

## Criterios de alineacion

| Gap | Nivel | Accion |
|-----|-------|--------|
| 0 | Alineado | Confirmar y reforzar |
| 1 nivel (escalas) | Desalineacion menor | Discutir con ejemplos, buscar acuerdo |
| 2+ niveles (escalas) | Desalineacion critica | Priorizar en reunion, aportar datos objetivos |
| Texto divergente | Percepcion diferente | Explorar causas, no quien tiene razon |

## Datos objetivos para preparar la evaluacion del TL

Antes de rellenar SU evaluacion, el TL debe recopilar:

| Dato | Fuente | Comando/skill |
|------|--------|---------------|
| Horas imputadas por proyecto | `account.analytic.line` | `/odoo-pilot` o `team_workload.py` |
| Tickets resueltos | `helpdesk.ticket` | `/ticket-report` |
| Tareas completadas | `project.task` | `/odoo-pilot` |
| Code reviews realizadas | Git/PRs | Consultar repositorio |
| Ratio facturabilidad | Timesheets | `/dev-assistant` |
| Ausencias/vacaciones | `hr.leave` | `/odoo-pilot` |
| Incidencias criticas resueltas | Tickets prioritarios | `/ticket-report` |
| Feedback de clientes | Comunicaciones | Manual |

## Objetivos: estructura y criterios

### Formato OBJ-XX

```
OBJ-{NN} — {Titulo descriptivo}
Motivo (1 linea): por que este objetivo es importante para este empleado ahora.
Bloques: {Skills / Knowledge / Values} (que dimensiones trabaja)
Que cambia (conducta/entregable):
  - {Cambio observable 1}
  - {Cambio observable 2}
  - {Cambio observable 3}
Evidencias:
  - {Evidencia medible 1}
  - {Evidencia medible 2}
  - {Evidencia medible 3}
30/60/90:
  30: {arrancar — primera accion visible}
  60: {consolidar — consistencia demostrada}
  90: {impacto — resultado visible y medible}
```

### Principios de buenos objetivos

1. **Maximo 4 objetivos** — mas es ruido, menos es falta de ambicion
2. **Reparto por perfil** — ver tabla de perfiles arriba
3. **Motivo siempre explicito** — "por que ESTE objetivo para ESTA persona AHORA"
4. **Evidencias medibles** — numeros, entregas, artefactos verificables
5. **30/60/90 progresivo** — de arranque a impacto, no todo al dia 90
6. **Alineados con gaps** — los objetivos deben atacar las desalineaciones detectadas
7. **Negociables** — el empleado debe poder opinar y ajustar en la reunion

## Reunion de formalizacion: como enfocarla

### Estructura recomendada (30-45 min)

| Bloque | Tiempo | Contenido |
|--------|--------|-----------|
| Apertura | 5 min | Tono positivo. "Esto no es un examen, es alineacion" |
| Puntos alineados | 5 min | Empezar por lo que coincide — refuerza confianza |
| Desalineaciones | 10-15 min | Ir de menor a mayor. Siempre con datos. Escuchar primero |
| Objetivos | 10 min | Presentar, explicar motivo, negociar ajustes |
| Compromisos mutuos | 5 min | Que se compromete el empleado + que se compromete el TL/empresa |
| Cierre | 2 min | Resumen acuerdos, proximos pasos, fecha seguimiento |

### Consejos para conversaciones dificiles

- **Empleado se sobrevalora mucho**: no confrontar directamente. Preguntar "que datos concretos me puedes dar que lo demuestren?" Luego presentar tus datos
- **Empleado se infravalora**: reforzar con datos positivos. "Quizas no lo ves, pero los datos dicen X"
- **Empleado a la defensiva**: parar, escuchar, validar sentimiento. "Entiendo que no esperabas esto. Vamos a verlo juntos"
- **Desalineacion en autonomia**: es la mas sensible. Usar ejemplos concretos (tickets donde necesito ayuda, tareas completadas sin supervision)
- **Objetivos rechazados**: negociar, no imponer. Si no acepta ninguno → escalar a RRHH/CTO
- **Empleado desmotivado**: priorizar escucha. Detectar causa (proyecto, equipo, crecimiento, personal). No forzar objetivos ambiciosos si la base esta rota

### Errores del TL a evitar

1. **No preparar la evaluacion con datos** → responderas vaguedades como "no tengo informacion"
2. **Comparar empleados entre si** → "fulanito si lo hace". NUNCA
3. **Prometer lo que no puedes cumplir** → subidas, cambios de rol sin validar con CTO
4. **Centrar todo en lo negativo** → ratio 3:1 (3 positivos por cada constructivo)
5. **No dar seguimiento** → los objetivos sin seguimiento no existen
6. **Evaluar por recencia** → usar datos de TODO el periodo, no solo el ultimo mes
