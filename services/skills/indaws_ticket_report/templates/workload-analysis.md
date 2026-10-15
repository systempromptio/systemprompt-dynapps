# Template: Analisis de Carga y Deteccion de Atascos

## Datos necesarios

- Script tickets abiertos: `scripts/get_user_tickets.sh <user_id> open "" "" [field]`
- Script chatter (para atascados): `scripts/get_ticket_chatter.sh <ticket_ids_json>`
- Calculos: `scripts/calculate_ticket_stats.py workload`
- Modelo: `references/models.md` > helpdesk.ticket, mail.message

## Flujo de datos segun Rol

### Desarrollador (DOS analisis + resumen combinado)

```bash
# Seccion 1: Carga como Desarrollador asignado
./scripts/get_user_tickets.sh <uid> open "" "" developer > /tmp/open_dev.json
cat /tmp/open_dev.json | python3 scripts/calculate_ticket_stats.py workload

# Seccion 2: Carga como Asignado a
./scripts/get_user_tickets.sh <uid> open "" "" assigned > /tmp/open_assigned.json
cat /tmp/open_assigned.json | python3 scripts/calculate_ticket_stats.py workload

# 3. Si hay tickets atascados en cualquiera, obtener chatter
./scripts/get_ticket_chatter.sh "[id1,id2,...]"
```

Formato de salida:

```
## Analisis de Carga - [Nombre del usuario] (Desarrollador)
**Fecha**: [fecha actual]

### Carga como Desarrollador asignado
**Nivel de carga**: [INDICADOR] [NIVEL]
**Tickets abiertos**: [N]

[resumen + distribucion + atascos]

---

### Carga como Asignado a
**Nivel de carga**: [INDICADOR] [NIVEL]
**Tickets abiertos**: [N]

[resumen + distribucion + atascos]

---

### Resumen Global
| Indicador | Dev asignado | Asignado a |
|-----------|-------------|------------|
| Tickets abiertos | [N] | [N] |
| Tickets antiguos (>30d) | [N] | [N] |
| Tickets atascados | [N] | [N] |
| Nivel de carga | [nivel] | [nivel] |

### Recomendaciones
[basadas en ambos analisis]
```

### Consultor (UN analisis)

```bash
./scripts/get_user_tickets.sh <uid> open "" "" assigned > /tmp/open.json
cat /tmp/open.json | python3 scripts/calculate_ticket_stats.py workload

# Si hay tickets atascados, obtener chatter
./scripts/get_ticket_chatter.sh "[id1,id2,...]"
```

Formato de salida:

```
## Analisis de Carga - [Nombre del usuario]
**Fecha**: [fecha actual]
**Nivel de carga**: [INDICADOR] [NIVEL]

[resumen + distribucion + atascos + recomendaciones]
```

## Encabezado con indicador de carga (comun)

Niveles de carga:
- `Normal` - Carga dentro de parametros normales
- `Moderada` - Carga por encima de lo normal, vigilar
- `Alta` - Carga elevada, requiere accion

## Resumen de carga (comun, por seccion)

```
### Resumen

| Indicador | Valor | Umbral |
|-----------|-------|--------|
| Tickets abiertos | [N] | <= 15 |
| Tickets antiguos (>30d) | [N] | <= 5 |
| Tickets atascados | [N] | <= 3 |

[Si carga es alta o moderada:]
**Motivos**: [lista de razones]
```

## Distribucion por etapa (comun, por seccion)

```
### Distribucion por Etapa

| Etapa | Cantidad | Antiguedad media |
|-------|----------|-----------------|
| [Nombre] | [N] | [X] dias |
```

## Tickets atascados (comun)

```
### Tickets Atascados

Los siguientes tickets muestran senales de atasco:

| # | Ticket | Etapa | Antiguedad | Motivo del atasco |
|---|--------|-------|------------|-------------------|
| 1 | #[ID] - [Nombre] | [Etapa] | [N] dias | [Motivo detectado] |
```

## Analisis de motivos (solo si hay tickets atascados)

Para cada ticket atascado, analizar el chatter y reportar:

```
### Analisis Detallado de Atascos

**#[ID] - [Nombre]** ([N] dias en [etapa])
- **Motivo probable**: [descripcion basada en el chatter]
- **Ultimo mensaje**: [fecha] - [resumen del mensaje]
- **Accion recomendada**: [que hacer para desbloquear]
```

## Tickets antiguos (>30 dias, comun)

```
### Tickets Antiguos (>30 dias)

| # | Ticket | Etapa | Antiguedad | Ultima actividad |
|---|--------|-------|------------|------------------|
| 1 | #[ID] - [Nombre] | [Etapa] | [N] dias | Hace [N] dias |
```

## Recomendaciones

```
### Recomendaciones

1. [Recomendacion 1 basada en los datos]
2. [Recomendacion 2]
```

## Reglas

1. SIEMPRE usar `calculate_ticket_stats.py workload` para los calculos
2. SIEMPRE mostrar el nivel de carga en el encabezado
3. Si hay tickets atascados, OBLIGATORIO consultar el chatter con `get_ticket_chatter.sh`
4. El analisis del chatter debe ser concreto: buscar palabras clave
5. Las recomendaciones deben ser accionables y especificas
6. No inventar motivos; si el chatter no revela el motivo, indicar "Motivo no identificado en chatter - revisar manualmente"
7. **Desarrollador**: SIEMPRE mostrar ambas secciones + tabla resumen global
8. **Consultor**: Una sola seccion sin tabla comparativa

## Umbrales de deteccion de atascos (referencia)

| Etapa | Umbral | Indica |
|-------|--------|--------|
| New | > 7 dias | Ticket sin asignar demasiado tiempo |
| Espera cliente | > 14 dias sin actividad | Cliente no responde |
| En desarrollo | > 21 dias | Desarrollo demasiado largo |
| Testeo Desarrollo | > 14 dias | Testing prolongado |
| Espera subida produccion | > 7 dias | Deploy pendiente |

## Palabras clave para analisis de chatter

Para identificar motivos de tardanza, buscar en el chatter:
- **Espera cliente**: "espera cliente", "esperando", "pendiente respuesta", "me confirma"
- **Falta informacion**: "falta informacion", "necesito", "podrian enviar"
- **Bloqueo tecnico**: "bloqueado", "error", "no se puede", "impedimento"
- **Testing**: "pruebas", "testing", "validar", "comprobar"
- **Produccion**: "produccion", "deploy", "subida", "master"
- **Baja prioridad**: "cuando se pueda", "no urgente", "baja prioridad"
