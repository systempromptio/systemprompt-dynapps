# Estándares Code Review Odoo — TL

Objetivo: calidad + mentoría. Pick-up <4h, resolución <24h.

## Checklist por Área

### Python
- [ ] Herencia correcta (`_inherit` vs `_inherit + _name`)
- [ ] Computed fields con `depends` correcto y completo
- [ ] Sin loops N+1 (no `search()` dentro de `for`)
- [ ] `sudo()` justificado y mínimo
- [ ] Batch `create()`/`write()` donde aplica
- [ ] API compatible versión objetivo (ej: `_compute_display_name` v18+)

### XML/Vistas
- [ ] XML IDs con prefijo módulo correcto
- [ ] Herencia con `xpath` específico
- [ ] Strings traducibles, campos sensibles ocultos por grupo

### JavaScript (si aplica)
- [ ] OWL 2 (**v16+**, no v17 — corregido 2026-08-18: `@odoo/owl` pasa de 0 imports en v15 a 261 en v16), servicios Odoo (no fetch directo)
- [ ] Cleanup listeners/timers en willUnmount

### Seguridad
- [ ] ACLs para modelos nuevos, record rules multi-company si aplica
- [ ] Sin credenciales hardcodeadas (`ir.config_parameter` o env vars)

### Tests
- [ ] Tests presentes para lógica negocio, pasan sin errores

### Manifest
- [ ] Versión X.0.1.0.0, licencia, deps completas y mínimas
- [ ] `auto_install` solo en bridge modules

## Severidad

| Nivel | Acción |
|-------|--------|
| **Blocker** | Bug seguridad, pérdida datos, crash → NO mergear |
| **Critical** | Rendimiento grave, lógica incorrecta → Fix antes merge |
| **Major** | Violación convenciones → Fix recomendado |
| **Minor** | Estilo, naming → Sugerencia, no bloquea |
| **Info** | Tip educativo → Para aprendizaje |

## Anti-patrones Comunes

- **God Module**: un módulo hace todo → dividir
- **Override Festival**: cadenas overrides que se pisan → consolidar
- **SQL Escape**: raw SQL para evitar ORM → casi siempre hay forma ORM
- **Copy-Paste Inheritance**: copiar método entero para cambiar 1 línea → `super()` + delta

> **Template**: `templates/code-review-checklist.md`
