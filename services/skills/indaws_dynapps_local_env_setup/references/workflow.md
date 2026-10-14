# Local Env Setup — Full Workflow (STEP 0 → STEP 8)

Configura el entorno de desarrollo local para un cliente concreto. Se ejecuta
una vez por cliente por máquina.

## STOP Points

1. **Datos del cliente** — preguntar nombre, versión Odoo destino, origen del repo
   (GitLab Dynapps / GitHub DynAppsNV) y path de la DB migrada disponible.
2. **DB migrada** — confirmar que la DB está disponible y accesible localmente antes de
   continuar con la inicialización.
3. **SmartGit y PyCharm** — guiar la configuración con instrucciones detalladas, pero el
   usuario confirma que cada paso está completado.

## STEP 0 — Recopilar datos del cliente [STOP]

Usar `AskUserQuestion` para obtener:
- Nombre del cliente (slug corto, ej: `acme`)
- Versión Odoo destino (ej: `17`, `18`)
- Repo: GitLab Dynapps (`gitlab.dynapps.be:customers/`) o GitHub DynAppsNV (`github.com:DynAppsNV/`)
- Tipo de virtualenv preferido: pyenv (1) o virtualenv (2)
- ¿El cliente está en Odoo SH? (sí/no)
- Path de la DB migrada disponible (o confirmar que se creará)

Construir variables:
- `CUSTOMER` = slug del cliente
- `VERSION` = versión Odoo
- `BUILDOUT_PATH` = `$ODOO_SRC/buildouts/${CUSTOMER}${VERSION}`

## STEP 1 — Crear el entorno local

Ejecutar `setup-local-environment` con los parámetros recogidos.
Ver `references/env-creation.md § setup-local-environment` para los comandos exactos.

Si el repo es de GitHub/DynAppsNV: responder `N` cuando el script pregunta por GitLab.

## STEP 2 — Verificar estructura del buildout

```bash
ls $ODOO_SRC/buildouts/${CUSTOMER}${VERSION}/
# Esperado: odoo.cfg, parts/, dyn-build.yaml, ...
```

Revisar y ajustar `dyn-build.yaml` si es un proyecto de migración.
Ver `references/env-creation.md § dyn-build.yaml`.

## STEP 2.5 — Verificar PostgreSQL y restaurar dump [STOP]

Antes de inicializar Odoo, garantizar que PostgreSQL corre y el dump migrado está restaurado.
Ver `references/env-creation.md § PostgreSQL setup`.

```bash
# 1. Comprobar que PostgreSQL está activo
pg_isready -h localhost
# Si falla → sudo systemctl start postgresql (o sudo service postgresql start)

# 2. Confirmar dump disponible (pedir path al usuario si no está en contexto)
# 3. Crear la BD y restaurar
createdb ${CUSTOMER}${VERSION}_migrated -T template0
pg_restore -d ${CUSTOMER}${VERSION}_migrated $DUMP_PATH/original.dump
```

## STEP 3 — Inicializar DB del cliente [STOP]

Confirmar que la BD migrada está restaurada. Ejecutar actualización con xx_base.
Ver `references/env-creation.md § Inicialización DB`.

## STEP 4 — SmartGit [STOP]

Guiar al usuario para añadir el repositorio y configurar Git-Flow.
Ver `references/ide-config.md § SmartGit`. Esperar confirmación.

## STEP 5 — PyCharm: Interpreter [STOP]

Proporcionar el path correcto del intérprete según el tipo de venv elegido.
Ver `references/ide-config.md § PyCharm Interpreter`. Esperar confirmación.

## STEP 6 — PyCharm: Run/Debug y Test configs [STOP]

Proporcionar los valores exactos para las configuraciones Run y Test.
Ver `references/ide-config.md § PyCharm Run/Debug` y `§ PyCharm Test`. Esperar confirmación.

## STEP 7 — PyCharm: Mark directories [STOP]

Indicar qué directorios marcar como Excluded y Sources Root.
Ver `references/ide-config.md § Mark Directories`. Esperar confirmación.

## STEP 8 — Verificación final

```bash
# Activar el venv y lanzar Odoo en modo check
cd $ODOO_SRC/buildouts/${CUSTOMER}${VERSION}
./parts/odoo/odoo/odoo-bin -c odoo.cfg --version
```

Mostrar resumen: buildout path, DB usada, tipo de venv, paths del intérprete.
