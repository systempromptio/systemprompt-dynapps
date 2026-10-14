# Template: Tickets Cerrados en Periodo

## Datos necesarios

- Script: `scripts/get_closed_tickets.sh <user_id> <date_from> <date_to> [field]`
- Modelo: `references/models.md` > helpdesk.ticket

## Formato de salida segun Rol

### Desarrollador (DOS secciones)

```bash
./scripts/get_closed_tickets.sh <uid> <from> <to> developer   # Seccion 1
./scripts/get_closed_tickets.sh <uid> <from> <to> assigned     # Seccion 2
```

Formato de salida:

```
## Tickets Cerrados - [Nombre del usuario] (Desarrollador)
**Periodo**: [fecha_inicio] a [fecha_fin]

### Cerrados como Desarrollador asignado
**Total**: [N] tickets

[tabla de tickets]

### Resumen de Resolucion (Desarrollador asignado)
- **Tiempo medio**: [duracion]
- **Mas rapido**: #[ID] - [nombre] ([duracion])
- **Mas lento**: #[ID] - [nombre] ([duracion])

---

### Cerrados como Asignado a
**Total**: [N] tickets

[tabla de tickets]

### Resumen de Resolucion (Asignado a)
- **Tiempo medio**: [duracion]
- **Mas rapido**: #[ID] - [nombre] ([duracion])
- **Mas lento**: #[ID] - [nombre] ([duracion])
```

### Consultor (UNA seccion)

```bash
./scripts/get_closed_tickets.sh <uid> <from> <to> assigned
```

Formato de salida:

```
## Tickets Cerrados - [Nombre del usuario]
**Periodo**: [fecha_inicio] a [fecha_fin]
**Total cerrados**: [N] tickets

[tabla de tickets]

### Resumen de Resolucion
- **Total cerrados**: [N] tickets
- **Tiempo medio de resolucion**: [duracion]
- **Mas rapido**: #[ID] - [nombre] ([duracion])
- **Mas lento**: #[ID] - [nombre] ([duracion])
```

## Tabla de tickets (comun a ambos roles)

Ordenar por: fecha de cierre (mas recientes primero).

```
| # | Ticket | Fecha cierre | Tiempo resolucion | Cliente/Proyecto |
|---|--------|-------------|-------------------|-----------------|
| 1 | #[ID] - [Nombre] | [DD/MM/YYYY] | [duracion] | [cliente_proyecto] |
```

## Reglas

1. SIEMPRE mostrar la tabla ordenada por fecha de cierre descendente
2. SIEMPRE incluir el resumen de tiempos
3. Si no hay tickets cerrados en el periodo, indicar "No se cerraron tickets en este periodo"
4. El tiempo de resolucion se calcula como: close_date - create_date
5. Formatear duracion como: "Xd Xh Xm" para duraciones largas, "Xh Xm" para mismo dia
6. Periodo por defecto: semana actual (lunes a hoy)
7. **Desarrollador**: SIEMPRE mostrar ambas secciones con resumenes separados
8. **Consultor**: Una sola seccion con resumen unico
