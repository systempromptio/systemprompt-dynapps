[options]
; ── Admin ──────────────────────────────────────────
admin_passwd = ${ADMIN_PASSWD}

; ── Database ───────────────────────────────────────
db_host = db
db_port = 5432
db_user = odoo
db_password = odoo
db_name = False
db_maxconn = 64
list_db = True
dbfilter =

; ── Paths ──────────────────────────────────────────
data_dir = /var/lib/odoo
; El entrypoint de indaws/odoo regenera addons_path con find_files('/addons').
; Mantenemos esta linea solo como hint - el orden real lo determina el entrypoint.
addons_path = /addons/custom,/addons/indaws,/addons/oca,/addons/enterprise

; ── HTTP ───────────────────────────────────────────
http_enable = True
http_interface = 0.0.0.0
http_port = 8069
gevent_port = 8072
proxy_mode = False

; ── Workers / concurrency (dev = sin workers) ──────
workers = 0
max_cron_threads = 0

; ── Limits desactivados (breakpoints largos) ───────
limit_time_cpu = 99999
limit_time_real = 99999
limit_memory_soft = 0
limit_memory_hard = 0

; ── Logging ────────────────────────────────────────
log_level = info
log_handler = :INFO,werkzeug:WARNING,odoo.modules.loading:WARNING
logfile = False

; ── Dev mode (off por defecto, activable por CLI) ──
dev_mode = False

; ── Tests / Demo ───────────────────────────────────
without_demo = False
test_enable = False
test_tags = False

; ── Server-wide modules ────────────────────────────
; ORIENTATIVO: el valor efectivo lo impone el shim con --load, leyendo
; ODOO_SERVER_WIDE_MODULES del .env (la CLI gana sobre este fichero).
; Sin queue_job a proposito: su jobrunner dispararia los jobs pendientes de
; un dump de produccion nada mas arrancar. Ver el .env.
; web_enterprise se incluye solo si el proyecto monta EE.
server_wide_modules = ${SERVER_WIDE_MODULES}

; ── Misc ───────────────────────────────────────────
unaccent = True
csv_internal_sep = ,
reportgz = False
screencasts = /tmp/odoo/screencasts
screenshots = /tmp/odoo/screenshots
