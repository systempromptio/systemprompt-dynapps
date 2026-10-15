# Playbook: REPLACE_WITH módulo OCA renombrado

OCA con sucesor v18 conocido (rename del repositorio OCA). Instalar el sucesor
y neutralizar el legacy.

## Requisitos del contexto
- `to_version`: cualquiera

## Aplicabilidad
- Módulo OCA legacy en `ir_module_module` con `state = 'installed'`
- Existe sucesor v18 en `catalogs/oca_renames`

## Flujo

1. **Confirmar mapping** (humano si el catálogo no tiene certeza)
2. **Instalar sucesor** (operación XML-RPC, no SQL):
   ```python
   env['ir.module.module'].search([('name','=','<successor>')]).button_immediate_install()
   ```
3. **Migrar datos**: depende del par concreto. Casos típicos:
   - `account_statement_import` → `account_statement_import_file`: ambos
     comparten modelo `account.statement.import`, no requiere migración
   - Si no comparten modelo: helper específico (un `data/migrate_records.md`)
4. **Neutralizar legacy** (variante reducida del playbook NEUTRALIZE):
   ```yaml
   - id: archive_legacy
     type: indaws_migration_kit.helpers.odoo.archive_artifacts.ArchiveModuleArtifacts
     params:
       modules: [<legacy>]
   ```

## Helpers a invocar
- `helpers.odoo.archive_artifacts.ArchiveModuleArtifacts`
- (data migration depende del par concreto)

## Pitfalls
- **Pharmactive**: borramos `ir_model_data` de `account_statement_import` sin
  copia. Para regenerar bastó reinstalar el módulo. Conclusión: **NO borrar
  registros, archivar.**
