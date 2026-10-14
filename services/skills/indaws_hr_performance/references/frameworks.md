# Frameworks de Evaluacion - Detalle Operativo

**CONFIDENTIAL - HR TEAM ONLY**

Detalle de las matrices, plantillas y scorecards que el skill `hr-performance` resume.
Complementa a `review_framework.md` (escala/ciclos), `competency_model.md` (competencias por
nivel) y `career_ladder.md` (niveles IC1-IC6 / M1-M3).

---

## 1. Matriz de Calibracion 9-box

Dos ejes, tres niveles cada uno. La posicion REQUIERE rating de **Rendimiento** (resultados del
periodo) Y rating de **Potencial** (capacidad de crecer de nivel). Si falta uno, el empleado **no
es ubicable** en la matriz: el skill debe detenerse y pedir el dato faltante.

| | Potencial Bajo | Potencial Medio | Potencial Alto |
|---|---|---|---|
| **Rendimiento Alto** | Trabajador solido (retener) | Alto rendidor (desarrollar) | Estrella (promocion) |
| **Rendimiento Medio** | Profesional efectivo | Core (invertir) | Emergente (estirar) |
| **Rendimiento Bajo** | Riesgo (PIP) | Inconsistente (diagnosticar) | Enigma (recolocar/coaching) |

Acciones por cuadrante: promocion (Estrella), plan de desarrollo (Alto rendidor / Emergente),
retencion (Trabajador solido), PIP (Riesgo), diagnostico de encaje (Inconsistente / Enigma).
Validar distribucion global frente a una bell curve razonable (no forzada).

---

## 2. Review Prep Scorecard (manager)

Estructura del documento de preparacion que genera el skill:
1. Contexto: nivel, antiguedad, equipo, manager evaluador, periodo y tipo de review.
2. Objetivos del periodo: progreso real por objetivo (% y estado on-track / at-risk / off-track)
   y progreso global. Fuente: HiBob Goals.
3. Evaluacion por competencia segun nivel del empleado, escala 1-5 (ver `review_framework.md`).
4. Comparativa con appraisals anteriores (tendencia: mejorando / estable / declinando).
   Fuente: Odoo hr.appraisal.
5. Fortalezas y areas de desarrollo.
6. Recomendaciones para el manager (puntos a discutir en la reunion).
7. Propuesta de calificacion global.
8. Alineacion self-assessment vs vision manager: marcar discrepancias para discutir.

---

## 3. Self-Assessment Guide (empleado)

1. Instrucciones para el empleado.
2. Objetivos del periodo con espacio para autoevaluacion.
3. Preguntas guia por competencia.
4. Logros y contribuciones.
5. Areas de mejora y desarrollo.
6. Aspiraciones de carrera.

La autoevaluacion y la evaluacion del manager son perspectivas independientes; ambas se cruzan en
el review prep. Una desalineacion fuerte (p. ej. empleado se puntua 5 y manager 2) es senal de
conversacion previa, no se "promedia" silenciosamente.

---

## 4. Plantilla de PIP (Performance Improvement Plan)

Estructura (ver tambien `templates/pip_template.md`):
1. Datos del empleado y contexto.
2. Areas de mejora claramente definidas.
3. Acciones SMART con plazos y responsables.
4. Hitos de revision: 30, 60 y 90 dias.
5. Criterios de exito medibles.
6. Consecuencias si no se alcanzan los objetivos.
7. Firma de empleado y manager.

Duracion estandar: **90 dias** (configurable). El PIP es un documento sensible RGPD/legal: lo
revisan RRHH y asesor laboral ANTES de entregarlo al empleado. Acciones previas (feedback,
coaching) deben constar como antecedente.

---

## 5. Cycle Status Dashboard

Formato tabla:
- % completitud del ciclo por departamento (objetivo: 95%, `REVIEW_COMPLETION_TARGET_PCT`).
- Reviews pendientes con dias de retraso.
- Reviews completadas vs objetivo.
- Alertas: reviews vencidas, managers sin completar.

---

## 6. Promotion Readiness (gap analysis)

1. Gap analysis: nivel actual vs nivel objetivo (ver `career_ladder.md`).
2. Competencias cumplidas vs pendientes para el nivel superior.
3. Evidencias de desempeno ya al nivel superior.
4. Recomendacion: Listo / Casi listo / Necesita desarrollo.
5. Plan de desarrollo para cerrar gaps.

Requisitos de promocion: **minimo 12 meses** en el nivel actual y **dos ciclos consecutivos** con
puntuacion >= 4. El career ladder es **por familia de puesto**: un IC4 de ingenieria y un IC4
funcional no comparten expectativas; usar el track correcto al evaluar readiness.

---

## 7. Umbrales de configuracion

| Constante (`lib/config.py`) | Valor por defecto | Uso |
|---|---|---|
| `REVIEW_COMPLETION_TARGET_PCT` | 95 | Objetivo de completitud del ciclo |
| `GOAL_COMPLETION_WARNING_PCT` | 50 | Progreso de objetivos < umbral → warning |
| Duracion PIP | 90 dias | Hitos a 30/60/90 |
