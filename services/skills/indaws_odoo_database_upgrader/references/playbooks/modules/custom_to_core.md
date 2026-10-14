# Playbook: Custom → Core (Odoo nativo)

Módulo custom cuya funcionalidad ya existe en Odoo v18 estándar o en `indaws_core`.
Reemplazar por la solución oficial.

## Requisitos del contexto
- `to_version`: cualquiera (>= versión del feature core)

## Aplicabilidad
- Funcionalidad **equivalente** disponible en core/EE o `indaws_core`
- El esquema de datos del custom es compatible con el nativo (o migrable)

## Flujo
1. **Confirmar equivalencia** (humano) — comprobar feature por feature
2. **Migrar datos**:
   - Identificar tablas/columnas del custom que NO están en el nativo
   - Helper `data/migrate_records.md` (custom playbook) si hay transformación
3. **Activar módulo core** (XML-RPC `button_immediate_install`)
4. **Marcar custom como STUB** (preserva datos legacy si los datos están en columnas
   propias). Si no, marcar `REMOVE`.

## Helpers a invocar
- Helpers de datos depende del par concreto
- Tras migración: `helpers.odoo.stub_generator.StubGenerator` o `archive_artifacts`

## Pitfalls
- "Equivalente" en feature ≠ equivalente en UX: validar pantallas con cliente
- Permisos: el módulo core puede usar grupos distintos. Re-asignar grupos a users
- Reports PDF: si el custom tenía templates QWeb, decidir si migrarlos al
  layout core o portarlos a `report_docx` (ver `modules/custom_report_to_docx.md`)
