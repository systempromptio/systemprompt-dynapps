---
name: indaws-dynapps-local-env-setup
description: >
  Use this skill when a Dynapps developer needs to configure the local environment for a
  specific client project on their machine. Covers running setup-local-environment, DB
  initialization, SmartGit Git-Flow config, and PyCharm interpreter + run/debug/test
  configurations. Run ONCE per client per machine. Requires dynapps-dev-bootstrap completed.
  Activates for: "configurar entorno cliente", "setup local env", "preparar proyecto cliente",
  "dynapps local env setup", "configurar buildout", "nuevo cliente local".
version: 1.0.3
model: sonnet
effort: medium
allowed-tools: [Bash, Read, Write, Edit, AskUserQuestion]
---

# Dynapps Local Env Setup

Configura el entorno de desarrollo local para un cliente concreto. Se ejecuta **una vez por
cliente por máquina**. Al finalizar, el desarrollador podrá ejecutar y depurar Odoo del cliente
desde PyCharm con la DB migrada.

## Workflow

Procedimiento completo (STEP 0 → STEP 8) y STOP points en **`references/workflow.md`**. Resumen:

| Step | Qué | STOP |
|------|-----|------|
| STEP 0 | Recopilar datos del cliente (slug, versión, repo, venv, dump) | ✓ |
| STEP 1 | Crear entorno local (`setup-local-environment`) | |
| STEP 2 | Verificar estructura del buildout + `dyn-build.yaml` | |
| STEP 2.5 | Verificar PostgreSQL + restaurar dump | ✓ |
| STEP 3 | Inicializar DB del cliente (`-u xx_base`) | ✓ |
| STEP 4 | SmartGit: añadir repo + Git-Flow | ✓ |
| STEP 5 | PyCharm: Interpreter | ✓ |
| STEP 6 | PyCharm: Run/Debug + Test configs | ✓ |
| STEP 7 | PyCharm: Mark directories | ✓ |
| STEP 8 | Verificación final (`odoo-bin --version`) | |

## GOTCHAS

1. **El venv debe usar la versión de Python que corresponde a la versión Odoo destino.** Un
   mismatch (p.ej. Python de v16 para un buildout v18) rompe el arranque de Odoo con errores
   crípticos de dependencias.
2. **SmartGit necesita Git-Flow configurado explícitamente** — añadir el repo no basta; hay que
   habilitar Git-Flow (ramas `develop`/`master`) o las acciones de release/hotfix no aparecen.
3. **El path del intérprete en PyCharm depende del tipo de venv elegido** (pyenv vs virtualenv);
   apuntar al binario equivocado hace que PyCharm no resuelva los imports de Odoo.
4. **Si el repo es de GitHub/DynAppsNV, responder `N`** cuando `setup-local-environment` pregunta
   por GitLab — si no, el script intenta clonar de GitLab y falla.
5. **El dump de producción llega de `original.dump` y debe estar neutralizado** (lo entrega
   `dynapps-db-upgrade-runner` / `odoo-database-upgrader`). Restaurar uno sin neutralizar y arrancar
   Odoo dispara emails y crons reales.
6. **`pg_isready` debe pasar antes de restaurar** — si PostgreSQL no corre, `createdb`/`pg_restore`
   fallan; arrancar con `systemctl start postgresql` antes de STEP 2.5.

## RULES

### NEVER
- Nunca continuar tras un STOP point sin que el usuario confirme que el paso manual está completado.
- Nunca restaurar/arrancar una DB sin confirmar que está neutralizada.
- Nunca apuntar el intérprete de PyCharm a un venv con la versión de Python equivocada.
- Nunca ejecutar este skill antes de que `dynapps-dev-bootstrap` esté completado en la máquina.

### ALWAYS
- Ejecutar este skill UNA vez por cliente por máquina.
- Recoger todos los datos del cliente (STEP 0) antes de crear nada.
- Verificar PostgreSQL (`pg_isready`) antes de restaurar el dump.
- Verificar el arranque final (`odoo-bin --version`) antes de dar por completado.

## Delegates to

None — self-contained (configura entorno, DB e IDE directamente).

## Consumed by

| Skill | Suite | Context |
|-------|-------|---------|
| `dynapps-migrator` | developer | Orquestado en BLOQUE 1 (entorno local) |

See `common:references/shared/` for cross-suite shared conventions.

## References

| Fichero | Cuándo leerlo |
|---------|--------------|
| `references/workflow.md` | Procedimiento completo STEP 0–8 + STOP points |
| `references/env-creation.md` | STEPS 1–3 — setup-local-environment, dyn-build.yaml, init DB |
| `references/ide-config.md` | STEPS 4–7 — SmartGit y PyCharm completo |
| `common:references/shared/` | Convenciones compartidas cross-suite |

## Requires

- `dynapps-dev-bootstrap` completado en esta máquina
- Acceso al repo del cliente en GitLab o GitHub
- DB del cliente migrada disponible (de `odoo-database-upgrader` o `dynapps-db-upgrade-runner`)
- SmartGit y PyCharm instalados
- pyenv con la versión Python correcta para la versión Odoo destino
