# Playbook: Reportar vistas Studio pendientes de fix manual

Vistas que no pudo reactivar `reactivate` ni `autofix`. Generar lista para
revisión humana / acción manual en Studio v18.

## Requisitos del contexto
- `features.studio: true`

## Aplicabilidad
- Existen vistas en `studio_customization` aún archivadas tras `reactivate` y `autofix`

## Flujo
1. Listar vistas Studio archivadas finales
2. Para cada vista: extraer model, name, causa raíz inferida
3. Renderizar markdown:
   ```
   ## <model>
   - **Vista:** <name>
   - **Causa raíz inferida:** Field `xxx` does not exist / XPath no matches
   - **Acción sugerida:** Editar manualmente en Studio v18 → ...
   ```
4. Output: `<workdir>/docs/studio_views_pending.md`

## Helpers a invocar
```yaml
- id: studio_pending_report
  type: indaws_migration_kit.helpers.studio.pending_report.StudioPendingReport
  params:
    output_path: docs/studio_views_pending.md
```
