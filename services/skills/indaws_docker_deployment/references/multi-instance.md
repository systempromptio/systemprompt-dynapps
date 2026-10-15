# Multi-Instance

Convivir N proyectos Odoo dockerizados en el mismo host sin colisiones.

## Los 4 puntos donde pueden chocar

### 1. Puertos del host
Resuelto con el sistema de **slots** → `references/slot-allocation.md`.

### 2. Nombres de contenedor
Cada contenedor se prefija con `${PROJECT_NAME}`:
- `acme_odoo`, `acme_db`
- `beta_odoo`, `beta_db`

Regla: los `container_name` tienen que ser únicos en todo Docker, no solo dentro del compose.

### 3. Volúmenes nombrados
Prefijados igual:
- `acme_pgdata`, `acme_filestore`
- `beta_pgdata`, `beta_filestore`

Si no se prefijaran, dos proyectos con `volumes: pgdata` compartirían silenciosamente los datos (sin error, pero con corrupción semántica — desastre).

### 4. Redes
Cada proyecto tiene su red `${PROJECT_NAME}_net`. Los contenedores de un proyecto solo se ven entre ellos; no pueden llamar al Odoo de otro proyecto por hostname.

## Comunicación entre proyectos (si alguna vez hace falta)

Raro pero puede pasar: un script en `ACME` necesita leer la DB de `BETA` (comparativa, migración de datos, etc.).

Opciones:
1. **Vía host**: `psql -h localhost -p 5433 ...` (usa el puerto expuesto del slot)
2. **Red externa compartida**: crear `docker network create shared_dev` y añadirla como `external` en ambos composes. Los contenedores se ven por nombre: `beta_db:5432` desde dentro de `acme_odoo`.

La skill no añade la red compartida por defecto. Si la necesitas, se edita a mano.

## Recursos del host

Cada proyecto en idle consume aprox:
- Odoo: 300–600 MB RAM
- Postgres: 150–300 MB RAM

5 proyectos en paralelo → ~4–5 GB RAM. Ajustar expectativas según el host.

CPU en idle es despreciable. En uso activo (instalación de módulos, tests), Odoo puede comer 1–2 CPUs cada uno.

## Cómo listar qué tienes levantado

```bash
scripts/list.sh
```

Muestra tabla con: proyecto, slot, puertos, versión Odoo, estado de los contenedores (up/down/restarting), tamaño del volumen pgdata.

## Arrancar / parar en bloque

No hay comando explícito en la skill. Un oneliner:

```bash
# Parar todos los proyectos registrados
SLOTS="${DYNAPPS_HOME:-$HOME/.dynapps}/state/docker-deployment/slots.json"
[ -f "$SLOTS" ] || SLOTS="$HOME/.claude/skills/docker-deployment/state/slots.json"   # legacy
jq -r '.projects | keys[]' "$SLOTS" | \
  while read p; do scripts/down.sh "$p"; done
```

## Conflictos no gestionados por la skill

- Otros servicios del host que usen puertos del rango 18069-19069, 5432-5500, 3000-3100 (p.ej. un dev server local, otro postgres nativo)
- Docker networks creadas fuera de la skill con el mismo nombre `${PROJECT}_net`
- Volúmenes externos con el mismo nombre `${PROJECT}_pgdata`

En caso de conflicto, el compose se quejará al arrancar. La solución es ajustar el slot o renombrar.
