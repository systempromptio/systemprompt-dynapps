# Template: Plan de Trabajo para Dev

> Usar cuando un Dev pregunta: "¿Por dónde empiezo?", "Plan de trabajo"
> Funcionalidad F2: Analizar tareas del proyecto y ordenar por dependencias.

---

## Plan de Trabajo — Migración {nombre_proyecto}

**Versión:** v{version_origen} → v{version_destino}
**Total tareas:** {num_tareas} | **Total horas:** {total_horas}h
**Fecha generación:** {fecha}

### Orden de ejecución recomendado

| Orden | ID | Tarea | Tags | Dependencias | Estimación | Riesgo |
|---|---|---|---|---|---|---|
| 1 | #{task_id} | [MIG] {modulo} | Migration, Standard | Ninguna | {horas}h | 🟢 Bajo |
| 2 | #{task_id} | [MIG] {modulo} | Migration, Studio | → {modulo_dep} | {horas}h | 🟡 Medio |
| 3 | #{task_id} | [MIG] {modulo} | Migration, Custom | → {modulo_dep1}, {modulo_dep2} | {horas}h | 🔴 Alto |
| ... | | | | | | |

### Criterios de ordenación
1. **Primero:** Módulos sin dependencias custom (se pueden migrar de forma independiente)
2. **Después:** Módulos que dependen de los anteriores (en orden topológico)
3. **Al final:** Módulos con más dependencias / mayor riesgo (JS/OWL, integraciones)

### Recomendaciones
- Empezar por los módulos marcados **"Standard"** (eliminar y usar estándar) — son los más rápidos
- Módulos **JS/OWL** requieren atención especial (v16+) — consultar `odoo-module-upgrader`
- Módulos con **integraciones externas** necesitan coordinación con BC para credenciales de test
- **OBLIGATORIO:** Informar a BC 2×/semana (ver Protocolo de Comunicación)

### Tu checklist diario (Dev)
- [ ] Actualizar log de avance en cada tarea tras cada sesión
- [ ] Escalar decisiones funcionales inmediatamente — NO asumir
- [ ] Mantener pipeline CI/CD verde
- [ ] Si un módulo tarda más de lo estimado (+20%), informar a TL

### Protocolo de comunicación
| Hacia | Frecuencia | Formato |
|---|---|---|
| BC | 2x/semana | Informe estructurado en tarea Odoo (usar plantilla Dev→BC) |
| TL | Diario | Actualización breve en tarea o chat |

---

## Reglas de formato

- Ordenación SIEMPRE calculada por analyze_task_dependencies.py — NO ordenar manualmente
- Mostrar ID de tarea Odoo para que el Dev pueda navegar directamente
- Dependencias: lista de módulos de los que depende (extraídos del bloque CONTEXTO de la tarea)
- Riesgo: 🟢 Bajo / 🟡 Medio / 🔴 Alto
- Si no hay tareas en el proyecto, indicarlo y sugerir que el PM ejecute "Crea las tareas de migración"
