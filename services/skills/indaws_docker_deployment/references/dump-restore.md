# Dump Restore

`scripts/restore.sh <proyecto> <path-al-dump>` gestiona 4 formatos. El script detecta el formato por los primeros bytes (magic bytes), no por la extensión.

## Formatos soportados

### 1. ZIP de Odoo (el más común)

Es el formato que Odoo genera desde `/web/database/manager` → "Backup". Contiene:

```
dump.sql          # SQL plano con schema + datos
filestore/        # attachments, imágenes de productos, etc.
manifest.json     # metadatos (opcional)
```

Flujo:
1. Extrae el zip a `/tmp/odoo-restore-XXXX`
2. Detecta `dump.sql` en la raíz
3. Drop + create DB dentro del contenedor `${PROJECT}_db`
4. `psql -f dump.sql` contra la DB recién creada
5. Copia el directorio `filestore/` al volumen `${PROJECT}_filestore` (ruta interna: `/var/lib/odoo/filestore/<DB_NAME>`)
6. Limpia `/tmp/odoo-restore-XXXX`
7. Reinicia el contenedor Odoo

Magic bytes: `PK\x03\x04` (o `PK\x05\x06` para zips vacíos).

### 2. SQL plano (`.sql`)

Fichero SQL exportado con `pg_dump -Fp` o por herramientas externas.

Flujo:
1. Drop + create DB
2. `psql -f <archivo.sql>`
3. Sin filestore (hay que gestionarlo aparte si aplica)

Detección: contenido empieza con `--`, `SET`, `BEGIN`, `CREATE`, o similar patrón SQL.

### 3. Custom pg_dump (`.dump`)

Formato binario de `pg_dump -Fc`. Más rápido de restaurar, paralelizable.

Flujo:
1. Drop + create DB
2. `pg_restore -d <db> --no-owner --no-privileges <archivo.dump>`
3. Sin filestore (gestionar aparte)

Magic bytes: `PGDMP` al principio.

### 4. Comprimido (`.gz`, `.tar.gz`)

Se descomprime a tmp y se procesa recursivamente con detección.

- `.gz` de un `.sql` → SQL plano tras gunzip
- `.gz` de un `.dump` → pg_restore tras gunzip
- `.tar.gz` con estructura Odoo (dump.sql + filestore/) → ZIP-equivalente

Magic bytes: `\x1f\x8b`.

## Parámetros de base de datos

El script usa los valores del `.env` del proyecto:
- Host: `localhost` (desde el host) o `db` (desde dentro del contenedor)
- Puerto: `DB_PORT` (calculado por slot)
- Usuario: `odoo`
- Password: `odoo`
- Nombre DB: `${PROJECT_NAME}` en minúsculas (configurable con `--db-name`)

## Pre-requisitos en el host

- `docker` y `docker compose` instalados
- `unzip`, `gzip`, `tar` disponibles (viene de serie en distros estándar)
- `psql` y `pg_restore` clientes (paquete `postgresql-client` en Debian/Ubuntu). El script usa los binarios dentro del contenedor cuando puede, para evitar depender del cliente en el host.

## Edge cases

### El contenedor Odoo está corriendo

El script detecta y hace `docker compose stop odoo` antes de operar sobre la DB. Al final, `docker compose up -d odoo`.

### La DB ya existe

Se hace `DROP DATABASE <name> WITH (FORCE)` (Postgres 13+) para cortar conexiones activas. Si falla, se avisa al usuario y se aborta.

### Filestore vacío o ausente en el ZIP

No error. Se avisa. Los attachments aparecerán rotos hasta que se rellene o se resuba.

### Versión Postgres del dump ≠ versión en el compose

`pg_restore`/`psql` avisan en el log pero suele funcionar entre versiones cercanas (13→16). Si falla, el script sugiere cambiar `PG_VERSION` en `.env` temporalmente.

### Dumps con encoding exótico

El script fuerza `PGCLIENTENCODING=UTF8`. Si el dump viene en LATIN1/LATIN9, habrá que convertirlo antes (no está en el alcance del script).

### Dumps con owner/permisos de otro servidor

Se restaura con `--no-owner --no-privileges`. El owner pasa a ser `odoo` siempre.

## Comandos manuales equivalentes

Si el script falla y necesitas hacerlo a mano:

```bash
# ZIP
unzip -d /tmp/rst client.zip
docker compose exec -T db dropdb -U odoo acme --force
docker compose exec -T db createdb -U odoo acme
docker compose exec -T db psql -U odoo -d acme < /tmp/rst/dump.sql
docker cp /tmp/rst/filestore <PROJECT>_odoo:/var/lib/odoo/filestore/acme

# pg_dump custom
docker compose exec -T db dropdb -U odoo acme --force
docker compose exec -T db createdb -U odoo acme
docker cp client.dump <PROJECT>_db:/tmp/
docker compose exec -T db pg_restore -U odoo -d acme --no-owner --no-privileges /tmp/client.dump
```
