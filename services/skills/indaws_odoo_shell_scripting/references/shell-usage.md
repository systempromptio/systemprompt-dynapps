# Odoo Shell — Uso por Versión

## Lanzar el shell

```bash
# Forma general (v14–v19)
python odoo-bin shell -c /etc/odoo/odoo.conf -d NOMBRE_BD

# Con virtualenv activo (Docker Indaws)
$ODOO_SRC/odoo18/odoo-bin shell -c /etc/odoo/odoo.conf -d NOMBRE_BD

# Sin fichero de conf (parámetros inline)
python odoo-bin shell --db_host=localhost --db_port=5432 --db_user=odoo -d NOMBRE_BD

# Ejecutar script directamente (sin prompt interactivo)
python odoo-bin shell -c /etc/odoo/odoo.conf -d NOMBRE_BD < fix_script.py

# O con pipe más portable
python odoo-bin shell -c /etc/odoo/odoo.conf -d NOMBRE_BD --no-http < fix_script.py
```

## Flag --no-http

Añadir `--no-http` evita levantar el servidor web; recomendado para scripts one-off en
producción para reducir carga. Disponible desde v14.

**Excepción: con `--test-enable` NO sirve de nada.** El daemon HTTP arranca igual, porque el guard
es `if config['test_enable'] or (config['http_enable'] and not stop): self.http_spawn()`
(**`odoo18/odoo/service/server.py:588-591`, `ThreadedServer.start`**, **`odoo19/…:653-655`** — remedidas 2026-08-17; decían
`:586-588`/`:649-651`), y el comentario de Odoo dice por qué:
*"some tests need the http daemon to be available…"*. Idéntico en v18 y v19. Si lo que quieres es
correr un one-shot de tests sin chocar con un server ya levantado, mueve los puertos
(`--http-port` / `--gevent-port`), no desactives http. Verificado 2026-08-07.

## Variables disponibles en el prompt

| Variable | v14–v16 | v17–v19 | Notas |
|----------|---------|---------|-------|
| `env` | Sí | Sí | `api.Environment` actual. Misma línea `local_vars['env'] = env` en v14→v19 (`odoo/cli/shell.py:116`, `Shell.ptpython` @cc0060e8` … `:142 @d1d161033`): **no hay frontera de versión** |
| `self` | Sí (res.users activo) | Sí | El propio usuario de sesión (`local_vars['self'] = env.user`), por eso `self.env` funciona en todas |
| `odoo` | Sí | Sí | Módulo raíz de Odoo |
| `openerp` | Sí (alias) | Sí (alias) | `'openerp': odoo` en **las seis versiones**: v14 `cli/shell.py:106`, `Shell.shell`, v15 `:106`, v16 `:105`, v17 `:106`, v18 `:107`, v19 `:132` (medido 2026-08-17). Desaconsejado por estilo, no por disponibilidad |

El alias `openerp` existe en v14→v19 sin interrupción (líneas arriba); usar `env`/`odoo` por estilo, no porque `openerp` no exista.

## Usuario y contexto

```python
# Usuario por defecto: el del -u flag o admin (uid=1)
# Para operar como admin explícitamente:
env = env(user=env.ref('base.user_admin'))

# Forzar idioma español en operaciones que generan traducciones
env = env(context=dict(env.context, lang='es_ES'))

# Combinar usuario + contexto
env = env(user=env.ref('base.user_admin'), context={'lang': 'es_ES', 'active_test': False})
```

## `active_test` en dominios

Por defecto Odoo filtra `active=False`. Para incluir registros archivados:
```python
records = env['res.partner'].with_context(active_test=False).search([...])
```

## Diferencias relevantes v17–v19

- **v17**: `env.cr` es el cursor de la sesión; savepoints funcionan igual. `env.transaction`
  YA existe en v17 (`odoo/api.py`); en shell sigue valiendo `env.cr.savepoint()`.
- **v18**: Igual que v17 para shell scripting.
- **v19**: Añade flags de shell `--shell-file` (script a ejecutar) y `--shell-interface`
  (intérprete: ipython/ptpython/bpython/python). `odoo-bin shell --shell-file=fix.py`.

## Entornos Docker Indaws

```bash
# Acceder al contenedor y lanzar shell
docker exec -it <container_odoo> bash
python $ODOO_SRC/odoo18/odoo-bin shell -c /etc/odoo/odoo.conf -d NOMBRE_BD --no-http

# Ejecutar script desde host (fichero montado)
docker exec -i <container_odoo> python $ODOO_SRC/odoo18/odoo-bin shell \
  -c /etc/odoo/odoo.conf -d NOMBRE_BD --no-http < /ruta/local/script.py
```
