# odoo.conf — justificación de cada parámetro

Este documento explica **por qué** cada parámetro tiene el valor que tiene. No es documentación oficial de Odoo; es el razonamiento de diseño para **dev local**.

## [options]

### Admin

```ini
admin_passwd = ${ADMIN_PASSWD}
```
Parametrizable vía `.env`. Default `admin`. No se hardcodea porque eventualmente se comparte compose con otros devs y no queremos leak de `admin` en caso de que se crucen puertos con un entorno menos aislado.

### Database

```ini
db_host = db
db_port = 5432
db_user = odoo
db_password = odoo
db_name = False
db_maxconn = 64
list_db = True
dbfilter =
```

- `db_host = db`: nombre del servicio Postgres dentro de la red Docker del proyecto. No es `localhost`: Odoo corre en su contenedor.
- `db_port = 5432`: puerto interno (no el del host).
- `db_user / db_password = odoo/odoo`: credenciales fijas. Estamos en local, no hay superficie de ataque.
- `db_name = False`: Odoo no fuerza una DB específica. Se elige desde el manager o con `-d` en CLI. Clave para poder restaurar dumps con cualquier nombre.
- `db_maxconn = 64`: por debajo del límite típico de Postgres (100). Con `workers=0` sobra mucho, pero deja margen para scripts ad-hoc.
- `list_db = True`: queremos ver el manager en dev. En prod sería `False`.
- `dbfilter = <vacío>`: ves todas las DBs. Decidido explícitamente frente a `^${PROJECT}.*$` para evitar frustración al restaurar dumps con nombres inesperados.

### Paths

```ini
data_dir = /var/lib/odoo
addons_path = /home/odoo/addons/custom,/home/odoo/addons/indaws,/home/odoo/addons/oca,/home/odoo/addons/enterprise,/home/odoo/addons/odoo
```

Orden del `addons_path`: **primera coincidencia gana**.

1. `custom` — módulos del cliente. Prioridad máxima porque pueden sobrescribir (heredar) cualquier cosa.
2. `indaws` — suite Indaws Core.
3. `oca` — OCA.
4. `enterprise` — módulos Enterprise oficiales.
5. `odoo` — addons Community base.

Si un nombre de módulo existiera en varios directorios (raro pero ocurre cuando OCA clona un estándar), se toma el de la izquierda.

`data_dir = /var/lib/odoo` coincide con el volumen nombrado `${PROJECT}_filestore`. Ahí viven los attachments. Es lo que Odoo entiende por "filestore path".

### HTTP

```ini
http_enable = True
http_interface = 0.0.0.0
http_port = 8069
gevent_port = 8072
proxy_mode = False
```

- `http_interface = 0.0.0.0`: escucha en todas las interfaces **del contenedor**. Luego Docker mapea al puerto del slot en el host.
- `gevent_port = 8072`: longpolling. Lo expone el compose al puerto del slot (18072, 18172...).
- `proxy_mode = False`: no hay nginx delante. Si algún día se pone un reverse proxy en dev, cambiar a `True` para que Odoo respete `X-Forwarded-*`.

### Workers / concurrency

```ini
workers = 0
max_cron_threads = 0
```

**Decisión crítica para dev**.

- `workers = 0`: modo "threaded" single-process. Necesario para que debugpy attache correctamente. Con `workers > 0`, Odoo prefork y debugpy solo pilla el master; los workers forkeados no heredan el attach.
- `max_cron_threads = 0`: **cero crons automáticos**. Hacer debug mientras te saltan crons cada 5 min es una tortura. Si quieres probar un cron concreto, ejecútalo a mano desde UI → Ajustes → Técnico → Acciones planificadas → "Ejecutar manualmente".

Si hay que probar escenarios que sí requieren crons (p.ej. colas queue_job), se activa puntualmente en el `.env` → `odoo.conf` y `docker compose restart odoo`.

### Límites desactivados

```ini
limit_time_cpu = 99999
limit_time_real = 99999
limit_memory_soft = 0
limit_memory_hard = 0
```

En prod matan workers que se quedan colgados. En dev te mata la request mientras piensas con un breakpoint abierto. Desactivados.

`limit_memory_*` a `0` significa "desactivado" en Odoo (no "0 bytes").

**Ojo si el módulo lanza SUBPROCESOS** (un CLI de IA, `wkhtmltopdf`, `pandoc`, `ffmpeg`): Odoo aplica
`limit_memory_soft` como **`RLIMIT_AS` del worker**, es decir **espacio de direcciones**, y el hijo lo
**hereda**. Runtimes node/rust/Go reservan direcciones muy por encima de la memoria que usan, así que
con el default (2560 MB) el hijo muere con *"OS can't spawn worker thread: Resource temporarily
unavailable"* mientras el MISMO código funciona en `odoo shell` (que no tiene rlimit). Ese contraste
es el diagnóstico: si va en shell y no en la request, mira el rlimit antes que tu código.

En dev con `0` no hay problema. En cualquier entorno donde quieras conservar el reciclado de workers
(`0` también lo desactiva), pon un valor **grande y FINITO** (p.ej. 20 GB de espacio de direcciones) y
vigila la memoria real a nivel de contenedor, no con `RLIMIT_AS`.

### Logging

```ini
log_level = info
log_handler = :INFO,werkzeug:WARNING,odoo.modules.loading:WARNING
logfile = False
```

- `log_level = info`: el sweet spot. `debug` satura de ruido (SQL, ORM internals), `warn` oculta cosas útiles.
- `log_handler`: silencia werkzeug (peticiones HTTP, cada F5 es una línea) y el ruido de carga de módulos al arrancar. El `:INFO` deja todo lo demás al nivel `info`.
- `logfile = False`: log a stdout → `docker compose logs -f odoo` lo ve.

### Dev mode

```ini
dev_mode = False
```

**Off por defecto**. Razones:

- `reload` a veces reinicia Odoo en medio de una request mientras editas y provoca errores confusos
- `qweb` y `xml` fuerzan no cachear, lo que ralentiza todo
- Preferimos activar explícitamente por CLI cuando interese:
  ```bash
  docker compose run --rm odoo odoo --dev=reload,qweb,xml
  ```

### Tests / Demo

```ini
without_demo = False
test_enable = False
test_tags = False
```

- `without_demo = False`: permite data demo al crear DBs vacías. Útil para probar un módulo desde cero sin dump del cliente.
- Tests off por defecto. Se activan por CLI cuando se corren suites:
  ```bash
  docker compose run --rm odoo odoo --test-enable --test-tags=/mi_modulo
  ```

### Server-wide modules

```ini
server_wide_modules = base,web,web_enterprise
```

`web_enterprise` cargado de serie porque la mayoría de proyectos del parque son EE. Si el proyecto es CE puro, editar `.env`/`odoo.conf` y quitarlo.

**No incluye `queue_job`** aunque lo use Indaws Core: provoca que el runner intente procesar jobs al arrancar y puede fallar si aún no hay DB configurada. Se carga a nivel de DB cuando se instala el módulo, que es suficiente.

### Misc

```ini
unaccent = True
csv_internal_sep = ,
reportgz = False
screencasts = /tmp/odoo/screencasts
screenshots = /tmp/odoo/screenshots
```

- `unaccent = True`: habilita el operador `=ilike` con insensibilidad a tildes. Requiere extensión Postgres `unaccent`, que `indaws/odoo` instala automáticamente al crear DB.
- `reportgz = False`: los reports PDF sin gzip; más fácil de inspeccionar.
- `screencasts/screenshots`: para tests con grabación. `/tmp` es efímero dentro del contenedor — si se quieren persistir, montar un volumen.

## Cambios que NO hay que hacer al conf

- No meter rutas de `addons_path` absolutas del host. Las rutas son del contenedor. El bind se hace en el `docker-compose.yml`.
- No cambiar `db_host` a `localhost`. Rompe la red interna de Docker.
- No meter `admin_passwd` en claro — usar la variable `${ADMIN_PASSWD}`.
