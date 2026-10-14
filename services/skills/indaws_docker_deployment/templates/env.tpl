# Generado por docker-deployment skill — editable
# Cambios aquí se aplican con: docker compose up -d --force-recreate

# Identidad del proyecto
PROJECT_NAME=${PROJECT_NAME}
# Nombre del proyecto compose. FIJARLO es imprescindible: sin esto docker
# compose usa el basename de la carpeta (p.ej. `.docker` -> proyecto `docker`),
# que colisiona con otros composes homónimos y puede recrear/absorber sus
# contenedores. Debe ir en minúsculas (requisito de compose).
COMPOSE_PROJECT_NAME=${PROJECT_SLUG}

# Versiones de imágenes
ODOO_VERSION=${ODOO_VERSION}
ODOO_MAJOR=${ODOO_MAJOR}
PG_VERSION=${PG_VERSION}

# Slot y puertos (calculados; no editar a mano sin actualizar slots.json)
SLOT=${SLOT}
ODOO_PORT=${ODOO_PORT}
LONGPOLL_PORT=${LONGPOLL_PORT}
DB_PORT=${DB_PORT}
DEBUGPY_PORT=${DEBUGPY_PORT}

# Credenciales
ADMIN_PASSWD=admin

# ── Workers ─────────────────────────────────────────────────────────
# ODOO_STAGE=development hace que el entrypoint pida --workers=0 y no
# fuerce limites de memoria. Ademas, el shim del volumen anade los flags
# de abajo al final de la linea de comandos, asi que mandan aunque la
# imagen o su entrypoint cambien.
# workers=0 -> ThreadedServer: imprescindible para debugpy, tours y HttpCase.
# Subirlos SOLO despues de restaurar y comprobar que la instancia va.
ODOO_STAGE=development
ODOO_WORKERS=0
ODOO_CRON_THREADS=0

# ── Modulos server-wide (--load) ────────────────────────────────────
# DELIBERADAMENTE SIN queue_job. El jobrunner de OCA queue_job no es un
# ir.cron: es un hilo que arranca con el servidor cuando el modulo esta
# aqui, y escucha por LISTEN/NOTIFY de Postgres. Con un dump de produccion
# recien restaurado, eso dispara DE GOLPE todos los jobs pendientes del
# cliente (exports REST, ficheros a SFTP...) — y `odoo-bin neutralize` no
# los para, porque queue_job no trae neutralize.sql.
# Anadelo SOLO despues de restaurar, neutralizar y comprobar la instancia:
#   ODOO_SERVER_WIDE_MODULES=base,web,queue_job   +   down.sh && up.sh
# `web_enterprise` se incluye AUTOMATICAMENTE si este proyecto monta EE, y se
# omite si no (sin EE, Odoo escupe un traceback en cada arranque). No hace
# falta para que Enterprise funcione: su manifest es 'auto_install': ['web'],
# asi que se instala solo con estar en el addons_path; en server_wide_modules
# solo cambia lo que se sirve SIN base de datos (el gestor de BDs).
ODOO_SERVER_WIDE_MODULES=${SERVER_WIDE_MODULES}
# 0 = sin limite. Evita que un worker muera a media importacion de un dump.
ODOO_LIMIT_MEM_SOFT=0
ODOO_LIMIT_MEM_HARD=0

# ── Red ─────────────────────────────────────────────────────────────
# true  = el contenedor sale a internet (necesario para apt/pip del arranque)
# false = cuarentena: sin salida. Lo conmuta net.sh; restore.sh lo cierra
#         automaticamente mientras hay un dump de cliente sin verificar.
NET_MASQUERADE=true

# Credenciales Postgres (consumidas tanto por el servicio db como por odoo)
# La imagen indaws/odoo lee HOST/USER/PASSWORD/PORT del environment al arrancar.
DB_USER=odoo
DB_PASSWORD=odoo
