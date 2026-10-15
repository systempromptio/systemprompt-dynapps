# Playbook: KEEP módulo OCA ya portado por el cliente

Módulo OCA presente en `migrated_modules/` con versión `to_version`.
**No tocar** — el upgrade lo carga normalmente.

## Requisitos del contexto
- `to_version`: cualquiera

## Aplicabilidad
- Módulo en `ir_module_module` con `state ∈ {installed, to upgrade}`
- Existe carpeta `migrated_modules/<modulo>/` con `__manifest__.py` válido
  para `to_version`

## Flujo
1. Verificar manifest: `version` empieza por `<to_version>.`
2. Verificar dependencias del manifest están todas en addons-path
3. (No genera step) — solo documentar decisión en playbook
4. Si en sesiones futuras se detecta error de carga: pasar a `oca/neutralize.md`
   o ejecutar `odoo-module-upgrader` para arreglarlo

## Helpers a invocar
- Ninguno — decisión documentada, sin step técnico

## Detección
- `helpers.detect.oca_keep_migrated.detect` (compara installed vs filesystem)
