# Template: Resource Advisor

> Usar cuando TL o PM pregunta: "¿Cómo arreglo esto?", "¿Qué recursos necesito?",
> "¿Cómo volvemos a estar en tiempo?", "El proyecto va mal, ¿qué hago?",
> "Resource advisor", "Necesito ayuda con el proyecto"
> Funcionalidad F15: Ejecutar `scripts/resource_advisor.py`.

---

## Resource Advisor — {nombre_proyecto}

**Fecha:** {fecha} | **Salud:** {health_score} ({health_level})

### Diagnóstico

| Indicador | Valor |
|-----------|-------|
| Velocidad actual | {h} h/semana |
| Horas disponibles | {hh:mm} de {budget_hh:mm} |
| Tareas pendientes | {n} de {total} |
| Forecast actual | {fecha} |
| Go-Live previsto | {fecha} ({dias} días) |
| Desviación | {+/- semanas} |

### Desarrolladores con disponibilidad

| Desarrollador | Equipo | Disponible (h) | h/semana | Proyectos actuales |
|---------------|--------|----------------|----------|-------------------|
| {nombre} | {equipo} | {h} | {h/sem} | {proyectos} |

### Escenario 1: Añadir recurso

| Desarrollador | Disponible | Nuevo forecast | Mejora |
|---------------|-----------|---------------|--------|
| {nombre} | {h/sem} h/sem | {fecha} | -{dias} días |

### Escenario 2: Reducir alcance

| Opción | Tareas a posponer | Horas ahorradas | Nuevo forecast | Mejora |
|--------|-------------------|-----------------|---------------|--------|
| {n} | {lista_tareas} | {h}h | {fecha} | -{dias} días |

### Escenario 3: Mover Go-Live

| Concepto | Fecha |
|----------|-------|
| Go-Live actual | {fecha} |
| Go-Live necesario (al ritmo actual) | {fecha} |
| Go-Live recomendado (con buffer) | {fecha} (+{n} semanas) |

---

## Reglas de formato

- SIEMPRE ejecutar `scripts/resource_advisor.py` — NO calcular manualmente
- Si el proyecto está en buen estado (salud verde, forecast a tiempo), informar que no se necesitan ajustes y mostrar solo la disponibilidad de desarrolladores
- Si no hay devs con bandwidth, indicar claramente y centrar los escenarios en scope y go-live
- Si el usuario es TL: mostrar todos los escenarios con detalle técnico, incluir nombres de tareas a recortar
- Si el usuario es PM: énfasis en impacto en fechas, resumen ejecutivo, sin nombres técnicos de tareas
- Los escenarios son SUGERENCIAS — el skill NO toma decisiones, solo presenta opciones
- El horizonte de disponibilidad por defecto es 8 semanas, ajustable con --weeks
