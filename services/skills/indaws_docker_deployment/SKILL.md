---
name: indaws-docker-deployment
version: 1.3.7
effort: medium
model: sonnet
description: >
  Genera y administra entornos Docker locales Odoo (dev only) con compose parametrizado, Postgres
  dedicado, slots multi-instancia y restore de dumps (zip/sql/dump/gz).
  Triggers: "levantar odoo local", "docker local para cliente", "restaurar dump en docker",
  "nuevo docker para proyecto", "entorno docker odoo", "/docker-deployment".
---

# docker-deployment

Entornos Docker locales Odoo (dev only). No aplica a producción. Ver `references/architecture.md` para detalle completo de estructura, puertos, volúmenes y flujo típico.

## Workflow

1. **Gather**: proyecto, versión Odoo, ruta destino (`--path`), slot (opcional), EE/OCA/Indaws flags.
2. **Generate**: `scripts/new.sh <proyecto> <odoo_version> [--slot N] [--pg 16] [--path DIR] [--ee]`
3. **Restore** (si hay dump): `scripts/restore.sh <proyecto> <path-al-dump>` — cierra la red, carga (detecta zip/sql/dump/gz), neutraliza, audita, arranca Odoo en cuarentena y te pregunta antes de reabrir la salida a internet.
4. **Operate**: `up.sh` / `down.sh` / `logs.sh` / `reset.sh` / `list.sh`

## Commands

| Script | Qué hace |
|---|---|
| `new.sh <proj> <ver> [opts]` | Crea carpeta + compose + .env + odoo.conf. Slot libre si no se especifica. `--mount SRC:TARGET[:ro]` (repetible) añade o **sustituye** un montaje: es como se apunta una instancia de review al worktree de un PR |
| `restore.sh <proj> <dump>` | Drop+create DB, restaura dump + filestore y **neutraliza antes de arrancar Odoo**. `--db-only`, `--allow-errors`, `--admin-password PASS`, `--no-neutralize` (peligroso) |
| `neutralize.sh <proj>` | Deja la BD incapaz de hablar con el exterior: neutralize estándar (o SQL propio si <v16 — `odoo/cli/neutralize.py` aparece en **v16** y no existe en v14/v15, verificado 2026-08-07) + integraciones custom + caducidad EE + contraseña de admin conocida |
| `clone.sh <origen> <destino>` | Copia infra + BD + filestore a un proyecto nuevo con su propio Postgres. No para el origen (snapshot MVCC) ni comparte volúmenes. Neutraliza el clon por defecto |
| `destroy.sh <proj>` | Elimina el proyecto ENTERO: contenedores, volúmenes, carpeta y **libera el slot**. Cierra el ciclo de instancias efímeras (reviews). No borra la carpeta si es un repo git o no tiene compose |
| `reset.sh <proj>` | `down -v` + borra volúmenes. Mantiene compose/.env. Pide confirmación |
| `up.sh / down.sh` | Levantar / parar (sin borrar volúmenes). `down.sh` rechaza `-v`: para borrar datos, `reset.sh` |
| `net.sh <proj> close\|open\|status` | Cuarentena: corta/restaura la salida a internet sin perder los puertos publicados. Verifica el estado REAL en docker |
| `audit-integrations.sh <proj>` | Inventario de lo que puede hablar con el exterior: BD (crons, SMTP, webhooks, exports, jobs, endpoints en parámetros) + código de `addons/custom` |
| `logs.sh <proj> [svc]` | `docker compose logs -f` |
| `list.sh` | Tabla: proyecto / slot / puertos / estado / versión |

## PARADAS Y PREGUNTAS

Cinco puntos donde un script espera respuesta. **Todos fallan CERRADOS**: sin terminal interactiva
nunca se asume el "sí".

| Script | Cuándo para | Sin TTY |
|---|---|---|
| `restore.sh` **[STOP]** | al final: informe de auditoría + URL + credenciales, y "¿Abro la red ahora?" | la red se queda **cerrada** y se imprime el comando para abrirla |
| `up.sh` **[STOP]** | los workers reales no coinciden con el `.env`: pregunta cuántos quieres, lo aplica y revalida | avisa y sigue sin tocar nada |
| `reset.sh` **[STOP]** | antes de borrar datos: exige teclear el nombre del proyecto | `read` vacío → cancela |
| `destroy.sh` **[STOP]** | antes de borrar el proyecto entero: idem | cancela |
| `list.sh --cleanup` **[STOP]** | antes de desregistrar proyectos huérfanos | no borra nada |

`--yes` salta la confirmación de `reset.sh` y `destroy.sh`; `up.sh --no-checks` se salta la de workers.
**No preguntan** `net.sh`, `logs.sh`, `neutralize.sh` ni `new.sh`: hacen lo que se les pide. `down.sh`
tampoco, pero **rechaza `-v`** y te remite a `reset.sh` para no borrar volúmenes por descuido.

## GOTCHAS

1. **Los workers NO se controlan desde `odoo.conf`.** El entrypoint de la imagen añade `--workers=<nproc*2>` y `--limit-memory-soft/hard` al final de la línea de comandos cuando `ODOO_STAGE` no es `development`/`staging`, y en Odoo **gana el último flag** (verificado con su propio parser: `--workers=0 --workers=7` → 7). Medido: pidiendo 0 en el conf, arrancaba con **36**. Se controla con `ODOO_WORKERS` / `ODOO_CRON_THREADS` / `ODOO_LIMIT_MEM_*` en el `.env`, que consume un shim que `up.sh` siembra en `/var/lib/odoo/.local/bin/odoo` (directorio que el propio entrypoint antepone al PATH). `up.sh` además lee los workers reales del proceso y, si no cuadran, te pregunta y lo repara. **workers=0 = ThreadedServer**: imprescindible para debugpy, tours y `HttpCase`.
2. **v19: imagen sin código Odoo** — el fuente se monta por compose. `new.sh` lo resuelve con `${ODOO_CE_ROOT}:/home/odoo/odoo:ro` y `ODOO_CUSTOM_ADDONS_PATH=/addons`. Verificado ejecutando (2026-08-07): en `indaws/odoo:19.0`, `/home/odoo/odoo/` **existe y está vacío** y `/usr/local/bin/odoo` es un **symlink a `/home/odoo/odoo/odoo-bin`** — o sea, cuelga si no montas el fuente. En `indaws/odoo:18.0` ese directorio **ni existe** y el binario es un fichero real. Corolario: sin el montaje, el contenedor v19 no falla con un error de Odoo, falla con **`sh: 1: odoo: not found`**; si ves eso, no es la imagen rota, es el mount que falta.
3. **`odoo.conf` va en `/etc/odoo/base/odoo.conf`**, no en `/etc/odoo/odoo.conf` — el entrypoint hace cp al arrancar. Bind directo al destino da "Read-only file system".
4. **`requirements.txt` vacío en el filestore**: el entrypoint hace `open('/var/lib/odoo/requirements.txt')` al arrancar y sin el fichero muere con `FileNotFoundError` (reproducido). `up.sh` lo crea, idempotente, en el volumen named antes de levantar Odoo. El uid de `odoo` en `indaws/odoo` es **2000** (no 101), pero da igual: `/sudo-entrypoint.py` hace `chown -R odoo:odoo /var/lib/odoo` en cada arranque.
5. **Env vars `HOST/USER/PASSWORD/PORT/I_KNOW_WHAT_IM_DOING` obligatorias**: el entrypoint Indaws las necesita para conectar a Postgres. Sin ellas: `fe_sendauth: no password supplied`.
6. **Paquetes Python que faltan**: el entrypoint hace `glob('/addons/**/requirements.txt')`, los mergea y hace `pip install` en cada arranque — con `indawsodoo/{MAJOR}` montado, `pandas`/`openpyxl`/`xlrd` ya vienen de su `requirements.txt` de raíz. Si aun así falta alguno: `docker exec -u root <proj>_odoo pip install --break-system-packages <paquete>`. Ojo: son **2 `pip install` incondicionales por arranque** (incluido `pip install --upgrade pip`), así que el arranque necesita red.
7. **Cambio de versión Odoo con DB existente**: el arranque intentará migrar. Mejor `reset` + `restore`.
8. **Dumps grandes en ZIP**: descompresión y `pg_restore` pueden tardar minutos, y el script **no** informa del progreso: avisa antes de cada fase y se queda mudo hasta terminar. El ZIP se descomprime en `/tmp` (partición raíz), no en la del proyecto.
9. **Cambios en JS/XML/SCSS no se ven: `-u` NO basta, hay que reiniciar el proceso servido**. El `-u <módulo>` reconstruye el bundle de assets en `ir.attachment`, pero el **proceso Odoo ya arrancado sigue sirviendo el bundle viejo** en memoria. Un `-u --stop-after-init` en un `exec` aparte tampoco recarga el server que atiende peticiones. Secuencia correcta: **`-u <módulo>` → `docker compose down` → `docker compose up -d`**. NUNCA `docker compose restart` (no recarga bien) [[docker-odoo-reload-cycle]]. Ver `references/troubleshooting.md`.
10. **Nombre de proyecto compose = basename de la carpeta → colisión destructiva**. Si el `--path` acaba en un dir genérico (`.docker`, `docker`, `build`…), compose nombra el proyecto por ese basename y puede **recrear/absorber contenedores de otro compose homónimo** aunque los `container_name` sean únicos. `new.sh` lo blinda fijando `COMPOSE_PROJECT_NAME=<proyecto en minúsculas>` en el `.env`. Proyectos pre-fix: añade esa línea a mano al `.env`.
11. **`all predefined address pools have been fully subnetted`**: con muchos proyectos Docker agota los pools de red por defecto y `up` falla. `new.sh` asigna **subred /24 fija por slot** (`10.199.<slot>.0/24`) en el compose. Proyectos pre-fix: añade `ipam: config: - subnet: 10.199.<slot>.0/24` a la red del compose. `docker network prune -f` ayuda puntualmente pero no resuelve el pool.
12. **Tests/one-shot `odoo-bin` dentro de un contenedor con el server ya arriba → `Port 8069 is in use`**. `--no-http` **no basta, y no es cosa de gevent ni de v19**: con `--test-enable` el daemon HTTP arranca **igual**, porque el guard es `if config['test_enable'] or (config['http_enable'] and not stop): self.http_spawn()` — **`odoo/service/server.py:588-591`, `ThreadedServer.start` (v18)** y **`odoo/service/server.py:653-655`, `ThreadedServer.start` (v19)**, con el comentario de Odoo al lado: *"some tests need the http daemon to be available…"*. La **consecuencia** es idéntica en v18 y v19, pero la condición no: v18 evalúa `test_mode = config['test_enable'] or config['test_file']` (`odoo/service/server.py:588`, `ThreadedServer.start`, v18) y v19 solo `config['test_enable']` (`odoo/service/server.py:653`, `ThreadedServer.start`, v19). **Cuidado con el motivo: NO es que `--test-file` desaparezca en v19** —sigue siendo opción declarada (`odoo/tools/config.py:283-284`, `configmanager._build_cli`, v19) y v19 incluso la valida (`odoo/tools/config.py:707-710`, `configmanager`)—; lo que cambió es que `server.py` dejó de consultarla, así que **en v19 un `--test-file` sin `--test-enable` se queda sin daemon HTTP y en v18 lo tenía** (detalle en `references/troubleshooting.md`). El spawn de gevent, en cambio, **sí** está dentro del `if config['http_enable']` (`odoo18:957`, `odoo19:1038`), así que no es lo que ocupa el 8069. Consecuencia práctica: pasar `--no-http` junto a `--test-enable` es inútil; lo que funciona es mover los puertos: `--http-port=8199 --gevent-port=8299 --stop-after-init`.
13. **Montar un repo de addons entero contamina el entorno vía `auto_install`**. Al testear un set concreto de módulos, montar p.ej. todo `indawsodoo/{MAJOR}` hace que módulos `auto_install` (que cumplen deps) se instalen solos y cambien el comportamiento (visto: un bridge de Veri*Factu que forzaba envío síncrono a AEAT en `action_post` reventaba el flujo). Para testear un módulo aislado, monta **solo** su set de addons (quita EE/OCA/Indaws no necesarios del compose) y recrea la BD.
14. **El entrypoint de `indaws/odoo` exige que la BD destino EXISTA antes de arrancar** (loop "Trying to connect to postgresql" → `Database connection failure ... database "<db>" does not exist`). Para un one-shot de test crea la BD vacía primero: `docker exec <proj>_db psql -U odoo -d postgres -c "CREATE DATABASE <db> OWNER odoo;"` y luego `run --rm odoo odoo -d <db> -i <mod> --test-enable --test-tags /<mod> --stop-after-init`.
15. **`HttpCase.setUpClass` peta con `'PreforkServer' object has no attribute 'httpd'`**: es la consecuencia directa del gotcha 1 — el proceso arrancó en prefork. Con `ODOO_WORKERS=0` en el `.env` (comportamiento por defecto desde la v1.1.0) el server es ThreadedServer y los tours funcionan. Si lo ves, comprueba primero los workers reales: `up.sh <proj>` te lo dice.
16. **`docker compose -p` exige minúsculas.** `-p ACME18` → `invalid project name`. Usa `-p acme18`. (Complementa el gotcha 10: aislar con `-p <minúsculas>` evita la colisión de proyecto.)
17. **Dep en un repo NO montado → el módulo no instala** (falla por dependencia ausente). Añade ese repo al `addons_path` con una línea en `docker-compose.override.yml` (`- /ruta/al/repo:/addons/<x>:ro`) y **recrea SOLO el servicio odoo sin tocar la BD**: `docker compose -p <minúsculas> up -d --no-deps --force-recreate odoo` (el entrypoint regenera el `addons_path` e incluye el nuevo mount; `--no-deps` no reinicia Postgres). Sin `-p <minúsculas>` aplica la colisión de los gotchas 10/16. Es el reverso del gotcha 13: aquí falta un mount, allí sobran.
18. **Controller público da 404 "No database is selected" en instancia multi-DB.** Al hacer `curl` a un controller (`auth='public'`, p.ej. un webhook) contra un docker con varias BD y sin `dbfilter`, Odoo no sabe qué BD usar → 404 servidor-wide. Pasa la cabecera **`-H 'X-Odoo-Database: <db>'`** (o fija `dbfilter`/`--db-filter`). `HttpCase` no lo sufre (gestiona su propia BD).
19. **`odoo-bin neutralize` NO cubre las integraciones custom.** Apaga crons, servidores de correo y webhooks de los 65 módulos que traen su `neutralize.sql` (incluidos `l10n_es_edi_sii`, `l10n_es_edi_verifactu`, `iap`, `payment_*`, `delivery`), pero un REST a medida no está. Caso verificado en `indaws_data_export`: `_register_hook()` parchea `create/write/unlink` (dispara al tocar un registro, **no** por cron) y encola con `.with_delay()` en **OCA queue_job, que no tiene `neutralize.sql`** y cuyo runner es un `server_wide_module` con LISTEN/NOTIFY, tampoco un `ir_cron`. Encima el endpoint vive como registro en la BD, así que no aparece grepeando el módulo. Por eso `neutralize.sh` añade su propio SQL (`sql/neutralize-custom.sql`).
20. **Dos cosas quedan vivas a propósito tras neutralizar** y no son un fallo: el cron `base.autovacuum_job` y un `ir_mail_server` falso apuntando a `invalid:1025` (evita que Odoo caiga al SMTP de la config). El recuento de `neutralize.sh` las descuenta.
21. **`queue_job` fuera de `server_wide_modules` hasta que la BD esté restaurada y neutralizada.** Su jobrunner NO es un `ir.cron`: es un hilo que arranca con el servidor cuando el módulo está en `--load`, y escucha por `LISTEN/NOTIFY` de Postgres. Con un dump de producción recién restaurado eso **vacía de golpe la cola de jobs pendientes del cliente** (exports REST, ficheros a SFTP…), y `odoo-bin neutralize` no lo evita porque queue_job no trae `neutralize.sql`. El valor efectivo lo impone el shim con `--load` desde `ODOO_SERVER_WIDE_MODULES` en el `.env` (default `base,web`); el `server_wide_modules` del `odoo.conf` es orientativo, la CLI gana. `up.sh` avisa en rojo si detecta queue_job cargado con alguna BD sin neutralizar. Para activarlo tras verificar: `ODOO_SERVER_WIDE_MODULES=base,web,queue_job` + `down.sh` + `up.sh`.
22. **`web_enterprise` en `server_wide_modules` va segun el proyecto, no siempre.** `new.sh` lo incluye si el proyecto monta EE y lo omite si no; sin EE, Odoo escupe `Failed to load server-wide module` en cada arranque (no es fatal — `load_server_wide_modules` lo captura con un `_logger.exception` y sigue: **v18 `odoo/service/server.py:1346`, `load_server_wide_modules`; v19 `odoo/service/server.py:1523`, `load_server_wide_modules`** — pero parece un entorno roto). Ojo con la confusion: **quitarlo NO desactiva Enterprise**. Su manifest es `'auto_install': ['web']`, asi que se instala solo con estar en el `addons_path`; en `server_wide_modules` solo cambia lo que se sirve SIN base de datos (basicamente el gestor de BDs).
23. **Cuarentena de red: `enable_ip_masquerade=false`, NO `internal: true`.** Medido en el mismo host: con `internal: true` el puerto publicado se rompe (`http=000`) aunque el contenedor sigue siendo alcanzable por su IP; con `masquerade=false` el puerto publicado responde **200** y la salida está cortada (sonda dentro de Odoo: `OSError`). Cambiar `driver_opts` obliga a **recrear la red**, y si un contenedor ajeno está enganchado (un `docker run --network` de depuración) `compose down` no puede borrarla y el cambio no aplica en silencio: `net.sh` desengancha lo que sobre y **verifica el estado real** con `docker network inspect`, no el `.env`.
24. **Arrancar con la red cerrada exige dos parches, o no arranca.** (a) El `sudo-entrypoint.py` lanza `apt-get update` si encuentra CUALQUIER `apt-packages.txt` bajo `/addons` —Indaws Core trae uno con `poppler-utils`— y hace `sys.exit` si falla: **el contenedor muere**. `net.sh close` los tapa con un fichero vacío montado encima (funciona aunque el bind padre sea `:ro`). (b) `pip` sin red **se cuelga minutos por paquete**; con `PIP_NO_INDEX=1` falla en **1 segundo** (medido). Los `pip` fallando no matan el arranque, solo lo hacían eterno. Ambos parches van en un `docker-compose.quarantine.yml` enganchado por `COMPOSE_FILE` en el `.env`, así que los respeta hasta un `docker compose` a mano.
25. **La imagen `indaws/odoo` NO trae `wget`** (sí `curl` y `python3`). Una sonda de conectividad con `wget` dentro del contenedor da "sin salida" siempre, tenga red o no — falso negativo que puede hacerte creer que estás aislado cuando no lo estás. Para comprobar salida real: `docker exec <c> python3 -c "import socket; socket.create_connection(('deb.debian.org',80),5)"`.
26. **Instancia efímera para revisar un PR**: `--mount <worktree>:/addons/indaws` sustituye el montaje por defecto sin tocar el clon compartido (`indawsodoo/<MAJOR>` lo montan 13 proyectos: un `git checkout` ahí se los cambia a todos, en caliente). Al cerrar, `destroy.sh` — un `down` deja el proyecto registrado y el slot ocupado. Para el worktree de la rama **estable** hace falta `git worktree add --detach`: sin `--detach` git rechaza el segundo checkout de una rama ya usada por el clon principal.
27. **Clonar un proyecto trae los módulos ya instalados del origen** — cómodo para reproducir con datos del cliente, pero el gate de "instala en limpio" deja de probar nada: un `-u` pasa sin demostrar que el módulo instala de cero. Dilo en el veredicto o usa `restore.sh` sobre BD limpia. Y `clone.sh` **no comparte volúmenes**: el destino tiene su propio Postgres, porque compartirlo es lo que acaba escribiendo el review encima de tus datos.

## NEVER / ALWAYS

**NEVER:** Usar esta skill para deploys de producción (no contempla SSL, reverse proxy, secrets, backups). Montar el CE base — viene built-in en la imagen (conflicto con python path).
**ALWAYS:** Asignar slot libre si no se especifica. Declarar TODAS las rutas del host en `~/.config/docker-deployment/defaults.env`: la skill no lleva ninguna escrita a fuego.

## Delegates to

None — opera directamente con Docker y scripts bash.

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-flow-tester | common | Levantar entorno local para ejecutar flujos en modo C |
| developer-skills | developer | Entorno de desarrollo con dump del cliente |
| odoo-rescue-pack | developer | Postgres + restore del dump (sin Odoo) en el diagnóstico de rescate |

See `references/shared/odoo-connection.md` for Odoo API connection pattern (used in restore/verify steps).

## References

- `references/architecture.md` — Arquitectura, puertos, volúmenes, flujo típico completo
- `references/slot-allocation.md` — Cálculo de puertos y registro de slots
- `references/dump-restore.md` — Formatos soportados, edge cases
- `references/multi-instance.md` — Convivir N proyectos en el mismo host
- `references/odoo-conf.md` — Cada parámetro del `odoo.conf` y por qué ese valor
- `references/volumes-and-addons.md` — Orden del `addons_path`, rutas bind
- `references/troubleshooting.md` — Errores comunes y cómo resolverlos
