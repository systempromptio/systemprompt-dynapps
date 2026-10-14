# Playbook: Custom → STUB

Módulo custom cuya funcionalidad ya no se quiere mantener pero cuyos **datos
deben preservarse** (histórico, auditoría, FKs vivas).

## Requisitos del contexto
- `to_version`: cualquiera

## Aplicabilidad
- Cliente confirma que no usa la funcionalidad
- Tablas contienen datos referenciados (FKs) o relevantes para histórico
- NO se requiere acceso UI a los datos

## Flujo
1. **Confirmar con cliente** que solo se preservan datos, no UI ni automations
2. **StubGenerator**: stub vacío en `<workdir>/stubs/<modulo>/`
   - `__manifest__.py` con dependencies = `['base']` y version `<to>.0.0.0.1`
   - Sin modelos, vistas, datos
3. **ArchiveModuleArtifacts**: archiva vistas/menús/acciones/access/rules
4. **NeutralizeOcaModule** (NO necesariamente — solo si el módulo dejó xml_ids
   huérfanos referenciables)

## Helpers a invocar
```yaml
- id: generate_stubs
  type: indaws_migration_kit.helpers.odoo.stub_generator.StubGenerator
  params:
    modules: [<modulo>]

- id: archive_artifacts
  type: indaws_migration_kit.helpers.odoo.archive_artifacts.ArchiveModuleArtifacts
  params:
    modules: [<modulo>]
```

## Pitfalls
- Si el módulo aportaba ACLs/permisos que otros módulos asumen, archivarlos
  puede dejar usuarios sin acceso. Revisar `ir.model.access` antes
- Si tenía `ir_cron` con `nextcall` futuro, archive_artifacts lo desactiva
