# Code Review

**PR**: {enlace} | **Autor**: {nombre} | **Reviewer**: {TL} | **Módulo**: {nombre} | **Odoo**: {X.0}
**Resultado**: APROBADO / APROBADO CON CAMBIOS / RECHAZADO

## Checklist

### Arquitectura
- [ ] Responsabilidad única, deps mínimas, no modifica core, bridge si aplica

### Python
- [ ] Herencia correcta, computed con depends, sin N+1, sudo() justificado, API versión correcta, batch ops

### XML
- [ ] IDs prefijados, xpath específico, strings traducibles, campos sensibles ocultos

### JS (si aplica)
- [ ] OWL 2, servicios Odoo, cleanup lifecycle

### Seguridad
- [ ] ACLs modelos nuevos, record rules multi-company, sin credenciales hardcoded

### Tests y Manifest
- [ ] Tests lógica negocio, versión X.0.1.0.0, deps completas, auto_install solo bridge

## Hallazgos

| # | Severidad | Archivo:Línea | Descripción | Sugerencia |
|---|-----------|--------------|-------------|-----------|
| 1 | {Blocker/Critical/Major/Minor/Info} | {path:line} | {hallazgo} | {acción} |

## Notas
{Feedback constructivo, cosas bien hechas}
