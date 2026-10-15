---
name: indaws-demo-proyecto-analisis
description: >
  Radiografía de un proyecto concreto de Odoo (project.project) cruzando
  presupuesto de horas, tareas y partes reales (account.analytic.line).
  Devuelve un Health Score (0-100) con 4 métricas: consumo de horas vs plan,
  ritmo (horas/calendario), tareas al día y balance de equipo. Incluye
  desglose por tarea (dónde se está yendo), tabla de consultores más cargados,
  diagnóstico narrativo y plan de acción generados por IA. Output: dashboard
  HTML visual. Usar cuando el usuario pida: analizar un proyecto, salud de un
  proyecto, health check, dónde se está yendo, desviación de horas, radiografía
  de proyecto.
version: 1.0.3
model: sonnet
effort: medium
---

# Demo · Proyecto análisis

Health check de un proyecto individual de Odoo con diagnóstico cualitativo por IA.

## Prerequisitos

- **odoo-pilot** (common): lectura de `project.project`, `project.task`, `account.analytic.line`, `hr.employee`, `res.users`
- **Credenciales Odoo** en env: `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY`
- **Python 3.8+** (solo stdlib)
- `config/config.json` con pesos del Health Score y umbrales

## Workflow

| Paso | Accion | Output |
|------|--------|--------|
| 1 | Autenticar (`auth.sh`). Leer `config/config.json`. | -- |
| 2 | Ejecutar `python scripts/main.py --project-id <ID>` (o `--project-name <nombre>` si el usuario lo dice por texto). El script lee el proyecto + tareas + timesheets + empleados, calcula 4 métricas (hours/pace/tasks/team), genera el HTML con placeholders `{{LLM_DIAGNOSIS}}` y `{{LLM_ACTIONS}}` y guarda `output/<project>_analysis.json`. | HTML + analysis.json |
| 3 | Leer `output/<project>_analysis.json`. Redactar cuatro bloques narrativos: (a) **diagnóstico** (3-4 frases que interpreten los números), (b) **3 acciones correctivas** (con nombres de tareas/consultores), (c) **próximas acciones** (lista de 4-6 hitos proactivos del plan de trabajo con ventana temporal), (d) **seguimiento al cliente** (texto de 3-4 párrafos listo para enviar como email externo). Escribir en `output/<project>_narrative.json` con este formato: `{"diagnosis": "...", "actions": [...], "next_steps": [{"when":"...", "text":"..."}, ...], "client_update": "..."}`. | narrative.json |
| 4 | Ejecutar `python scripts/main.py --project-id <ID> --finalize`. Aplica el contenido LLM al HTML. Ruta final en `output/<project>_report.html`. | HTML final |
| 5 | Presentar al usuario: ruta del HTML, Health Score y top 3 hallazgos. | -- |

## Métricas (Health Score 0-100)

Media ponderada de:

| Componente | Peso | Cálculo |
|---|---|---|
| **Hours** | 30% | Consumo horas vs plan. `100` si consumo ≤ 100%. Penaliza cuanto más se desvía: 112% → 64, 130% → 10 |
| **Pace** | 25% | Relación horas consumidas / tiempo transcurrido. Si ratio ≤ 1 → 100. Si las horas van más rápido que el calendario, penaliza |
| **Tasks** | 25% | % de tareas sin retraso (deadline no vencido o estado "done") |
| **Team** | 20% | Balance del equipo. Penaliza si un consultor acapara >40% de las horas del proyecto |

Semáforo: 🟢 ≥80 · 🟡 60-79 · 🔴 <60

## Diagnóstico cualitativo (paso 3)

El LLM redacta 2 bloques con los datos del `analysis.json`:

### `diagnosis` (3-4 frases)
NO repite los números. Interpreta:
- Dónde se está yendo el tiempo/dinero (qué tareas, qué consultor)
- Qué pauta temporal hay (consumido demasiado rápido, hitos atrasados, cuellos de botella)
- Riesgo para el cierre del proyecto

Ejemplos ilustrativos (varía estructura y énfasis según los datos):
> *"El proyecto consume horas al mismo ritmo que avanza el calendario en las fases centrales, pero dos tareas de instalación concentran el 55% del desborde: 'Integración con SAP' (+38% horas) y 'Pruebas SAT' (+42%). Rubén Ferrer ha absorbido el 48% del total de horas del equipo — perfil claro de dependencia en un solo consultor. Riesgo medio-alto de no cerrar en plazo si el sprint actual no se reequilibra."*
> *"Proyecto por delante del calendario, con el equipo repartido sin dependencias. El riesgo real está fuera: el cliente aún no ha validado los datos maestros y eso bloquea la carga final."*

### `actions` (3 items, correctivas)
Ejecutables esta semana, con nombres reales de tareas/consultores. Ejemplo:
- "Reasignar parte de 'Integración con SAP' de Rubén Ferrer a Nacho Tello (tiene holgura)"
- "Revisar estimación de 'Pruebas SAT' con el cliente — aceptar ampliación de alcance o renegociar"
- "Bloquear 2 jornadas de Rubén esta semana para cerrar 'Cierre documentación' (task en rojo desde hace 8 días)"

### `next_steps` (4-6 items, roadmap proactivo)
Hitos naturales del plan, no correctivos. Cada item es un dict `{"when": "Esta semana" | "Próxima semana" | "28 abr" | ...,  "text": "..."}`. Ejemplo:
- `{"when":"Esta semana", "text":"Reunión con el cliente para cerrar criterios de aceptación..."}`
- `{"when":"28 abr · 2 may", "text":"Sesiones de formación al personal (2 sesiones de 4h)"}`
- `{"when":"4 may", "text":"Entrega final, firma del acta de recepción y cierre administrativo"}`

### `client_update` (texto para email al cliente)
3-4 párrafos en tono profesional, apto para envío externo. Debe cubrir:
1. Apertura corta + **avances de la semana** (qué se ha completado, sin tecnicismos internos)
2. **Próximos pasos** relevantes para el cliente (lo que necesitará de su parte, fechas, formación)
3. Estado del calendario / fecha de entrega (reafirmar compromiso)
4. Cierre con disponibilidad y firma del PM

Tono: cercano pero profesional, sin mencionar problemas internos (no decir "subestimamos horas"), sin mostrar desviaciones de presupuesto. Foco en lo que el cliente necesita saber para seguir el proyecto.

## RULES

1. **SIEMPRE** usa la suma de `account.analytic.line.unit_amount` agrupada por `project_id` como "horas reales" (más fiable que `project.task.effective_hours`)
2. **NUNCA** repitas los números en el diagnóstico — interpreta
3. **SIEMPRE** las 3 acciones deben nombrar tareas/consultores concretos (usar `top_tasks_by_deviation` y `team_load` del JSON)
4. Si el proyecto no tiene `date_start` o `date`, el componente "pace" se omite y se redistribuye su peso (25%) entre los otros tres
5. Si el proyecto tiene 0 tareas o 0 timesheets, abortar con mensaje claro — no tiene sentido analizar

## GOTCHAS

1. `project.task.effective_hours` solo cuenta horas **con task asignada**. Si hay timesheets sin task, la cifra por task es menor que la real del proyecto — no lo uses como total
2. Odoo 19 JSON2: el parámetro en `write` se llama `vals` (no `values`)
3. El `read_group` exige kwargs (`domain`, `fields`, `groupby`)
4. Algunos proyectos no tienen `user_id` (PM) — mostrar "Sin asignar" en el informe
5. La relación `project.project` ↔ `sale.order` en esta demo es por convención de nombre (`S00XXX - Cliente`). Extraer el código del inicio del nombre
6. Empleados con `unit_amount: 0` (horas neutralizadas) se ignoran en el balance

## Delegates to

| Skill | When |
|-------|------|
| **common:odoo-pilot** | lectura de project/tasks/analytic.line/hr.employee |

## References

| File | Content |
|------|---------|
| `config/config.json` | Pesos del Health Score + umbrales |
