# Path resolution en helpers del kit

Convención de cómo se resuelven los paths en los params de cada helper. La
inconsistencia ha sido fuente de bugs (M9 detectada en Tecniman 2026-05).

## Las 3 referencias

| Referencia | Qué es | Cuándo usar |
|---|---|---|
| **absoluto** | path empezando por `/` | Cuando el path es global, no depende del cliente |
| **relativo a `ctx.workdir`** | resuelto contra `migration_kit_<cliente>/.indaws_migrate/<cliente>/` | Para artefactos generados (stubs, logs, sql/ del cliente) |
| **relativo a `ctx.pipeline_dir`** | resuelto contra el dir del `migration.yaml` (= `migration_kit_<cliente>/`) | Para inputs del cliente (steps/, sql/, dumps/) |

## Regla por defecto

**Un path en `params` es:**
1. Absoluto si empieza por `/` → usar tal cual.
2. Si no, **relativo a `ctx.workdir`** por defecto.

Esto es lo que hacen `NeutralizeOcaModule`, `StubGenerator`, `ArchiveModuleArtifacts`, etc.

## Convención por helper

| Helper | Param | Resolución |
|---|---|---|
| `NeutralizeOcaModule` | `migrated_modules_path` | absoluto o relativo a workdir |
| `NeutralizeOcaModule` | `stubs_path` | absoluto o relativo a workdir |
| `ReplaceWithSuccessor` | `migrated_modules_path` | absoluto o relativo a workdir |
| `ReplaceWithSuccessor` | `successor_paths` (list) | cada uno absoluto o relativo a workdir |
| `DropModules` | `log_path` | absoluto o relativo a workdir |
| `RunSqlFile` | `file` | relativo a pipeline_dir (no workdir) |
| `RunPython` | `module` | dotted-path; pipeline_dir está en sys.path |

## Recomendación práctica en `migration.yaml`

**Para evitar ambigüedad, usar paths absolutos en `context.source_repos`:**

```yaml
context:
  source_repos:
    custom: $PROJECTS_ROOT/<cliente>/migrated_modules    # absoluto
    indaws_core: $PROJECTS_ROOT/indawsodoo/18            # absoluto
    core: $ODOO_SRC/odoo18/addons                       # absoluto
    enterprise: $ODOO_ENTERPRISE_SRC/18                 # absoluto
    oca: $OCA_SRC/v18                                   # absoluto
```

El compiler propaga estos paths a `ReplaceWithSuccessor.successor_paths` y a
`NeutralizeOcaModule.migrated_modules_path`, así todos los helpers buscan en
la misma lista de fuentes.

**Para `steps_custom` con scripts locales del cliente, usar relativos:**

```yaml
steps_custom:
  - id: run_brenda_partner
    type: indaws_migration_kit.helpers.sql.RunPython
    params:
      module: steps.migrate_tecniman_partner   # relativo a pipeline_dir
```

## Gotchas

- **No mezclar:** `migrated_modules: ./migrated_modules` en `source_repos` se
  resuelve relativo a `ctx.workdir` (no a `pipeline_dir`). En Tecniman esto
  resultaba en `<workdir>/.indaws_migrate/<cliente>/migrated_modules` (path
  inexistente). Mejor poner absoluto.
- **`successor_paths` busca también en sub-repos (1 nivel).** Patrón OCA:
  `$OCA_SRC/v18/<repo>/<modulo>/__manifest__.py`. El helper recorre 1 nivel
  extra automáticamente. Para profundidades mayores, añadir el sub-repo
  como path independiente en `source_repos`.
