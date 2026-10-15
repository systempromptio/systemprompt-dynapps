## Tiers de Reversion

### Tier 1 — Reversion ORM (Preferida)

La reversion estandar via API de Odoo. Segura, respeta constraints y triggers.
Secuencia: `action_cancel` → `button_draft` → `unlink`

**Usar siempre como primera opcion.** Ver secciones de "Estrategias de Reversion por Modelo" mas abajo.

### Tier 2 — ORM avanzado + execute_method (Segundo recurso)

Cuando la reversion ORM estandar falla, usar metodos internos con contextos especiales
que bypaseen restricciones. No requiere acceso SQL.

**Cuando escalar de Tier 1 a Tier 2:**

| Situacion | Por que falla Tier 1 | Accion Tier 2 |
|-----------|---------------------|---------------|
| `button_draft` falla por `inalterable_hash` | Hash de integridad fiscal impide draft | `_reverse_moves()` para crear asiento inverso |
| `button_draft` falla por exchange diff | `_check_draftable()` detecta exchange_move_id | Desreconciliar primero (el exchange se revierte solo) |
| `button_draft` falla por cash basis | `_check_draftable()` detecta tax_cash_basis | Desreconciliar primero (CABA entry se revierte solo) |
| `unlink` falla por FK no gestionada | Registro referenciado por tabla inesperada | Borrar dependencias primero con search + delete |
| `action_cancel` lanza `UserError` | Modulo custom anade validacion | Contextos especiales (`force_cancel`, `force_delete`) |
| Reconciliacion bloquea cancelacion pago | Pago reconciliado con factura | `action_draft()` desreconcilia automaticamente (llama `button_draft` → `remove_move_reconcile`) |
| Secuencia incrementada por test | Secuencia avanzada innecesariamente | `update_record` en `ir.sequence` con `number_next_actual` |

Ver `flows.py --revert-tema "NIVEL 2"` para detalles completos.

### Tier 3 — SQL Directo (Ultimo Recurso)

Cuando la reversion ORM y los metodos avanzados fallan por constraints irrecuperables,
hash fiscal o estados corruptos. Requiere acceso SSH + psql al servidor.

**Cuando escalar de Tier 2 a Tier 3:**

| Situacion | Por que falla Tier 2 | Accion SQL |
|-----------|---------------------|------------|
| `inalterable_hash` impide cualquier cambio | Hash criptografico fiscal | DELETE directo (solo con aprobacion del riesgo fiscal) |
| Registro en estado inconsistente (bug) | Estado invalido, no responde a ningun metodo | UPDATE estado + DELETE |
| Timeout en operacion ORM por volumen | Demasiados registros encadenados | DELETE masivo con transaccion |
| Constraint de BD no gestionable via ORM | Trigger PostgreSQL custom | DELETE tras desactivar constraint temporalmente |

**Procedimiento de escalacion:**

1. Intentar Tier 1 (ORM estandar)
2. Si falla, capturar el error exacto
3. Intentar Tier 2 (metodos avanzados + contextos)
4. Si falla, escalar a Tier 3 (SQL)
5. Consultar `flows.py --revert <modelo>` (Nivel 3) para el modelo afectado
6. Ejecutar dentro de `BEGIN; ... ROLLBACK;` primero para verificar
7. Registrar en el log que se uso SQL (para auditoria)

Ver `flows.py --revert <modelo>` para las queries SQL detalladas de ese modelo.

---

