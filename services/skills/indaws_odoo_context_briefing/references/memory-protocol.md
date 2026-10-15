# Memory Protocol

Lifecycle, atomic writes y recovery de `.ai-memory/inventory/`.

## Ubicacion

Por defecto: **cwd del usuario** (la carpeta desde la que se invoca la skill / desde la que se abrio la terminal). NO se sube al git toplevel automaticamente.

```bash
# Default
PROJECT_ROOT="$PWD"
INVENTORY_DIR="$PROJECT_ROOT/.ai-memory/inventory"
```

El usuario puede sobreescribir explicitamente:

| Mecanismo | Prioridad |
|-----------|-----------|
| Flag `--inventory-dir <path>` en cualquier script | 1 (maxima) |
| Variable de entorno `ODOO_CONTEXT_INVENTORY_DIR` | 2 |
| Argumento `inventory_dir: <path>` al invocar la skill | 3 |
| **`$PWD/.ai-memory/inventory/`** (default) | 4 |

Si el usuario indica algo como "usa la raiz del repo" o "usa /path/X/.ai-memory", obedecer su instruccion. En caso contrario, usar `$PWD`.

NUNCA escribir en otra ruta sin instruccion explicita. NUNCA tocar otros subdirs de `.ai-memory/` (los maneja `memory-check`).

### Lectura (read_inventory.py)

Para LECTURA, `read_inventory.py` busca el inventario en el siguiente orden:
1. `--inventory-dir` si se especifica.
2. `$PWD/.ai-memory/inventory/` si existe.
3. Subir hacia parents buscando `.ai-memory/inventory/` (auto-detect — solo lectura, no escritura).

Esto permite consultar el inventario desde subdirectorios del proyecto sin tener que estar en la raiz.

## Lifecycle

### Primera ejecucion (no existe `.ai-memory/inventory/`)

1. Crear `.ai-memory/inventory/` con `mkdir -p`.
2. Crear `.ai-memory/` si tampoco existe + bootstrap minimo (delegar a `memory-check sync` si esta disponible).
3. Forzar modo `full`.
4. Despues del scan: escribir los 4 ficheros + `deep/` + opcional `.config.yaml`.
5. Crear `.gitignore` local en `.ai-memory/inventory/` ignorando `.tmp/`:
   ```
   .tmp/
   *.lock
   ```

### Ejecuciones siguientes

1. Leer `SNAPSHOT.md` para hash global y fecha previa.
2. Leer `modules.yaml` para hashes individuales.
3. Ejecutar el modo solicitado.
4. Calcular delta (modulos nuevos / actualizados / desinstalados).
5. Atomic write de los 4 ficheros + deep-dives nuevos.
6. Limpiar `deep/<name>.md` de modulos desinstalados.

## Atomic writes

Toda escritura pasa por `update_memory.py`. Protocolo:

```
1. mkdir -p .ai-memory/inventory/.tmp
2. Escribir cada fichero a .tmp/<name> (modules.yaml.tmp, etc.)
3. fsync explicito tras escribir
4. Acquire lock: .ai-memory/inventory/.lock (flock o file existence check)
5. mv atomico de cada .tmp -> destino final (mismo filesystem garantizado)
6. Release lock
7. rm -rf .tmp
```

Si el proceso muere a mitad:
- `.tmp/` tiene basura -> se limpia en la siguiente ejecucion
- ficheros finales nunca se quedan a medias (mv es atomico)

## Hash global

Calculado al final de cada update:

```python
import hashlib, yaml, json
content = {
    "modules": yaml.safe_load(open("modules.yaml")),
    "studio": yaml.safe_load(open("studio.yaml")),
    "collisions": yaml.safe_load(open("collisions.yaml")),
}
canonical = json.dumps(content, sort_keys=True, separators=(",", ":"))
global_hash = hashlib.sha256(canonical.encode()).hexdigest()
```

Se persiste en `SNAPSHOT.md` como `Global hash: sha256:...`.

Sirve para:
- Detectar drift cuando otra herramienta toca los ficheros (memory-check lo valida).
- Permitir a otras skills confiar en que el inventario no ha cambiado desde su ultimo read.

## Delta calculation

Antes de sobrescribir `modules.yaml`, comparar la nueva version contra la anterior:

```python
prev = yaml.safe_load(open("modules.yaml"))
prev_by_name = {m["name"]: m for m in prev}
new_by_name = {m["name"]: m for m in nuevo}

added = set(new_by_name) - set(prev_by_name)
removed = set(prev_by_name) - set(new_by_name)
upgraded = [
    name for name in (set(new_by_name) & set(prev_by_name))
    if new_by_name[name]["version"] != prev_by_name[name]["version"]
       or new_by_name[name]["hash"] != prev_by_name[name]["hash"]
]
```

El delta se renderiza en `SNAPSHOT.md` seccion "Delta vs scan anterior".

## Recovery

Si `update_memory.py` detecta que `modules.yaml` esta corrupto (YAML invalido), no falla — backup + re-baseline:

```
1. mv modules.yaml modules.yaml.broken-<timestamp>
2. log warning al usuario
3. forzar modo full
4. regenerar
```

Mismo trato para `studio.yaml` y `collisions.yaml`. `SNAPSHOT.md` corrupto: regenerar sin backup (es derivado).

## Concurrency

Si dos ejecuciones simultaneas de la skill:
- La segunda detecta `.lock` y aborta con mensaje al usuario ("inventory en uso, espera").
- Lock con timeout de 5 min (si esta huerfano se rompe automaticamente).

## Validacion post-write

Tras cada update, llamar a `memory-check sync` (si esta disponible como skill horizontal). Reporta drift/issues. Resultado se loguea en SNAPSHOT.md como `Last validation: PASS|WARN|FAIL`.

## Compatibilidad con `memory-check`

El skill `memory-check` mantiene `.ai-memory/SNAPSHOT.md` global con un hash del inventory de modulos del proyecto. `odoo-context-briefing` debe:

1. Tras update, escribir su propio `.ai-memory/inventory/SNAPSHOT.md`
2. NO tocar `.ai-memory/SNAPSHOT.md` global
3. Llamar a `memory-check sync` que reconcilia el global con el sub-inventory

Si `memory-check` no esta disponible: log un warning pero continuar (no es bloqueante).
