## Reglas de Seguridad

1. **NUNCA** ejecutar revert en produccion sin backup verificado
2. **NUNCA** ejecutar revert sin dry-run aprobado por el consultor
3. **SIEMPRE** verificar pre_state coincide tras reversion
4. Si revert falla en un registro → **PARAR** inmediatamente, no continuar
5. **NUNCA** forzar borrado (unlink) de registros con inalterable_hash
6. Si un modelo no puede revertirse limpiamente → informar y proponer alternativa (neutralizacion)
7. Los registros cascade (generados automaticamente) se revierten ANTES que los padres
8. **SIEMPRE** usar prefijo `[TEST-YYYYMMDD-HHMM]` en todos los registros de test
9. **SIEMPRE** restaurar integraciones desactivadas, incluso si la limpieza falla
10. **SIEMPRE** capturar pre-state snapshot ANTES de ejecutar cualquier flujo
11. **SIEMPRE** ejecutar la verificacion post-revert completa (12 puntos del checklist)
12. **NUNCA** escalar a Tier 3 (SQL) sin haber agotado Tier 1 y Tier 2
13. **NUNCA** borrar account.full.reconcile directamente — borrar sus partials (los fulls se limpian solos)
14. **NUNCA** restaurar secuencias de facturas si el diario tiene hash activo

---

