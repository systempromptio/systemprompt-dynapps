# Input Detection — Dynapps Migrator

## Orden de detección al startup

Ejecutar las siguientes comprobaciones en orden secuencial antes de pedir nada al usuario.

### 1. Parsear el prompt inicial

Extraer del prompt de entrada (regex permisivo):

```
cliente:          primera entidad propia mencionada antes de "BD:" o "Repo:"
db_source:        ruta de archivo (.dump/.sql/.gz/.zip/.tar[.gz]) O DSN postgres://...
                  (alias en el prompt: db, dump, db_connection)
repo:             git@... o https://gitlab/github...
version_destino:  "v18", "v16", "18.0", etc.
infraestructura:  "Combell" | "SH" | "OdooSH" | "on-premise" | "local"
version_origen:   inferida del repo path (v10/, v14/) o de la BD
```

Si algún campo no se puede inferir con certeza, marcarlo como `MISSING`.

### 1.b Resolver `scenario` (determinista)

Con lo parseado, resolver `scenario` ∈ {`onpremise_gitlab`, `odoosh_github`} por
precedencia (detalle en `scenarios.md`): declaración explícita → host del `repo`
(`gitlab.dynapps.be` vs `github.com`) → `.copier-answers.yml → ci_system` si hay buildout
→ keyword `infraestructura` → si sigue ambiguo, preguntar. `scenario` gobierna el track de
código (BLOQUE 3) y el go-live (BLOQUE 5); `infraestructura` se conserva como matiz para
BLOQUE 2 (path del Upgrade Service).

### 1.c Normalizar `db_source` (archivo → BD restaurada)

Si `db_source` es una **ruta de fichero** (caso normal: dump comprimido en local), hay que
descomprimir + restaurar en PostgreSQL local ANTES del BLOQUE 2 (el Upgrade Service parte de
una BD existente). Si es un **DSN** `postgres://…`, saltar este paso.

```bash
detect_db_source() {  # devuelve: dsn | dumpfile
  case "$1" in postgres://*|postgresql://*) echo dsn ;; *) echo dumpfile ;; esac
}
# dumpfile → restaurar en la BD de trabajo <cliente>_v<origen> (idempotente: drop+create)
restore_dump() {  # $1=archivo  $2=dbname
  createdb "$2" -T template0 2>/dev/null || { dropdb "$2"; createdb "$2" -T template0; }
  case "$1" in
    *.zip)              tmp=$(mktemp -d); unzip -o "$1" -d "$tmp"; f=$(find "$tmp" -type f | head -1) ;;
    *.tar.gz|*.tgz)     tmp=$(mktemp -d); tar xzf "$1" -C "$tmp"; f=$(find "$tmp" -type f | head -1) ;;
    *.gz)               f="${1%.gz}"; gunzip -kf "$1" ;;
    *)                  f="$1" ;;
  esac
  case "$f" in
    *.sql)   psql -d "$2" -f "$f" ;;          # plano
    *)       pg_restore -d "$2" --no-owner "$f" ;;   # custom/directory (.dump)
  esac
}
```

El dump restaurado es la BD origen del BLOQUE 2. **Ojo**: sigue siendo de producción → el
gate de neutralización del BLOQUE 2 aplica igual. El path exacto de restore lo puede asumir
`dynapps-local-env-setup`/`dynapps-db-upgrade-runner`; lo de arriba es el fallback del
orquestador cuando el input es un archivo suelto.

### 2. Comprobar prerequisites de máquina

```bash
# Busca el script en las raíces de skills conocidas de cualquier agente.
# $SKILLS_ROOT, si está definida, tiene prioridad.
for _root in "${SKILLS_ROOT:-}" \
             "$HOME/.claude/plugins" \
             "$HOME/.config/opencode/skills" \
             "$HOME/.dyn-skills" \
             "$HOME/.agents/skills"; do
    [ -n "$_root" ] && [ -d "$_root" ] || continue
    SKILL_DIR_BOOTSTRAP=$(find "$_root" -path "*/dynapps-dev-bootstrap/scripts/check-prerequisites.sh" 2>/dev/null | head -1)
    [ -n "$SKILL_DIR_BOOTSTRAP" ] && break
done

if [ -n "${SKILL_DIR_BOOTSTRAP:-}" ]; then
    bash "$SKILL_DIR_BOOTSTRAP" 2>&1
fi
```

Resultado esperado: exit 0 si OK, exit 1 con lista de tools faltantes.

Herramientas mínimas a verificar manualmente si el script no existe:
- `python3`, `pip`, `git`, `psql`, `pg_restore`
- `buildout` o `python-buildout`
- `vault` (Hashicorp)
- `indaws-migrate` (el CLI del migration kit)

### 3. Comprobar buildout del cliente

```bash
BUILDOUT_DIR="$ODOO_SRC/buildouts/${CLIENTE}${VERSION_DESTINO}"
if [ -d "$BUILDOUT_DIR" ]; then
    echo "BUILDOUT_EXISTS: $BUILDOUT_DIR"
else
    echo "BUILDOUT_MISSING: $BUILDOUT_DIR"
fi
```

Variantes de path a probar:
- `$ODOO_SRC/buildouts/<cliente><version>/`
- `$ODOO_SRC/buildouts/<cliente>-<version>/`
- `$ODOO_SRC/buildouts/<cliente>_v<version>/`
- `~/odoo/<cliente>/<version>/`

### 4. Comprobar workdir de migración en curso

```bash
SESSION=$(indaws-migrate playbook-session 2>/dev/null)
if [ -n "$SESSION" ]; then
    echo "SESSION_ACTIVE: $SESSION"
    indaws-migrate modules list 2>/dev/null
fi
```

También buscar directorio por convención:
```bash
ls ~/migrations/migration_kit_${CLIENTE}/ 2>/dev/null || \
ls /tmp/migration_kit_${CLIENTE}/ 2>/dev/null || \
ls ~/odoo-migkit-${CLIENTE}/ 2>/dev/null
```

Si existe workdir → activar **Resume Mode** (ver `resume-mode.md`).

## Qué preguntar al usuario (AskUserQuestion)

Preguntar SOLO los campos marcados como `MISSING` tras el parseo y las comprobaciones. Agrupar todos en una sola pregunta.

### Plantilla de pregunta inicial

```
Para iniciar la migración necesito confirmar algunos datos:

[Si MISSING: cliente]      → Nombre del cliente (ej: tecniman)
[Si MISSING: version_origen] → Versión origen de Odoo (ej: 10, 14, 16)
[Si MISSING: version_destino] → Versión destino (ej: 18)
[Si MISSING: infraestructura] → Infraestructura destino: Combell / OdooSH / On-premise
[Si MISSING: db_source]      → Ruta al dump local (.dump/.sql/.gz/.zip/.tar) o DSN postgres://host:port/dbname
[Si MISSING: repo]           → URL del repo del cliente (GitLab/GitHub)

Responde en formato libre, inferiré los valores del texto.
```

### Tabla de campos y validaciones

| Campo | Tipo | Validación | Default |
|---|---|---|---|
| `cliente` | string | sin espacios, lowercase | — |
| `version_origen` | int | 8, 10, 12, 14, 16, 17 | inferido del repo |
| `version_destino` | int | 16, 17, 18 | 18 |
| `infraestructura` | enum | Combell / OdooSH / On-premise | — |
| `db_source` | ruta \| DSN | archivo existente (.dump/.sql/.gz/.zip/.tar) o postgres:// | — |
| `repo` | URL | git@ o https:// | — |

### Inferencias automáticas (no preguntar si se pueden inferir)

- `version_origen` desde el path del repo: `customers/v10/...` → 10
- `version_origen` desde el DSN o el nombre del dump: `...tecniman_v10`, `tecniman-16.0.dump` → 10/16
- `cliente` desde el nombre del repo o del dump: `tecniman.git`, `tecniman.dump` → tecniman
- `db_source_kind`: empieza por `postgres://`/`postgresql://` → dsn; si no → dumpfile
- `infraestructura` desde palabras clave en el prompt: "Combell", "SH", "local"

## Estado resultante

Tras el startup, el agente tiene un objeto de contexto:

```yaml
migration_context:
  cliente: tecniman
  version_origen: 10
  version_destino: 18
  infraestructura: Combell
  scenario: onpremise_gitlab | odoosh_github
  db_source: ~/dumps/tecniman.dump      # archivo local O DSN postgres://...
  db_source_kind: dumpfile | dsn
  db_workdir: tecniman_v10              # BD restaurada (si db_source era archivo)
  repo: git@gitlab.dynapps.be:customers/v10/tecniman.git
  machine_ready: true | false
  buildout_exists: true | false
  workdir_active: true | false
  workdir_path: ~/migrations/migration_kit_tecniman
  resume_from_block: null | 1 | 2 | 3 | 4 | 5
```

Este contexto se pasa como parámetros a cada skill delegada.
