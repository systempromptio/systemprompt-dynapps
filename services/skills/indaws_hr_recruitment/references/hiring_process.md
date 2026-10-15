# Proceso de Contratacion End-to-End

**CONFIDENTIAL - HR TEAM ONLY**

Workflow completo de contratacion en Indaws, desde la solicitud de nueva posicion
hasta la incorporacion del candidato seleccionado.

---

## Fases del Proceso

### Fase 1: Solicitud y Aprobacion (Dias 1-5)

| Paso | Responsable | Accion | Entregable |
|------|-------------|--------|------------|
| 1.1 | Hiring Manager | Solicitar nueva posicion via formulario | Requisicion aprobada |
| 1.2 | RRHH | Validar contra Workforce Planning (HiBob) | Confirmacion de headcount |
| 1.3 | Direccion | Aprobar presupuesto y posicion | Aprobacion formal |
| 1.4 | RRHH | Crear posicion en Odoo (hr.job) | Job ID asignado |
| 1.5 | RRHH + Hiring Manager | Definir perfil del puesto (Job Description) | JD aprobada |

**Criterios de aprobacion**:
- Posicion contemplada en el Workforce Plan: aprobacion directa de RRHH
- Posicion no contemplada: requiere aprobacion de Direccion + justificacion de negocio

### Fase 2: Publicacion y Captacion (Dias 5-15)

| Paso | Responsable | Accion | Entregable |
|------|-------------|--------|------------|
| 2.1 | RRHH | Publicar en canales de captacion | Anuncio publicado |
| 2.2 | RRHH | Activar busqueda activa (LinkedIn, referrals) | Candidatos identificados |
| 2.3 | Equipo | Compartir en redes personales (referral program) | Referidos recibidos |

**Canales de captacion**:
- Portal de empleo propio (web Indaws)
- LinkedIn (publicacion + busqueda activa)
- InfoJobs / Indeed (segun perfil)
- Programa de referidos internos (bonus si contratado)
- Universidades / bootcamps (perfiles junior)
- Agencias de seleccion (solo para perfiles criticos, con aprobacion)

### Fase 3: Screening (Dias 10-20)

| Paso | Responsable | Accion | Entregable |
|------|-------------|--------|------------|
| 3.1 | RRHH | Revision de CVs contra requisitos minimos | Candidatos filtrados |
| 3.2 | RRHH | Screening telefonico (15-20 min) | Notas de screening |
| 3.3 | RRHH | Scoring inicial del candidato (candidate_scorer.py) | Puntuacion 0-100 |
| 3.4 | RRHH | Shortlist para entrevistas | Lista de candidatos preseleccionados |

**Criterios de screening**:
- Requisitos minimos del puesto (eliminatorios)
- Experiencia relevante (anos y sector)
- Competencias tecnicas basicas
- Disponibilidad y expectativas salariales (rango)
- Derecho a trabajar en Espana

### Fase 4: Entrevistas (Dias 15-30)

| Paso | Responsable | Accion | Entregable |
|------|-------------|--------|------------|
| 4.1 | RRHH | Entrevista de competencias (45-60 min) | Scorecard completado |
| 4.2 | Hiring Manager | Entrevista tecnica / funcional (60 min) | Scorecard completado |
| 4.3 | Equipo | Prueba tecnica o caso practico (si aplica) | Evaluacion de prueba |
| 4.4 | Direccion | Entrevista final (30 min, perfiles senior) | Validacion |
| 4.5 | RRHH | Consolidar scorecards y generar candidate brief | Brief para decision |

**Estructura de entrevistas**:
- Usar siempre la rubrica de `references/interview_scorecard.md`
- Minimo 2 entrevistadores distintos por candidato
- Feedback escrito dentro de las 24h siguientes a la entrevista
- Perfiles senior (IC4+, M2+): incluir entrevista con Direccion

### Fase 5: Decision y Oferta (Dias 25-35)

| Paso | Responsable | Accion | Entregable |
|------|-------------|--------|------------|
| 5.1 | RRHH + Hiring Manager | Reunion de decision (debrief) | Candidato seleccionado |
| 5.2 | RRHH | Verificacion de referencias (2 minimo) | Informe de referencias |
| 5.3 | RRHH + Direccion | Definir paquete de compensacion | Oferta aprobada |
| 5.4 | RRHH | Comunicar oferta al candidato | Oferta verbal + escrita |
| 5.5 | RRHH | Gestionar negociacion (si aplica) | Oferta final |
| 5.6 | RRHH | Comunicar rechazo a otros candidatos | Emails de rechazo |

### Fase 6: Pre-Onboarding (Dias 30-40)

| Paso | Responsable | Accion | Entregable |
|------|-------------|--------|------------|
| 6.1 | RRHH | Recopilar documentacion del nuevo empleado | Documentacion completa |
| 6.2 | RRHH | Alta en sistemas (HiBob, Odoo) | Perfil creado |
| 6.3 | IT | Preparar equipo y accesos | Setup completado |
| 6.4 | RRHH | Enviar email de bienvenida | Email enviado (D-3) |
| 6.5 | Manager | Preparar plan de onboarding | Plan Day-1/Week-1/Month-1 |

---

## SLAs del Proceso

| Metrica | Objetivo | Warning | Critico |
|---------|----------|---------|---------|
| Time-to-hire total | 30 dias | > 30 dias | > 60 dias |
| Tiempo en screening | 5 dias | > 7 dias | > 14 dias |
| Tiempo entre entrevistas | 3 dias | > 5 dias | > 10 dias |
| Feedback post-entrevista | 24 horas | > 48 horas | > 72 horas |
| Oferta tras decision | 48 horas | > 72 horas | > 1 semana |
| Comunicacion de rechazo | 1 semana | > 2 semanas | > 1 mes |

---

## Metricas de Seguimiento

| Metrica | Formula | Objetivo |
|---------|---------|----------|
| Time-to-hire | Fecha contratacion - Fecha apertura | < 30 dias |
| Conversion total | Contratados / Total candidatos | > 5% |
| Ofertas aceptadas | Ofertas aceptadas / Ofertas realizadas | > 80% |
| Coste por contratacion | Coste total / Contrataciones | Minimizar |
| Calidad de contratacion | % que supera periodo de prueba | > 90% |
| Diversidad de fuentes | Candidatos de fuente no principal | > 30% |
