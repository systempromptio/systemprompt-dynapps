# FASE 2 — Planificación módulos

Combinar `estudio_normalizado.yaml` (FASE 1) + inferencias técnicas (FASE 1.5) → poblar `migration.yaml.modules`.

## Procedimiento

Por cada módulo:
1. Si tiene decisión clara en el estudio → escribir en `migration.yaml`
2. Si es ambiguo → leer playbook correspondiente, presentar al usuario, **[STOP] `AskUserQuestion`**
3. Si es de tipo `KEEP` (port código) → marcar para FASE 3, no se invoca `odoo-module-upgrader` aún

Output: `migration.yaml` completo con `modules` + `fixes`. `playbook-init` crea `playbook.md`.

## Decisiones macro (D5)

Las acciones válidas por módulo:

| Acción | Cuándo |
|---|---|
| `KEEP` | Custom o OCA portado en `migrated_modules/` — mantener |
| `STUDIO` | Funcionalidad expresable en Studio EE — migrar no-code |
| `CORE_REPLACE` | Existe equivalente nativo v18 o Indaws Core |
| `STUB` | No se usa pero hay datos legacy a preservar |
| `NEUTRALIZE` | OCA no portado + datos vivos → stub + archive + xml_id cleanup |
| `REPLACE_WITH` | OCA renombrado con sucesor v18 conocido |
| `REMOVE` | Uninstall + cleanup ir_model_data (tablas conservadas) |
| `TO_DROP` | Uninstall + DROP TABLE completo (datos perdidos, BD limpia). Mejor ANTES del upgrade si es posible — ver `playbooks/modules/to_drop.md` |

Cada decisión tiene su playbook en `references/playbooks/oca/`, `references/playbooks/modules/`.

## Delegación: fit-check para módulos `STUDIO`

Cuando un módulo cae en categoría `STUDIO` (propuesto por el consultor o
emergido como alternativa al port), NO decidir a ojo — delega:

```python
Agent(model="opus", description=f"Studio fit {modulo}",
      prompt=<ver agent-tree.md §Delegaciones — odoo-studio-migration>)
```

El sub-agente devuelve `{viable: bool, studio_spec, blockers}`. Si
`viable: true` → copiar `studio_spec` a `modules.<name>.studio_spec` del
yaml. Si `viable: false` → reevaluar contra `KEEP` (port) o
`NEUTRALIZE`.
