# Metodología de Trabajo Indaws

> Fuente: Knowledge #1664

## Matriz RACI — Equipo de Programación

R=Responsable, A=Aprobador, C=Consultado, I=Informado

### I. Ejecución Diaria y Riesgos

| Función | TL | PM | CTO Ops (Jesús) | CTO Innov (David) |
|---------|----|----|-----------------|-------------------|
| Estimación/Planificación semanal | **R** | **R/A** | C (si escala) | I |
| Planificación mensual | **R** | **R** | C (si requiere) | I |
| Incidencias día a día | **R** (soluciona) | **R** (comunica cliente) | C (escalada) | I (si impacto mayor) |
| Cambio alcance/nuevas valoraciones | C (re-estima) | **R** (lidera con cliente) | **A** (si compleja) | I |
| Code review y estándares | **R** (ejecuta) | N/A | **C/A** (audita) | **C/A** (audita) |
| Conversaciones técnicas con cliente | **R** (criterio técnico) | I | **I/A** (políticas) | I (si I+D) |

### II. Estrategia, Arquitectura y Procesos

| Función | TL | PM | CTO Ops | CTO Innov |
|---------|----|----|---------|-----------|
| Metodología Indaws Core | **R** (cumplimiento) | I | **A** | C |
| Personalizaciones/deuda técnica | **R** (propone) | I | **A** | C (si I+D) |
| Documentar procesos técnicos | **R** (aporta) | N/A | **R/C** (lidera) | **R/C** (lidera) |
| Nueva herramienta/tecnología | **R** (propone) | I | **A** (operativa) | **A** (estratégica) |
| Análisis técnico leads (preventas) | **R/C** | **R** (lidera) | **R/C** (viabilidad) | **R/C** (si I+D) |

### III. Gestión de Equipo

| Función | TL | PM | CTO Ops | CTO Innov |
|---------|----|----|---------|-----------|
| Seguimiento TLs (bloqueos) | **R** (reporta) | I | **R** (lidera) | **R/I** |
| Horas facturables/productivas | **R** (valida diario) | **R** (time-tracking) | **A** | I |
| Problema desempeño dev | **R** (identifica) | I | **R/A** (plan mejora) | **R/A** |
| Mentoría a TLs | C (recibe) | I | **R** (capacita) | **R/C** |
| Evaluaciones 1&1 devs | **R/C** (su equipo) | I | **R/A** (lidera) | **R/A** |

### IV. I+D

| Función | TL | PM | CTO Ops | CTO Innov |
|---------|----|----|---------|-----------|
| Coordinación Indaws Core | I | I | **R** (mantiene) | **R** (lidera) |
| Backlog producto Core | C (necesidades) | **R/C** (prioriza) | **A/R** | **R** (lidera) |
| Indaws Cloud | I | I | **A/C** | **R** (lidera) |

## Roles Clave

- **TL**: Ejecución técnica diaria, planificación semanal, incidencias, code review, criterio técnico con clientes, validación horas equipo
- **PM**: Gestión proyecto, relación cliente, tiempos, facturación, cambios alcance
- **CTO Ops (Jesús)**: Viabilidad y calidad operativa, mentoría TLs, aprueba soluciones complejas, vela por Core
- **CTO Innov (David)**: Estrategia tecnológica, Indaws Core I+D, innovación, alineación estratégica

## Niveles de Valoración de Esfuerzo (Oficiales)

| Nivel | Descripción | Quién valora |
|-------|-------------|-------------|
| **Baja** | 1 cambio aislado: campo informativo, computed sencillo, campo en vista, dato en PDF | PM directamente |
| **Media** | ≥4 cambios pequeños agrupados (individualmente serían Baja) | PM directamente |
| **Alta** | Integraciones conocidas (ya hechas antes), cambios moderados flujo estándar | PM + TL |
| **Muy Alta** | Integraciones complejas, cambios drásticos flujo estándar | **SIEMPRE Co-CTOs** |

Valoraciones complejas (migraciones, integraciones nuevas): **SIEMPRE Co-CTOs**.

## Gestión de Consultoría

- **Incidencias**: equipo resuelve internamente → si no puede, TL escala a Co-CTOs por chat equipo
- **Sin tareas**: dev comunica → consultoría asigna → si no hay, escala Co-CTOs
- **Planificación**: quincenal. Objetivo futuro: equipos autosuficientes
- **Reuniones técnicas con cliente**: dev gestiona si se ve capacitado, pide apoyo a Co-CTO si necesita
- **Reuniones internas**: sin pasar por Co-CTOs
- **Análisis proyectos**: con Jesús V., él decide si involucrar a David

## Gestión Servicios (Soporte)

- Dev junior dedicado a servicios (ve muchos casos, conoce proyectos)
- Ticket en "En desarrollo" → **obligatorio** tener desarrollador asignado
- Urgencias: pedir apoyo por chat equipo → si no resuelve → escala Co-CTOs **en minutos**
- Ángel gestiona prioridades urgentes

## Filosofía de Desarrollo

### Dependencia Cero
No OCA. No dependencias externas (Python, JS, herramientas) sin evaluar riesgos. Objetivo: depender solo de Indaws.

### Cero Desarrollos Específicos
"Indaws no desarrolla, aporta valor." Mentalidad: resolver **casos de negocio** (horizontal), no problemas puntuales. Product Owners por área. Cotejar con casos ya resueltos. Homogeneizar proyectos.

### Usar 100% de Odoo
Acciones automáticas/planificadas, Chatter, Actividades, Studio. Reuniones consultoría+desarrollo para casos complejos.
