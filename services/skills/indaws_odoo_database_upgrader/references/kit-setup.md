# Localización del motor `indaws_migration_kit`

La skill `odoo-database-upgrader` necesita el CLI `indaws-migrate` para casi
todo. Este documento define **cómo localizarlo** al abrir una sesión.

---

## Protocolo de descubrimiento (orden de preferencia)

1. **Variable de entorno `INDAWS_MIGRATION_KIT_PATH`** (si está exportada)
2. **Fichero de contexto global del agente** — busca la línea
   `INDAWS_MIGRATION_KIT_PATH: <path>` en la sección "Indaws Migration Kit".
   El script sondea, en este orden: `$AGENT_CONTEXT_FILE` (si está definida),
   `~/.config/opencode/AGENTS.md`, `~/.claude/CLAUDE.md`, `~/.agents/AGENTS.md`.
3. **Default convencional**: `$PROJECTS_ROOT/indaws_migration_kit`

En cualquiera de los 3 casos, el CLI vive en `<KIT_PATH>/venv/bin/indaws-migrate`.

---

## Apertura de sesión — comprobación obligatoria

Antes de invocar el CLI por primera vez en una sesión, ejecutar:

```bash
bash <SKILL_DIR>/scripts/locate_kit.sh
```

Salidas posibles:

| Salida | Interpretación | Acción |
|---|---|---|
| `OK:/opt/.../venv/bin/indaws-migrate` | Localizado, ejecutable | Continuar |
| `NOT_CONFIGURED` | No hay ruta guardada en el contexto del agente ni en env | **[STOP] preguntar al usuario** (ver §"Diálogo cuando no está") |
| `PATH_MISSING:<path>` | Hay ruta guardada, pero no existe en disco | **[STOP] preguntar** (cambió de ruta, o falta clonar) |
| `VENV_MISSING:<path>` | Carpeta sí, venv no | Proponer `python3 -m venv venv && venv/bin/pip install -e .` (no autónomo) |

Si la salida es OK, exportar la ruta:

```bash
export INDAWS_MIGRATE_CLI="$(bash <SKILL_DIR>/scripts/locate_kit.sh | cut -d: -f2-)"
alias indaws-migrate="$INDAWS_MIGRATE_CLI"
```

A partir de aquí, todas las invocaciones `indaws-migrate ...` funcionan.

---

## Diálogo cuando no está configurado

Usar `AskUserQuestion` con esta pregunta:

> El motor `indaws_migration_kit` no está localizado. ¿Cómo procedemos?

Opciones:

| Opción | Significa | Acción |
|---|---|---|
| **Ya lo tengo clonado en otra ruta** | El usuario da el path | Validar que existe + tiene venv funcionando, guardar en el contexto del agente |
| **Clonarlo en la ruta default** | Clonar en `$PROJECTS_ROOT/indaws_migration_kit` | `git clone` + `python3 -m venv venv && venv/bin/pip install -e .`, guardar en el contexto del agente |
| **Clonarlo en otra ruta** | El usuario indica destino | Idem, en el path indicado |

### Tras la elección — guardar en el contexto del agente

Editar el fichero de contexto global del agente en uso (`~/.config/opencode/AGENTS.md`,
`~/.claude/CLAUDE.md`, …), añadir/actualizar sección:

```markdown
## Indaws Migration Kit (motor de migraciones Odoo)
- `INDAWS_MIGRATION_KIT_PATH`: <path resuelto>
- CLI: <path>/venv/bin/indaws-migrate
- Git remote: git@github.com:indawsodoo/indaws-migration-kit.git
```

Así la próxima sesión arranca sin preguntas.

### Tras clonar — instalar

```bash
cd <KIT_PATH>
python3 -m venv venv
venv/bin/pip install -e .
venv/bin/indaws-migrate --version   # smoke
```

---

## Validación rápida del entorno

```bash
$INDAWS_MIGRATE_CLI --version
$INDAWS_MIGRATE_CLI playbook-session 2>/dev/null   # ver §"Apertura de sesión" del SKILL.md
```

Si el segundo falla con "no migration in progress", es OK: significa que el
CLI funciona pero estás en un proyecto sin workdir aún.

---

## Convención: repo del workdir cliente

Cada cliente vive en su propio repo GitHub. **Una vez localizado el motor**,
la siguiente decisión de la skill es **dónde vivirá el workdir del cliente**.

### Naming

| Concepto | Convención |
|---|---|
| Nombre repo GitHub | `odoo-migkit-<cliente>` (kebab-case) |
| Org | `indawsodoo` |
| URL SSH | `git@github.com:indawsodoo/odoo-migkit-<cliente>.git` |
| Visibilidad | privado |
| Branch | `master` (rama única) |
| Path local | `<project_root>/migration_kit_<cliente_snake>/` |

### Diálogo en apertura de sesión

Antes de `indaws-migrate init`, preguntar al usuario vía `AskUserQuestion`:

> ¿Tiene el cliente ya repo de workdir en GitHub?

| Opción | Significa | Acción de la skill |
|---|---|---|
| Sí, ya existe | El usuario aporta URL | `indaws-migrate workdir-repo clone --remote URL --dest <project_root>` |
| No, créalo nuevo | Crear en `indawsodoo` | `indaws-migrate init --client X --create-remote ...` |
| Solo local por ahora | Trabajo sin git | `indaws-migrate init --client X` sin `--git-remote` |

### Si `gh repo create` falla

Si no hay `gh` autenticado, el CLI devuelve instrucciones manuales:

```
1. Abre https://github.com/new
2. Owner: indawsodoo
3. Repository name: odoo-migkit-<cliente>
4. Visibility: Private
5. Sin README/gitignore/license
```

Tras crearlo, repetir con `--git-remote git@github.com:indawsodoo/odoo-migkit-<cliente>.git`.

### Auto-push tras eventos clave

Una vez el remote está configurado, la skill puede pedir al CLI que haga
auto-commit + push tras eventos importantes:

| Evento | Comando |
|---|---|
| Tras `suggest --apply` exitoso | `indaws-migrate suggest --apply --commit` |
| Tras `iterate` con éxito | `indaws-migrate iterate ... --push` |
| Tras `ack <gate>` | `indaws-migrate ack <gate> ... --push` |
| Manual ad-hoc | `indaws-migrate workdir-repo push --workdir . --message "X"` |

Esto cumple la convención: cada hito significativo queda reflejado en el
repo remoto.

---

## Cuándo se rompe esto

| Síntoma | Causa probable | Fix |
|---|---|---|
| `indaws-migrate: command not found` | Venv no activado / contexto del agente sin ruta | Re-ejecutar `locate_kit.sh` |
| `ModuleNotFoundError: indaws_migration_kit` | `pip install -e .` no se hizo | `venv/bin/pip install -e .` en KIT_PATH |
| Comando funciona pero versión vieja | Has hecho `git pull` pero el venv no se reinstaló | Idem (editable install no requiere reinstall, salvo deps cambiadas) |
| Funciona en una shell pero no en otra | Olvidaste exportar `INDAWS_MIGRATE_CLI` | Re-fuente `locate_kit.sh` o usa path absoluto |
