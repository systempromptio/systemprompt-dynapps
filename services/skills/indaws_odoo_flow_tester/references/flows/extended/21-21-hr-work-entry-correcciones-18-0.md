## 21. hr.work.entry — Correcciones 18.0

### Correcciones validadas en tests en vivo (2026-03-10)

#### Entradas validadas NO se pueden eliminar via ORM
- Las entradas de trabajo (`hr.work.entry`) en estado **validado** no se pueden eliminar via ORM.
- Intentar `unlink()` sobre entradas validadas genera un UserError.
- **Workaround:** Archivar el contrato o empleado relacionado en lugar de intentar eliminar las entradas.

**Secuencia de limpieza para entradas de trabajo:**
1. Si las entradas estan en estado `draft`: se pueden eliminar normalmente.
2. Si estan validadas: archivar el contrato asociado (`hr.contract` → `active=False`).
3. Alternativa: cancelar el contrato primero (`state='cancel'`), luego intentar eliminar entradas en draft.

---

