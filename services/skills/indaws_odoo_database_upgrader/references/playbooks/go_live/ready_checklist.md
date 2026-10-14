# Playbook: Ready checklist consolidado

Checklist final que el agente repasa con el usuario antes del go-live real.

## Requisitos del contexto
- `to_version`: cualquiera
- `target_deployment`: declarado

## Aplicabilidad
- Una vez que `indaws-migrate ready` devuelve verde

## Checklist

- [ ] `pipeline_clean` — `run --apply` reciente sin errores ni warnings
- [ ] `smoke_test_passed` — última iteración con `success=true`
- [ ] `comparison_with_ref` — sin críticos, ack manual del consultor
- [ ] `studio_pending_resolved` — lista resuelta o aceptada por cliente
- [ ] `pdf_validation` — cliente firmó samples representativos
- [ ] `rollback_plan` — `docs/rollback.md` revisado, dumps de seguridad listos
- [ ] `security_review` — grupos, ACLs y users coinciden con v15 (excepto cambios deseados)
- [ ] `data_freeze_communicated` — fecha + ventana comunicada al cliente

### Adicionales por destino

**`target_deployment = odoo_sh`**
- [ ] Rama staging mergeada y subida a odoo.sh
- [ ] "Test Upgrade" lanzado y verde en staging
- [ ] Commit listo para trigger Phase 2

**`target_deployment = indaws | custom`**
- [ ] `aim=production` listo para 1 envío al Upgrade Service
- [ ] Servidor destino accesible, filestore copiado
- [ ] DNS / proxy reverso preparado

## Comando final
```bash
indaws-migrate go-live --workdir $PROJECTS_ROOT/MICLIENTE
```

## Pitfalls
- Olvidar el freeze: el cliente sigue trabajando en v15 y el dump migrado queda obsoleto
- `aim=production` solo se puede usar 1 vez por contrato — no malgastarlo
