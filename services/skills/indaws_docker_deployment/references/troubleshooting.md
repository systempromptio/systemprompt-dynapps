# Troubleshooting

Errores comunes al usar la skill, con causa y solución.

---

## Arranque del compose

### `AttributeError: module 'lib' has no attribute 'X509_V_FLAG_NOTIFY_POLICY'` (imagen indaws/odoo v18)

**Causa**: mismatch pyOpenSSL/cryptography dentro de la imagen `indaws/odoo:18.0` cuando
se arranca Odoo con `--entrypoint odoo` directo (saltándose el entrypoint normal de la
imagen). `base`/`web` fallan a cargar y la BD no inicializa.

**Solución**:
- Para un **smoke-test throwaway** (instalar + tests, sin persistencia), usar la imagen
  oficial `odoo:18.0`, que trae deps limpias. Montar Enterprise/Indaws como addons:
  ```bash
  docker network create t_net; \
  docker run -d --name t_pg --network t_net \
    -e POSTGRES_USER=odoo -e POSTGRES_PASSWORD=odoo -e POSTGRES_DB=postgres postgres:16
  docker run --rm --network t_net \
    -v $ODOO_ENTERPRISE_SRC/18:/mnt/ee:ro -v /ruta/addons:/mnt/custom:ro \
    --entrypoint odoo odoo:18.0 \
    -d test --db_host=t_pg --db_user=odoo --db_password=odoo --data-dir=/tmp/od \
    --addons-path=/usr/lib/python3/dist-packages/odoo/addons,/mnt/ee,/mnt/custom \
    -i <module> --test-enable --test-tags /<module> --stop-after-init \
    --http-port=8199 --gevent-port=8299
  ```
  **No pongas `--no-http` aquí**: con `--test-enable` el daemon HTTP arranca igual
  (`if config['test_enable'] or (config['http_enable'] and not stop)` —
  `odoo/service/server.py:586`, `ThreadedServer.start` (v18); `odoo/service/server.py:649`, `ThreadedServer.start` (v19), y el comentario de Odoo lo dice:
  *"some tests need the http daemon to be available"*). Si hay otro server arriba en el
  mismo contenedor, lo que evita el `Port 8069 is in use` es **mover los puertos**, no
  desactivar http. Verificado 2026-08-07 en v18 y v19 (mismo código).
- **Ojo con el addons-path por imagen**: oficial `odoo:18.0` →
  `/usr/lib/python3/dist-packages/odoo/addons`; `indaws/odoo:18.0` →
  `/usr/local/lib/python3.11/dist-packages/odoo/addons`.
- `iot` (Enterprise) solo depende de `mail`+`web` (community): basta montar
  `$ODOO_ENTERPRISE_SRC/18` para instalarlo, sin `web_enterprise`.

---

### `bind source path does not exist: $ODOO_SRC/odoo18/addons`

**Causa**: Falta una de las rutas de sources oficiales en el host.

**Solución**:
- Clonar el repo correspondiente. Ver el CLAUDE.md global del entorno para las rutas esperadas.
- O comentar el bind en `docker-compose.yml` si no te interesa (p.ej. no tienes Enterprise).

---

### `Error response from daemon: Conflict. The container name "acme_odoo" is already in use`

**Causa**: Existe un contenedor con ese nombre de un `up` anterior que no se limpió.

**Solución**:
```bash
docker rm -f acme_odoo acme_db
docker compose up -d
```

O usar `scripts/down.sh` antes.

---

### `port is already allocated`

**Causa**: Otro proceso ocupa el puerto del slot (otro proyecto, un postgres nativo del host, un dev server).

**Solución**:
1. `ss -tln | grep <puerto>` → ver qué lo ocupa
2. Si es otro proyecto de la skill: `scripts/list.sh` + `scripts/down.sh <otro>`
3. Si es otro servicio: asignar otro slot al proyecto editando `.env` y `slots.json`, luego `docker compose up -d`

---

### `pull access denied for indaws/odoo`

**Causa**: Imagen no existe con ese tag, o falta auth en Docker Hub.

**Solución**:
```bash
docker login
docker pull indaws/odoo:18.0        # o la versión que toque
```

Verificar tags disponibles en Docker Hub antes de pedir versiones raras.

---

## Conexión a Odoo

### Cambio en JS/XML/SCSS no se refleja (bundle de assets viejo)

**Síntoma**: editas un `.js`/`.xml`/`.scss` del módulo, corres `-u <módulo>` y en el navegador sigue
el comportamiento/estilo antiguo (aunque los datos/campos sí se actualizaron).

**Causa**: `-u` reconstruye el bundle de assets en `ir.attachment`, pero el **proceso Odoo ya
arrancado sirve el bundle cacheado en memoria**. Un `-u --stop-after-init` ejecutado en un `exec`
aparte actualiza la BD pero NO el server que atiende el navegador.

**Solución (secuencia, regla dura — NUNCA `restart`)**:
```bash
# 1) reconstruir el bundle en BD
docker compose exec -T odoo bash -lc '$ODOO_SRC/odooNN/odoo-bin -c /etc/odoo/odoo.conf -d <DB> -u <módulo> --stop-after-init'
# 2) reiniciar el proceso servido para que cargue el bundle nuevo
docker compose down && docker compose up -d
```
`docker compose restart` no recarga fiable el bundle; usa siempre `down`→`up`. (Cambios solo de
Python: basta `-u` + `down`/`up`; NO hace falta tocar assets.)

---

### `<body>` vacío y CERO errores: la pestaña está oculta (rAF congelado)

**Síntoma**: el backend responde 200, el login pinta bien, pero `/odoo` deja el `<body>` vacío (0
hijos). Ni un error en consola, ni una petición fallida: los assets, `load_menus`, `translations` y
`/mail/data` devuelven 200, `odoo.loader.failed` está vacío y `odoo.isReady` es `false` para siempre.

**Causa**: el scheduler de Owl monta con `requestAnimationFrame`, y Chrome **no dispara rAF en una
pestaña oculta** (ventana minimizada, en otro escritorio/workspace, o pestaña de fondo). Sin rAF no
monta nada — ni el WebClient ni un componente trivial con plantilla inline — y como no hay excepción
tampoco hay nada en consola. `setTimeout` sí corre, así que el resto de la página "parece viva".

**Comprobación decisiva (2 líneas, hazla PRIMERO)**:

```js
document.hidden               // true  -> es esto, no tu código
requestAnimationFrame(() => (window.__raf = 1)); setTimeout(() => console.log(window.__raf), 200)
// undefined tras 200 ms con setTimeout funcionando = rAF congelado
```

**Solución**: hacer visible la ventana (traerla al frente / desminimizar). Al hacerlo montan de golpe
todos los árboles pendientes. Si conduces el navegador por automatización (CDP/MCP) y no puedes
garantizar el foco, parchea el scheduler de la app ya montada para las pruebas:

```js
const app = odoo.__WOWL_DEBUG__.root.__owl__.app;
app.scheduler.requestAnimationFrame = (cb) => setTimeout(() => cb(performance.now()), 16);
```

Ojo: es de ida y vuelta — con la pestaña oculta **el DOM se queda congelado también al leerlo**, así
que un `querySelector` puede devolver el estado de hace tres acciones. Verifica el estado en el
servicio/servidor, no en el DOM, mientras no tengas foco.

**Lo que NO es**: no es la BD, ni los bundles, ni la memoria, ni subprocesos huérfanos. Este triage
(módulos JS cargados, `odoo.loader.failed` vacío, bundle que parsea con `node --check`, endpoints en
milisegundos, RSS normal) sirve para descartar "es mi código", pero si todo eso está bien **la
conclusión correcta es mirar `document.hidden`, no recrear la BD**. Una versión anterior de esta
página recomendaba recrear la BD: era falso, y costó una instancia sana y varias horas.

---

### El websocket no conecta (`Couldn't bind the websocket... evented port 8072`)

**Síntoma**: en el log, `RuntimeError: Couldn't bind the websocket. Is the connection opened on the
evented port (8072)?` + `KeyError: 'socket'` repetidos. En el navegador, `new WebSocket('ws://host/websocket')`
queda en `readyState 3`.

**Causa**: con `workers > 0` Odoo sirve el websocket en el **puerto gevent (8072)**, no en el 8069. El
compose de dev solo publica el 8069, así que el upgrade HTTP muere en un worker normal.

**Consecuencia real (no cosmética)**: todo lo que dependa del bus (`bus.bus`) no llega al cliente —
notificaciones de Discuss, indicadores en vivo, cualquier UI que hayas diseñado sobre `bus_service`.
Si tu módulo lo necesita, publica también el 8072 y enruta `/websocket` con un proxy; si no, asume que
el bus no existe en este entorno y no construyas encima.

---

### El navegador no conecta a `http://localhost:18069`

**Causa posibles**:
1. El contenedor Odoo no ha arrancado → `docker compose ps`, `docker compose logs odoo`
2. Odoo está arrancando y tarda (primera vez instala DB, puede tardar 1-2 min)
3. Puerto del slot incorrecto → revisar `.env` `ODOO_PORT`

**Solución**: esperar, leer logs, verificar puerto.

---

### Página "Internal Server Error" tras restaurar dump

**Causa frecuente**: El dump tiene una versión de Odoo distinta a la imagen.

**Síntomas en logs**: `psycopg2.errors.UndefinedColumn`, módulos que no se pueden cargar, errores de ORM.

**Solución**: Cambiar `ODOO_VERSION` en `.env` a la versión del dump y `docker compose up -d --force-recreate odoo`. O pedir al cliente un dump de la versión que tengas.

---

### Odoo arranca pero se queda en "Database not initialized"

**Causa**: Instaló una DB demo al primer arranque y ahora no encuentra la DB del cliente.

**Solución**: ir a `http://localhost:18069/web/database/manager`, seleccionar la DB restaurada. O forzar `-d <db_name>` en el compose.

---

## Restore de dumps

### `pg_restore: error: could not open input file`

**Causa**: El path pasado a `restore.sh` no existe o no es legible.

**Solución**: usar ruta absoluta, verificar con `ls -la`.

---

### `ERROR: database "xxx" is being accessed by other users`

**Causa**: Odoo tiene conexiones abiertas.

**Solución**: El script de restore ya hace `stop odoo` antes, pero si hay conexiones de `psql` manuales del usuario, cerrarlas. También `DROP DATABASE ... WITH (FORCE)` (Postgres 13+) o `pg_terminate_backend` manual.

---

### Filestore vacío tras restore

**Causa**: El ZIP no traía carpeta `filestore/` o estaba en ruta distinta.

**Solución**: Inspeccionar el ZIP (`unzip -l client.zip`). Si el filestore viene por separado, copiarlo manualmente:
```bash
docker cp ./filestore ${PROJECT}_odoo:/var/lib/odoo/filestore/${DB_NAME}
```

---

### Restore tarda mucho

Normal para dumps grandes (>500 MB). Señales a observar:
- `docker stats` → la CPU del contenedor `db` estará al 100%
- Tamaño del volumen creciendo (`docker volume inspect ${PROJECT}_pgdata`)

Si **no** pasa nada en logs durante 5 min seguidos: probablemente está bloqueado. Cancelar con Ctrl+C, verificar logs del `db`, puede ser un dead-lock de índices.

Tip: usar dumps en formato `.dump` (pg_restore custom) en vez de `.sql` para proyectos grandes. Paraleliza con `-j N`.

---

## Debugpy

### VSCode/PyCharm no conecta al `localhost:3000`

**Causa**: Debugpy no está escuchando en el contenedor.

**Verificar**:
```bash
docker compose exec odoo netstat -tln | grep 3000
```

Si no aparece, debugpy no está iniciado. La imagen `indaws/odoo` tiene que traerlo; depende del tag. Verificar docs de la imagen.

**Alternativa manual**:
```bash
docker compose exec odoo bash
python3 -m debugpy --listen 0.0.0.0:3000 --wait-for-client /usr/bin/odoo
```

---

### Debugpy attacha pero los breakpoints no paran

**Causa frecuente**: `workers > 0` en el `odoo.conf` → el worker forkeado no tiene debugpy attached, solo el master.

**Solución**: `workers = 0` (es el default de la skill). Si lo cambiaste, volver atrás.

---

## Volúmenes y datos

### "Quiero borrar la DB pero no el filestore" (o viceversa)

La skill no tiene ese comando; `reset.sh` borra ambos. Manual:
```bash
# Borrar solo DB, mantener filestore
docker compose down
docker volume rm ${PROJECT}_pgdata
docker compose up -d

# Borrar solo filestore
docker compose down
docker volume rm ${PROJECT}_filestore
docker compose up -d
```

---

### Permisos raros en `./addons/custom` tras editar desde IDE

**Causa**: UID host (1000 típico) ≠ UID contenedor (101 típico).

**Solución**:
```bash
sudo chown -R $USER:$USER ./addons/custom
chmod -R g+w ./addons/custom
```

---

## Listado y estado

### `scripts/list.sh` muestra un proyecto pero la carpeta no existe

**Causa**: Se borró la carpeta manualmente sin limpiar el registro.

**Solución**:
```bash
scripts/list.sh --cleanup       # detecta y ofrece borrar registros huérfanos
```

O editar el `slots.json` del registro a mano (ver `references/slot-allocation.md` para la ruta).

---

### Dos proyectos aparecen con el mismo slot

Imposible por diseño: `new.sh` valida contra el registro. Si pasa, es que alguien editó `slots.json` a mano y rompió la invariante.

**Solución**: editar `slots.json`, asignar slots distintos, regenerar `.env` de uno de los dos con los nuevos puertos, `docker compose up -d --force-recreate`.

---

## Red y proyecto compose

### `up` falla: `all predefined address pools have been fully subnetted`

**Causa**: Docker asigna a cada red una /24 de sus pools por defecto; con muchos proyectos (cada uno con su `_net`) se agotan.

**Solución**: `new.sh` ya fija una subred /24 determinista por slot en el compose:
```yaml
networks:
  default:
    name: ${PROJECT_NAME}_net
    ipam:
      config:
        - subnet: 10.199.<slot>.0/24
```
Para un proyecto **generado antes del fix**, añade ese bloque `ipam` a la red y `up` de nuevo. `docker network prune -f` libera redes sin contenedores, pero no amplía los pools.

---

### `up` recrea/borra contenedores de OTRO proyecto (`docker-odoo-1`, etc.)

**Causa**: sin `COMPOSE_PROJECT_NAME`, docker compose nombra el proyecto por el **basename de la carpeta**. Si es genérico (`.docker`, `docker`, `build`), colisiona con otro compose homónimo y compose reconcilia (recrea/absorbe) sus contenedores — aunque los `container_name` sean únicos.

**Solución**: `new.sh` fija `COMPOSE_PROJECT_NAME=<proyecto en minúsculas>` en el `.env`. Proyectos pre-fix: añade esa línea al `.env` y vuelve a `up`. Verifica con `docker compose config --format json | grep name`.

---

### `Port 8069 is in use` al correr tests/one-shot en un contenedor ya levantado

**Causa**: el server Odoo del contenedor ya ocupa 8069/8072. Un `odoo-bin ... --stop-after-init` en un `exec` aparte intenta reabrirlos.

**Por qué `--no-http` NO lo arregla** (corregido 2026-08-07; esta línea decía "sigue reservando el
puerto gevent" y "en v19", y las dos son falsas). Con `--test-enable`, el daemon HTTP arranca
**aunque pases `--no-http`**:

```python
# odoo18/odoo/service/server.py:588-591  ·  odoo19/odoo/service/server.py:653-655
# (remedidas 2026-08-17; este comentario decia :586-588 / :649-651)
# OJO: la linea de v18 NO es esta. v18 es:  test_mode = config['test_enable'] or config['test_file']
#                                          if test_mode or (config['http_enable'] and not stop):
if config['test_enable'] or (config['http_enable'] and not stop):   # <- v19
    # some tests need the http daemon to be available...
    self.http_spawn()
```

Es el propio comentario de Odoo el que lo explica: los tests necesitan el daemon. Y el código es
**idéntico en v18 y v19 salvo por una condición**, así que no es un cambio de versión… con un matiz
que sí importa si lanzas tests:

- v18: `test_mode = config['test_enable'] or config['test_file']` y luego
  `if test_mode or (config['http_enable'] and not stop)` (`odoo/service/server.py:588-591`, `ThreadedServer.start`, v18).
- v19: `if config['test_enable'] or (config['http_enable'] and not stop)`
  (`odoo/service/server.py:653-655`, `ThreadedServer.start`, v19) — **`test_file` ya no se consulta aquí**.

**Y `--test-file` NO ha desaparecido en v19**: sigue siendo opción declarada
(`odoo/tools/config.py:283`, `configmanager._build_cli`, v19) y v19 incluso la valida (`odoo/tools/config.py:707-709`, `configmanager`). Lo que cambió es que
`server.py` dejó de mirarla para decidir si levanta el daemon HTTP. Consecuencia práctica: en v19 un
`--test-file` **sin** `--test-enable` se queda **sin daemon HTTP**, y en v18 lo tenía. Si un test que
pasaba en v18 falla en v19 por no poder hablar con el puerto, es esto.

El puerto **gevent**, en cambio, sí obedece a `--no-http`: `long_polling_spawn()` está dentro del
`if config['http_enable']` de `PreforkServer.process_spawn` (`odoo18:957`, `odoo19:1038`). O sea, lo
que ocupa el 8069 es el HTTP forzado por los tests, no gevent.

**Solución**: puertos alternativos en el one-shot:
```bash
docker exec -u odoo <proj>_odoo /home/odoo/odoo/odoo-bin -c /etc/odoo/odoo.conf \
  -d <db> --http-port=8199 --gevent-port=8299 \
  -u <modulo> --test-enable --test-tags "/<modulo>" --stop-after-init
```

---

### Módulos `auto_install` se instalan solos y cambian el comportamiento

**Causa**: montar un repo de addons entero (p.ej. todo `indawsodoo/{MAJOR}`) mete en el `addons_path` módulos `auto_install` que, al cumplirse sus deps, se instalan sin pedirlo y alteran flujos (visto: un bridge Veri*Factu forzando envío síncrono a AEAT en `action_post`).

**Solución**: para testear un set concreto, monta **solo** sus addons — quita del compose los binds EE/OCA/Indaws que no necesites — y recrea la BD (`DROP DATABASE` + `-i`).
