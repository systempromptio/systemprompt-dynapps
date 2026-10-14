## NIVEL 3 — SQL Directo (requiere acceso SSH + psql)

```
+==================================================================+
|  ADVERTENCIA CRITICA                                              |
|                                                                    |
|  1. SQL directo NO ejecuta triggers del ORM de Odoo               |
|  2. No hay recompute de campos calculados                          |
|  3. No hay ondelete cascade del ORM (solo FK de PostgreSQL)        |
|  4. No hay tracking, no hay mensajes en chatter                    |
|  5. Puede dejar la BD en estado inconsistente                      |
|  6. SIEMPRE dentro de BEGIN; ... ROLLBACK; primero para verificar  |
|  7. Requiere acceso SSH al servidor + credenciales psql            |
|  8. HACER BACKUP ANTES de cualquier operacion                      |
|  9. stock.quant NO SE ACTUALIZA — requiere recalculo posterior     |
| 10. Cadena de hash fiscal se ROMPE si se borra un move hasheado    |
+==================================================================+
```

