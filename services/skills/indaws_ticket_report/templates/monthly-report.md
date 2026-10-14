# Template: Informe Mensual de Tickets

## Datos necesarios

- Scripts: `get_closed_tickets.sh`, `get_user_tickets.sh`, `calculate_ticket_stats.py`
- Modelo: `references/models.md` > helpdesk.ticket

## Selección de Periodo

Antes de ejecutar, calcular las fechas según lo que indique el usuario:

| Selección | date_from | date_to |
|-----------|-----------|---------|
| "mes actual" | Primer día del mes actual (YYYY-MM-01) | Hoy (YYYY-MM-DD) |
| "mes pasado" | Primer día del mes anterior | Último día del mes anterior |
| "enero 2026" / mes concreto | Primer día de ese mes | Último día de ese mes (o hoy si es el mes en curso) |

Si el usuario no especifica el mes, preguntar con AskUserQuestion:
- Mes actual
- Mes pasado
- Mes concreto (pedir que especifique)

## Flujo de datos segun Rol

### Desarrollador

```bash
# Tickets cerrados del mes
./scripts/get_closed_tickets.sh <uid> <date_from> <date_to> developer > /tmp/closed_dev.json
./scripts/get_closed_tickets.sh <uid> <date_from> <date_to> assigned > /tmp/closed_assigned.json

# Estadísticas de resolución
cat /tmp/closed_dev.json | python3 scripts/calculate_ticket_stats.py resolution > /tmp/stats_dev.json
cat /tmp/closed_assigned.json | python3 scripts/calculate_ticket_stats.py resolution > /tmp/stats_assigned.json

# Tickets abiertos al cierre del periodo
./scripts/get_user_tickets.sh <uid> open "" "" developer
./scripts/get_user_tickets.sh <uid> open "" "" assigned
```

### Team Leader

Ejecutar el bloque de Desarrollador para sí mismo, MÁS para cada miembro del equipo:

```bash
# Para cada desarrollador a cargo (user_id de cada uno)
./scripts/get_closed_tickets.sh <uid_dev> <date_from> <date_to> developer | python3 scripts/calculate_ticket_stats.py resolution
./scripts/get_closed_tickets.sh <uid_dev> <date_from> <date_to> assigned | python3 scripts/calculate_ticket_stats.py resolution
```

## Formato de salida - Desarrollador

```
## Informe Mensual - [Nombre del usuario] (Desarrollador)
**Periodo**: [Nombre del mes] [año] ([date_from] - [date_to])

---

### Resumen del mes

| Indicador | Desarrollador asignado | Asignado a |
|-----------|:---:|:---:|
| Tickets cerrados | [N] | [N] |
| Tickets abiertos actualmente | [N] | [N] |
| Tiempo medio de resolución | [duracion] | [duracion] |
| Mediana de resolución | [duracion] | [duracion] |

---

### Tickets Cerrados - Como Desarrollador asignado
**Total**: [N] tickets

| # | Ticket | Fecha cierre | Tiempo resolución | Cliente/Proyecto |
|---|--------|-------------|-------------------|-----------------|
| 1 | #[ID] - [Nombre] | [DD/MM/YYYY] | [duracion] | [cliente_proyecto] |

#### Resumen de Resolución (Desarrollador asignado)
- **Tiempo medio**: [duracion]
- **Mediana**: [duracion]
- **Más rápido**: #[ID] - [nombre] ([duracion])
- **Más lento**: #[ID] - [nombre] ([duracion])

---

### Tickets Cerrados - Como Asignado a
**Total**: [N] tickets

| # | Ticket | Fecha cierre | Tiempo resolución | Cliente/Proyecto |
|---|--------|-------------|-------------------|-----------------|
| 1 | #[ID] - [Nombre] | [DD/MM/YYYY] | [duracion] | [cliente_proyecto] |

#### Resumen de Resolución (Asignado a)
- **Tiempo medio**: [duracion]
- **Mediana**: [duracion]
- **Más rápido**: #[ID] - [nombre] ([duracion])
- **Más lento**: #[ID] - [nombre] ([duracion])

---

### Estadísticas de Resolución del Mes

> Si la muestra es < 3 tickets, advertir que no es representativa.

#### Como Desarrollador asignado
**Tickets analizados**: [N]

| Métrica | Valor |
|---------|-------|
| **Media** | [duracion] ([X.XX] días) |
| **Mediana** | [duracion] ([X.XX] días) |
| **Más rápido** | #[ID] - [nombre] ([duracion]) |
| **Más lento** | #[ID] - [nombre] ([duracion]) |

| Rango | Tickets | % |
|-------|:-------:|:---:|
| < 1 hora | [N] | [X.X]% |
| 1-8 horas | [N] | [X.X]% |
| 1-3 días | [N] | [X.X]% |
| 4-7 días | [N] | [X.X]% |
| 8-14 días | [N] | [X.X]% |
| 15-30 días | [N] | [X.X]% |
| > 30 días | [N] | [X.X]% |

**Top 5 Más Rápidos**

| # | Ticket | Duración |
|---|--------|----------|
| 1 | #[ID] - [Nombre] | [duracion] |

**Top 5 Más Lentos**

| # | Ticket | Duración |
|---|--------|----------|
| 1 | #[ID] - [Nombre] | [duracion] |

---

#### Como Asignado a
**Tickets analizados**: [N]

[misma estructura que "Como Desarrollador asignado"]

---

#### Comparativa

| Métrica | Dev asignado | Asignado a |
|---------|:-----------:|:----------:|
| Total tickets | [N] | [N] |
| Media | [duracion] | [duracion] |
| Mediana | [duracion] | [duracion] |
| Más rápido | [duracion] | [duracion] |
| Más lento | [duracion] | [duracion] |

---

### Tickets Abiertos al Cierre del Periodo

#### Como Desarrollador asignado
**Total**: [N] tickets

| # | Ticket | Prioridad | Etapa | Antigüedad | Última actividad |
|---|--------|-----------|-------|------------|-----------------|
| 1 | #[ID] - [Nombre] | [emoji] [texto] | [Etapa] | [N] días | Hace [N] días |

#### Como Asignado a
**Total**: [N] tickets

| # | Ticket | Prioridad | Etapa | Antigüedad | Última actividad |
|---|--------|-----------|-------|------------|-----------------|
| 1 | #[ID] - [Nombre] | [emoji] [texto] | [Etapa] | [N] días | Hace [N] días |

#### Tickets que requieren atención (>30 días abiertos)
(Mostrar solo si existen)

- #[ID] - [Nombre]: [N] días en etapa [etapa] → [Acción recomendada]
```

## Formato de salida - Team Leader

Mostrar primero el bloque de Desarrollador (informe individual del TL), y luego añadir:

```
---

## Comparativa del Equipo — [Nombre del mes] [año]

> La mediana es más representativa que la media cuando hay tickets muy lentos.

| Desarrollador | Tickets cerrados | Media | Mediana | Más rápido | Más lento |
|---------------|:---:|-------|---------|------------|-----------|
| [Nombre dev 1] | [N] | [duracion] | [duracion] | [duracion] | [duracion] |
| [Nombre dev 2] | [N] | [duracion] | [duracion] | [duracion] | [duracion] |
| [Nombre dev N] | [N] | [duracion] | [duracion] | [duracion] | [duracion] |
| **[TL - tú mismo]** | [N] | [duracion] | [duracion] | [duracion] | [duracion] |

### Distribución por Tiempo de Resolución — Comparativa

| Rango | [Dev 1] | [Dev 2] | [Dev N] |
|-------|:-------:|:-------:|:-------:|
| < 1 hora | [N] ([X]%) | [N] ([X]%) | [N] ([X]%) |
| 1-8 horas | [N] ([X]%) | [N] ([X]%) | [N] ([X]%) |
| 1-3 días | [N] ([X]%) | [N] ([X]%) | [N] ([X]%) |
| 4-7 días | [N] ([X]%) | [N] ([X]%) | [N] ([X]%) |
| 8-14 días | [N] ([X]%) | [N] ([X]%) | [N] ([X]%) |
| 15-30 días | [N] ([X]%) | [N] ([X]%) | [N] ([X]%) |
| > 30 días | [N] ([X]%) | [N] ([X]%) | [N] ([X]%) |

### Observaciones del equipo

- [Observación basada en los datos reales: quién resuelve más rápido, quién tiene más carga, outliers, etc.]
```

## Reglas

1. SIEMPRE usar `calculate_ticket_stats.py resolution` para los cálculos. NUNCA calcular manualmente.
2. SIEMPRE mostrar ambas secciones (developer + assigned) para el rol Desarrollador.
3. Si no hay tickets cerrados en una sección, indicar "No se cerraron tickets en este periodo como [rol]".
4. Si la muestra es < 3 tickets, advertir que las estadísticas no son representativas.
5. Ordenar tickets cerrados por fecha de cierre descendente (más recientes primero).
6. Ordenar tickets abiertos por prioridad descendente, luego antigüedad ascendente.
7. Para Team Leader: SIEMPRE incluir la tabla comparativa del equipo al final.
8. Las observaciones deben basarse en datos reales, no inventar.
9. Si el usuario pide "mes actual" y estamos a día 1-5 del mes, advertir que la muestra es muy pequeña y sugerir usar el mes pasado.
10. Indicadores de prioridad: 3=!!! Crítico, 2=!! Muy urgente, 1=! Urgente, 0=- Normal.
