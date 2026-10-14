# Reglas de mutación Odoo — Ticket Solver

- `search_read` SIEMPRE con `fields=[...]` explícito.
- **Backup** a `/tmp/rollback_<ticket>_<ts>.json` antes de `write`/`create`/`unlink`. En `create`, guarda `created_ids`.
- **Excepción sin backup JSON**: write a 1 registro por id unívoco a campo simple reversible — deja el comando inverso inline en `Resolución:`.
- **Pregunta antes con `AskUserQuestion`** si: >9 registros, toca `account.move` posted / periodo cerrado / `res.groups` / `ir.cron` / `ir.config_parameter` crítico, automation activa con `active=True`, target ambiguo, o multicompañía.
- **Escala** (no auto-apply aunque cumpla threshold): >100 registros, código Python custom, asientos posted / periodos cerrados.
- **Antes de tocar `ir.ui.view` / `ir.actions.*` / `ir.rule` / `base.automation` / campos Studio**: usa `odoo_query.py resolve_menu "<nombre parcial>"` y `resolve_action <model>`. Inferir target por intuición NO vale.

## Auto-fixes existentes (patrón `[CLA]`/`[AUTO]`/`[FIX]`)

Si el modelo del ticket tiene `ir.actions.server` o `base.automation` con prefijo `[CLA]`/`[AUTO]`/`[FIX]`:
1. Lee el código: `execute_method.sh ir.actions.server read '[[<id>]]' '{"fields":["code"]}'`.
2. Entiende por qué no disparó (condición filtrada, trigger mal configurado, llamada interna).
3. Amplía el código (un `write` sobre `code`), backup del anterior a `/tmp/rollback_<ticket>_action<id>_code.txt`.
4. Ejecuta sobre el registro fallido o fuerza trigger con un write trivial.
NO reinventes la lógica en paralelo.
