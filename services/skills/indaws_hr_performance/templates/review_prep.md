# Plantilla: Review Prep para Manager

**CONFIDENTIAL - HR TEAM ONLY**

Documento de preparacion de evaluacion de desempeno para uso del manager. Generado
automaticamente por `review_generator.py` con datos de HiBob y Odoo.

---

## Datos del Empleado

| Campo | Valor |
|-------|-------|
| Nombre | {nombre_empleado} |
| Departamento | {departamento} |
| Puesto | {puesto} |
| Nivel | {nivel_ic_m} |
| Fecha de incorporacion | {fecha_incorporacion} |
| Antiguedad | {antiguedad} |
| Manager | {nombre_manager} |
| Periodo de evaluacion | {periodo} |
| Tipo de review | {anual / semestral / prueba / ad-hoc} |

---

## 1. Progreso de Objetivos (50% del peso)

| # | Objetivo | Peso | Progreso | Estado | Puntuacion |
|---|----------|------|----------|--------|-----------|
| 1 | {objetivo_1} | {peso_1}% | {progreso_1}% | {on-track/at-risk/off-track} | {1-5} |
| 2 | {objetivo_2} | {peso_2}% | {progreso_2}% | {estado_2} | {1-5} |
| 3 | {objetivo_3} | {peso_3}% | {progreso_3}% | {estado_3} | {1-5} |

**Progreso global**: {progreso_global}%
**Puntuacion media de objetivos**: {media_objetivos}/5

### Notas sobre Objetivos
- Objetivos alcanzados: {lista_alcanzados}
- Objetivos en riesgo: {lista_en_riesgo}
- Objetivos no alcanzados: {lista_no_alcanzados}
- Factores externos relevantes: {factores}

---

## 2. Evaluacion de Competencias (30% del peso)

Evaluadas contra las expectativas del nivel **{nivel_ic_m}** segun el modelo
de competencias (`references/competency_model.md`).

### Competencias Core

| Competencia | Puntuacion | Evidencias |
|-------------|-----------|------------|
| Comunicacion | {1-5} | {evidencias} |
| Trabajo en equipo | {1-5} | {evidencias} |
| Orientacion a resultados | {1-5} | {evidencias} |
| Adaptabilidad | {1-5} | {evidencias} |
| Resolucion de problemas | {1-5} | {evidencias} |

### Competencias de Liderazgo (Solo M1+)

| Competencia | Puntuacion | Evidencias |
|-------------|-----------|------------|
| Gestion de equipos | {1-5} | {evidencias} |
| Vision estrategica | {1-5} | {evidencias} |
| Desarrollo de personas | {1-5} | {evidencias} |
| Toma de decisiones | {1-5} | {evidencias} |

**Puntuacion media de competencias**: {media_competencias}/5

---

## 3. Valores y Cultura (20% del peso)

| Valor | Puntuacion | Evidencias |
|-------|-----------|------------|
| Colaboracion | {1-5} | {evidencias} |
| Innovacion | {1-5} | {evidencias} |
| Excelencia | {1-5} | {evidencias} |
| Integridad | {1-5} | {evidencias} |

**Puntuacion media de valores**: {media_valores}/5

---

## 4. Puntuacion Global

| Componente | Peso | Puntuacion | Ponderado |
|-----------|------|-----------|-----------|
| Objetivos | 50% | {media_obj}/5 | {pond_obj} |
| Competencias | 30% | {media_comp}/5 | {pond_comp} |
| Valores | 20% | {media_val}/5 | {pond_val} |
| **TOTAL** | **100%** | | **{total}/5** |

**Calificacion global**: {calificacion} ({1-No Cumple / 2-Parcial / 3-Cumple / 4-Excede / 5-Excepcional})

---

## 5. Comparativa con Evaluaciones Anteriores

| Periodo | Puntuacion | Tendencia |
|---------|-----------|-----------|
| {periodo_anterior_2} | {punt_2}/5 | - |
| {periodo_anterior_1} | {punt_1}/5 | {subio/bajo/estable} |
| {periodo_actual} | {punt_actual}/5 | {subio/bajo/estable} |

**Tendencia general**: {mejorando / estable / declinando}

---

## 6. Fortalezas y Areas de Desarrollo

### Fortalezas (Top 3)
1. {fortaleza_1}: {descripcion}
2. {fortaleza_2}: {descripcion}
3. {fortaleza_3}: {descripcion}

### Areas de Desarrollo (Top 3)
1. {area_1}: {descripcion} | Accion sugerida: {accion}
2. {area_2}: {descripcion} | Accion sugerida: {accion}
3. {area_3}: {descripcion} | Accion sugerida: {accion}

---

## 7. Guia para la Conversacion de Review

### Puntos a Discutir con el Empleado
1. {punto_1}
2. {punto_2}
3. {punto_3}

### Preguntas para el Empleado
- Que logros destacarias de este periodo?
- Que obstaculos encontraste y como los gestionaste?
- Que necesitas de mi como manager para mejorar?
- Cuales son tus aspiraciones de carrera para los proximos 12 meses?
- Hay algo que cambiarias del equipo o los procesos?

### Feedback Constructivo (areas de mejora)
- Estructura: Situacion + Comportamiento + Impacto + Expectativa
- {feedback_1}
- {feedback_2}

### Reconocimiento (fortalezas)
- {reconocimiento_1}
- {reconocimiento_2}

---

## 8. Recomendaciones

| Item | Recomendacion |
|------|---------------|
| Promocion | {si/no/considerar en proximo ciclo} |
| Revision salarial | {si/no - consultar con RRHH} |
| Plan de desarrollo | {areas especificas} |
| PIP necesario | {si/no} |
| Riesgo de salida | {bajo/medio/alto} |
| Posicion 9-box | {cuadrante} |

---

**Documento generado**: {fecha_generacion}
**Generado por**: Skill hr-performance v1.0.0
**NOTA**: Este documento es para uso interno del manager y RRHH. No compartir con el empleado directamente.
