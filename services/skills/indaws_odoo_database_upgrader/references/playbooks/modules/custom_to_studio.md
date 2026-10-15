# Playbook: Custom → Studio

Módulo custom con baja complejidad (pocos campos, sin lógica Python compleja).
Migrar la funcionalidad a **Odoo Studio** y eliminar el código del cliente.

## Requisitos del contexto
- `features.studio: true` (obligatorio — sin Studio EE no aplica)
- `to_version`: 18.0 (Studio v18 es target)

## Aplicabilidad
- Módulo custom del cliente
- Funcionalidad expresable en Studio: campos extras en modelo existente,
  filtros, vistas form/list, automations simples
- **NO aplica si**: hereda modelos abstractos, override de métodos compute con
  lógica, integraciones externas (REST/SOAP), módulos report PDF complejos

## Flujo
1. **Mapear**: extraer del manifest los campos `fields.*` declarados en
   `models/*.py` → tabla destino en Studio
2. **Confirmar mapping** con humano (categoría 3)
3. **Delegar a skill `odoo-studio-migration`** para crear los campos/automations
   en la BD destino
4. **Eliminar código fuente** del módulo custom (post-validación)
5. **Marcar el módulo como STUB**:
   - Tablas conservadas (datos preservados)
   - Stub auto-generado en `<workdir>/stubs/`
   - Artefactos UI archivados (las nuevas vistas las hace Studio)

## Helpers a invocar
- Skill externa: `odoo-studio-migration`
- Helpers del kit (post-mapping):
  - `helpers.odoo.stub_generator.StubGenerator`
  - `helpers.odoo.archive_artifacts.ArchiveModuleArtifacts`

## Pitfalls
- Studio NO permite override de métodos Python: si el módulo tenía `@api.depends`,
  la lógica compute debe re-implementarse via "Server Action" / "Automation"
- Validar con cliente que las nuevas vistas Studio reproducen la UX previa
- Datos de campos `Many2one` con `ondelete='restrict'` que apuntan a registros
  del módulo custom: revisar antes de eliminar
