# Mecanica del MigrationManager

Fuente de verdad: `$ODOO_SRC/odoo18/odoo/modules/migration.py` y
`$ODOO_SRC/odoo18/odoo/modules/loading.py`. Todas las citas fichero:linea son v18
salvo que se indique.

## 1. Estructura en disco

```
<modulo>/
`-- migrations/
    |-- 18.0.1.1.0/
    |   |-- pre-migrate.py        # cualquier *.py que empiece por "pre-"
    |   |-- post-rename.py        # cualquier *.py que empiece por "post-"
    |   `-- end-cleanup.py        # cualquier *.py que empiece por "end-"
    |-- 0.0.0/
    |   `-- end-invariants.py     # corre en CADA cambio de version
    `-- foo.py                    # ignorado (no esta en carpeta de version)
```

- La carpeta debe llamarse como una version valida (regex `VERSION_RE`,
  migration.py:23-47). Carpetas no-version (`README.txt`, `foo.py` suelto) se
  ignoran.
- El nombre del fichero solo importa por su PREFIJO (`pre-`, `post-`, `end-`):
  el matching es `os.path.basename(f).startswith(f"{stage}-")` (migration.py:194).
  El resto del nombre es libre; multiples ficheros por fase se ordenan por
  basename (migration.py:189-197).
- Tambien existe la carpeta hermana `upgrades/` y, para scripts servidos por core,
  `odoo/upgrade/<modulo>/` (migration.py:138-147). En custom normal solo usas
  `migrations/`.

## 2. Firma obligatoria

```python
def migrate(cr, version):
    ...
```

- El fichero DEBE definir una funcion `migrate`. Si falta:
  - v18/v19: se LEVANTA `AttributeError` y aborta (migration.py:240-245 v18).
  - v17: solo se loggea un error y continua (v17 migration.py:236-237).
- La firma se valida en v18/v19 contra `VALID_MIGRATE_PARAMS` = producto de
  `('cr','_cr') x ('version','_version')` y los params deben ser posicionales
  (migration.py:225-256 v18). Una firma distinta LEVANTA `TypeError`. En v17 NO
  hay esa validacion.
- Que recibes:
  - `cr`: el **cursor** de la transaccion del upgrade. NO es `env`.
  - `version`: el `installed_version` (la version ANTERIOR, la que habia en BD).
    Ojo: es la version vieja, no la del manifest. Se pasa
    `mod.migrate(cr, installed_version)` (migration.py:259).

### Construir env (solo en post/end)
```python
from odoo import api, SUPERUSER_ID
env = api.Environment(cr, SUPERUSER_ID, {})
```
Patron usado por el propio core (loading.py:430, registry.py:312).

## 3. Las tres fases y CUANDO corre cada una

`load_module_graph` (loading.py) procesa cada modulo del grafo en orden de
dependencias. Por modulo, dentro del bucle:

| Fase | Linea (loading.py v18) | Momento exacto |
|------|------------------------|----------------|
| `pre` | :182 | Tras `setup_models`, ANTES de `load_openerp_module`, ANTES de `init_models` (:206) y ANTES de cargar data (:228). El schema ORM del modulo aun NO se ha aplicado. |
| `post` | :233 | DESPUES de `init_models` (:206, aplica schema) y DESPUES de `load_data` kind='data' (:228). Columnas y registros nuevos del modulo YA existen. |
| `end` | :523 (STEP 3.5) | FUERA del bucle de carga, cuando TODOS los modulos del grafo ya estan actualizados y su data cargada (:520-523). |

**Confirmacion end-migrate vs post (la pregunta clave):**
- `post` corre DENTRO del bucle por-modulo, inmediatamente tras cargar la data de
  ESE modulo: `migrations.migrate_module(package, 'post')` en
  **loading.py:233**, justo despues de `load_data(... kind='data' ...)` en :228.
- `end` corre en un segundo barrido, DESPUES de que el bucle completo termine,
  bajo el comentario `# STEP 3.5: execute migration end-scripts`:
  `migrations.migrate_module(package, 'end')` en **loading.py:523** (v18).
  En v19 el mismo STEP 3.5 esta en **loading.py:501**.
- Consecuencia practica: en `post` de modulo A, el modulo B (que depende de A)
  todavia NO se ha cargado. En `end` de A, B ya esta cargado. Para cruzar datos
  entre modulos => `end`.

## 4. Naming de versiones y cuando se ejecuta

`convert_version` (migration.py:161-166):
- `"0.0.0"` -> se queda `"0.0.0"` (caso especial).
- Version con >2 puntos (`18.0.1.1.0`) -> ya incluye el major del server, se usa
  tal cual.
- Version con <=2 puntos (`1.1`) -> se le antepone el major del server:
  `"18.0.1.1"`. Es decir, una carpeta `1.1` se interpreta como `18.0.1.1` en un
  server 18.

`compare(version)` (migration.py:203-216) decide si la carpeta corre:
- Regla general: `installed_version < full_version <= current_version`, donde
  `current_version` = version del manifest. O sea: la carpeta corre si su version
  es mayor que la instalada y menor-o-igual que la del manifest.
- **Carpeta "majorless"** (sin major, p.ej. `1.1`): se comparan SOLO las partes de
  modulo (`[2:]`), para no re-ejecutar scripts viejos al subir de major
  (migration.py:209-214). Un `2.0` en un server 9 no se re-ejecuta al pasar a server 10.
- **`0.0.0`**: corre siempre que `installed_version < current_version`
  (migration.py:203-204), o sea en cualquier subida de version del manifest.

**Implicacion:** si no subes `version` en `__manifest__.py`, NINGUN script corre
(ni `0.0.0`), porque `installed_version < current_version` es falso.

## 5. Orden de ejecucion de carpetas dentro de una fase

`_get_migration_versions` (migration.py:168-182) ordena las carpetas por version
parseada. `0.0.0` se reordena:
- en `pre`: se inserta el PRIMERO.
- en `post`/`end`: se anade el ULTIMO.

Asi los invariantes `0.0.0` corren antes que todo en pre y despues de todo en post/end.

## 6. Que NO hace el mecanismo

- NO te recarga la data `noupdate="1"` en un `-u` (eso es politica de `load_data`).
- NO hace rollback selectivo: todo corre en la transaccion del upgrade. Un
  `cr.commit()` manual rompe esa atomicidad; no lo hagas.
- NO te da `env` gratis; lo construyes tu desde `cr`.
