# Catalogo de Roles y Niveles

**CONFIDENTIAL - HR TEAM ONLY**

Definicion de niveles profesionales en Indaws. Marco de referencia para contratacion,
evaluacion de desempeno y planificacion de carrera.

---

## Niveles de Contribuidor Individual (IC)

### IC1 - Junior

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | 0-2 anos de experiencia relevante |
| **Autonomia** | Trabaja con supervision directa. Tareas definidas y acotadas |
| **Impacto** | Contribuye a tareas del equipo. Impacto individual |
| **Complejidad** | Problemas bien definidos con soluciones conocidas |
| **Comunicacion** | Reporta progreso al manager. Pide ayuda cuando la necesita |
| **Desarrollo** | Aprendizaje activo de herramientas, procesos y dominio |
| **Ejemplo Desarrollo** | Desarrollador junior Odoo, implementacion de modulos basicos |
| **Ejemplo Comercial** | SDR, apoyo comercial |
| **Ejemplo Consultoria** | Consultor junior, soporte en implementaciones |

### IC2 - Mid-Level

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | 2-4 anos de experiencia relevante |
| **Autonomia** | Trabaja con supervision limitada. Gestiona su carga de trabajo |
| **Impacto** | Contribuye de forma consistente. Colabora efectivamente |
| **Complejidad** | Problemas con multiples soluciones. Elige la adecuada |
| **Comunicacion** | Comunica estado y bloqueos proactivamente |
| **Desarrollo** | Profundiza expertise. Empieza a mentorizar IC1 |
| **Ejemplo Desarrollo** | Desarrollador Odoo, modulos de complejidad media |
| **Ejemplo Comercial** | Account Executive, gestion de pipeline propio |
| **Ejemplo Consultoria** | Consultor funcional, liderazgo de workstreams |

### IC3 - Senior

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | 4-7 anos de experiencia relevante |
| **Autonomia** | Autonomo en la mayoria de situaciones. Toma decisiones tecnicas |
| **Impacto** | Impacto a nivel de equipo. Mejora procesos y estandares |
| **Complejidad** | Problemas ambiguos. Descompone en partes manejables |
| **Comunicacion** | Comunica efectivamente cross-equipo. Presenta a stakeholders |
| **Desarrollo** | Referente tecnico. Mentoriza IC1-IC2 activamente |
| **Ejemplo Desarrollo** | Senior Developer, arquitectura de modulos, code review |
| **Ejemplo Comercial** | Senior AE, cuentas estrategicas, deals complejos |
| **Ejemplo Consultoria** | Senior Consultant, lider de proyecto, cliente autonomo |

### IC4 - Staff / Especialista

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | 7-10 anos de experiencia relevante |
| **Autonomia** | Completamente autonomo. Define enfoque tecnico |
| **Impacto** | Impacto a nivel de multiples equipos o area de negocio |
| **Complejidad** | Problemas con alcance organizacional. Multiples sistemas |
| **Comunicacion** | Influye en decisiones tecnicas a nivel de empresa |
| **Desarrollo** | Define estandares. Lidera iniciativas tecnicas transversales |
| **Ejemplo Desarrollo** | Staff Engineer, arquitectura global, decisiones tecnicas clave |
| **Ejemplo Comercial** | Key Account Manager, relaciones estrategicas |
| **Ejemplo Consultoria** | Practice Lead, metodologia y estandares de delivery |

### IC5 - Principal

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | 10-15 anos de experiencia relevante |
| **Autonomia** | Define la direccion tecnica del area |
| **Impacto** | Impacto a nivel de empresa. Decisiones estrategicas |
| **Complejidad** | Problemas a nivel de industria. Innovacion |
| **Comunicacion** | Representa la empresa externamente. Thought leadership |
| **Desarrollo** | Mentoriza IC4. Define roadmap tecnico |

### IC6 - Distinguished

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | 15+ anos, reconocimiento externo |
| **Autonomia** | Define vision a largo plazo |
| **Impacto** | Impacto en la industria. Referente externo |
| **Complejidad** | Problemas sin precedentes. Crea nuevos paradigmas |
| **Comunicacion** | Portavoz tecnico de la empresa |
| **Desarrollo** | Define cultura tecnica de la organizacion |

---

## Niveles de Management (M)

### M1 - Team Lead / Manager

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | Minimo IC3 + potencial de liderazgo |
| **Personas a cargo** | 3-7 reportes directos |
| **Foco** | Ejecucion del equipo. Desarrollo de personas |
| **Responsabilidades** | 1:1s semanales, feedback, planning, hiring, performance reviews |
| **Indicadores** | Productividad del equipo, retencion, satisfaccion |
| **Ejemplo** | Tech Lead, Sales Manager, Delivery Manager |

### M2 - Senior Manager / Head

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | Minimo M1 con track record de resultados |
| **Personas a cargo** | 2-4 managers o equipos (8-20 personas) |
| **Foco** | Estrategia del area. Alineacion cross-funcional |
| **Responsabilidades** | Objetivos del area, presupuesto, hiring strategy, talent development |
| **Indicadores** | Resultados del area, crecimiento, eficiencia |
| **Ejemplo** | Head of Development, Head of Sales, Head of Consulting |

### M3 - Director / VP

| Dimension | Descripcion |
|-----------|-------------|
| **Experiencia tipica** | Minimo M2 con vision estrategica demostrada |
| **Personas a cargo** | Multiples areas (20+ personas) |
| **Foco** | Estrategia de negocio. Direccion de la empresa |
| **Responsabilidades** | P&L, estrategia, cultura, representacion externa |
| **Indicadores** | Resultados de negocio, crecimiento, posicionamiento |
| **Ejemplo** | Director de Operaciones, VP de Producto |

---

## Uso en Recruitment

- Al abrir una posicion, definir el **nivel objetivo** (IC1-IC6, M1-M3)
- El scoring de candidatos (`candidate_scorer.py`) compara contra el perfil del nivel
- Las expectativas salariales se basan en bandas por nivel (gestionado por RRHH)
- La entrevista con Direccion es obligatoria para perfiles IC4+ y M2+
