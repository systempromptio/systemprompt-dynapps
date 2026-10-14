# Docker Deployment — Architecture & Detail

## Arquitectura completa

Cada proyecto = un `docker-compose.yml` propio, con su propio contenedor Postgres dedicado, volúmenes nombrados prefijados (`${PROJECT}_pgdata`, `${PROJECT}_filestore`), red propia (`${PROJECT}_net`), y un slot numérico que determina sus puertos.

## Registro de slots

`${DYNAPPS_HOME:-~/.dynapps}/state/docker-deployment/slots.json` mantiene qué proyecto usa qué slot. Los puertos se calculan deterministas:

- Odoo HTTP: `18069 + slot*100`
- Longpolling: `18072 + slot*100`
- Postgres: `5432 + slot`
- Debugpy: `3000 + slot`

Ver `references/slot-allocation.md` para detalle completo.

## Imágenes Docker

- `indaws/odoo:${ODOO_VERSION}` (Docker Hub)
- `postgres:${PG_VERSION}` (por defecto 16)

El `.env` del proyecto controla las versiones; cambiar de 17 a 18 es editar `ODOO_VERSION=18.0` y `./scripts/up.sh`.

## Volúmenes

**Persistentes** (sobreviven a `down`, mueren con `reset`):
- `${PROJECT}_pgdata` → `/var/lib/postgresql/data`
- `${PROJECT}_filestore` → `/var/lib/odoo`

**Binds** (montajes desde host, solo lectura del source oficial):
- `./config/odoo.conf` → `/etc/odoo/base/odoo.conf` (ver gotcha 8)
- `./addons/custom` → `/addons/custom`
- `${EE_SRC}` → `/addons/enterprise` (ro) — Odoo EE
- `${OCA_SRC}` → `/addons/oca` (ro) — repos OCA
- `${INDAWS_SRC}` → `/addons/indaws` (ro) — Indaws Core
- Odoo CE base viene built-in en la imagen, no se montea.

Las rutas se resuelven desde **`~/.config/docker-deployment/defaults.env`** (se crea en la primera ejecución de `new.sh`, y las claves nuevas se le añaden solas en actualizaciones posteriores). Cada variable es una plantilla con `{MAJOR}` → se sustituye con la mayor Odoo. Si una ruta no existe o está comentada, ese montaje se omite (sin fallar).

## Archivos generados por proyecto

```
<PROJECT_PATH>/
├── docker-compose.yml    # parametrizado por .env
├── .env                  # PROJECT_NAME, ODOO_VERSION, SLOT, ports, admin_passwd
├── .gitignore
├── config/
│   └── odoo.conf         # sin dev_mode, sin workers, sin crons (dev-friendly)
├── addons/
│   └── custom/           # aquí va el código del cliente
├── dumps/                # .gitignored; aquí caen los .zip/.sql/.dump del cliente
└── README.md             # URL, puertos, comandos rápidos
```

## Flujo típico

```bash
./scripts/new.sh ACME 18.0 --path $PROJECTS_ROOT/ACME
cp ~/Downloads/acme-prod.zip $PROJECTS_ROOT/ACME/dumps/
./scripts/up.sh ACME
./scripts/restore.sh ACME $PROJECTS_ROOT/ACME/dumps/acme-prod.zip
# Abrir http://localhost:18069 (slot 0) · Debugpy en :3000
./scripts/down.sh ACME          # parar, mantener datos
./scripts/reset.sh ACME         # empezar de cero (borra DB + filestore)
```
