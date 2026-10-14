# Playbook: Smoke test post-upgrade

Validación mínima de que la BD migrada es funcional: arranca, no tiene errores
críticos, sirve HTTP 200.

## Requisitos del contexto
- `to_version`: cualquiera
- Existe `docker-compose.yml` con servicio Odoo configurado

## Aplicabilidad
- Tras cualquier `run --apply` o iteración

## Flujo
1. Ejecutar `odoo -u all --stop-after-init` en container Odoo:
   ```bash
   docker compose run --rm odoo odoo -u all --stop-after-init -d <db>
   ```
2. Buscar en log: `Registry loaded` (success marker)
3. Contar `CRITICAL` y `ERROR` críticos
4. Arrancar Odoo en modo demonio + curl `http://localhost:8069/web/login` → esperar HTTP 200

## Helpers a invocar
```yaml
- id: smoke_test
  type: indaws_migration_kit.helpers.safety.smoke_test.SmokeTestRegistry
  params: {}
```

## Pitfalls
- Si el container reusa pip envs entre runs, deps custom pueden faltar — usar
  Dockerfile extendido con todas las deps cookeadas
- HTTP 200 en `/web/login` no implica login funcional — solo que el servidor responde
- Tests `--test-tags` puede ralentizar mucho el smoke; usar solo `-u all`
