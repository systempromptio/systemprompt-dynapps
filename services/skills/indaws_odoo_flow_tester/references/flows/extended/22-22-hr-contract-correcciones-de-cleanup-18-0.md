## 22. hr.contract — Correcciones de Cleanup 18.0

### Correcciones validadas en tests en vivo (2026-03-10)

#### Auto-generacion de work entries al crear con state='open'
- Cuando se crea un contrato directamente con `state='open'`, Odoo auto-genera entradas de trabajo (`hr.work.entry`) inmediatamente.
- Esto complica significativamente la limpieza/reversion.

**Secuencia correcta de cleanup:**
1. Cambiar contrato a estado cancel: `write({'state': 'cancel'})`
2. Eliminar work entries en estado draft asociados
3. Si hay work entries validados: **NO se pueden eliminar** → archivar el contrato/empleado
4. Eliminar contrato

**Recomendacion para tests:** Crear contratos en estado `'draft'` y activarlos manualmente despues, para evitar la auto-generacion de work entries.

**⚠️ IMPORTANTE:** Si las work entries ya estan validadas, la unica opcion segura es archivar el contrato y el empleado asociado en lugar de intentar eliminar.

---

