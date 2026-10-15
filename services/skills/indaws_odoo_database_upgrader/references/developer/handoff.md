# Entregar el workdir a otro dev (prueba de reproducibilidad)

El workdir es la **entrega completa** de la migración. Otro dev clona, consigue
el dump aparte, y reproduce el mismo estado.

> **Convención (v2.4.0)**: cada cliente tiene su repo `odoo-migkit-<cliente>`
> en `indawsodoo`. La entrega es **la URL del repo + la ubicación del dump**.
> Ver `references/kit-setup.md` §"Convención repo del workdir".
>
> Clonado: `indaws-migrate workdir-repo clone --remote URL --dest $PROJECTS_ROOT/<Cliente>`

## Qué viaja con el workdir (git)

```
migration_kit_<cliente>/
├── migration.yaml          ← fuente de verdad de la migración
├── sql/                    ← scripts custom del cliente
├── python/                 ← scripts custom del cliente
├── docs/                   ← estudio, decisiones funcionales
├── playbook.md             ← narrativa viva
├── gates.json              ← acks go-live
├── README.md               ← peculiaridades cliente
└── .gitignore
```

## Qué NO viaja (entrega aparte)

| Cosa | Cómo se la haces llegar |
|---|---|
| `original.dump` (dump post Upgrade Service) | NAS interno / S3 / SCP / Drive / Wormhole |
| `filestore/` (adjuntos clientes) | Idem dump |
| Credenciales (`logs/upgrade_credentials.json`) | Mensaje privado (Slack, password manager) |
| `audit.sqlite`, `iterations/`, `logs/` | NO se entregan — los regenera el runner |

## Comandos exactos para el dev receptor

```bash
# 1. Instalar motor (una vez por máquina)
export INDAWS_MIGRATION_KIT_PATH=$PROJECTS_ROOT/indaws_migration_kit   # o donde prefieras
git clone git@github.com:indawsodoo/indaws-migration-kit.git $INDAWS_MIGRATION_KIT_PATH
cd $INDAWS_MIGRATION_KIT_PATH && python3 -m venv venv && venv/bin/pip install -e .

# 2. Registrar la ruta en el contexto global de tu agente (para que la skill la encuentre)
#    Añade a ~/.config/opencode/AGENTS.md o ~/.claude/CLAUDE.md la sección:
#      ## Indaws Migration Kit
#      - INDAWS_MIGRATION_KIT_PATH: $PROJECTS_ROOT/indaws_migration_kit
#      - CLI: $PROJECTS_ROOT/indaws_migration_kit/venv/bin/indaws-migrate

# 3. Alias para no escribir la ruta completa cada vez
alias indaws-migrate="$INDAWS_MIGRATION_KIT_PATH/venv/bin/indaws-migrate"

# 4. Clonar el workdir del cliente
git clone <repo-workdir-cliente>
cd migration_kit_<cliente>

# 5. Conseguir el dump aparte y exportar la ruta
export DUMP_PATH=/ruta/a/donde/tengo/el/dump

# 6. Levantar entorno + restaurar
docker compose up -d db
docker exec -i <client>_db psql -U odoo -c "CREATE DATABASE <db> OWNER odoo;"
docker exec -i <client>_db pg_restore -U odoo -d <db> < $DUMP_PATH/original.dump

# 7. Ejecutar el plan
indaws-migrate inspect --plan migration.yaml      # sanidad
indaws-migrate run --plan migration.yaml --apply  # aplica
```

> El paso 1+2 puede saltárselo: la skill detecta la falta del motor en la
> apertura de sesión y le ofrecerá clonar e instalar en ese momento. Ver
> `../kit-setup.md`.

## Test de reproducibilidad

El dev receptor debería poder:
1. Ejecutar el plan en < 1h (incluyendo restore)
2. NO necesitar preguntarte nada no documentado (más de 2-3 preguntas = playbook incompleto)
3. Obtener una BD funcionalmente idéntica a la tuya (`indaws-migrate compare --reference <tu_dump_final>` sin diffs significativos)

Si fallan los 3, hay deuda en el yaml/playbook/sql que arreglar.
