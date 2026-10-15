# Plantilla: Candidate Brief para Hiring Manager

**CONFIDENTIAL - HR TEAM ONLY**

Resumen de candidato para presentar al hiring manager antes de la entrevista
o para la reunion de decision. Generado por `candidate_scorer.py`.

---

## Datos del Candidato

| Campo | Valor |
|-------|-------|
| Nombre | {nombre_candidato} |
| Puesto al que aplica | {puesto} |
| Departamento | {departamento} |
| Nivel objetivo | {nivel_ic_m} |
| Fuente | {fuente_captacion} |
| Fecha de aplicacion | {fecha_aplicacion} |
| Recruiter asignado | {nombre_recruiter} |

---

## Scoring del Candidato

**Puntuacion total: {score_total}/100** - {recomendacion}

| Dimension | Peso | Puntuacion | Ponderado |
|-----------|------|-----------|-----------|
| Experiencia (experience_match) | 30% | {score}/100 | {pond} |
| Competencias (skills_match) | 25% | {score}/100 | {pond} |
| Cultura (culture_fit) | 20% | {score}/100 | {pond} |
| Formacion (education) | 15% | {score}/100 | {pond} |
| Disponibilidad (availability) | 10% | {score}/100 | {pond} |

### Clasificacion
- **75-100**: Recomendado - Candidato fuerte, alineado con el perfil
- **50-74**: Considerar - Candidato con potencial, requiere evaluacion adicional
- **0-49**: No recomendado - Desalineamiento significativo con el perfil

---

## Perfil Profesional

### Experiencia Relevante
{resumen_experiencia}

### Formacion
{resumen_formacion}

### Competencias Tecnicas Destacadas
- {competencia_1}
- {competencia_2}
- {competencia_3}

### Idiomas
| Idioma | Nivel |
|--------|-------|
| {idioma_1} | {nivel_1} |

---

## Resumen de Entrevistas

| Fecha | Tipo | Entrevistador | Puntuacion | Recomendacion |
|-------|------|---------------|-----------|---------------|
| {fecha_1} | {tipo_1} | {entrev_1} | {punt_1}/4 | {rec_1} |

### Notas Clave de Entrevistas
{notas_consolidadas}

---

## Analisis de Fortalezas y Riesgos

### Fortalezas
- {fortaleza_1}
- {fortaleza_2}
- {fortaleza_3}

### Areas de Riesgo / Desarrollo
- {riesgo_1}
- {riesgo_2}

### Banderas Rojas (si aplica)
- {flag_1}

---

## Comparativa con el Perfil Ideal

| Requisito del Puesto | Candidato | Match |
|----------------------|-----------|-------|
| {requisito_1} | {cumple_candidato} | {si/parcial/no} |
| {requisito_2} | {cumple_candidato} | {si/parcial/no} |

---

## Informacion Logistica

| Item | Detalle |
|------|---------|
| Disponibilidad para incorporacion | {fecha_disponible} |
| Preaviso actual | {dias_preaviso} |
| Ubicacion preferida | {ubicacion} |
| Modalidad de trabajo preferida | {presencial/remoto/hibrido} |

> **NOTA**: La informacion salarial NO se incluye en este documento (GDPR).
> Consultar con RRHH para detalles de compensacion.

---

## Recomendacion del Recruiter

{texto_recomendacion_recruiter}

**Decision sugerida**: {Avanzar a siguiente etapa / Oferta / Rechazar / Pendiente}

---

**Brief generado**: {fecha_generacion}
**Generado por**: Skill hr-recruitment v1.0.0
