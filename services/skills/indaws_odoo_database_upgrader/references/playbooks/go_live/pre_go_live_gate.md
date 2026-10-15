# Playbook: Pre go-live gate

Comprobaciones automáticas que el agente lanza antes de cualquier intento de
go-live (FASE 4 del skill o `indaws-migrate ready`).

## Requisitos del contexto
- `to_version`: cualquiera
- `target_deployment`: declarado (odoo_sh | indaws | custom)

## Aplicabilidad
- Siempre — playbook obligatorio antes de go-live

## Flujo

Ejecuta los 8 gates de `migration.gates`:

| Gate                       | Cómo                                                |
|---|---|
| `pipeline_clean`           | `indaws-migrate run --apply` exit 0                 |
| `smoke_test_passed`        | Última `iterations/N/state.json` con `success=true` |
| `comparison_with_ref`      | `indaws-migrate compare` + ack manual               |
| `studio_pending_resolved`  | ack (lista resuelta o aceptada)                     |
| `pdf_validation`           | ack (cliente firmó samples)                         |
| `rollback_plan`            | ack + existe `docs/rollback.md`                     |
| `security_review`          | ack (ACLs/users revisados)                          |
| `data_freeze_communicated` | ack (cliente notificado)                            |

## Comando
```bash
indaws-migrate ready --workdir $PROJECTS_ROOT/MICLIENTE
```

Devuelve exit 0 si todos los gates están en verde, 1 en caso contrario.

## Pitfalls
- En `odoo.sh` el go-live técnico lo dispara la plataforma; el gate aquí solo
  garantiza que la rama staging es upgradable
- En `indaws/custom`, tras `ready` verde, ejecutar `indaws-migrate go-live`
