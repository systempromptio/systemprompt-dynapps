# Restauracion de dump y estrategia de backup/dumps

## Dónde viven los dumps

**Fuera del workdir.** El workdir es versionable; los dumps no.

Tres opciones aceptadas, todas via `--dump <path>`:

| Almacenamiento | Cuándo |
|---|---|
| NAS / S3 / Drive interno | Recomendado. Cliente con dump > 1GB, trabajo en equipo. |
| Carpeta hermana (`../dumps/<cliente>/`) | Dev solo, prototipado. |
| `backup/` dentro del workdir | Solo si el dump es pequeño y NO se entrega. Gitignored por defecto. |

Convención en docs: usaremos `$DUMP_PATH` / `<dump_path>` como placeholder.

## Estructura de carpetas de trabajo

```
<workdir>/                    # versionable (git)
+-- migration.yaml
+-- playbook.md
+-- gates.json
+-- docs/
+-- sql/                      # SQL fixes acumulados (NNN_*.sql)
+-- python/                   # scripts custom (RunScript)
+-- logs/                     # gitignored
|   +-- upgrade_credentials.json
|   +-- iter_001/submission.json
+-- iterations/               # gitignored
+-- audit.sqlite              # gitignored

$DUMP_PATH                    # fuera del workdir (NAS/S3/path local)
  original.dump               # inmutable, input del kit
  to_upload.dump              # temporal, regenera cada iteración
```

## Scripts

```bash
# Detectar tipo de dump (single_custom | single_plain | cluster)
python3 <SKILL_DIR>/scripts/restore_dump.py detect /ruta/al/dump

# Restaurar (detecta tipo automaticamente)
python3 <SKILL_DIR>/scripts/restore_dump.py restore <db_name> /ruta/al/dump

# Verificar restauracion (modulos, usuarios, BDs extra)
python3 <SKILL_DIR>/scripts/restore_dump.py verify <db_name>

# Crear backup inmutable (una sola vez)
python3 <SKILL_DIR>/scripts/restore_dump.py backup <db_name>

# Generar dump para upload (borra anteriores)
python3 <SKILL_DIR>/scripts/restore_dump.py upload_dump <db_name>

# Reset BD desde backup (para nueva iteracion)
python3 <SKILL_DIR>/scripts/restore_dump.py reset <db_name>
```

## Tipos de dump

| Tipo | Creado con | Como restaurar |
|------|-----------|----------------|
| `single_custom` | `pg_dump -Fc` | `pg_restore -d <db>` |
| `single_plain`  | `pg_dump`     | `psql -d <db> < dump` |
| `cluster`       | `pg_dumpall`  | `psql -d postgres < dump` |

## Reglas clave

- `$DUMP_PATH/original.dump` se crea UNA SOLA VEZ y nunca se modifica
- `$DUMP_PATH/to_upload.dump` se borra y regenera al inicio de cada iteracion
- Los dumps viven FUERA del workdir (no se versionan)
- Siempre usar `pg_dump -Fc` (formato custom binario) — ver upgrade-api.md
- El Upgrade Service devuelve un ZIP (dump.sql + filestore parcial) — ver upgrade-api.md

## Bootstrap del entorno Docker (delegación)

Si el cliente NO tiene `docker-compose.yml` previo, delega a la skill
hermana:

```python
Agent(model="sonnet", description="Bootstrap docker env",
      prompt=<ver agent-tree.md §Delegaciones — common-skills:docker-deployment>)
```

Inputs: `cliente`, `to_version`, `DUMP_PATH/original.dump`. Devuelve el
path al compose + nombres de contenedores `odoo` y `db`. Tras esto, los
comandos `pg_restore`/`psql` de arriba ya pueden ejecutarse contra el
contenedor.
