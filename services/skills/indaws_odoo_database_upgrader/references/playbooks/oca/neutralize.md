# Playbook: NEUTRALIZE módulo OCA no portado

Módulo OCA que **no existe en la versión destino** y para el cual el cliente
**no quiere perder datos**. Solución: stub vacío + archivar artefactos UI +
limpiar xml_ids de menús/acciones/vistas.

## Requisitos del contexto
- `to_version`: cualquiera (obligatorio)
- `features.*`: ninguna específica

## No requiere
- países específicos
- edición específica

## Aplicabilidad
- Módulo en `ir_module_module` con `state = 'installed'` o `'to upgrade'`
- Sin sucesor v18 conocido (ver `oca/replace_with.md`)
- Sus tablas contienen datos del cliente
- El módulo NO está en `migrated_modules/` (ver `oca/keep_migrated.md`)

## Detección automática
- `helpers.detect.oca_not_ported.detect`
- Cruzar con `catalogs.oca_renames` para confirmar ausencia de sucesor

## Flujo

1. **Pre-check** (obligatorio):
   ```sql
   SELECT name FROM ir_module_module WHERE name = '<modulo>' AND state = 'installed';
   ```
2. **Confirmar con humano** (categoría 3 D1) — decisión de diseño
3. **StubGenerator**: crea módulo vacío en `<workdir>/stubs/`
   ```yaml
   - id: generate_stubs
     type: indaws_migration_kit.helpers.odoo.stub_generator.StubGenerator
     params:
       modules: [<modulo>]
       stubs_path: '{workdir}/stubs/'
   ```
4. **ArchiveModuleArtifacts**: archiva vistas/menús/acciones/crons/rules/access
   ```yaml
   - id: archive_stub_modules_artifacts
     type: indaws_migration_kit.helpers.odoo.archive_artifacts.ArchiveModuleArtifacts
     params:
       modules: [<modulo>]
   ```
5. **NeutralizeOcaModule**: limpia xml_ids huérfanos pero preserva los datos en
   tablas. Comprueba `migrated_modules_path` para no neutralizar lo ya portado.
   ```yaml
   - id: neutralize_modules
     type: indaws_migration_kit.helpers.odoo.neutralize.NeutralizeOcaModule
     params:
       modules: [<modulo>]
       from_version: '{context.from_version}'
       to_version: '{context.to_version}'
       migrated_modules_path: '{workdir}/migrated_modules/'
   ```

## Helpers a invocar
- `indaws_migration_kit.helpers.odoo.stub_generator.StubGenerator`
- `indaws_migration_kit.helpers.odoo.archive_artifacts.ArchiveModuleArtifacts`
- `indaws_migration_kit.helpers.odoo.neutralize.NeutralizeOcaModule`

## Pitfalls aprendidos
- **Pharmactive**: `stock_quant_manual_assign` se neutralizó accidentalmente
  estando en `migrated_modules/` y perdió funcionalidad. **NeutralizeOcaModule
  comprueba `migrated_modules_path` ANTES de neutralizar.**
- Si el módulo tiene `automation`/server-actions con código Python, su `arch_db`
  queda archivado pero las automations siguen disparándose. Revisar `base_automation`.
