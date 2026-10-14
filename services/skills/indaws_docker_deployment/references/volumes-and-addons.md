# Volumes & Addons

Cómo se montan los addons y los volúmenes persistentes, y por qué.

## Volúmenes persistentes (named)

Dos volúmenes nombrados, prefijados por proyecto:

| Volumen | Ruta en contenedor | Contenido |
|---|---|---|
| `${PROJECT}_pgdata` | `/var/lib/postgresql/data` | Datos de Postgres (DB al completo) |
| `${PROJECT}_filestore` | `/var/lib/odoo` | Filestore de Odoo (attachments binarios, sessions, etc.) |

**Qué sobrevive a qué**:

| Acción | pgdata | filestore |
|---|---|---|
| `docker compose restart` | ✓ | ✓ |
| `down.sh` (= `docker compose down`) | ✓ | ✓ |
| `reset.sh` (= `down -v`) | ✗ | ✗ |
| Regenerar compose (`new.sh` sobre existente) | ⚠ ver abajo | ⚠ ver abajo |

**⚠ Regeneración**: si se invoca `new.sh` sobre un proyecto que ya existe, el script aborta por defecto. Con `--force`, reescribe compose/.env/conf pero **no toca los volúmenes**, así que los datos persisten. Para limpiar, hacer `reset.sh` primero.

## Binds (montajes desde host)

### Config (read-write desde el host)

```yaml
- ./config/odoo.conf:/etc/odoo/base/odoo.conf:ro
```

El `odoo.conf` se edita en el host, montado como `ro` dentro del contenedor.
El destino es `/etc/odoo/base/odoo.conf` (no `/etc/odoo/odoo.conf`) porque
el entrypoint de `indaws/odoo` hace `cp /etc/odoo/base/odoo.conf
/etc/odoo/odoo.conf` al arrancar; si bindeamos directo al destino, el `cp`
falla con "Read-only file system". Para aplicar cambios: `docker compose
restart odoo`.

### Addons del cliente (read-write)

```yaml
- ./addons/custom:/addons/custom
```

Aquí va el código del cliente. **Read-write** porque puedes querer que Odoo escriba (p.ej. `__pycache__`), aunque lo normal es que el editor lo toque desde el host.

La carpeta se crea vacía al generar el proyecto. El usuario copia/clona ahí lo que toque.

### Sources oficiales (read-only)

Las 4 rutas de fuentes externas (Odoo CE, EE, OCA, Indaws) se resuelven por proyecto según un **config global** editable por el usuario:

```
~/.config/docker-deployment/defaults.env
```

Que contiene plantillas con placeholder `{MAJOR}`:

```bash
ODOO_SRC_TEMPLATE=$ODOO_SRC/odoo{MAJOR}/addons
EE_SRC_TEMPLATE=$ODOO_ENTERPRISE_SRC/{MAJOR}
OCA_SRC_TEMPLATE=$OCA_SRC/v{MAJOR}
INDAWS_SRC_TEMPLATE=$PROJECTS_ROOT/indawsodoo/{MAJOR}
```

`{MAJOR}` se sustituye con la mayor Odoo del proyecto (de `18.0` → `18`). Reglas:

- Variable vacía o comentada → montaje omitido.
- Ruta resuelta no existe en el host → montaje omitido (warning, no falla).
- `new.sh` hornea la ruta final ya resuelta en el `docker-compose.yml` generado, así que cambiar el config global después no afecta proyectos existentes (regenera con `--force` si lo necesitas).

El fichero se crea automáticamente la primera vez que corres `new.sh`, partiendo de `templates/defaults.env.tpl`. Trae unos valores por defecto de arranque; con otro layout se edita ese fichero una vez y listo. En cada actualización de la skill se añaden al fichero las claves nuevas sin tocar las que ya tengas.

**Read-only** adrede: si editas un módulo de OCA o Enterprise, lo haces intencionadamente. Si hay que editarlos (parche local), se cambia el montaje a RW temporalmente en el compose del proyecto.

**Override por proyecto**: `new.sh --no-ee` fuerza EE off aunque el path exista. Para OCA/Indaws, si no los quieres en un proyecto concreto: deja el path global vacío temporalmente y genera, o edita el `docker-compose.yml` generado.

## Orden del addons_path

Orden dentro del contenedor:

```
/addons/custom        # prioridad máxima
/addons/indaws
/addons/oca
/addons/enterprise
/usr/local/lib/python3.11/dist-packages/odoo/addons  # CE built-in, mínima
```

Odoo toma el **primero que encuentra** con el nombre del módulo. Detalle en
`references/odoo-conf.md`.

**Nota**: la imagen `indaws/odoo` regenera `addons_path` en su entrypoint
ejecutando `find_files('/addons', '__manifest__.py')`. Por eso los binds
deben ir a `/addons/*` y no a `/home/odoo/addons/*` — el escaneo solo
mira `/addons`. Odoo CE base no se montea: viene built-in en
`/usr/local/lib/python3.11/dist-packages/odoo/addons` dentro de la imagen.

## Permisos de los binds

El contenedor `indaws/odoo` corre con usuario `odoo` (UID 101 típicamente, puede variar por versión).

Si escribes desde el host en `./addons/custom` (UID 1000, típico), puede haber mismatch con el UID de dentro. Síntomas: Odoo no puede borrar `__pycache__` al reiniciar, o no puede escribir logs en `/var/lib/odoo/sessions`.

Soluciones (si aparece el problema):
1. `chmod -R g+w ./addons/custom` — permisos laxos
2. Añadir `user: "1000:1000"` al servicio `odoo` en el compose — fuerza UID host
3. En la imagen `indaws/odoo`, ver si respeta `PUID`/`PGID` por env (depende de cómo esté construida)

Esta skill no fuerza UID. Si aparece el problema, se resuelve caso por caso.

## ¿Y los dumps?

`./dumps/` es **solo un bind conveniencia** para que el usuario deje ahí los archivos del cliente. No se monta en el contenedor Odoo (el contenedor no necesita ver los dumps). `restore.sh` los procesa desde el host.

Está `.gitignored` por defecto — los dumps no van a git.
