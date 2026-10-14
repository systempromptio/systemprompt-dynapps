---
name: indaws-ticket-report
description: Informe y análisis de tickets de Helpdesk para usuarios de Indaws (desarrolladores y consultores). Usa este skill cuando el usuario pregunte sobre sus tickets abiertos ("¿Qué tickets tengo?", "tickets prioritarios"), tickets cerrados ("¿Qué tickets he cerrado esta semana?"), tiempos de resolución ("media de resolución", "tiempo medio de cierre"), o carga de trabajo ("¿Tengo mucha carga?", "¿Hay atascos?", "carga de tickets"). Solicita las credenciales de conexión al usuario si no están configuradas.
version: 1.1.2
effort: medium
model: sonnet
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion]
---

# Ticket Report - Informe de Tickets Helpdesk

Skill para consultar, analizar y reportar tickets de Helpdesk del Odoo corporativo de Indaws.
Soporta 4 roles: Desarrollador, Consultor, Team Leader y Project Manager.

See `references/shared/odoo-connection.md` for connection pattern and `references/shared/batch-limits.md` for pagination rules.

## Flujo de inicio

1. Solicitar credenciales (ver `references/credentials.md`) -- usuario y API Key obligatorios
2. Exportar variables de entorno con valores por defecto para URL/DB/version
3. Preguntar rol con AskUserQuestion (ver `references/roles.md`)
4. Identificar usuario en Odoo (buscar res.users por login para obtener ID)
5. Mostrar mensaje de bienvenida segun rol

## Reglas CRITICAS

- **NUNCA** hacer llamadas directas a la API -- usar SIEMPRE los scripts de `scripts/`
- **NUNCA** inventar nombres de modelos o campos -- si no esta en `references/models.md`, no existe
- **NUNCA** calcular estadisticas manualmente -- usar `calculate_ticket_stats.py`
- **SIEMPRE** incluir `"context": {"lang": "es_ES"}` (los scripts lo hacen automaticamente)
- **SIEMPRE** leer el template correspondiente ANTES de generar cualquier informe
- **SIEMPRE** seguir EXACTAMENTE el formato del template (columnas, orden, indicadores)
- Si el informe solicitado no existe en la tabla de informes: PREGUNTAR con AskUserQuestion

## Comportamiento por rol (resumen)

| Rol | Campos de busqueda | Secciones por informe |
|-----|--------------------|-----------------------|
| Desarrollador | `x_studio_assigned_developer_id` + `user_id` | 2 (una por campo) |
| Consultor | `user_id` | 1 |
| Team Leader | Como Desarrollador + informe de equipo (dept 4) | 2 + equipo |
| Project Manager | Solo `user_id` (field=assigned siempre) | 1 + equipo |

Detalle completo de cada rol, welcome messages y ejecucion: `references/roles.md`

## Gotchas

1. **Duplicados entre secciones**: Un ticket puede aparecer en ambas secciones (dev + assigned) para Desarrolladores. NO eliminar duplicados -- cada seccion representa un rol diferente
2. **PM vs TL**: PM usa SOLO `user_id` (field=assigned), TL usa ambos campos. PM busca subordinados en cualquier departamento, TL solo en dept 4
3. **Periodos en curso**: Si el periodo incluye hoy, fecha_fin = HOY (no el final teorico). Ver `references/stages.md`
4. **Informe mensual dia 1-5**: Advertir que la muestra es muy pequena y sugerir mes pasado
5. **Paginacion**: Los scripts paginan automaticamente (100 en 100). No limitar manualmente
6. **Etapas abiertas**: IDs [1, 2, 5, 6, 14, 15]. Solved (3) y Cancelled (4) son cerrados
7. **Seguimiento >1 semana**: Filtrar client-side tickets abiertos con antiguedad >7 dias, con accion recomendada segun etapa y tiempo

## Estructura

```
ticket-report/
  SKILL.md
  templates/          # Formatos de salida obligatorios
  references/         # Documentacion tecnica
  scripts/            # Scripts para consultas Odoo
```

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Ticket data from Helpdesk API |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| Bug triage | developer | Open ticket context |
| Service-desk context | consultant | Ticket analysis and reporting |

## REFERENCES

| Archivo | Contenido |
|---------|-----------|
| `references/models.md` | Modelos Odoo, campos, relaciones, filtros obligatorios |
| `references/roles.md` | Deteccion de rol, comportamiento por rol, welcome messages, ejecucion PM |
| `references/reports.md` | Tabla de informes con triggers, ejecucion detallada por tipo de informe |
| `references/scripts.md` | Documentacion de cada script: parametros, ejemplos, modos |
| `references/credentials.md` | Solicitud de credenciales, valores por defecto, variables de entorno |
| `references/stages.md` | Etapas de tickets (IDs), reglas de periodos en curso |

## Principios de respuesta

- Indica siempre el periodo consultado.
