# Diffs por version del mecanismo de migracion

Base v18. Citas fichero:linea reales.

## El nucleo es ESTABLE v17/v18/v19

El contrato no cambia entre versiones:
- Carpeta `migrations/<version>/`, ficheros `pre-`/`post-`/`end-`.
- Funcion `def migrate(cr, version):` recibiendo cursor + version instalada.
- `convert_version`, `compare` y el caso `0.0.0` son IDENTICOS en v17/v18/v19
  (mismo codigo en `migrate_module`).
- `odoo.tools.sql` expone los mismos helpers principales (`column_exists`,
  `rename_column`, `convert_column`, `table_exists`, `set_not_null`,
  `drop_view_if_exists`) en las tres versiones.

Lo que escribas funciona igual en v17/v18/v19 salvo los matices siguientes.

## DIFF v17 (vs v18 base): la firma NO se valida

En **v17**, `exec_script` (`odoo17/odoo/modules/migration.py::exec_script`):
- NO existe `VALID_MIGRATE_PARAMS` ni validacion de firma. Cualquier `migrate`
  invocable se ejecuta; no se levanta `TypeError` por firma rara.
- Si falta la funcion `migrate`, solo se LOGGEA un `AttributeError`
  (v17 migration.py:236-237) y CONTINUA — no aborta el upgrade.
- Usa `pkg.installed_version` / `pkg.data['version']` y el estado `pkg.state`
  (v17 migration.py:198-200, :155-158).

En **v18** (base, migration.py:225-256) la firma SI se valida: una firma que no
sea una combinacion de `(cr|_cr, version|_version)` posicional LEVANTA `TypeError`,
y la ausencia de `migrate` LEVANTA `AttributeError`. Practico: en v18+ un script
con firma incorrecta rompe el upgrade en vez de pasar desapercibido.

## DIFF v19 (vs v18 base): registry, manifest y force-upgrade

`$ODOO_SRC/odoo19/odoo/modules/migration.py`:
- Validacion de firma identica a v18 (LEVANTA `TypeError`/`AttributeError`).
- El gate de "que modulos corren scripts" cambia: usa `pkg.load_state` y
  `Registry(self.cr.dbname)._force_upgrade_scripts` (v19 migration.py:133-134,
  :151) en vez del `hasattr(pkg,'update')`/`pkg.state` de v18 (v18
  migration.py:132-134, :156-158). Permite forzar scripts de un modulo via el
  registry.
- Lee la version del manifest desde `pkg.manifest['version']` y la instalada desde
  `pkg.load_version` (v19 migration.py:192-194) en vez de `pkg.data['version']` /
  `getattr(pkg,'load_version', pkg.installed_version)` de v18 (v18 :199-201).
- `load_script` se importa de `odoo.modules.module` y el nombre de modulo cargado
  pasa a ser `odoo.upgrade.{addon}.{version}.{name}` (v19 migration.py:18, :234).
- STEP 3.5 (end-scripts) esta en `odoo/modules/loading.py:501`, `def load_modules` (v18: `odoo/modules/loading.py:523`, `def load_modules`). El
  orden relativo end-despues-de-todo es el mismo.

Ninguno de estos cambios afecta a como ESCRIBES un `migrate(cr, version)` custom:
la firma y las fases son las mismas. Solo cambian internals del manager.

## Tabla resumen

| Aspecto | v17 | v18 (base) | v19 |
|---------|-----|------------|-----|
| Firma `migrate(cr, version)` | requerida (no validada) | requerida + validada (TypeError) | requerida + validada (TypeError) |
| Falta `migrate` | loggea, continua | LEVANTA AttributeError | LEVANTA AttributeError |
| Version del manifest | `pkg.data['version']` | `pkg.data['version']` | `pkg.manifest['version']` |
| Estado que dispara scripts | `pkg.state` / `hasattr update` | `pkg.state` / `hasattr update` | `pkg.load_state` + `_force_upgrade_scripts` |
| STEP 3.5 end-scripts | loading.py | loading.py:523 | loading.py:501 |
| `convert_version` / `0.0.0` / `compare` | igual | igual | igual |
| `odoo.tools.sql` helpers | iguales | iguales | iguales |

Para firmas de API que uses DENTRO del `migrate` (ORM, `add_to_compute`, etc.),
no las asumas de memoria: delega en `odoo-version-api-reference`.
