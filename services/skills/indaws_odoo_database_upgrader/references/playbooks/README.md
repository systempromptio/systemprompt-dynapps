# Playbooks de migración — Recetarios del skill

Conjunto de **recetarios** (markdown estático) que el agente consulta durante
una migración para aplicar patrones conocidos. Cada playbook responde a una
situación recurrente y enumera el flujo + helpers del `indaws_migration_kit`.

## Estructura

```
playbooks/
  modules/       custom_to_studio, custom_to_core, custom_to_stub, custom_port, custom_report_to_docx
  oca/           neutralize, replace_with, keep_migrated
  studio/        reactivate, autofix, pending
  data/          aeat_map_tax, preserve_orphan_xmlid, missing_table, missing_column, constraint_violation
  go_live/       pre_go_live_gate, comparison, smoke_test, pdf_validation, ready_checklist
```

## Cabecera estándar (G2)

Cada playbook empieza con:

```markdown
## Requisitos del contexto
- features.studio: true (obligatorio)
- to_version: cualquiera (info)

## No requiere
- países específicos

## Aplicabilidad
- criterios de detección

## Flujo
1. ...

## Helpers a invocar
- helpers.X.Y
```

El agente NO activa una playbook si los requisitos no se cumplen. Esto permite
filtrar automáticamente por contexto (cliente español, edición EE, features, …).

## Estado MVP (D5)

Implementados los grupos 1, 2 y 4 (pilares, módulos custom, go-live).
Grupos 3 (datos avanzados) y 5 (especiales) on-demand.
