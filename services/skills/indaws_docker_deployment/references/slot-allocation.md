# Slot Allocation

Cada proyecto dockerizado recibe un **slot** numérico (entero ≥ 0). El slot determina los puertos que el contenedor expone al host. Esto permite convivir N proyectos sin que colisionen.

## Cálculo de puertos

| Servicio | Puerto interno | Puerto en host |
|---|---|---|
| Odoo HTTP | `8069` | `18069 + slot*100` |
| Odoo longpolling/gevent | `8072` | `18072 + slot*100` |
| Postgres | `5432` | `5432 + slot` |
| Debugpy | `3000` | `3000 + slot` |

## Tabla de referencia (slots 0–9)

| Slot | Odoo | Longpoll | DB | Debug |
|------|------|----------|------|-------|
| 0 | 18069 | 18072 | 5432 | 3000 |
| 1 | 18169 | 18172 | 5433 | 3001 |
| 2 | 18269 | 18272 | 5434 | 3002 |
| 3 | 18369 | 18372 | 5435 | 3003 |
| 4 | 18469 | 18472 | 5436 | 3004 |
| 5 | 18569 | 18572 | 5437 | 3005 |
| 6 | 18669 | 18672 | 5438 | 3006 |
| 7 | 18769 | 18772 | 5439 | 3007 |
| 8 | 18869 | 18872 | 5440 | 3008 |
| 9 | 18969 | 18972 | 5441 | 3009 |

## Registro

`${DYNAPPS_HOME:-~/.dynapps}/state/docker-deployment/slots.json` (legacy: `~/.claude/skills/docker-deployment/state/slots.json`):

```json
{
  "projects": {
    "ACME": {
      "slot": 0,
      "odoo_version": "18.0",
      "pg_version": "16",
      "path": "$PROJECTS_ROOT/ACME",
      "created_at": "2026-04-21T10:15:00Z"
    },
    "BETA": {
      "slot": 1,
      "odoo_version": "18.0",
      "pg_version": "16",
      "path": "$PROJECTS_ROOT/BETA",
      "created_at": "2026-04-21T11:00:00Z"
    }
  }
}
```

## Asignación

`scripts/new.sh` acepta `--slot N` explícito o asigna el **menor slot libre** si no se pasa. Un slot se considera libre si:

1. No aparece en `slots.json`
2. Todos sus puertos están libres en el host (`ss -tln` check)

Si el slot explícito ya está tomado, el script aborta y sugiere el siguiente libre.

## Por qué este esquema (y no otros)

- **No colisiona** entre proyectos gestionados por la skill (garantizado por el registro)
- **Determinista**: el mismo proyecto siempre abre en los mismos puertos → tus bookmarks, tus `launch.json` de VSCode, tus clientes SQL, todos siguen funcionando entre sesiones
- **Legible**: ver `18369` y saber que es el slot 3 es fácil
- **Escalable a ~99 proyectos** antes de solaparse con rangos altos (muy por encima del uso realista)

## Liberación

Un slot se libera cuando:

- `scripts/list.sh --cleanup` detecta que la carpeta del proyecto ya no existe
- Se borra manualmente del `slots.json`

`reset.sh` **NO** libera el slot (el proyecto sigue existiendo, solo se borran datos). Para liberar hay que borrar el registro explícitamente.
