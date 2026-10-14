# Reports - Definicion y Ejecucion de Informes

## Tabla de Informes

| Tipo de Informe | Template OBLIGATORIO | Triggers |
|-----------------|---------------------|----------|
| **Informe semanal completo** | `templates/open-tickets.md` + `templates/closed-tickets.md` + `templates/resolution-stats.md` | "informe semanal", "informe completo", "dame mi informe", "resumen de tickets" |
| **Informe semanal de equipo** (TL) | `templates/open-tickets.md` + `templates/closed-tickets.md` + `templates/resolution-stats.md` (por dev) | "informe de mi equipo", "tickets de mi equipo", "como van mis desarrolladores" |
| **Informe semanal de equipo** (PM) | `templates/open-tickets.md` + `templates/closed-tickets.md` + `templates/resolution-stats.md` (por persona, solo `user_id`) | "informe de mi equipo", "tickets de mi equipo", "seguimiento de mi equipo" |
| Tickets abiertos prioritarios | `templates/open-tickets.md` | "mis tickets", "tickets abiertos", "tickets prioritarios", "que tickets tengo" |
| Tickets cerrados | `templates/closed-tickets.md` | "tickets cerrados", "que he cerrado esta semana", "tickets resueltos" |
| Estadisticas de resolucion | `templates/resolution-stats.md` | "media de resolucion", "tiempo medio", "estadisticas de tickets" |
| Analisis de carga | `templates/workload-analysis.md` | "carga de trabajo", "tengo mucha carga", "atascos", "bottlenecks" |
| **Informe mensual** | `templates/monthly-report.md` | "informe mensual", "informe de [mes]", "como fue [mes]", "resumen mensual", "estadisticas del mes", "mes pasado" |
| **Informe todos los tiempos** | `templates/alltime-report.md` | "todos los tiempos", "informe historico", "mi progreso", "evolucion de tickets", "estoy mejorando", "tendencia de resolucion" |

### Reglas OBLIGATORIAS

1. **ANTES** de generar cualquier informe, **LEER el template correspondiente** con la herramienta Read
2. Seguir **EXACTAMENTE** el formato del template (columnas, orden, indicadores)
3. **NO inventar** columnas, formatos, secciones ni estilos
4. **NO omitir** columnas definidas en el template
5. Si el usuario solicita un informe que NO esta en la tabla anterior: **PREGUNTAR** con AskUserQuestion antes de generar nada

---

## Informe Semanal Completo

Informe combinado que agrupa toda la informacion relevante de la semana en un solo bloque.

**Secciones del informe** (en orden):

1. **Tickets abiertos** -- usando get_user_tickets.sh (ambos campos para Desarrollador)
2. **Tickets con mas de 1 semana abiertos que requieren seguimiento** -- filtrar client-side los tickets abiertos con antiguedad > 7 dias, ordenados por antiguedad descendente, con columna "Accion recomendada":
   - "Espera cliente" > 7 dias: "Hacer ping al cliente"
   - "En desarrollo" > 14 dias: "Bloqueado? Contactar PM"
   - "En progreso" > 14 dias: "Actualizar estado o escalar"
   - "En progreso" > 30 dias: "Revisar con PM, posible atasco"
3. **Tickets cerrados esta semana** (lunes a hoy) -- usando get_closed_tickets.sh
4. **Estadisticas de resolucion del mes en curso** (1 del mes a hoy) -- usando get_closed_tickets.sh + calculate_ticket_stats.py resolution

**Ejecucion para Desarrolladores** (en paralelo):
```bash
# Bloque 1+2: Tickets abiertos
./scripts/get_user_tickets.sh <uid> open "" "" developer
./scripts/get_user_tickets.sh <uid> open "" "" assigned

# Bloque 3: Cerrados esta semana
./scripts/get_closed_tickets.sh <uid> <lunes> <hoy> developer
./scripts/get_closed_tickets.sh <uid> <lunes> <hoy> assigned

# Bloque 4: Estadisticas del mes
./scripts/get_closed_tickets.sh <uid> <primer_dia_mes> <hoy> developer | python3 scripts/calculate_ticket_stats.py resolution
./scripts/get_closed_tickets.sh <uid> <primer_dia_mes> <hoy> assigned | python3 scripts/calculate_ticket_stats.py resolution
```

**Este es el informe recomendado para uso semanal habitual.**

---

## Informe Mensual

Informe completo del mes: tickets cerrados, estadisticas de resolucion (media, mediana, distribucion, top 5) y tickets abiertos al cierre del periodo. Para Team Leader incluye tabla comparativa del equipo.

**Seleccion de periodo** (preguntar si no se especifica usando AskUserQuestion):

| Opcion | date_from | date_to |
|--------|-----------|---------|
| Mes actual | Primer dia del mes en curso | Hoy |
| Mes pasado | Primer dia del mes anterior | Ultimo dia del mes anterior |
| Mes concreto | Primer dia de ese mes | Ultimo dia de ese mes (o hoy si es el mes en curso) |

**Si estamos a dia 1-5 del mes y pide "mes actual"**: advertir que la muestra es muy pequena y sugerir usar el mes pasado.

**Ejecucion para Desarrolladores** (en paralelo):
```bash
# Tickets cerrados del mes
./scripts/get_closed_tickets.sh <uid> <date_from> <date_to> developer
./scripts/get_closed_tickets.sh <uid> <date_from> <date_to> assigned

# Estadisticas (con calculate_ticket_stats.py)
./scripts/get_closed_tickets.sh <uid> <date_from> <date_to> developer | python3 scripts/calculate_ticket_stats.py resolution
./scripts/get_closed_tickets.sh <uid> <date_from> <date_to> assigned | python3 scripts/calculate_ticket_stats.py resolution

# Tickets abiertos actualmente
./scripts/get_user_tickets.sh <uid> open "" "" developer
./scripts/get_user_tickets.sh <uid> open "" "" assigned
```

**Ejecucion adicional para Team Leader** (para cada desarrollador a cargo):
```bash
./scripts/get_closed_tickets.sh <uid_dev> <date_from> <date_to> developer | python3 scripts/calculate_ticket_stats.py resolution
./scripts/get_closed_tickets.sh <uid_dev> <date_from> <date_to> assigned | python3 scripts/calculate_ticket_stats.py resolution
```

Al final del informe del TL, generar la tabla comparativa del equipo (incluyendo al propio TL) con: tickets cerrados, media, mediana, mas rapido y mas lento.

---

## Informe de Todos los Tiempos (Progreso)

Estadisticas historicas completas + evolucion mensual para medir si el usuario resuelve tickets cada vez mas rapido o mas lento. Sin filtro de fecha: incluye todos los tickets cerrados desde el inicio.

**Ejecucion para Desarrolladores** (en paralelo):
```bash
./scripts/get_all_closed_tickets.sh <uid> developer | python3 scripts/calculate_ticket_trend.py
./scripts/get_all_closed_tickets.sh <uid> assigned  | python3 scripts/calculate_ticket_trend.py
```

**Ejecucion adicional para Team Leader** (para cada dev a cargo):
```bash
./scripts/get_all_closed_tickets.sh <uid_dev> developer | python3 scripts/calculate_ticket_trend.py
```

**Salida de calculate_ticket_trend.py**:
- total_tickets, global (media, mediana, distribucion, top 5)
- trend: "mejorando" | "empeorando" | "estable" | "sin_datos"
- trend_label, monthly_breakdown (count, media, mediana por mes)

**La tendencia** se calcula comparando la mediana de la primera mitad vs la segunda mitad de los ultimos 6 meses con datos. Una diferencia > 10% marca el cambio.

---

## Tickets Abiertos Prioritarios

Muestra los tickets abiertos del usuario ordenados por prioridad y antiguedad.

**Para Desarrolladores** (dos secciones):
```bash
./scripts/get_user_tickets.sh <uid> open "" "" developer
./scripts/get_user_tickets.sh <uid> open "" "" assigned
```

**Para Consultores** (una seccion):
```bash
./scripts/get_user_tickets.sh <uid> open "" "" assigned
```

**Filtros de etapa**: NO incluir Solved ni Cancelled. Solo: New (1), In Progress (2), En desarrollo (6), Testeo Desarrollo (15), Espera cliente (5), Espera subida produccion (14).

**Ordenamiento**: Por prioridad descendente, luego por create_date ascendente.

---

## Tickets Cerrados en Periodo

Muestra los tickets cerrados (etapa Solved) en un periodo especificado.

**Para Desarrolladores** (dos secciones):
```bash
./scripts/get_closed_tickets.sh <uid> <from> <to> developer
./scripts/get_closed_tickets.sh <uid> <from> <to> assigned
```

**Para Consultores** (una seccion):
```bash
./scripts/get_closed_tickets.sh <uid> <from> <to> assigned
```

**Periodo por defecto**: Si no se especifica, usar la semana actual (lunes a hoy).

---

## Estadisticas de Tiempo de Resolucion

Calcula la media, mediana y distribucion de tiempos de resolucion.

**Para Desarrolladores**: Calcular estadisticas separadas por campo y un resumen combinado:
```bash
./scripts/get_closed_tickets.sh <uid> <from> <to> developer | python3 scripts/calculate_ticket_stats.py resolution
./scripts/get_closed_tickets.sh <uid> <from> <to> assigned | python3 scripts/calculate_ticket_stats.py resolution
```

**Para Consultores**: Una sola estadistica:
```bash
./scripts/get_closed_tickets.sh <uid> <from> <to> assigned | python3 scripts/calculate_ticket_stats.py resolution
```

**Periodo por defecto**: Ultimo mes.

---

## Analisis de Carga y Deteccion de Atascos

Analiza la carga de trabajo actual del usuario y detecta posibles atascos.

**Para Desarrolladores**: Analisis separado por campo y resumen combinado:
```bash
./scripts/get_user_tickets.sh <uid> open "" "" developer | python3 scripts/calculate_ticket_stats.py workload
./scripts/get_user_tickets.sh <uid> open "" "" assigned | python3 scripts/calculate_ticket_stats.py workload
```

**Para Consultores**: Un solo analisis:
```bash
./scripts/get_user_tickets.sh <uid> open "" "" assigned | python3 scripts/calculate_ticket_stats.py workload
```

**Indicadores de carga elevada**:
- Mas de 15 tickets abiertos simultaneamente
- Mas de 5 tickets con antiguedad > 30 dias
- Mas de 3 tickets en la misma etapa durante > 14 dias (atasco)

**Indicadores de atasco**:
- Tickets en "Espera cliente" > 14 dias sin actividad en chatter
- Tickets en "En desarrollo" > 21 dias
- Tickets en "Testeo Desarrollo" > 14 dias
- Tickets en "Espera subida produccion" > 7 dias

**Analisis de chatter**: Para tickets atascados, consultar el chatter para identificar el motivo probable de la tardanza.
