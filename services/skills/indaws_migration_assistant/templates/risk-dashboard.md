# Template: Dashboard de Salud del Proyecto

> Usar cuando PM, COO o TL pregunta: "¿Cómo va el proyecto?", "Dashboard", "Estado del proyecto",
> "Riesgos del proyecto", "¿Cuántas horas quedan?"
> Funcionalidad F12: Ejecutar `scripts/risk_dashboard.py` o generar resumen narrativo.

---

## Dashboard — {nombre_proyecto}

**Fecha:** {fecha} | **Go-Live:** {go_live_date} ({dias_restantes} días)

### KPIs principales

| Indicador | Valor |
|-----------|-------|
| Tareas completadas | {n}/{total} ({pct}%) |
| Horas consumidas | {hh:mm} / {budget_hh:mm} ({pct}%) |
| Horas disponibles | {available_hh:mm} |
| Gates aprobados | {n}/{total} |
| Días hasta Go-Live | {n} |

### Focos de atención

> Alertas y warnings generados automáticamente por el script.
> Tipos: `critical` (rojo), `warning` (amarillo), `info` (verde).

| Tipo | Área | Mensaje |
|------|------|---------|
| {tipo} | {area} | {mensaje} |

### Horas por empleado

| Empleado | Facturable | No facturable | Total |
|----------|-----------|---------------|-------|
| {nombre} | {hh:mm} | {hh:mm} | {hh:mm} |

### Gates

| Gate | Estado | Comentario |
|------|--------|-----------|
| {nombre_gate} | {estado} | {nota} |

### Tareas activas

| Tarea | Estado | Etapa | Planificadas | Consumidas | Asignada |
|-------|--------|-------|-------------|-----------|----------|
| {nombre} | {estado} | {etapa} | {hh:mm} | {hh:mm} | {si/no} |

### Forecast

| Métrica | Valor |
|---------|-------|
| Velocidad media | {h} h/semana |
| Velocidad reciente | {h} h/semana |
| Fecha estimada fin tareas | {fecha} |
| Fecha estimada agotamiento horas | {fecha} |

---

## Reglas de formato

- Si se dispone de API: ejecutar `scripts/risk_dashboard.py` y usar los datos JSON resultantes
- Si NO se dispone de API: generar resumen narrativo basándose en datos conocidos del contexto
- Focos de atención: solo incluir los generados por el script, no inventar
- Si el usuario pide `--lang en`, las etiquetas y fechas se muestran en inglés
- Si el usuario es PM/COO: énfasis en horas y forecast
- Si el usuario es TL: énfasis en tareas y gates
- Acciones recomendadas: derivar de los focos de atención, máximo 3, accionables
