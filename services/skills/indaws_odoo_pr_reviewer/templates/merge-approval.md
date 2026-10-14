# Review del PR — Aprobado · {título de la tarea}

**PR**: {url} | **Tarea**: {project.task id/nombre} | **Rama**: `{rama}` | **Revisor**: {arquitecto}
**Veredicto**: ✅ Aprobado → merge

## Checklist de merge (todo verde)

- [x] Módulo instala/actualiza en Docker limpio (versión {X.0}{ + dump cliente}).
- [x] Contrato intacto (firmas/campos `# CONTRACT`, fronteras, deps, integración).
- [x] Criterios de aceptación cubiertos con tests unit + e2e.
- [x] Tests en verde real ejecutados en el Docker: {output/enlace}.
- [x] Impacto Odoo resuelto y comprobado en el Docker (datos/multi-company/seguridad/upgrade/rendimiento).
- [x] `/review` y `quality-gate` sin bloqueantes.
- [x] Docker de review tumbado.

## Notas

- {observaciones menores no bloqueantes, si las hay}.
- {lo destacable de la implementación — reconocimiento}.

## Acciones

- Merge de `{rama}` (vía `git-review-pr`).
- `project.task` actualizada a {estado} con constancia del review (vía `odoo-pilot`/MCP).
