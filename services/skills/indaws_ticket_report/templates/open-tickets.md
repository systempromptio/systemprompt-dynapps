# Template: Tickets Abiertos Prioritarios

## Datos necesarios

- Script: `scripts/get_user_tickets.sh <user_id> open [date_from] [date_to] [field]`
- Modelo: `references/models.md` > helpdesk.ticket

## Formato de salida según Rol

### Desarrollador (DOS secciones)

```bash
# Ejecutar DOS veces
./scripts/get_user_tickets.sh <uid> open "" "" developer   # Sección 1
./scripts/get_user_tickets.sh <uid> open "" "" assigned     # Sección 2
```

Formato de salida:

```
## Tickets Abiertos - [Nombre del usuario] (Desarrollador)
**Periodo**: [fecha_inicio] a [fecha_fin] (o "Todos los abiertos" si no hay filtro)

### Tickets como Desarrollador asignado
**Total**: [N] tickets

[tabla de tickets]

### Tickets como Asignado a
**Total**: [N] tickets

[tabla de tickets]

### Resumen global
**Total combinado**: [N dev] + [N asignado] = [total] tickets (pueden existir duplicados entre secciones)
```

### Consultor (UNA seccion)

```bash
# Ejecutar UNA vez
./scripts/get_user_tickets.sh <uid> open "" "" assigned
```

Formato de salida:

```
## Tickets Abiertos - [Nombre del usuario]
**Periodo**: [fecha_inicio] a [fecha_fin] (o "Todos los abiertos" si no hay filtro)
**Total**: [N] tickets abiertos

[tabla de tickets]
```

## Tabla de tickets (comun a ambos roles)

Ordenar por: prioridad (descendente), luego por antiguedad (mas antiguos primero).

```
| # | Ticket | Prioridad | Etapa | Antiguedad | Ultima actividad | Cliente/Proyecto |
|---|--------|-----------|-------|------------|------------------|-----------------|
| 1 | #[ID] - [Nombre] | [emoji_prioridad] [texto] | [Nombre etapa] | [N] dias | Hace [N] dias | [cliente_proyecto] |
```

### Indicadores de prioridad

| Valor | Emoji | Texto |
|-------|-------|-------|
| 3 | !!! | Critico |
| 2 | !! | Muy urgente |
| 1 | ! | Urgente |
| 0 | - | Normal |

### Resumen por etapa

Mostrar para cada seccion (si es desarrollador) o una vez (si es consultor):

```
### Distribucion por Etapa

| Etapa | Cantidad |
|-------|----------|
| [Nombre etapa] | [N] |
```

### Alertas

Si hay tickets con antiguedad > 30 dias:

```
### Tickets que requieren atencion

Los siguientes tickets llevan mas de 30 dias abiertos:
- #[ID] - [Nombre]: [N] dias en etapa [etapa]
```

## Reglas

1. SIEMPRE mostrar la tabla ordenada por prioridad y antiguedad
2. SIEMPRE incluir el resumen por etapa
3. Si hay tickets > 30 dias, SIEMPRE mostrar la seccion de alertas
4. Si no hay tickets abiertos, indicar "No tienes tickets abiertos en esta categoria"
5. Mostrar la antiguedad en dias (no horas ni minutos)
6. La "Ultima actividad" se calcula como: hoy - write_date
7. **Desarrollador**: SIEMPRE mostrar ambas secciones, incluso si una esta vacia
8. **Consultor**: Una sola seccion sin distincion de campo
