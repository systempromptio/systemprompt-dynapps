# Reglas del agente para `odoo-database-upgrader`

## Autonomía D1

1. **Lecturas/archivos** = silencioso (sin pedir permiso).
2. **Fixes reversibles** (NEUTRALIZE, Studio autofix, `-u all` staging) =
   autónomo con log.
3. **Diseño por módulo** (Studio / Core / Remove / STUB) = **[STOP]
   pregunta** vía `AskUserQuestion`. Batch si mismo tipo.
4. **Acciones externas** (`gh pr create`, push, email) = pregunta.
5. **Destructivos irreversibles** (DROP, DELETE datos, `--no-verify`) =
   **nunca solo**.

En **go-live**, las categorías 3-4 que ya estaban decididas se suben a
nivel ⛔ (no se aplican sin confirmación explícita en el momento).

## Reglas globales

- **Preservar v_prev**: cambios mínimos durante el port. No refactor.
- **Severidad de logs**: ignorar `WARNING`s. Bloquear solo
  `ERROR`/`CRITICAL`.
- **Nunca borrar `ir_model_data` / vistas sin preguntar**. Usar
  `PreserveOrphanXmlid` o el flag opt-in del kit (P5/P6/P7).
- **Helpers y playbooks**: siempre genéricos, nunca con valores cliente
  hardcoded.
- **Idempotencia obligatoria** en cualquier fix SQL emitido por la skill
  o por el kit: re-ejecutar no debe romper la BD ni duplicar registros.

## Casuística típica

| Situación | Categoría | Acción esperada |
|---|---|---|
| Iteración con error parseable conocido | (2) | Auto-aplicar fix (P5/P6/P7/P8) tras `suggest --apply` |
| Módulo OCA sin equivalente en v18 | (3) | Preguntar: STUDIO / CORE_REPLACE / TO_DROP / port |
| `migrated_modules/` necesita módulo nuevo | (2) | Crearlo + declarar en `modules: KEEP` |
| Conflicto en `migration.yaml` post `suggest` | (3) | Mostrar diff + preguntar antes de aplicar |
| Push a `origin` del workdir cliente | (4) | Preguntar (puede afectar a otro dev) |
| `DROP TABLE` o `DELETE FROM` sin where | (5) | Bloquear, mostrar query, pedir override explícito |
