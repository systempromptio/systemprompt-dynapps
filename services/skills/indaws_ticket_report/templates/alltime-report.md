# Template: Informe de Todos los Tiempos (Progreso de Resolución)

## Objetivo

Medir el progreso del usuario en resolución de tickets a lo largo del tiempo.
Muestra estadísticas globales + evolución mensual para detectar si el usuario
resuelve tickets cada vez más rápido o más lento.

## Datos necesarios

- Script tickets: `scripts/get_all_closed_tickets.sh <user_id> [field]`
- Script tendencia: `scripts/calculate_ticket_trend.py`
- Modelo: `references/models.md` > helpdesk.ticket

## Flujo de datos segun Rol

### Desarrollador (DOS secciones)

```bash
# Todos los tickets cerrados (sin filtro de fecha)
./scripts/get_all_closed_tickets.sh <uid> developer | python3 scripts/calculate_ticket_trend.py
./scripts/get_all_closed_tickets.sh <uid> assigned  | python3 scripts/calculate_ticket_trend.py
```

### Team Leader

Ejecutar el bloque de Desarrollador para sí mismo MÁS para cada miembro del equipo:

```bash
# Para cada dev a cargo
./scripts/get_all_closed_tickets.sh <uid_dev> developer | python3 scripts/calculate_ticket_trend.py
```

Al final del informe del TL, generar tabla comparativa del equipo.

## Formato de salida - Desarrollador

```
## Informe de Todos los Tiempos — [Nombre del usuario] (Desarrollador)
**Datos**: Todos los tickets cerrados desde el inicio

---

### Resumen Global

| Indicador | Dev asignado | Asignado a |
|-----------|:---:|:---:|
| Total tickets cerrados | [N] | [N] |
| Tiempo medio (histórico) | [duracion] | [duracion] |
| Mediana (histórico) | [duracion] | [duracion] |
| Tendencia (últimos 6 meses) | [emoji] [label] | [emoji] [label] |

> La tendencia se calcula comparando la mediana de la primera mitad vs la segunda mitad de los últimos 6 meses con datos.
> ↓ Mejorando = resuelves más rápido · ↑ Empeorando = tardas más · → Estable

---

### Como Desarrollador asignado
**Total tickets**: [N]

#### Estadísticas Globales

| Métrica | Valor |
|---------|-------|
| **Media** | [duracion] ([X.XX] días) |
| **Mediana** | [duracion] ([X.XX] días) |
| **Más rápido** | #[ID] - [nombre] ([duracion]) |
| **Más lento** | #[ID] - [nombre] ([duracion]) |

#### Distribución Global por Tiempo de Resolución

| Rango | Tickets | % |
|-------|:-------:|:---:|
| < 1 hora | [N] | [X.X]% |
| 1-8 horas | [N] | [X.X]% |
| 1-3 días | [N] | [X.X]% |
| 4-7 días | [N] | [X.X]% |
| 8-14 días | [N] | [X.X]% |
| 15-30 días | [N] | [X.X]% |
| > 30 días | [N] | [X.X]% |

#### Top 5 Más Rápidos (histórico)

| # | Ticket | Duración |
|---|--------|----------|
| 1 | #[ID] - [Nombre] | [duracion] |

#### Top 5 Más Lentos (histórico)

| # | Ticket | Duración |
|---|--------|----------|
| 1 | #[ID] - [Nombre] | [duracion] |

#### Evolución Mensual

Ordenado cronológicamente (más antiguo primero). Permite ver si la mediana baja con el tiempo.

| Mes | Tickets | Media | Mediana | Tendencia |
|-----|:-------:|-------|---------|:---------:|
| [Mes YYYY] | [N] | [duracion] | [duracion] | [emoji vs mes anterior] |

> Para la columna Tendencia de cada mes: comparar mediana con el mes anterior.
> ↓ si mejora (mediana menor), ↑ si empeora (mediana mayor), → si es similar (diferencia <10%), — si es el primero.

---

### Como Asignado a
**Total tickets**: [N]

[misma estructura que "Como Desarrollador asignado"]

---

### Observaciones

- [Observación basada en datos reales: tendencia general, meses con picos, si hay mejora sostenida, etc.]
- [Si la tendencia es "empeorando": indicar en qué meses empeoró y posible causa si es visible en los datos]
- [Si la tendencia es "mejorando": destacar el progreso]
```

## Formato de salida - Team Leader

Mostrar primero el bloque de Desarrollador (informe individual del TL), y luego:

```
---

## Comparativa del Equipo — Todos los Tiempos

| Desarrollador | Total tickets | Media global | Mediana global | Tendencia (6m) |
|---------------|:---:|-------|---------|:--------------:|
| [Nombre dev 1] | [N] | [duracion] | [duracion] | [emoji] [label] |
| [Nombre dev 2] | [N] | [duracion] | [duracion] | [emoji] [label] |
| **[TL - tú mismo]** | [N] | [duracion] | [duracion] | [emoji] [label] |

### Observaciones del equipo

- [Quién tiene mejor tendencia de mejora]
- [Quién tiene mayor volumen histórico]
- [Diferencias significativas en mediana entre miembros]
```

## Reglas

1. SIEMPRE usar `get_all_closed_tickets.sh` + `calculate_ticket_trend.py` para los datos. NUNCA usar get_closed_tickets.sh sin fechas.
2. SIEMPRE mostrar DOS secciones (developer + assigned) para Desarrollador.
3. La evolución mensual debe estar ordenada cronológicamente (ascendente), no descendente.
4. La columna "Tendencia" de cada fila mensual compara con el mes anterior (no con la media global).
5. Si un mes tiene menos de 3 tickets, advertir con una nota "(muestra pequeña)" en esa fila.
6. Para Team Leader: incluir siempre la tabla comparativa del equipo al final.
7. Las observaciones deben basarse en datos reales. No inventar causas.
8. Si el total de tickets es < 5, advertir que no hay suficientes datos para analizar tendencias.
