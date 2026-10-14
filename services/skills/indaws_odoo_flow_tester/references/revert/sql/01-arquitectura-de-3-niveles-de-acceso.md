## Arquitectura de 3 Niveles de Acceso

```
+--------------------------------------------------------------------+
|  NIVEL 1 — ORM via odoo-pilot (PREFERIDO)                          |
|  Scripts: search_records.sh, execute_method.sh, delete_record.sh   |
|  Seguro, respeta constraints y triggers del ORM                    |
+--------------------------------------------------------------------+
         |  Si falla por constraint/estado irreversible
         v
+--------------------------------------------------------------------+
|  NIVEL 2 — ORM + execute_method avanzado                           |
|  Metodos internos, contextos especiales, bypass de restricciones   |
|  Requiere conocer metodos no documentados del modelo               |
+--------------------------------------------------------------------+
         |  Si falla por hash fiscal, estado corrupto, etc.
         v
+--------------------------------------------------------------------+
|  NIVEL 3 — SQL directo (requiere SSH + psql)                       |
|  DELETE/UPDATE directo en PostgreSQL                                |
|  NO ejecuta triggers ORM — ULTIMO RECURSO ABSOLUTO                 |
+--------------------------------------------------------------------+
```

---

