## Resumen de Decision

```
¿Se crearon registros de test que consumen secuencias?
  │
  ├─ NO → No se requiere restauracion
  │
  └─ SI → ¿Hay registros REALES intercalados?
           │
           ├─ SI → NO restaurar. Aceptar huecos. Documentar.
           │
           └─ NO → ¿Secuencias fiscales afectadas (SII/TicketBAI/VeriFactu)?
                    │
                    ├─ SI → Restaurar con maxima precaucion.
                    │       Verificar cadena de hashes.
                    │       Confirmar que ningun dato fue comunicado a hacienda.
                    │
                    └─ NO → Restaurar via ORM (Estrategia A).
                            Verificar con snapshot.py verify.
```
