# Verificación en Docker (Step 1)

No aprobamos sobre la palabra del dev ni sobre un CI que no controlamos: **levantamos un entorno
limpio, instalamos la rama del PR y ejecutamos ahí**. La mecánica del Docker la hace la skill
`docker-deployment`; el **montaje aislado y su destrucción** están en `review-instance.md`
(worktree del PR, pinneo de los demás repos, instancia efímera, teardown). Aquí va el QUÉ
comprobar una vez levantado.

---

## 1. Qué levantar (delegar a `docker-deployment`)

- **Versión Odoo objetivo** de la tarea (la que fijó el arquitecto en el MD) — no otra.
- **Rama del PR** en los addons (checkout de la rama, no de `main`).
- **Datos**:
  - Si el comportamiento depende de datos del cliente (cálculos sobre registros existentes, config
    concreta, Studio) → **restaurar el dump del cliente**. Probar en vacío puede ocultar el bug real.
  - Si es funcionalidad autónoma → base limpia basta.
- **Con dump del cliente, la BD va NEUTRALIZADA, sin excepción.** `restore.sh` de `docker-deployment`
  lo hace por defecto (crons, servidores de correo, webhooks, integraciones custom, caducidad EE) y
  deja la red en cuarentena hasta que das el visto bueno. **Nunca uses `--no-neutralize` en un
  review**: estás metiendo datos de producción en una instancia que va a ejecutar código nuevo sin
  revisar. Si el dump entró por otra vía, pasa `neutralize.sh <proj> --db <bd>` antes de arrancar.
- Slot determinista por proyecto (varios reviews pueden convivir) — lo gestiona `docker-deployment`.
- **Ojo con montar repos enteros**: los módulos `auto_install` cuyas dependencias se cumplan se
  instalan solos y cambian el comportamiento (gotcha 13 de `docker-deployment`; visto con un bridge
  de Veri*Factu forzando envío síncrono en `action_post`). En un review eso significa que puedes
  estar juzgando un comportamiento que no es el del PR. Para revisar un módulo aislado, monta solo
  su set de addons.

## 2. Instalar / actualizar el módulo (primer gate)

- Instalar (`-i`) si es nuevo, o actualizar (`-u`) si extiende algo ya instalado en el dump.
- **Si no instala/actualiza en limpio → correcciones inmediatas.** Un módulo que no carga no se
  revisa más allá: es bloqueante de entrada (import errors, XML mal, dependencias, constraints).
- Mirar el log de arranque: warnings de vistas, campos y deprecations. **Ojo con qué esperas ver**:
  `attrs`/`states` **no** salen como deprecation, lanzan `ValidationError` desde 17.0 (*"Since 17.0,
  the 'attrs' and 'states' attributes are no longer used"*) y `<tree>` en v18 tampoco avisa: el tipo
  ya no existe. Lo que SÍ es un warning de log en v18 es `kanban-box`
  (`odoo/addons/base/models/ir_ui_view.py:1517`, `View._validate_tag_kanban`: *"'kanban-box' is deprecated, define a 'card' template instead"*). Un log
  limpio de deprecations no significa que la vista esté migrada.

## 3. Dejar listo para tests y prueba manual

- El entorno queda con el módulo cargado para:
  - ejecutar la suite de tests (Step 4, ver `test-verification.md`),
  - y probar el **flujo e2e a mano** o con `odoo-flow-tester` si el PR no lo cubre claro.

## 4. Comando de tests (se ejecuta en el Docker)

```bash
# dentro del contenedor / contra la db del Docker (añadir el filtro de logs habitual)
docker exec <proj>_odoo odoo -d <db_review> -u <modulo> \
  --test-enable --test-tags /<modulo> --stop-after-init \
  --http-port=8199 --gevent-port=8299 --max-cron-threads=0 2>&1 | tail -40
```

> **El comando es `odoo`, NO `odoo-bin`** — pero **ojo con 19.0, que no es lo mismo en las dos**
> (verificado en runtime 2026-08-07 sobre las imágenes reales):
>
> | Imagen | `/usr/local/bin/odoo` | ¿funciona en la imagen desnuda? |
> |---|---|---|
> | `indaws/odoo:18.0` | fichero real ejecutable (176 B) y **no hay** `odoo-bin` | **sí** |
> | `indaws/odoo:19.0` | **symlink a `/home/odoo/odoo/odoo-bin`**, y ese directorio está **vacío** (es el punto de montaje del fuente) | **no**: `sh: 1: odoo: not found` |
>
> O sea: en 19.0 lo que corre **sí se llama `odoo-bin`**, y sin montar el fuente el contenedor no
> falla con un error de Odoo, falla con **`odoo: not found`** por symlink roto. Si ves eso, no es que
> la imagen esté mal: es que falta el montaje del código.
>
> Y los **puertos alternativos van SIEMPRE**, no solo en la vía rápida: con el server del proyecto
> arriba, un segundo proceso peta con *"Port 8069 is in use"* y `--no-http` no basta (en v19 sigue
> reservando el puerto gevent).

Capturar el output (verde/rojo real) para adjuntarlo a la decisión.

## 4.1 Nunca parchear el árbol compartido

La versión anterior de esta guía permitía reusar un contenedor vivo y aplicar los ficheros del PR
con `git checkout pr-<n> -- <ficheros>` sobre el árbol montado. **Eso está prohibido**: ese árbol lo
comparten muchos proyectos del host (**remedido 2026-08-17: 20 proyectos únicos** — `/16` 2, `/17` 3,
`/18` 13, `/19` 7) y suele estar en una rama de trabajo tuya. **Hoy lo están 2 de los 4**: `/17` en
`17-perf-import-phase3` y `/18` en `feat/portal-picking-delivery-state`; `/16` y `/19` están limpios
en `16.0` y `19.0`.

> **Ojo con leer eso como una mejora.** La versión anterior de esta línea decía *"hoy las tres lo
> están"* citando `/19` en `feature/indaws-uom-refactor` — y era verdad al escribirla. Que hoy sean
> dos no debilita la regla: **el estado de las ramas cambia de semana en semana y no lo controlas
> tú**. Míralo antes de cada review con
> `for v in 16 17 18 19; do git -C $P/indawsodoo/$v branch --show-current; done`, y no des por
> estable un major porque lo estuviera la última vez.

O sea que el riesgo no es de un major, es del parque entero. Se monta el PR en
un `git worktree` y una instancia efímera aparte — ver `review-instance.md`.

**Gotcha de puerto:** una instancia viva ocupa 8069/8072. Un segundo proceso `odoo` para tests peta
con *"Port 8069 is in use"* **aunque pongas `--no-http`**. Por eso los puertos alternativos del
comando de §4 son obligatorios, no opcionales.

> **El porqué, desde la fuente (corregido 2026-08-17; antes esta línea decía "en v19 sigue reservando
> el puerto gevent", y el mecanismo no es ese).** Lo que ata el puerto es **`--test-enable`**, no el
> gevent:
>
> ```python
> # odoo19/odoo/service/server.py:653   (v18: :588-589, con el extra de test_file)
> if config['test_enable'] or (config['http_enable'] and not stop):
>     self.http_spawn()          # :629-630 -> ThreadedWSGIServerReloadable(self.interface, self.port, ...)
> ```
>
> `self.port` es **`config['http_port']`** (`:412`), o sea **8069**. El `or` está antes que
> `http_enable`: con `--test-enable` el daemon se levanta **igual**, y `--no-http` no se llega a
> mirar. El puerto gevent es de `GeventServer` (`:767`, `self.port = config['gevent_port']` `:770`),
> que solo se instancia en la vía gevent/multi-worker (`:1622`) — **no en la que usa el comando de
> §4**. Se pasa `--gevent-port` por higiene, pero el que choca es el HTTP.
>
> Consecuencia práctica: **no pierdas el tiempo con `--no-http`**; cambia el puerto. Y es el mismo
> `odoo/service/server.py:653`, `ThreadedServer.run` del que salió el matiz de `--test-file` en `common:docker-deployment` — tercera vez
> que esta línea aparece por una puerta distinta.

## 5. Cierre — tumbar el entorno

Al terminar el review se **destruye** la instancia: contenedores, volúmenes, carpeta, **slot del
registro** y worktree del PR. No basta con `down`: un `down` deja el proyecto registrado y los
volúmenes con los datos del cliente dentro. Secuencia completa en `review-instance.md` §6.

---

## Por qué este paso

El principio del arquitecto era "verde real, no declarado". Esto lo hace literal: el mismo entorno,
la misma versión, los mismos datos que producción (cuando aplica). Un test que pasa aquí es una
prueba; un test que pasa "en la máquina del dev" es una promesa.
