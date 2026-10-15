# Template: Estadisticas de Tiempo de Resolucion

## Datos necesarios

- Script: `scripts/get_closed_tickets.sh <user_id> <date_from> <date_to> [field]`
- Calculos: `scripts/calculate_ticket_stats.py resolution`
- Modelo: `references/models.md` > helpdesk.ticket

## Flujo de datos segun Rol

### Desarrollador (DOS calculos + resumen combinado)

```bash
# Seccion 1: Como Desarrollador asignado
./scripts/get_closed_tickets.sh <uid> <from> <to> developer > /tmp/closed_dev.json
cat /tmp/closed_dev.json | python3 scripts/calculate_ticket_stats.py resolution

# Seccion 2: Como Asignado a
./scripts/get_closed_tickets.sh <uid> <from> <to> assigned > /tmp/closed_assigned.json
cat /tmp/closed_assigned.json | python3 scripts/calculate_ticket_stats.py resolution
```

Formato de salida:

```
## Estadisticas de Resolucion - [Nombre del usuario] (Desarrollador)
**Periodo**: [fecha_inicio] a [fecha_fin]

### Como Desarrollador asignado
**Tickets analizados**: [N]

[tabla de metricas]
[distribucion]
[top 5]

---

### Como Asignado a
**Tickets analizados**: [N]

[tabla de metricas]
[distribucion]
[top 5]

---

### Comparativa
| Metrica | Dev asignado | Asignado a |
|---------|-------------|------------|
| Total tickets | [N] | [N] |
| Media | [duracion] | [duracion] |
| Mediana | [duracion] | [duracion] |
```

### Consultor (UN calculo)

```bash
./scripts/get_closed_tickets.sh <uid> <from> <to> assigned > /tmp/closed.json
cat /tmp/closed.json | python3 scripts/calculate_ticket_stats.py resolution
```

Formato de salida:

```
## Estadisticas de Resolucion - [Nombre del usuario]
**Periodo**: [fecha_inicio] a [fecha_fin]
**Total tickets analizados**: [N]

[tabla de metricas]
[distribucion]
[top 5]
[observaciones]
```

## Tabla de metricas (comun)

```
### Tiempos de Resolucion

| Metrica | Valor |
|---------|-------|
| **Media** | [duracion] ([X.XX] dias) |
| **Mediana** | [duracion] ([X.XX] dias) |
| **Mas rapido** | #[ID] - [nombre] ([duracion]) |
| **Mas lento** | #[ID] - [nombre] ([duracion]) |
```

## Distribucion por rangos (comun)

```
### Distribucion por Tiempo de Resolucion

| Rango | Tickets | Porcentaje |
|-------|---------|------------|
| < 1 hora | [N] | [X.X]% |
| 1-8 horas | [N] | [X.X]% |
| 1-3 dias | [N] | [X.X]% |
| 4-7 dias | [N] | [X.X]% |
| 8-14 dias | [N] | [X.X]% |
| 15-30 dias | [N] | [X.X]% |
| > 30 dias | [N] | [X.X]% |
```

## Top tickets (comun)

```
### Top 5 Mas Rapidos

| # | Ticket | Duracion |
|---|--------|----------|
| 1 | #[ID] - [Nombre] | [duracion] |

### Top 5 Mas Lentos

| # | Ticket | Duracion |
|---|--------|----------|
| 1 | #[ID] - [Nombre] | [duracion] |
```

## Observaciones

```
### Observaciones

- [Observacion 1 basada en los datos]
- [Observacion 2]
```

## Reglas

1. SIEMPRE usar `calculate_ticket_stats.py resolution` para los calculos
2. SIEMPRE incluir media, mediana, distribucion y top 5
3. La mediana es mas representativa que la media; indicar esto si difieren mucho
4. Periodo por defecto: ultimo mes
5. Si hay menos de 3 tickets, advertir que la muestra es pequena
6. Las observaciones deben basarse en datos reales, no inventar
7. **Desarrollador**: SIEMPRE mostrar ambas secciones + tabla comparativa
8. **Consultor**: Una sola seccion sin comparativa
