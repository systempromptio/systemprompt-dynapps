# Validación post-upgrade (v2.0)

Desde la integración con `indaws_migration_kit`, la validación se descompone en:

| Paso | Comando | Playbook |
|---|---|---|
| Restore + addons-path | `docker-deployment` skill | — |
| Smoke test `-u all` | `indaws-migrate run --apply` + step `smoke_test` | `playbooks/go_live/smoke_test.md` |
| Comparison vs ref | `indaws-migrate compare --reference <dump>` | `playbooks/go_live/comparison.md` |
| Validación PDFs | manual con cliente | `playbooks/go_live/pdf_validation.md` |
| Ready checklist | `indaws-migrate ready` | `playbooks/go_live/ready_checklist.md` |
| Go-live | `indaws-migrate go-live` | `playbooks/go_live/pre_go_live_gate.md` |

## Cuándo leer este fichero

Solo si se necesita el detalle histórico (docker-compose, odoo.conf, addons-path).
**Versión pre-kit archivada** fuera del repo. La validación actual la cubre `phases/fase-4-validacion.md` + helpers `view_arch_validator`/`view_arch_fixer`.

## Referencias actuales

- `references/playbooks/go_live/*.md` — recetarios go-live
- skill externa `docker-deployment` — levantar entorno local para validar
- skill externa `odoo-pilot` — interactuar con Odoo via API durante validación
