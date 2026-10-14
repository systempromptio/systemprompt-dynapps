---
name: indaws-standup-facilitator
description: >
  Facilita reuniones de standup diarias y semanales del equipo de desarrollo.
  Genera agendas automaticas desde planning slots, tickets, bloqueos y ausencias.
  Triggers: "standup", "daily standup", "agenda standup", "preparar standup",
  "reunion diaria", "sprint review", "que esta haciendo el equipo".
version: 1.1.1
effort: low
model: sonnet
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion]
---

# Standup Facilitator v1.0

Genera agendas de standup **automaticas** cruzando datos de Odoo.

> **No duplicar**: `/team-task-assigner` planifica, `/dev-assistant` gestiona horas,
> `/ticket-report` analiza tickets. Este skill **facilita la reunion** con datos agregados.

## Scripts

```
scripts/
├── odoo_api.py              # Modulo base API
└── standup_snapshot.py       # Generador de agenda
```

| Script | Entrada | Salida |
|--------|---------|--------|
| `standup_snapshot.py` | team_id [--mode daily\|weekly] [--date] | Agenda estructurada |

### Variables de entorno

```bash
ODOO_URL   # default: https://portal.dynapps.es
ODOO_DB    # default: master
ODOO_KEY   # OBLIGATORIO
ODOO_USER  # OBLIGATORIO
```

## Uso

### Daily Standup (default)

```bash
# desde el directorio de esta skill
python3 scripts/standup_snapshot.py <team_id>
```

Ejemplo: `python3 scripts/standup_snapshot.py 28`

### Weekly Review

```bash
python3 scripts/standup_snapshot.py 28 --mode weekly
```

### Fecha especifica

```bash
python3 scripts/standup_snapshot.py 28 --date 2026-03-23
```

## Que muestra

La agenda incluye **por desarrollador**:

1. **Tareas en curso** — planning slots activos (proyecto, tarea, horas)
2. **Tickets urgentes** — tickets con prioridad alta/urgente
3. **Bloqueos** — tareas en desarrollo sin actividad >3 dias
4. **Ausencias** — quien no esta hoy/esta semana

Y un resumen con:
- Temas a tratar (bloqueos, urgentes, devs sin planificar)
- Recordatorios contextuales (viernes = marcar slots hechos)

## Workflow del TL

### WF1: Standup Diario (5-10 min)

1. Ejecutar `standup_snapshot.py <team_id>` antes de la reunion
2. Presentar la agenda al equipo (el script ya la formatea)
3. Recorrer cada dev **presente**:
   - "Que estas haciendo?" → Ya aparece en slots
   - "Algun bloqueo?" → Los stale tasks ya lo anticipan
   - "Necesitas algo?" → Tickets urgentes como contexto
4. Apuntar action items que surjan
5. Terminar en <10 min

### WF2: Weekly Review (15-20 min)

1. Ejecutar con `--mode weekly`
2. Vision panoramica de la semana
3. Combinar con `/team-task-assigner` snapshot para ver capacidad
4. Decidir: replanificar? redistribuir? escalar?

### WF3: Sprint Retrospective

Para retro, complementar con:
- `/timesheet-report` — horas reales vs planificadas
- `/ticket-report` — tickets cerrados y tiempos de resolucion
- `estimation_history.py` de tl-assistant — calibracion estimaciones

El facilitator aporta los **datos de la semana en curso**,
los otros skills aportan **metricas historicas**.

## Reglas

1. **SIEMPRE usar el script** — no construir curl/API inline
2. **No sustituir la conversacion** — la agenda es punto de partida, no acta
3. **<10 min standup diario** — si se extiende, agendar reunion aparte
4. **Bloqueos primero** — son lo mas actionable
5. **No juzgar** — datos factuales, no evaluacion de rendimiento

## Delegates to

> Routing: skill ligera — corre entera como **Agent(model: sonnet)**. La conexión Odoo del script sigue el contrato compartido `references/shared/odoo-connection.md`.

- **`/team-task-assigner`** → Planificar la semana (despues del standup)
- **`/dev-assistant`** → Ver planificacion individual de un dev
- **`/ticket-report`** → Analisis profundo de tickets
- **`/timesheet-report`** → Horas facturables/no facturables
- **`/tl-assistant`** → Decisiones tecnicas que surjan en el standup

## GOTCHAS

1. **`team_id` es el `crm.team`, no el `hr.department`** — el script filtra por equipo comercial/de desarrollo; pasar el id equivocado devuelve una agenda vacía.
2. **Bloqueo = tarea en desarrollo sin actividad >3 días** — es heurístico; una tarea legítimamente larga puede aparecer como falso bloqueo. Contrastar en la reunión, no dar por hecho.
3. **La agenda NO es acta** — es punto de partida. No sustituye la conversación ni sirve como evaluación de rendimiento (regla 5).
4. **Viernes**: el snapshot recuerda marcar los slots como hechos; si no se hace, el forecast de `team-task-assigner` de la semana siguiente arrastra pendientes falsos.

## Consumed by

Ninguna skill publicada la consume — es **standalone**, invocada directamente por el TL para preparar la daily/weekly.
