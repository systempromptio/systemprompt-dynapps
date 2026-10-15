# ${PROJECT_NAME} — docker local

Entorno Odoo dockerizado generado por `docker-deployment` skill.

## Acceso

- Odoo: http://localhost:${ODOO_PORT}
- Postgres: `localhost:${DB_PORT}` (user: `odoo`, pass: `odoo`)
- Debugpy: `localhost:${DEBUGPY_PORT}`
- Admin password: ver `.env` (`ADMIN_PASSWD`)

## Versiones

- Odoo: `${ODOO_VERSION}`
- Postgres: `${PG_VERSION}`
- Slot: `${SLOT}`

## Comandos rápidos

```bash
# Arrancar (y verificar que ha arrancado bien)
${SCRIPTS_DIR}/up.sh ${PROJECT_NAME}

# Parar (mantiene datos)
${SCRIPTS_DIR}/down.sh ${PROJECT_NAME}

# Logs en vivo
${SCRIPTS_DIR}/logs.sh ${PROJECT_NAME}

# Restaurar dump
${SCRIPTS_DIR}/restore.sh ${PROJECT_NAME} dumps/cliente.zip

# Empezar de cero (borra DB + filestore)
${SCRIPTS_DIR}/reset.sh ${PROJECT_NAME}
```

O directamente con docker compose desde esta carpeta:

```bash
docker compose up -d
docker compose logs -f odoo
docker compose down
```

## Estructura

```
.
├── docker-compose.yml   # NO editar vars; editar .env
├── .env                 # versiones, puertos, admin_passwd
├── config/
│   └── odoo.conf        # config de Odoo
├── addons/
│   └── custom/          # código del cliente aquí
├── dumps/               # dumps del cliente (gitignored)
└── README.md            # este fichero
```

## Addons montados (read-only salvo `custom`)

Rutas resueltas al generar este proyecto desde `~/.config/docker-deployment/defaults.env`:

${MOUNTS_LIST}

El entrypoint de la imagen escanea `/addons` y reconstruye el `addons_path` solo,
así que el orden del `odoo.conf` es solo orientativo. Odoo CE ≤18 va dentro de la
imagen y no se monta.

## Gotchas

- **Workers y crons a 0** (`ODOO_WORKERS` / `ODOO_CRON_THREADS` en `.env`, no en
  `odoo.conf`: el entrypoint pisa el fichero, y un shim en el volumen impone estos
  valores por encima). 0 workers = ThreadedServer, necesario para debugpy y tours.
- **Red** (`NET_MASQUERADE` en `.env`): `false` deja el contenedor sin salida a
  internet. `restore.sh` la cierra mientras hay un dump de cliente sin verificar.
- El filestore persiste entre `down`/`up`. Solo `reset.sh` lo borra.
- Cambiar la versión Odoo con una DB ya restaurada puede provocar migración al arrancar.
